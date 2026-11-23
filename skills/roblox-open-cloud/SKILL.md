---
name: roblox-open-cloud
description: >-
  Use when working with Roblox Open Cloud through rbxcloud, including API key
  setup, typed command discovery, batch asset uploads, or repeatable workflows.
  Skip Studio-only editing and operations using Roblox login cookies.
metadata:
  version: "@RBXCLOUD_VERSION@"
---

# Roblox Open Cloud

Use typed `rbxcloud` commands for documented Roblox Open Cloud operations. Run
`rbxcloud --version`, `rbxcloud coverage`, and `rbxcloud <family> <command> --help`
before guessing an option. This skill ships with its matching CLI build; reinstall
after upgrading.

## Credentials

Create a personal Open Cloud API key at the official
[Roblox Credentials dashboard](https://create.roblox.com/credentials). Grant only
the required operations and resources; set expiration and IP restrictions where
practical. For group assets, your Roblox account must have permission to create
the group's assets, and the same key must have the necessary Assets permissions.
Creator type and ID select ownership, not authentication.

Inject `ROBLOX_OPEN_CLOUD_API_KEY` via a secret manager. Typed commands also accept
`RBXCLOUD_API_KEY`, or read either name from the current user's regular,
owner-only `~/.config/opencode/secrets/roblox-open-cloud.env` (`chmod 600` on Unix).
Never put the key in command arguments, workflow inputs, project files, chat,
or logs. Some legacy commands only read `RBXCLOUD_API_KEY`; check `--help`.
`rbxcloud auth status` checks local loading only, not server permissions.

## Batch asset creation

```sh
rbxcloud assets upload-batch --asset-type image --creator-type group --creator-id 1234 --dry-run 'icons/**/*.png'
```

Review creator, source paths and hashes. Confirm remote creation before removing
`--dry-run`. Keep the same manifest on every resume. The manifest tracks
creator, asset type and content; identical content under the same owner and type
shares an upload. `accepted` entries resume by operation ID; `unconfirmed`
entries never resend. If no ID was returned, inspect Creator Dashboard before
deciding how to proceed; do not delete the manifest to retry blindly.

## Workflows

Use `rbxcloud workflow init icons --template upload-images`, inspect with
`workflow show`, and run `workflow validate` and `workflow run ... --dry-run`
before executing. Workflow steps use a small typed command allowlist. They do
not run shell, arbitrary HTTP, Luau, or nested workflows. Keep credentials out
of workflow TOML. Retry reads only; never replay creates.
