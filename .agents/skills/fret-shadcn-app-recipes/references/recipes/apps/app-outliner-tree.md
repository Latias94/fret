# App recipe: Outliner / file tree

Goal: an editor-grade tree surface (file tree, scene outliner) that stays fast for large hierarchies and remains stable under cache-root reuse.

## References

- UI Gallery torture page docs: `apps/fret-ui-gallery/src/docs.rs` (File tree torture)
- UI Gallery implementation: `apps/fret-ui-gallery/src/ui.rs` (`preview_file_tree_torture`)
- File tree view helper: `ecosystem/fret-ui-kit/src/declarative/file_tree.rs` (`file_tree_view_retained_v0`)

## Building blocks

- Data model:
  - `Model<Vec<TreeItem>>` for the hierarchy (app-owned)
  - `Model<TreeState>` for UI state (selected + expanded)
- Virtualization:
  - `VirtualListScrollHandle` stored in element-local state
  - retained-host VirtualList window shifts (virt-003 / ADR 0192) to avoid full rerenders at window boundaries
- View:
  - `fret_ui_kit::declarative::file_tree::file_tree_view_retained_v0(...)`
  - `FileTreeViewProps` for row height / overscan / keep-alive budgets and debug `test_id`s

## Checklist

- Identity:
  - row identity is `TreeItemId` (stable; not index)
  - expansions and selection remain correct after reorder/insert/remove
- Virtualization:
  - overscan is tuned for “fast scroll + no popping”
  - `keep_alive` is set intentionally (trade memory for window-shift stability)
- UX:
  - clicking a folder toggles expand; click selects (policy can be customized)
  - selection scrolls into view when changed programmatically (nearest strategy)
- Semantics:
  - roles include `TreeItem` where appropriate
  - disabled nodes are not actionable and are announced

## `test_id` strategy (optional but recommended)

Use the built-in debug hooks so scripts don’t rely on pixel coordinates:

- `FileTreeViewProps::debug_root_test_id`
- `FileTreeViewProps::debug_row_test_id_prefix`

## See also

- `fret-scroll-and-virtualization` (stable keys + revisions mindset)
- `references/mind-models/mm-a11y-and-testid.md`
- `fret-diag-workflow`
