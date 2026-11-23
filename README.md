![rbxcloud banner showing a cloud emblem beside the product name](.github/assets/rbxcloud-banner.svg)

# rbxcloud

[![CI](https://github.com/abran-labs/rbxcloud/actions/workflows/ci.yaml/badge.svg)](https://github.com/abran-labs/rbxcloud/actions/workflows/ci.yaml)

**Typed access to every API-key Roblox Open Cloud operation.**

## Install

```sh
cargo install --git https://github.com/abran-labs/rbxcloud --branch main
```

## Configure

```sh
export RBXCLOUD_API_KEY="your-api-key"
```

Create API keys on the [Roblox Credentials page](https://create.roblox.com/credentials).

## Commands

| Command | Use |
| -- | -- |
| `rbxcloud coverage` | Coverage summary by API family |
| `rbxcloud assets upload-batch --help` | Preview and resume new asset uploads |
| `rbxcloud auth status` | Check local API key setup without exposing it |
| `rbxcloud skill install` | Install the matching Roblox Open Cloud agent skill |
| `rbxcloud workflow --help` | Run allowlisted TOML workflows |
| `rbxcloud datastore-v2 --help` | Data Stores, scopes, snapshots, revisions |
| `rbxcloud memory-store --help` | Memory Store queues and sorted maps |
| `rbxcloud campaign-management --help` | Advertising campaigns and estimates |
| `rbxcloud asset-multipart-start --help` | Resumable multipart asset uploads |
| `rbxcloud analytics-query --help` | Analytics metrics and benchmarks |

See the [batch upload](docs/cli/cli-assets.md#upload-batch), [agent skill](docs/cli/cli-skill.md), and [workflow](docs/cli/cli-workflow.md) guides.

## Supported APIs

397 of 397 Open Cloud operations with API-key authentication are covered by typed SDK methods and explicit commands. Operations without API-key authentication are recorded as exclusions, not silent gaps.

| Scope | Covered |
| -- | -- |
| Assets, Data Stores, Memory Stores, Messaging | Yes |
| Groups, Universes, Places, Users, Inventory | Yes |
| Advertising, Analytics, Matchmaking, Localization | Yes |
| Economy, Creator Store, Collectibles, Virtual Events | Yes |
| Configs, Experiments, Server Management, Generative AI | Yes |

See [docs/COVERAGE.md](docs/COVERAGE.md) for the per-operation ledger.

## Rust SDK

```toml
[dependencies]
rbxcloud = { git = "https://github.com/abran-labs/rbxcloud", branch = "main" }
```
