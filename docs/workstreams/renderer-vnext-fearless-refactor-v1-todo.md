# Renderer vNext Fearless Refactor v1 — TODO Tracker

Status: Draft (workstream tracker)

This document tracks TODOs for:

- `docs/workstreams/renderer-vnext-fearless-refactor-v1.md`
- `docs/workstreams/renderer-vnext-fearless-refactor-v1-milestones.md`

Tracking format:

- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked
- ID: `REN-VNEXT-{area}-{nnn}`

When completing an item, prefer leaving 1–3 evidence anchors:

- file paths + key functions/tests
- and/or a `fretboard diag` script/suite name

## M0 — Design baseline

- [ ] REN-VNEXT-docs-001 Add a short “invariants checklist” appendix for renderer refactors (what must never change).
- [x] REN-VNEXT-adr-001 Draft ADR: isolated opacity / saveLayer(alpha) (group alpha).
- [x] REN-VNEXT-adr-002 Draft ADR: clip path + image mask sources (bounded, cacheable, deterministic).
- [x] REN-VNEXT-adr-003 Draft ADR: paint/material portability closure (capabilities + fallbacks + conformance gates).
- [x] REN-VNEXT-guard-000 Record the workstream’s baseline gate set and a baseline perf/telemetry snapshot (linkable).

## M1 — RenderPlan compilation substrate

- [ ] REN-VNEXT-plan-001 Define the internal RenderPlan IR (segments, sequence points, state snapshots).
- [ ] REN-VNEXT-plan-002 Move budget/degradation decisions into plan compilation (deterministic ordering).
- [ ] REN-VNEXT-plan-003 Add telemetry hooks: per-window intermediate peak bytes and degradations applied.
- [ ] REN-VNEXT-plan-004 Introduce a switch to run old vs new paths and compare results for a small fixed scene set.

## M2 — Isolated opacity (saveLayerAlpha)

- [x] REN-VNEXT-alpha-001 Decide contract shape: extend `CompositeGroupDesc` vs add a dedicated opacity group op.
- [x] REN-VNEXT-alpha-002 Add a GPU conformance test for isolated alpha vs non-isolated alpha mismatch cases.

## M3 — ClipPath + image masks (bounded + cacheable)

### M3a — ClipPath v1

- [ ] REN-VNEXT-clip-001 Decide v1 clip-path contract shape (prepared path handle vs dedicated clip handle).
- [ ] REN-VNEXT-clip-003 Add conformance tests:
  - clip-before-transform scrolling cases,
  - clip under rotation (affine),
  - nested clips + clips + groups.

### M3b — Image masks v1

- [ ] REN-VNEXT-clip-002 Decide image-mask v1 sampling semantics (minimal enum, deterministic degradation).
- [ ] REN-VNEXT-mask-001 Add conformance tests for nested masks + groups and paint-only hit-testing invariants.

## M4 — Paint/Material evolution (controlled extensibility)

### M4a — Capability matrix + deterministic fallbacks

- [ ] REN-VNEXT-paint-001 Inventory where `Paint` is supported vs missing (quad/path/stroke/mask).
- [ ] REN-VNEXT-paint-002 Decide whether `SceneOp::Path` should accept `Paint` in v1/v2 (or remain solid-only).
- [ ] REN-VNEXT-mat-001 Document the renderer’s MaterialId capability matrix and deterministic fallbacks for wasm/mobile.
- [ ] REN-VNEXT-mat-002 Fill the capability matrix table with concrete “Must/May/Degrade” decisions per target.
- [ ] REN-VNEXT-mat-003 Add at least one conformance scene for `Paint::Material` fallback behavior (unsupported registration, missing id, and budget pressure).

### M4b — Optional contract expansion

- [ ] REN-VNEXT-paint-010 If `Path` accepts `Paint`, add a conformance gate for gradient/material path fills.

## M5 — Sampling hints (bounded state surface)

- [ ] REN-VNEXT-samp-001 Decide where sampling hints live (image op, viewport op, or material).
- [ ] REN-VNEXT-samp-002 Add a small conformance scene that exercises nearest/linear on mixed primitives without reordering.

## Always-run guardrails (before/after each milestone)

- [ ] REN-VNEXT-guard-001 Keep `python3 tools/check_layering.py` green for all intermediate steps.
- [ ] REN-VNEXT-guard-002 Add/extend at least one renderer conformance test per new contract.
- [ ] REN-VNEXT-guard-003 Record a perf snapshot baseline and keep “worst bundles” attachable to milestones.
