# Canvas Draw Region Gap Audit v1

Date: 2026-05-28

## Truth

- ProgressIndicator and Slider need automation evidence for painted subparts, not just component
  roots.
- Exact per-draw-op names cannot be represented today because `SceneOp` is anonymous.
- Material recipe anchors are acceptable only when they match deterministic rectangular geometry.
- Circular arcs and animated canvas segments must remain covered by scene/golden proof until a
  generic scene diagnostics contract exists.

## Artifacts

- `ecosystem/fret-ui-material3/src/progress_indicator.rs`: linear/circular progress canvas paint.
- `ecosystem/fret-ui-material3/src/slider.rs`: slider/range slider canvas paint.
- `crates/fret-ui/src/canvas.rs`: canvas painter exposes scene access, hosted-resource caches, and
  scene fragments, but no named draw-region output.
- `crates/fret-core/src/scene/mod.rs`: `SceneOp` variants carry geometry/paint data but no label.
- `ecosystem/fret-bootstrap/src/ui_diagnostics`: snapshots export scene counts/fingerprints,
  semantics/test-id bounds, and paint hotspots, not named scene-op regions.

## Owner Classification

- Mechanism gap: exact named canvas/scene draw regions.
- Material foundation gap: reusable hidden diagnostic anchor helpers for rectangular recipe parts.
- Recipe gap: ProgressIndicator and Slider do not yet stamp part anchors for canvas-painted
  rectangular regions.
- Diagnostics script gap: UI Gallery scripts can only target ids that recipe/foundation surfaces
  expose.

## Proof Plan

- Add a Material foundation helper for layout-only diagnostic anchors.
- Use focused `automation_surface` tests to prove anchors exist.
- Keep `material3_headless_progress_indicator_suite_goldens_v1` and
  `material3_headless_slider_suite_goldens_v1` as exact scene-output gates.
- Split a `crates/*` follow-on only if the repo needs true named `SceneOp` regions across multiple
  design systems.

## Residual Risk

- Hidden layout anchors are not a replacement for renderer-aware named scene ops.
- Axis-aligned anchors cannot truthfully describe circular arcs, rotations, or clipped animated
  bars.
- Adding too many anchors could pollute the semantics/debug surface, so this lane should keep the
  first slice deliberately narrow.
