# MiniMap + Controls overlays (contract)

This document defines the **stable contract** for the built-in node graph add-ons:

- `NodeGraphMiniMapOverlay`
- `NodeGraphControlsOverlay`

These add-ons are **UI-only overlays**: they are not serialized into the graph document, and they
are hosted outside the canvas render transform (ADR 0135) so they can use regular UI
infrastructure (focus, pointer capture, semantics).

## Composition model

Recommended composition:

- Use `NodeGraphEditor` to layer children and give overlays full access to the editor bounds.
- For app-level layout, use `NodeGraphPanel` and place overlays inside panels instead of drawing
  them over the canvas when you want strict spatial ownership.

## Placement contract

Both overlays support two placement modes:

- **Floating in canvas bounds (default)**: the overlay computes its internal panel/rect using the
  full editor bounds and `NodeGraphStyle` sizing/margins.
- **Panel bounds mode**: call `.in_panel_bounds()` to treat `cx.bounds` as the overlay's panel
  bounds (intended for `NodeGraphPanel` composition).

## Input routing contract

The key invariant is: **overlays must not accidentally steal canvas input outside their own
interactive bounds**.

### Controls

`NodeGraphControlsOverlay` behaves as a screen-space toolbar:

- Hit-testing: inside the controls panel (floating mode) or inside the given panel bounds
  (`in_panel_bounds`).
- Pointer routing: any left-button pointer down inside the panel **blocks** underlying canvas
  interactions; button clicks capture the pointer until release.
- Commands: clicking a button dispatches the corresponding command (zoom, frame, reset, etc.).
- Focus: successful button activation requests focus to the `canvas_node` so keyboard input remains
  on the canvas.
- Keyboard: when focused, arrow keys move the active button selection and Enter/Space activates the
  selected button. Escape returns focus to the canvas without dispatching a command.

### MiniMap

`NodeGraphMiniMapOverlay` behaves as an interactive minimap:

- Hit-testing: inside the minimap rect (floating mode) or inside the given panel bounds
  (`in_panel_bounds`).
- Pointer routing: minimap drags capture the pointer and **block** underlying canvas interactions
  while active.
- View updates: dragging updates `NodeGraphViewState.pan`. When `.with_store(store)` is used, the
  overlay must also update the store viewport (`NodeGraphStore::set_viewport`) for B-layer
  integration.

## Styling / theme tokens

Sizing and placement are driven by `NodeGraphStyle`:

- MiniMap: `minimap_width`, `minimap_height`, `minimap_margin`, `minimap_world_padding`
- Controls: `controls_button_size`, `controls_gap`, `controls_margin`, `controls_padding`,
  `controls_text_style`, `controls_text`, `controls_hover_background`, `controls_active_background`

## Accessibility baseline

Both overlays must contribute a stable semantics surface:

- each overlay emits a semantics node with a stable `test_id`,
- each overlay provides a human-readable label (at least “Controls” / “MiniMap”).

Per-button semantics for the controls overlay is intentionally out of scope for this baseline and
may be added later via a dedicated composite widget.

## Conformance gates

- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
