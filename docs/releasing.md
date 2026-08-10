# Releasing

Two pipelines, deliberately decoupled. Nothing here is ever hand-cut.

| | Owns | Trigger | Config |
|---|---|---|---|
| **release-plz** | `peira-core`, `peira-lens`, `peira-court`, `peira-index` | push to `main` | `release-plz.toml`, `.github/workflows/release-plz.yml` |
| **release.yml** | the `peira` binary + the `peira-cli` crate | a signed `v[0-9]*` tag | `.github/workflows/release.yml` |

## Libraries — release-plz

Conventional-commit types drive the bump: `feat`→minor, `fix`→patch, breaking→major.
`chore`/`ci`/`test`/`style`/`build` ride along without cutting a release.

release-plz opens a **release PR** that edits versions and writes the `CHANGELOG`s.
**Merging that PR publishes.** The PR is not code review — it is the one reviewable
checkpoint before an irreversible crates.io publish, since versions yank but never
delete and names are claimed forever.

- **Merge it with a MERGE commit, never squash.** Squashing rewrites the version-bump
  commit release-plz keys on.
- **A security fix uses `fix(security):`**, never a bare `security:` type —
  `release_commits` does not match `security:`, so the fix would sit on `main`
  unpublished while crates.io keeps serving the vulnerable version.

## The CLI — a signed tag

```bash
git tag -s v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

One tag produces macOS (aarch64 + x86_64), Linux (aarch64 + x86_64, musl-static) and
Windows (signed `.exe`, `.msi`, `.zip`) artifacts with a `checksums.txt`, publishes
`peira-cli` to crates.io, builds `.deb`s for amd64/arm64, and dispatches to Homebrew,
winget and Cloudsmith.

## The two tag controls, which only work as a pair

`release.yml` triggers on **`v[0-9]*`, never `v*`**, and `release-plz.toml` sets
**`git_tag_name = "{{ package }}-v{{ version }}"`**.

Neither is sufficient alone. Without the trigger, a library tag can fire a binary
build. Without `git_tag_name`, release-plz defaults to a bare `v{{ version }}` tag
that collides with the binary tags, and release-plz then dies on a manually-pushed
`vX.Y.Z` with *"local package has a greater version … but the git tag exists"*.

Verified: `v0.1.0` and `v1.2.3` match the trigger; `peira-core-v0.1.0` and
`peira-cli-v0.1.0` do not.

## Ordering for the FIRST release

The libraries must reach crates.io **before** the first `v0.1.0` tag. `peira-cli`
depends on `peira-core` by `version` as well as `path`, so packaging it against an
empty registry fails outright:

```
cargo package -p peira-cli
  -> no matching package named `peira-core` found
     location searched: crates.io index
```

So: libraries first (release-plz), then the tag. After the first release the
constraint disappears — the registry always has a previous version to resolve
against.

## Before the first tag — bootstrap that lives outside this repo

The repo inherits all 14 SecurityRonin org secrets and carries no repo-level copies,
so the shadowing trap is clear. What is **not** yet in place:

1. **Azure federated credential — required, or every Windows build fails.**
   peira was created 2026-08-09, after the 2026-07-15 immutable-subjects cutover, so
   GitHub sends an immutable-ID subject that the fleet's plain-name wildcard
   credential cannot match (`AADSTS7002131 No matching federated identity record`).
   The wildcard cannot be broadened — the flexible-expression validator rejects the
   `@id` shape. Add a standard subject-based credential:

   ```bash
   az ad app federated-credential create \
     --id 1381bf9d-c6b2-4f17-becd-0fb83083b90d \
     --parameters '{"name":"peira-release-immutable",
                    "issuer":"https://token.actions.githubusercontent.com",
                    "subject":"repo:SecurityRonin@233419394/peira@1328949340:environment:release",
                    "audiences":["api://AzureADTokenExchange"]}'
   ```

   The subject is built from this repo's real org and repo IDs. If it still fails,
   take the subject verbatim from the failing run's log, which prints `subject claim - …`.

2. **`update-peira` handler in `SecurityRonin/homebrew-tap`** — copy
   `update-blazehash.yml`. Until it exists the dispatch fires at nothing; the step is
   `continue-on-error` so it cannot sink a release.

3. **`securityronin/peira` repository on Cloudsmith** — the `.deb` reaches the GitHub
   Release regardless; only the apt channel waits on this.

4. **winget first submission is manual** — `vedantmgoyal9/winget-releaser` performs
   *updates* only. The step is `continue-on-error` until `SecurityRonin.peira` is
   registered in `microsoft/winget-pkgs`.

Windows signing is deliberately **not** `continue-on-error`. Degrading silently to an
unsigned binary is worse than a failed release: the failure is visible and fixable,
an unsigned artifact ships and is trusted.

## Crate names

`peira-core`, `peira-lens`, `peira-court`, `peira-index`, `peira-cli` were all
confirmed unclaimed on crates.io. The bare `peira` is also unclaimed and is
deliberately not a crate — under the fleet naming grammar a suite's repo name is an
umbrella, and the front-end takes the `-cli` suffix. Registering it defensively is a
separate decision, not a naming fix.
