# Diag CLI Main Lanes Hardening v1

Status: Closeout-ready
Last updated: 2026-03-26

Source lane:

- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`

## Scope

Finish the remaining hardening for the main execution command families after the parser reset
landed:

- `run`
- `suite`
- `repro`
- `repeat`

## Landed slice

The first hardening slice for this lane is now merged into the parser-local contract + cutover
path:

- `repeat --repeat 0` is rejected by the parser instead of being silently coerced to `1`
- `suite` rejects empty input before runtime dispatch instead of falling through to a later
  indexing failure path
- `run` and `suite` reject DevTools transport usage when `--launch` or `--reuse-launch` is also
  requested
- DevTools transport arguments now enforce their required peers structurally in the contract tree
  (`--devtools-ws-url` + `--devtools-token`, plus `--devtools-session-id` requiring a WS URL)
- `repro` now resolves its targets during cutover, rejects unknown suite names / missing script
  paths before execution, and rejects conflicting script `meta.env_defaults` before launch

## Carries

- `DCR-core-021`
- `DCR-core-022`
- `DCR-core-023`
- `DCR-core-024`
- `DCR-core-027`
- `DCR-core-028`
- `DCR-core-029`
- `DCR-core-030`

## Exit criteria

- parser-local validation for the main lanes is explicit and local
- help/examples for the main lanes are fully generated and current
- representative valid and invalid parser coverage exists for the main lanes
- no remaining “partial migration” wording is needed for `run` / `suite` / `repro` / `repeat`

## Repro, Gate, Evidence

Gate commands:

- `cargo nextest run -p fret-diag cli::contracts::tests:: cli::cutover::tests::`
- `cargo build -p fretboard --message-format short`

Evidence anchors:

- `crates/fret-diag/src/cli/contracts/shared/devtools.rs`
- `crates/fret-diag/src/cli/contracts/commands/repeat.rs`
- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `crates/fret-diag/src/diag_repro.rs`
- `crates/fret-diag/src/diag_repro/scripts.rs`

## Closeout notes

This lane is closeout-ready.

The originally named hardening targets for `run`, `suite`, `repro`, and `repeat` now land at the
parser-local contract + cutover boundary, and the highest-risk help/example drift is now covered by
focused contract tests.

If further invalid-combination tightening is needed later, it should land as a new narrow follow-up
instead of reopening this lane.

## Closeout decision

This lane is closed out as the main execution hardening pass.

Evidence for the landed slices is tracked in:

- `208131db8` (`fix(diag): harden main lane cli validation`)
- `cbd1c5fa4` (`fix(diag): frontload repro cli validation`)
- `4b4635153` (`test(diag): harden help drift guards`)
