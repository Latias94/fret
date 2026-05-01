# ImUi Table Column Width Diagnostics Gate v1

Status: closed
Last updated: 2026-05-01

## Problem

`imui-table-column-resize-v1` added table header resize responses, and
`imui-table-column-width-demo-proof-v1` made those responses visible in
`imui_shadcn_adapter_demo` through app-owned inspector column width state. This lane promotes that
visible proof into a deterministic `fretboard diag` gate.

## Scope

- Add one schema v2 diagnostics script under `tools/diag-scripts/ui-editor/imui/`.
- Add one suite manifest for the promoted gate.
- Add a lightweight source-marker gate tying the script to stable demo selectors and app-owned
  resize markers.
- Keep column width state app-owned and keep `fret-ui-kit::imui` as the response/mechanism layer.

## Non-Goals

- No helper-owned table sizing state.
- No column width persistence or saved layouts.
- No grouped header resize policy.
- No declarative/headless table interop recipe.
- No runtime table semantics changes in `fret-ui`.

## Target Behavior

The script opens `imui_shadcn_adapter_demo` in the regular layout, observes the initial field column
width summary, drags
`imui-shadcn-demo.inspector.table.header.cell.inspector-field.resize`, and verifies the visible
summary reaches `field 180px`.

This proves the important runtime path: stable resize handle selector -> pointer drag held across
frames -> `TableColumnResizeResponse::drag_delta_x()` -> app-owned width state update -> next-frame
width summary.

## Design Note

The gate uses `drag_pointer_until`, not a plain `drag_pointer`, because plain `drag_pointer` emits
down/move/up as a burst in one diagnostics frame. The demo consumes the resize response during
render while the drag is active, so the script must keep the pointer down across frames until the
summary predicate passes.
