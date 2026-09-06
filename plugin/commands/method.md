---
description: Print peira's anti-summarization method (the seven doors), version-stamped
argument-hint: "[method-name]"
---

Run the peira binary to print a method document, version-stamped so any copy made from it
records which peira version it matches.

- If `$ARGUMENTS` is empty, run `peira method` to list the available method documents, then run `peira method anti-summarization` to show the default one.
- Otherwise run `peira method $ARGUMENTS`.

If the command is not found, tell the user the peira binary is not installed and point them at
`cargo install --git https://github.com/SecurityRonin/peira peira-cli` (or their platform package);
this plugin ships the skill and the MCP wiring, not the binary. Then present the seven doors and
offer to run the anti-summarization pass over whatever they are distilling.
