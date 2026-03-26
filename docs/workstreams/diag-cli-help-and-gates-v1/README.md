# Diag CLI Help And Gates v1

Status: Closeout-ready
Last updated: 2026-03-26

Source lane:

- `docs/workstreams/diag-cli-fearless-refactor-v1/FOLLOWUPS.md`

## Scope

Add durable regression protection around the user-facing CLI surface after the parser reset landed.

## Landed slice

The first slice for this lane adds focused help drift guards for the highest-risk execution
surfaces:

- root `diag --help` examples now include `suite` and `repro` alongside `run` / `perf` / `campaign`
- contract tests now lock key help text for `run`, `suite`, `repro`, `perf`, and `campaign run`
- top-level `fretboard` help examples were updated to mirror the intended generated `diag` entry
  points

## Carries

- `DCR-repo-053`
- `DCR-repo-054`

## Exit criteria

- help output has snapshot coverage or equivalent drift guards
- the highest-risk command families have focused smoke coverage:
  - `run`
  - `suite`
  - `repro`
  - `perf`
  - `campaign`

## Repro, Gate, Evidence

Gate commands:

- `cargo nextest run -p fret-diag cli::contracts::tests:: cli::cutover::tests::`
- `cargo build -p fretboard --message-format short`

Evidence anchors:

- `crates/fret-diag/src/cli/contracts/mod.rs`
- `crates/fret-diag/src/cli/cutover.rs`
- `apps/fretboard/src/cli.rs`

## Closeout decision

This lane is closeout-ready.

The highest-risk user-facing help surfaces now have focused drift guards, and the first-party
`fretboard` help examples have been updated to match the generated `diag` contract surface.

If broader snapshot coverage is added later, it should land as a new narrow follow-up instead of
reopening this lane.

Evidence for the landed slice is tracked in:

- `4b4635153` (`test(diag): harden help drift guards`)
