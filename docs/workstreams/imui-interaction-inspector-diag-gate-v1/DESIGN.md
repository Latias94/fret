# ImUi Interaction Inspector Diag Gate v1

Status: closed follow-on
Last updated: 2026-04-24

## Scope

This narrow follow-on promotes the product-facing response inspector in
`imui_interaction_showcase_demo` from a visual/demo-only surface into a `fretboard diag` gate.

It follows `imui-interaction-inspector-v1`, which intentionally closed before full diag automation.
The old lane stays closed; this lane only owns the proof artifact that drives one response edge and
asserts that the inspector exposes it through stable diagnostics selectors.

## Ownership

- Demo-local selector anchors live in `apps/fret-examples/src/imui_interaction_showcase_demo.rs`.
- Scripted repro lives under `tools/diag-scripts/ui-editor/imui/`.
- Promotion into registry/suite ownership lives under `tools/diag-scripts/suites/`.
- No `fret-imui`, public `fret-ui-kit::imui`, or `crates/fret-ui` API is widened.

## Target Invariant

The showcase inspector must expose the latest meaningful response edge in a way diagnostics can
query:

- the initial inspector summary is stable before interaction,
- clicking the pulse control updates the header and inspector summary,
- the `clicked` inspector flag detail changes from the initial state to the primary click edge,
- a promoted suite makes the script visible to registry checks.

## Out Of Scope

- Exhaustive automation for every response flag.
- Public response API changes.
- Shared inspector state helpers.
- Runtime response contract changes.
