# Releasing dex-parser-analyzer

From github.com/dhilipsiva/products/templates/RELEASING.md. The release workflow (`.github/workflows/release.yml`) and this runbook change in the same commit.

## What a release is

- Channels: crates.io (the three crates below), a GitHub Release with the `dexpa` binaries and SHA256SUMS, and the GUI on GitHub Pages (https://dhilipsiva.github.io/dex-parser-analyzer/).
- Crates to publish (`CRATES` in `release.yml`; cargo orders them by dependency): `dexpa-core`, `dexpa-analysis`, `dexpa-cli`. `dex-gui` is `publish = false`.
- Binary and targets (`BIN` and the `build` matrix): `dexpa` for `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `aarch64-apple-darwin` and `x86_64-pc-windows-msvc`, built with the toolchain pinned in `rust-toolchain.toml`.
- Secrets: `CARGO_REGISTRY_TOKEN` (crates.io, publish-update scope on exactly these three crates, with an expiry). The first publish of a new crate name needs a token with the publish-new scope; narrow it afterwards.

## One-time setup

- Settings, Environments, `github-pages`, "Deployment branches and tags": add a tag rule `v*`. Without it the `gui` job is refused, because it deploys from the tag; the release itself is already public by then, and the job can be re-run after the fix.
- Settings, Pages: the source is "GitHub Actions".

## Before tagging

1. CI is green on `main` for the commit you will tag.
2. Bump the version in `Cargo.toml` (`[workspace.package] version`). The tag must equal it; the preflight refuses a mismatch.
3. Roll `CHANGELOG.md`: rename `[Unreleased]` to `[X.Y.Z] - YYYY-MM-DD` and open a new empty `[Unreleased]`. The release notes are that section, and the preflight refuses a tag without it.
4. `nix develop -c just release` passes locally: the gate, `cargo publish --dry-run` for the three crates, and `nix build` of `.#dex-cli` and `.#dex-gui`.
5. Rehearse: run the Release workflow by hand (Actions, Release, "Run workflow"). A manual run is always a dry run. It builds every binary, writes SHA256SUMS, renders the notes and dry-runs the publish, and creates and deploys nothing. Before step 3 it rehearses with the `[Unreleased]` notes and says so in a warning.

## Cutting the release (the owner)

```sh
git switch main && git pull --ff-only
git tag -a vX.Y.Z -m "dex-parser-analyzer X.Y.Z"
git push origin vX.Y.Z
```

The workflow then runs preflight -> CI -> binaries -> a draft GitHub Release (checksums, build provenance) -> crates.io -> undraft -> GUI redeploy. The publish is the only irreversible step, and it runs after everything it depends on.

## If it fails

- Before `publish`: fix the cause on `main`, delete the tag locally and on the remote, and tag again. Nothing has shipped.
- During or after `publish`: never re-tag or reuse the version. A crates.io version is consumed forever, yanked or not. Use GitHub's "Re-run failed jobs" on the same run. If a crate must be withdrawn, `cargo yank`, then release X.Y.Z+1.
- An expired or revoked token fails fast at "The crates.io token is live", before anything is published.
- `gui` only: fix the environment rule (above) or the build, then re-run the job. The demo keeps the previous build until then.

## After the release

- docs.rs: check each crate's build page (`https://docs.rs/crate/<crate>/X.Y.Z/builds`) within a few hours.
- `curl -s -o /dev/null -w '%{http_code}' https://dhilipsiva.github.io/dex-parser-analyzer/` prints 200.
- The launch note (what works, the rule catalogue in docs/rules.md, known limits, how to report a crash on hostile input) goes in the GitHub Release body, stating only measured claims.
