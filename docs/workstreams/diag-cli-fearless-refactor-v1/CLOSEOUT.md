# Diag CLI Fearless Refactor v1 — Closeout

Status: Closeout-ready
Last updated: 2026-03-26

Related:

- `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/OWNERSHIP.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `apps/fretboard/src/cli.rs`

## 0) Summary

This workstream landed the pre-release hard reset for `fretboard diag`.

The repo no longer carries parser-v1, no longer keeps a legacy simple-dispatch helper, and no
longer treats `clap` as a side scaffold. `clap` is now the single parser/help source of truth for
the diagnostics CLI contract.

## 1) What this lane completed

- deleted the old mutable parser loop from `crates/fret-diag/src/lib.rs`
- deleted the legacy simple-dispatch helper
- centralized `diag` help generation in the `clap` contract tree
- migrated all currently shipped `diag` command families onto the typed contract + cutover path
- removed duplicated help/usage ownership from migrated execution modules
- kept the execution layer `clap`-free by normalizing parser structs into internal contexts
- rejected deleted syntax explicitly instead of silently guessing or falling back

## 2) Final contract decision

The final merged state for this lane is:

- one parser tree for `fretboard diag`
- one help surface generated from that tree
- one parser-to-execution normalization layer
- zero compatibility fallbacks retained only for historical spellings

This is intentionally a hard break window. Any future command-shape work should extend the typed
contract directly instead of reopening parser-v1 compatibility behavior.

## 3) Repro, Gate, Evidence

Repro / smoke commands:

- `target/debug/fretboard diag --help`
- `target/debug/fretboard diag agent --help`
- `target/debug/fretboard diag path --help`
- `target/debug/fretboard diag poke --help`
- `target/debug/fretboard diag latest --help`
- `target/debug/fretboard diag agent target/fret-diag-ai-model-selector-focus-gate --json --out target/fret-diag-agent.plan.json`

Gate commands:

- `cargo nextest run -p fret-diag cli::contracts::tests:: cli::cutover::tests::`
- `cargo build -p fretboard --message-format short`

Evidence anchors:

- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `apps/fretboard/src/cli.rs`

## 4) What this lane intentionally did not keep

- no dual-parser merged state
- no deleted alias compatibility layer
- no execution-module-local `--help` branches for migrated commands
- no hand-maintained prose mirror of the full `diag` surface in `apps/fretboard`

## 5) Residual work policy

Residual hardening from the parser reset was split into narrow follow-up lanes in
`docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`:

- main execution lane hardening
- first-party caller migration
- help snapshots and smoke gates

Those follow-up lanes are now closeout-ready as well. Any future `diag` CLI contract work should
start in a new narrowly scoped workstream rather than reopening this parser reset lane.

## 6) Closeout decision

This workstream is closed out as the parser reset lane.

Future work should land in the named follow-up lanes rather than reintroducing compatibility debt
into `diag-cli-fearless-refactor-v1`.
