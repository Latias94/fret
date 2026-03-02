# Wire paint cookbook (v1)

This cookbook documents **paint-only** wire styling recipes for `ecosystem/fret-node` using:

- `EdgePaintOverrideV1` (ADR 0309)
- `PaintBindingV1` + `PaintEvalSpaceV1` (ADR 0306)

The key contract is that these recipes:

- do **not** mutate the serialized `Graph`
- do **not** invalidate derived geometry/hit-testing
- only affect emitted `SceneOp` paint + paint caches

## Recipe 1: Solid wire

Use this for the baseline “data wire” / “exec wire” looks.

```rust
overrides.set_edge_override(edge_id, Some(EdgePaintOverrideV1 {
    stroke_paint: Some(Color::from_srgb_hex_rgb(0x6b_72_80).into()),
    stroke_width_mul: None,
    dash: None,
}));
```

## Recipe 2: Dashed wire (preview / invalid / emphasis)

Use this to express non-semantic state (preview connection, invalid connection, “convertible”).

```rust
overrides.set_edge_override(edge_id, Some(EdgePaintOverrideV1 {
    stroke_paint: Some(Color::from_srgb_hex_rgb(0xff_bf_24).into()),
    stroke_width_mul: Some(1.2),
    dash: Some(DashPatternV1::new(Px(8.0), Px(4.0), Px(0.0))),
}));
```

## Deferred (v2): Along-wire gradient (Blueprint / ShaderGraph style)

Use `PaintEvalSpaceV1::StrokeS01` and author the gradient in the `(s01, 0)` domain:

- `start=(0,0)`
- `end=(1,0)`

```rust
let paint = PaintBindingV1::with_eval_space(
    Paint::LinearGradient(LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(1.0), Px(0.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: 2,
        stops: [
            GradientStop::new(0.0, Color::from_srgb_hex_rgb(0x22_d3_ee)),
            GradientStop::new(1.0, Color::from_srgb_hex_rgb(0xf4_72_b6)),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
        ],
    }),
    PaintEvalSpaceV1::StrokeS01,
);

overrides.set_edge_override(edge_id, Some(EdgePaintOverrideV1 {
    stroke_paint: Some(paint),
    stroke_width_mul: Some(1.4),
    dash: None,
}));
```

Notes:

- Dashed `StrokeV2` preserves `StrokeS01` continuity on wgpu (conformance:
  `crates/fret-render-wgpu/tests/paint_eval_space_stroke_s01_conformance.rs`).
- Edge markers reuse wire paint bindings except when the wire uses `StrokeS01` (policy: markers
  fall back to the resolved solid color because marker stroke parameterization is unrelated).

## Deferred (v2): Viewport-fixed highlight (shimmer / emphasis)

Use `PaintEvalSpaceV1::ViewportPx` when you want a highlight that stays stable on screen (not
sticking to the wire’s local geometry).

```rust
let paint = PaintBindingV1::with_eval_space(
    Paint::LinearGradient(LinearGradient {
        start: Point::new(Px(0.0), Px(0.0)),
        end: Point::new(Px(0.0), Px(720.0)),
        tile_mode: TileMode::Clamp,
        color_space: ColorSpace::Srgb,
        stop_count: 2,
        stops: [
            GradientStop::new(0.0, Color::from_srgb_hex_rgb(0x4a_de_80)),
            GradientStop::new(1.0, Color::from_srgb_hex_rgb(0x38_bd_f8)),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
            GradientStop::new(0.0, Color::TRANSPARENT),
        ],
    }),
    PaintEvalSpaceV1::ViewportPx,
);

overrides.set_edge_override(edge_id, Some(EdgePaintOverrideV1 {
    stroke_paint: Some(paint),
    stroke_width_mul: Some(1.6),
    dash: None,
}));
```

## Demo / diagnostics harness

The node graph demo has an opt-in cookbook mode:

- set `FRET_NODE_GRAPH_DEMO_WIRE_PAINT_COOKBOOK=1`
- optionally set `FRET_NODE_GRAPH_DEMO_PRESET_JSON_PATH` to load your own preset JSON

Today this harness only exercises the v1 recipes (solid + dashed). The gradient recipes are
documented as v2 to avoid coupling visual policy too early.

Diagnostics:

- Registry id: `cookbook-node-graph-wire-paint-cookbook-screenshots`
- Requires `FRET_NODE_GRAPH_DEMO_WIRE_PAINT_COOKBOOK=1` in the demo environment.
