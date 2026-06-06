# ImUi Floating Layer Dismissal Test Split v1 TODO

Status: Closed
Last updated: 2026-06-06

## M1 - Layer Dismissal Proof Split

- [x] Keep `ecosystem/fret-imui/src/tests/floating/layer_dismissal.rs` as a hub.
- [x] Move the menu non-click-through outside-press proof into `layer_dismissal/menu.rs`.
- [x] Move the popover click-through outside-press proof into `layer_dismissal/popover.rs`.
- [x] Add source-gate coverage for the hub, menu owner, and popover owner files.
- [x] Record focused proof gates.

## Future Follow-Ons

- [ ] Edit future floating-layer dismissal coverage in the specific owner module.
- [ ] Start a behavior-specific follow-on before changing runtime overlay contracts.
- [ ] Keep `fret-imui` as a proof layer; do not move floating-layer policy from
  `fret-ui-kit::imui` into it.
