# ADR 0048: Inspector Property Protocol and Custom Editor Registry


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Godot: https://github.com/godotengine/godot

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
Status: Accepted
Scope: Example editor layer (out of scope for the Fret framework)

## Context

Editor-grade workflows (Unity/Unreal/Godot-class) depend heavily on a productive **Inspector**:

- show a structured property tree for the current selection (single or multi-select),
- edit values with appropriate controls (sliders, dropdowns, color pickers, etc.),
- support custom editors for complex types and engine-specific components,
- integrate cleanly with undo/redo and “dirty” tracking.

Fyrox demonstrates that “reflection path + property changed events + command stack” yields a highly
productive editor workflow:

- Command pattern foundation (undo/redo, coalescing, significant vs insignificant).
- Plugin MVC discipline (UI does not own business state; sync-to-model after commands).

Note: the Fyrox reference checkout is optional; see `docs/repo-ref.md` for how to clone it into `repo-ref/`.

Godot demonstrates a scalable “custom inspector plugin stack” where later-registered plugins can
override default editors:

- Inspector plugin chain patterns:
  - `repo-ref/godot/editor/inspector/editor_inspector.cpp`

Fret must keep the **framework vs editor** boundary clean (ADR 0027) and keep plugins renderer-free
(ADR 0016), while still making it easy to build a real inspector on top.

## Decision

Define an **engine-agnostic inspector protocol** for the example editor layer:

1) a stable **Property Path** format to identify fields in an object graph,
2) a minimal **Property Tree** schema (groups + leaf values + metadata),
3) a data-first **Property Edit** event that can be converted into editor commands/transactions,
4) a **Custom Editor Registry** model (plugin chain / last-registered-wins) to choose editors.

This ADR does **not** require any specific reflection system.
An engine can provide an adapter that implements this protocol using:

- Rust reflection (Serde/Any/custom), ECS component metadata, schema DSL, or hand-written bindings.

### 1) Property path is stable and serializable

A property is identified by a stable path:

- `PropertyPath = Vec<PropertyPathSegment>`

Where segments are:

- `Field(String)` — a named field (e.g. `transform`, `position`, `x`)
- `Index(u32)` — an array/vector index
- `Key(String)` — a map/dictionary key (stringified for portability)

Rules:

- Paths must be stable across frames for the same object schema.
- Paths are **IDs**, not UI labels. Labels are presentation metadata.
- Paths may be used as stable keys for UI caching and editor customization.

### 2) Property tree schema

The inspector renders a tree of nodes derived from a selection:

- group nodes (foldable sections)
- leaf nodes (editable values)

Each leaf provides:

- `path: PropertyPath`
- `label: String` (UI-facing)
- `type_tag: PropertyTypeTag` (engine-agnostic identifier for editor selection)
- `value: PropertyValue`
- `meta: PropertyMeta` (read-only, range, step, enum choices, units, doc, etc.)

Multi-selection:

- A leaf may carry `PropertyValue::Mixed` when selected objects differ.
- An edit to a `Mixed` value applies the new value to all compatible targets.

### 3) Property edits are data-first “requests”

Inspector UI does not mutate engine/editor models directly.
Instead, it emits a request:

- `PropertyEdit { selection: SelectionId, path: PropertyPath, value: PropertyValue, kind: EditKind }`

Where `EditKind` supports:

- `Begin` / `Update` / `Commit` / `Cancel` for drag-like interactions (sliders, gizmos),
  enabling undo coalescing boundaries (ADR 0024).

The example editor layer converts `PropertyEdit` into editor-owned commands/transactions, then calls
the canonical `sync_to_model` step to refresh UI state (Fyrox-style).

### 4) Custom editor registry (plugin chain)

Define a registry that resolves which editor widget to use for a property leaf.

Resolution order:

1. Registered custom editors for `(type_tag, path-prefix)` pairs (most specific first).
2. Registered custom editors for `type_tag` (last registered wins for ties).
3. Built-in fallback editors for `PropertyValue` kinds (bool, number, string, enum).

Notes:

- This models Godot’s “inspector plugin stack” and is compatible with Fret’s plugin boundaries
  (ADR 0016) because it is app/editor-layer policy.
- The registry API must be renderer-free and should only construct UI nodes/widgets.

## Consequences

- The inspector becomes buildable without coupling Fret to any reflection/ECS implementation.
- Editor plugins can extend the inspector by registering editors, without forking UI internals.
- Undo/redo integration is straightforward: property edits become explicit operations with clear
  coalescing boundaries (ADR 0024).

## Future Work

- Define a minimal `fret-components-inspector` crate (or example editor module) that implements:
  - property tree rendering,
  - editor registry,
  - basic editors (bool/int/float/string/enum/color/vecN),
  - multi-selection `Mixed` UX.
- Add a reference adapter for a simple demo model (non-engine) to validate the protocol early.
- Decide how to encode richer type tags (e.g. namespaced strings vs UUIDs).
- Decide optional validation/error reporting channel for failed edits.

## Implementation Notes (Current Prototype)

- Protocol types + editor registry (example editor layer): `apps/fret-editor/src/inspector_protocol.rs`
- Property edit request plumbing (phased edits): `apps/fret-editor/src/property_edit.rs`
- Inspector edit UI surface (example editor layer widgets): `apps/fret-editor/src/inspector_edit.rs`
- Demo wiring status: no dedicated inspector demo harness is kept stable yet; if needed, integrate into `fret-demo --bin components_gallery` and treat the demo entrypoint as evolving.

Phased edit validation:

- Demo Inspector supports **Alt + drag** on numeric (`f32`) rows to emit `Begin/Update/Commit` and create a single undo entry.
- Press `Esc` during the drag to emit `Cancel` (reverts the value and restores the scene dirty flag).

## References

- Framework/editor boundary: `docs/adr/0027-framework-scope-and-responsibilities.md`
- Plugin boundaries: `docs/adr/0016-plugin-and-panel-boundaries.md`
- Undo/redo transactions (deferred): `docs/adr/0024-undo-redo-and-edit-transactions.md`
- Fyrox command stack + plugin MVC (optional reference checkout; see `docs/repo-ref.md`).
- Godot inspector plugin chain:
  - `repo-ref/godot/editor/inspector/editor_inspector.cpp`
