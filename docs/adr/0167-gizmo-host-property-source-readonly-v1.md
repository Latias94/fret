# ADR 0167: Gizmo Host Property Source (Read-only) v1

Status: Proposed

## Context

Fret's gizmo plugin surface (ADR 0155) allows tools to contribute custom handles and emit domain
edits via `GizmoUpdate.custom_edits` (`GizmoCustomEdit` keyed by `GizmoPropertyKey`). This works well
for writing changes, but **plugins currently have no stable way to read the host's current domain
values**.

As a result, integrations tend to fall into one of two patterns:

1. **Host-push caching**: the host pushes values into the plugin via plugin-specific methods.
   This is brittle:
   - the plugin risks diverging from the host model,
   - multi-target edits require bespoke glue,
   - undo/redo "restore" values must be tracked out-of-band.
2. **Opaque edits only**: the plugin emits `custom_edits` but cannot display correct HUD values or
   compute stable drag-start snapshots when the host value changes externally.

Godot is a useful reference point: `EditorNode3DGizmoPlugin` supports a full handle lifecycle with
explicit `get_handle_value` (read), `set_handle` (interactive change), and `commit_handle` (undo/redo
commit with `p_restore`) (`repo-ref/godot/editor/scene/3d/node_3d_editor_gizmos.h`).

Zed/GPUI provides a complementary boundary model: the UI runtime does not own domain state; updates
flow through controlled entry points (`Entity::update`, `Context::notify`) while domain logic
remains app-owned (`repo-ref/zed/crates/gpui/src/app/entity_map.rs`,
`repo-ref/zed/crates/gpui/src/app/context.rs`).

Fret's intent is aligned with these references:

- `ecosystem/fret-gizmo` remains a **mechanism-level** crate (host-driven, backend-agnostic).
- Undo/redo and transaction policy remains **host/app-owned** (ADR 0024 / ADR 0136 direction).
- Viewport tools remain **engine-pass** (ADR 0139) and input is forwarded explicitly (ADR 0147).

What is missing is a small, stable, and portable **read-only "property source" contract** that a
host can implement and pass to gizmo plugins.

## Goals

1. Allow gizmo plugins to query **current host values** needed for correct rendering and
   interaction (for example, scalar radii, angles, limits).
2. Keep `ecosystem/fret-gizmo` portable: no dependency on a specific scene/ECS/inspector model.
3. Keep undo/redo policy host-owned: the property source is read-only and does not own
   transactions.
4. Make multi-target editing ergonomic (query per `GizmoTargetId`).

## Non-goals

- Defining a general inspector property protocol or schema (see ADR 0048).
- Providing a write API for plugins to mutate host state directly.
- Implementing 3D picking acceleration / BVH (tracked separately; see ADR 0155 gaps).

## Decision

### Introduce a read-only `GizmoPropertySource` trait (ecosystem-level)

Add a host-implemented trait in `ecosystem/fret-gizmo` that supports reading a small set of
primitive property types keyed by:

- `GizmoTargetId` (which target/entity the property belongs to),
- `GizmoPropertyKey` (plugin-scoped property key, already used by `GizmoCustomEdit`).

v1 is intentionally minimal and starts with scalar reads:

```rust
pub trait GizmoPropertySource {
    fn read_scalar(
        &self,
        target: GizmoTargetId,
        key: GizmoPropertyKey,
    ) -> Option<f32>;
}
```

Future versions may add additional typed reads (e.g. `Vec2`, `Vec3`, `Quat`) if required by real
plugins. v1 explicitly avoids a `Variant`/`Any`-style API to keep the contract allocation-free and
portable.

### Pass the property source through the plugin context

Extend `GizmoPluginContext` to include an optional property source reference:

- `properties: Option<&dyn GizmoPropertySource>`

This keeps the core `Gizmo` surface unchanged and scopes the contract to plugins (ADR 0155).

### Host-side undo/redo integration stays explicit

The host remains responsible for applying `GizmoCustomEdit`s to its model and recording undo/redo.

Recommended host ordering on `GizmoPhase::Begin`:

1. Call `plugin_manager.update(...)` and obtain a `GizmoUpdate` containing `custom_edits`.
2. Before applying edits to the domain model, snapshot restore values for each `(target, key)` via
   `GizmoPropertySource::read_scalar(...)`.
3. Apply the edits (and coalesce subsequent updates until `Commit`).

This mirrors Godot's `p_restore` concept without coupling `fret-gizmo` to a specific undo API.

## Consequences

- Plugins can render correct HUD/readouts without host-specific setter APIs.
- Plugins can capture stable drag-start snapshots even if host values change externally.
- Hosts gain a uniform place to integrate undo restore snapshots for custom edits.
- The contract is deliberately narrow; some plugins may still need richer types in a future v2.

## Alternatives considered

1. **Host-specific "push" APIs per plugin**
   - Rejected: scales poorly, encourages divergent caches, and fragments the ecosystem.
2. **Opaque `Any`/`Variant` value reads**
   - Rejected: introduces allocations and type ambiguity, and makes portability harder.
3. **Write-capable property source**
   - Deferred: blurs transaction/validation responsibility and is harder to make safe. A future ADR
     can introduce a write path if we need it, guided by real integrations.

## Follow-ups (expected)

1. Implement `GizmoPropertySource` in `ecosystem/fret-gizmo` and thread it through
   `GizmoPluginContext`.
2. Refactor example plugins (e.g. `LightRadiusGizmoPlugin`) to read via the property source and
   remove host-push caches.
3. Add a dedicated follow-up ADR for 3D picking primitives + acceleration (Godot-style collision
   registry + BVH), building on ADR 0155.

