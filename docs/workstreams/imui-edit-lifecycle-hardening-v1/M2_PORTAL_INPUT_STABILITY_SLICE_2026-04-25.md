# ImUi Edit Lifecycle Hardening v1 - M2 Portal Input Stability Slice - 2026-04-25

## Decision

Retained node portal editors are visible editor surfaces, so they must not embed raw
`TextInputProps` without an editor-owned sizing policy. The runtime `TextInput` mechanism stays
unchanged; `ecosystem/fret-node` now owns the portal editor policy that keeps inline text and
number inputs size-stable while focus, typing, drag, and stepper affordances change state.

## Shipped Invariant

- `PortalTextInputUi` centralizes portal text-input chrome, fixed control text line boxes, and
  exact height resolution in `ecosystem/fret-node/src/ui/editors/chrome.rs`.
- `PortalTextEditor` and `PortalNumberEditor` now build inputs through the shared helper instead
  of hand-rolling bare `TextInputProps`.
- Portal text inputs set `height`, `min_height`, and `max_height` to the same resolved value, so
  text metrics, focus state, or sibling button composition cannot resize the field.
- Stepper and drag rows resolve input height from the fixed button stack height when the adjacent
  affordance column is taller than the default input field.
- No public `crates/fret-ui`, `fret-authoring::Response`, or public IMUI contract was widened.

## Evidence

- `ecosystem/fret-node/src/ui/editors/chrome.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`

Verified on 2026-04-25:

```bash
cargo fmt --package fret-node --check
cargo nextest run -p fret-node --features compat-retained-canvas portal_text_input_ --jobs 2
cargo nextest run -p fret-node --features compat-retained-canvas portal_button_stack_height --jobs 2
cargo check -p fret-node --features compat-retained-canvas --jobs 2
```

## Residual Risk

This slice proves the retained portal editor policy at unit/check level. If a future repro shows a
node-graph demo still moving bounds on pointer focus, add a small diagnostics script that captures
layout sidecar bounds before and after clicking a portal editor input.
