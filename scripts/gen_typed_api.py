#!/usr/bin/env python3
"""Generate explicit typed Open Cloud models and operation/CLI definitions."""
from pathlib import Path
import argparse
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from tools.typed_api.emit import emit_cli, emit_models, emit_operations
from tools.typed_api.models import Models
from tools.typed_api.spec import Contract


def generate():
    contract = Contract(ROOT)
    operations = contract.operations({"data_stores", "ordered_data_stores", "memory_stores"})
    models = Models(contract)
    model_source = emit_models(contract, operations, models)
    operation_source = emit_operations(contract, operations, models)
    cli_source, _ = emit_cli(contract, operations, models)
    sources = {
        ROOT / "src/rbx/generated/models.rs": model_source,
        ROOT / "src/rbx/generated/storage.rs": operation_source,
        ROOT / "src/cli/generated/storage.rs": cli_source,
    }
    for path in list(sources):
        formatted = subprocess.run(["rustfmt", "--edition", "2021"], input=sources[path], text=True, capture_output=True, check=True)
        sources[path] = formatted.stdout
    return sources


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    files = generate()
    stale = []
    for path, content in files.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        old = path.read_text() if path.exists() else None
        if old != content:
            if args.check:
                stale.append(str(path.relative_to(ROOT)))
            else:
                path.write_text(content)
    if stale:
        print("Generated files are stale:", *stale, sep="\n", file=sys.stderr)
        raise SystemExit(1)
    print(f"Generated {len(files)} typed API source files")


if __name__ == "__main__":
    main()
