# shadcn recipe focus and builder render closure v1 - TODO

Tracking doc: `docs/workstreams/shadcn-recipe-focus-and-builder-render-closure-v1/DESIGN.md`

Milestones: `docs/workstreams/shadcn-recipe-focus-and-builder-render-closure-v1/MILESTONES.md`

This board tracks the remaining maintenance work after the initial April 2026 landing slice.

## M1 - Text-entry active chrome closure

- [x] Reclassify shadcn text-entry chrome so active border/ring state follows `focused` rather than
      `focus_visible`.
- [x] Land the focused chrome rule in `Input`.
- [x] Land the focused chrome rule in `Textarea`.
- [x] Land the focused chrome rule in `InputGroup`.
- [x] Keep the Todo demo aligned with the shared recipe behavior instead of carrying local chrome
      overrides.
- [x] Lock the landing with focused unit tests and Todo diag proof surfaces.

Current evidence:

- `ecosystem/fret-ui-shadcn/src/input.rs`
- `ecosystem/fret-ui-shadcn/src/textarea.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `apps/fret-examples/src/todo_demo.rs`
- `tools/diag-scripts/tooling/todo/todo-baseline.json`
- `tools/diag-scripts/tooling/todo/todo-shortcuts-screenshot.json`

## M2 - Builder single-render discipline closure

- [x] Remove same-frame probe-rendering from `SidebarMenuItem::into_element_with_children(...)`.
- [x] Derive `focus_within` from the real menu-item root instead of speculative child rendering.
- [x] Add a regression test that proves the builder runs only once per frame.
- [x] Keep sidebar hover-only action visibility working after the refactor.

Current evidence:

- `ecosystem/fret-ui-shadcn/src/sidebar.rs`
- `cargo test -p fret-ui-shadcn sidebar_menu_item_children_builder_runs_once_per_frame --lib`
- `cargo test -p fret-ui-shadcn sidebar_menu_action_show_on_hover_hides_until_item_hovered_on_desktop --lib`
- `cargo test -p fret-ui-shadcn sidebar_menu_action_show_on_hover_visible_when_menu_item_focus_within --lib`

## M3 - Follow-on audits and authoring discipline

- [ ] Audit any remaining text-entry wrappers in `ecosystem/fret-ui-shadcn` that own outer focus
      chrome instead of delegating to the landed `Input` / `Textarea` / `InputGroup` behavior.
- [ ] Write down a short recipe-author review checklist that bans same-frame probe renders for
      builder callbacks.
- [ ] Only add new unit tests or diag scripts when a concrete parity mismatch appears; avoid cloning
      the Todo proof surface onto unrelated components without fresh evidence.
- [ ] Revisit adjacent recipe surfaces only if they show one of these concrete symptoms:
      pointer-focused text-entry looks inactive, or builder callbacks execute twice in one frame.

## Explicitly out of scope for this lane

- [x] Do not widen the `focused` chrome rule to `Select`, `NativeSelect`, `Checkbox`, `RadioGroup`,
      `Slider`, or similar non-text-entry controls without new evidence.
- [x] Do not change runtime `focus-visible` behavior in `crates/fret-ui`.
- [x] Do not add Todo-demo-only compatibility hacks.
