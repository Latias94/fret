---
title: Path Stroke Style v2 — TODO
status: active
date: 2026-02-16
---

# Path Stroke Style v2 — TODO Tracker

Status: Active (workstream tracker)

Workstream narrative: `docs/workstreams/path-stroke-style-v2/path-stroke-style-v2.md`
Milestone board: `docs/workstreams/path-stroke-style-v2/path-stroke-style-v2-milestones.md`

## Tracking format

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `PSSV2-{area}-{nnn}`

Leave 1–3 evidence anchors when completing an item (paths + key functions/tests), and prefer
renderer conformance tests for correctness-sensitive semantics.

## M0 — Contract design (bounded + portable)

- [x] PSSV2-contract-010 Decide the contract surface (recommended):
  - Add `StrokeStyleV2` (join/cap/miter/dash) without breaking existing `StrokeStyle`.
  - Add `PathStyle::StrokeV2(StrokeStyleV2)` while keeping `PathStyle::Stroke(StrokeStyle { width })` supported.
  - Evidence: updated types in `crates/fret-core/src/vector_path.rs` + updated renderer path cache key.

- [x] PSSV2-contract-020 Define deterministic dash semantics for vector path strokes:
  - Reuse `DashPatternV1` semantics (dash/gap/phase; logical px; no fitting).
  - Reuse the exact `DashPatternV1` type (shared across stroke-like primitives).
  - Evidence: `crates/fret-core/src/vector_path.rs` (`StrokeStyleV2.dash: Option<DashPatternV1>`), `docs/adr/0277-path-stroke-style-v2.md` (“Dash pattern”).

- [x] PSSV2-contract-030 Define transform semantics explicitly:
  - Non-uniform transform deformation is expected and must be deterministic (no hidden backend “corrections”).
  - “Constant pixel width” is out-of-scope for v2 (explicitly deferred).
  - Evidence: `docs/adr/0277-path-stroke-style-v2.md` (“Transforms”).

- [x] PSSV2-adr-040 Draft an ADR that locks v2 stroke semantics and fallback policy:
  - join/cap/miter/dash rules
  - sanitize/clamp behavior for non-finite values
  - deterministic degradations
  - Draft: `docs/adr/0277-path-stroke-style-v2.md`

## M1 — Core plumbing (cache keys + sanitize)

- [x] PSSV2-core-100 Extend path cache key mixing to include v2 style fields:
  - join/cap/miter limit/dash
  - ensure stable hashing and no float NaN divergence (sanitize before hashing)
  - Note: dash fields are part of the cache key once dash segmentation is implemented (required for determinism).
  - Evidence: `crates/fret-render-wgpu/src/renderer/path.rs` (`mix_path_style`, `path_cache_key`)

- [ ] PSSV2-core-110 Update any callsites that want v2 strokes (keep v1 working):
  - prefer leaving existing v1 callsites untouched unless a v2 feature is needed

## M2 — Renderer implementation (wgpu default)

 - [x] PSSV2-render-200 Implement v2 stroke tessellation via lyon:
   - map join/cap/miter to lyon `StrokeOptions`
   - implement dash by segmenting the path pre-tessellation (bounded, deterministic)
   - Evidence: `crates/fret-render-wgpu/src/renderer/path.rs` (`build_dashed_lyon_path`, `tessellate_path_commands`)

 - [x] PSSV2-render-210 Add a conformance test (GPU readback) for:
   - join rendering stability across scale factors (miter vs bevel corner coverage)
   - cap coverage and dashed stroke periodicity / phase anchoring
   - Evidence: `crates/fret-render-wgpu/tests/path_stroke_style_v2_conformance.rs`

- [ ] PSSV2-render-220 Add a “no perf cliff” check:
  - ensure v2 fields are zero-cost when not used (no extra allocations on v1 path)
  - only add a perf gate if evidence shows a regression

## M3 — Adoption (optional / follow-up)

- [x] PSSV2-adopt-300 Wire at least one real consumer to use v2 join/cap/dash:
  - pick a small, isolated demo surface (plot/node graph/canvas) to validate ergonomics.
  - Evidence: `ecosystem/fret-node/src/ui/canvas/paint.rs` (edge wire paths use `PathStyle::StrokeV2` with join/cap), `ecosystem/fret-node/src/ui/canvas/widget/tests/cached_edges_tile_equivalence_conformance.rs` (accepts v2 style).
