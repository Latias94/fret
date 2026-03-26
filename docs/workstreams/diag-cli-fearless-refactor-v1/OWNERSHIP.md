# Diag CLI Fearless Refactor v1 — Final Ownership

Status: Active maintainer reference
Last updated: 2026-03-26

Related:

- `docs/workstreams/diag-cli-fearless-refactor-v1/README.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/CLOSEOUT.md`
- `docs/workstreams/diag-cli-fearless-refactor-v1/PARSER_MODEL.md`
- `crates/fret-diag/src/lib.rs`
- `crates/fret-diag/src/cli/mod.rs`
- `crates/fret-diag/src/cli/contracts/`
- `crates/fret-diag/src/cli/cutover.rs`
- `apps/fretboard/src/cli.rs`

## 0) Purpose

This note records the final ownership split after the `clap` cutover landed in production code.

Use this file as the maintainer map for future `diag` CLI work. The older `PARSER_MODEL.md` remains
useful as the design baseline, but this file is the final ownership reference.

## 1) Top-level ownership split

- `apps/fretboard`
  - owns the top-level application shell
  - owns non-`diag` command families
  - should not mirror the full `diag` help surface in prose
- `crates/fret-diag`
  - owns the entire `diag` contract surface
  - owns parser-to-execution normalization
  - owns execution entrypoints for `diag`

## 2) File-level ownership

- `crates/fret-diag/src/lib.rs`
  - stable entrypoint for `diag_cmd`
  - delegates directly to the canonical CLI dispatcher
- `crates/fret-diag/src/cli/mod.rs`
  - workspace/path helpers and shared CLI plumbing
- `crates/fret-diag/src/cli/contracts/mod.rs`
  - top-level `clap` command tree
  - generated help source of truth
- `crates/fret-diag/src/cli/contracts/commands/`
  - one module per command family or command helper surface
  - field-level declaration of the public CLI contract
- `crates/fret-diag/src/cli/contracts/shared/`
  - reusable arg families
  - only for flags reused with the same semantics across command families
- `crates/fret-diag/src/cli/cutover.rs`
  - command-local semantic normalization
  - parser-to-execution context conversion
  - explicit rejection of deleted syntax
- execution modules such as `diag_run.rs`, `diag_suite.rs`, `diag_perf.rs`, `commands/*.rs`
  - remain `clap`-free
  - consume normalized internal contexts or minimal arguments

## 3) Rules for future command work

- Add or remove `diag` commands in the contract tree first.
- Keep generated help in the contract tree; do not reintroduce hand-written usage ownership in
  `apps/fretboard` or execution modules.
- Reject deleted syntax explicitly with a focused message; do not silently guess or fall back.
- Promote a flag family into `shared/` only when at least two command families use the same
  semantics.
- Keep execution modules focused on runtime behavior, not parser shape.

## 4) What does not belong here

- non-`diag` top-level app CLI behavior
- policy-only documentation for unrelated command families
- compatibility shims whose only purpose is to preserve deleted pre-release spellings

## 5) Change checklist for maintainers

When changing the `diag` surface:

1. Update `crates/fret-diag/src/cli/contracts/commands/*`.
2. Update `crates/fret-diag/src/cli/contracts/mod.rs`.
3. Update `crates/fret-diag/src/cli/cutover.rs`.
4. Add or update parser/cutover tests.
5. Update repo-owned docs and workstream notes that teach the command.

If the remaining work is broader than one small command family, start a narrow follow-up lane
instead of expanding this closeout lane again.
