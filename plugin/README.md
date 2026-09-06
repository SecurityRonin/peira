# peira — Claude Code plugin

Brings the [peira](https://github.com/SecurityRonin/peira) critical-thinking checker into
Claude Code: the **anti-summarization pass** as a skill, a command that prints peira's method
documents version-stamped, and the **peira MCP server** that refuses to promote a claim you
have not examined.

## What it bundles

| Component | What it does |
|---|---|
| `skills/anti-summarization` | The seven doors for distilling a source without letting compression smooth away its faults. Triggers on summarise / distil / audit / decompress. |
| `commands/method.md` | `/peira:method [name]` — prints a peira method document, version-stamped for regeneration. |
| `.mcp.json` | Registers the `peira` MCP server, exposing `check_prose` (no vault needed) plus vault-backed `examine`, `gates`, `freeze`, `verify`. |

## Install — the binary first, then the plugin

This plugin ships the skill and the MCP **wiring**, not the peira binaries. Install those first:

```bash
cargo install --git https://github.com/SecurityRonin/peira peira-cli   # the `peira` CLI
cargo install --git https://github.com/SecurityRonin/peira peira-mcp   # the MCP server
```

(or your platform package once published). Then add the plugin:

```
/plugin marketplace add SecurityRonin/peira
/plugin install peira@peira
```

The plugin's `.mcp.json` launches `peira-mcp` from your `PATH`; if the binary is absent, the
MCP server simply will not start — install it and reload.

## Why a plugin and not the OS package

A Claude Code skill lives in the per-user plugin space, not in `/usr/local/bin`, and is managed
by Claude Code — so it cannot ship inside an MSI/deb/brew formula. The binary reaches the machine
through those channels; this plugin reaches Claude Code through the marketplace and *depends on*
the binary being installed.

---

[Privacy](https://securityronin.github.io/peira/privacy) · [Terms](https://securityronin.github.io/peira/terms) · © Security Ronin Ltd
