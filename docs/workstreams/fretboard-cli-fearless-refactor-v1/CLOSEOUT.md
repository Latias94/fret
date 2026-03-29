# Fretboard CLI Fearless Refactor v1 — Closeout

Status: Closed
Last updated: 2026-03-26

Tracking doc: `docs/workstreams/fretboard-cli-fearless-refactor-v1/README.md`

## Outcome

This lane is closed.

`apps/fretboard` now uses one typed top-level `clap` tree for every repo-owned command family:

- `assets`
- `config`
- `dev`
- `hotpatch`
- `new`
- `theme`

`diag` remains intentionally delegated to the canonical typed contract in `crates/fret-diag`.

## What landed

- Added family-local typed contracts for `config`, `theme`, and `new`.
- Deleted the compatibility-only `init` alias from the top-level shell.
- Replaced the hand-maintained root usage blob with contract-generated root help plus curated
  examples.
- Updated first-party ADR/alignment references from `fretboard init todo` to `fretboard new todo`.
- Added parser/help tests for `new`, root help, `config`, and `theme`.
- Re-ran scaffold generation smoke and compile tests after the contract cutover.

## Evidence

- Top-level shell: `apps/fretboard/src/cli/contracts.rs`
- Top-level cutover: `apps/fretboard/src/cli/cutover.rs`
- Root help rendering: `apps/fretboard/src/cli/help.rs`
- Scaffold contract: `apps/fretboard/src/scaffold/contracts.rs`
- Scaffold execution: `apps/fretboard/src/scaffold/mod.rs`

## Smoke commands

- `cargo nextest run -p fretboard cli::contracts::tests:: cli::help::tests:: scaffold::tests::`
- `target/debug/fretboard --help`
- `target/debug/fretboard new --help`
- `target/debug/fretboard new hello --help`
- `target/debug/fretboard init todo` (expected rejection)
