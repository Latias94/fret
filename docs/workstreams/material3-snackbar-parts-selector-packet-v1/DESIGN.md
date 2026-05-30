# Material 3 Snackbar Parts Selector Packet v1 - Design

Status: Closed
Last updated: 2026-05-28

## Problem

The Material 3 overlay feedback packet left Snackbar with only a root `test_id`. That proved the
Material recipe forwarded `Snackbar::test_id` into the shared toast layer, but consumers still could
not target the live action or close affordance without brittle structure assumptions.

Upstream references split this responsibility across parts:

- MUI `SnackbarContent` has separate `message` and `action` slots.
- Compose Material3 `Snackbar` accepts optional `action` and `dismissAction` content.
- Base UI and Radix model toast `Action` and `Close` as headless button parts.

Fret should keep toast viewport, action routing, dismiss/close, and live-region policy in
`fret-ui-kit`, while allowing Material Snackbar automation to target the action and close parts
derived from the toast root id.

## Target State

- A toast with root id `toast` exposes `toast.action`, `toast.cancel`, and `toast.close` when those
  parts are rendered.
- Material `Snackbar::test_id("snackbar")` proves live `snackbar`, `snackbar.action`, and
  `snackbar.close` selectors through the existing `SnackbarHost` path.
- Material recipe code remains a request skin; it does not duplicate toast composition.
- No `crates/*` mechanism change is needed because the current semantics/test-id contract already
  supports this.

## Truth Set

- Truth 1: Kit toast action/cancel/close buttons derive stable dotted ids from the toast root id.
- Truth 2: Material Snackbar action/close selectors are live when a Snackbar with an action is
  shown through `SnackbarHost`.
- Truth 3: Toast live-region and dismiss behavior remain owned by `fret-ui-kit`.
- Truth 4: No Material foundation helper is introduced for a shared toast concern.

## Layer Mapping

- `ecosystem/fret-ui-kit/src/window_overlays/render.rs`: kit policy owns rendered toast parts,
  action routing, cancel/close dismissal, and root-derived part selectors.
- `ecosystem/fret-ui-material3/src/snackbar.rs`: Material recipe owns request skinning and root id
  forwarding only.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`: Material-facing proof that root ids
  propagate to live action/close parts.
- `crates/*`: no mechanism change.

## Non-Goals

- Do not add Material-specific toast renderer forks.
- Do not alter Snackbar timing, stacking, live-region behavior, or visual chrome.
- Do not add message/supporting-text part selectors in this slice; the concrete residual risk was
  action/close automation.

## Upstream References

- MUI Material `SnackbarContent`: root, message, and action slots.
- Compose Material3 `Snackbar`: optional `action` and `dismissAction` content.
- Base UI / Radix Toast: separate `Action` and `Close` button parts.
