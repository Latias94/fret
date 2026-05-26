# IMUI List Box Container Proof v1 - Design

Status: Closed
Last updated: 2026-05-25

## Boundary

This lane adds the Fret IMUI equivalent of Dear ImGui's `BeginListBox` container:

- a stable `list_box(id, label, |ui| ...)` facade helper,
- an options form for diagnostics ids, scroll handle, layout, and multiselectable semantics,
- `SemanticsRole::ListBox` on the container,
- a scrollable vertical row host for existing `selectable` / `multi_selectable` rows.

It explicitly does not add a generic collection helper. The closed
`imui-collection-helper-readiness-v1` verdict remains authoritative for dense asset-browser grids,
shell collection outlines, command packages, and selection policy.

## Non-Goals

- No selection model.
- No filtering, typeahead, active-descendant, or keyboard owner policy.
- No command package, context-menu package, rename, duplicate, delete, or select-all helper.
- No virtualization.
- No shadcn/recipe overlay policy.
- No `fret-imui` dependency growth.

## Implementation Shape

- `ListBoxOptions` lives with container options because this is a container/host primitive.
- `list_box_controls.rs` owns private rendering.
- `facade_writer.rs` exposes `list_box` and `list_box_with_options`.
- Children are built through the existing IMUI child builder and can call current row helpers.
- The container owns only the listbox semantics wrapper and scroll/vflex host.

## Why This Is Not Generic Collection Growth

The helper does not know item count, keys, selected values, commands, layout policy, or app state.
It is a semantic/scrolled host. That makes it closer to `BeginChild` / `BeginListBox` than to a
collection abstraction.
