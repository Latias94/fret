# ImUi Floating Layer Dismissal Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-06-06

This lane is a narrow follow-on from the current IMUI gap-closure audit. It keeps the IMUI
floating-layer outside-press proofs small enough to review. The original
`floating/layer_dismissal.rs` file mixed menu and popover dismissal behavior in one aggregate test
file even though the two proofs assert different underlay-input contracts.

## Ownership

- `fret-imui` owns the proof tests and test module registration.
- `fret-ui-kit::imui` continues to own floating-layer and popup policy behavior.
- `crates/fret-ui` overlay runtime contracts are not changed.

## Must-Be-True Outcomes

- `ecosystem/fret-imui/src/tests/floating/layer_dismissal.rs` remains a small module hub.
- Menu outside-press coverage lives in `floating/layer_dismissal/menu.rs`.
- Click-through popover outside-press coverage lives in `floating/layer_dismissal/popover.rs`.
- The menu proof continues to show non-click-through outside press dismissal without activating the
  underlay window.
- The popover proof continues to show click-through outside press dismissal with underlay window
  activation.

## Fixture Decision

These remain Rust interaction tests because they exercise real `UiTree` hit testing, overlay stack
arbitration, model updates, and floating-window z-order through the IMUI host.

## Non-Goals

- No behavior or public API changes.
- No fixture schema.
- No overlay runtime contract changes.
- No changes to floating-window positioning, z-order, focus, or pointer-capture semantics.
