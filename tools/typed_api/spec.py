"""Resolve official Open Cloud operation contracts without endpoint passthrough."""
from dataclasses import dataclass
from pathlib import Path
import json
import re


@dataclass(frozen=True)
class Parameter:
    name: str
    location: str
    schema: dict
    required: bool


@dataclass(frozen=True)
class Operation:
    id: str
    method: str
    path: str
    server: str
    family: str
    cli: str
    summary: str
    parameters: tuple[Parameter, ...]
    body: dict | None
    response: dict | None


class Contract:
    def __init__(self, root: Path):
        self.root = root
        self.spec = json.loads((root / "openapi/roblox.json").read_text())
        self.manifest = json.loads((root / "openapi/coverage.json").read_text())

    def resolve(self, value):
        seen = set()
        while isinstance(value, dict) and "$ref" in value:
            ref = value["$ref"]
            if not ref.startswith("#/") or ref in seen:
                raise ValueError(f"invalid or cyclic reference: {ref}")
            seen.add(ref)
            target = self.spec
            for segment in ref[2:].split("/"):
                target = target[segment.replace("~1", "/").replace("~0", "~")]
            value = target
        return value

    def operations(self, families):
        for name in ["IncrementDataStoreEntryRequest", "IncrementOrderedDataStoreEntryRequest"]:
            self.spec["components"]["schemas"][name]["properties"]["amount"]["x-integer"] = True
        self.spec["components"]["schemas"]["OrderedDataStoreEntry"]["properties"]["value"]["x-integer"] = True
        result = []
        for record in self.manifest["operations"]:
            if record["family"] not in families:
                continue
            path_item = self.spec["paths"][record["path"]]
            raw = self.resolve(path_item[record["method"].lower()])
            parameters = []
            for parameter in path_item.get("parameters", []) + raw.get("parameters", []):
                parameter = self.resolve(parameter)
                parameters.append(Parameter(parameter["name"], parameter["in"], self.resolve(parameter.get("schema", {})), bool(parameter.get("required"))))
            body = raw.get("requestBody")
            if body:
                body = self.resolve(body)
            response = None
            for status, candidate in sorted(raw.get("responses", {}).items()):
                if status.startswith("2"):
                    candidate = self.resolve(candidate)
                    for media in candidate.get("content", {}).values():
                        if "schema" in media:
                            response = media["schema"]
                            break
                    if response is not None:
                        break
            path_names = set(re.findall(r"{([^}]+)}", record["path"]))
            supplied = {p.name for p in parameters if p.location == "path"}
            if path_names != supplied:
                raise ValueError(f"path parameter mismatch for {record['id']}: {path_names} != {supplied}")
            result.append(Operation(record["id"], record["method"], record["path"], record["server"], record["family"], record["cli"], raw.get("summary", record["id"]), tuple(parameters), body, response))
        ids = [operation.id for operation in result]
        if len(ids) != len(set(ids)):
            raise ValueError("duplicate operation IDs")
        return result
