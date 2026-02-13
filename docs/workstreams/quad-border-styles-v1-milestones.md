---
title: Quad Border Styles v1 — Milestones
status: draft
date: 2026-02-13
scope: dashed borders, scene contract, renderer conformance, shadcn parity
---

# Quad Border Styles v1 — Milestones

This document is a **one-screen milestone board** for dashed border support.

Source of truth for detailed TODOs: `docs/workstreams/quad-border-styles-v1-todo.md`.
Narrative + contracts: `docs/workstreams/quad-border-styles-v1.md`.

## Definition of done (workstream-level)

We consider “dashed borders” shipped when:

1. The contract is explicit and reviewed (ADR updated/added, linked from this workstream).
2. The renderer produces stable dashed borders across scale factors and transforms (within the
   contract’s defined behavior).
3. There is at least one hard regression gate (renderer readback test and/or diag script).
4. shadcn parity: `border-dashed` outcomes are visibly dashed in UI Gallery.

## Milestones

### M0 — Contract lock-in

Acceptance criteria:

- Decision recorded: `SceneOp::StrokeRRect` vs “extend `SceneOp::Quad`”.
- Dash semantics are locked (pattern model, parameterization, phase anchoring, snapping rules).
- API is future-proofed for extension (prefer `#[non_exhaustive]` on new enums/types).

Status: Not started.

### M1 — Renderer implementation (dashed stroke works)

Acceptance criteria:

- `fret-render-wgpu` supports dashed border masking for rounded rect strokes.
- A conformance test reads back pixels and asserts periodicity/stability at multiple scale factors.
- No performance cliff for typical UI scenes (dash fields are cheap when disabled).

Status: Not started.

### M2 — shadcn parity wiring (`border-dashed` is real)

Acceptance criteria:

- `fret-ui-shadcn` maps `border-dashed` usages to the mechanism capability.
- UI Gallery shows at least one “tasks-style” dashed border control.
- A regression gate exists for parity (scripted or renderer-level).

Status: Not started.

### M3 — Marching ants (optional, editor UX)

Acceptance criteria:

- A demo exists that animates dash phase deterministically.
- A small invariant gate exists (phase update determinism).

Status: Not started.

