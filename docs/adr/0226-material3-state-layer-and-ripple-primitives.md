# ADR 0226: Material 3 State Layer + Ripple Primitives (Mechanism-Level)

Status: Proposed

## Context

Material 3 interaction feedback is defined primarily through:

- **State layers** (hover/pressed/focus/dragged): semi-transparent overlays with prescribed opacity.
- **Ripples**: transient ink reactions with bounded/unbounded variants and motion rules.

Fret already provides mechanism-level primitives for:

- focus rings (ADR 0061)
- shadows/elevation (ADR 0060)
- overlay roots + dismissal/focus-scope policy boundary (ADR 0067)

However, without a mechanism-level primitive for state layers and ripple drawing, every component
crate will re-implement scene emission, leading to drift and inconsistent performance.

## Decision

### 1) State layer primitive

Add a renderer-friendly, low-level primitive to paint a state layer:

- Input: bounds, corner radii, color, opacity
- Output: one or more `SceneOp` operations

The primitive is paint-only by default and should integrate with the interactivity pseudoclass
contract (ADR 0166): hover/pressed transitions must not require structural changes.

### 2) Ripple primitive (drawing only)

Provide a mechanism-level ripple drawing primitive that can be driven by ecosystem-owned policy:

- Input: bounds, origin point, progress, color/opacity, bounded/unbounded
- Output: scene ops representing the ink circle (or equivalent)

The runtime does not own the ripple state machine; that remains a component/policy concern.

## Consequences

- Material 3 and other design systems can share a consistent interaction feedback substrate.
- Components remain cache-friendly and structurally stable while still showing rich interaction
  feedback.

