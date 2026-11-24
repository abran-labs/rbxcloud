use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, ensure, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::target::{Action, Target};

const MAX_STEPS: usize = 64;
const MAX_LIST: usize = 128;
const MAX_TOKEN: usize = 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Document {
    version: u8,
    #[serde(default)]
    inputs: BTreeMap<String, Input>,
    #[serde(default)]
    pub steps: Vec<Step>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    #[serde(rename = "type")]
    kind: Kind,
    default: Option<Value>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Kind {
    String,
    Integer,
    Boolean,
    Strings,
    Integers,
    Booleans,
}

impl Kind {
    fn accepts(self, value: &Value) -> bool {
        match self {
            Self::String => value.as_str().is_some_and(valid_text),
            Self::Integer => value.as_i64().is_some(),
            Self::Boolean => value.is_boolean(),
            Self::Strings | Self::Integers | Self::Booleans => {
                let item = match self {
                    Self::Strings => Self::String,
                    Self::Integers => Self::Integer,
                    _ => Self::Boolean,
                };
                value
                    .as_array()
                    .is_some_and(|xs| xs.len() <= MAX_LIST && xs.iter().all(|x| item.accepts(x)))
            }
        }
    }

    fn parse(self, value: &str) -> Result<Value> {
        ensure!(
            value.len() <= MAX_LIST * MAX_TOKEN,
            "workflow input is too long"
        );
        match self {
            Self::String => Ok(Value::String(value.to_owned())),
            _ => serde_json::from_str(value).map_err(|_| anyhow::anyhow!("invalid typed input")),
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Step {
    pub id: String,
    pub command: Vec<String>,
    #[serde(default)]
    args: Vec<String>,
    pub for_each: Option<String>,
    when: Option<When>,
    #[serde(default)]
    pub retries: u8,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum When {
    Bool(bool),
    Reference(String),
}

pub(super) fn identifier(name: &str) -> bool {
    name.len() <= 64
        && name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-'))
}

fn valid_text(s: &str) -> bool {
    s.len() <= MAX_TOKEN && !s.chars().any(char::is_control)
}

impl Document {
    pub fn parse(source: &str) -> Result<Self> {
        let document: Self = toml::from_str(source).map_err(|e| {
            let line = e.span().map(|range| {
                source.as_bytes()[..range.start.min(source.len())]
                    .iter()
                    .filter(|b| **b == b'\n')
                    .count()
                    + 1
            });
            anyhow::anyhow!(
                "invalid workflow TOML or schema{}",
                line.map(|n| format!(" at line {n}")).unwrap_or_default()
            )
        })?;
        ensure!(document.version == 1, "unsupported workflow version");
        ensure!(
            document.steps.len() <= MAX_STEPS && document.inputs.len() <= MAX_STEPS,
            "too many steps or inputs"
        );
        for (name, input) in &document.inputs {
            ensure!(identifier(name), "invalid input name");
            ensure!(
                input.default.as_ref().is_none_or(|v| input.kind.accepts(v)),
                "invalid workflow input default"
            );
        }
        let mut prior = BTreeSet::new();
        for step in &document.steps {
            ensure!(
                identifier(&step.id) && !prior.contains(step.id.as_str()),
                "invalid or duplicate step id"
            );
            step.validate(&document.inputs, &prior)
                .with_context(|| format!("step {}", step.id))?;
            prior.insert(step.id.as_str());
        }
        Ok(document)
    }

    pub fn bind(&self, overrides: &[String]) -> Result<BTreeMap<String, Value>> {
        let mut bindings = BTreeMap::new();
        for binding in overrides {
            let (name, value) = binding
                .split_once('=')
                .context("expected --input NAME=VALUE")?;
            let declaration = self.inputs.get(name).context("unknown workflow input")?;
            ensure!(!bindings.contains_key(name), "workflow input bound twice");
            let value = declaration.kind.parse(value)?;
            ensure!(
                declaration.kind.accepts(&value),
                "workflow input has the wrong type or size"
            );
            bindings.insert(name.to_owned(), value);
        }
        for (name, declaration) in &self.inputs {
            if !bindings.contains_key(name) {
                let default = declaration
                    .default
                    .clone()
                    .with_context(|| format!("required input {name} is missing"))?;
                bindings.insert(name.clone(), default);
            }
        }
        Ok(bindings)
    }
}

#[derive(Clone, Copy)]
pub(super) struct Scope<'a> {
    pub inputs: &'a BTreeMap<String, Value>,
    pub outputs: &'a BTreeMap<String, Option<Value>>,
    pub each: Option<&'a Value>,
}

impl<'a> Scope<'a> {
    pub fn with_each(self, each: Option<&'a Value>) -> Self {
        Self { each, ..self }
    }
}

enum Reference<'a> {
    Input(&'a str),
    Output(&'a str, Vec<&'a str>),
    Each(Vec<&'a str>),
}

fn valid_json_path(parts: &[&str]) -> bool {
    parts.len() <= 16
        && parts.iter().all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
        })
}

fn descend<'a>(mut value: &'a Value, path: &[&str]) -> Result<&'a Value> {
    for component in path {
        value = match value {
            Value::Array(xs) => component.parse::<usize>().ok().and_then(|i| xs.get(i)),
            Value::Object(object) => object.get(*component),
            _ => None,
        }
        .context("JSON value does not contain referenced field or index")?;
    }
    Ok(value)
}

impl<'a> Reference<'a> {
    fn parse(token: &'a str) -> Result<Option<Self>> {
        ensure!(
            valid_text(token),
            "argument is too long or contains control characters"
        );
        if !token.contains("${") {
            return Ok(None);
        }
        let expr = token
            .strip_prefix("${")
            .and_then(|s| s.strip_suffix('}'))
            .context("references must occupy one full argument")?;
        let parts: Vec<_> = expr.split('.').collect();
        let reference = match parts.as_slice() {
            ["each", rest @ ..] if valid_json_path(rest) => Self::Each(rest.to_vec()),
            ["inputs", name] if identifier(name) => Self::Input(name),
            ["steps", step, "output", rest @ ..] if identifier(step) && valid_json_path(rest) => {
                Self::Output(step, rest.to_vec())
            }
            _ => bail!("invalid workflow reference"),
        };
        Ok(Some(reference))
    }

    fn validate(
        &self,
        inputs: &BTreeMap<String, Input>,
        prior: &BTreeSet<&str>,
        has_each: bool,
    ) -> Result<()> {
        match self {
            Self::Input(name) => {
                ensure!(inputs.contains_key(*name), "reference to undeclared input")
            }
            Self::Output(step, _) => {
                ensure!(prior.contains(step), "reference must use a prior step")
            }
            Self::Each(_) => ensure!(has_each, "each is only available inside for_each"),
        }
        Ok(())
    }

    fn resolve(&self, scope: Scope<'_>) -> Result<Option<Value>> {
        let value = match self {
            Self::Input(name) => Some(scope.inputs.get(*name).context("unbound workflow input")?),
            Self::Each(path) => Some(descend(
                scope.each.context("missing iteration value")?,
                path,
            )?),
            Self::Output(step, path) => {
                let Some(value) = scope
                    .outputs
                    .get(*step)
                    .context("missing prior step")?
                    .as_ref()
                else {
                    return Ok(None);
                };
                Some(descend(value, path)?)
            }
        };
        Ok(value.cloned())
    }
}

fn required_reference(value: &str) -> Result<Reference<'_>> {
    Reference::parse(value)?.context("expected a workflow reference")
}

impl Step {
    fn validate(&self, inputs: &BTreeMap<String, Input>, prior: &BTreeSet<&str>) -> Result<()> {
        let target = Target::select(&self.command)?;
        target.layout(&self.args)?;
        ensure!(
            self.retries <= 3 && (self.retries == 0 || target.is_read()),
            "only read steps may retry (at most three retries)"
        );
        if let Some(expression) = &self.for_each {
            let reference = required_reference(expression)?;
            reference.validate(inputs, prior, false)?;
            if let Reference::Input(name) = reference {
                ensure!(
                    matches!(
                        inputs[name].kind,
                        Kind::Strings | Kind::Integers | Kind::Booleans
                    ),
                    "for_each input must be an array type"
                );
            }
        }
        if let Some(When::Reference(expression)) = &self.when {
            let reference = required_reference(expression)?;
            reference.validate(inputs, prior, self.for_each.is_some())?;
            if let Reference::Input(name) = reference {
                ensure!(
                    matches!(inputs[name].kind, Kind::Boolean),
                    "when input must be boolean"
                );
            }
        }
        for (index, token) in self.args.iter().enumerate() {
            if let Some(reference) = Reference::parse(token)? {
                reference.validate(inputs, prior, self.for_each.is_some())?;
            } else {
                target.literal(index, token)?;
            }
        }
        Ok(())
    }

    pub fn iterations(&self, scope: Scope<'_>) -> Result<Option<Vec<Option<Value>>>> {
        let Some(expression) = &self.for_each else {
            return Ok(Some(vec![None]));
        };
        let Some(value) = required_reference(expression)?.resolve(scope)? else {
            return Ok(None);
        };
        let Value::Array(items) = value else {
            bail!("for_each must resolve to an array")
        };
        ensure!(
            items.len() <= MAX_LIST,
            "for_each exceeded its iteration limit"
        );
        Ok(Some(items.into_iter().map(Some).collect()))
    }

    pub fn should_run(&self, scope: Scope<'_>) -> Result<Option<bool>> {
        match &self.when {
            None => Ok(Some(true)),
            Some(When::Bool(value)) => Ok(Some(*value)),
            Some(When::Reference(expression)) => required_reference(expression)?
                .resolve(scope)?
                .map(|value| value.as_bool().context("when must resolve to a boolean"))
                .transpose(),
        }
    }

    pub fn render(&self, scope: Scope<'_>) -> Result<Option<Action>> {
        let target = Target::select(&self.command)?;
        let mut args = Vec::new();
        let mut deferred = false;
        for (index, token) in self.args.iter().enumerate() {
            let Some(reference) = Reference::parse(token)? else {
                args.push(token.clone());
                continue;
            };
            let Some(value) = reference.resolve(scope)? else {
                deferred = true;
                continue;
            };
            if target.path_slot(index) {
                if let Value::Array(values) = value {
                    ensure!(values.len() <= MAX_LIST, "path list exceeds its size limit");
                    for value in values {
                        let value = value.as_str().context("upload paths must be strings")?;
                        target.literal(index, value)?;
                        args.push(value.to_owned());
                    }
                    continue;
                }
            }
            let scalar = match value {
                Value::String(value) => value,
                Value::Number(number) if number.is_i64() || number.is_u64() => number.to_string(),
                _ => bail!("command argument must be a string or integer"),
            };
            target.literal(index, &scalar)?;
            args.push(scalar);
        }
        if deferred {
            return Ok(None);
        }
        target.build(&self.command, args).map(Some)
    }
}
