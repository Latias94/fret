# ADR 0013: Docking Operations, Stable Panel Identity, and Layout Persistence

Status: Accepted

## Context

To match Unity/Unreal/ImGui-class editor UX, docking must support:

- arbitrary splits and tab stacks,
- tear-off and re-dock across windows,
- drag previews and drop-zone hints,
- persistence across restarts,
- plugins registering dockable panels.

If docking state is not modeled as a stable, pure data structure with well-defined operations,
it becomes difficult to:

- serialize/deserialize layouts,
- implement undo/redo,
- extend docking behavior without UI-layer entanglement.

References:

- Dear ImGui docking + viewports (interaction vocabulary and extensibility expectations):
  - https://github.com/ocornut/imgui
- Godot dock manager implementation (useful for persistence + UI affordances):
  - `repo-ref/godot/editor/docks/editor_dock_manager.cpp`

## Decision

### 1) Dock graph is pure data

`DockGraph` remains a backend-agnostic data model describing splits and tab stacks.

Platform/window objects are not stored in the graph; window geometry and OS handles remain outside.

### 2) Docking changes are expressed as transactions

Define docking changes as high-level operations (conceptually `DockOp`) such as:

- move tab (including reorder within tab bar),
- split and insert,
- tear-off to a new window root,
- merge windows,
- close empty floating windows.

The UI layer produces these operations; the app layer applies them to the graph.

### 3) Stable panel identity for persistence and plugins

Runtime-only identities (such as tab index position or ephemeral runtime handles) are insufficient for persistence.

Introduce:

- `PanelKind` (stable identifier, e.g. string/UUID) for persistence and plugin registration,
- optional instance identity if multiple instances of a kind can exist.

### 4) Layout persistence is versioned

Serialized dock layout includes a schema/version number to support migrations.

### 5) Canonical on-disk format (v1)

The canonical persistence format is a versioned, backend-agnostic representation.

Key properties:

- Stores `PanelKind` / `PanelKey` (stable identity), not runtime-only transient identities.
- Stores window roots as logical window entries.
- Stores split/tabs structure as a graph/tree of nodes.

The exact serialization format (JSON vs RON) is an implementation choice, but the schema must include:

- `layout_version: u32`
- `windows: [...]` entries mapping logical windows to root nodes + optional placement hints
- `nodes: [...]` describing `Split` and `Tabs` nodes
- `tabs: [...]` entries containing `PanelKind` (and optional instance identity)

#### Example (JSON, v1 shape)

This is an illustrative example of the *shape* of the format (field names can be bikeshedded, but
the structure and invariants should remain stable):

```json
{
  "layout_version": 1,
  "windows": [
    {
      "logical_window_id": "main",
      "root": 1,
      "placement": {
        "width": 1280,
        "height": 720,
        "x": 120,
        "y": 80,
        "monitor_hint": "primary"
      }
    },
    {
      "logical_window_id": "floating-1",
      "root": 4,
      "placement": {
        "width": 640,
        "height": 480,
        "x": 980,
        "y": 140
      }
    }
  ],
  "nodes": [
    {
      "id": 1,
      "kind": "split",
      "axis": "horizontal",
      "children": [2, 3],
      "fractions": [0.25, 0.75]
    },
    {
      "id": 2,
      "kind": "tabs",
      "tabs": [
        { "panel_kind": "core.hierarchy" }
      ],
      "active": 0
    },
    {
      "id": 3,
      "kind": "split",
      "axis": "vertical",
      "children": [5, 6],
      "fractions": [0.7, 0.3]
    },
    {
      "id": 4,
      "kind": "tabs",
      "tabs": [
        { "panel_kind": "core.inspector" }
      ],
      "active": 0
    },
    {
      "id": 5,
      "kind": "tabs",
      "tabs": [
        { "panel_kind": "core.scene", "instance": "default" }
      ],
      "active": 0
    },
    {
      "id": 6,
      "kind": "tabs",
      "tabs": [
        { "panel_kind": "core.console" }
      ],
      "active": 0
    }
  ]
}
```

Notes:

- `logical_window_id` is stable across runs and is distinct from runtime `AppWindowId`.
- `placement` is best-effort and may be ignored if it is invalid on the current machine (see ADR 0017).
- `panel_kind` must be stable and namespaced to avoid plugin collisions.

### 6) Migration strategy

- Each persisted layout carries `layout_version`.
- Loading code migrates older versions forward in a dedicated migration module.
- Migrations are pure data transforms (no platform/window objects).

## Consequences

- Docking can evolve without entangling UI widgets with persistence formats.
- Undo/redo becomes feasible (operations are explicit).
- Plugins can register panels without leaking runtime-only IDs into user config files.

## Future Work

- Finalize the concrete v1 schema field naming and define a strict parser.
- Implement migrators for future schema changes.
- Add support for multi-monitor placement metadata (optional, platform-dependent).

## Implementation Notes (Current Prototype)

- Stable identity: implemented as `PanelKind`/`PanelKey` in `crates/fret-core/src/panels.rs`.
- Persistence v1: implemented as `DockLayoutV1` in `crates/fret-core/src/dock_layout.rs`.
- Transaction vocabulary: `DockOp` exists in `crates/fret-core/src/dock_op.rs`.
  - The demo applies `DockOp` via the runner’s effect drain path (`Effect::Dock`), which is the intended integration point.
  - Floating windows are requested via `DockOp::RequestFloatPanelToNewWindow` (window creation remains app/runner-owned).
  - Closing a floating OS window merges its panels back into the main window by default via `DockOp::MergeWindowInto`.
