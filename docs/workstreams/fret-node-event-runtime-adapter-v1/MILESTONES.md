# Fret Node Event Runtime Adapter v1 - Milestones

Status: Active
Last updated: 2026-05-25

## M0 - Scope And Evidence Freeze

Exit criteria:

- Problem, target state, and non-goals are explicit.
- First proof target is the retained event runtime entrypoint.
- Gate set is recorded.

Status: Complete.

Evidence:

- `docs/workstreams/fret-node-event-runtime-adapter-v1/DESIGN.md`
- `docs/workstreams/fret-node-event-runtime-adapter-v1/TODO.md`

## M1 - Event Runtime Adapter Proof

Exit criteria:

- A named retained-agnostic event runtime adapter seam exists.
- Retained `EventCx` binding is isolated in an explicit retained adapter module.
- Source-policy tests lock route orchestration away from retained contexts.

Primary gates:

- `cargo check -p fret-node --features compat-retained-canvas`
- `cargo test -p fret-node --features compat-retained-canvas event_runtime_adapter`

## M2 - Retained Edge Shrink

Exit criteria:

- One direct retained event runtime edge is deleted or quarantined.
- Default and compat `fret-node` checks pass.

Primary gates:

- `cargo check -p fret-node`
- `cargo check -p fret-node --features compat-retained-canvas`

## M3 - Closeout

Exit criteria:

- Evidence gates are fresh.
- Remaining event route work is either complete, deferred, or split into a follow-on.
- `WORKSTREAM.json` status is updated.
