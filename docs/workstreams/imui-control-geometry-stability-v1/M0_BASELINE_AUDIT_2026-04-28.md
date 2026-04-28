# M0 Baseline Audit - 2026-04-28

## Evidence Reviewed

- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json`
- `docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json`
- `docs/workstreams/imui-menu-tab-policy-depth-v1/WORKSTREAM.json`
- `docs/workstreams/imui-interaction-inspector-v1/WORKSTREAM.json`
- `docs/workstreams/imui-interaction-inspector-diag-gate-v1/WORKSTREAM.json`
- `docs/workstreams/imui-collection-helper-readiness-v1/WORKSTREAM.json`
- `docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs`
- `ecosystem/fret-imui/src/tests/floating.rs`

## Findings

### 1. The existing control-chrome lane is closed

`imui-control-chrome-fearless-refactor-v1` is the right historical record for the shared IMUI
control-chrome rewrite, but its `continue_policy` directs future field-width or family-specific
parity pressure into narrower follow-ons. This lane should not be reopened for a cross-control
geometry audit.

### 2. The text-control follow-on is closed and supplies the inherited floor

`imui-text-control-chrome-stability-v1` fixed the concrete input/textarea focus-size report by
moving compact IMUI text controls away from shadcn recipe focus-ring chrome. The next problem is
broader: every base control should have the same state-invariant geometry posture.

### 3. Linux/Wayland belongs to docking parity, not this lane

`docking-multiwindow-imgui-parity` is still active and owns Wayland compositor acceptance and
runner/backend multi-window behavior. This lane is intentionally local and control-surface scoped.

### 4. Floating window z-order appears tracked, but one stale public comment remains suspicious

The historical v3 notes and current `fret-imui` floating tests show that floating layer z-order,
focus, no-inputs, passthrough, and bring-to-front behavior have substantial coverage. The comment in
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` saying z-order/focus arbitration is a separate
work item appears stale. That should be handled as a small floating contract hygiene follow-on or
cleanup after this lane's first geometry slice, not mixed into this lane.

### 5. Identity ergonomics is not currently tracked by an active lane

The historical `imui-ecosystem-facade-v3` note still mentions ImGui `"##"` / `"###"` identity
ergonomics. That folder is historical/pre-reset and should not be resumed directly. A future
identity lane is justified only if it starts from a fresh design and proof surface.

### 6. Admitted base-control inventory

The lane admits compact IMUI controls that are local and authorable through the current immediate
facade:

- input text / textarea: inherited floor from `imui-text-control-chrome-stability-v1`
- button, checkbox, radio, switch, slider, combo trigger, selectable: focused geometry gate only
- menu trigger, submenu trigger, tab trigger: focused geometry gate only, with menu/tab behavior
  policy kept in the already-closed policy-depth lane
- Linux compositor behavior, docking, floating OS windows, and editor-grade tab overflow/reorder:
  different owners

## Decision

Open `imui-control-geometry-stability-v1` as the next local IMUI follow-on.

The first implementation slice should audit the admitted base-control families and add focused
state-invariant geometry gates before refactoring any control that still changes outer bounds during
interaction.
