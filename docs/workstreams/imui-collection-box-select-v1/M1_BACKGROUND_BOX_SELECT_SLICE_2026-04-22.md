# ImUi Collection Box Select v1 - M1 Background Box Select Slice

Status: landed bounded slice
Date: 2026-04-22

## Goal

Land one background-only marquee / box-select slice on the existing collection-first proof surface
without widening `fret-ui-kit::imui` or `crates/fret-ui`.

## Landed surface

1. `apps/fret-examples/src/imui_editor_proof_demo.rs` now mounts an explicit background pointer
   region around the collection browser content.
2. Background drag now draws a marquee overlay and updates collection selection app-locally.
3. Direct clicks on asset tiles still use the existing `multi_selectable_with_options(...)`
   semantics; the new slice is background-only policy, not a replacement for item selection.
4. Selection stays normalized to visible collection order even when the asset list is reversed.
5. Plain background click clears the selection; primary-modifier background drag appends to the
   baseline set.

## Explicit rejects

1. No new public `fret-ui-kit::imui` box-select helper is admitted in this lane.
2. No lasso / freeform drag-rectangle policy is admitted in this lane.
3. No collection keyboard-owner or richer shortcut-owner widening is admitted here.
4. No runtime contract changes were needed in `crates/fret-ui`.

## Gates tied to this slice

- `cargo nextest run -p fret-examples --test imui_editor_collection_box_select_surface --no-fail-fast`
- `python tools/gate_imui_workstream_source.py`
- `cargo nextest run -p fret-examples --lib proof_collection_drag_rect_normalizes_drag_direction proof_collection_box_select_replace_uses_visible_collection_order proof_collection_box_select_append_preserves_baseline_and_adds_hits --no-fail-fast`

## Evidence anchors

- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/tests/imui_editor_collection_box_select_surface.rs`
- `tools/gate_imui_workstream_source.py`
