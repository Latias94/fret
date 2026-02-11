# ADR 0133: Gizmo Picking Primitives and Precision Policy

Status: Proposed

## Context

Fret’s gizmo rendering strategy is intentionally engine-pass oriented (depth-tested 3D geometry)
rather than an immediate-mode UI overlay (ADR 0130). Input forwarding at the viewport boundary is
explicit-units and includes mapping geometry + scale factor (ADR 0132).

Today, `ecosystem/fret-gizmo` performs picking largely from the same analytic geometry used to draw
handles (axis shafts, plane quads, rings, etc) and resolves overlaps using a score policy
(`GizmoPickPolicy`). This is already robust for a small, fixed set of built-in handles and is
covered by invariants tests.

However, “mature editor” gizmo systems (Godot, Unreal, Unity) tend to converge on an additional
conceptual boundary:

- **Draw geometry**: what the user sees (often stylized, thickened, LOD’d, depth-mode dependent).
- **Pick geometry**: what the user interacts with (often simplified, stable, and policy-driven).

Godot is an explicit reference here: editor gizmos register collision segments/meshes and maintain a
BVH for interaction rather than relying on render topology alone (`repo-ref/godot/editor/scene/3d/*`).

Separately, transform-gizmo emphasizes numeric robustness and uses double-precision math internally.
Fret currently uses `glam` f32 types for all gizmo computations. For very large worlds or extreme
camera distances, f32 drift can cause subtle “feel” issues (jitter, asymmetry when returning to the
start, unstable picking thresholds).

## Goals

1. Introduce a first-class **picking primitive** surface that is decoupled from draw topology.
2. Keep the core transform manipulator “feel” stable across:
   - DPI / scale factor,
   - viewport fit modes (`Contain`/`Cover` letterboxing),
   - near-plane and behind-camera edge cases,
   - large-world / high-distance scenarios (where feasible).
3. Enable future “custom gizmo plugins” to provide their own handles with:
   - explicit pick shapes,
   - explicit begin/update/commit/cancel lifecycle (already modeled as `GizmoPhase`).
4. Provide a pragmatic path to optional f64 math without forcing an immediate breaking API change.

## Non-goals

- Defining editor tool policy (shortcut mapping, snapping hotkeys, command routing).
- Replacing engine-pass rendering with a 2D overlay-only approach (ADR 0130 remains the boundary).
- Implementing a general physics/collision system; gizmo picking primitives are scoped and small.

## Decision

### 1) Add a picking-primitive contract in `ecosystem/fret-gizmo`

Add a new internal contract (ecosystem-level, policy-heavy) that describes interactive shapes used
for hit testing. The key point is that these shapes are not required to match draw geometry exactly.

Example surface (illustrative, not final):

```rust
pub enum GizmoPickShape3d {
    Sphere { center: Vec3, radius: f32 },
    Capsule { a: Vec3, b: Vec3, radius: f32 },
    PlaneQuad { origin: Vec3, u: Vec3, v: Vec3, thickness: f32 },
    CircleBand { center: Vec3, normal: Vec3, radius: f32, half_thickness: f32 },
    Aabb { min: Vec3, max: Vec3 },
}

pub struct GizmoPickItem {
    pub handle: HandleId,
    pub kind: GizmoMode,
    pub shape: GizmoPickShape3d,
    /// A score bias (lower wins) to encode “intent” resolution (e.g. tip beats shaft).
    pub bias: f32,
}

pub struct GizmoPickHit {
    pub handle: HandleId,
    pub kind: GizmoMode,
    /// A comparable score (smaller is better), typically derived in screen pixels.
    pub score: f32,
}
```

Rules:

- Picking returns a **stable score** comparable across shapes and modes.
- Pick radii/thickness are expressed in **screen pixels** and scaled via
  `ViewportInputEvent.geometry.pixels_per_point` (ADR 0132).
- Dragging uses the **raw cursor** (unclamped) projected into target space; legacy/clamped coords
  remain available for policies that want them.

### 2) Use pick primitives as the source of truth for built-in gizmo picking

Migrate `Gizmo::pick_*` paths to build a small `Vec<GizmoPickItem>` describing the current gizmo’s
interactive surface, then resolve the best hit via a shared picker.

This reduces “feel” coupling between:

- visual thickness and AA choices, and
- hit testing thresholds.

It also provides a direct extensibility point: custom tools can add pick items without “faking”
draw geometry.

### 3) Optional acceleration structure (BVH) is permitted but not required

The built-in transform gizmo has a small, fixed number of handles, so a linear scan is acceptable.
The contract should allow future use of a BVH/AABB tree for plugin-heavy scenes without changing
the public editor API.

### 4) Establish a precision policy with an f64 internal option

Adopt a precision policy that keeps the public surface stable but allows internal f64 math for
robustness:

- Public API remains `glam::{Mat4, Vec2, Vec3, Quat}` (f32) for now.
- Internal computations for picking and drag projection may opt into f64 by converting to
  `glam::{DMat4, DVec2, DVec3, DQuat}` and converting results back to f32.
- Introduce an internal type alias layer (for staged refactors):

```rust
#[cfg(feature = "gizmo-f64")]
type S = f64;
#[cfg(not(feature = "gizmo-f64"))]
type S = f32;
```

Constraints:

- The `gizmo-f64` feature must remain optional and not introduce new third-party dependencies
  beyond `glam` (already in use).
- The f64 path should first target the most sensitive operations:
  - ray/plane intersections,
  - line/axis closest-point computations,
  - angle accumulation for rotation rings,
  - pixel-to-world scale derivations.

## Migration Plan

1. Add `GizmoPickShape3d` / `GizmoPickItem` / `GizmoPickHit` surfaces in `fret-gizmo`.
2. Migrate built-in picking to use primitives for one mode at a time (Translate -> Rotate -> Scale).
3. Add regression tests for:
   - letterboxed viewport input mapping,
   - “return to start” invariants under extreme camera parameters,
   - large coordinate magnitudes (synthetic).
4. Introduce the optional `gizmo-f64` feature and migrate the most sensitive math first.
5. (Future) Add a “custom gizmo plugin” API that can contribute pick primitives + draw lists.

## Consequences

Pros:

- Clear separation between visuals and interaction → more predictable “editor feel”.
- Enables Godot-like extensibility (custom handles) without hacking draw topology.
- Provides a path to large-world robustness with an incremental, opt-in f64 strategy.

Cons:

- Adds a new conceptual layer (pick primitives) that must be maintained.
- Potentially increases code surface area; requires careful tests to avoid regressions.

## Related ADRs

- ADR 0130: viewport gizmos engine-pass and UI overlay boundary
- ADR 0132: viewport input forwarding explicit units
- ADR 0049: viewport tools input capture and overlays
- ADR 0127 / ADR 0024: undo/redo and edit transaction direction (gizmo phases)

