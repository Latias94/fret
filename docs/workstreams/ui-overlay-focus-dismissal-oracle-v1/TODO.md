# UI Overlay Focus Dismissal Oracle v1 - TODO

Status: Active
Last updated: 2026-05-25

## ODO-M0 - Oracle Vocabulary

- [ ] ODO-010 [owner=planner] [deps=none] [scope=docs/workstreams/ui-overlay-focus-dismissal-oracle-v1]
  Goal: Define the minimal oracle vocabulary for dismissal and focus restore outcomes.
  Validation: DESIGN.md or a fixture note names fields and first cases.
  Evidence: `docs/workstreams/ui-overlay-focus-dismissal-oracle-v1/HANDOFF.md`
  Handoff: Keep the oracle policy-owned.

## ODO-M1 - First Fixture Family

- [ ] ODO-020 [owner=unassigned] [deps=ODO-010] [scope=crates/fret-ui,ecosystem/fret-ui-kit,tools/diag-scripts]
  Goal: Convert one existing outside-press/focus-restore behavior into an oracle-backed fixture.
  Validation: `cargo test -p fret-ui outside_press`
  Evidence: fixture path and test path.
  Handoff: Expand by behavior family after the first fixture proves useful.
