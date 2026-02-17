---
title: Text Paint Surface v1 — Milestones
status: active
date: 2026-02-17
scope: SceneOp::Text accepts Paint
---

# Text Paint Surface v1 — Milestones

This document is a **one-screen milestone board** for making `SceneOp::Text` support `Paint`.

Source of truth for detailed TODOs: `docs/workstreams/text-paint-surface-v1-todo.md`.

## Definition of done (workstream-level)

We consider “text paint surface v1” shipped when:

1. The contract is explicit and reviewed (ADR added/updated).
2. The default renderer (`fret-render-wgpu`) can render text with:
   - `Paint::Solid`
   - `Paint::LinearGradient`
   with deterministic degradations for unsupported variants.
3. At least one hard correctness gate exists (GPU readback conformance test).
4. Existing solid-color text users remain supported (no silent breakage beyond the contract change).

## Milestones

### M0 — Contract lock-in

Acceptance criteria:

- `SceneOp::Text` accepts `Paint`.
- ADR locks the coordinate semantics and fallback/degradation policy.

Status: Landed.

### M1 — Renderer implementation

Acceptance criteria:

- wgpu text pipeline evaluates paint correctly in logical text-local space.
- material paint is capability-gated and degrades deterministically.

Status: Landed (wgpu default).

### M2 — Conformance gate

Acceptance criteria:

- GPU readback test(s) assert expected gradient properties across scale factors.

Status: Landed (GPU readback).

### M3 — Adoption (optional)

Acceptance criteria:

- At least one consumer uses non-solid `Paint` on text to validate ergonomics.

Status: Landed (ui-gallery probe).

### M4 — Text shadow (bounded) v1 (optional)

Acceptance criteria:

- `SceneOp::Text` supports an optional `TextShadowV1` (single layer, no blur).
- A conformance gate verifies shadow ordering and basic rendering behavior.

Status: Landed (single-layer shadow + conformance).
