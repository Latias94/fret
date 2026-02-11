# ADR 0140: Custom Gizmo Plugins and Handle Contract

Status: Proposed

## Context

Fret already supports an editor-grade 3D transform manipulator in `ecosystem/fret-gizmo`, with:

- engine-pass (depth-tested) rendering as the primary topology (ADR 0130),
- explicit viewport input units and mapping (ADR 0132),
- explicit pick primitives and overlap policy (ADR 0133),
- begin/update/commit/cancel phases suitable for undo coalescing (ADR 0024 direction).

However, a game engine editor must go far beyond the core transform manipulator. Mature editors
(Godot, Unreal, Unity) expose a *custom gizmo surface* so that tools and domain objects can provide
their own interactive handles:

- lights (cone angle + range),
- cameras (frustum + near/far planes),
- audio emitters (radius),
- physics volumes (shape extents),
- skeleton/bone manipulators,
- navigation volumes and constraints,
- editor-only debug widgets.

Godot is a strong reference: `EditorNode3DGizmoPlugin` separates rendering from interaction and
maintains dedicated collision geometry (segments/meshes) for picking, with an internal BVH for
performance.

Without a first-class extensibility contract, Fret risks accumulating bespoke per-tool glue code:
each tool reinvents picking shapes, handle IDs, overlap resolution, and interaction lifecycles.

## Goals

1. Provide a stable, ecosystem-level contract for **custom gizmo plugins** to contribute:
   - draw geometry (what users see),
   - pick geometry (what users interact with),
   - an explicit begin/update/commit/cancel interaction lifecycle.
2. Keep the boundary “mechanism-only in core crates; tool policy in ecosystem/app” (ADR 0027).
3. Preserve the engine-pass vs UI overlay separation (ADR 0130):
   - depth-tested 3D gizmos are not encoded as UI `SceneOp` primitives.
4. Ensure handle identity is stable and composable across plugins (undo keys, selection mapping).
5. Provide a performance path for “many gizmos in one viewport” without locking into a heavy physics
   dependency (linear scan is acceptable at small scale; BVH is an optional future).

## Non-goals

- Defining editor hotkeys, snapping conventions, or command routing (app scope).
- Committing to a full editor plugin system (loading/unloading, sandboxing, asset packaging).
- Implementing a general collision/physics system.
- Forcing a breaking change to `ecosystem/fret-gizmo`’s existing `Gizmo` API.

## Decision

### 1) Introduce a `GizmoPlugin` contract (ecosystem-level)

Add a new, policy-heavy surface in `ecosystem/fret-gizmo` (or an adjacent ecosystem crate) that
allows editor/app code to register “gizmo plugins” that can contribute per-frame draw + pick data,
and optionally handle interaction updates.

Illustrative shape:

```rust
pub struct GizmoPluginId(pub u32);

pub struct GizmoPluginContext<'a> {
    pub view_projection: glam::Mat4,
    pub viewport: ViewportRect,
    pub depth_range: DepthRange,
    pub pixels_per_point: f32,
    pub pick_radius_px: f32,
    pub time_seconds: f32,
    pub user_precision: f32,
    pub snap_enabled: bool,
}

pub struct GizmoPluginFrame {
    pub draw: GizmoDrawList3d,
    pub pick: Vec<GizmoPickItem>,
}

pub trait GizmoPlugin {
    fn plugin_id(&self) -> GizmoPluginId;

    fn build_frame(
        &mut self,
        ctx: &GizmoPluginContext<'_>,
        targets: &[GizmoTarget3d],
        out: &mut GizmoPluginFrame,
    );

    fn update_interaction(
        &mut self,
        ctx: &GizmoPluginContext<'_>,
        phase: GizmoPhase,
        active_handle: HandleId,
        targets: &[GizmoTarget3d],
    ) -> Option<GizmoUpdate>;
}
```

Notes:

- This API is intentionally “data-first”: it deals in `GizmoTarget3d`, stable IDs, and draw/pick
  primitives, not engine objects.
- The `update_interaction` method is optional and may be omitted by “purely visual” plugins.
- The built-in transform manipulator (`Gizmo`) may later be wrapped as an implementation of
  `GizmoPlugin` to validate the contract and unify code paths.

### 2) Standardize pick geometry as explicit primitives

Plugins must provide **pick geometry** explicitly as a list of items, separate from draw geometry.

The v1 contract uses screen-space primitives (already proven stable for editor feel), and may be
extended to world-space primitives later.

Illustrative shape:

```rust
pub enum GizmoPickShape {
    ScreenCircle(PickCircle2d),
    ScreenCapsule(PickSegmentCapsule2d),
    ScreenConvexQuad(PickConvexQuad2d),
}

pub struct GizmoPickItem {
    pub handle: HandleId,
    pub kind: GizmoMode,
    pub shape: GizmoPickShape,
    /// Lower wins. Bias is used to encode intent rules (tip beats shaft, plane beats axis when inside).
    pub bias: f32,
}
```

Picking resolution remains based on a stable, comparable score (pixels or a monotonic proxy), and
overlap resolution is policy-driven (`GizmoPickPolicy` for transform gizmos; a similar policy can
be provided for plugins).

### 3) Handle identity is namespaced and stable

`HandleId(u64)` remains the public handle identifier type.

To avoid collisions between plugins, we standardize a namespacing scheme:

- upper 32 bits: `GizmoPluginId` (namespace),
- lower 32 bits: plugin-local handle ID.

We will add helper constructors and accessors (additive API):

```rust
impl HandleId {
    pub const fn from_parts(plugin: GizmoPluginId, local: u32) -> Self { /* ... */ }
    pub const fn plugin(self) -> GizmoPluginId { /* ... */ }
    pub const fn local(self) -> u32 { /* ... */ }
}
```

Built-in gizmos reserve a well-known namespace (e.g. `GizmoPluginId(0)`).

### 4) Interaction lifecycle is explicit and compatible with undo/redo

The contract uses `GizmoPhase::{Begin,Update,Commit,Cancel}` as the interaction lifecycle boundary.

Editor apps should map these phases into undo transactions (ADR 0024):

- begin transaction on `Begin`,
- coalesce intermediate updates on `Update`,
- finalize on `Commit`,
- revert/abort on `Cancel`.

### 5) Rendering topology stays consistent with ADR 0130

Plugins output `GizmoDrawList3d` in world space with `DepthMode::{Test,Ghost,Always}`.

The editor/engine decides how to render these primitives:

- in-engine (preferred, depth-tested, matching viewport pipeline),
- or via an engine-side immediate overlay helper (as used by demos).

UI overlays (text labels, HUD readouts) remain out of this contract and should be rendered via the
UI layer above the viewport surface (ADR 0130). A future extension may add an optional
“viewport-anchored UI overlay contribution” surface, but it should remain separate from 3D draw
lists.

### 6) Performance and future extensibility

The contract is designed so that:

- small handle counts can be resolved with a linear scan,
- large plugin-heavy scenes can later introduce an optional acceleration structure (BVH/AABB tree)
  without changing the public plugin API.

Similarly, the pick/math layer may adopt an internal f64 path (ADR 0133) without changing the public
plugin signatures.

## Migration plan

1. Define the `GizmoPlugin` trait and the `GizmoPickItem` surface.
2. Provide a small “plugin manager” helper that:
   - collects pick items from all plugins,
   - resolves a single hovered hit,
   - maintains the active handle lock during drags,
   - routes phases to the active plugin.
3. Wrap the built-in transform gizmo (`Gizmo`) as a plugin to validate the contract.
4. Add a first custom plugin in the demo (e.g. light radius handle) to validate “domain gizmo”
   ergonomics.

## Alternatives considered

1. **No plugin contract; keep per-tool bespoke gizmos**
   - Pros: minimal surface area now.
   - Cons: duplication, drift in UX feel, inconsistent picking/lifecycle semantics across tools.
2. **Encode 3D gizmo geometry as UI `SceneOp` primitives**
   - Rejected by ADR 0130 (breaks depth-tested rendering quality and engine pipeline integration).
3. **World-space collision-only picking (segments/meshes) as v1**
   - Pros: closer to Godot’s architecture.
   - Cons: higher complexity, requires robust ray intersection + optional BVH earlier than needed.
   - Decision: start with screen-space primitives; add world-space primitives as a follow-up when
     a plugin needs it (e.g. dense meshes).

## Consequences

Pros:

- Enables Godot/Unreal/Unity-style extensibility for domain gizmos.
- Stabilizes editor “feel” by reusing a shared picking and lifecycle model.
- Keeps core Fret crates unchanged while enabling powerful editor behavior in ecosystem/app layers.

Cons:

- Adds a new contract that must be maintained and tested.
- Requires careful handle ID namespacing rules to avoid collisions.

## Related ADRs

- ADR 0130: Viewport gizmos — engine-pass rendering and UI overlay boundary
- ADR 0132: Viewport input forwarding — explicit units
- ADR 0133: Gizmo picking primitives and precision policy
- ADR 0049: Viewport tools — input capture and overlays (editor layer)
- ADR 0024: Undo/redo and edit transactions (editor layer)
- ADR 0127: Undo/redo infrastructure boundary

## Open questions

- Should v1 include world-space pick primitives (sphere/capsule/OBB) or keep them as a follow-up ADR?
- Do we need a shared “style tokens for gizmos” vocabulary across plugins (beyond `GizmoVisuals`)?
- How should plugin-provided pick items be scored consistently against transform gizmo items when
  both are present (cross-tool overlap policy)?
