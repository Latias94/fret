---
title: Path Stroke Style v2 — Milestones
status: active
date: 2026-02-16
scope: vector path stroke semantics v2
---

# Path Stroke Style v2 — Milestones

This document is a **one-screen milestone board** for v2 vector path stroke semantics.

Source of truth for detailed TODOs: `docs/workstreams/path-stroke-style-v2-todo.md`.
Narrative + design notes: `docs/workstreams/path-stroke-style-v2.md`.

## Definition of done (workstream-level)

We consider “path stroke style v2” shipped when:

1. The contract is explicit and reviewed (ADR added/updated).
2. The default renderer (`fret-render-wgpu`) produces stable output for join/cap/dash across
   multiple scale factors.
3. At least one hard correctness gate exists (GPU readback conformance test).
4. Existing v1 stroke paths (width-only) remain supported (no silent behavior change).

## Milestones

### M0 — Contract lock-in

Acceptance criteria:

- `StrokeStyleV2` exists (bounded fields) without breaking existing v1 `StrokeStyle`.
- `PathStyle::StrokeV2` exists and is handled end-to-end in:
  - `fret-core` contract,
  - renderer path cache keys,
  - and tessellation pipeline.
- ADR locks:
  - join/cap/miter/dash semantics,
  - sanitize rules,
  - deterministic degradations.

Status: Completed.

### M1 — Renderer implementation

Acceptance criteria:

- lyon stroke tessellation maps v2 join/cap/miter correctly.
- dash pattern is implemented deterministically (dash/gap/phase, scale-aware).
- no correctness regression in existing path drawing (fill + v1 stroke paths).

Status: Completed.

### M2 — Conformance gate

Acceptance criteria:

- GPU readback test(s) assert:
  - join/cap shape properties,
  - dash periodicity and phase anchoring,
  - stability across scale factors.

Status: Completed.

### M3 — Adoption (optional)

Acceptance criteria:

- At least one consumer uses v2 style in a visible demo (optional, but recommended to validate API ergonomics).

Status: Completed (ecosystem consumer wired: node graph).
