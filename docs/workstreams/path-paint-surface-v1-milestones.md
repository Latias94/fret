---
title: Path Paint Surface v1 — Milestones
status: active
date: 2026-02-16
scope: SceneOp::Path accepts Paint
---

# Path Paint Surface v1 — Milestones

This document is a **one-screen milestone board** for making `SceneOp::Path` support `Paint`.

Source of truth for detailed TODOs: `docs/workstreams/path-paint-surface-v1-todo.md`.

## Definition of done (workstream-level)

We consider “path paint surface v1” shipped when:

1. The contract is explicit and reviewed (ADR added/updated).
2. The default renderer (`fret-render-wgpu`) can render path fills with:
   - `Paint::Solid`
   - `Paint::LinearGradient`
   - `Paint::RadialGradient` (optional for v1, but recommended)
   with deterministic degradations for unsupported variants.
3. At least one hard correctness gate exists (GPU readback conformance test).
4. Existing solid-color path users remain supported (no silent breakage beyond the contract change).

## Milestones

### M0 — Contract lock-in

Acceptance criteria:

- `SceneOp::Path` accepts `Paint`.
- ADR locks the coordinate semantics and fallback/degradation policy.

Status: Done.

### M1 — Renderer implementation

Acceptance criteria:

- wgpu path pipeline evaluates paint correctly in path local space.
- material paint is capability-gated and degrades deterministically.

Status: Done.

### M2 — Conformance gate

Acceptance criteria:

- GPU readback test(s) assert expected gradient properties across scale factors.

Status: Done.

### M3 — Adoption (optional)

Acceptance criteria:

- At least one consumer uses non-solid `Paint` on paths to validate ergonomics.

Status: Not started.
