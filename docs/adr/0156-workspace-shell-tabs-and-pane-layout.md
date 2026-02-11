# ADR 0156: Workspace Shell (Tabs + Pane Splits) Contract

Status: Proposed

## Context

Fret targets "editor-grade" UI shells (Unity/Unreal/Godot-class workflows). Beyond docking panels
(ADR 0013), real-world editor apps need a stable, reusable workspace chrome layer:

- document tabs (MRU cycling, close/dirty, activate by id),
- editor pane splits (multiple tab groups per window),
- a minimal menu bar surface that integrates with the command registry (ADR 0023).

These behaviors are policy-heavy and should iterate faster than `crates/fret-ui`’s runtime contract
surface (ADR 0066). At the same time, app templates should not need to reinvent persistence formats
and command IDs repeatedly, as that causes churn and later migrations.

## Decision

### 1) Introduce an ecosystem-level workspace shell crate

Define workspace-shell building blocks in `ecosystem/fret-workspace`:

- `WorkspaceTabs`: a small in-memory tab model (string IDs) with MRU or in-order cycling.
- `WorkspaceLayout`: a window-scoped pane tree (leaf panes contain `WorkspaceTabs`).
- `workspace_default_menu_bar`: a minimal editor-style menu bar (data-only; `fret-runtime`).
- `WorkspaceFrame` / `WorkspaceTopBar` / `WorkspaceStatusBar` / `WorkspaceTabStrip`: lightweight UI
  elements for building editor chrome.

### 2) Commands and keybindings are stable and namespaced

The workspace crate defines command IDs and a default registration helper:

- `workspace.tab.next`
- `workspace.tab.prev`
- `workspace.tab.close`

Additionally, it defines prefix-based command families (not registry-enumerated) so apps can map
their own tab/document IDs without exposing internal IDs via generic payload enums:

- `workspace.tab.activate.<id>`
- `workspace.tab.close.<id>`

### 3) Persistence shapes are versioned and docking-independent

Workspace persistence is versioned and intentionally avoids embedding dock layout details:

- Dock graph and persistence remain owned by docking contracts (ADR 0013).
- Workspace layout focuses on editor panes + document tabs (which are not `PanelKind`).

The canonical persisted format is `WorkspaceLayoutV1` (`layout_version = 1`) containing:

- `windows`: list of logical windows, each with:
  - `id`: stable logical window id (distinct from runtime `AppWindowId`),
  - `pane_tree`: recursive split tree (axis + fraction) with leaf panes,
  - `active_pane`: optional focused pane id.

Each leaf pane stores `WorkspaceTabsV1` (tabs + active + MRU + dirty + cycle mode).

## Non-goals

- Defining a document/buffer model. Tab IDs remain app-defined strings.
- Replacing docking for panels. Docking remains the contract for panel tabs/splits/tear-off
  (ADR 0013 / ADR 0017).
- Locking down visual styling. The provided widgets are intentionally minimal.

## Consequences

- Apps get a stable "workspace shell" surface without expanding the `fret-ui` runtime contract.
- Workspace tabs/pane layout persistence can be reused across apps and iterated via versioned
  migrations, independently from docking persistence.
- Docking and multi-viewport remain compatible: editor panes can host viewport surfaces (ADR 0007),
  while docking continues to manage tool panels.

## Implementation Notes

- Workspace shell building blocks: `ecosystem/fret-workspace/src/lib.rs`.
- Tabs model + snapshots + tests: `ecosystem/fret-workspace/src/tabs.rs`.
- Pane layout + snapshots: `ecosystem/fret-workspace/src/layout.rs`.
- Menu bar helper: `ecosystem/fret-workspace/src/menu.rs`.

