# Workspace shell starter set v1

Status: Active baseline

Last updated: 2026-03-16

## Purpose

This note freezes the reusable `fret-workspace` shell starter set for v1.

The goal is to keep one boring answer to "what is the reusable shell baseline for an editor-like
Fret app?" without:

- collapsing shell and docking ownership,
- moving product-specific app protocols into `fret-workspace`,
- or teaching design-system seeding as an owner-crate responsibility.

This is a boundary and adoption note, not a new runtime contract.
ADR 0066 and ADR 0316 still provide the normative mechanism-vs-policy and token-ownership rules.

## v1 decision

The reusable `fret-workspace` shell starter set is:

- `WorkspaceFrame`
- `WorkspaceTopBar`
- `WorkspaceStatusBar`
- `workspace_pane_tree_element_with_resize`
- `WorkspaceTabStrip`
- `WorkspaceTab`
- `WorkspaceCommandScope`
- `WorkspacePaneContentFocusTarget`

Supporting policy/model surfaces that remain in-bounds for this starter set are:

- `layout::{WorkspaceLayout, WorkspaceWindowLayout, WorkspacePaneTree, WorkspacePaneLayout}`
- `tabs::WorkspaceTabs`
- `close_policy::*`
- `commands::*`
- `WorkspaceTabDragState` and `DRAG_KIND_WORKSPACE_TAB`

These supporting surfaces are allowed because the starter set needs a shell-level pane/tab model,
command vocabulary, and drag hand-off seam.
They are not a reason to move document services, page routing, menus, status schemas, or docking
policy into the crate.

## What is frozen

| Surface | Why it is in the starter set | What it still does not own |
| --- | --- | --- |
| `WorkspaceFrame` | Outer shell frame with top/left/right/bottom slots and `workspace.frame.*` fallback reads. | Product menu bars, command palettes, inspectors, sidebars, or any app-specific shell state. |
| `WorkspaceTopBar` | Context-free top-bar aggregator with left / center / right child lanes. | Toolbar recipes, breadcrumbs, page search, settings buttons, or command-palette triggers. |
| `WorkspaceStatusBar` | Context-free status-bar aggregator with left / right child lanes. | Status schemas, diagnostics strings, editor readouts, or app-specific status protocols. |
| `workspace_pane_tree_element_with_resize` plus `WorkspaceWindowLayout` | Pane tree rendering, split layout, resize handles, active-pane framing, and pane-local split/drop preview. | Dock graph policy, dock overlays, document routing, or app-owned pane content composition. |
| `WorkspaceTabStrip` plus `WorkspaceTab` | Editor-grade shell tabstrip behavior: activation, overflow/scroll, close affordances, pinned/preview rendering, and shell tab DnD hooks. | Dock-graph-aware tab/drop arbitration, document save policy, or app-specific tab metadata schemas. |
| `WorkspaceCommandScope` | Shell-level command routing between pane content and the active tab strip. | Global app command dispatch outside the workspace shell or product-specific command ownership. |
| `WorkspacePaneContentFocusTarget` | Pane-content registration seam so shell commands can return focus from tab chrome back into real content. | The content widget itself, its own focus semantics, or app-local content state. |

## Integration recipe

The intended shell composition story is:

1. The app owns `WorkspaceWindowLayout`, page/document routing, menu/status content, and any
   product-specific chrome.
2. The app renders pane content and wraps the real focusable content root in
   `WorkspacePaneContentFocusTarget`.
3. The app renders pane chrome through `workspace_pane_tree_element_with_resize`, typically using
   `WorkspaceTabStrip` inside each pane header.
4. The app places shell chrome inside `WorkspaceFrame`, usually with `WorkspaceTopBar` and an
   optional `WorkspaceStatusBar`.
5. The app wraps the composed shell in `WorkspaceCommandScope` so
   `workspace.pane.focus_tab_strip`, `workspace.pane.focus_content`, and
   `workspace.pane.toggle_tab_strip_focus` remain shell-owned behaviors.

The promoted first-party exemplar for this composition is the UI Gallery `workspace_shell` profile:

- `apps/fret-ui-gallery/src/driver/chrome.rs`
- `apps/fret-ui-gallery/src/driver/render_flow.rs`
- `apps/fret-ui-gallery/src/driver/status_bar.rs`

`apps/fret-examples/src/workspace_shell_demo.rs` remains the broader stress/demo surface for
tab/pane policy, but it is not the canonical shell-chrome teaching surface anymore.

## Boundary rules

### Shell vs docking

- `fret-workspace` owns pane-local split preview, shell tabstrip chrome, and shell focus/command
  coordination.
- `fret-docking` owns dock-graph-aware tab/drop chrome, tab-insert overlays, and docking
  arbitration.
- Visual alignment between the two belongs in adapter seeding/aliasing, not in crate coupling.

### Owner crate vs adapter crates

- `fret-workspace` may read `workspace.*` token families and keep a small resolver surface.
- Stable design-system seeding for `workspace.*` remains adapter-owned, such as
  `fret-ui-shadcn`.
- `fret-workspace` should not grow a reverse dependency on shadcn, Material, or any other skin.

### Reusable shell chrome vs app protocols

- `WorkspaceTabs` and `close_policy` are allowed starter-set helpers, but they remain shell policy
  helpers rather than app/service protocols.
- Menu models, command-palette data, inspector/property protocols, file-tree domain data,
  document services, and dirty-close prompt UX stay app-owned.
- New shell-level primitives should default to app-local composition until a second real consumer
  justifies promoting them into `fret-workspace`.

## Promoted evidence and gates

The shell-level proof surface for this freeze is the recurring UI Gallery suite:

- `tools/diag-scripts/suites/ui-gallery-workspace-shell/suite.json`

That suite groups:

- `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-chrome-shadcn-screenshot.json`
- `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-focus-command-scope-smoke.json`
- `tools/diag-scripts/ui-gallery/workspace-shell/ui-gallery-workspace-shell-tab-commands-smoke.json`

Focused crate-level evidence for shell focus/command invariants remains in `ecosystem/fret-workspace`
tests:

- `ecosystem/fret-workspace/tests/pane_focus_tab_strip_command_focuses_active_tab.rs`
- `ecosystem/fret-workspace/tests/workspace_command_scope_focus_content_restores_previous_focus.rs`
- `ecosystem/fret-workspace/tests/workspace_command_scope_focus_content_fallbacks_to_registered_pane_content.rs`
- `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_toggles_between_content_and_tab_strip.rs`
- `ecosystem/fret-workspace/tests/workspace_command_scope_toggle_tab_strip_focus_multi_pane_returns_to_last_non_tabstrip_focus.rs`
- `ecosystem/fret-workspace/tests/workspace_commands_default_keybindings_include_ctrl_f6_toggle.rs`

Lower-level tabstrip behavior references still live in the dedicated tabstrip workstreams:

- `docs/workstreams/workspace-tabstrip-editor-grade-v1/`
- `docs/workstreams/workspace-tabstrip-fearless-refactor-v1/`

Those documents remain useful for tabstrip semantics and historical rationale, but this note is the
current shell starter-set contract for the editor ecosystem workstream.

## Deliberately out of scope

This freeze does not decide:

- whether `workspace.tab.*` also needs adapter-side seeding for v1,
- how shell and docking tab chrome should be visually aliased in each adapter,
- whether app-local inspector/property protocols deserve extraction yet,
- or whether future viewport/gizmo tooling should move into separate ecosystem crates.

Those decisions remain tracked by:

- `EER-SHELL-121`
- `EER-THEME-122`
- `EER-THEME-123`
- `EER-EXTRACT-124`
- `EER-EXTRACT-125`
