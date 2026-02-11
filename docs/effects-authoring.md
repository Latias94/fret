# Effects Authoring (EffectLayer)

This document describes how to author scoped UI post-processing effects using the portable effect semantics
defined in `docs/adr/0117-effect-layers-and-backdrop-filters-scene-semantics-v1.md`.

For the recommended user-facing story (Tier A vs Tier B) and the recipe authoring pattern, see:
`docs/adr/0134-effect-recipes-and-tier-selection-v1.md`.

## TL;DR

- Use `fret-ui`'s declarative `EffectLayer` wrapper to emit `SceneOp::PushEffect/PopEffect` around a subtree.
- `bounds` is a **computation bound**, not an implicit clip; use the existing clip stack (e.g. overflow clip + rounded corners)
  to define clipping behavior (ADR 0078).
- For heavy GPU panels (video playback, game viewports, NLE-class effects), prefer Tier A (`RenderTargetId` + `SceneOp::ViewportSurface`)
  instead of trying to expose `wgpu` to components (ADR 0123).
  - Declarative helper: `cx.viewport_surface(...)` (`crates/fret-ui/src/elements/cx.rs`)

## Scheduling and timers (avoid split-brain)

Keep timing-driven behavior aligned with Fret's runner-owned scheduling model:

- UI-visible timers/animation ticks SHOULD be scheduled via effects (e.g. `Effect::SetTimer`, `Effect::RequestAnimationFrame`) so they participate in the Effect pipeline and runner flush points (ADR 0034, ADR 0110).
- The execution `Dispatcher` may expose a low-level `dispatch_after` primitive, but this is primarily for executor utilities/harnesses; user-facing UI timing should not depend on runner-specific scheduling hooks directly (ADR 0175).

## Declarative usage (recommended)

`EffectLayer` lives in `crates/fret-ui` and is available via `ElementContext`:

```rust
let chain = fret_core::EffectChain::from_steps(&[
    fret_core::EffectStep::GaussianBlur { radius_px: Px(6.0), downsample: 2 },
    fret_core::EffectStep::ColorAdjust { saturation: 1.1, brightness: 1.0, contrast: 1.0 },
]);

cx.effect_layer(
    fret_core::EffectMode::Backdrop,
    chain,
    |cx| {
        let mut panel = fret_ui::ContainerProps::default();
        panel.layout.overflow = fret_ui::Overflow::Clip;
        panel.corner_radii = fret_core::Corners::all(Px(12.0));
        vec![cx.container(panel, |_| Vec::new())]
    },
);
```

Notes:

- `EffectMode::Backdrop` samples the already-rendered content behind the effect boundary.
- `EffectMode::FilterContent` renders the subtree to an offscreen intermediate and filters that output.
- Use `EffectLayerProps.quality` (`Auto/Low/Medium/High`) when you need predictable trade-offs; otherwise keep `Auto` and
  rely on budgets + deterministic degradation (ADR 0118).

## Glass recipe template (fret-ui-kit)

For a concrete, token-driven recipe example (suitable as a copy/paste template for your own custom recipes), see:

- Token/resolve/clamp helpers: `ecosystem/fret-ui-kit/src/recipes/glass.rs`
- Declarative wrapper (feature-gated): `ecosystem/fret-ui-kit/src/declarative/glass.rs`

The `glass_panel` helper is compiled behind the `fret-ui-kit` `recipes` feature.

## Pixelate recipe template (fret-ui-kit)

- Token/resolve/clamp helpers: `ecosystem/fret-ui-kit/src/recipes/pixelate.rs`
- Declarative wrapper (feature-gated): `ecosystem/fret-ui-kit/src/declarative/pixelate.rs`

The `pixelate_panel` helper is compiled behind the `fret-ui-kit` `recipes` feature.

## Pixelate semantics (what to expect)

Fret's current pixelation implementation is a **nearest**-based downsample/upscale chain that is **anchored to the
effect bounds** (not the window origin). This is intentional: it makes nested/translated effects deterministic and
keeps the "pixel grid" stable within the effect region.

Practical implications:

- Pixelate is not a blur: high-frequency patterns (1–2 px stripes, checkerboards) will alias as `scale` changes.
- Some `scale` values can make thin patterns appear to disappear or turn into a flat tint, depending on how the block
  origin lines up with the underlying pattern.
- If you need a more predictable look across arbitrary content, prefer blur/color-adjust steps, or author content that
  matches the target `scale` (pixel-art inputs, deliberate block sizes).

## Token naming conventions (recommended)

Fret theme tokens are plain string keys (ADR 0050) and can be extended with dotted namespaces.

Recommendations:

- Use a stable dotted namespace per recipe/component: `component.<recipe>.<field>`.
- Keep app-/plugin-owned tokens under your own namespace (e.g. `acme.*`) and pass them via `*TokenKeys` when needed.
- Prefer `Theme::color_by_key(...)` / `Theme::metric_by_key(...)` reads with explicit fallbacks and clamping.
- Avoid introducing implicit unit semantics: treat metrics as `Px` values (even when they represent "scale" or "downsample").

Current `fret-ui-kit` effect recipe token keys:

- Glass chrome:
  - `component.glass.padding_x`, `component.glass.padding_y`
  - `component.glass.radius`, `component.glass.border_width`
  - `component.glass.tint`, `component.glass.border`
- Glass effect:
  - `component.glass.blur_radius_px`, `component.glass.blur_downsample`
  - `component.glass.saturation`, `component.glass.brightness`, `component.glass.contrast`
- Pixelate chrome:
  - `component.pixelate.padding_x`, `component.pixelate.padding_y`
  - `component.pixelate.radius`, `component.pixelate.border_width`
  - `component.pixelate.bg`, `component.pixelate.border`
- Pixelate effect:
  - `component.pixelate.scale`

## Minimal app-side token override example

Theme overrides are applied by building a `ThemeConfig` (JSON) and calling `Theme::apply_config(...)`.
Colors support `#RRGGBB` / `#RRGGBBAA` and also `hsl(...)` / `oklch(...)` (see parser in `crates/fret-ui/src/theme.rs`).

Example `theme.json`:

```json
{
  "name": "MyTheme",
  "colors": {
    "component.glass.tint": "#FFFFFF99",
    "component.glass.border": "#FFFFFF33"
  },
  "metrics": {
    "component.glass.blur_radius_px": 18.0,
    "component.glass.blur_downsample": 2.0,
    "component.pixelate.scale": 12.0
  }
}
```

Example apply code (early in app init):

```rust
let cfg = fret_ui::ThemeConfig::from_slice(include_bytes!("theme.json"))?;
fret_ui::Theme::with_global_mut(app, |theme| theme.apply_config(&cfg));
```

Per-instance overrides (when you want multiple variants in one app) can be done by passing custom keys:

```rust
use fret_ui_kit::declarative::glass::{glass_panel, GlassPanelProps};
use fret_ui_kit::recipes::glass::{GlassEffectTokenKeys, GlassTokenKeys};

let props = GlassPanelProps {
    chrome_keys: GlassTokenKeys { tint: Some("acme.glass.tint"), ..GlassTokenKeys::none() },
    effect_keys: GlassEffectTokenKeys { blur_radius_px: Some("acme.glass.blur_radius_px"), ..GlassEffectTokenKeys::none() },
    ..Default::default()
};
let _ = glass_panel(cx, props, |_| Vec::new());
```

## When to use Tier A instead (video / viewport / NLE-class)

If your "component" wants to do substantial rendering work (custom shaders, video decoding, engine viewports):

- Render into a texture registered as `RenderTargetId`.
- Present it in the UI via `SceneOp::ViewportSurface` (or `cx.viewport_surface(...)` in declarative trees).
- Keep `wgpu::Device/Queue` ownership centralized in the runner (ADR 0038).

If you also need to forward pointer + wheel input into the viewport (e.g. engine panels), use
`fret-ui-kit`'s `viewport_surface_panel` helper: `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs`.

This keeps the UI contracts deterministic and portable, while still enabling advanced GPU workloads.

## Debugging and profiling

- Renderer perf snapshots (including optional per-pipeline breakdown):
  - `FRET_RENDERER_PERF_PIPELINES=1`
  - `FRET_EFFECTS_DEMO_PROFILE=1 FRET_EFFECTS_DEMO_EXIT_AFTER_FRAMES=600 cargo run -p fret-demo --bin effects_demo`
- RenderDoc workflow: `docs/renderdoc-inspection.md`
- Practical checklist: `docs/debugging-playbook.md`
