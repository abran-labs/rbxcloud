# Agent Skill

`roblox-open-cloud` ships in the binary and matches `rbxcloud --version`.
It explains credentials, CLI discovery, uploads, and workflows.

```sh
rbxcloud skill install             # ~/.agents/skills
rbxcloud skill install --claude    # ~/.claude/skills
rbxcloud skill install --opencode  # ${XDG_CONFIG_HOME:-~/.config}/opencode/skills
rbxcloud skill install --dir ./my-skills
rbxcloud skill install --print    # stdout only
rbxcloud skill status --opencode
```

Pass the same destination to `status`. Reinstall after upgrading. An identical
file stays untouched; differing files require explicit `--force`.
