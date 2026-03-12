---
title: Quad Border Styles v1 (Dashed Borders) — Workstream
status: draft
date: 2026-02-13
scope: fret-core scene ops, fret-ui container chrome, fret-render-wgpu quad shader, shadcn parity
---

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.

# Quad Border Styles v1 (Dashed Borders) — Workstream

This workstream adds **first-class dashed borders** to Fret’s default rendering stack, with two
drivers:

1. Editor-grade UX primitives (selection rectangles, drop-zone highlights, drag previews).
2. shadcn/ui parity (`border-dashed` is used by upstream examples, including tasks).

Detailed TODOs live in: `docs/workstreams/quad-border-styles-v1/quad-border-styles-v1-todo.md`.
Milestone board (one-screen): `docs/workstreams/quad-border-styles-v1/quad-border-styles-v1-milestones.md`.

## Upstream / reference anchors

- shadcn/ui v4 tasks faceted filter uses `border-dashed` on the filter trigger:
  - `repo-ref/ui/apps/v4/app/(app)/examples/tasks/components/data-table-faceted-filter.tsx`
- Existing shadcn parity tests already recognize `border-dashed` as a token, but Fret currently
  renders it as a solid border:
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_empty.rs`
  - `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/empty.rs`

## Current state (2026-02)

- `SceneOp::Quad` supports rounded rect fill + **solid** per-edge borders (inside alignment).
  - Contract: `crates/fret-core/src/scene/mod.rs`
  - UI mapping: `crates/fret-ui/src/declarative/host_widget/paint.rs`
  - Renderer: `crates/fret-render-wgpu/src/renderer/shaders.rs` (analytic SDF)
- Dashed borders are explicitly deferred in ADR 0030 (P0) due to high entropy:
  - `docs/adr/0030-shape-rendering-and-sdf-semantics.md` (“Dashed borders are deferred”)

This workstream is the “P1+ option” ADR 0030 leaves open: implement dashed borders as a renderer
feature, **but** lock semantics up-front to avoid churn.

## Non-negotiables (future-proofing)

To keep this change “fearless refactor” friendly and reduce future rework, we must lock:

- **Dash semantics** (perimeter mode vs per-edge restart).
- **Phase anchoring** rules (stable under resize; no perimeter-fit adjustments).
- **Rounded corner behavior** (continuous perimeter; no corner special-casing that creates seams).
- **Per-edge widths** behavior (what happens when border widths differ per edge).
- **Scale factor** rule (logical px vs physical px).
- **Transform interaction** (dash stability under transforms, especially non-uniform scale).
- **Pixel snapping** interaction (when `snap_to_device_pixels` is enabled).
- **Clip/mask interaction** (dashes must respect clip stacks; no leaking).

## Design options (contract-level)

ADR 0030 prefers a stroke primitive rather than bloating `Quad`. Two viable designs:

### Option A — Extend `SceneOp::Quad` with `border_style`

Pros:

- Single op for fill + border outcomes.
- Minimal UI-layer churn.

Cons:

- Makes `Quad` a “kitchen sink” over time (solid + dashed + dotted + …).
- Harder to later unify with path strokes (joins/caps/dashes) without semantic duplication.

### Option B (recommended) — Add a dedicated `SceneOp::StrokeRRect`

Key idea:

- Keep `SceneOp::Quad` semantics stable.
- Add a stroke op that can be encoded through the **same quad pipeline** (same SDF math), but with:
  - fill always transparent,
  - border coverage computed the same way as `Quad`,
  - optional dash mask applied to border coverage.

Pros:

- Aligns with ADR 0030’s “stroke semantics live with stroke” guidance.
- Easier future extension path:
  - `StrokeRRect` → `StrokePath` (joins/caps/dashes) without turning `Quad` into a stroke API.
- Allows UI authoring to stay simple: `ContainerProps` can map dashed borders to “Quad fill + StrokeRRect”.

Cons:

- Still requires a `SceneOp` contract change (new variant).
- Requires renderer + encode + tests work in a single wave.

Recommendation: **Option B**.

## Proposed v1 semantics (to lock)

### Contract sketch (recommended)

The intent is to avoid inventing a general path stroke API prematurely, while still creating a
reusable “stroke semantics” nucleus:

- `fret-core`:
  - `DashPatternV1 { dash: Px, gap: Px, phase: Px }`
  - `StrokeStyleV1 { dash: Option<DashPatternV1> }` (reserve room for future joins/caps)
  - `SceneOp::StrokeRRect { order, rect, corner_radii, stroke: Edges, stroke_paint: Paint, style: StrokeStyleV1 }`
- `fret-ui`:
  - `ContainerProps` gains an **opt-in** dashed border request (e.g. `border_dash: Option<DashPatternV1>`).
  - When enabled, host-widget painting emits:
    1) a normal `Quad` for fill (with border suppressed), then
    2) a `StrokeRRect` for the dashed border.

This keeps `SceneOp::Quad` semantics unchanged while still supporting the “filled control with
dashed outline” outcome needed by shadcn recipes.

### Dash model

- Pattern: `dash_px`, `gap_px`, `phase_px`.
- Units: defined in **logical px**, scaled by `scale_factor` (equivalently “scale-aware logical px”).
- No “perimeter fitting” (no attempts to evenly divide the perimeter).

### Perimeter parameterization

- Perimeter-continuous (not per-edge restart).
- Anchor: start at the rrect’s **top edge**, at the point `(x + r_tl, y)`.
- Direction: clockwise.
- Rounded corners use quarter-arc lengths (`(π/2) * r`) to keep continuity.

Rationale:

- Matches typical “single border loop” expectation (no discontinuities at corners).
- Stable under resize because the start anchor is stable and there is no perimeter fitting.

### Border widths and corner radii

- Border coverage stays “inside-aligned” (matches current quad shader semantics).
- Dash mask is applied to the **border coverage** after it is computed (i.e. it gates border alpha).
- Per-edge border widths remain supported (dash parameterization follows the outer perimeter; inner
  radii continue to be derived from adjacent border widths as today).

### Pixel snapping

- When the UI opts into device-pixel snapping, dash evaluation uses the snapped rect bounds so the
  dash pattern does not “swim” relative to the pixels.

### Transforms

- Dash evaluation is based on the local rrect perimeter coordinate before transform.
- Under non-uniform transforms, dashes visually deform along with the shape (expected and stable).

## Why we should also consider “marching ants”

Editor workflows often need animated dashed borders (selection rectangles). The above contract
supports this via `phase_px` updates, without any dedicated animation system.

TODO: add a small demo + diag/test gate that validates phase-driven updates remain deterministic.

## Related future render semantics (out of scope for v1, but plan for them)

This change touches border semantics, so we should explicitly inventory adjacent needs to avoid
painting ourselves into a corner:

- **Dotted borders** (`border-dotted`) as a dash-derived mode or a dedicated dot mask.
- **Double borders** (`border-double`) as two inside strokes with split widths.
- **Stroke joins/caps/dashes for paths** (charts/editor vector tools):
  - see ADR 0080 follow-ups: `docs/adr/0080-vector-path-contract.md`
- **Outside/center stroke alignment** (ADR 0030 open question #1).
- **Border image / patterned borders** (likely never in P0/P1; keep separate).

The v1 contract should keep room for extension (prefer `#[non_exhaustive]` on new enums/types, and
avoid UI-facing APIs that assume “only dashed exists”).

## Evidence / regression gates (required)

We consider dashed borders “landed” only if there are hard gates:

- A renderer conformance test that renders a dashed rrect and validates periodic coverage with GPU
  readback (`crates/fret-render-wgpu/tests/*` style).
- At least one shadcn parity demo page that exercises `border-dashed` in UI Gallery (tasks-style
  controls), plus a small scripted interaction gate if applicable.

## Implementation notes (to keep the refactor honest)

- Renderer batching: `StrokeRRect` should encode through the same quad instance batch as `Quad`
  whenever possible, so we do not introduce an additional hot path for UI scenes.
- Shader approach: reuse current border coverage (outer/inner SDF) and gate it with a `dash_mask`.
  Anti-alias dash edges using derivative-aware smoothing (similar to the existing SDF AA strategy).
- Perimeter coordinate (`s`) for a rounded rect should be derived from the closest point on the
  outer perimeter, partitioned into:
  - top/right/bottom/left straight segments, plus
  - four quarter-arc segments.
  This keeps corner continuity and makes behavior deterministic.
