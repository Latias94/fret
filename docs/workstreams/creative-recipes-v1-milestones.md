# Creative Recipes (v1) — Milestones

This file tracks milestone gates for the creative recipes workstream.

See:

- Overview: `docs/workstreams/creative-recipes-v1.md`
- Task list: `docs/workstreams/creative-recipes-v1-todo.md`

## M0 — Recipe foundation (ecosystem-only)

- Introduce a shared “recipe resolve + fallback reporting” shape in `ecosystem/fret-ui-kit`.
- Keep existing public helpers (e.g. glass/pixelate wrappers) stable; migrate internals only.
- Add a minimal diagnostics sink seam for “recipe degraded” events (best-effort).

Status: Landed (partial; catalog skeleton pending)

Evidence:

- `ecosystem/fret-ui-kit/src/recipes/resolve.rs`
- `ecosystem/fret-ui-kit/src/declarative/glass.rs`
- `ecosystem/fret-ui-kit/src/declarative/pixelate.rs`

## M1 — Paint v1 (gradients)

- Land `Paint` and gradient evaluation (ADR 0233) and wire it to `SceneOp::Quad` (and optionally `Path`).
- Add renderer conformance tests for linear/radial mapping and deterministic sanitization.

Status: Landed

Evidence:

- `crates/fret-core/src/scene/paint.rs`
- `crates/fret-core/src/scene/mod.rs` (`SceneOp::Quad` uses `Paint`)
- `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`

## M2 — Materials v1 (params-only)

- Land `MaterialId` + registration API and baseline kinds (ADR 0235).
- Implement determinism rules (explicit seeds, no hidden time; ADR 0244).
- Add renderer telemetry counters and a minimal “procedural paint conformance” test.

Status: Landed

Evidence:

- `crates/fret-core/src/materials.rs`
- `crates/fret-core/src/scene/paint.rs` (`Paint::Material { id, params }`)
- `crates/fret-render-wgpu/src/renderer/services.rs` (`MaterialService`)
- `crates/fret-render-wgpu/tests/materials_conformance.rs`
- `crates/fret-render-wgpu/src/renderer/types.rs` (`RenderPerfSnapshot` material counters)

## M3 — Masks v1 (gradient alpha masks)

- Land `PushMask/PopMask` (ADR 0239) with gradient-only sources.
- Add conformance tests for coverage correctness and clip/effect interaction boundaries.

Status: Landed

Evidence:

- `crates/fret-core/src/scene/mask.rs`
- `crates/fret-core/src/scene/mod.rs` (`SceneOp::PushMask/PopMask`)
- `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs`

## M4 — Compositing groups v1 (blend modes)

- Land `PushCompositeGroup/PopCompositeGroup` (ADR 0247) with a small blend vocabulary.
- Budget intermediates and deterministic degradation.

Status: Landed

Evidence:

- `crates/fret-core/src/scene/composite.rs`
- `crates/fret-core/src/scene/mod.rs` (`SceneOp::PushCompositeGroup/PopCompositeGroup`)
- `crates/fret-render-wgpu/tests/composite_group_conformance.rs`

## M5 — Motion + pointer snapshots

- Land frame clock reads (non-reactive) and pointer motion snapshot seam (ADR 0240 / ADR 0243).
- Add a reduced-motion policy helper in `fret-ui-kit` and verify fallback behavior.

Status: Landed (reduced-motion helper pending)

Evidence:

- `crates/fret-core/src/window.rs` (`WindowFrameClockService`)
- `crates/fret-ui/src/pointer_motion.rs`
- `crates/fret-ui/src/widget.rs` (widget-facing read helpers)

## M6 — Effect steps extension

- Land `ColorMatrix` and `AlphaThreshold` effect steps (ADR 0236) with conformance tests.
- Provide a “bloom-like” Tier B recipe example using threshold + blur + additive composite (depends on M4).

Status: Landed

Evidence:

- `crates/fret-core/src/scene/mod.rs` (`EffectStep::ColorMatrix` / `EffectStep::AlphaThreshold`)
- `crates/fret-core/src/scene/validate.rs`
- `crates/fret-core/src/scene/fingerprint.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs`
- `crates/fret-render-wgpu/src/renderer/render_scene/render.rs`
- `crates/fret-render-wgpu/src/renderer/pipelines/{color_matrix.rs,alpha_threshold.rs}`
- `crates/fret-render-wgpu/src/renderer/shaders.rs`
- `crates/fret-render-wgpu/tests/{effect_color_matrix_conformance.rs,effect_alpha_threshold_conformance.rs}`

## M7 — Sampled materials v2a (catalog textures)

This milestone is the recommended first step for ADR 0242: sampled materials bind a renderer-owned
catalog texture selected at registration time (no per-instance `ImageId` yet).

- Land `BindingShape::ParamsPlusCatalogTexture` in the renderer registry and capability-gate it.
- Ship at least one baked catalog texture (blue-noise/dither) and a sampled baseline material.
- Add a conformance test for sampled materials and deterministic fallback behavior.

Status: Not started

Evidence (planned):

- `docs/adr/0242-sampled-materials-and-fixed-binding-shapes-v2.md`
- `crates/fret-render-wgpu/src/renderer/services.rs` (`MaterialService` binding shapes)
- `crates/fret-render-wgpu/src/renderer/resources.rs` (catalog texture upload/lifetime)
- `crates/fret-render-wgpu/tests/materials_sampled_conformance.rs`

## M8 — `fret-ui-magic` (Phase 0)

- Land `ecosystem/fret-ui-magic` as a MagicUI-aligned wrapper crate.
- Implement 3–5 seed components (Lens/MagicCard/BorderBeam/Marquee/Dock).
- Add UI gallery entries and `fretboard diag` scripts for each seed component.

Status: Not started

Evidence (planned):

- `ecosystem/fret-ui-magic/` (crate surface + recipes)
- `apps/fret-ui-gallery/` (entries)
- `tools/diag-scripts/` (scripts)

## M9 — External texture imports (v1)

- Land a “contract-path demo” for imported GPU textures wired to `ViewportSurface` (ADR 0234).
- Add at least one capability-gated real backend path (web or native) plus a clear copy/zero-copy policy.

Status: Not started

Evidence (planned):

- `docs/adr/0234-imported-render-targets-and-external-texture-imports-v1.md`
- `crates/fret-render-wgpu/src/renderer/render_targets.rs` / `ImportedViewportRenderTarget`
- `apps/fret-demo*` or `apps/fretboard` (contract-path demo)
- `docs/workstreams/diag-extensibility-and-capabilities-v1/capabilities.md` (cap gating notes)
