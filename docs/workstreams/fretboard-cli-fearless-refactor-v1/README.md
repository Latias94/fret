# Fretboard CLI Fearless Refactor v1

Status: Closeout-ready
Last updated: 2026-03-26

Tracking files:

- `docs/workstreams/fretboard-cli-fearless-refactor-v1/README.md`
- `docs/workstreams/fretboard-cli-fearless-refactor-v1/CLOSEOUT.md`
- `docs/workstreams/fretboard-cli-fearless-refactor-v1/TODO.md`
- `docs/workstreams/fretboard-cli-fearless-refactor-v1/MILESTONES.md`

Related context:

- Top-level CLI shell: `apps/fretboard/src/cli/mod.rs`
- Top-level typed contract: `apps/fretboard/src/cli/contracts.rs`
- Top-level cutover: `apps/fretboard/src/cli/cutover.rs`
- Scaffold command family: `apps/fretboard/src/scaffold/mod.rs`
- Diagnostics parser closeout: `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`

## 0) Why this workstream exists

`fretboard` no longer needs a compatibility-first CLI migration plan.

The repository is still pre-release, so the right target is a single typed command model with clear
ownership boundaries, not a pile of forwarded argv bags and hand-maintained help prose.

Recent progress already moved `assets`, `dev`, `hotpatch`, `config`, and `theme` onto typed `clap`
contracts. That work proved the shape we want:

- top-level command declaration in `apps/fretboard/src/cli/contracts.rs`,
- one cutover dispatch layer in `apps/fretboard/src/cli/cutover.rs`,
- per-family `contracts.rs` for parser ownership,
- execution modules that stay focused on behavior rather than parsing.

This lane is now closeout-ready because the remaining top-level drift has been removed:

- `new` is now a typed family-local contract,
- `init` has been deleted instead of preserved as a compatibility alias,
- root help now renders from the executable contract and only appends curated examples.

## 1) Main decision

This lane is a pre-release hard reset for the top-level `fretboard` shell.

Decision:

- use `clap` as the parser contract for repo-owned top-level command families,
- keep one typed top-level command tree in `apps/fretboard/src/cli/contracts.rs`,
- keep parser/cutover ownership modular by family instead of rebuilding one new blob,
- do not keep compatibility-only parsing paths or aliases in merged code,
- update first-party docs and tests atomically with command-surface changes.

Clarification:

- `fretboard-dev diag` remains user-facing through the same binary, but its canonical parser/help
  contract lives in `crates/fret-diag`; this lane should not duplicate that tree in
  `apps/fretboard`.

## 2) Scope

### In scope

- top-level `fretboard` command-family ownership in `apps/fretboard`,
- typed `clap` contracts for repo-owned command families,
- command-family cutover dispatch and module boundaries,
- root help ownership and drift reduction,
- repo docs/tests that teach the changed top-level command shapes.

### Explicit non-goals

- redesigning `fret-diag` internals,
- redesigning scaffold template contents by default,
- preserving `init` or other aliases purely for historical compatibility,
- keeping hand-written help mirrors for surfaces already expressed by executable contracts.

## 3) End-state requirements

The final shipped state should satisfy all of the following:

1. `apps/fretboard` has one typed top-level command tree.
2. Each repo-owned command family has an obvious home for parser structs and execution code.
3. New parser work does not widen `cli/contracts.rs` into another god file.
4. `scaffold` no longer hand-rolls argv parsing.
5. Compatibility-only aliases are deleted instead of hidden behind forwarding shims.
6. Root help does not manually duplicate subcommand syntax that `clap` already knows.
7. First-party docs and tests teach only the intended surface.

## 4) Locked technical direction

The command-shell structure is now locked to this shape:

- `apps/fretboard/src/cli/contracts.rs`
  - top-level command enum and shared root-local arg groups only
- `apps/fretboard/src/cli/cutover.rs`
  - dispatch only
- `apps/fretboard/src/<family>/contracts.rs`
  - family-local typed parser model
- `apps/fretboard/src/<family>/*.rs`
  - execution helpers that stay as `clap`-light as practical

Current merged progress:

- `assets`: typed contract complete
- `dev`: typed contract complete, execution split into `native.rs` and `web.rs`
- `hotpatch`: typed contract complete
- `config`: typed contract complete
- `theme`: typed contract complete
- `new`: typed contract complete
- `diag`: forwarded to the canonical typed contract in `crates/fret-diag`
- `init`: deleted as a compatibility-only alias
- root help: generated from the top-level `clap` contract with curated examples appended

## 5) Evidence anchors

- `apps/fretboard/src/cli/contracts.rs`
- `apps/fretboard/src/cli/cutover.rs`
- `apps/fretboard/src/assets/contracts.rs`
- `apps/fretboard/src/dev/contracts.rs`
- `apps/fretboard/src/hotpatch/contracts.rs`
- `apps/fretboard/src/config/contracts.rs`
- `apps/fretboard/src/theme/contracts.rs`
- `apps/fretboard/src/scaffold/contracts.rs`
- `apps/fretboard/src/scaffold/mod.rs`
- `apps/fretboard/src/cli/help.rs`

## 6) Current smoke evidence

- `cargo nextest run -p fretboard-dev cli::contracts::tests:: config::tests:: theme::tests::`
- `cargo nextest run -p fretboard-dev cli::contracts::tests:: cli::help::tests:: scaffold::tests::`
- `target/debug/fretboard --help`
- `target/debug/fretboard-dev new --help`
- `target/debug/fretboard-dev new hello --help`
- `target/debug/fretboard init todo` (expected rejection)
