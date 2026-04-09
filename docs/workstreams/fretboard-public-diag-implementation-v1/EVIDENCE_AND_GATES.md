# Fretboard Public Diag Implementation v1 Evidence And Gates

Status: Active
Last updated: 2026-04-09

## Repro

Current public entrypoint repro:

```bash
cargo run -p fretboard -- diag --help
cargo run -p fretboard -- diag suite ui-gallery
```

Observed current state:

- `fretboard diag --help` teaches only the public-core diagnostics verbs
- repo-only top-level verbs like `suite` are rejected from the public path
- `fretboard-dev diag --help` still keeps the richer maintainer surface

## Gates

```bash
cargo test -p fret-diag public_mode_root_help_uses_public_branding_and_examples
cargo test -p fret-diag public_mode_query_help_uses_public_branding
cargo test -p fret-diag public_mode_rejects_repo_only_top_level_commands
cargo test -p fret-diag high_risk_main_lane_help_has_drift_guards
cargo test -p fretboard
cargo run -p fretboard-dev -- diag --help
cargo run -p fretboard -- --help
cargo run -p fretboard -- diag --help
cargo run -p fretboard -- diag latest --help
cargo run -p fretboard -- diag run --help
cargo run -p fretboard -- diag suite ui-gallery
cargo fmt --check
git diff --check
```

Current publish-boundary audit:

```bash
python3 tools/release_closure_check.py --config release-plz.toml --print-publish-commands
cargo publish --dry-run --allow-dirty -p fret-diag
cargo publish --dry-run --allow-dirty -p fretboard
```

Observed result:

- release closure reports 53 crates, 0 internal dependency issues, and orders `fret-diag` before
  `fretboard`
- `cargo publish --dry-run --allow-dirty -p fret-diag` succeeds
- `cargo publish --dry-run --allow-dirty -p fretboard` still fails during local package preparation
  with crates.io resolution error: `no matching package named 'fret-diag' found`
- this remaining failure is expected until `fret-diag` is actually published to crates.io

## Evidence anchors

- `docs/workstreams/fretboard-public-app-author-surface-v1/DIAG_TARGET_INTERFACE_STATE.md`
- `crates/fret-diag/src/cli/mod.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/contracts/commands/query.rs`
- `crates/fret-diag/src/cli/contracts/commands/script.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `crates/fret-diag/src/lib.rs`
- `crates/fretboard/Cargo.toml`
- `crates/fretboard/src/cli/contracts.rs`
- `crates/fretboard/src/cli/help.rs`
- `crates/fretboard/src/cli/mod.rs`
- `crates/fretboard/src/diag.rs`
