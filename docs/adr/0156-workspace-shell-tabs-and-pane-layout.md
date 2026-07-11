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

Ordinary Rust authoring uses typed unit markers from `fret_workspace::commands::act`; those markers
lower to the same stable `ActionId == CommandId` identity used by the registry, keymaps, menus, and
diagnostics. Dynamic prefix commands remain the explicit lower-level lane for app-defined IDs.

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

### 4) Workspace model commands have one window owner

`WorkspaceWorkbench` is the default window-scoped owner for tab/pane model commands, dirty-close
transactions, and post-transition focus requests. The app/advanced driver routes those commands to
Workbench before retained UI dispatch and records the final driver-handled outcome.

`WorkspaceCommandScope` does not provide a second workspace model writer. It owns focus-only
commands and publishes pane tab-strip/content focus registries and the current focus lane so
Workbench can preserve that lane across a model transition.

While a modal input barrier is active, Workbench may claim a workspace model command only when a
direct pointer or keyboard source is proven live inside the active barrier scope. Shortcut,
programmatic, missing, stale, and underlay sources fail closed; focus-only commands and window
close remain on the modal-gated UI route. The authority is live element membership reported by
`UiTree::element_is_within_active_input_barrier_scope`, not diagnostic source metadata.

### 5) Last-tab close behavior is an explicit app policy

`WorkspaceLastTabClosePolicy::AllowEmptyPane` is the default, preserving the general workspace
contract that a pane may become empty. Apps whose router/content projection requires a live tab may
opt into `WorkspaceLastTabClosePolicy::PreserveLastTab`.

The policy applies consistently to close, close-by-id, explicit-pane close, and dirty-close replay.
A protected final close is handled but does not mutate layout state. This policy is owned by
`fret-workspace` and selected by the app; it is not a mechanism knob in `crates/fret-ui`.

## Non-goals

- Defining a document/buffer model. Tab IDs remain app-defined strings.
- Replacing docking for panels. Docking remains the contract for panel tabs/splits/tear-off
  (ADR 0013 / ADR 0017).
- Locking down visual styling. The provided widgets are intentionally minimal.

## Implementation Status (as of 2026-07-12)

The contract is implemented in `ecosystem/fret-workspace`, with the ordinary desktop composition
provided by `fret::workspace::WorkspaceApp`. UI Gallery also installs a direct
`WorkspaceWorkbench` in its explicitly advanced custom driver and opts into `PreserveLastTab`; the
default Workbench remains `AllowEmptyPane`. Focus-only commands still dispatch through the Gallery
`WorkspaceCommandScope`, while model commands report window scope, `handled_by_driver`, and typed
domain outcomes through the shared diagnostics path.

The two proof surfaces are intentionally non-interchangeable:

- `ui-gallery-workspace-shell` passed 4/4 in session `1783801365683-65846` and proves shared chrome
  plus the direct Workbench owner/trace path.
- `workspace-shell-app-facing` passed 10/10 in session `1783801123285-47715` and proves the real
  `WorkspaceApp` frame, split/move, dirty-close, focus, keyboard, semantics, and diagnostics chain.

Artifact roots and exact rerun commands are recorded in
`docs/workstreams/workspace-shell-tabstrip-fearless-refactor-v1/EVIDENCE_AND_GATES.md`.

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
- Window model owner, dirty-close transaction, and last-tab policy:
  `ecosystem/fret-workspace/src/workbench.rs`.
- App-facing driver composition: `ecosystem/fret/src/workspace.rs`.
