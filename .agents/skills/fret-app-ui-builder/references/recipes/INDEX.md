# Recipe index (and what’s missing)

This is a living inventory for `fret-ui-shadcn` app/component recipes.

Each recipe should include:

- Upstream references (URLs first; do not depend on local snapshots)
- Fret building blocks (crate/module anchors + model shapes)
- Checklist (dismiss/focus/placement/semantics)
- `test_id` suggestions
- Regression gates (a small invariant test + a `tools/diag-scripts/*.json` repro when state machines are involved)

## Component recipes (high regression risk)

**Implemented recipes:**

- `components/select.md`
- `components/dropdown-menu.md`
- `components/context-menu.md`
- `components/dialog-and-sheet.md`
- `components/combobox.md`
- `components/tooltip.md`
- `components/popover.md`
- `components/hover-card.md`
- `components/menubar.md`
- `components/navigation-menu.md`
- `components/toast-and-sonner.md`
- `components/tabs.md`
- `components/resizable.md`
- `components/scroll-area.md`
- `components/sidebar.md`
- `components/table.md`
- `components/data-grid.md`

**Backlog (suggested next, roughly by fragility):**

- `components/command.md` (cmdk-style list: filtering + keyboard nav + active-descendant; shared by combobox/palette)
- `components/accordion-and-collapsible.md` (keyboard nav + presence/animation policy)
- `components/slider.md` (pointer capture + keyboard increments)
- `components/calendar-and-date-picker.md` (grid navigation + locale/range policy)
- `components/toggle-group.md` (roving focus + pressed/disabled states)

## App recipes (editor-grade surfaces)

**Implemented recipes:**

- `apps/app-command-palette.md`
- `apps/app-settings-form.md`
- `apps/app-data-table.md`
- `apps/app-docking-workspace.md`
- `apps/app-outliner-tree.md`
- `apps/app-inspector-panel.md`

**Backlog (recommended for “build an editor UI” work):**

- `apps/app-asset-browser.md` (grid virtualization + drag/drop)
- `apps/app-file-picker.md` (tree + breadcrumbs + preview + keyboard nav)
- `apps/app-markdown-viewer.md` (rich text + code blocks + scroll)
- `apps/app-node-graph.md` (canvas + pan/zoom + hit testing)
- `apps/app-console-log-panel.md` (virtualized log list + filters + copy/export)
- `apps/app-problems-panel.md` (diagnostics list + “jump to location” affordances)
- `apps/app-search-panel.md` (search results + highlight + replace flows)

## Contributing a new recipe

1. Add the recipe doc under `references/recipes/...`.
2. Add stable `test_id` anchors in the relevant `fret-ui-shadcn` component(s).
3. Add a small invariant test (geometry/semantics) close to the code.
4. Add a `tools/diag-scripts/*.json` repro if interaction state machines are involved.
5. Update `fret-app-ui-builder/SKILL.md` (and references if needed) and this index.
