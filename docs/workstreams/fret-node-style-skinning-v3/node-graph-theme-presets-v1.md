# Node graph theme presets (v1): workflow presets + custom JSON loading

This note documents the **paint-only** node-graph preset system and the intended way to iterate on
editor looks (Blueprint / ShaderGraph / Dify-style) without mutating the serialized `Graph`.

## What exists today

- Built-in presets live in `themes/node-graph-presets.v1.json` and are loaded by
  `NodeGraphPresetSkinV1::new_builtin(...)`.
- Theme-derived presets can be generated from a `ThemeSnapshot` via
  `NodeGraphPresetSkinV1::new_from_snapshot(...)`.
- A new JSON entry-point exists for custom preset iteration:
  - `fret_ui_kit::node_graph::presets::parse_node_graph_theme_presets_v1(...)`
  - `fret_node::ui::NodeGraphPresetSkinV1::try_new_from_json_str(...)`

Presets are applied via the `NodeGraphSkin` contract and must remain **paint-only**: switching a
preset should not rebuild derived geometry.

## Recommended layering

To reach editor-grade looks without future rewrites, keep a strict split:

- **Theme / presets (paint tokens):** broad palette, chrome intensity, grid, rings, port shapes.
- **Skin policy:** glows/outlines/rings; how selection/hover is expressed.
- **Per-entity overrides (ADR 0309):** runtime-only exceptions (error wires, debugging highlights)
  using `PaintBindingV1` + `PaintEvalSpaceV1` (LocalPx / ViewportPx / StrokeS01).

## Authoring presets (JSON)

The schema ID is `node_graph_theme_presets.v1`.

The JSON document is expected to provide:

- `paint_only_tokens.canvas/grid/node/port/wire/states`
- optional `layout_tokens` (node radius, header height, pin radius, etc.)

Unknown / non-normative fields (e.g. `interaction_state_matrix`) are allowed and are treated as
documentation payload.

## Wiring example (custom JSON)

At app startup, parse a JSON blob and install the skin as a global:

```rust
let raw = include_str!("../../themes/node-graph-presets.v1.json");
let skin = NodeGraphPresetSkinV1::try_new_from_json_str(raw, NodeGraphPresetFamily::WorkflowClean)?;
app.set_global(skin);
```

This keeps the serialized `Graph` clean and lets host apps iterate on visuals by swapping the JSON.

## Notes for Blueprint-style wires

The renderer-level contract already supports:

- solid wires (`Paint::Solid`)
- viewport-stable effects (`PaintEvalSpaceV1::ViewportPx`)
- along-wire gradients (`PaintEvalSpaceV1::StrokeS01`)

For dashed wires + `StrokeS01` gradients, the wgpu backend preserves `StrokeS01` continuity across
dashes (see conformance in `crates/fret-render-wgpu/tests/paint_eval_space_stroke_s01_conformance.rs`).

