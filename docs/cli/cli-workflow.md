# Workflows

Run native TOML workflows with typed inputs and prior JSON results. The nearest
ancestor containing `.rbxcloud/workflows` is the project root; otherwise the
current directory is the root. Personal workflows live under
`${XDG_CONFIG_HOME:-~/.config}/rbxcloud/workflows`. Project names shadow personal
names. Personal workflows still use the current project for asset paths and
state.

```sh
rbxcloud workflow init icons --template upload-images
rbxcloud workflow list
rbxcloud workflow show icons
rbxcloud workflow validate icons
rbxcloud workflow run icons --input creator_id=1234 --dry-run
rbxcloud workflow run icons --input creator_id=1234 --input 'paths=["icons/**/*.png"]'
```

`init` never overwrites; `--personal` selects personal storage.
`--template blank` is the default. `validate` checks schema, references and
allowlisted argument shapes. `run --dry-run` binds inputs, checks locally
resolvable commands and paths, and reports steps that need unavailable prior
outputs as deferred. It loads no credentials, writes no state, sends no requests,
and doesn't invent remote responses. Execution repeats those checks against
actual inputs and outputs. Steps execute in order; a failure stops subsequent
steps but does not roll back earlier writes.

## Syntax

```toml
version = 1

[inputs.asset_id]
type = "integer"

[inputs.versions]
type = "integers"
default = [1, 2]

[[steps]]
id = "versions"
command = ["asset-tools", "get-asset-version"]
args = ["--asset-id", "${inputs.asset_id}", "--version-number", "${each}"]
for_each = "${inputs.versions}"
retries = 2

[[steps]]
id = "published"
command = ["asset-tools", "list-asset-versions"]
args = ["--asset-id", "${inputs.asset_id}"]
when = "${steps.versions.output.0.published}"
```

Inputs: `string`, `integer` (signed 64-bit), `boolean`, or `strings`,
`integers`, `booleans`. Omit `default` for required inputs; use JSON arrays
for list overrides. References occupy a whole argument. Use `${inputs.name}`,
`${steps.ID.output.field.0}`, or `${each}` / `${each.field}` inside a loop.
`for_each` requires an array and returns an array of executed iteration results.
`when` is a TOML boolean or a reference resolving to a JSON boolean.
Missing prior JSON fields fail rather than silently becoming defaults.

## Allowed targets

Command names, flag order, and fixed tokens are literal:

- `asset-tools get-asset-version`: `--asset-id ID --version-number NUMBER`
- `asset-tools list-asset-versions`: `--asset-id ID`
- `asset-tools list-asset-quotas`: `--user-id ID`
- `assets upload-batch`: `--creator-type user|group --creator-id ID --asset-type image -- PATH...`

An image step can expand an array of paths at `PATH...`. Paths must stay
inside the project. Its manifest is fixed to `.rbxcloud/upload-batch.json`.
Workflow uploads (including upload dry runs) require Linux with `openat2`
available (kernel 5.6 or newer, not blocked by sandbox policy). Project-relative
asset paths cannot traverse symlinks or mounts; hard-linked files are rejected.
The `.rbxcloud` state directory must be owned by the current user and not writable
by group or others. Use a trusted project and keep its upload state under your
control. Read-only workflow steps and direct `assets upload-batch` do not require
`openat2`.
Read steps can request up to three bounded retries after the initial attempt.
The upload step never retries a create. No shell, raw HTTP, nested workflows,
script execution or publication, downloads, custom manifest destinations,
or skill installs. Never put API keys in TOML or `--input`; workflow commands
load credentials internally via [API Key](cli-api-key.md).
