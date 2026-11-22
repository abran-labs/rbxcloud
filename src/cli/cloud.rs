use std::collections::BTreeMap;

use clap::Args;
use serde::Deserialize;

const MANIFEST: &str = include_str!("../../openapi/coverage.json");

#[derive(Debug, Deserialize)]
struct Manifest {
    operations: Vec<Operation>,
}

#[derive(Debug, Deserialize)]
struct Operation {
    family: String,
    status: String,
}

#[derive(Debug, Default)]
struct Counts {
    implemented: usize,
    existing: usize,
    defective: usize,
    pending: usize,
}

impl Counts {
    fn total(&self) -> usize {
        self.implemented + self.existing + self.defective + self.pending
    }

    fn add(&mut self, status: &str) -> anyhow::Result<()> {
        match status {
            "implemented" => self.implemented += 1,
            "existing" => self.existing += 1,
            "defective" => self.defective += 1,
            "pending" => self.pending += 1,
            other => anyhow::bail!("unknown coverage status `{other}`"),
        }
        Ok(())
    }
}

/// Show Open Cloud API coverage status by family (read-only, offline)
#[derive(Debug, Args)]
pub struct Coverage {}

impl Coverage {
    pub fn run(self) -> anyhow::Result<Option<String>> {
        render(MANIFEST).map(Some)
    }
}

fn render(manifest: &str) -> anyhow::Result<String> {
    let manifest: Manifest = serde_json::from_str(manifest)?;
    let mut families: BTreeMap<&str, Counts> = BTreeMap::new();
    let mut all = Counts::default();
    for op in &manifest.operations {
        families.entry(&op.family).or_default().add(&op.status)?;
        all.add(&op.status)?;
    }
    let width = families
        .keys()
        .map(|f| f.len())
        .max()
        .unwrap_or(0)
        .max("family".len());

    let mut lines = vec![row(
        width,
        "family",
        "implemented",
        "existing",
        "defective",
        "pending",
        "total",
    )];
    let mut push = |name: &str, c: &Counts| {
        lines.push(row(
            width,
            name,
            &c.implemented.to_string(),
            &c.existing.to_string(),
            &c.defective.to_string(),
            &c.pending.to_string(),
            &c.total().to_string(),
        ));
    };
    for (name, c) in &families {
        push(name, c);
    }
    push("total", &all);
    Ok(lines.join("\n"))
}

fn row(w: usize, a: &str, b: &str, c: &str, d: &str, e: &str, f: &str) -> String {
    format!("{a:<w$}  {b:>11}  {c:>8}  {d:>9}  {e:>7}  {f:>5}")
}
