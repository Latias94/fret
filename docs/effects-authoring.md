# Effects Authoring (EffectLayer)

This document describes how to author scoped UI post-processing effects using the portable effect semantics
defined in `docs/adr/0119-effect-layers-and-backdrop-filters-scene-semantics-v1.md`.

## TL;DR

- Use `fret-ui`'s declarative `EffectLayer` wrapper to emit `SceneOp::PushEffect/PopEffect` around a subtree.
- `bounds` is a **computation bound**, not an implicit clip; use the existing clip stack (e.g. overflow clip + rounded corners)
  to define clipping behavior (ADR 0078).
- For heavy GPU panels (video playback, game viewports, NLE-class effects), prefer Tier A (`RenderTargetId` + `SceneOp::ViewportSurface`)
  instead of trying to expose `wgpu` to components (ADR 0125).

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
  rely on budgets + deterministic degradation (ADR 0120).

## When to use Tier A instead (video / viewport / NLE-class)

If your "component" wants to do substantial rendering work (custom shaders, video decoding, engine viewports):

- Render into a texture registered as `RenderTargetId`.
- Present it in the UI via `SceneOp::ViewportSurface`.
- Keep `wgpu::Device/Queue` ownership centralized in the runner (ADR 0038).

This keeps the UI contracts deterministic and portable, while still enabling advanced GPU workloads.

## Debugging and profiling

- Renderer perf snapshots (including optional per-pipeline breakdown):
  - `FRET_RENDERER_PERF_PIPELINES=1`
  - `FRET_EFFECTS_DEMO_PROFILE=1 FRET_EFFECTS_DEMO_EXIT_AFTER_FRAMES=600 cargo run -p fret-demo --bin effects_demo`
- RenderDoc workflow: `docs/renderdoc-inspection.md`
- Practical checklist: `docs/debugging-playbook.md`

