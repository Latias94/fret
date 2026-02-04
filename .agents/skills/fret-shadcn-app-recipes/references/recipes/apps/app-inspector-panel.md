# App recipe: Inspector panel (property list)

Goal: a dense, long-scrolling inspector surface (like Unity/Unreal/Godot) with stable identity for per-row editing and no “stale paint” while scrolling.

## References

- UI Gallery torture page docs: `apps/fret-ui-gallery/src/docs.rs` (Inspector torture)
- UI Gallery implementation: `apps/fret-ui-gallery/src/ui.rs` (`preview_inspector_torture`)
- Inspector protocol scaffolding (app side): `apps/fret-editor/src/inspector_protocol.rs`

## Building blocks

- Data model:
  - app-owned property graph / flattened rows
  - stable row keys (property path IDs), not list indices
- UI state:
  - selection / hover is UI-owned; edits are app-owned (`Model<T>`)
  - editing popups should be overlays (text input / sliders / vec3 editors), not “always mounted” per-row widgets
- Virtualization:
  - use `virtual_list_keyed_retained_with_layout_fn` for the baseline “50k rows” inspector workload
  - keep the per-row element tree small (text + small chrome) and move heavy editors into overlays

## Checklist

- Identity:
  - editing state (focus/caret/selection) stays attached to the correct property after reorder/filter
- IME + text editing:
  - IME composing doesn’t trigger global shortcuts
  - caret/focus restore is correct when closing editor popovers
- UX:
  - row hover/active chrome does not force full rerenders
  - keyboard navigation is predictable (up/down selection; Enter opens editor; Escape closes)
- Performance:
  - no per-frame allocations proportional to total row count
  - scrolling remains stable under cache-root reuse

## `test_id` suggestions

- `inspector-root`
- `inspector-row-<property_id>`
- `inspector-editor-<property_id>`

## See also

- `fret-text-input-and-ime` (command-vs-text arbitration + IME rules)
- `fret-overlays-and-focus` (editor popovers, focus trap/restore policies)
- `fret-scroll-and-virtualization`
