"""Typed Rust model generation from the resolved official schemas."""
from .naming import pascal, snake


class Models:
    def __init__(self, contract):
        self.contract = contract
        self.schemas = contract.spec["components"]["schemas"]
        self.definitions = {}
        self.enums = {}
        self.request_enums = set()
        self.anonymous = 0

    def reference(self, schema):
        if not schema:
            return "serde_json::Value"
        if "$ref" in schema:
            name = schema["$ref"].split("/")[-1]
            if schema.get("x-request-model"):
                schema = dict(self.schemas[name])
                schema["x-request-model"] = True
                schema["x-generated-name"] = name + "Input"
                return self.object(name + "Input", schema)
            self.collect(name)
            return pascal(name)
        if schema.get("enum") and schema.get("type") == "string" and schema.get("x-generated-name"):
            return self.inline_enum(schema["x-generated-name"], schema["enum"], bool(schema.get("x-request-enum")))
        if "allOf" in schema:
            variants = [self.reference(self.contract.resolve(item)) for item in schema["allOf"]]
            if len(variants) == 1:
                return variants[0]
            return "serde_json::Value"
        if "anyOf" in schema or "oneOf" in schema:
            return "serde_json::Value"
        kind = schema.get("type")
        if kind == "string" or kind is None:
            if kind is None and not schema.get("properties"):
                return "serde_json::Value"
            if "enum" in schema:
                owner = schema.get("x-generated-name")
                if owner:
                    return self.inline_enum(owner, schema["enum"], bool(schema.get("x-request-enum")))
            if schema.get("format") == "binary":
                return "std::path::PathBuf"
            return "String"
        if kind == "boolean":
            return "bool"
        if kind == "integer":
            return {"int32": "i32", "int64": "i64", "uint32": "u32", "uint64": "u64"}.get(schema.get("format"), "i64")
        if kind == "number":
            if schema.get("x-integer"):
                return "i64"
            return "f32" if schema.get("format") == "float" else "f64"
        if kind == "array":
            return f"Vec<{self.reference(schema.get('items', {}))}>"
        if kind == "object" or "properties" in schema:
            if not schema.get("properties") and (schema.get("additionalProperties") or schema.get("description", "").lower().startswith("an arbitrary")):
                return "serde_json::Value"
            if not schema.get("properties"):
                owner = schema.get("x-generated-name")
                if owner:
                    self.definitions.setdefault(pascal(owner), "#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]\npub struct " + pascal(owner) + " {}\n")
                    return pascal(owner)
                return "std::collections::BTreeMap<String, serde_json::Value>"
            owner = schema.get("x-generated-name")
            if not owner:
                return "serde_json::Value"
            return self.object(owner, schema)
        raise ValueError(f"unsupported schema: {schema}")

    def inline_enum(self, owner, values, request_only=False):
        name = pascal(owner)
        if request_only:
            self.request_enums.add(name)
        if name not in self.enums:
            self.enums[name] = tuple(values)
        elif self.enums[name] != tuple(values):
            raise ValueError(f"conflicting enum {name}")
        return name

    def collect(self, name):
        if name in self.definitions or name in self.enums:
            return
        schema = self.schemas[name]
        if schema.get("type") == "string" and "enum" in schema:
            self.inline_enum(name, schema["enum"])
            return
        if schema.get("type") == "object" or "properties" in schema:
            self.object(name, schema)
            return
        if "allOf" in schema and len(schema["allOf"]) == 1:
            self.reference(schema["allOf"][0])
            return
        self.definitions[pascal(name)] = f"pub type {pascal(name)} = serde_json::Value;\n"

    def object(self, name, schema):
        rust_name = pascal(name)
        if rust_name in self.definitions:
            return rust_name
        self.definitions[rust_name] = None
        required = set(schema.get("required", []))
        fields = []
        for wire, original in schema.get("properties", {}).items():
            if schema.get("x-request-model") and original.get("readOnly"):
                continue
            schema_copy = dict(original)
            schema_copy["x-generated-name"] = rust_name + pascal(wire)
            typ = self.reference(schema_copy)
            optional = wire not in required
            if optional:
                typ = f"Option<{typ}>"
            field = snake(wire)
            rename = "" if wire == field else f'    #[serde(rename = "{wire}")]\n'
            attr = ""
            if optional:
                attr = '    #[serde(default, skip_serializing_if = "Option::is_none")]\n'
            fields.append(f"{rename}{attr}    pub {field}: {typ},")
        body = "#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]\n"
        body += f"pub struct {rust_name} {{\n" + "\n".join(fields) + "\n}\n"
        self.definitions[rust_name] = body
        return rust_name

    def render(self):
        lines = ["// @generated by scripts/gen_typed_api.py. Do not edit.",  ""]
        for name, values in sorted(self.enums.items()):
            lines.append("#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, serde::Serialize, serde::Deserialize)]")
            lines.append(f"pub enum {pascal(name)} {{")
            for value in values:
                lines.append(f'    #[serde(rename = "{value}")]')
                lines.append(f"    {pascal(str(value))},")
            if name not in self.request_enums and "Unknown" not in values:
                lines.append("    #[serde(other)]\n    #[value(skip)]\n    Unknown,\n")
            lines.append("}\n")
        for name, values in sorted(self.enums.items()):
            arms = [f'            Self::{pascal(str(value))} => "{value}",' for value in values]
            if name not in self.request_enums and "Unknown" not in values:
                arms.append('            Self::Unknown => "UNKNOWN",')
            lines.append(f"impl std::fmt::Display for {pascal(name)} {{\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{\n        f.write_str(match self {{\n" + "\n".join(arms) + "\n        })\n    }\n}\n")
        for name, body in sorted(self.definitions.items()):
            if body is None:
                raise ValueError(f"cyclic model requires indirection: {name}")
            lines.append(body)
        return "\n".join(lines)
