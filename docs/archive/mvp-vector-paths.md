> Archived: keep for history; prefer ADRs + `docs/roadmap.md` for active decisions.

# MVP: Vector Paths + SVG Icons (GPUI-Aligned)

Status: MVP-PATH-0..2 implemented; SVG alpha-mask+tint implemented; SVG RGBA implemented; Path MSAA+composite implemented

Last updated: 2025-12-26

Owner: Codex (tracking document created per request; does not modify existing MVP queues)

## Why this document exists

This repository has historical MVP planning docs (`docs/archive/mvp.md`, `docs/archive/mvp/active-plan.md`) that are kept for context.
This document is intentionally **separate** to avoid mixing scopes while we explore a larger renderer contract expansion.

Goal: define an incremental, GPUI-inspired path/SVG rendering plan for Fret with clear risks and acceptance checks.

## Reference: what GPUI actually does (baseline we follow)

We “follow the mature upstream” here:

- SVG icons are not GPU path rendering; they are **CPU rasterization to an alpha mask** and then drawn as a tinted sprite.
  - `repo-ref/zed/crates/gpui/src/svg_renderer.rs`
  - `repo-ref/zed/crates/gpui/src/window.rs` (`paint_svg`)
- General vector paths are not exposed as “triangles”; GPUI exposes a `Path` primitive and internally stores per-vertex
  triangle data, produced via `lyon` tessellation.
  - `repo-ref/zed/crates/gpui/src/path_builder.rs`
  - `repo-ref/zed/crates/gpui/src/scene.rs` (`Path`, `PathVertex`)
- GPUI’s “path quality” pipeline includes a dedicated shader path rasterization stage.
  - `repo-ref/zed/crates/gpui/src/platform/blade/shaders.wgsl` (path rasterization + sampling)

## Implementation pointers (Fret)

- Core contract:
  - `crates/fret-core/src/vector_path.rs` (`PathCommand`, `PathService`)
  - `crates/fret-core/src/scene.rs` (`SceneOp::Path`, `SceneOp::MaskImage`)
- Renderer:
  - `crates/fret-render-wgpu/src/renderer/mod.rs` (path tessellation cache, path MSAA intermediate + composite, mask pipeline)
  - `crates/fret-render-wgpu/src/svg.rs` (CPU SVG rasterization + upload helpers)
- UI primitives:
  - `SvgIcon` (tinted alpha-mask icon): `crates/fret-ui/src/element.rs` (`SvgIconProps`), paint in `crates/fret-ui/src/declarative/host_widget/paint.rs`
  - `SceneOp::Path` (used by plots today): `ecosystem/fret-plot/src/retained/canvas/mod.rs`

Non-goal for Fret: introduce an app-facing “offscreen render target + composite” API as the primary vector path solution.
If we use intermediate textures internally for correctness/quality (like GPUI’s path pipeline), that is an implementation
detail inside `fret-render`, not a new UI-level feature.

## Current Fret constraint (today)

`fret-core::SceneOp` now supports:

- `Quad`, `Text`, `Image`/`ImageRegion`, `ViewportSurface`
- `Path` (handle-based; payload lives behind `PathService`)
- `MaskImage` (alpha mask + tint; intended for icons)
- `PushClipRect/PopClip` (scissor stack in `fret-render`)

It still intentionally does **not** expose a “submit triangles/mesh” UI-level API; triangles remain an internal renderer
detail.

## Goals

- Add a **stable, renderer-friendly vector path contract** to `fret-core` that:
  - keeps `fret-core` backend-agnostic (no `wgpu`, no `lyon`),
  - preserves strict operation ordering (ADR 0009),
  - composes with clipping (today: scissor; later: soft/rounded clip).
- Implement the contract in `fret-render` using:
  - CPU tessellation (lyon) initially, with clear caching keys,
  - a quality path that can evolve (MSAA now; analytic coverage / GPUI-like later).
- Provide enough primitives to build:
  - SVG icon rendering (single-color, tintable),
  - plot lines/areas (ImPlot-like basics) without offscreen composition as the primary strategy.

## Non-goals (explicit)

- Full SVG 2.0 support, filters, blur, clip paths, masking, text-on-path.
- Multi-color SVG fidelity as the default icon path.
  - (GPUI’s “tinted alpha mask” model is the baseline; multi-color can be a separate feature later.)
- A general “submit a mesh of arbitrary triangles” API in `fret-ui`.
  - (We can add it later if we have a strong reason, but it expands the core contract surface area significantly.)
- Introducing a new runner-level “vector offscreen compositor” as the mainline path.

## Proposed contract shape (Fret)

### 1) Add a path handle type (core)

Add a new stable ID:

- `PathId` in `crates/fret-core/src/ids.rs`

### 2) Path payload ownership: avoid per-frame scene-local storage

Important Fret-specific constraint:

- Fret’s `SceneOp` is small and `Copy`.
- Fret’s UI paint cache replays `Vec<SceneOp>` slices across frames.

That means a design where `SceneOp` references scene-local path payloads (stored inside `SceneRecording`) would break
paint-cache replay, because those payloads would not exist in the next frame’s `Scene`.

So for Fret, the path payload must be owned by a **stable service/registry** (similar to text):

- UI/components describe a path (commands + style) to a `PathService`.
- The service returns a stable `PathId` (renderer-owned).
- `SceneOp` records only the `PathId` + placement + color.

This keeps `SceneOp` replayable and avoids embedding big arrays in the display list.

### 3) Minimal path description payload (for `PathService::prepare`)

MVP1 payload should support plots and simple icons:

- Command stream:
  - `MoveTo`, `LineTo`, `Close`
- (MVP2+) add:
  - `QuadTo`, `CubicTo`, `ArcTo` (or allow conversion at the builder level)
- Per-path metadata:
  - `bounds` (optional but strongly recommended for culling and caching)

### 4) Minimal draw op

The display list should stay minimal and renderer-friendly:

- `SceneOp::Path { order, origin, path: PathId, color }`

Stroke/fill style is part of `PathId` (prepared by `PathService`), similar to how `TextBlobId` encodes
constraints/typography at prepare time.

## Implementation plan (incremental milestones)

### MVP-PATH-0: Contract prep (core-only)

Deliverables:

- `PathId` added to `fret-core`.
- `PathService` + minimal path command/types added to `fret-core` (renderer-agnostic).
- `SceneOp` gains `Path` (uses only `PathId` + placement + color).
- Scene fingerprinting includes `PathId` + placement deterministically; correctness assumes `PathService` reuses `PathId`
  for identical (commands + style + scale-factor) inputs instead of allocating new IDs every frame.

Acceptance:

- `cargo check --workspace` passes.
- Renderer can accept `PathService` calls (stub) but may ignore `SceneOp::Path` drawing for now.
  - (This is consistent with “contract-first” delivery; rendering lands in MVP-PATH-1.)

### MVP-PATH-1: Renderer “tessellate + draw triangles” (no special AA)

Deliverables:

- `fret-render`:
  - Convert `ScenePath` → tessellated triangle list (CPU) using `lyon_tessellation`.
  - Add a simple pipeline that draws the triangles with premultiplied alpha.
  - Preserve strict op ordering (paths must interleave with quads/text/images correctly).
  - Respect scissor clip stack.
- `fret-demo`:
  - Minimal plot panel: polyline + filled area under curve + axes text.

Acceptance:

- Demo renders plots correctly under clipping and z-order.
- No catastrophic perf regressions on a “reasonable” point count (define a baseline, e.g. 10k points).

### MVP-PATH-2: Caching + scale-factor correctness

Deliverables:

- Renderer cache keyed by:
  - path command stream hash + command length,
  - stroke/fill style,
  - scale factor (tessellation tolerance is scale-dependent).
- Basic culling using path bounds + current clip rect.

Acceptance:

- Repainting without path changes does not re-tessellate.
- Dragging/resizing windows does not explode allocations.

### MVP-PATH-3: SVG icon (GPUI-style alpha mask + tint)

Deliverables (implemented in a non-atlas form):

- `SceneOp::MaskImage` + mask shader pipeline (tint on GPU).
- `fret-render::SvgRenderer::render_alpha_mask` (CPU `usvg+resvg`), `upload_alpha_mask` helper.

Acceptance (met for MVP):

- Icons are tintable via theme colors.
- Quality is stable across DPI changes via `SMOOTH_SVG_SCALE_FACTOR = 2`.

### MVP-PATH-4: Quality upgrades (current direction)

We follow GPUI’s “intermediate + composite” strategy where it exists, but keep it internal to the renderer.

Option A (pragmatic, implemented): MSAA for path pass + composite the premultiplied intermediate texture into the main pass.

Option B (GPUI-like, future): analytic coverage path rasterization stage (per-vertex signed-distance/coverage) and a sampling pass.

Acceptance:

- Plot lines do not shimmer badly during scroll/animation.
- Corners/joins look acceptable for editor UI scale.

## Risks (what can go wrong, and how we mitigate)

1) Ordering correctness regressions (hard contract)
   - Risk: implementing paths in a separate pass breaks ADR 0009 interleaving.
   - Mitigation: ensure the renderer’s submission model supports interleaving; if an internal intermediate texture is
     introduced, it must still be driven by the same ordered op stream (stateful “flush points”).

2) CPU tessellation cost and jank
   - Risk: plots can generate very large paths; naive per-frame tessellation will hitch.
   - Mitigation: caching keys + incremental updates; cap point counts; provide downsampling helpers at the component layer.

3) Memory pressure / allocation churn
   - Risk: per-frame vertex buffers and caches can balloon; multi-window multiplies the cost.
   - Mitigation: reuse buffers, explicit cache budgets (ADR 0004), metrics in debug HUD.

4) Clip interactions (scissor now, soft/rounded later)
   - Risk: scissor-only clipping is insufficient for rounded clip semantics and can cause visible artifacts.
   - Mitigation: MVPs assume rect scissor. Do not “fake rounded clip” at the component layer; keep it an explicit
     renderer evolution (ADR 0030/0019).

5) DPI + transforms (future ADR 0019)
   - Risk: when transforms are added, tessellation quality and AA behavior can change dramatically.
   - Mitigation: keep the contract transform-agnostic for MVP; include scale factor in cache keys; design for a future
     “state stack” integration point rather than baking assumptions into component code.

6) API surface explosion in `fret-core`
   - Risk: adding too many stroke/fill knobs early locks us into semantics we regret.
   - Mitigation: start with the smallest plot-driven subset; expand only with an explicit “semantics lock” note (ADR or
     a dedicated section here).

7) `UiServices` method name collisions (`prepare`/`measure`/`release`)
   - Risk: `TextService` and `PathService` share method names, which makes direct calls on `&mut dyn UiServices`
     ambiguous in Rust.
   - Mitigation: call text operations via `services.text().*` and path operations via `services.path().*` (inherent
     methods on `dyn UiServices`).

## Progress tracker (edit in place)

- [x] MVP-PATH-0: core contract scaffolding
- [x] MVP-PATH-1: renderer triangle pipeline + demo plot
- [x] MVP-PATH-2: caching + bounds culling
- [x] MVP-PATH-3: SVG alpha-mask icons + tint
- [x] MVP-PATH-4: quality upgrade (Option A: MSAA + composite)
