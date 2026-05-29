# ImUi Dear ImGui Gap Closure v1 - Evidence & Gates

Status: Active
Last updated: 2026-05-29

## Facade Root Surface Owner-Split Evidence - 2026-05-29

Claim verified: remaining IMUI facade root scope/basic/disclosure trait default method declarations
split out of the root facade writer without changing the public `UiWriterImUiFacadeExt` trait,
caller import behavior, push-id/disabled-scope, text, separator, debug-draw, collapsing-header, or
tree-node method names and behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
  `UiWriterImUiFacadeExt` trait hub plus surface macro expansion only.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/scope_surface.rs` owns `push_id` and
  `disabled_scope` trait default declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/basic_surface.rs` owns text, wrapped text,
  bullet text, debug-draw, separator, and separator-text trait default declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure_surface.rs` owns collapsing-header and
  tree-node trait default declarations/forwarding.
- `scope_methods.rs`, `basic_items.rs`, and `disclosure_controls` remain the behavior owners.
- `tools/gate_imui_workstream_source.py` now rejects scope/basic/disclosure trait default bodies
  from drifting back into `facade_writer.rs` and checks the new surface owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui imui_text --no-fail-fast`: pass
  (2 passed, 748 skipped).
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke
  --test imui_disclosure_smoke --no-fail-fast`: pass (2 passed).
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`: pass (7 passed, 179 skipped).
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass (10 passed,
  176 skipped).

## Facade Container Surface Owner-Split Evidence - 2026-05-29

Claim verified: IMUI facade container/layout trait default method declarations split out of the
root facade writer without changing the public `UiWriterImUiFacadeExt` trait, caller import
behavior, item-flow, same-line, dummy/spacing/indent, layout group, menu/tab bar, ListBox, grid,
table, virtual-list, scroll, child-region method names, porting-sugar spacing docs, or container
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
  `UiWriterImUiFacadeExt` trait hub and expands `container_surface::container_surface_methods!()`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface.rs` owns item-flow, same-line,
  dummy/spacing/indent, layout group, menu/tab bar, ListBox, grid, table, virtual-list, scroll, and
  child-region trait default declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/*` still owns the concrete
  layout/container behavior.
- `tools/gate_imui_workstream_source.py` now rejects container trait default bodies from drifting
  back into `facade_writer.rs` and checks the new container surface owner, including ListBox and
  porting-sugar spacing anchors.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  --test imui_virtual_list_smoke --test imui_sortable_recipe_smoke --no-fail-fast`: pass
  (12 passed).
- `cargo nextest run -p fret-imui layout_collections --no-fail-fast`: pass (28 passed,
  158 skipped).
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass (18 passed,
  168 skipped).
- `cargo nextest run -p fret-imui virtual_list --no-fail-fast`: pass (2 passed, 184 skipped).

## Facade Container Surface Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade container surface macro ownership split into layout, menu/tab,
collection, and region child owners without changing the public `UiWriterImUiFacadeExt` trait,
caller import behavior, item-flow, same-line, menu/tab bar, ListBox, grid, table, virtual-list,
scroll, child-region, or concrete `container_methods/*` behavior ownership.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface.rs` is now a module/re-export
  hub for the four container facade surface macro owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface/layout.rs` owns item-flow,
  same-line, spacing, indent, horizontal, and vertical trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface/menu_tabs.rs` owns menu-bar and
  tab-bar trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface/collections.rs` owns ListBox,
  grid, table, and virtual-list trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface/regions.rs` owns scroll and
  child-region trait forwarding.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects the macro bodies
  from drifting back into `facade_writer/container_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --test imui_table_smoke --test imui_virtual_list_smoke --test imui_sortable_recipe_smoke
  --no-fail-fast`: pass (15 passed).
- `cargo nextest run -p fret-imui layout_collections --no-fail-fast`: pass (28 passed,
  158 skipped).
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass (18 passed,
  168 skipped).

## Facade Container Layout Method Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade container layout method behavior ownership split into linear,
grid/scroll, and child-region child owners without changing public facade methods, build-focus
forwarding, horizontal/vertical/grid/scroll element routing, child-region response behavior, or
`container_methods` re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/layout.rs` is now a
  module/re-export hub for the three layout method behavior owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/layout/linear.rs` owns
  horizontal and vertical forwarding to `horizontal_container_element` / `vertical_container_element`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/layout/grid_scroll.rs` owns grid
  and scroll forwarding to `grid_container_element` / `scroll_container_element`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/layout/child_region.rs` owns
  child-region forwarding and response return from `child_region::child_region_element`.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects concrete element
  routing from drifting back into `container_methods/layout.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --no-fail-fast`: pass (3 passed).
- `cargo nextest run -p fret-imui layout_collections --no-fail-fast`: pass (28 passed,
  158 skipped).

## Facade Container Collection Method Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade container collection method behavior ownership split into ListBox,
Table, and VirtualList child owners without changing public facade methods, build-focus forwarding,
collection element routing, response returns, or `container_methods` re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections.rs` is now a
  module/re-export hub for the three collection method behavior owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections/list_box.rs` owns
  ListBox option normalization and forwarding to `list_box_controls::list_box_element`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections/table.rs` owns Table
  forwarding and `TableResponse` return from `table_controls::table_element`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections/virtual_list.rs`
  owns VirtualList forwarding and `VirtualListResponse` return from
  `virtual_list_controls::virtual_list_element`.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects concrete collection
  element routing from drifting back into `container_methods/collections.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  --test imui_virtual_list_smoke --no-fail-fast`: pass (10 passed).
- `cargo nextest run -p fret-imui layout_collections --no-fail-fast`: pass (28 passed,
  158 skipped).

## Facade Container Flow Method Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade container flow method behavior ownership split into sequence, spacer,
and indent child owners without changing public facade methods, build-focus forwarding,
porting-sugar layout routing, or `container_methods` re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/flow.rs` is now a
  module/re-export hub for the three flow method behavior owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/flow/sequences.rs` owns
  item-flow and same-line forwarding to the scoped layout-sugar elements.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/flow/spacers.rs` owns dummy and
  spacing forwarding to the spacer layout-sugar elements.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/flow/indent.rs` owns indent
  forwarding to the scoped layout-sugar indent element.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects concrete
  layout-sugar routing from drifting back into `container_methods/flow.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui
  porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens
  --no-fail-fast`: pass (1 passed, 185 skipped).

## Facade Container Menu/Tab Method Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade container menu/tab method behavior ownership split into menu-bar and
tab-bar child owners without changing public facade methods, build-focus forwarding, menu/tab
element routing, tab response return, or `container_methods` re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/menu_tabs.rs` is now a
  module/re-export hub for the two menu/tab method behavior owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/menu_tabs/menu.rs` owns menu-bar
  forwarding to `menu_family_controls::menu_bar_element`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/menu_tabs/tabs.rs` owns tab-bar
  forwarding and `TabBarResponse` return from `tab_family_controls::tab_bar_element`.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects concrete menu/tab
  element routing from drifting back into `container_methods/menu_tabs.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass (18 passed,
  168 skipped).

## Facade Container Wrapper ListBox Routing Evidence - 2026-05-29

Claim verified: the IMUI facade inherent ListBox wrapper now delegates through the collection
method owner instead of constructing the ListBox element directly, without changing public inherent
method names, build-focus forwarding, or ListBox behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/collections.rs` delegates
  `list_box_with_options(...)` to `container_methods::list_box_with_options(...)`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections/list_box.rs` remains
  the concrete owner of `list_box_controls::list_box_element` routing.
- `tools/gate_imui_workstream_source.py` now rejects `list_box_controls::list_box_element` from
  drifting back into `container_wrappers/collections.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui
  list_box_container_stamps_semantics_scroll_and_hosts_selectables --no-fail-fast`: pass
  (1 passed, 185 skipped).

## Facade Floating Surface Owner-Split Evidence - 2026-05-29

Claim verified: IMUI facade floating/popup/tooltip/drag/window trait default method declarations
split out of the root facade writer without changing the public `UiWriterImUiFacadeExt` trait,
caller import behavior, floating area/window, popup, tooltip, drag/drop method names, popup/window
docs, or floating/popup/drag behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
  `UiWriterImUiFacadeExt` trait hub and expands `floating_surface::*` macros at the original
  floating, tooltip/drag, and window positions.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface.rs` owns floating layer/area,
  popup open/drop/begin, tooltip, drag/drop, and in-window floating-window trait default
  declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/*` still delegates to concrete
  floating, popup, tooltip, drag/drop, and window behavior modules.
- `tools/gate_imui_workstream_source.py` now rejects floating/popup/tooltip/drag/window trait
  default bodies from drifting back into `facade_writer.rs` and checks the new floating surface
  owner, including the window scope notes.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_floating_area_options_smoke
  --test imui_floating_window_options_smoke --test imui_popup_options_smoke
  --test imui_tooltip_smoke --test imui_drag_preview_smoke --test imui_drag_drop_smoke
  --no-fail-fast`: pass (9 passed).
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass (25 passed, 161 skipped).
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass (21 passed, 165 skipped).
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass (8 passed, 178 skipped).

## Facade Floating Surface Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade floating surface macro ownership split into popup/floating-area,
tooltip/drag, and in-window floating-window child owners without changing the public
`UiWriterImUiFacadeExt` trait, facade macro expansion points, method names, popup/window docs, or
concrete `floating_popup/*` behavior ownership.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface.rs` is now a module/re-export
  hub for the three floating facade surface macro owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup.rs` owns floating
  layer/area and popup open/drop/begin trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/tooltip_drag.rs` owns tooltip
  and drag/drop trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/window.rs` owns in-window
  floating-window trait forwarding.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects the macro bodies
  from drifting back into `facade_writer/floating_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_floating_area_options_smoke
  --test imui_floating_window_options_smoke --test imui_popup_options_smoke
  --test imui_tooltip_smoke --test imui_drag_preview_smoke --test imui_drag_drop_smoke
  --no-fail-fast`: pass (9 passed).
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass (25 passed, 161 skipped).
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass (21 passed, 165 skipped).
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass (8 passed, 178 skipped).

## Facade Floating Popup Surface Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade floating popup surface macro ownership split into floating-area/layer,
popup state, and begin-popup child owners without changing the public `UiWriterImUiFacadeExt`
trait, caller import behavior, floating layer/area/drag-surface, popup open/drop/close, popup
menu/modal method names, or concrete `floating_popup/*` behavior ownership.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup.rs` is now a
  module/re-export hub for the three popup/floating facade surface macro owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup/area.rs` owns floating
  layer, floating area, and area drag-surface trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup/state.rs` owns popup
  open-model, drop, open, anchor-open, and close trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup/begin.rs` owns popup
  menu/modal begin trait forwarding.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects the macro bodies
  from drifting back into `facade_writer/floating_surface/popup.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_floating_area_options_smoke
  --test imui_floating_window_options_smoke --test imui_popup_options_smoke --no-fail-fast`:
  pass (6 passed).
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass (25 passed, 161 skipped).
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass (21 passed, 165 skipped).

## Facade Menu/Selection Surface Owner-Split Evidence - 2026-05-29

Claim verified: IMUI facade menu/selection trait default method declarations split out of the root
facade writer without changing the public `UiWriterImUiFacadeExt` trait, caller import behavior,
menu item, begin menu/submenu, selectable, multi-selectable, combo, context-menu method names,
focusable recording wrappers, popup/menu behavior, or selectable/combo behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
  `UiWriterImUiFacadeExt` trait hub and expands
  `menu_selection_surface::menu_selection_surface_methods!()`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface.rs` owns menu item, begin
  menu/submenu, selectable, multi-selectable, combo, and context-menu trait default
  declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`,
  `selection_combo.rs`, and `floating_popup/*` still carry inherent wrappers and underlying
  behavior.
- `tools/gate_imui_workstream_source.py` now rejects menu/selection trait default bodies from
  drifting back into `facade_writer.rs` and checks the new menu/selection surface owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --test imui_combo_smoke --test imui_disclosure_smoke --test imui_popup_options_smoke
  --no-fail-fast`: pass (5 passed).
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass (18 passed,
  168 skipped).
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass (21 passed, 165 skipped).
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass (11 passed, 175 skipped).
- `cargo nextest run -p fret-imui multi_select --no-fail-fast`: pass (1 passed, 185 skipped).

## Facade Menu/Selection Surface Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade menu/selection surface macro ownership split into menu-item,
menu-family, selection/combo, and context-popup child owners without changing the public
`UiWriterImUiFacadeExt` trait, caller import behavior, menu item, begin menu/submenu, selectable,
multi-selectable, combo, context-menu method names, or underlying behavior ownership.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface.rs` is now a
  module/re-export hub for the four menu/selection facade surface macro owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface/menu_items.rs` owns menu
  item trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface/menu_family.rs` owns begin
  menu/submenu trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface/selection_combo.rs` owns
  selectable, multi-selectable, and combo trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface/context_popup.rs` owns
  context-menu popup trait forwarding.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects the macro bodies
  from drifting back into `facade_writer/menu_selection_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --test imui_combo_smoke --test imui_disclosure_smoke --test imui_popup_options_smoke
  --no-fail-fast`: pass (5 passed).
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass (18 passed,
  168 skipped).
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass (21 passed, 165 skipped).
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass (11 passed, 175 skipped).
- `cargo nextest run -p fret-imui multi_select --no-fail-fast`: pass (1 passed, 185 skipped).

## Facade Selectable/Combo Inherent Wrapper Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade selectable/combo inherent wrapper behavior ownership split into
selectable and combo child owners without changing public inherent method names, focusable
recording, disabled checks, selectable/multi-selectable delegation, combo delegation, or
`fret-imui` thinness.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs` is now a module hub for the two
  selectable/combo inherent wrapper owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo/selectables.rs` owns selectable and
  multi-selectable inherent wrappers, including disabled checks and focusable recording.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo/combo.rs` owns direct combo
  inherent wrappers, including disabled checks and focusable recording.
- `tools/gate_imui_workstream_source.py` checks the new child owners and rejects focusable-recording
  wrapper bodies from drifting back into `selection_combo.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --test imui_combo_smoke --no-fail-fast`: pass (3 passed).
- `cargo nextest run -p fret-imui models_combo multi_select --no-fail-fast`: pass (12 passed,
  174 skipped).

## Facade Model Surface Owner-Split Evidence - 2026-05-29

Claim verified: IMUI facade model/control trait default method declarations split out of the root
facade writer without changing the public `UiWriterImUiFacadeExt` trait, caller import behavior,
checkbox/radio/switch, slider/combo model, input text, picker/history text, textarea method names,
focusable recording wrappers, or model-control behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
  `UiWriterImUiFacadeExt` trait hub and expands `model_surface::model_surface_methods!()`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface.rs` owns checkbox/radio/switch,
  slider/combo model, input text model, input text picker/history model, and textarea model trait
  default declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`,
  `value_models.rs`, and `text_models.rs` still own focusable-recording inherent wrappers.
- `tools/gate_imui_workstream_source.py` now rejects model/control trait default bodies from
  drifting back into `facade_writer.rs` and checks the new model surface owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke
  --test imui_combo_smoke --test imui_textarea_smoke --test imui_input_text_picker_options_smoke
  --no-fail-fast`: pass (8 passed).
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass (11 passed, 175 skipped).
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass (11 passed, 175 skipped).
- `cargo nextest run -p fret-imui models_text --no-fail-fast`: pass (29 passed, 157 skipped).

## Facade Model Surface Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade model surface macro ownership split into boolean, value/combo, and
text child owners without changing the public `UiWriterImUiFacadeExt` trait, caller import
behavior, checkbox/radio/switch, slider/combo model, input text, picker/history text, textarea
method names, focusable recording wrappers, or model-control behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface.rs` is now a module/re-export hub for
  the three model facade surface macro owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface/boolean.rs` owns checkbox, radio,
  and switch model trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface/value_combo.rs` owns slider and
  combo-model trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface/text.rs` owns input text, input text
  picker/history, and textarea trait forwarding.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects the macro bodies
  from drifting back into `facade_writer/model_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_leaf_control_options_smoke
  --test imui_combo_smoke --test imui_textarea_smoke --test imui_input_text_options_smoke
  --test imui_input_text_picker_options_smoke --no-fail-fast`: pass (10 passed).
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass (11 passed, 175 skipped).
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass (11 passed, 175 skipped).
- `cargo nextest run -p fret-imui models_text --no-fail-fast`: pass (29 passed, 157 skipped).

## Facade Boolean-Control Inherent Wrapper Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade boolean-control inherent wrapper behavior ownership split into checkbox,
radio, and switch child owners without changing public inherent method names, disabled checks,
focusable recording, trait delegation paths, or `fret-imui` thinness.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs` is now a module hub for the
  three boolean-control inherent wrapper owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers/checkbox.rs` owns checkbox model
  inherent wrappers, including disabled checks and focusable recording.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers/radio.rs` owns radio inherent
  wrappers, including disabled checks and focusable recording.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers/switch.rs` owns switch model
  inherent wrappers, including disabled checks and focusable recording.
- `tools/gate_imui_workstream_source.py` checks the new child owners and rejects focusable-recording
  wrapper bodies from drifting back into `boolean_wrappers.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke
  --test imui_button_smoke --no-fail-fast`: pass (4 passed).
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass (11 passed, 175 skipped).

## Facade Value/Combo-Model Inherent Wrapper Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade value/combo-model inherent wrapper behavior ownership split into slider
and combo-model child owners without changing public inherent method names, disabled checks,
focusable recording, trait delegation paths, or `fret-imui` thinness.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs` is now a module hub for the two
  value/combo-model inherent wrapper owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models/slider.rs` owns slider model inherent
  wrappers, including disabled checks and focusable recording.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models/combo_model.rs` owns combo-model
  inherent wrappers, including disabled checks and focusable recording.
- `tools/gate_imui_workstream_source.py` checks the new child owners and rejects focusable-recording
  wrapper bodies from drifting back into `value_models.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_combo_smoke --no-fail-fast`:
  pass (2 passed).
- `cargo nextest run -p fret-imui models_combo models_controls --no-fail-fast`: pass (22 passed,
  164 skipped).

## Facade Text-Model Inherent Wrapper Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade text-model inherent wrapper behavior ownership split into input,
picker, and textarea child owners without changing public inherent method names,
disabled/focusable checks, picker focusable calculation, trait delegation paths, or `fret-imui`
thinness.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs` is now a module hub for the three
  text-model inherent wrapper owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models/input.rs` owns single-line input model
  inherent wrappers, including disabled checks and focusable recording.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models/picker.rs` owns completion/history
  picker inherent wrappers, including picker-specific focusable calculation and focus recording.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models/textarea.rs` owns textarea model
  inherent wrappers, including disabled checks and focusable recording.
- `tools/gate_imui_workstream_source.py` checks the new child owners and rejects focusable-recording
  wrapper bodies from drifting back into `text_models.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_input_text_options_smoke
  --test imui_input_text_picker_options_smoke --test imui_textarea_smoke --no-fail-fast`: pass
  (5 passed).
- `cargo nextest run -p fret-imui models_text --no-fail-fast`: pass (29 passed, 157 skipped).

## Facade Button Surface Owner-Split Evidence - 2026-05-29

Claim verified: IMUI facade button/image/action trait default method declarations split out of the
root facade writer without changing the public `UiWriterImUiFacadeExt` trait, caller import
behavior, button/image/action method names, button-command presentation forwarding, focusable
recording wrappers, image-button option normalization, or button/image smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
  `UiWriterImUiFacadeExt` trait hub and expands `button_surface::button_surface_methods!()`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface.rs` owns button,
  small/arrow/invisible button, image item/button, action button, payload action button, and
  button-command trait default declarations/forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` and
  `button_actions/*` still own focusable-recording inherent wrappers and command presentation
  forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/image_items.rs` still owns image-button option
  normalization.
- `tools/gate_imui_workstream_source.py` now rejects button/image/action trait default bodies from
  drifting back into `facade_writer.rs` and checks the new button surface owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke
  --test imui_image_item_smoke --no-fail-fast`: pass (3 passed).
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass (10 passed,
  176 skipped).
- `cargo nextest run -p fret-imui image_item --no-fail-fast`: pass (2 passed, 184 skipped).
- `cargo nextest run -p fret-imui control_geometry --no-fail-fast`: pass (4 passed, 182 skipped).

## Facade Button Surface Sub-Owner Evidence - 2026-05-29

Claim verified: IMUI facade button surface macro ownership split into plain-button, image-button,
and action-button child owners without changing the public `UiWriterImUiFacadeExt` trait, caller
import behavior, button/image/action method names, command presentation forwarding, focusable
recording wrappers, or image-button option normalization.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface.rs` is now a module/re-export hub
  for the three button facade surface macro owners.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface/plain.rs` owns plain, small, arrow,
  and invisible button trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface/images.rs` owns image item/button
  trait forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface/actions.rs` owns action button,
  payload action button, and button-command trait forwarding.
- `tools/gate_imui_workstream_source.py` checks the new sub-owners and rejects the macro bodies
  from drifting back into `facade_writer/button_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke
  --test imui_image_item_smoke --no-fail-fast`: pass (3 passed).
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass (10 passed,
  176 skipped).
- `cargo nextest run -p fret-imui image_item --no-fail-fast`: pass (2 passed, 184 skipped).
- `cargo nextest run -p fret-imui control_geometry --no-fail-fast`: pass (4 passed, 182 skipped).

## Item Behavior Pointer Hook Owner-Split Evidence - 2026-05-29

Claim verified: IMUI shared pressable item pointer hook bodies split into down/move/up private
owners without changing shared button, checkbox/radio, selectable, combo, image-item, debug-draw
pressable, context-menu, pointer-click, double-click, drag, long-press, lifecycle, or response
population behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/item_behavior/install.rs` keeps hook clearing, model capture, and
  `PressableItemBehavior` assembly.
- `ecosystem/fret-ui-kit/src/imui/item_behavior/install/pointer_down.rs` owns lifecycle activation
  and drag start preparation.
- `ecosystem/fret-ui-kit/src/imui/item_behavior/install/pointer_move.rs` owns drag-threshold move
  handling.
- `ecosystem/fret-ui-kit/src/imui/item_behavior/install/pointer_up.rs` owns lifecycle deactivation,
  drag finish, context-menu transients, pointer-click modifier capture, and double-click
  transients.
- `tools/gate_imui_workstream_source.py` now rejects pointer hook bodies drifting back to
  `install.rs` and checks the dedicated down/move/up owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_press --no-fail-fast`: pass (9 passed, 177 skipped).
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass (8 passed, 178 skipped).
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass (21 passed, 165 skipped).

## Table Response Owner-Split Evidence - 2026-05-29

Claim verified: IMUI table response header/resize accessors split into dedicated owners without
changing public response names, field privacy, header lookup behavior, resize drag accessors, width
clamping, table smoke behavior, or table-column visibility behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/widgets/table.rs` now stays the `TableResponse`
  aggregation and header lookup owner.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/table/header.rs` owns `TableHeaderResponse`
  storage and accessors.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/table/resize.rs` owns
  `TableColumnResizeResponse` storage, drag accessors, and width clamping.
- `tools/gate_imui_workstream_source.py` now follows the opaque-struct checks to the split owners
  and rejects header/resize response bodies from drifting back into `table.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  --no-fail-fast`: pass (9 passed).
- `cargo nextest run -p fret-ui-kit --features imui table_column_visibility
  --no-fail-fast`: pass (11 passed, 739 skipped).

## Debug-Draw Root Owner-Split Evidence - 2026-05-29

Claim verified: IMUI debug-draw root draw-list state and facade entry glue split into dedicated
owners without changing public debug-draw list/options/response names, command recording, channel
merging, summary projection, element mounting, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` now stays a thin module/re-export hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list.rs` owns `ImUiDebugDrawList` and
  channel-split state storage.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/facade.rs` owns `debug_draw_with_options(...)`
  list capture, channel merge, command-summary projection, command boxing, keyed element mounting,
  and `DebugDrawResponse` assembly.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/element.rs` now imports the element
  owner directly instead of relying on a root helper import.
- `tools/gate_imui_workstream_source.py` now rejects debug-draw list state and facade entry bodies
  from drifting back into `debug_draw_controls.rs` and checks the dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui imui::debug_draw_controls
  --no-fail-fast`: pass (38 passed, 712 skipped).
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke
  --no-fail-fast`: pass (1 passed).

## Popup Overlay Root Owner-Split Evidence - 2026-05-29

Claim verified: IMUI popup-overlay root state helpers and context-menu wrapper split into private
owners without changing popup open/close/drop/open-at behavior, context-menu anchor fallback,
menu/modal entrypoints, popup options smoke behavior, or fret-imui popup-hover behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay.rs` now stays a thin menu/modal entrypoint hub and
  re-exports the popup state and context-menu owners.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/state.rs` owns popup open model lookup, scope drop,
  open/open-at/close mutations, keep-alive generation writes, anchor writes, and redraw requests.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/context_menu.rs` owns context-menu request
  inspection, 1px fallback anchor creation, and menu delegation.
- `tools/gate_imui_workstream_source.py` now rejects popup state/context-menu bodies from drifting
  back into `popup_overlay.rs` and checks the dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_popup_options_smoke
  --no-fail-fast`: pass (1 passed).
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass
  (21 passed, 165 skipped).

## Tab-Bar Item Method Owner-Split Evidence - 2026-05-29

Claim verified: IMUI tab-bar item builder methods split out of
`ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs` into a private owner without changing
public `ImUiTabBar` method names, label identity parsing, panel test-id fallback, focused-child
capture, tab smoke behavior, or fret-imui tab behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs` keeps `ImUiTabBar` storage and
  `tab_bar_element` assembly.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/item_methods.rs` owns `tab_item`,
  `tab_item_with_options`, `begin_tab_item`, and `begin_tab_item_with_options`, including label
  identity parsing, panel test-id fallback, focused-child capture, and `BuiltTabItem` population.
- `tools/gate_imui_workstream_source.py` now rejects item-builder bodies from drifting back into
  `tab_family_controls.rs` and checks the dedicated item-method owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui imui::tab_family_controls::tests
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui interaction_menu_tabs::tabs
  --no-fail-fast`: pass (3 passed, 183 skipped).

## Floating Area Option/Context Owner-Split Evidence - 2026-05-29

Claim verified: IMUI floating-area option/context records split into options and opaque context
private owners without changing public option/context names, area defaults, accessor-first context
shape, root re-exports, floating-area smoke behavior, or fret-imui floating behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_options/area.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/floating_options/area/options.rs` owns
  `FloatingAreaOptions`, including test-id prefix, explicit test-id, pass-through, and no-inputs
  defaults.
- `ecosystem/fret-ui-kit/src/imui/floating_options/area/context.rs` owns opaque
  `FloatingAreaContext` storage and read-only accessors.
- `ecosystem/fret-ui-kit/tests/imui_floating_area_options_smoke.rs` freezes area defaults,
  pass-through/no-inputs overrides, and the public `FloatingAreaContext` re-export surface.
- `tools/gate_imui_workstream_source.py` now follows the opaque context check to
  `floating_options/area/context.rs`, rejects area option/context bodies from drifting back into
  `floating_options/area.rs`, and checks the dedicated owners plus smoke test.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_floating_area_options_smoke
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui floating --no-fail-fast`: pass
  (25 passed, 161 skipped).

## Floating Window Option Owner-Split Evidence - 2026-05-29

Claim verified: IMUI floating-window option records split into behavior, resize, and root window
option private owners without changing public option names, defaults, builder methods, root
re-exports, floating-window smoke behavior, or fret-imui floating behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_options/window.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/floating_options/window/behavior.rs` owns
  `FloatingWindowOptions` behavior flags and defaults.
- `ecosystem/fret-ui-kit/src/imui/floating_options/window/resize.rs` owns
  `FloatingWindowResizeOptions` min/max size defaults.
- `ecosystem/fret-ui-kit/src/imui/floating_options/window/options.rs` owns `WindowOptions`,
  including the `open`, `size`, `resize`, `behavior` fields and builder methods.
- `ecosystem/fret-ui-kit/tests/imui_floating_window_options_smoke.rs` freezes behavior defaults,
  resize defaults, and the window option builder API through the public IMUI re-export surface.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `floating_options/window.rs` and checks the three dedicated owners plus smoke test.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_floating_window_options_smoke
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui floating --no-fail-fast`: pass
  (25 passed, 161 skipped).

## Editor Theme Preset Picker Option Owner-Split Evidence - 2026-05-29

Claim verified: editor-owned IMUI style/theme preset picker option records split into a private
owner without changing public control names, option fields/defaults, controls re-exports, IMUI
adapter callsite, listbox semantics, selected-state behavior, or reversible preset replay.

Evidence:

- `ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker.rs` keeps picker rendering,
  listbox/listbox-option semantics, selected-state behavior, and preset install/replay behavior.
- `ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker/options.rs` owns
  `EditorThemePresetPickerOptions` and its default layout, enabled/focusable state, label, and
  diagnostics fields.
- `ecosystem/fret-ui-editor/tests/imui_adapter_smoke.rs` now freezes
  `EditorThemePresetPickerOptions` defaults through the promoted IMUI adapter surface.
- `tools/gate_imui_workstream_source.py` now rejects the option body from drifting back into
  `editor_theme_preset_picker.rs` and checks the dedicated options owner.

Focused gates:

- `cargo fmt -p fret-ui-editor --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-editor --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-editor editor_theme_preset_picker_stamps_listbox_options_and_selected_state editor_theme_preset_picker_click_updates_model_and_replays_reversible_preset --no-fail-fast`:
  pass.

## Leaf Control Option Owner-Split Evidence - 2026-05-29

Claim verified: IMUI leaf control option records split into selection, tab-item, and slider private
owners without changing public option type names, fields, defaults, root re-exports, selectable
smoke behavior, tab/menu behavior, or slider model behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/selection.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/selection/options.rs` owns
  `SelectableOptions`, including highlighted row, popup-close model, a11y role, shortcut fields,
  and defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/tab.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/tab/options.rs` owns `TabItemOptions`,
  including default-selection, panel test-id, shortcut fields, and defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/value.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/value/slider.rs` owns `SliderOptions`,
  including min/max/step defaults.
- `ecosystem/fret-ui-kit/tests/imui_leaf_control_options_smoke.rs` freezes selectable, tab-item,
  and slider option defaults.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  the leaf hub files and checks the three dedicated owners plus smoke test.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_leaf_control_options_smoke
  --test imui_selectable_smoke --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui models_controls::slider
  interaction_menu_tabs::tabs --no-fail-fast`: pass.

## Input-Text Picker Option Owner-Split Evidence - 2026-05-29

Claim verified: IMUI input-text-picker option records split into filter and options private owners
without changing public option type names, filter matching behavior, default input/popup/options,
root re-exports, picker smoke behavior, or fret-imui picker behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/text/picker.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/picker/filter.rs` owns
  `InputTextPickerFilter` and its matching policy.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/picker/options.rs` owns
  `InputTextPickerOptions`, including default input/popup/options and picker flags.
- `ecosystem/fret-ui-kit/tests/imui_input_text_picker_options_smoke.rs` freezes picker filter
  matching and default popup/options values.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `picker.rs` and checks the two dedicated owners plus smoke test.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_input_text_picker_options_smoke
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass.

## Textarea Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI textarea option records split into submit-key and options private owners
without changing public option type names, multiline submit/cancel defaults, stable-line-box
default, root re-exports, textarea smoke behavior, or fret-imui textarea model behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea/submit_key.rs` owns
  `TextAreaSubmitKey`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea/options.rs` owns
  `TextAreaOptions`, including multiline submit/cancel defaults and stable line-box default.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `textarea.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_textarea_smoke --no-fail-fast`:
  pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui models_text_area --no-fail-fast`: pass.

## Input-Text Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI input-text option records split into mode and options private owners without
changing public option type names, text-field semantics default, command-policy defaults, filter
fields, root re-exports, input-text option smoke behavior, or fret-imui text-model behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/text/input.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/input/mode.rs` owns `InputTextMode`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/input/options.rs` owns `InputTextOptions`,
  including text-field semantics and command-policy defaults.
- `ecosystem/fret-ui-kit/tests/imui_input_text_options_smoke.rs` freezes input text option defaults
  and password mode construction.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `input.rs` and checks the two dedicated owners plus smoke test.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_input_text_options_smoke
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui models_text_modes models_text_commands
  models_text_filters --no-fail-fast`: pass.

## Popup Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI popup option records split into popup-menu and popup-modal private owners
without changing public option type names, popup placement defaults, menu size/modal/auto-focus
defaults, modal size/outside-press defaults, root re-exports, popup smoke behavior, or popup-hover
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/menus/popup.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/menus/popup/menu.rs` owns `PopupMenuOptions` and its
  popper placement/menu size/modal/auto-focus defaults.
- `ecosystem/fret-ui-kit/src/imui/options/menus/popup/modal.rs` owns `PopupModalOptions` and its
  size/outside-press defaults.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `popup.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_popup_options_smoke
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass.

## Misc Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI misc option records split into drag-source, drop-target, separator-text, and
bullet-text private owners without changing public option type names, default enabled/cross-window
behavior, `test_id` fields, root re-exports, drag/drop smoke behavior, separator/bullet smoke
behavior, or fret-imui composition/drag behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/misc.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/misc/drag_source.rs` owns `DragSourceOptions` and its
  default enabled/cross-window policy.
- `ecosystem/fret-ui-kit/src/imui/options/misc/drop_target.rs` owns `DropTargetOptions` and its
  default enabled policy.
- `ecosystem/fret-ui-kit/src/imui/options/misc/separator_text.rs` owns `SeparatorTextOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/misc/bullet_text.rs` owns `BulletTextOptions`.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `misc.rs` and checks the four dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_drop_smoke --test
  imui_drag_preview_smoke --test imui_separator_text_smoke --test imui_bullet_text_smoke
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui composition separator_text bullet_text
  --no-fail-fast`: pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass.

## Spacer Flow Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI spacer flow option records split into dummy, spacing, and indent private
owners without changing public option type names, default optional size, indent token default,
`test_id` fields, flow re-exports, container smoke behavior, or porting-sugar behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer.rs` is now a private re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer/dummy.rs` owns `DummyOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer/spacing.rs` owns
  `SpacingOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer/indent.rs` owns `IndentOptions`
  and its IMUI indent spacing default.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `flow/spacer.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib containers::tests --no-fail-fast`:
  pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui
  porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens
  --no-fail-fast`: pass.

## Linear Flow Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI linear flow option records split into horizontal and vertical private owners
without changing public option type names, default gaps, default center/stretch item alignment,
`test_id` fields, flow re-exports, container smoke behavior, or porting-sugar behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear.rs` is now a private re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear/horizontal.rs` owns
  `HorizontalOptions` and its center item alignment default.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear/vertical.rs` owns
  `VerticalOptions` and its stretch item alignment default.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `flow/linear.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib containers::tests --no-fail-fast`:
  pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui
  porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens
  --no-fail-fast`: pass.

## Inline Flow Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI inline flow option records split into item-flow and same-line private owners
without changing public option type names, default gaps, default alignment/stretch behavior,
`test_id` fields, flow re-exports, container smoke behavior, or porting-sugar behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline.rs` is now a private re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline/item_flow.rs` owns
  `ItemFlowOptions` and its IMUI vertical item spacing default.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline/same_line.rs` owns
  `SameLineOptions` and its IMUI horizontal item spacing default.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `flow/inline.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib containers::tests --no-fail-fast`:
  pass.
- `CARGO_INCREMENTAL=0 cargo nextest run -p fret-imui
  porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens
  --no-fail-fast`: pass.

Notes:

- The broader `cargo nextest run -p fret-ui-kit --features imui --lib containers --no-fail-fast`
  filter timed out before returning test results, so the focused `containers::tests` gate above was
  used for this narrow option-owner split.
- A first `fret-imui` rerun after stopping a timed-out `rustc` process failed during MSVC linking
  with unresolved externals. `cargo clean -p fret-imui` removed the interrupted test artifacts, and
  the non-incremental focused rerun above passed.

## Child-Region Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI child-region option records split into chrome, body options, and resize
private owners without changing public option type names, default chrome, scroll/layout fields,
resize defaults, resize builder methods, root re-exports, child-region smoke behavior, or
composition behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/containers/child_region.rs` is now a public re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/child_region/chrome.rs` owns
  `ChildRegionChrome`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/child_region/options.rs` owns
  `ChildRegionOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/child_region/resize.rs` owns
  `ChildRegionResizeXOptions`, `ChildRegionResizeYOptions`, and their builder methods.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `child_region.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui composition child_region --no-fail-fast`:
  pass.

## Table Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI table/row/cell option records split into private owners without changing
public option type names, default values, `TableOptions` debug formatting, horizontal-scroll
defaults, row/cell background seams, root re-exports, table smoke behavior, or table composition
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/collections/table.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table/root.rs` owns `TableOptions`, its
  debug projection, and table defaults.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table/row.rs` owns `TableRowOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table/cell.rs` owns `TableCellOptions`.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `table.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui composition table --no-fail-fast`:
  pass.

## Menu Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menu option records split into menu-bar, begin-menu/submenu, and menu-item
private owners without changing public option type names, default values, submenu popup placement,
shortcut fields, root re-exports, menu smoke behavior, or menu/tab interaction behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/menus/menu.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/menus/menu/bar.rs` owns `MenuBarOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/menus/menu/begin.rs` owns `BeginMenuOptions`,
  `BeginSubmenuOptions`, and the submenu popup placement defaults.
- `ecosystem/fret-ui-kit/src/imui/options/menus/menu/item.rs` owns `MenuItemOptions`.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `menu.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`:
  pass.

## Combo Control Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI combo control option types split into direct-combo and model-combo private
owners without changing public option type names, default values, placeholder text, popup defaults,
shortcut fields, facade imports, combo smoke behavior, or combo-model behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/combo.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/combo/direct.rs` owns `ComboOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/combo/model.rs` owns `ComboModelOptions` and
  its default placeholder text.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `combo.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_combo_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`:
  pass.

## Boolean Control Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI boolean control option types split into checkbox, radio, and switch private
owners without changing public option type names, default values, shortcut fields, facade imports,
button smoke coverage, or boolean control behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/boolean.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/boolean/checkbox.rs` owns `CheckboxOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/boolean/radio.rs` owns `RadioOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/boolean/switch.rs` owns `SwitchOptions`.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `boolean.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`:
  pass.

## Disclosure Control Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI disclosure control option types split into collapsing-header and tree-node
private owners without changing public option type names, default values, shortcut fields,
hierarchy metadata defaults, facade imports, disclosure smoke behavior, or disclosure-control
tests.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/disclosure.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/disclosure/collapsing_header.rs` owns
  `CollapsingHeaderOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/disclosure/tree_node.rs` owns
  `TreeNodeOptions`.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `disclosure.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_disclosure_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib collapsing_header_default_open_mounts_body tree_node_default_options_start_at_level_one tree_node_leaf_uses_tree_item_semantics --no-fail-fast`:
  pass.

## Button/Image Control Option Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button/image control option types split into button and image-item private
owners without changing public option type names, default values, image-item builder methods,
shortcut fields, facade imports, button smoke behavior, or image-item smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/button_image.rs` is now a public re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/button_image/button.rs` owns
  `ButtonArrowDirection`, `ButtonVariant`, and `ButtonOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/button_image/image.rs` owns `ImageItemVariant`
  and `ImageItemOptions`.
- `tools/gate_imui_workstream_source.py` now rejects these option bodies from drifting back into
  `button_image.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_image_item_smoke --no-fail-fast`:
  pass.

## Debug-Draw Rect Path Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw rect path construction split into plain-rect and rounded-rect
private owners without changing clockwise rect command ordering, rounded-rect effective rounding
clamp, per-corner sampling, fallback square points, path-builder call sites, path tests, or public
debug-draw behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/rects.rs` is now a private re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/rects/plain.rs` owns plain closed rect
  path commands.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/rects/rounded.rs` owns rounded-rect
  point generation and corner arc sampling.
- `tools/gate_imui_workstream_source.py` now rejects rect path bodies from drifting back into
  `rects.rs` and checks the two dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib rect_path_closes_clockwise_edges debug_draw_path_builder_appends_rounded_rect_corner_samples --no-fail-fast`:
  pass.

## Debug-Draw Linear Path Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw linear path construction split into polyline, polygon fill, and
primitive private owners without changing open/closed stroke point requirements, polyline command
ordering, convex/concave fill forwarding, triangle/quad closure, paint-shape call sites, path
tests, or public debug-draw behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/linear.rs` is now a private re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/linear/polyline.rs` owns stroke point
  requirements and polyline command construction.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/linear/fills.rs` owns convex/concave
  fill forwarding to closed polyline construction.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/linear/primitives.rs` owns triangle
  and quad path construction.
- `tools/gate_imui_workstream_source.py` now rejects these linear path bodies from drifting back
  into `linear.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib polyline_path_requires_enough_points_and_closes_when_requested convex_poly_fill_path_requires_three_points_and_closes concave_poly_fill_path_requires_three_points_and_closes triangle_path_closes_and_degenerate_triangles_are_detected quad_path_closes_four_ordered_points --no-fail-fast`:
  pass.

## Debug-Draw Round Path Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw round path construction split into circle, ngon, and ellipse
private owners without changing circle cubic approximation, ngon validation/point generation,
ellipse default segment fallback/rotation, paint-shape call sites, path tests, or public debug-draw
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/round.rs` is now a private re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/round/circle.rs` owns circle cubic
  path construction.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/round/ngon.rs` owns regular polygon
  path validation and point generation.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/round/ellipse.rs` owns ellipse path
  validation, default segment fallback, and rotation sampling.
- `tools/gate_imui_workstream_source.py` now rejects these round path bodies from drifting back
  into `round.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib circle_path_uses_four_cubic_arcs_and_closes ngon_path_requires_three_segments_and_positive_radius ellipse_path_defaults_segments_and_supports_rotation --no-fail-fast`:
  pass.

## Debug-Draw Path Sampling Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw path sampling helpers split into segment, arc, and Bezier private
owners without changing default segment fallback, arc/elliptical arc point generation, Bezier point
interpolation, path-builder command output, or public debug-draw behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/sampling.rs` is now a private
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/sampling/segments.rs` owns default
  segment fallback for arc, Bezier, and elliptical arc commands.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/sampling/arcs.rs` owns circular and
  elliptical arc point sampling.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/sampling/beziers.rs` owns quadratic and
  cubic Bezier point interpolation.
- `tools/gate_imui_workstream_source.py` now rejects these helper bodies from drifting back into
  `sampling.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_path_builder_appends_arc_samples debug_draw_path_builder_arc_helpers_handle_fast_default_and_degenerate_inputs debug_draw_path_builder_appends_elliptical_arc_samples debug_draw_path_builder_elliptical_arc_handles_rotation_default_and_invalid_inputs debug_draw_path_builder_appends_bezier_curve_samples debug_draw_path_builder_bezier_helpers_require_a_start_point_and_default_segments --no-fail-fast`:
  pass.

## Debug-Draw Geometry Helper Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw geometry helpers split into finite, rectangle, and triangle private
owners without changing finite checks, rect emptiness/finite checks, rect quad point generation,
effective rounding clamp rules, triangle degeneracy checks, indexed triangle lookup, sequential
index generation, or public debug-draw behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry.rs` is now a private re-export hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry/finite.rs` owns point, UV, and
  debug-draw vertex finite checks.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry/rects.rs` owns rect checks, quad
  point generation, and effective rounding clamp rules.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry/triangles.rs` owns triangle
  degeneracy/drawability checks, indexed triangle lookup, and sequential index generation.
- `tools/gate_imui_workstream_source.py` now rejects these helper bodies from drifting back into
  `geometry.rs` and checks the three dedicated owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_path_builder_rect_rounding_clamps_and_handles_invalid_inputs debug_draw_list_records_triangle_mesh_commands debug_draw_list_reports_command_summaries_in_merge_order --no-fail-fast`:
  pass.

## Facade Menu-Item Inherent Wrapper Owner-Split Evidence - 2026-05-28

Claim verified: IMUI facade menu-item inherent wrappers split into a private owner without changing
plain menu-item wrappers, checkbox/radio wrappers, action menu item wrappers, focusable recording,
begin-menu/submenu wrappers, command presentation forwarding, or public facade method names.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs` keeps begin-menu/submenu inherent
  wrappers and command button wiring.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items/item_methods.rs` owns plain,
  checkbox/radio, and action menu item inherent wrappers.
- `tools/gate_imui_workstream_source.py` now rejects plain/checked/action menu item wrappers from
  drifting back into `menu_items.rs` and checks the dedicated item-method owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `$env:CARGO_INCREMENTAL='0'; cargo test -p fret-imui menu_item_checkbox --no-run`: pass.
- `$env:CARGO_INCREMENTAL='0'; cargo test -p fret-imui menu_item_checkbox -- --nocapture`:
  pass; 1 checkbox semantics test passed.
- `$env:CARGO_INCREMENTAL='0'; cargo test -p fret-imui menu_item_activate_shortcut -- --nocapture`:
  pass; 2 menu shortcut tests passed.
- `$env:CARGO_INCREMENTAL='0'; cargo test -p fret-imui begin_menu_helper_toggles -- --nocapture`:
  pass; 1 begin-menu action test passed.

## Menu Dispatch Entry Variant Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menu dispatch checked/action entry variants split into private owners without
changing plain menu-item routing, pressable-hook entry routing, checkbox/radio semantics, action
dispatch forwarding, label identity handling, mount routing, or public facade menu item behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries.rs` keeps plain
  menu-item entry routing, shared implementation forwarding, pressable-hook entry routing, and
  private re-exports.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries/checked.rs` owns
  checkbox/radio entry wrappers and checked-state semantics.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries/action.rs` owns action
  entry forwarding to the shared menu item implementation.
- `tools/gate_imui_workstream_source.py` now rejects checked/action entry variants from drifting
  back into `dispatch/entries.rs` and checks the dedicated variant owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `$env:CARGO_INCREMENTAL='0'; cargo nextest run -p fret-imui menu_item_checkbox --no-fail-fast`:
  pass; 1 checkbox semantics test passed.
- `$env:CARGO_INCREMENTAL='0'; cargo nextest run -p fret-imui menu_item_activate_shortcut begin_menu_helper_toggles --no-fail-fast`:
  pass; 3 menu action/shortcut tests passed.

## Facade Button-Action Inherent Wrapper Owner-Split Evidence - 2026-05-28

Claim verified: IMUI facade button action inherent wrappers split into a private owner without
changing plain button wrappers, command button forwarding, action dispatch, payload action dispatch,
focusable recording, response projection, or public facade method names.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` keeps ordinary button inherent
  wrappers and command button wiring.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions/action_methods.rs` owns
  `action_button`, `action_button_with_options`, `action_payload_button`, and
  `action_payload_button_with_options` inherent wrappers.
- `tools/gate_imui_workstream_source.py` now rejects action/payload wrappers from drifting back
  into `button_actions.rs` and checks the dedicated action-method owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui button_command button_shortcuts --no-fail-fast`: pass; 3
  button/command shortcut tests passed.

## Text-Picker Core Input-Root Owner-Split Evidence - 2026-05-28

Claim verified: IMUI text-picker core input-root phase split into a private owner without changing
model reads, candidate filtering, keyboard snapshot preparation, popup snapshot reads, expanded
semantics, input-root request construction, keyboard-handler installation, popup open policy, pick
response merge, or public text-picker facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` keeps model/candidate/keyboard/
  open-policy/popup/response orchestration.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/core/input_root.rs` owns prepared
  input-root request construction, root mounting, response extraction, and popup item test-id base
  forwarding.
- `tools/gate_imui_workstream_source.py` now rejects input-root request construction from drifting
  back into `text_picker_controls/core.rs` and checks the dedicated input-root owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass; 6 text-picker tests
  passed.

## Facade-Core Disabled-Scope Owner-Split Evidence - 2026-05-28

Claim verified: IMUI facade-core disabled-scope behavior split into a private owner without
changing `ImUiFacade` storage, keyed id helpers, `UiWriter` implementation, disabled-depth
handling, pointer event swallowing, opacity dimming, focus traversal gating, scoped runtime
preparation, or public facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/facade_core.rs` keeps the facade storage shape,
  focus recording, keyed id helpers, and `UiWriter` implementation.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/facade_core/disabled_scope.rs` owns
  `ImUiFacade::disabled_scope` behavior.
- `tools/gate_imui_workstream_source.py` now rejects disabled-scope behavior from drifting back
  into `facade_core.rs` and checks the dedicated disabled-scope owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui disabled_scope --no-fail-fast`: pass; disabled-scope hover
  behavior test passed.
- `cargo nextest run -p fret-imui control_disabled_state_changes_keep_outer_bounds_stable
  --no-fail-fast`: pass; disabled control-geometry test passed.

## Table Builder Row/Cell Owner-Split Evidence - 2026-05-28

Claim verified: IMUI table builder row and cell methods split into private owners without changing
`ImUiTable` / `ImUiTableRow` public methods, row key scopes, row/cell test-id derivation, child
IMUI mounting, cell packing, text-cell rendering, or table facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/builder.rs` keeps built row/cell data shapes and
  `build_table_rows`.
- `ecosystem/fret-ui-kit/src/imui/table_controls/builder/row_methods.rs` owns `row` /
  `row_with_options` row collection and keyed row scopes.
- `ecosystem/fret-ui-kit/src/imui/table_controls/builder/cell_methods.rs` owns `cell` /
  `cell_with_options` / `cell_text` / `cell_text_with_options` child mounting and cell packing.
- `tools/gate_imui_workstream_source.py` now rejects row/cell method bodies from drifting back into
  `builder.rs` and checks the dedicated row/cell owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`:
  pass; 9 table smoke tests passed.
- `cargo nextest run -p fret-ui-kit --features imui "imui::table_controls::tests"
  --no-fail-fast`: pass; 4 table control tests passed.

## Child-Region Resize-Stack Owner-Split Evidence - 2026-05-28

Claim verified: IMUI child-region resize stack assembly split into a private owner without
changing resizable child-region detection, scroll/content construction, resize handle test-id
defaults, X/Y resize response writes, stack layout refinement, stack root test-id routing, or
public child-region facade/response behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/child_region.rs` keeps option normalization, scroll owner
  dispatch, response aggregation, and the non-resizable vs resizable branch.
- `ecosystem/fret-ui-kit/src/imui/child_region/resize_stack.rs` owns resize handle test-id
  derivation, X/Y handle creation, stack layout/style projection, children ordering, and resizable
  root test-id stamping.
- `tools/gate_imui_workstream_source.py` now rejects resize-stack assembly from drifting back into
  `child_region.rs` and checks the dedicated resize-stack owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --no-fail-fast`: pass; 3 child-region smoke tests passed.
- `cargo nextest run -p fret-imui child_region --no-fail-fast`: pass; 6 child-region composition
  tests passed.

## Child-Region Scroll Owner-Split Evidence - 2026-05-28

Claim verified: IMUI child-region scroll/content/chrome assembly split into a private owner without
changing IMUI child mounting, scroll axis/options forwarding, framed/bare chrome, content/viewport/
root test-id routing, resize layout override, resize handle assembly, stack root test-id routing,
or public child-region facade/response behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/child_region.rs` keeps resize option detection, resize handle
  assembly, stack layout/test-id routing, and response aggregation.
- `ecosystem/fret-ui-kit/src/imui/child_region/scroll.rs` owns scroll-area builder construction,
  content mounting, framed chrome, handle forwarding, viewport test-id routing, and non-resizable
  root test-id stamping.
- `tools/gate_imui_workstream_source.py` now rejects scroll/content/chrome construction from
  drifting back into `child_region.rs` and checks the dedicated scroll owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --no-fail-fast`: pass; 3 child-region smoke tests passed.
- `cargo nextest run -p fret-imui child_region --no-fail-fast`: pass; 6 child-region composition
  tests passed.

## Popup-Modal Request Owner-Split Evidence - 2026-05-28

Claim verified: IMUI popup-modal overlay identity and request submission split into a private
owner without changing overlay id naming, modal root naming, trigger forwarding, open-model
forwarding, instant modal presence, layer children, dismiss request forwarding, initial focus
handoff, or public popup modal facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` keeps open-state gating,
  layout/dismiss/layer owner dispatch, and final request input assembly.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/request.rs` owns modal overlay id/root-name
  construction and `OverlayRequest::modal` submission.
- `tools/gate_imui_workstream_source.py` now rejects modal overlay identity and request submission
  from drifting back into `popup_overlay/modal.rs` and checks the dedicated request owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 popup/hover tests
  passed.

## Popup-Modal State Owner-Split Evidence - 2026-05-28

Claim verified: IMUI popup-modal open and keep-alive state handling split into a private owner
without changing popup store model identity, closed-modal render gating, keep-alive generation
writeback, modal root naming, dismiss policy, layer assembly, focus initialization, overlay request
assembly, or public popup modal facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` keeps dismiss policy creation, overlay
  identity/root naming, layout owner dispatch, layer owner dispatch, overlay request assembly, and
  final focus target selection.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/state.rs` owns modal open-model lookup,
  is-open reads, and keep-alive generation writeback.
- `tools/gate_imui_workstream_source.py` now rejects modal popup-store reads and keep-alive
  writeback from drifting back into `popup_overlay/modal.rs` and checks the dedicated state owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 popup/hover tests
  passed.

## Popup-Modal Layer Owner-Split Evidence - 2026-05-28

Claim verified: IMUI popup-modal layer and panel assembly split into a private owner without
changing popup store open state reads, keep-alive generation, modal root naming, backdrop barrier
behavior, centered panel layout, facade child mounting, focus initialization, overlay request
assembly, or public popup modal facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` keeps open-state gating, keep-alive
  writeback, dismiss policy creation, overlay request assembly, and final focus target selection.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layer.rs` owns modal layer/root mounting,
  barrier construction, panel semantics mounting, facade child rendering, focus-state
  construction, and panel focus handoff.
- `tools/gate_imui_workstream_source.py` now rejects modal layer/panel assembly from drifting
  back into `popup_overlay/modal.rs` and checks the dedicated layer owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 popup/hover tests
  passed.

## Popup-Modal Dismiss Request Policy Owner-Split Evidence - 2026-05-28

Claim verified: IMUI popup-modal dismiss request policy split into a private owner without changing
popup store open state reads, keep-alive generation, modal root naming, backdrop/panel assembly,
Escape close behavior, outside-press close option, dismiss prevention, focus initialization, or
public popup modal facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` keeps open-state gating, keep-alive
  writeback, layer/panel assembly, overlay request assembly, and focus initialization.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/dismiss.rs` owns modal `OnDismissRequest`
  policy for Escape, optional outside press, and default prevention.
- `tools/gate_imui_workstream_source.py` now rejects modal dismiss policy from drifting back into
  `popup_overlay/modal.rs` and checks the dedicated dismiss owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 popup/hover tests
  passed.

## Button Pressable Props/A11y Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button pressable props and a11y assembly split into a private owner without
changing enabled/focusable projection, variant layout application, button a11y metadata, chrome
assembly, activation/keyboard/response dispatch, or public button facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` keeps chrome owner dispatch,
  behavior owner dispatch, response projection dispatch, and visual resolution.
- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior/props.rs` owns `PressableProps`
  construction, focusable gating, variant layout application, and button a11y metadata.
- `tools/gate_imui_workstream_source.py` now rejects button pressable props/a11y assembly from
  drifting back into `button_controls/behavior.rs` and checks the dedicated props owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass; 10 shortcut tests
  passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 button API smoke test passed.

## Button Pressable Response Projection Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button pressable response projection split into a private owner without
changing clicked transient consumption, shared pressable item response population, lifecycle/hover/
drag response projection, visual assembly, or public button facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` keeps pressable props/chrome
  assembly, activation/keyboard owner dispatch, and visual assembly.
- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior/response.rs` owns button clicked
  transient consumption and shared `PressableItemResponseInput` projection.
- `tools/gate_imui_workstream_source.py` now rejects button response projection from drifting back
  into `button_controls/behavior.rs` and checks the dedicated response owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass; 10 shortcut tests
  passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 button API smoke test passed.

## Button Pressable Activation Behavior Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button pressable activation behavior split into a private owner without
changing pressable props/chrome assembly, keyboard lifecycle marking, clicked transient recording,
action dispatch, response population, or public button facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` keeps pressable props/chrome
  assembly, keyboard owner dispatch, response population, and visual assembly.
- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior/activation.rs` owns pressable
  activate-hook installation, keyboard activation lifecycle marking, clicked transient recording,
  action dispatch, and notify.
- `tools/gate_imui_workstream_source.py` now rejects pressable activation handlers from drifting
  back into `button_controls/behavior.rs` and checks the dedicated activation owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass; 10 shortcut tests
  passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 button API smoke test passed.

## Button Pressable Keyboard Behavior Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button pressable keyboard behavior split into a private owner without
changing pressable props/chrome assembly, action dispatch, shortcut repeat policy, keyboard
lifecycle marking, context-menu key handling, response population, or public button facade
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` keeps pressable props/chrome
  assembly, action activation, response population, and visual assembly.
- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior/keyboard.rs` owns focused button
  activate-shortcut handling, keyboard lifecycle marking, shortcut action dispatch, and keyboard
  context-menu requests.
- `tools/gate_imui_workstream_source.py` now rejects button key handlers from drifting back into
  `button_controls/behavior.rs` and checks the dedicated keyboard owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast`: pass; 10 shortcut tests
  passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 button API smoke test passed.

## Debug-Draw Round Path-Command Dispatch Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw round path-command paint dispatch split into stroked and filled
private owners without changing public draw-list commands, round path paint routing, stroke/fill
painter calls, command fallthrough, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/round.rs` now
  only dispatches to stroked and filled round command owners.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/round/stroked.rs`
  owns circle, ngon, and ellipse stroke paint routing.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/round/filled.rs`
  owns circle, ngon, and ellipse fill routing.
- `tools/gate_imui_workstream_source.py` now rejects concrete round command paint routing from
  drifting back into `path_commands/round.rs` and checks the dedicated stroked/filled owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast`: pass; 39
  debug-draw tests passed.

## Debug-Draw Linear Path-Command Dispatch Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw linear path-command paint dispatch split into stroked and filled
private owners without changing public draw-list commands, path paint routing, stroke/fill painter
calls, command fallthrough, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/linear.rs` now
  only dispatches to stroked and filled linear command owners.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/linear/stroked.rs`
  owns line, polyline, rect, quad, and triangle stroke paint routing.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/linear/filled.rs`
  owns convex/concave polygon, quad-fill, and triangle-fill routing.
- `tools/gate_imui_workstream_source.py` now rejects concrete linear command paint routing from
  drifting back into `path_commands/linear.rs` and checks the dedicated stroked/filled owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast`: pass; 39
  debug-draw tests passed.

## Debug-Draw Geometry Summary Projection Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw geometry summary projection split into linear, mesh, round, and
Bezier private owners without changing public command summaries, point/vertex/index/triangle
counts, clip-state projection, media/text summary projection, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry.rs`
  now only dispatches to geometry family owners.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry/linear.rs`
  owns line/polyline/polygon/rect/quad/triangle summary counts.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry/mesh.rs`
  owns rect multi-color and triangle-mesh vertex/index/triangle summary counts.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry/round.rs`
  owns circle/ngon/ellipse summary counts.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry/beziers.rs`
  owns quadratic/cubic Bezier summary counts.
- `tools/gate_imui_workstream_source.py` now rejects concrete geometry command summary counts from
  drifting back into `summary_projection/geometry.rs` and checks each dedicated family owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast`: pass; 39
  debug-draw tests passed.

## Debug-Draw Stroked Linear Path Painter Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw stroked linear path painters split into line/polyline and
rect/quad/triangle private owners without changing public draw-list commands, path command
generation, shared stroke style dispatch, canvas path dispatch, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear.rs` keeps
  private re-exports.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear/line_poly.rs`
  owns line and polyline stroke painting.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear/rect_quad_triangle.rs`
  owns rect, quad, and triangle stroke painting.
- `tools/gate_imui_workstream_source.py` now rejects stroked linear path painter bodies from
  drifting back into `stroked/linear.rs` and checks the dedicated line/poly and
  rect/quad/triangle owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast`: pass; 39
  debug-draw tests passed.

## Debug-Draw List Summary Classification Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw list summary command-kind classification split into a private owner
without changing public summary accessors, aggregate counts, clip-stack depth accounting, command
summary shape, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/list.rs` keeps aggregate counters,
  public accessors, final clip-depth writeback, and include-time counter updates.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/list/classification.rs` owns
  command-kind to list summary class mapping.
- `tools/gate_imui_workstream_source.py` now rejects the command-kind classification table from
  drifting back into `summaries/list.rs` and checks the dedicated classification owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui summaries --no-fail-fast`: pass; 4
  debug-draw summary tests passed.

## Text-Picker Popup Item Owner-Split Evidence - 2026-05-28

Claim verified: IMUI text-picker popup item rendering and pick commit split into a private owner
without changing popup lifecycle, keyboard handler installation, candidate filtering, selectable
row presentation, active-descendant writeback, model update, popup close, or public picker response
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/popup.rs` keeps popup lifetime, keyboard
  handler installation, and aggregate pick result merging.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/popup/item.rs` owns selectable candidate
  rows, item test-id derivation, active element writeback, model update, popup close, and click
  pick result.
- `tools/gate_imui_workstream_source.py` now rejects selectable row/model update bodies from
  drifting back into `popup.rs` and checks the dedicated item owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass; 6 text-picker popup
  and keyboard tests passed.

## Table Header Resize Grip Visual Owner-Split Evidence - 2026-05-28

Claim verified: IMUI table header resize grip visual split into a private owner without changing
pointer region hit width, resize drag lifecycle hooks, cursor behavior, drag response edge merging,
resize test-id attachment, or table column resize public behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/header/resize.rs` keeps pointer-region drag setup,
  cursor behavior, response writeback, drag response edge merging, and test-id attachment.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header/resize/visual.rs` owns resize grip color,
  disabled alpha, and visual dimensions.
- `tools/gate_imui_workstream_source.py` now rejects grip visual bodies from drifting back into
  `header/resize.rs` and checks the dedicated visual owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui table_resizable_header_reports_drag_response --no-fail-fast`:
  pass; 1 table resize drag response test passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_resizable_column_api_compiles --no-fail-fast`:
  pass; 1 table resize API smoke test passed.

## Debug-Draw Filled Path Painter Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw filled path painters split into polygon-fill and round-fill private
owners without changing public draw-list commands, path command generation, shared fill style,
canvas path dispatch, summaries, or debug-draw smoke behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled.rs` keeps the
  shared fill style and private re-exports.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled/polygons.rs` owns
  convex/concave/quad/triangle fill painting.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled/round.rs` owns
  circle/ngon/ellipse fill painting.
- `tools/gate_imui_workstream_source.py` now rejects filled path painter bodies from drifting back
  into `filled.rs` and checks the dedicated polygon/round fill owners.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast`: pass; 39
  debug-draw tests passed.

## Disclosure Header Metrics Owner-Split Evidence - 2026-05-28

Claim verified: IMUI disclosure header indicator, padding, and border metrics split into a private
owner without changing header row composition, palette resolution, indicator glyph text role, tree
row label text role, or tree-node/collapsing-header public behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/header.rs` keeps palette lookup, row
  element composition, glyph/text rendering, and spacer layout.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/header/metrics.rs` owns indicator
  glyph selection, tree indentation padding, and header border edges.
- `tools/gate_imui_workstream_source.py` now rejects indicator/padding/border metric bodies from
  drifting back into `visual/header.rs` and checks the dedicated metrics owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui disclosure_indicator_uses_shared_chrome_glyph_text_role tree_row_label_uses_shared_list_row_text_role tree_node_default_options_start_at_level_one tree_node_leaf_uses_tree_item_semantics --no-fail-fast`:
  pass; 6 focused disclosure/header tests passed.

## Menu-Item Command Helper Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menu-item command presentation, shortcut defaulting, and enabled gating split
into a private owner without changing public menu item wrapper methods, focusable-recording
behavior, command metadata lookup, shortcut propagation, or action menu item dispatch.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs` keeps public menu item wrappers,
  focusable recording, and the private helper re-export.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items/command.rs` owns command presentation
  lookup, enabled gating, shortcut fallback, and action menu item dispatch.
- `tools/gate_imui_workstream_source.py` now rejects command presentation lookup and shortcut
  fallback from drifting back into `menu_items.rs` and checks the dedicated helper owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui menu_item_command_uses_command_metadata_shortcut_and_gating --no-fail-fast`:
  pass; 1 menu command metadata/shortcut/gating test passed.

## Tooltip Runtime Layout Owner-Split Evidence - 2026-05-28

Claim verified: IMUI tooltip runtime layout/placement calculation split into a private owner
without changing trigger-id validation, event/open model setup, pointer-move open gate installation,
interaction update, open model synchronization, overlay request submission, or public tooltip
facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime.rs` keeps trigger gates, interaction
  updates, open state writeback, and overlay request submission.
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime/layout.rs` owns anchor bounds,
  measured/estimated panel sizing, and floating bounds calculation.
- `tools/gate_imui_workstream_source.py` now rejects tooltip layout/placement calculation from
  drifting back into `runtime.rs` and checks the dedicated layout owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui tooltip --no-fail-fast`: pass; 32 tooltip
  tests passed.

## Button-Command Helper Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button-command presentation and enabled gating split into a private owner
without changing public button wrapper methods, focusable-recording behavior, command metadata
lookup, or action button dispatch.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` keeps public button wrappers
  and the private helper re-export.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions/button_command.rs` owns command
  presentation lookup and enabled gating.
- `tools/gate_imui_workstream_source.py` now rejects command presentation lookup and action-button
  dispatch from drifting back into `button_actions.rs` and checks the dedicated helper owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui button_command_uses_command_metadata_and_gating --no-fail-fast`:
  pass; 1 button-command metadata/gating test passed.

## Pressable Drag State-Machine Owner-Split Evidence - 2026-05-28

Claim verified: IMUI pressable drag state machine split into a private interaction-runtime owner
without changing drag kind derivation, theme threshold reads, pointer-down active item marking,
long-press cancellation/arming, thresholded move transitions, drag started/stopped transient events,
pointer-up cleanup, or public response drag state.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs` keeps drag kind/threshold helpers
  and private sub-owner re-exports.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/pressable.rs` owns pressable pointer
  down/move/up drag state transitions, long-press timer coordination, active item cleanup, and
  drag-started/stopped transient events.
- `tools/gate_imui_workstream_source.py` now rejects thresholded pressable drag state-machine
  bodies from drifting back into `interaction_runtime/drag.rs` and checks the dedicated pressable
  owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass; 8 drag interaction tests
  passed.

## Drag-Source Payload Lifecycle Owner-Split Evidence - 2026-05-28

Claim verified: IMUI drag-source payload lifecycle hooks split into a private owner without
changing drag-source trigger-id gating, enabled/cross-window pointer-down policy, active payload
tracking, hovered-target preservation, drop delivery writeback, or public drag/drop response
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks.rs` keeps enabled gating, cross-window
  pointer-down drag upgrade policy, and delegates payload lifecycle hook installation.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks/payload_lifecycle.rs` owns pointer-move
  active payload tracking, hovered-target preservation, and pointer-up delivery insertion.
- `tools/gate_imui_workstream_source.py` now rejects active payload / delivered payload writeback
  from drifting back into `drag_drop/source/hooks.rs` and checks the dedicated payload lifecycle
  owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass; 8 drag interaction tests
  passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_drop_smoke --test imui_drag_preview_smoke --no-fail-fast`:
  pass; 2 drag/drop API smoke tests passed.

## Table-Column Visibility Menu Item Owner-Split Evidence - 2026-05-28

Claim verified: IMUI table-column visibility menu-item toggle behavior split into a private owner
without changing header context-menu trigger selection, menu item group composition, test-id suffix
generation, shared visibility state updates, changed/edited response flags, or public visibility
helper behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu.rs` keeps header context-menu
  orchestration, item group composition, identity/test-id filtering, and the private item-owner
  re-export.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu/item.rs` owns single checkbox item
  rendering, visible-state reads, model mutation, and changed/edited response flags.
- `tools/gate_imui_workstream_source.py` now rejects checkbox item rendering/state mutation from
  drifting back into `menu.rs` and checks the dedicated item owner.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui table_column_visibility --no-fail-fast`: pass; 4 table-column
  visibility behavior tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_column_visibility_helpers_compile --no-fail-fast`:
  pass; 1 public helper compile smoke passed.

## Menubar Active-Trigger Reconcile Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menubar active-trigger reconciliation split into a private owner without
changing open-menu synchronization, active-trigger installation, close-after-render reconciliation,
popup close restoration, or begin-menu public behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy/active_trigger.rs`
  keeps active-trigger open-menu sync, active-trigger installation, and the private reconcile
  re-export.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy/active_trigger/reconcile.rs`
  owns close-after-render reconciliation, popup close restoration, and open-menu/group-active
  cleanup.
- `tools/gate_imui_workstream_source.py` now rejects close/reconcile state cleanup from drifting
  back into `active_trigger.rs` and checks the dedicated reconcile owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18
  menu/submenu/tab tests passed.

## Begin Menu Capture Read Owner-Split Evidence - 2026-05-28

Claim verified: IMUI begin-menu capture read helpers split into a private owner without changing
row/popup/was-open model identity, open-menu model reads, render-state writeback, or menubar
open-policy behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/capture.rs` keeps
  `BeginMenuState`, `MenuRenderState`, row/popup/was-open model identity, render-state writeback,
  and read facade methods.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/capture/read.rs` owns bool and
  open-menu model reads for begin-menu capture/open-policy.
- `tools/gate_imui_workstream_source.py` now rejects direct `read_model` / `Invalidation::Paint`
  reads from drifting back into `capture.rs` and checks the dedicated read owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `git diff --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18
  menu/submenu/tab tests passed.

## Table Builder Test-Id Owner-Split Evidence - 2026-05-28

Claim verified: IMUI table builder row/cell test-id derivation split into a private owner without
changing public `ImUiTable` / `ImUiTableRow` methods, explicit row test-id override behavior,
default row/cell test-id strings, child `ImUiFacade` mounting, or table render behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/builder.rs` keeps row/cell collection, keyed row
  scopes, child mounting, `cell::pack_cell_children(...)`, `text_table_cell(...)`, and the public
  table-builder method surface.
- `ecosystem/fret-ui-kit/src/imui/table_controls/builder/test_ids.rs` owns explicit row test-id
  fallback and default `.row.*` / `.cell.*` test-id derivation.
- `tools/gate_imui_workstream_source.py` now rejects inline row/cell test-id format strings from
  drifting back into `builder.rs` and checks the dedicated test-id owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib imui::table_controls::tests --no-fail-fast`:
  pass; 4 table-control lib tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`:
  pass; 9 table smoke tests passed.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls --no-fail-fast`: first run
  timed out at 124 seconds and is not counted as evidence; rerun with a longer timeout passed 7
  matched tests, then the precise `imui::table_controls::tests` filter above was used as evidence.

## Selectable Popup-Nav Owner-Split Evidence - 2026-05-28

Claim verified: IMUI selectable popup-menu keyboard navigation split into a private owner without
changing selectable shortcut activation, popup close-on-shortcut, context-menu key handling,
inherited popup menu item registration, Arrow/Home/End focus movement, or public selectable/menu
item behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/selectable_controls/keyboard.rs` keeps shortcut activation,
  popup close-on-shortcut, and context-menu key handling.
- `ecosystem/fret-ui-kit/src/imui/selectable_controls/keyboard/popup_nav.rs` owns inherited popup
  menu nav item registration and Arrow/Up/Home/End focus movement.
- `tools/gate_imui_workstream_source.py` now rejects popup nav store/focus movement from drifting
  back into `keyboard.rs` and checks the dedicated popup-nav owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18 menu/submenu/tab
  tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib selectable_controls::keyboard --no-fail-fast`:
  not counted as evidence; the filter matched 0 tests and nextest returned `error: no tests to run`.

## Menu-Family Trigger Menubar Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menu-family trigger menubar behavior split into a private owner without
changing active trigger install/population, click/shortcut activation, menubar trigger-row registry
sync, patient-click timer wiring, ArrowDown/ArrowUp open behavior, or public menu facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior.rs` keeps base
  active-trigger behavior, shortcut activation, click transient recording, and response
  population.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior/menubar.rs` owns menubar
  trigger-row registration, state sync, patient-click timer wiring, toggle-on-activate, and
  ArrowDown/ArrowUp open support.
- `tools/gate_imui_workstream_source.py` now rejects menubar trigger-row wiring from drifting back
  into `behavior.rs` and checks the dedicated menubar owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18 menu/submenu/tab
  tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_family_controls::tests --no-fail-fast`:
  pass; 1 menu-family visual role test passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Disclosure Trigger Response Owner-Split Evidence - 2026-05-28

Claim verified: IMUI disclosure trigger response projection split into a private owner without
changing disclosure pointer/shortcut behavior, context-menu and double-click transient signaling,
hover-state projection, active-item hover blocking, or public disclosure facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger/behavior.rs` keeps pressable/key/
  pointer hook installation and delegates response projection.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger/behavior/response.rs` owns trigger
  response population, hover query hook attachment, active-item hover blocking, and response
  sanitization.
- `tools/gate_imui_workstream_source.py` now rejects response projection from drifting back into
  `behavior.rs` and checks the dedicated response owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib disclosure_controls::tests --no-fail-fast`:
  pass; 6 disclosure behavior tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::tabs --no-fail-fast`: timed out while
  waiting on Cargo build locks; not used as evidence.

## Slider Pointer Value-Update Owner-Split Evidence - 2026-05-28

Claim verified: IMUI slider pointer value-update logic split into a private owner without changing
pointer down/move/up capture, active-item mutation, lifecycle activation/deactivation, changed
response emission, or slider pointer behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer.rs` keeps pointer hook
  installation, active-item updates, capture/release, focus, lifecycle activation/deactivation,
  and transient change emission.
- `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer/value_update.rs` owns
  pointer-to-value projection, clamp/snap, and changed-detection writes.
- `tools/gate_imui_workstream_source.py` now rejects pointer value-update math/write logic from
  drifting back into `pointer.rs` and checks the dedicated value-update owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_controls::slider --no-fail-fast`: pass; 2 slider behavior
  tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Combo Trigger Visual Owner-Split Evidence - 2026-05-28

Claim verified: IMUI combo trigger visual props, chrome, and children assembly split into a
private visual owner without changing ComboBox semantics, trigger activation behavior,
open/close toggling, shortcut handling, preview/label rendering, or public combo facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/combo_controls/trigger.rs` keeps trigger behavior installation
  and visual-owner dispatch.
- `ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/visual.rs` owns trigger props, field
  chrome lookup, visual children assembly, and the a11y label helper.
- `tools/gate_imui_workstream_source.py` now rejects trigger props/chrome/children assembly from
  drifting back into `trigger.rs` and checks the dedicated visual owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib combo_trigger_a11y_label --no-fail-fast`:
  pass; 2 combo trigger a11y tests passed.
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass; 11 combo facade behavior
  tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `git diff --check`: pass.

## Popup Menu Overlay Request Owner-Split Evidence - 2026-05-28

Claim verified: IMUI popup-menu overlay request assembly split into a private request owner without
changing overlay id/root naming, popup open model forwarding, trigger fallback, auto-focus targets,
focus-outside submenu preservation, menubar close-auto-focus suppression, submenu pointer move
handler installation, modal flag forwarding, or public popup/menu facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu.rs` keeps overlay id/root creation,
  popup/menubar policy lookup, panel build orchestration, and request owner dispatch.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/request.rs` owns open model lookup, trigger
  fallback, initial focus targets, dismiss/close-auto-focus handlers, submenu pointer-move handler
  installation, modal flag forwarding, and `OverlayController::request(...)` submission.
- `tools/gate_imui_workstream_source.py` now rejects request assembly and handler logic from
  drifting back into `menu.rs` and checks the dedicated request owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 popup/hover tests passed.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18 menu/submenu/tab
  tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Text Picker Keyboard Preparation Owner-Split Evidence - 2026-05-28

Claim verified: IMUI text-picker keyboard preparation split into a private core sub-owner without
changing keyboard model identity, enabled/empty/exact-match reconciliation, pending keyboard pick
projection, active descendant element projection, input-root forwarding, popup forwarding, or
public text-picker facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` keeps model reads, candidate
  visibility, input-root mounting, open-policy application, popup rendering, and pick response
  merging.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/core/keyboard_state.rs` owns keyboard model
  creation, keyboard state reconciliation, active source projection, pending pick projection, and
  active element projection.
- `tools/gate_imui_workstream_source.py` now rejects keyboard model creation/reconciliation state
  mutation from drifting back into `core.rs` and checks the dedicated keyboard preparation owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass; 6 text-picker facade
  behavior tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib text_picker --no-fail-fast`: not counted
  as evidence; the filter matched 0 tests and nextest returned `error: no tests to run`.

## Child-Region Resize Handle Owner-Split Evidence - 2026-05-28

Claim verified: IMUI child-region resize pointer-handle behavior split into a private handle owner
without changing X/Y resize response setup, min/max forwarding, handle layout/axis constants,
pointer-region drag start/move/up behavior, cursor selection, drag response population, started/
stopped edge synthesis, or handle test-id stamping.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/child_region/resize.rs` keeps X/Y resize entry points and
  response option/min/max wiring.
- `ecosystem/fret-ui-kit/src/imui/child_region/resize/handle.rs` owns the shared pointer-region
  handle, drag-kind setup, drag threshold handling, pointer down/move/up hooks, drag response
  projection, edge synthesis, and handle test-id stamping.
- `ecosystem/fret-ui-kit/src/imui/child_region/resize/axis.rs` remains the axis layout/cursor owner.
- `tools/gate_imui_workstream_source.py` now rejects pointer/drag handle behavior from drifting
  back into `resize.rs` and checks the dedicated handle owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui layout_collections --no-fail-fast`: pass; 28 layout/collection
  tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib child_region --no-fail-fast`: pass; 2
  child-region resize response tests passed. Initial parallel attempt timed out while waiting on
  Cargo locks; the single rerun passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Submenu State Owner-Split Evidence - 2026-05-28

Claim verified: IMUI submenu state mutation split into private clear/select owners without
changing submenu open-value updates, trigger updates, geometry clearing, pending-open cleanup,
pointer-grace timer cleanup, close/focus/open timer cleanup, focus retry reset, or submenu
selection writes.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state/clear.rs` owns submenu clear
  behavior, including open/trigger matching, optional geometry clearing, pending-open cleanup,
  pointer-grace cleanup, close/focus/open timer cleanup, and focus retry reset.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state/select.rs` owns submenu
  selection writes and pending/open-timer cleanup.
- `tools/gate_imui_workstream_source.py` now checks clear/select owners separately and rejects
  submenu state bodies from drifting back into the root index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18 menu/submenu/tab
  tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu Keyboard Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menu-item keyboard behavior split into private popup-menu and menubar owners
without changing popup item nav registration, Arrow/Home/End focus movement, shortcut activation,
popup-close-on-key activation, lifecycle instant marking, menubar horizontal-arrow suppression, or
menubar primitive wiring.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard.rs` is now a private module/re-export
  index.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard/popup.rs` owns popup-menu item
  registration, menu-item shortcut activation, popup close on keyboard activation, and popup item
  focus movement.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard/menubar.rs` owns menubar horizontal-arrow
  close-focus suppression and primitive trigger-row horizontal switching wiring.
- `tools/gate_imui_workstream_source.py` now checks popup/menubar keyboard owners separately and
  rejects keyboard behavior bodies from drifting back into the root index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`: pass; 39
  menu/submenu/popup-hover tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Interaction Runtime Models Owner-Split Evidence - 2026-05-28

Claim verified: IMUI interaction runtime model helpers split into private element, window, scope,
and state owners without changing context-menu anchor model creation, long-press signal storage,
pointer-click modifier storage, lifecycle session storage, active-item per-window storage, float
window collapsed storage, disabled-scope depth reads, or public interaction runtime surfaces.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models/element.rs` owns element-scoped model
  stores for context-menu anchors, long-press signals, pointer-click modifiers, lifecycle
  sessions, and collapsed float windows.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models/window.rs` owns the window-scoped
  active-item store.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models/scope.rs` owns the disabled-scope
  depth store.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/models/state.rs` owns the public state
  shapes for long-press, lifecycle, and active-item models.
- `tools/gate_imui_workstream_source.py` now checks the new owner files and rejects the old
  storage/helper bodies from drifting back into the root index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_press interaction_drag floating popup_hover
  models_text_lifecycle --no-fail-fast`: pass; 65 interaction/floating/text-lifecycle tests
  passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Input Text Props Owner-Split Evidence - 2026-05-28

Claim verified: IMUI input-text props and assistive-semantics assembly split into a private owner
without changing model reads, response lifecycle population, select-all-on-focus effect dispatch,
input filters/custom filter ordering, password mode, accessibility metadata, placeholder/command
forwarding, compact IMUI chrome/style, or public input-text surfaces.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_controls/input.rs` keeps input model reads, response
  lifecycle population, select-all-on-focus command emission, text-input mounting, and policy
  command installation.
- `ecosystem/fret-ui-kit/src/imui/text_controls/input/props.rs` owns
  `InputTextAssistiveSemantics`, `TextInputProps` construction, built-in/custom insert filters,
  password-mode projection, accessibility metadata, placeholder/submit/cancel forwarding, and
  compact IMUI input chrome/style.
- `tools/gate_imui_workstream_source.py` now rejects props/filter/style bodies from drifting back
  into `input.rs` and checks the dedicated props owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib text_controls::tests
  --no-fail-fast`: pass; 3 text-control tests passed.
- `cargo nextest run -p fret-imui models_text_area models_text_picker models_text_filters
  --no-fail-fast`: pass; 17 text model/picker/filter tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text Policy Command Owner-Split Evidence - 2026-05-28

Claim verified: IMUI text-control policy command installation split into private input and textarea
owners without changing input completion/history/undo/redo key handling, repeat gating, IME/Alt/Meta
guards, textarea Enter/CtrlEnter/Escape capture policy, repeat-consume semantics, or public text
control surfaces.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_controls/policy_commands.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/text_controls/policy_commands/input.rs` owns input text
  completion/history/undo/redo key-down command dispatch.
- `ecosystem/fret-ui-kit/src/imui/text_controls/policy_commands/textarea.rs` owns textarea
  submit/cancel key-down capture command dispatch.
- `tools/gate_imui_workstream_source.py` now rejects input/textarea command bodies from drifting
  back into the root policy index and checks each owner independently.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib text_controls::tests
  --no-fail-fast`: pass; 3 text-control tests passed.
- `cargo nextest run -p fret-imui models_text_area models_text_picker models_text_filters
  --no-fail-fast`: pass; 17 text model/picker/filter tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Column Visibility State Owner-Split Evidence - 2026-05-28

Claim verified: IMUI table-column visibility state methods split into private override, snapshot-IO,
and column-application owners without changing public state constructors/accessors, empty-id
filtering, last-entry-wins behavior, snapshot roundtrips, table-column visibility application, or
opaque state storage.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state.rs` now owns only the
  state/override storage shape and public snapshot re-export.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state/overrides.rs` owns runtime
  override construction, mutation, query, toggle, removal, and clear behavior.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state/snapshot_io.rs` owns snapshot
  conversion and restoration.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state/columns.rs` owns `TableColumn`
  application.
- `tools/gate_imui_workstream_source.py` now rejects override/snapshot/column-application bodies
  from drifting back into `state.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_column_visibility
  --no-fail-fast`: pass; 7 table-column visibility tests passed.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_visibility --no-fail-fast`: pass; 4 table smoke tests passed.
- `cargo nextest run -p fret-imui layout_collections table_column_visibility --no-fail-fast`:
  pass; 28 composition tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Child-Region Resize Response Owner-Split Evidence - 2026-05-28

Claim verified: IMUI child-region resize responses split into private X/Y response owners without
changing public `ChildRegionResizeXResponse` / `ChildRegionResizeYResponse` re-exports, enabled/
min/max accessors, drag edge accessors, drag delta/total projection, clamp-from-start helpers, or
opaque response fields.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize/x.rs` owns width-axis
  response projection and clamp tests.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize/y.rs` owns height-axis
  response projection and clamp tests.
- `tools/gate_imui_workstream_source.py` now checks the opaque X/Y response structs in their
  dedicated owner files and rejects response bodies from drifting back into `resize.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib child_region_resize --no-fail-fast`:
  pass; 2 response clamp tests passed.
- `cargo nextest run -p fret-imui layout_collections table_headers --no-fail-fast`: pass; 31
  composition/table-header tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Input Text Filter Owner-Split Evidence - 2026-05-28

Claim verified: IMUI input-text filter options split into private built-in and custom-filter owners
without changing `InputTextFilters` constructors, public filter flags, character filtering,
uppercase/no-blank behavior, `InputTextCustomFilter` closure storage, debug output, or public text
option exports.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/text/filters.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/filters/builtin.rs` owns
  `InputTextFilters` plus decimal/scientific/hex/uppercase/no-blank character filtering.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/filters/custom.rs` owns
  `InputTextCustomFilter` closure storage and debug output.
- `tools/gate_imui_workstream_source.py` now rejects built-in/custom filter bodies from drifting
  back into `filters.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_filters --no-fail-fast`: pass; 3 text-filter tests
  passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug Draw Draw-List Image-Authoring Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw draw-list image authoring split into private mesh, raster, and
rounded-image owners without changing `ImUiDebugDrawList` image method names, default option
forwarding, command payload variants, vertex/index collection, image-region/quad recording, rounded
command recording, summaries, paint dispatch, or public debug-draw APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/images.rs` is now a private module
  index.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/images/mesh.rs` owns image
  triangle-mesh command recording.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/images/raster.rs` owns image,
  image-region, and image-quad command recording.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/images/rounded.rs` owns rounded
  image/region command recording.
- `tools/gate_imui_workstream_source.py` now rejects image-authoring command bodies from drifting
  back into `draw_list/images.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw --no-fail-fast`: pass;
  38 debug-draw tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug Draw Paint-Helper Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw paint helpers split into private media, mesh, and rounded-corner
owners without changing opacity sanitization, UV validation, rounded-corner projection, triangle
mesh filtering, image triangle mesh filtering, raster image scene ops, region scene ops, or public
debug-draw APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_helpers.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_helpers/media.rs` owns opacity/UV
  validation plus raster image scene ops.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_helpers/meshes.rs` owns vertex-color
  and image triangle mesh scene ops.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_helpers/rounded.rs` owns rounded-corner
  visibility/projection.
- `tools/gate_imui_workstream_source.py` now rejects media/mesh/rounded helper bodies from
  drifting back into `paint_helpers.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw --no-fail-fast`: pass;
  38 debug-draw tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug Draw Path-Builder Shape-Method Owner-Split Evidence - 2026-05-28

Claim verified: IMUI debug-draw path-builder shape methods split into private rect, Bezier, and
arc owners without changing `ImUiDebugDrawPath` method names, path point storage, invalid input
handling, default segment fallback, rounded-rect sampling, Bezier sampling, arc sampling, or public
debug-draw APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods.rs` is now a
  private module index.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods/rects.rs` owns
  rect and rounded-rect point appending.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods/beziers.rs` owns
  quadratic/cubic Bezier sampling.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods/arcs.rs` owns
  circular, fast 12-step, and elliptical arc sampling.
- `tools/gate_imui_workstream_source.py` now rejects rect/Bezier/arc shape-method bodies from
  drifting back into `shape_methods.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_path_builder
  --no-fail-fast`: pass; 12 path-builder tests passed.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Shared Hover-Delay State Owner-Split Evidence - 2026-05-28

Claim verified: IMUI shared hover-delay state/store/model lookup moved into a private owner without
changing window-scoped shared-delay model allocation, short/normal delay flags, hover-enter timer
scheduling, hover-leave clear timer scheduling, clear-timer cancellation, or shared delay reads.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay.rs` keeps hover-enter/
  leave shared timer policy and clear-timer handling.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay/state.rs` owns
  `ImUiSharedHoverDelayState`, `ImUiSharedHoverDelayStore`, `model_for_window`, and `delay_flags`.
- `tools/gate_imui_workstream_source.py` now rejects shared hover-delay state/store/model lookup
  bodies from drifting back into `shared_delay.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Hover Query Delay Read Owner-Split Evidence - 2026-05-28

Claim verified: IMUI hover query delay read/projection moved into a private owner without changing
hover hook installation, stationary/short/normal delay timers, shared-delay timers, long-press
timer handling, active-item blocking, transient consumption, or `HoverQueryDelayRead` values.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs` keeps active-item blocking,
  hover-change hook installation, timer dispatch, shared-delay delegation, and long-press
  delegation.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/read.rs` owns local hover-delay state,
  transient consumption, shared-delay flag reads, and `HoverQueryDelayRead` projection.
- `tools/gate_imui_workstream_source.py` now rejects hover delay read/projection bodies from
  drifting back into `hover.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Layout Sugar Scoped/Spacer Owner-Split Evidence - 2026-05-28

Claim verified: IMUI porting layout sugar split into private scoped-layout and spacer owners
without changing `items`, `same_line`, `dummy`, `spacing`, `indent`, layout-token defaults,
explicit dummy sizing, indent composition, test-id stamping, or public-in-IMUI APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/layout_sugar.rs` is now a private module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/layout_sugar/scoped.rs` owns item-flow, same-line, and indent
  container composition.
- `ecosystem/fret-ui-kit/src/imui/layout_sugar/spacers.rs` owns dummy/spacing spacer construction
  and default IMUI spacing token projection.
- `tools/gate_imui_workstream_source.py` now rejects scoped/spacer bodies from drifting back into
  `layout_sugar.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui
  porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text-Picker Keyboard Handler Owner-Split Evidence - 2026-05-28

Claim verified: IMUI input-text picker keyboard handler moved into a private owner without
changing keyboard navigation enablement, repeat/IME/modifier gating, arrow highlight movement,
Enter pick handling, popup close, model writes, or picker response projection.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard.rs` keeps keyboard pick/state/
  snapshot storage and reconciliation.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard/handler.rs` owns key-down capture,
  Arrow/Enter navigation and pick handling, repeat/IME/modifier gating, model writes, and popup
  close.
- `tools/gate_imui_workstream_source.py` now rejects key-down handler bodies from drifting back
  into `keyboard.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu Routing Dispatch Owner-Split Evidence - 2026-05-28

Claim verified: IMUI menu item routing dispatch split into private entry and core owners without
changing menu item method names, checkbox/radio/action semantics roles, action forwarding,
label-identity scoping, pressable hook injection, mount routing, response population, or
public-in-IMUI APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch.rs` is now a private
  module/re-export index.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries.rs` owns public-in-IMUI
  menu-item entry wrappers plus semantics/action selection.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/core.rs` owns no-op pressable
  hook and identity-to-mount dispatch.
- `tools/gate_imui_workstream_source.py` now rejects entry/core bodies from drifting back into
  `dispatch.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Facade Support Slider Math Owner-Split Evidence - 2026-05-28

Claim verified: IMUI slider math helpers moved into a private owner without changing slider range
normalization, step fallback, clamp/snap behavior, pointer-to-value mapping, slider a11y value
projection, slider pointer/keyboard interaction, or public-in-IMUI APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_support.rs` keeps writer bridge support, transient keys,
  runtime frame prep, device-pixel snapping, point arithmetic, and model-change detection.
- `ecosystem/fret-ui-kit/src/imui/facade_support/slider_math.rs` owns `slider_step_or_default`,
  `slider_normalize_range`, `slider_clamp_and_snap`, and `slider_value_from_pointer`.
- `tools/gate_imui_workstream_source.py` now rejects slider math bodies from drifting back into
  `facade_support.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui slider_lifecycle --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Drag Source Hook Owner-Split Evidence - 2026-05-28

Claim verified: IMUI drag source hook installation moved into a private owner without changing
drag source trigger-id validation, payload boxing, store pruning, drag kind selection,
cross-window drag promotion, active payload publication, pointer-up delivery, or
`DragSourceResponse` population.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/drag_drop/source.rs` keeps trigger validation, payload boxing,
  store model lifecycle/pruning, drag-kind selection, hook owner dispatch, and source response
  projection.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks.rs` owns enabled/cross-window policy,
  pointer-down cross-window promotion, pointer-move active payload publication, and pointer-up
  delivery insertion.
- `tools/gate_imui_workstream_source.py` now rejects drag source hook bodies from drifting back
  into `source.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_drop_smoke --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Interaction Lifecycle Response Owner-Split Evidence - 2026-05-28

Claim verified: IMUI interaction lifecycle response projection moved into a private owner without
changing lifecycle activation/deactivation/edit mutation, transient consumption, active-state frame
diffing, `ResponseExt` lifecycle signal setters/mergers, or public-in-IMUI APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs` keeps pointer-down/up lifecycle
  mutation, instant edit mutation, lifecycle edit mutation, and private re-exports for callers.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle/response.rs` owns
  transient-to-response population, active-state lifecycle frame diffing, edited-state stamping,
  and activated/deactivated merge application.
- `tools/gate_imui_workstream_source.py` now rejects lifecycle response projection from drifting
  back into `lifecycle.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui lifecycle --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Tooltip Overlay Request Owner-Split Evidence - 2026-05-28

Claim verified: IMUI tooltip overlay request assembly moved into a private owner without changing
trigger-id validation, pointer-move open gating, hover/focus interaction updates, panel layout,
dismiss behavior, hoverable-content pointer tracking, overlay request semantics, or public-in-IMUI
APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime.rs` keeps trigger-id validation,
  event/open models, pointer-move open gate installation, hover/focus update gates, interaction
  updates, panel-size/anchor projection, and open-state synchronization.
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/request.rs` owns panel child construction,
  tooltip overlay request creation, trigger binding, dismiss close-request signaling, optional
  hoverable-content pointer tracker installation, and request submission.
- `tools/gate_imui_workstream_source.py` now rejects tooltip request assembly from drifting back
  into `runtime.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib tooltip_overlay::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Area Layout Owner-Split Evidence - 2026-05-28

Claim verified: IMUI floating area shell layout and hit-test gate selection moved into a private
owner without changing floating-area registration, drag position reconciliation, no-input behavior,
hit-test passthrough behavior, area test IDs, response population, or public-in-IMUI APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_surface/area.rs` keeps layer child registration, drag
  snapshot/state reconciliation, child mounting, final test-id stamping, and
  `FloatingAreaResponse` construction.
- `ecosystem/fret-ui-kit/src/imui/floating_surface/area/layout.rs` owns absolute area layout props,
  `interactivity_gate_props` selection for `no_inputs`, `hit_test_gate_props` selection for
  hit-test passthrough, and the container fallback.
- `tools/gate_imui_workstream_source.py` now rejects floating-area layout/gate bodies from drifting
  back into `area.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating::input_modes --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Layer Z-Order Owner-Split Evidence - 2026-05-28

Claim verified: IMUI floating layer z-order state and snapshot projection moved into a private
owner without changing floating layer child registration, bring-to-front activation,
missing-window pruning, rank sorting, hit-test order, absolute layer layout, or floating layer
public-in-IMUI APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_surface/layer.rs` keeps layer marker state, child
  registration, activation dispatch, layer child mounting, rank sort application, and absolute fill
  layout.
- `ecosystem/fret-ui-kit/src/imui/floating_surface/layer/z_order.rs` owns
  `FloatWindowLayerZOrder`, z-order membership, bring-to-front reordering, missing-window pruning,
  and rank snapshot projection.
- `tools/gate_imui_workstream_source.py` now rejects z-order state/snapshot bodies from drifting
  back into `layer.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating::movement_z_order --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Shared Item Behavior Install Owner-Split Evidence - 2026-05-28

Claim verified: shared IMUI pressable item hook installation moved into a private owner without
changing shared button, checkbox/radio, selectable, combo, image-item, debug-draw pressable,
context-menu, pointer-click, double-click, drag, long-press, lifecycle, or response population
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/item_behavior.rs` keeps shared data shapes plus install/response
  re-exports.
- `ecosystem/fret-ui-kit/src/imui/item_behavior/install.rs` owns pressable pointer hook clearing,
  active-item/long-press/lifecycle model capture, and assembly; later pointer-hook sub-owner splits
  move down/move/up transient bodies into `item_behavior/install/*`.
- `ecosystem/fret-ui-kit/src/imui/item_behavior/response.rs` continues to own shared `ResponseExt`
  population, drag response merging, and hover query hook installation.
- `tools/gate_imui_workstream_source.py` now rejects hook-installation and pointer transient bodies
  from drifting back into `item_behavior.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test
  imui_combo_smoke --test imui_selectable_smoke --test imui_drag_drop_smoke --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_press interaction_drag --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Facade Floating/Popup Owner-Split Evidence - 2026-05-28

Claim verified: IMUI facade floating/popup thin forwarding moved into private floating, popup,
tooltip, drag/drop, and window owners without changing trait method names, facade forwarding, popup
open/close behavior, tooltip forwarding, drag/drop forwarding, or floating-window routing.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs` is now a private module/re-export
  index for the floating/popup facade cluster.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/floating.rs` owns floating
  layer/area forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/popup.rs` owns popup open/close,
  menu, modal, and context-menu forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/tooltip.rs` owns tooltip
  forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/drag_drop_facade.rs` owns drag/drop
  forwarding.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/window.rs` owns floating-window
  forwarding.
- `tools/gate_imui_workstream_source.py` now rejects forwarding bodies from drifting back into
  `floating_popup.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_tooltip_smoke --test
  imui_drag_drop_smoke --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui floating popup_hover --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Button Behavior Action Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button action payload storage and command dispatch moved into a private owner
without changing button pressable behavior, shortcut activation, command gating, action payload
forwarding, context-menu signaling, visual resolution, or `ResponseExt` population.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` keeps pressable props,
  shortcut/context-menu handlers, enabled gating, lifecycle marking, response population, and visual
  resolution.
- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior/action.rs` owns `ButtonAction`, action
  payload storage, command dispatch source recording, pending payload recording, and final action
  dispatch.
- `tools/gate_imui_workstream_source.py` now rejects action dispatch bodies from drifting back into
  `behavior.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui button_shortcuts --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui button_command_uses_command_metadata_and_gating --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Button Visual A11y/Variant Owner-Split Evidence - 2026-05-28

Claim verified: IMUI button visual a11y and variant layout/glyph policy moved into private owners
without changing public button APIs, button a11y labels, arrow a11y labels, arrow glyphs, variant
sizing, or chrome/content assembly.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` keeps `ButtonVisual`,
  `ButtonVisualContent`, chrome resolution, and visible/invisible content assembly.
- `ecosystem/fret-ui-kit/src/imui/button_controls/visual/a11y.rs` owns button `PressableA11y`
  construction, `SemanticsRole::Button`, custom label fallback, and arrow a11y labels.
- `ecosystem/fret-ui-kit/src/imui/button_controls/visual/variant.rs` owns variant sizing and arrow
  glyph selection.
- `tools/gate_imui_workstream_source.py` now rejects a11y and variant-layout bodies from drifting
  back into `visual.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Tab Item List/Panel Owner-Split Evidence - 2026-05-28

Claim verified: IMUI tab item list semantics and selected-panel rendering moved into private
owners without changing selected-model normalization, trigger response collection, focus fallback
behavior, tab-list semantics, tab-panel semantics, or public tab-bar APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items.rs` keeps `BuiltTabItem`,
  selected-model normalization, build-focus propagation, final column assembly, and
  `TabBarResponse` construction.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items/list.rs` owns tab-list semantics,
  trigger rendering, selected/first-focusable trigger tracking, and `TabTriggerResponse`
  collection.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items/panel.rs` owns selected tab-panel
  semantics and panel child mounting.
- `tools/gate_imui_workstream_source.py` now rejects tab-list and tab-panel rendering bodies from
  drifting back into `items.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib tab_family_controls --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text-Picker Core Owner-Split Evidence - 2026-05-28

Claim verified: IMUI input-text picker core orchestration moved into a private owner without
changing completion/history wrapper calls, candidate filtering, keyboard navigation, input root
semantics, popup open policy, popup pick handling, or `InputTextPickerResponse`.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` is now a private module index and
  re-export hub for the core picker and completion/history entry wrappers.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` owns model reads, candidate
  visibility, keyboard snapshot reconciliation, input root mounting, open-policy application,
  popup rendering, and pick response merging.
- `tools/gate_imui_workstream_source.py` now rejects picker orchestration bodies from drifting back
  into `text_picker_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Header-Cell Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table header cell layout and resize wrapping moved into a private owner
without changing sortable/plain header behavior, resize handle wiring, header test IDs, table
layout, or `TableHeaderResponse` collection.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` keeps sortable/plain header trigger
  orchestration and `BuiltHeaderCell` response assembly.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header/cell.rs` owns header cell layout,
  resize-handle attachment, resize test-id suffixing, and header content flex wrapping.
- `tools/gate_imui_workstream_source.py` now rejects header cell layout and resize wrapping bodies
  from drifting back into `header.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Path-Family Owner-Split Evidence - 2026-05-27

Claim verified: IMUI debug-draw path construction moved into private shape-family owners without
changing path helper names, validation behavior, sampling helpers, rect path construction, paint
dispatch, or debug-draw tests.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs` is now a private path-family
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/linear.rs` now indexes the linear path
  subowners; the 2026-05-28 follow-up moved polyline, fill, and primitive construction into
  `paths/linear/{polyline,fills,primitives}.rs`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/round.rs` now indexes the round path
  subowners; the 2026-05-28 follow-up moved circle, ngon, and ellipse construction into
  `paths/round/{circle,ngon,ellipse}.rs`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/beziers.rs` owns quadratic and cubic
  bezier path construction.
- `tools/gate_imui_workstream_source.py` now rejects linear, round, and bezier path construction
  bodies from drifting back into `paths.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests::paths
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Command Payload Owner-Split Evidence - 2026-05-27

Claim verified: IMUI debug-draw command payload variants moved into a private owner without
changing command variant names, draw-list recording paths, summary projection, paint dispatch,
public debug-draw summaries, or facade APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands.rs` keeps command module wiring,
  summary projection installation, and the parent-visible `DebugDrawCommand` re-export.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types.rs` owns private
  `DebugDrawCommand` payload variants.
- `tools/gate_imui_workstream_source.py` now rejects command payload variants from drifting back
  into `commands.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Builder Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table row/cell builder collection moved into a private owner without changing
`table(...)` / `table_with_options(...)` facade calls, public `ImUiTable` / `ImUiTableRow` method
names, row/cell test-id derivation, child `ImUiFacade` mounting, `cell_text(...)` text routing, or
final table rendering.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls.rs` keeps module wiring, public table builder
  re-exports, `table_element(...)`, and final render dispatch.
- `ecosystem/fret-ui-kit/src/imui/table_controls/builder.rs` owns `ImUiTable` / `ImUiTableRow`,
  built row/cell records, row/cell test-id derivation, child `ImUiFacade` mounting, and
  `cell_text(...)` table-cell text routing.
- `tools/gate_imui_workstream_source.py` now rejects table builder row/cell collection bodies from
  drifting back into root `table_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Input-Text Element Owner-Split Evidence - 2026-05-27

Claim verified: IMUI input-text element assembly moved into a private owner without changing
input-text facade calls, assistive semantics wiring for text pickers, response lifecycle
population, select-all-on-focus behavior, insert filters, submit/cancel command policy, compact
chrome, or text style selection.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_controls.rs` is now a private
  focus/input/policy/style/textarea module index and re-export hub.
- `ecosystem/fret-ui-kit/src/imui/text_controls/input.rs` owns input-text model element assembly,
  assistive semantics, response lifecycle population, select-all command emission, input filters,
  policy-command installation, and compact input chrome/style selection.
- `tools/gate_imui_workstream_source.py` now rejects input-text element/lifecycle/policy bodies
  from drifting back into root `text_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib text_controls::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu-Item Routing Dispatch Owner-Split Evidence - 2026-05-27

Claim verified: IMUI menu-item routing dispatch moved into a private owner without changing public
menu item entry points, checkbox/radio/action role selection, noop-hook routing, `##/###` label
identity scoping, pressable-hook forwarding, response assembly, or final element mounting.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` is now a private
  dispatch/identity/mount module index and re-export hub.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch.rs` owns public menu-item entry
  wrappers, checkbox/radio/action role selection, noop-hook routing, and identity-to-mount
  dispatch.
- `tools/gate_imui_workstream_source.py` now rejects public menu-item dispatch bodies from
  drifting back into root `menu_controls/routing.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Disclosure Layout Owner-Split Evidence - 2026-05-27

Claim verified: IMUI disclosure content/root layout moved into a private owner without changing
label identity parsing, open-model reads, trigger mounting, content body building, content/root
test IDs, open/toggled response population, or public disclosure facade calls.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs` keeps label identity parsing, open-model
  reads, trigger mounting, and aggregate `DisclosureResponse` writes.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/layout.rs` owns content container
  composition, body `ImUiFacade` construction, root column layout, and content/root test-id
  application.
- `tools/gate_imui_workstream_source.py` now rejects disclosure content/root layout bodies from
  drifting back into root `disclosure_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib disclosure_controls::tests
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text Picker Entry Owner-Split Evidence - 2026-05-27

Claim verified: IMUI input-text picker completion/history entry wrappers moved into a private
entry owner without changing public facade call paths, completion picker behavior, history picker
filter/open normalization, core picker orchestration, or final response shape.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` keeps core picker orchestration and
  re-exports the entry helpers.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/entry.rs` owns
  `input_text_completion_model_with_options(...)`, `input_text_history_model_with_options(...)`,
  and history option normalization.
- `tools/gate_imui_workstream_source.py` now rejects completion/history wrapper bodies from
  drifting back into root `text_picker_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Window Shell Props Owner-Split Evidence - 2026-05-27

Claim verified: IMUI floating-window shell frame/title/body props moved into a private owner
without changing window frame sizing, title-bar clipping/padding/border radii, collapsed sizing,
inner content clipping, blocker mounting, resize-stack composition, or public IMUI surface.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_shell.rs` keeps shell composition, blocker
  mounting, and resize-stack composition.
- `ecosystem/fret-ui-kit/src/imui/floating_window_shell/props.rs` owns
  `window_frame_props(...)`, `shell_column_props(...)`, `title_bar_container_props(...)`, and
  `clipped_body_props(...)`.
- `tools/gate_imui_workstream_source.py` now rejects shell props helpers from drifting back into
  `floating_window_shell.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui
  floating::window_options::floating_window_title_bar_double_click_toggles_collapsed
  floating::movement_z_order::floating_window_close_button_sets_open_false
  floating::input_modes::floating_window_activate_on_click_can_be_disabled_for_resize_handles
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu-Item Routing Mount Owner-Split Evidence - 2026-05-27

Claim verified: IMUI menu-item final mounting and response assembly moved into a private routing
owner without changing public menu item dispatch, semantic role/action selection, `##/###`
identity scoping, pressable-hook forwarding, final element insertion, or `ResponseExt` assembly.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` keeps public dispatch,
  checkbox/radio/action role selection, noop-hook routing, and label identity scoping.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/mount.rs` owns
  `element::menu_item_element_with_pressable_hook_inner(...)`, `ResponseExt::default()`, final
  `ui.add(...)`, and response return.
- `tools/gate_imui_workstream_source.py` now rejects direct menu-item element mounting or response
  assembly from drifting back into `routing.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls::tests --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui label_identity popup_hover::item_pointer
  interaction_menu_tabs::menu_activation::begin_menu_helper_toggles_popup_and_closes_after_command_activate
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Body-Row Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table body-row preparation moved into a private render owner without changing
hidden-column filtering, fallback empty cells, default/explicit test-id precedence,
striped/background selection, pinned/horizontal-scroll wrapping, or aggregate `TableResponse`
headers.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` keeps palette resolution,
  visible-column filtering for header/scroll decisions, header rendering, root chrome, semantics,
  and final `TableResponse` assembly.
- `ecosystem/fret-ui-kit/src/imui/table_controls/render/body_rows.rs` owns keyed body row
  assembly, cell iteration, hidden-column filtering, fallback empty-cell insertion, body cell
  wrapping, and body row wrapping.
- The source gate rejects direct body row preparation from drifting back into `render.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table-Column Visibility Response Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table-column visibility response structs and accessors moved into a private
response owner without changing public response type names, accessors, changed/clicked semantics,
opaque fields, menu item construction, or header context-menu response aggregation.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility.rs` keeps options, state re-exports,
  public helper forwarding, and tests.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/response.rs` owns
  `TableColumnVisibilityMenuResponse`, `TableColumnVisibilityHeaderContextMenuResponse`, and
  `TableColumnVisibilityMenuItemResponse` plus their public accessors.
- `tools/gate_imui_workstream_source.py` now requires the response owner split and keeps the
  response structs in the opaque-struct catalog.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_column_visibility::tests
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Control Chrome Palette/Button/Field Owner-Split Evidence - 2026-05-27

Claim verified: IMUI shared control chrome palette, button theme resolution, and field theme
resolution moved into narrower private owners without changing `control_chrome::button_chrome`,
`field_chrome`, `ImUiControlPalette`, dense button/field chrome defaults, theme token fallback
order, or caller paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/control_chrome/chrome.rs` is now a private module/re-export
  index.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/chrome/palette.rs` owns
  `ImUiControlPalette`.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/chrome/button.rs` owns button theme resolution
  and compact button chrome props.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/chrome/field.rs` owns field theme resolution and
  fill-width field chrome props.
- The source gate rejects theme resolution, palette records, or chrome prop bodies from drifting
  back into `control_chrome/chrome.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib control_chrome::tests --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Container Element Owner-Split Evidence - 2026-05-27

Claim verified: IMUI child building, linear container composition, scroll container composition,
and grid row assembly moved into private owners without changing public horizontal, vertical, grid,
and scroll helpers, option forwarding, test-id placement, viewport test-id placement, or
build-focus propagation.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/containers.rs` is now a private module/re-export index plus
  tests.
- `ecosystem/fret-ui-kit/src/imui/containers/children.rs` owns child `ImUiFacade` mounting and
  build-focus propagation.
- `ecosystem/fret-ui-kit/src/imui/containers/linear.rs` owns horizontal/vertical flex container
  construction.
- `ecosystem/fret-ui-kit/src/imui/containers/scroll.rs` owns scroll-area construction and viewport
  test-id forwarding.
- `ecosystem/fret-ui-kit/src/imui/containers/grid.rs` owns grid child collection, row batching,
  keyed row assembly, and outer grid container construction.
- The source gate rejects container implementation bodies from drifting back into `containers.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib containers::tests --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Title-Bar Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI floating-window title-bar behavior moved into a private owner without changing
title text roles, drag-surface wiring, double-click collapse signaling, Escape close behavior,
close-button activation, or close-glyph rendering.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs` keeps title-bar row composition,
  title text-role selection, close-button prop selection, and close-glyph text construction.
- `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar/behavior.rs` owns double-click
  collapse event recording, title-bar Escape close key behavior, close-button activation wiring,
  and model update/notify calls.
- `tools/gate_imui_workstream_source.py` now requires the title-bar behavior owner and rejects
  inline title-bar key/press behavior bodies from drifting back into `floating_window_title_bar.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib imui::floating_window_title_bar::tests
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  floating::movement_z_order::floating_window_close_button_sets_open_false
  floating::window_options::floating_window_closable_false_hides_close_button_and_escape_does_not_close
  floating::window_options::floating_window_title_bar_double_click_toggles_collapsed
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Content Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI floating-window content-surface behavior moved into a private owner without
changing content scrolling, content container padding, background-click focus behavior,
activate-on-click z-order behavior, or input-disabled bypass behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_content.rs` keeps content scroll/container
  composition and IMUI child mounting.
- `ecosystem/fret-ui-kit/src/imui/floating_window_content/behavior.rs` owns content-surface
  pointer-region wrapping, focusable key stub installation, background-click focus requests,
  activate-on-click event recording, and float-layer bring-to-front delegation.
- `tools/gate_imui_workstream_source.py` now requires the content behavior owner and rejects
  inline content pointer/focus/activation behavior bodies from drifting back into
  `floating_window_content.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui
  floating::input_modes::floating_window_activate_on_click_can_be_disabled_for_content
  floating::input_modes::floating_window_focus_on_click_can_be_independent_from_z_order_activation
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Tab Trigger Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI tab trigger behavior moved into a private owner without changing tab a11y,
active-trigger lifecycle, selected-tab model writes, activate-shortcut handling, clicked response
population, or tab visual construction.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger.rs` keeps tab trigger props, tab
  collection a11y, keyed trigger assembly, and visual mounting.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger/behavior.rs` owns active-trigger
  behavior installation, keyboard lifecycle marking, selected-model writes, activate-shortcut
  handling, clicked transient reads, and `ResponseExt` population.
- `tools/gate_imui_workstream_source.py` now requires the tab trigger behavior owner and rejects
  inline active-trigger behavior bodies from drifting back into `tab_family_controls/trigger.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib tab_family_controls --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui
  interaction_menu_tabs::tabs::tab_bar_helper_switches_selected_panel_and_updates_selection_model
  interaction_menu_tabs::tabs::tab_item_activate_shortcut_is_scoped_to_focused_trigger
  interaction_menu_tabs::tabs::tab_bar_helper_reports_selected_change_and_trigger_edges
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Drag Surface Owner-Split Evidence - 2026-05-27

Claim verified: IMUI floating-area drag-surface behavior moved into a private owner without
changing drag setup delegation, focusable key stub installation, double-click hooks, activation
signals, drag threshold handling, or IMUI child mounting.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` is now a private floating-surface module
  index and re-export hub for area, drag-surface, kinds, layer, and state owners.
- `ecosystem/fret-ui-kit/src/imui/floating_surface/drag_surface.rs` owns
  `floating_area_drag_surface_element(...)`, pointer-region wiring, double-click dispatch,
  activation event recording, pointer drag move/up handling, setup callback invocation, and IMUI
  child mounting into the drag surface.
- `tools/gate_imui_workstream_source.py` now requires the drag-surface owner and rejects
  pointer-region drag behavior from drifting back into `floating_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui
  floating::movement_z_order::floating_area_bring_to_front_updates_hit_test_order
  floating::movement_z_order::floating_layer_bring_to_front_updates_hit_test_order
  floating::window_options::floating_window_title_bar_double_click_toggles_collapsed
  floating::input_modes::floating_window_activate_on_click_can_be_disabled_for_resize_handles
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Flow Options Owner-Split Evidence - 2026-05-27

Claim verified: IMUI flow/layout option records and token defaults moved into narrower private
owners without changing option type names, fields, defaults, token keys, or public re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/containers/flow.rs` is now a private module/re-export
  index.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacing.rs` owns the IMUI layout-token
  defaults.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline.rs` is the current inline option
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline/item_flow.rs` owns
  `ItemFlowOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline/same_line.rs` owns
  `SameLineOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear.rs` is the current linear option
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear/horizontal.rs` owns
  `HorizontalOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear/vertical.rs` owns
  `VerticalOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer.rs` is the current spacer option
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer/dummy.rs` owns `DummyOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer/spacing.rs` owns
  `SpacingOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer/indent.rs` owns `IndentOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/flow/grid.rs` owns `GridOptions`.
- `tools/gate_imui_workstream_source.py` now requires the flow owner split and rejects option
  records or token-default bodies from drifting back into the root flow index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib containers --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  porting_sugar_items_same_line_spacing_dummy_and_indent_use_imgui_style_layout_tokens
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Popup Menu Panel Layout/Content Owner-Split Evidence - 2026-05-27

Claim verified: IMUI popup-menu panel popper/layout/chrome and content-provider nesting moved into
private owners without changing popup panel placement, menu semantics, nav-state installation,
IMUI child mounting, or focus target extraction.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel.rs` keeps popup open/anchor lifecycle
  reads, keepalive updates, nav-state installation, panel id storage, and `PopupMenuBuilt`
  assembly.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel/layout.rs` owns popper placement, menu
  semantics layout, panel palette/chrome, and panel column props.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel/content.rs` owns popup/menubar policy
  provider nesting and IMUI child mounting.
- `tools/gate_imui_workstream_source.py` now requires the popup panel layout/content owners and
  rejects popper, semantics, chrome, and content rendering bodies from drifting back into
  `panel.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui
  popup_menu_uses_environment_viewport_bounds_for_popper_outer_bounds --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Checkbox Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI checkbox pressable behavior moved into a private owner without changing label
identity, checkbox a11y, model toggling, shortcut gating, context-menu keyboard requests, changed
response population, field chrome, or visual row layout.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox.rs` keeps label identity,
  `CheckboxOptions` a11y wiring, field chrome, checkbox indicator mounting, boolean label mounting,
  and fill-row visual assembly.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox/behavior.rs` owns pressable behavior
  installation, activate/shortcut model toggling, context-menu key handling, transient changed
  reads, and `ResponseExt` population.
- `tools/gate_imui_workstream_source.py` now requires the checkbox behavior owner and rejects
  pressable behavior bodies from drifting back into `checkbox.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui models_controls::checkbox
  interaction_shortcuts::disclosure_tree::checkbox_activate_shortcut_preserves_shift_f10_context_menu_request
  interaction_press::lifecycle::checkbox_lifecycle_reports_edit_and_deactivated_after_edit
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Radio Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI radio pressable behavior moved into a private owner without changing label
identity, radio a11y, shortcut gating, context-menu keyboard requests, click response population,
field chrome, or visual row layout.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/boolean_controls/radio.rs` keeps label identity, `RadioOptions`
  a11y wiring, field chrome, radio indicator mounting, boolean label mounting, and fill-row visual
  assembly.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls/radio/behavior.rs` owns pressable behavior
  installation, activate/shortcut click signaling, context-menu key handling, transient clicked
  reads, and `ResponseExt` population.
- `ecosystem/fret-imui/src/tests/models_controls/radio.rs` covers click, shortcut, and Shift+F10
  behavior at the thin `fret-imui` facade layer.
- `tools/gate_imui_workstream_source.py` now requires the radio behavior owner and rejects
  pressable behavior bodies from drifting back into `radio.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit -p fret-imui`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui models_controls::radio --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui models_controls::radio label_identity::model_controls
  composition::control_geometry::button_family_variants_and_radio_mount_with_expected_bounds
  composition::control_geometry::control_disabled_state_changes_keep_outer_bounds_stable
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Image Item Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI image-item pressable behavior moved into a private owner without changing
image/image-button roles, focusability, context-menu keyboard requests, activation lifecycle
marking, pointer-click reporting, click response population, item sizing, chrome, or image props.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs` keeps a11y props, size props, key
  activation policy for plain images, chrome mounting, and image visual assembly.
- `ecosystem/fret-ui-kit/src/imui/image_item_controls/behavior.rs` owns pressable behavior
  installation, keyboard-activation lifecycle marking, context-menu key handling, transient clicked
  reads, and `ResponseExt` population.
- `ecosystem/fret-imui/src/tests/models_controls/image_item.rs` covers image-button click and
  Shift+F10 behavior at the thin `fret-imui` facade layer.
- `tools/gate_imui_workstream_source.py` now requires the image-item behavior owner and rejects
  pressable behavior bodies from drifting back into `image_item_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit -p fret-imui`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_image_item_smoke
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui models_controls::image_item --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Selectable Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI selectable pressable behavior moved into a private owner without changing
label identity, selectable a11y, pointer click reporting, keyboard activation lifecycle marking,
popup close-on-activate behavior, shortcut/context-menu/nav delegation, response population, or row
visual composition.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs` keeps label identity,
  `SelectableOptions` a11y wiring, selected/highlighted state reads, and row visual assembly.
- `ecosystem/fret-ui-kit/src/imui/selectable_controls/behavior.rs` owns pressable behavior
  installation, activate-handler popup close/click signaling, keyboard owner delegation, transient
  clicked reads, and `ResponseExt` population.
- `ecosystem/fret-ui-kit/src/imui/selectable_controls/keyboard.rs` continues to own shortcut
  handling, context-menu requests, and popup menu arrow-key navigation.
- `tools/gate_imui_workstream_source.py` now requires the selectable behavior owner and rejects
  pressable behavior bodies from drifting back into `selectable_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_shortcuts::selectable_shortcuts
  interaction_drag::multi_select models_combo --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Header Trigger Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table header trigger behavior moved into a private owner without changing
sortable/plain trigger props, primary activation policy, keyboard lifecycle marking, context-menu
request propagation, plain-header click suppression, response population, or sortable header visual
layout.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/header/trigger.rs` keeps header trigger props,
  a11y/key-activation policy, keyed surface assembly, and sortable header visual construction.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header/trigger/behavior.rs` owns active-trigger
  behavior installation, sortable keyboard activation lifecycle marking, clicked transient draining
  for plain headers, and `ResponseExt` population.
- `tools/gate_imui_workstream_source.py` now requires the table header trigger behavior owner and
  rejects active-trigger behavior bodies from drifting back into `trigger.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::layout_collections::table_plain_header_left_click_does_not_activate_or_click
  composition::layout_collections::table_plain_header_reports_context_menu_request_from_keyboard_without_clicking
  composition::layout_collections::table_sortable_header_reports_context_menu_request_from_keyboard
  composition::layout_collections::table_sortable_header_reports_context_menu_request_on_right_click
  label_identity::table_headers::table_sortable_header_reports_app_owned_trigger_without_sorting_rows
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Element Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI debug-draw pressable element behavior moved into a private owner without
changing noninteractive canvas output, pressable canvas wrapping, keyboard activation lifecycle
marking, pointer-click reporting, click response population, cache policy, clipping, or paint
routing.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element.rs` keeps canvas composition,
  fill-layout policy for interactive canvases, cache policy, clipping, test-id routing, and
  debug-draw command painting.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element/behavior.rs` owns pressable behavior
  installation, keyboard activation lifecycle marking, clicked transient reads, and `ResponseExt`
  population.
- `tools/gate_imui_workstream_source.py` now requires the debug-draw element behavior owner and
  rejects pressable behavior bodies from drifting back into `element.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests::element
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Media Paint Owner-Split Evidence - 2026-05-27

Claim verified: debug-draw media paint routing moved into private raster, rounded, and SVG owners
without changing image/region/quad/SVG paint behavior, opacity filtering, UV validation, rounded
clip push/pop balancing, or no-op routing for non-media commands.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` keeps
  `paint_debug_draw_media_command(...)` routing plus shared media paint key/type aliases.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media/raster.rs` owns image,
  image-region, and image-quad paint behavior.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media/rounded.rs` owns rounded
  image/region validation plus clip push/pop balancing.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media/svg.rs` owns SVG image and
  mask-icon paint behavior.
- `tools/gate_imui_workstream_source.py` now rejects paint bodies from the root media router and
  requires each private media paint owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls --no-fail-fast`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Row-Group Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table row-group pinned-cell splitting, row flex layout, and center horizontal
scroll wrapping moved into private owner modules without changing table row/header layout,
pinned-column ordering, horizontal-scroll wrapping, hidden-column behavior, or table public APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/row_groups.rs` keeps
  `wrap_pinned_table_row_groups(...)` orchestration.
- `ecosystem/fret-ui-kit/src/imui/table_controls/row_groups/split.rs` owns pinned-cell detection
  and left/center/right grouping.
- `ecosystem/fret-ui-kit/src/imui/table_controls/row_groups/layout.rs` owns row outer/fill/pinned
  and scroll-content horizontal flex wrappers.
- `ecosystem/fret-ui-kit/src/imui/table_controls/row_groups/scroll.rs` owns center horizontal
  scroll wrapping.
- `tools/gate_imui_workstream_source.py` now rejects split/layout/scroll bodies from the root row
  group orchestrator and requires the private owners.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls --no-fail-fast`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Draw-List Linear Owner-Split Evidence - 2026-05-27

Claim verified: debug-draw draw-list linear shape authoring moved into private line/poly and
rect/quad/triangle owners without changing public authoring method names, command payloads,
summary projection, or paint behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear.rs` is now a private
  module index.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear/line_poly.rs` owns
  line, polyline, convex polygon fill, and concave polygon fill command recording.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear/rect_quad_triangle.rs`
  owns rect, quad, triangle, and filled variant command recording.
- `tools/gate_imui_workstream_source.py` now rejects command recording from the root linear module
  and requires each shape-family owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests::draw_list --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table-Column Method-Family Owner-Split Evidence - 2026-05-27

Claim verified: IMUI `TableColumn` construction, identity/accessor, visibility, sort, resize, and
pin helper methods moved into private owner modules without changing public type names, method
names, chainable builder behavior, stable-id inference, primitive re-exports, or table composition
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/collections/table_column.rs` keeps the `TableColumn`
  storage shape and primitive re-exports.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table_column/construction.rs` owns
  `px(...)`, `fill(...)`, `weighted(...)`, and `unlabeled(...)`.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table_column/identity.rs` owns explicit ids,
  header/id accessors, width access, and stable-id inference from label identity.
- `visibility.rs`, `sorting.rs`, `resize.rs`, and `pinning.rs` own the corresponding method
  families.
- `tools/gate_imui_workstream_source.py` now rejects method bodies from the root `table_column.rs`
  storage owner and requires each method family in its private owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke table_column --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_column --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Drag/Drop Store Owner-Split Evidence - 2026-05-27

Claim verified: IMUI drag/drop store state, lifecycle cleanup, source response projection, and
target payload lookup moved into private owners without changing source/target public helpers,
payload downcast behavior, stale-session pruning, or one-tick delivered-payload expiry.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/drag_drop/store/state.rs` owns the shared model plus
  active/delivered payload records.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/store/lifecycle.rs` owns global model creation and
  stale active-session plus delivered-payload pruning.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/store/source_response.rs` owns source response
  projection from the active drag store.
- `ecosystem/fret-ui-kit/src/imui/drag_drop/store/target_payloads.rs` owns active preview payload
  and delivered payload lookup/downcast behavior.
- `tools/gate_imui_workstream_source.py` now requires the narrower owners and rejects state/query
  bodies from the root `drag_drop/store.rs` re-export index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib drag_drop --no-fail-fast`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_drop_smoke --no-fail-fast`:
  pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Summary Projection Owner-Split Evidence - 2026-05-27

Claim verified: debug-draw command summary geometry and clip-state projection moved into private
owners without changing command summary values, channel assignment, clip stack depth, media
summaries, or list summary behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry.rs`
  owns point/vertex/index/triangle-count summaries for geometric commands.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/clip_state.rs`
  owns push/pop/current clip rect projection and clip-depth updates.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection.rs` keeps
  `summary_with_clip_state(...)`, media summary routing, text summary routing, and clip command
  kind routing.
- `tools/gate_imui_workstream_source.py` now requires the geometry and clip-state owners while
  rejecting geometric summary bodies and clip-depth mutation from the root summary projection file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls --no-fail-fast`:
  pass; 38 tests.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass; 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Canonical Workbench Teaching-Doc Refresh Evidence - 2026-05-27

Claim verified: first-party teaching docs no longer claim the canonical workbench delegates to the
workspace shell owner after the route moved to direct editor-notes workflow mounting.

Evidence:

- `docs/examples/README.md`, `apps/fret-cookbook/README.md`, and
  `apps/fret-cookbook/EXAMPLES.md` now describe `imui_editor_workbench_demo` as the canonical
  editor route that mounts editor-notes directly.
- The same docs keep `imui_editor_proof_demo` as the supporting dense panel / explicit stable
  identity proof instead of restoring it as the canonical route.
- `tools/gate_imui_facade_teaching_source.py` now requires the canonical workbench/supporting-proof
  wording and the direct editor-notes workflow wording.

Focused gates:

- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast`:
  pass; 2 tests.
- `python -m py_compile tools\gate_imui_facade_teaching_source.py tools\gate_imui_workstream_source.py`:
  pass.
- `python tools\gate_imui_facade_teaching_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## P0/P2 Canonical Workbench Current-Status Refresh Evidence - 2026-05-27

Claim verified: the active gap map no longer treats `imui_editor_proof_demo` as the canonical
editor-panel route after the golden-path workbench lane closed.

Evidence:

- `docs/workstreams/imui-imgui-gap-closure-v1/P0_CURRENT_SOURCE_AUDIT_2026-05-06.md` now names
  `cargo run -p fret-demo --bin imui_editor_workbench_demo` as the canonical product-facing editor
  workbench route.
- `docs/workstreams/imui-imgui-gap-closure-v1/TODO.md` keeps `imui_editor_proof_demo`,
  `workspace_shell_demo`, and docking demos as supporting proof surfaces.
- `apps/fret-examples/tests/imui_editor_workbench_golden_path_surface.rs` verifies the workbench
  route, direct editor workflow mounting, style/theme preset picker integration, export path,
  `fret-demo` dispatch, docs promotion, and Demo/Metrics/Debug discovery.
- `tools/gate_imui_workstream_source.py` now rejects the stale P0/P2 wording that said the default
  editor-panel story was still fragmented.

Focused gates:

- `cargo check -p fret-demo --bin imui_editor_workbench_demo`: pass.
- `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface --no-fail-fast`:
  pass; 2 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Path-Command Family Owner-Split Evidence - 2026-05-27

Claim verified: path-command dispatch is split by shape family without changing canvas keys, draw
order, path command routing, or the underlying path paint helpers.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/linear.rs` owns
  line, polyline, polygon, rect-outline, quad, and triangle command dispatch.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/round.rs` owns
  circle, ngon, and ellipse command dispatch.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/beziers.rs` owns
  quadratic and cubic bezier command dispatch.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands.rs` is now a thin
  family router, and `tools/gate_imui_workstream_source.py` freezes the owner split.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls --no-fail-fast`:
  pass; 38 tests.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass; 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Path-Shape Dispatch Owner-Split Evidence - 2026-05-27

Claim verified: path-shape paint dispatch moved out of the root debug-draw paint-shapes module
without changing command order, canvas keys, stroke/fill geometry, mesh painting, text painting, or
media no-op routing.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands.rs` now owns
  line/polyline/polygon, rect-outline, quad, triangle, circle, ngon, ellipse, and bezier dispatch
  into the existing path paint owners.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes.rs` keeps draw-order/key setup,
  filled rect routing, mesh routing, text routing, and ignored media/clip command routing.
- `tools/gate_imui_workstream_source.py` now requires the path-command dispatch owner and rejects
  path paint dispatch from drifting back into `paint_shapes.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls --no-fail-fast`:
  pass; 38 tests.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass; 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu-Item Routing Owner-Split Evidence - 2026-05-27

Claim verified: menu item entry routing and label identity scoping moved out of the root menu
controls module without changing public menu item, checkbox, radio, action, or submenu
pressable-hook call paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` now owns menu item entry dispatch,
  checkbox/radio/action semantic role selection, `##/###` label identity parsing, item-id scoping,
  `ResponseExt` assembly, and final element insertion.
- `ecosystem/fret-ui-kit/src/imui/menu_controls.rs` is now a thin module/re-export index beside the
  existing element, interaction, keyboard, visual, and tests owners.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/tests.rs` no longer depends on the root module
  carrying routing helper imports.
- `tools/gate_imui_workstream_source.py` now requires the routing owner and rejects identity,
  element insertion, behavior, and visual bodies from drifting back into the root menu module.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls --no-fail-fast`: pass;
  4 tests.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::submenu_hover --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu-Item Routing Identity Owner-Split Evidence - 2026-05-27

Claim verified: menu item label identity scoping moved into a private owner without changing public
menu item dispatch, `##/###` visible-label semantics, stable item-id scoping, response assembly, or
final element insertion.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` keeps public menu item entry dispatch,
  checkbox/radio/action semantic role selection, `ResponseExt` assembly, and final element
  insertion.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/routing/identity.rs` owns
  `parse_label_identity(...)`, visible label extraction, and `menu-item-label` `push_id` scoping.
- `tools/gate_imui_workstream_source.py` now requires the routing identity owner and rejects
  direct label parsing or menu-item identity `push_id` scoping from drifting back into
  `routing.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls::tests --no-fail-fast`:
  pass; 4 tests, 685 skipped.
- `cargo nextest run -p fret-imui label_identity popup_hover::item_pointer
  interaction_menu_tabs::menu_activation::begin_menu_helper_toggles_popup_and_closes_after_command_activate
  --no-fail-fast`: pass; 13 tests, 173 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## P3 Component Catalog Current-Gap Refresh Evidence - 2026-05-27

Claim verified: the active component-surface catalog no longer sends future work toward stale
ListBox, plot adapter, or style/theme preset candidate lanes after those proof lanes landed.

Evidence:

- `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md` now
  records ListBox as a kit-owned container proof through `list_box`, `list_box_with_options`, and
  `ListBoxOptions`.
- The same catalog records plot parity as an optional `fret-plot/imui` adapter over declarative
  plot panels, keeping `fret-imui` and `fret-ui-kit::imui` free of plot dependencies.
- The same catalog records style/theme parity as editor-owned preset tooling through
  `fret-ui-editor::imui::editor_theme_preset_picker` and the canonical workbench, not as a runtime
  mutable style stack.
- `tools/gate_imui_workstream_source.py` now rejects the stale ListBox/plot/style candidate-only
  wording from returning.

Focused gates:

- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Active-Trigger Behavior Owner-Split Evidence - 2026-05-27

Claim verified: shared active-trigger keyboard, pointer, and response behavior moved into focused
private owners without changing active-item lifecycle, context-menu keyboard requests, secondary
click anchor signaling, hover response population, or the shared active-trigger call path used by
menus, tabs, switches, and table headers.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior/keyboard.rs` now owns ContextMenu and
  Shift+F10 request handling.
- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior/pointer.rs` now owns primary active-item
  pointer lifecycle, focus request policy, and secondary-click context-menu anchor signaling.
- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior/response.rs` now owns secondary-click and
  context-menu response fields, hover query hookup, and shared pressable response population.
- `ecosystem/fret-ui-kit/src/imui/active_trigger_behavior.rs` keeps handler clearing, model lookup,
  option/input structs, and owner dispatch.
- `tools/gate_imui_workstream_source.py` now requires the active-trigger sub-owners and rejects
  keyboard, pointer, and response bodies from drifting back into the root active-trigger module.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_controls::switch --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::tabs --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- An initial `models_controls::switch` run timed out after 184 seconds without test failure output;
  the same focused nextest command passed immediately on rerun.

## Tooltip Runtime Owner-Split Evidence - 2026-05-27

Claim verified: tooltip lifecycle/update/request orchestration moved out of the tooltip overlay
root into a private runtime owner without changing the public tooltip facade, text-tooltip helper,
pointer-open gating, panel composition, dismissal behavior, or hoverable-content tracking.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime.rs` now owns trigger-id validation,
  tooltip event model setup, interaction bounds calculation, open/update scheduling, open-model
  sync, dismiss request handling, hoverable-content tracking, and `request_tooltip(...)`
  orchestration.
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs` is now a thin module index that wires the
  runtime, trigger, panel, text, and tests owners.
- `tools/gate_imui_workstream_source.py` now requires the runtime owner and rejects runtime,
  trigger, panel, and text body responsibilities from drifting back into the root tooltip module.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_tooltip_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-ui-kit --features imui --lib tooltip_overlay::tests
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Slider Pointer/Keyboard Interaction Owner-Split Evidence - 2026-05-27

Claim verified: slider pointer and keyboard interaction moved out of the slider interaction entry
owner into private pointer/key owners without changing pointer capture, active-item state, pointer
model editing, keyboard step/page/home/end semantics, lifecycle edit signals, or changed response
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer.rs` now owns pointer
  down/move/up capture, active-item set/clear, pointer value projection, pointer model mutation,
  and pointer lifecycle edit signals.
- `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/keyboard.rs` now owns enabled
  keyboard gating, arrow/page/home/end value edits, snapping, and keyboard lifecycle edit signals.
- `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction.rs` keeps handler clearing,
  active/lifecycle model lookup, shared range input, and owner dispatch.
- `tools/gate_imui_workstream_source.py` now rejects pointer/key handler bodies from drifting back
  into `interaction.rs`, rejects keyboard behavior from the pointer owner, and rejects pointer
  capture behavior from the keyboard owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_controls::slider --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::control_geometry::menu_and_tab_trigger_state_changes_keep_outer_bounds_stable
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Begin-Menu Trigger Behavior Owner-Split Evidence - 2026-05-27

Claim verified: begin-menu trigger behavior moved out of the trigger shell into a private behavior
owner without changing menu trigger a11y, label identity, activate shortcut gating, keyboard
lifecycle marking, menubar registry synchronization, arrow-open behavior, click response
population, or trigger visual chrome.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior.rs` now owns
  active-trigger behavior installation, keyboard activation lifecycle marking, activate shortcut
  handling, menubar row registry/sync wiring, arrow-down/up open behavior, transient click reads,
  and trigger `ResponseExt` population.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger.rs` keeps label identity,
  `PressableA11y`, pressable shell construction, and `visual::menu_trigger_visual(...)` mounting.
- `tools/gate_imui_workstream_source.py` now rejects active-trigger/menubar-row/response
  population code from drifting back into `trigger.rs` and rejects a11y/visual shell code from
  drifting into the behavior owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Switch Behavior Owner-Split Evidence - 2026-05-27

Claim verified: switch pressable behavior moved out of the switch visual/model shell into a private
behavior owner without changing label identity, switch a11y, model toggling, shortcut gating,
active-trigger lifecycle semantics, changed/clicked response population, field chrome, or visual row
layout.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/boolean_controls/switch/behavior.rs` now owns active-trigger
  behavior installation, activate handler model toggling, shortcut model toggling, lifecycle edit
  marking, transient changed/clicked reads, and `ResponseExt` population.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls/switch.rs` keeps label identity,
  `SwitchOptions` a11y wiring, field chrome, switch state badge mounting, boolean label mounting,
  and fill-row visual assembly.
- `tools/gate_imui_workstream_source.py` now rejects active-trigger/model-toggle/response
  population code from drifting back into `switch.rs` and rejects switch a11y/visual assembly from
  drifting into the behavior owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_controls::switch --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::control_geometry::menu_and_tab_trigger_state_changes_keep_outer_bounds_stable
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Disclosure Trigger Behavior Owner-Split Evidence - 2026-05-27

Claim verified: disclosure trigger pressable behavior and trigger-response population moved out of
the trigger shell into a private behavior owner without changing activation toggles, shortcut
gating, context-menu keyboard requests, right-click anchor capture, double-click signaling,
hover-delay reads, enabled sanitization, or trigger `ResponseExt` population.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger/behavior.rs` now owns pressable
  callback installation, activate shortcut/context-menu key handling, pointer down/up hooks,
  hover-delay reads, context-menu anchor reporting, enabled sanitization, and trigger
  `ResponseExt` population.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger.rs` keeps pressable props, a11y,
  header visual mounting, collapsible trigger controls, and test-id application.
- `tools/gate_imui_workstream_source.py` now rejects behavior/response-population code from
  drifting back into `trigger.rs` and rejects visual/collapsible shell code from drifting into the
  behavior owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib disclosure_controls::tests
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_shortcuts::disclosure_tree --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Tab-Family Selection Owner-Split Evidence - 2026-05-27

Claim verified: selected-tab normalization moved out of the tab-family item/list owner into a
private sub-owner without changing selected fallback order, disabled-tab filtering,
selected-model correction, trigger response aggregation, focus fallback, tab-list semantics, panel
mounting, or `TabBarResponse` assembly.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items/selection.rs` now owns selected model
  reads, current-tab validity checks, default-selected fallback, first-enabled fallback, and model
  correction writes.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items.rs` keeps `BuiltTabItem`, trigger
  response aggregation, focus fallback, tab-list/panel assembly, and final `TabBarResponse`
  construction.
- `tools/gate_imui_workstream_source.py` now rejects selection model reads/writes from drifting
  back into `tab_family_controls/items.rs` and rejects tab-list/panel/trigger assembly from
  drifting into the selection owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::tabs --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::layout_collections::tab_bar_helper_arranges_tabs_horizontally_and_stamps_tab_semantics
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Control Chrome Palette Owner-Split Evidence - 2026-05-27

Claim verified: shared IMUI control palette/theme chrome moved behind a private owner without
changing `control_chrome::button_chrome`, `field_chrome`, `ImUiControlPalette`, text/layout helper
call paths, dense row/stack defaults, or button/combo/textarea/image-item smoke coverage.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` now keeps style constants, private module
  wiring, and re-exports the chrome/layout/text owners.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/chrome.rs` owns `ImUiControlPalette`, button
  theme color resolution, field theme color resolution, and compact button/field container chrome.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/layout.rs` and
  `ecosystem/fret-ui-kit/src/imui/control_chrome/text.rs` remain the row/stack layout and text/pill
  owners.
- `tools/gate_imui_workstream_source.py` now rejects palette/theme chrome from drifting back into
  the root `control_chrome.rs` index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui imui_control_chrome --no-fail-fast`: pass; 1
  test, 736 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test
  imui_combo_smoke --test imui_textarea_smoke --test imui_image_item_smoke --no-fail-fast`: pass;
  6 tests, 0 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Options Owner-Split Evidence - 2026-05-27

Claim verified: floating window options and floating area options/context moved behind separate
private owners without changing public re-export paths, option field names/defaults, builder
methods, `FloatingAreaContext` accessor-first shape, floating behavior tests, or title-bar chrome
text-role coverage.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_options.rs` is now a thin re-export index for the
  area/window option owners.
- `ecosystem/fret-ui-kit/src/imui/floating_options/window.rs` owns
  `FloatingWindowResizeOptions`, `FloatingWindowOptions`, `WindowOptions`, default behavior flags,
  resize defaults, and window option builder methods.
- `ecosystem/fret-ui-kit/src/imui/floating_options/area.rs` owns `FloatingAreaOptions`,
  `FloatingAreaContext`, area option defaults, and context accessors.
- `tools/gate_imui_workstream_source.py` now follows the opaque `FloatingAreaContext` check to the
  area owner and rejects the public option structs from drifting back into the root index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role --no-fail-fast`: pass; 1 test,
  736 skipped.
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass; 25 tests, 156 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Drag Response Source/Target Owner-Split Evidence - 2026-05-27

Claim verified: drag-source and typed drop-target response records moved behind private owners
without changing public re-export paths, accessor-first response shape, internal payload/position
visibility, `ResponseExt` drag accessors, drag/drop smoke coverage, or retained `fret-imui`
interaction-drag behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/drag.rs` now keeps the generic `DragResponse`
  edge/motion record and re-exports the source/target response owners.
- `ecosystem/fret-ui-kit/src/imui/response/drag/source.rs` owns `DragSourceResponse`, inactive/new
  constructors, and source read accessors.
- `ecosystem/fret-ui-kit/src/imui/response/drag/target.rs` owns `DropTargetResponse<T>`, empty
  construction, delivered/preview payload reads, position reads, source id reads, and session reads.
- `tools/gate_imui_workstream_source.py` now registers each opaque response owner separately and
  rejects source/target response structs from drifting back into the root drag response owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_drag_drop_smoke --no-fail-fast`:
  pass; 1 test, 0 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests, 0 skipped.
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass; 8 tests, 173 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Combo Trigger Behavior Owner-Split Evidence - 2026-05-27

Claim verified: ComboBox trigger behavior moved behind a private owner without changing trigger
props, ComboBox semantics, a11y label derivation, shortcut activation, context-menu request
handling, pressable response population, popup behavior, or model/direct combo interaction tests.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/combo_controls/trigger.rs` now keeps trigger options, pressable
  props, ComboBox a11y, chrome/pill visual assembly, and trigger a11y label derivation.
- `ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/behavior.rs` owns activate handling,
  activate-shortcut handling, context-menu shortcut handling, transient clicked/context-menu events,
  and `ResponseExt` population via shared pressable item behavior.
- `tools/gate_imui_workstream_source.py` now requires the behavior owner and rejects activate/key
  handlers plus `populate_pressable_item_response(...)` from the root trigger owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui combo_trigger --no-fail-fast`: pass; 2 tests,
  735 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_combo_smoke --no-fail-fast`: pass;
  2 tests, 0 skipped.
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass; 11 tests, 170 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Child-Region Resize Axis Owner-Split Evidence - 2026-05-27

Claim verified: child-region resize axis geometry moved behind a private owner without changing
resize handle keys, cursor selection, absolute handle layout, pointer-region drag wiring,
response writes, or child-region resize option smoke coverage.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/child_region/resize.rs` now keeps X/Y handle entry points,
  enabled/min/max response writes, pointer-region drag lifecycle wiring, and drag edge merging.
- `ecosystem/fret-ui-kit/src/imui/child_region/resize/axis.rs` owns X/Y handle width/height
  constants, axis keys, resize cursors, and absolute handle layout.
- `tools/gate_imui_workstream_source.py` now requires the axis owner and rejects layout constants,
  `ChildRegionResizeAxis`, `CursorIcon`, `InsetStyle`, `Length::Px`, and `PositionStyle` from the
  root resize owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast`: pass; 4
  tests, 733 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --no-fail-fast`: pass; 3 tests, 0 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Child-Region Resize Response Owner-Split Evidence - 2026-05-27

Claim verified: child-region X/Y resize response records moved behind a private owner without
changing public re-export paths, aggregate `ChildRegionResponse` accessors, X/Y drag accessors,
min/max clamping helpers, or existing resize option smoke coverage.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region.rs` now keeps aggregate
  `ChildRegionResponse` storage, aggregate accessors, and re-exports the resize response types.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize.rs` owns
  `ChildRegionResizeXResponse`, `ChildRegionResizeYResponse`, drag accessors, min/max accessors,
  width/height clamping helpers, and the clamping unit tests.
- `tools/gate_imui_workstream_source.py` now checks the aggregate response and resize response
  owners separately, including opaque field checks for the moved resize records.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast`: pass; 4
  tests, 733 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --no-fail-fast`: pass; 3 tests, 0 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text-Picker Pick-Response Merge Owner-Split Evidence - 2026-05-27

Claim verified: IMUI input-text picker picked-response merge moved behind a private owner without
changing model re-read timing, element-id change detection, `changed`/`edited`/
`deactivated_after_edit` merge semantics, popup rendering, keyboard navigation, or final
`InputTextPickerResponse` shape.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` now keeps input/popup orchestration and
  delegates picked-response mutation to a private response owner.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/response.rs` owns selected-value re-read,
  `model_value_changed_for(...)` lookup, and `ResponseExt` merge writes for picked candidates.
- `tools/gate_imui_workstream_source.py` now requires the response owner and rejects direct
  `model_value_changed_for(...)` and `input.merge_*` writes from the root text-picker owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass; 6 tests, 175
  skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text-Picker Open-Policy Owner-Split Evidence - 2026-05-27

Claim verified: IMUI input-text picker popup open policy moved behind a narrower private owner
without changing popup snapshot reads, active-descendant wiring, open-on-focus behavior,
empty/exact-match close behavior, candidate filtering, keyboard navigation, or picked-response
merge semantics.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` keeps completion/history entry points,
  candidate resolution, keyboard reconciliation, input-root rendering, popup rendering, and final
  `InputTextPickerResponse` assembly.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/open_policy.rs` owns popup open/panel-id
  snapshot reads, expanded-state calculation, empty/exact-match popup close policy, and
  open-on-focus anchoring.
- `tools/gate_imui_workstream_source.py` now requires the open-policy owner and rejects direct
  popup-store reads plus `open_popup_at` / `close_popup` calls from the root text-picker owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass; 6 tests, 175
  skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Media Summary Projection Owner-Split Evidence - 2026-05-27

Claim verified: debug-draw image/SVG/media command summary projection moved behind a narrower
private owner without changing command summary kind, image id stamping, point counts, mesh counts,
triangle counts, channel stamping, or effective clip-stack tracking.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection.rs` keeps
  `summary_with_clip_state(...)`, clip-stack mutation, non-media command projection, and delegates
  image/SVG/media count assembly to a private owner.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/media.rs` owns
  image triangle mesh, image rect/region/rounded, image quad, and SVG rect summary assembly.
- `tools/gate_imui_workstream_source.py` now requires the media owner and rejects media
  `DebugDrawCommandSummary::new(...)` construction from the root summary projection owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui debug_draw_controls::tests --no-fail-fast`:
  pass; 38 tests, 699 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke
  --no-fail-fast`: pass; 1 test, 0 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Begin-Menu Active-Trigger Open-Policy Owner-Split Evidence - 2026-05-27

Claim verified: begin-menu menubar active-trigger open-policy moved behind a narrower private
owner without changing trigger activation, top-level menu switching, popup close reconciliation,
open-request resolution, or disabled-popup cleanup behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy.rs` now keeps
  trigger-click toggling, open-request resolution, disabled-popup cleanup, and the private
  active-trigger owner re-export.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy/active_trigger.rs`
  owns active-trigger open-menu synchronization, post-trigger menubar reconciliation, and
  `MenubarActiveTrigger` group-active writes.
- `tools/gate_imui_workstream_source.py` now requires the active-trigger owner and rejects
  `GlobalElementId`, `MenubarActiveTrigger`, and `policy.group_active` writes from the root
  open-policy owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui menu_family_controls::tests
  --no-fail-fast`: pass; 1 test, 736 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`: pass; 39
  tests, 142 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Begin-Submenu Trigger/Open-Policy Owner-Split Evidence - 2026-05-27

Claim verified: begin-submenu trigger wiring and open-policy reconciliation moved into narrower
private owners without changing submenu trigger geometry hints, hover/shortcut behavior, sibling
switching, popup open/close anchoring, or `DisclosureResponse` open/toggled reporting.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu.rs` keeps public
  `begin_submenu_with_options(...)` orchestration, popup policy/model reads, popup menu mounting,
  disabled-popup cleanup, was-open recording, and `DisclosureResponse` assembly.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu/trigger.rs` owns submenu menu-item
  trigger assembly, submenu flag/expanded semantics, shortcut forwarding, and
  `sub_trigger::wire(...)` geometry hints.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu/open_policy.rs` owns clicked-trigger
  submenu-state reconciliation, stale-open cleanup, selected submenu reads, and popup open/close
  anchoring.
- `tools/gate_imui_workstream_source.py` now requires the submenu trigger/open-policy owners and
  rejects submenu menu-item trigger wiring, geometry hints, submenu-state mutations, and popup
  open/close anchoring from drifting back into `submenu.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::submenu_shortcuts
  interaction_menu_tabs::submenu_hover popup_hover::item_keyboard
  interaction_menu_tabs::menu_activation --no-fail-fast`: pass; 20 tests, 166 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table-Column Primitive Option Owner-Split Evidence - 2026-05-27

Claim verified: table-column primitive option types moved out of the `TableColumn` builder owner
without changing public re-export paths, width constructors, resize defaults, sort direction,
pinning helpers, column identity inference, visibility policy, or table composition behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/collections/table_column.rs` keeps `TableColumn`
  construction, identity inference, visibility/sort/resize/pin builder methods, accessor-first
  seams, and `set_visible_for_policy(...)`.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table_column/primitives.rs` owns
  `TableColumnWidth`, `TableColumnResizeOptions`, `TableSortDirection`, `TableColumnPin`, width
  constructors, and default resize limits.
- Existing callers still import the same names through `fret_ui_kit::imui` and
  `options::collections::table_column` re-exports.
- `tools/gate_imui_workstream_source.py` now requires the primitive owner and rejects primitive
  type bodies from drifting back into the `TableColumn` builder owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_helpers_compile table_column_pinning_helpers_compile
  table_resizable_column_api_compiles table_sortable_header_api_compiles --no-fail-fast`: pass; 4
  tests, 5 skipped.
- `cargo nextest run -p fret-imui
  table_helper_pins_left_and_right_columns_while_center_columns_scroll
  table_column_visibility_menu_item_updates_visibility_state --no-fail-fast`: pass; 2 tests, 179
  skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating-Window Resize Drag-Apply Owner-Split Evidence - 2026-05-27

Claim verified: floating-window resize delta application moved out of the resize state lifecycle
owner without changing active resize snapshots, left/top origin updates, min/max size clamping,
corner-handle resize behavior, collapsed reset policy, device-pixel snapping, or resize test ids.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` keeps `cx.state_for(...)`
  lifecycle, collapsed/non-drag reset policy, device-pixel snapping, resize output assembly, and
  handle test-id packaging.
- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/drag_apply.rs` owns drag delta
  calculation, min/max clamping, left/top origin reconciliation, all eight resize-handle branches,
  and `last_resize_position` advancement.
- `tools/gate_imui_workstream_source.py` now requires the drag-apply owner and rejects resize
  geometry branches from drifting back into the lifecycle state owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating_window_resizes_when_dragging_corner_handle
  floating_window_resizes_from_left_updates_origin_and_width --no-fail-fast`: pass; 2 tests, 179
  skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table-Column Visibility Snapshot Owner-Split Evidence - 2026-05-27

Claim verified: runtime table-column visibility state no longer owns the persistence-friendly
snapshot data shapes without changing public re-export paths, serde payload shape, empty-id
filtering, duplicate restore policy, or runtime column visibility application.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state.rs` keeps runtime visibility
  overrides, model mutation helpers, snapshot restore/apply orchestration, and
  `TableColumn::set_visible_for_policy(...)` use.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/state/snapshot.rs` owns
  `TableColumnVisibilitySnapshot`, `TableColumnVisibilityEntry`, serde derives, public data
  fields, and snapshot/entry accessors.
- Existing callers continue to import `TableColumnVisibilitySnapshot` and
  `TableColumnVisibilityEntry` through `table_column_visibility::state` re-exports and the
  root IMUI surface.
- `tools/gate_imui_workstream_source.py` now requires the snapshot owner and rejects serde/data
  shapes from drifting back into the runtime state owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_column_visibility::tests
  --no-fail-fast`: pass; 7 tests, 682 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_visibility_snapshot_api_compiles_and_roundtrips
  table_column_visibility_snapshot_entries_are_public_data_shape
  table_column_visibility_state_applies_runtime_visibility_by_column_id --no-fail-fast`: pass; 3
  tests, 6 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table-Column Visibility Menu Identity Owner-Split Evidence - 2026-05-27

Claim verified: table-column visibility menu identity helpers moved into a private owner without
changing stable column-id filtering, visible-label filtering, generated test-id suffixes, menu item
state updates, or table header context-menu behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu.rs` keeps header context-menu
  composition, menu item/group rendering, visibility model updates, and response population.
- `ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu/identity.rs` owns stable menu
  column id extraction, visible menu label parsing via label identity, and slug-like test-id suffix
  generation.
- `tools/gate_imui_workstream_source.py` now requires the identity owner and rejects identity,
  visible-label parsing, and slug-generation bodies from drifting back into the menu composition
  owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_column_visibility::tests
  --no-fail-fast`: pass on retry; first run timed out before reporting a test failure, retry
  reported 7 passed and 682 skipped.
- `cargo nextest run -p fret-imui
  composition::layout_collections::table_column_visibility_menu_item_updates_visibility_state
  composition::layout_collections::table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns
  composition::layout_collections::table_column_visibility_header_context_menu_opens_and_updates_state
  composition::layout_collections::table_column_visibility_header_context_menu_opens_from_plain_header
  --no-fail-fast`: pass on retry; first run timed out before reporting a test failure, retry
  reported 4 passed and 182 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Paint Media Owner-Split Evidence - 2026-05-27

Claim verified: debug draw image/SVG media painting moved out of the root paint dispatcher without
changing clip-stack balancing, shape command dispatch, image opacity/UV sanitization, rounded-image
clip behavior, SVG paint behavior, or debug-draw public authoring APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint.rs` keeps clip-stack balancing and
  command-class dispatch to media vs shape painters.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` owns image, image-region,
  image-quad, rounded-image, rounded-image-region, SVG image, and SVG mask-icon painting.
- `tools/gate_imui_workstream_source.py` now requires the media owner and rejects image/SVG paint
  helpers from drifting back into root `paint.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke
  --no-fail-fast`: pass; 1 test.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests
  --no-fail-fast`: pass; 38 tests, 651 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Path-Builder Shape-Method Owner-Split Evidence - 2026-05-27

Claim verified: debug draw path-builder rect/Bezier/arc authoring methods moved out of the root
path-builder owner without changing the public `ImUiDebugDrawPath` API, path sampling behavior,
invalid-input handling, stroke/fill command recording, or path state accessors.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder.rs` keeps the path type,
  point-list basics, stroke/fill command recording, and point-count/empty accessors.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods.rs` owns
  `rect`, `rect_with_rounding`, quadratic/cubic Bezier sampling, arc/fast-arc sampling, and
  elliptical-arc sampling.
- `tools/gate_imui_workstream_source.py` now requires the shape-method owner and rejects sampling
  helpers from drifting back into root `path_builder.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib
  debug_draw_controls::tests::path_builder --no-fail-fast`: pass; 12 tests, 677 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Header Label Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table header label/sort helpers moved out of the header assembly owner without
changing visible-label parsing, sortable/plain header wrapping, sort glyph text role, sortable a11y
labels, resize handle placement, or header response aggregation.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` now keeps sortable/plain header-cell
  assembly and resize-handle wrapping.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header/labels.rs` owns visible-header label
  parsing, column sortability reads, sort-indicator text, sortable a11y labels, header content
  boxes, and header label text.
- Existing callers continue to use the same `header::visible_header_label`,
  `header::column_is_sortable`, `header::table_header_label_text`, and
  `header::table_sort_indicator_text` surface through root re-exports.
- `tools/gate_imui_workstream_source.py` now requires the labels owner and rejects label/sort/text
  helpers from drifting back into the root header assembly owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui
  table_header_label_uses_shared_table_cell_text_role
  table_sort_indicator_uses_shared_chrome_glyph_text_role
  hidden_table_columns_do_not_render_header_body_or_response
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`: pass; 4
  tests, 733 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Image-Item Visual Owner-Split Evidence - 2026-05-27

Claim verified: IMUI image-item visual/props helpers moved out of the interaction owner without
changing image/image-button roles, focusability, pressable response population, item sizing,
opacity sanitization, UV filtering, or smoke-test surface.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs` originally kept pressable behavior,
  context-menu key handling, activation lifecycle, and `ResponseExt` population after the visual
  split. 2026-05-27 follow-up: `image_item_controls.rs` now keeps a11y props, size props, key
  activation policy for plain images, chrome mounting, and image visual assembly while
  `image_item_controls/behavior.rs` owns concrete pressable behavior.
- `ecosystem/fret-ui-kit/src/imui/image_item_controls/visual.rs` owns image item chrome selection,
  image props, size sanitization, opacity normalization, and UV validation.
- `ecosystem/fret-ui-kit/src/imui/image_item_controls/tests.rs` now targets the visual owner for
  props/sanitizer coverage.
- `tools/gate_imui_workstream_source.py` now requires the visual owner and rejects props/sanitizer
  helpers from drifting back into the root interaction owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib image_item_controls::tests
  --no-fail-fast`: pass; 2 tests, 687 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_image_item_smoke
  --no-fail-fast`: pass; 2 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Begin Menu Open-Policy Owner-Split Evidence - 2026-05-27

Claim verified: IMUI begin-menu open-policy mutations moved out of the state index without changing
menubar open-menu synchronization, active-trigger writes, trigger-click toggling,
open-request resolution, disabled-popup cleanup, or menu/popup interaction behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state.rs` is now a thin
  capture/open-policy module index.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy.rs` owns
  `sync_open_menu_for_active_trigger`, `reconcile_menubar_after_trigger`,
  `toggle_menu_on_trigger_click`, `resolve_open_requested`,
  `activate_menubar_trigger_if_requested`, and `close_disabled_popup_if_opened`.
- `tools/gate_imui_workstream_source.py` now requires the open-policy owner and rejects open-policy
  mutations from drifting back into the root state index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui menu_family_controls::tests
  --no-fail-fast`: pass; 1 test, 736 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`: pass; 39
  tests, 142 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Debug-Draw Summary Owner-Split Evidence - 2026-05-27

Claim verified: debug draw command summary types and aggregate list summary ownership moved out of
the shared summary file without changing public summary type names, accessor-first storage,
command-kind classification, clip-stack metrics, or external smoke usage.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries.rs` is now a thin module/re-export
  index.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/command.rs` owns
  `DebugDrawCommandKind` and `DebugDrawCommandSummary`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/list.rs` owns
  `DebugDrawListSummary` counters, final clip-depth update, and command classification.
- `tools/gate_imui_workstream_source.py` now requires the summary index/sub-owner shape and moves
  the opaque-summary checks to the concrete owner files.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib
  debug_draw_controls::tests::draw_list::summaries --no-fail-fast`: pass; 4 tests, 685 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke
  --no-fail-fast`: pass; 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Facade Container Wrapper Owner-Split Evidence - 2026-05-27

Claim verified: root `ImUiFacade` container wrapper methods moved into private flow, layout,
collection, and menu/tab wrapper owners without changing method names, focus forwarding, list-box
label normalization, table/virtual-list responses, or child-region/menu/tab composition behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs` is now a thin module index.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/flow.rs` owns `items`,
  `same_line`, `dummy`, `spacing`, and `indent` root wrappers.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/layout.rs` owns horizontal,
  vertical, grid, scroll, and child-region root wrappers.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/collections.rs` owns list-box,
  table, and virtual-list root wrappers.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/menu_tabs.rs` owns menu-bar and
  tab-bar root wrappers.
- `tools/gate_imui_workstream_source.py` now requires the wrapper index/sub-owner shape and rejects
  container behavior from drifting back into the root wrapper index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui composition::layout_collections --no-fail-fast`: pass; 28 tests,
  153 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Debug-Draw Options Sub-Owner Split Evidence - 2026-05-27

Claim verified: public debug draw root/interaction options, stroke/path-style policy,
rounded-corner flags, media option bags, and mesh vertex helpers moved behind private option
sub-owners without changing public re-export names, draw-list command behavior, path-style
conversion, image/svg options, or mesh vertex conversion.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options.rs` is now a thin module/re-export
  index.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options/root.rs` owns `DebugDrawOptions` and
  `DebugDrawInteractionOptions`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options/stroke.rs` owns
  `DebugDrawStrokeStyle`, including dash/miter validation and crate-local `path_style()`.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options/round_corners.rs` owns
  `DebugDrawRoundCorners` constants and bitwise composition.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options/media.rs` owns image, image-quad,
  image-mesh, and SVG option bags.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options/vertex.rs` owns `DebugDrawVertex`
  and crate-local scene vertex conversion.
- `tools/gate_imui_workstream_source.py` now requires the option index/sub-owner shape and rejects
  these public option/style/vertex type bodies from drifting back into the index.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests
  --no-fail-fast`: pass; 38 tests, 651 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke
  --no-fail-fast`: pass; 1 test.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Control Chrome Layout Owner-Split Evidence - 2026-05-27

Claim verified: IMUI shared control chrome row/stack layout helper props moved into a private
layout owner without changing row direction, fill-width behavior, gap tokens, justification,
alignment, or existing `control_chrome::*_props` call paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/control_chrome/layout.rs` now owns `fill_row_props`,
  `centered_row_props`, and `fill_stack_props`.
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` now keeps style constants,
  `ImUiControlPalette`, button/field chrome, text helper re-exports, and test module wiring.
- `ecosystem/fret-ui-kit/src/imui/control_chrome/tests.rs` now covers row/stack layout helper
  direction, width, gap, justification, and alignment defaults.
- `tools/gate_imui_workstream_source.py` now requires the layout owner and rejects direct row/stack
  layout helper bodies from drifting back into `control_chrome.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib control_chrome::tests
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Control Chrome Text Owner-Split Evidence - 2026-05-27

Claim verified: IMUI shared control text helpers, caption color routing, and pill badge chrome
moved into a private control-chrome text owner without changing compact button/control label text
roles, caption muted-foreground routing, pill badge chrome, or existing `control_chrome::*` call
paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/control_chrome/text.rs` now owns `control_text`,
  `fill_text`, `caption_text`, and `pill`.
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` now keeps style constants,
  `ImUiControlPalette`, button/field chrome, row/stack layout props, and test module wiring.
- `tools/gate_imui_workstream_source.py` now requires the text owner and rejects direct text/pill
  helper bodies from drifting back into `control_chrome.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib control_chrome::tests
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Area Composition Owner-Split Evidence - 2026-05-27

Claim verified: floating-area layer registration, drag snapshot application, state/test-id updates,
IMUI facade content mounting, absolute area layout, no-input/pass-through gates, and
`FloatingAreaResponse` assembly moved into a private area owner without changing floating-area
position, dragging, test-id, no-inputs, pointer pass-through, layer ordering, or response
semantics.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_surface/area.rs` now owns floating-area composition,
  state reconciliation, interaction gates, and response assembly.
- `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` now keeps drag-surface pointer-region
  behavior, layer/kind/state re-exports, and module wiring.
- `tools/gate_imui_workstream_source.py` now requires the area owner and rejects direct area
  layout/state/gate construction from drifting back into `floating_surface.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text Picker Input Root Owner-Split Evidence - 2026-05-27

Claim verified: IMUI input-text picker input option/test-id preparation, ComboBox semantics
normalization, assistive semantics, root fill container construction, text input mounting, and
input-focused keyboard handler installation moved into a private input-root owner without changing
completion/history picker behavior, active-descendant wiring, popup lifecycle policy, picked
response merging, or public facade APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/input.rs` now owns picker input preparation,
  assistive semantics, root container construction, text input mounting, and input-focused keyboard
  handler installation.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` now keeps candidate visibility,
  popup-open state reads, keyboard-state snapshot reconciliation, popup lifecycle policy, popup
  rendering delegation, and final `InputTextPickerResponse` merge.
- `tools/gate_imui_workstream_source.py` now requires the input-root owner and rejects direct input
  semantics/container/keyboard-handler construction from drifting back into the root picker file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Disclosure Header Row Visual Owner-Split Evidence - 2026-05-27

Claim verified: disclosure header row container/flex assembly, indicator glyph mounting, label
text mounting, row padding, border, and radius props moved out of the broader visual owner without
changing collapsing-header/tree-node a11y, palette policy, indicator glyphs, label text roles,
indentation, row chrome, or trigger behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/header.rs` now owns header row
  construction, including row container props, flex row layout, indicator glyph, label text, and
  spacer assembly.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual.rs` now keeps disclosure a11y,
  content padding, and palette resolution.
- `tools/gate_imui_workstream_source.py` now requires the header-row owner and rejects direct
  container/flex/text row construction from drifting back into `disclosure_controls/visual.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check --verbose`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib disclosure_controls::tests
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_disclosure_smoke
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  interaction_shortcuts::disclosure_tree::tree_node_children_stack_vertically_inside_open_parents
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Popup Modal Layout Owner-Split Evidence - 2026-05-27

Claim verified: IMUI popup modal palette, centered panel geometry, layer/backdrop props, dialog
semantics layout, and panel chrome construction moved into a private layout owner without changing
modal open/keepalive policy, Escape or outside-press dismissal, barrier behavior, focus handoff,
centered panel placement, test ids, or `OverlayRequest::modal` assembly.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layout.rs` now owns modal palette resolution,
  centered panel layout, absolute layer and backdrop props, dialog semantics layout, and panel
  chrome props.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/modal.rs` now keeps popup store reads, keepalive
  generation, dismiss policy, focus tracking, IMUI facade content mounting, and overlay request
  assembly.
- `tools/gate_imui_workstream_source.py` now requires the layout owner and rejects direct dialog
  semantics/chrome/layout construction from drifting back into `popup_overlay/modal.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover::lifecycle_modal --no-fail-fast`: pass.
- `cargo nextest run -p fret-ui-kit modal_barrier_is_hidden_from_accessibility_tree_but_still_invokable
  select_pointer_up_guard_barrier_is_hidden_from_accessibility_tree --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Pressable Item Response Owner-Split Evidence - 2026-05-27

Claim verified: shared IMUI pressable item response population moved out of the hook-installation
owner without changing button, checkbox/radio, selectable, combo, image item, or debug-draw
pressable behavior; context-menu signals; pointer-click modifiers; drag response merging; hover
query hooks; or `ResponseExt` population semantics.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/item_behavior/response.rs` now owns transient signal reads,
  context anchor/modifier reads, drag response merging, hover query hook installation, and final
  pressable response population.
- `ecosystem/fret-ui-kit/src/imui/item_behavior.rs` now keeps pressable hook installation,
  active-item/long-press/lifecycle/context-menu models, pointer-up transient emission, and the
  existing `item_behavior::populate_pressable_item_response(...)` re-exported call surface.
- `tools/gate_imui_workstream_source.py` now requires the response owner and rejects `ResponseExt`
  population, drag response merging, and hover response wiring from drifting back into
  `item_behavior.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib button_controls::tests
  boolean_controls::tests selectable_controls::tests debug_draw_controls::tests::element
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui interaction_press interaction_drag
  interaction_shortcuts::command_metadata::button_command_dispatches_with_metadata_and_payload
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Row-Group Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table pinned/scroll row-group mechanics moved out of the row/cell body owner
without changing row semantics, cell wrapping, pinned left/right grouping, horizontal center-scroll
wrapping, column gaps, or public table response behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/row_groups.rs` now owns pinned-cell splitting,
  left/center/right row-group assembly, horizontal center-scroll wrapping, and the shared
  horizontal flex primitive.
- `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs` now keeps `PreparedTableCell`,
  `TablePalette`, row semantics/background selection, and cell wrapping.
- `tools/gate_imui_workstream_source.py` now requires the row-group owner and rejects pinned split,
  horizontal scroll, and row-flex mechanics from drifting back into `body.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls::tests --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::layout_collections::table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells
  composition::layout_collections::table_helper_pins_left_and_right_columns_while_center_columns_scroll
  composition::layout_collections::table_helper_skips_hidden_columns_in_header_and_body
  label_identity::table_headers::table_resizable_header_reports_drag_response
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Table Header Row Owner-Split Evidence - 2026-05-27

Claim verified: IMUI table header row assembly moved out of the root render owner without changing
header visibility, sortable/plain header behavior, resize response metadata, pinned/horizontal
scroll wrapping, header test ids, or aggregate `TableResponse` headers.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/header_row.rs` now owns the keyed header row,
  visible-header-cell assembly, sortable/plain wrapper selection, resize response initialization,
  `TableHeaderResponse` collection, and header row wrapping.
- `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` now keeps table palette resolution,
  visible-column filtering, horizontal-scroll/header-presence decisions, body row assembly, root
  chrome, semantics, and final `TableResponse` assembly.
- `tools/gate_imui_workstream_source.py` now requires the header-row owner and rejects header
  label/sort/resize response assembly from drifting back into `render.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls::tests --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::layout_collections::table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells
  label_identity::table_headers::label_identity_table_headers_hide_suffixes_from_visible_labels
  label_identity::table_headers::table_sortable_header_reports_app_owned_trigger_without_sorting_rows
  label_identity::table_headers::table_resizable_header_reports_drag_response
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Begin Menu State Capture Owner-Split Evidence - 2026-05-27

Claim verified: IMUI begin-menu state capture/read helpers moved out of the menubar mutation owner
without changing row/popup/was-open model identity, render-state recording, open-menu reads, or
menubar open-policy behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/capture.rs` now owns
  `BeginMenuState`, `MenuRenderState`, row/popup/was-open model capture, row/open-menu reads, and
  render-state recording.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state.rs` now focuses on menubar
  open-menu mutation, active-trigger synchronization, open-request resolution, and disabled-popup
  cleanup.
- `tools/gate_imui_workstream_source.py` now requires the capture owner and rejects model capture
  and render-state recording from drifting back into `menu_state.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_family_controls::tests --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation
  interaction_menu_tabs::submenu_shortcuts
  interaction_shortcuts::command_metadata::menu_item_command_uses_command_metadata_shortcut_and_gating
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Begin Menu State Owner-Split Evidence - 2026-05-27

Claim verified: IMUI begin-menu state/open-policy behavior moved out of the flow owner without
changing menubar trigger activation, popup open/close, active-trigger synchronization,
disabled-popup cleanup, or `DisclosureResponse` open/toggled semantics.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state.rs` now owns begin-menu state
  capture, row/popup/was-open models, menubar open-menu synchronization, active trigger state
  writes, open-request resolution, disabled-popup cleanup, and render-state recording.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu.rs` now keeps begin-menu flow
  orchestration, trigger mounting, popup mounting, and final `DisclosureResponse` assembly.
- `tools/gate_imui_workstream_source.py` now requires the state owner and rejects local model
  capture, menubar `open_menu` / `group_active` mutation, and row-open mutation from drifting back
  into `menu.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_family_controls::tests --no-fail-fast`:
  pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation
  interaction_menu_tabs::submenu_shortcuts
  interaction_shortcuts::command_metadata::menu_item_command_uses_command_metadata_shortcut_and_gating
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Window Resize Handle Owner-Split Evidence - 2026-05-26

Claim verified: IMUI floating-window resize handle layout and pointer behavior moved out of the
handle stack owner without changing handle placement, cursors, pointer capture/release, runtime drag
begin/update/cancel, activation handoff, or resize behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/layout.rs` now owns handle
  geometry and resize cursor selection for all eight resize handles.
- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/pointer.rs` now owns
  pointer-region wiring, pointer capture/release, runtime drag begin/update/cancel, cursor updates,
  and resize-handle activation handoff.
- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles.rs` now only stacks the
  body/blocker with the eight resize handles.
- `tools/gate_imui_workstream_source.py` now rejects layout and pointer behavior from drifting back
  into `handles.rs`, while also keeping layout and pointer owners separate from resize state.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating::window_options::floating_window_resizes_when_dragging_corner_handle
  floating::window_options::floating_window_resizes_from_left_updates_origin_and_width
  floating::window_options::floating_window_resizable_false_hides_resize_handles
  floating::window_options::floating_window_title_bar_double_click_toggles_collapsed
  floating::input_modes::floating_window_activate_on_click_can_be_disabled_for_resize_handles
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Window Resize Snapshot Owner-Split Evidence - 2026-05-26

Claim verified: IMUI floating-window active resize snapshot discovery moved out of the resize state
owner without changing resize handle enumeration, runtime drag matching, downstream resize
calculation, or public facade behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/snapshot.rs` now owns active resize drag
  discovery, resize-handle enumeration, runtime drag kind matching, and snapshot capture.
- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` now focuses on applying resize
  deltas, min/max clamping, origin updates for left/top handles, collapse/non-drag reset,
  device-pixel snapping, and resize output assembly.
- `tools/gate_imui_workstream_source.py` now requires the snapshot owner and rejects runtime drag
  lookup from drifting back into `floating_window_resize/state.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating::window_options::floating_window_resizes_when_dragging_corner_handle
  floating::window_options::floating_window_resizes_from_left_updates_origin_and_width
  floating::window_options::floating_window_resizable_false_hides_resize_handles
  floating::window_options::floating_window_title_bar_double_click_toggles_collapsed
  floating::input_modes::floating_window_activate_on_click_can_be_disabled_for_resize_handles
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Floating Window Resize State Owner-Split Evidence - 2026-05-26

Claim verified: IMUI floating-window resize snapshot/state calculation moved out of the root
resize module without changing resize handles, left/right/corner resize behavior, collapse reset,
device-pixel snapping, or resize test-id output.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` now owns active resize snapshot
  lookup, drag delta application, min/max size clamping, origin updates for left/top handles,
  collapse reset, device-pixel snapping, and resize state/test-id output.
- `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` is now a thin `handles`/`state` index
  plus the shared `FloatingWindowResizeHandleTestIds` record.
- `ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles.rs` remains the owner for resize
  handle layout, pointer-region drag lifecycle wiring, cursor updates, and activation handoff.
- `tools/gate_imui_workstream_source.py` now requires the state owner and rejects snapshot lookup,
  resize drag calculation, and pixel snapping from drifting back into `floating_window_resize.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating::window_options::floating_window_resizes_when_dragging_corner_handle
  floating::window_options::floating_window_resizes_from_left_updates_origin_and_width
  floating::window_options::floating_window_resizable_false_hides_resize_handles
  floating::window_options::floating_window_title_bar_double_click_toggles_collapsed
  floating::input_modes::floating_window_activate_on_click_can_be_disabled_for_resize_handles
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Textarea Owner-Split Evidence - 2026-05-26

Claim verified: IMUI textarea element assembly moved out of the root text-controls owner without
changing textarea facade routing, response semantics, select-all-on-focus behavior, submit/cancel
command policy, compact chrome, or text style selection.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_controls/textarea.rs` now owns textarea props assembly,
  response/lifecycle population, select-all command emission, submit/cancel policy command
  installation, and text-area chrome/text-style selection.
- `ecosystem/fret-ui-kit/src/imui/text_controls.rs` keeps input-text assembly, shared text-model
  changed detection, assistive semantics, input policy command installation, and the
  `text_controls::textarea_model_with_options` re-export used by the facade.
- `tools/gate_imui_workstream_source.py` now requires the textarea owner and rejects
  `TextAreaProps`, text-area chrome/style selection, and textarea policy installation from drifting
  back into `text_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui text_controls::tests --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui models_text_area --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu Item Keyboard Owner-Split Evidence - 2026-05-26

Claim verified: IMUI menu-item keyboard/navigation behavior moved out of the menu item
interaction owner without changing popup menu roving focus, item-local shortcuts, menubar
horizontal-arrow switching, command dispatch metadata, or response population.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard.rs` now owns item-local activate shortcut
  handling, popup menu roving focus, menubar close-auto-focus suppression, and horizontal-arrow
  menu switching.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` keeps enabled/action gating,
  pressable props, activation dispatch, command dispatch metadata helper, and active-trigger
  response population.
- `tools/gate_imui_workstream_source.py` now requires the keyboard owner and rejects popup roving
  focus, arrow-key handling, and menubar horizontal switching from drifting back into
  `menu_controls/interaction.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation
  interaction_menu_tabs::submenu_shortcuts
  interaction_shortcuts::command_metadata::menu_item_command_uses_command_metadata_shortcut_and_gating
  popup_hover::item_keyboard --no-fail-fast`: pass; 15 tests, 166 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu Item Behavior Owner-Split Evidence - 2026-05-27

Claim verified: IMUI menu-item active-trigger, activation, command dispatch, and response behavior
moved into a nested private owner without changing menu item facade entry points, popup close
behavior, popup menu keyboard navigation, menubar horizontal-arrow switching, command dispatch
metadata, or row visual structure.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` keeps `MenuItemInteractionParts`,
  `MenuItemInteraction`, enabled/action gating, pressable prop construction, and thin forwarding
  call sites used by the element and keyboard owners.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/interaction/behavior.rs` owns active-trigger
  installation, activate-handler wiring, clicked transient reads, command dispatch source metadata,
  keyboard owner wiring, and active-trigger `ResponseExt` population.
- `tools/gate_imui_workstream_source.py` now requires the nested behavior owner and rejects inline
  pressable activation, active-trigger installation, clicked signaling, command dispatch metadata,
  and response population from drifting back into `menu_controls/interaction.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls::tests --no-fail-fast`:
  pass on retry; first run timed out before reporting a test failure, retry reported 4 passed and
  685 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation
  interaction_menu_tabs::submenu_shortcuts
  interaction_shortcuts::command_metadata::menu_item_command_uses_command_metadata_shortcut_and_gating
  popup_hover::item_keyboard --no-fail-fast`: pass; 15 tests, 171 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text Picker Popup Owner-Split Evidence - 2026-05-26

Claim verified: IMUI input-text picker popup item rendering and pick commit moved into a focused
private owner without changing completion/history picker behavior, keyboard navigation, active
descendant semantics, or response merge semantics.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/popup.rs` now owns
  `InputTextPickerPopupInput`, `InputTextPickerPopupResult`, popup mounting, popup-scoped keyboard
  handler installation, selectable candidate rows, active-element synchronization, click commit,
  popup close, and picked-result reporting.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` keeps input composition, assistive
  semantics, open/close policy, candidate/keyboard snapshots, and final `ResponseExt` merge after a
  pick changes the model.
- `tools/gate_imui_workstream_source.py` now requires the popup owner and rejects direct
  `selectable_with_options(...)`, selectable option construction, and direct model/popup-open
  updates from drifting back into root `text_picker_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`: pass; 6 tests, 175 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Menu Item Interaction Owner-Split Evidence - 2026-05-26

Claim verified: IMUI menu-item interaction behavior moved into a focused private owner without
changing menu item facade entry points, popup menu keyboard navigation, menubar horizontal-arrow
switching, command dispatch metadata, or row visual structure.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` now owns
  `MenuItemInteractionParts`, `MenuItemInteraction`, enabled/action gating, pressable props,
  activation handlers, item-local shortcuts, popup roving-focus keyboard movement, menubar
  horizontal-arrow switching, command dispatch source metadata, and active-trigger response
  population.
- `ecosystem/fret-ui-kit/src/imui/menu_controls/element.rs` keeps the menu row panel, indicator,
  label, shortcut/submenu glyph visual assembly, and the custom `pressable_hook` insertion point
  used by submenu helpers.
- `tools/gate_imui_workstream_source.py` now requires the interaction owner and rejects
  pressable/a11y props, active-trigger installation, keyboard handlers, menubar wiring, command
  dispatch, and response population from drifting back into `menu_controls/element.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_controls::tests --no-fail-fast`:
  pass; 4 tests, 684 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation
  interaction_menu_tabs::submenu_shortcuts
  interaction_shortcuts::command_metadata::menu_item_command_uses_command_metadata_shortcut_and_gating
  popup_hover::item_keyboard --no-fail-fast`: pass; 15 tests, 166 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Button Behavior Owner-Split Evidence - 2026-05-26

Claim verified: IMUI button pressable/action behavior moved into a focused private owner without
changing button facade entry points, label identity, command gating, shortcut behavior, or response
accessors.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` now owns `ButtonAction`,
  `button_pressable(...)`, command gating, pressable props, shortcut/context-menu handling, command
  dispatch source metadata, payload forwarding, and button `ResponseExt` population.
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs` keeps public entry routing for default,
  small, arrow, invisible, action, and payload-action buttons plus label identity scoping.
- `ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` remains the layout, a11y, and chrome
  owner and is consumed by the behavior owner.
- `tools/gate_imui_workstream_source.py` now requires the behavior owner and rejects pressable
  props, control chrome pressable assembly, key handlers, action dispatch, and direct response
  population from drifting back into root `button_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-imui interaction_shortcuts::button_shortcuts
  interaction_shortcuts::command_metadata::button_command_uses_command_metadata_and_gating
  interaction_press --no-fail-fast`: pass; 12 tests, 169 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Combo Trigger Owner-Split Evidence - 2026-05-26

Claim verified: IMUI combo trigger behavior and trigger chrome moved into a focused private owner
without changing the public combo/combo-model facade, popup model wiring, or response accessors.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/combo_controls/trigger.rs` now owns `ComboTriggerOptions`,
  `combo_trigger(...)`, ComboBox semantics, shortcut activation, context-menu request handling,
  pressable `ResponseExt` population, and open/menu trigger chrome.
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs` keeps label identity normalization, popup
  model reads, trigger click-to-open/close, popup mounting, disabled close policy, and aggregate
  `ComboResponse` open/toggled lifecycle flags.
- `ecosystem/fret-ui-kit/src/imui/combo_controls/tests.rs` follows the a11y-label helper to the
  new trigger owner.
- `tools/gate_imui_workstream_source.py` now requires the new trigger owner and rejects
  `PressableProps`, `PressableA11y`, `control_chrome_pressable_with_id_props`,
  `pressable_on_activate`, `key_on_key_down_for`, and direct pressable response population from
  drifting back into root `combo_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib combo_controls::tests
  --no-fail-fast`: pass; 2 tests, 686 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_combo_smoke --no-fail-fast`:
  pass; 2 tests.
- `cargo nextest run -p fret-imui models_combo --no-fail-fast`: pass; 11 tests, 170 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Disclosure Trigger Owner-Split Evidence - 2026-05-26

Claim verified: IMUI disclosure trigger shell moved into a focused private owner without changing
the public collapsing-header/tree-node facade, open model semantics, content mounting, or response
accessors. Trigger behavior and response population were split deeper into
`disclosure_controls/trigger/behavior.rs` on 2026-05-27.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger.rs` now owns header pressable shell
  construction, a11y wiring, visual mounting, collapsible trigger controls, and test-id
  application.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs` keeps label identity normalization,
  spec/open-model wiring, content mounting, and aggregate `DisclosureResponse` open/toggled state.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/spec.rs` remains the option-to-spec owner,
  while `visual.rs` remains the a11y/visual row owner.
- `tools/gate_imui_workstream_source.py` now requires the split trigger owner and rejects
  pressable/keyboard/context-menu/response-population bodies from drifting back into root
  `disclosure_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib disclosure_controls::tests
  --no-fail-fast`: pass; 6 tests, 682 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs::submenu_shortcuts --no-fail-fast`:
  pass; 3 tests, 178 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs::menu_activation --no-fail-fast`: pass; 6
  tests, 175 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- A broader combined `cargo nextest run -p fret-imui interaction_menu_tabs::submenu_shortcuts
  interaction_menu_tabs::menu_activation --no-fail-fast` attempt timed out after 244 seconds while
  its cargo/nextest process was still compiling. The process finished naturally afterward, and both
  focused filters passed when rerun serially.

## Popup Menu Policy/Panel Owner-Split Evidence - 2026-05-26

Claim verified: IMUI popup-menu policy state and popup panel composition moved into focused private
owners without changing begin-popup facade entry points, menu/submenu behavior, focus handling, or
overlay request dispatch.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/policy.rs` now owns `ImUiMenuNavState`,
  `ImUiPopupMenuPolicyState`, and root submenu-policy synchronization.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel.rs` originally owned popper placement,
  menu semantics, nav-state installation, panel chrome, IMUI child mounting, and initial focus
  targets after the root menu split. 2026-05-27 follow-up: `panel.rs` now keeps lifecycle and
  assembly while `panel/layout.rs` and `panel/content.rs` own the concrete layout/chrome/content
  bodies.
- `ecosystem/fret-ui-kit/src/imui/popup_overlay/menu.rs` keeps begin-popup orchestration, menubar
  policy lookup, focus-outside dismissal preservation, close auto-focus suppression, and overlay
  request dispatch.
- `ecosystem/fret-ui-kit/tests/imui_perf_guard_smoke.rs` now follows the popper viewport source
  anchor to `popup_overlay/menu/panel/layout.rs`, where placement policy lives after the 2026-05-27
  panel split.
- `tools/gate_imui_workstream_source.py` now requires the split policy/panel owners and rejects
  policy-state or panel-composition bodies from drifting back into root `popup_overlay/menu.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui popup_hover interaction_menu_tabs --no-fail-fast`: pass; 39
  tests, 142 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_combo_smoke --no-fail-fast`:
  pass; 2 tests.
- `cargo nextest run -p fret-ui-kit --features imui
  popup_menu_uses_environment_viewport_bounds_for_popper_outer_bounds --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- The first `popup_menu_uses_environment_viewport_bounds_for_popper_outer_bounds` run failed after
  the code split because the source guard still read `popup_overlay/menu.rs`. The source anchor was
  updated to `popup_overlay/menu/panel.rs`, and the focused test then passed. The 2026-05-27 deeper
  panel split moved that source anchor again to `popup_overlay/menu/panel/layout.rs`.

## Tab Family Items Owner-Split Evidence - 2026-05-26

Claim verified: IMUI tab-family item collection, tab-list semantics, trigger response aggregation,
focus fallback, and selected-panel assembly moved into a focused private owner without changing the
public tab-bar builder or response surface. Selected-model normalization was split deeper into
`tab_family_controls/items/selection.rs` on 2026-05-27.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/items.rs` now owns `BuiltTabItem`,
  tab-list semantics, trigger response aggregation, focus fallback, and selected panel assembly.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs` keeps the public `ImUiTabBar` builder,
  `tab_item(...)` / `begin_tab_item(...)` collection API, and `tab_bar_element(...)` entrypoint.
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger.rs` remains the per-trigger owner for
  activation, keyboard shortcut handling, a11y, and response population.
- `tools/gate_imui_workstream_source.py` now requires the split tab item owner and rejects tab-list
  semantics, panel assembly, and selection normalization from drifting back into root
  `tab_family_controls.rs` or trigger-local code.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_menu_tabs::tabs --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::layout_collections::tab_bar_helper_arranges_tabs_horizontally_and_stamps_tab_semantics
  --no-fail-fast`: pass.
- `cargo nextest run -p fret-imui
  composition::control_geometry::menu_and_tab_trigger_state_changes_keep_outer_bounds_stable
  --no-fail-fast`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- An earlier parallel tab nextest attempt timed out after 244 seconds while unrelated cargo jobs in
  other worktrees were holding package/build locks. The focused tab gates passed when rerun
  serially after source/doc updates.

## Boolean Controls Behavior Owner-Split Evidence - 2026-05-26

Claim verified: IMUI checkbox/radio boolean-control behavior moved into focused private owners
without changing the public checkbox/radio/switch facade surface, response semantics, shortcuts, or
label-identity behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox.rs` originally owned checkbox label
  identity, model toggling, focused shortcut handling, context-menu request handling, and pressable
  response population after the root boolean-controls split. 2026-05-27 follow-up:
  `checkbox.rs` now keeps label/a11y/chrome/visual assembly while `checkbox/behavior.rs` owns the
  concrete pressable behavior.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls/radio.rs` originally owned radio label identity,
  focused shortcut handling, context-menu request handling, click reporting, and pressable response
  population after the root boolean-controls split. 2026-05-27 radio follow-up:
  `radio.rs` now keeps label/a11y/chrome/visual assembly while `radio/behavior.rs` owns the
  concrete pressable behavior.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs` is now a module/re-export index beside the
  existing `switch.rs` and `visual.rs` owners.
- `tools/gate_imui_workstream_source.py` now requires the split checkbox/radio owners and rejects
  checkbox/radio behavior from drifting back into root `boolean_controls.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass; 6 tests, 175 skipped.
- `cargo nextest run -p fret-imui composition::control_geometry --no-fail-fast`: pass; 4 tests,
  177 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 test.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- The focused nextest runs waited on package-cache/build-directory locks, then completed
  successfully in the same runs.

## Menu Options Owner-Split Evidence - 2026-05-26

Claim verified: IMUI menu/popup/tab/tooltip option records moved into focused private owners
without changing public option names, fields, defaults, or top-level re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/menus/popup.rs` is the current popup option re-export
  hub.
- `ecosystem/fret-ui-kit/src/imui/options/menus/popup/menu.rs` owns `PopupMenuOptions`, including
  popper placement, default size, modal, and auto-focus defaults.
- `ecosystem/fret-ui-kit/src/imui/options/menus/popup/modal.rs` owns `PopupModalOptions`,
  including modal size and outside-press defaults.
- `ecosystem/fret-ui-kit/src/imui/options/menus/menu.rs` is the menu-option re-export hub. The
  2026-05-28 menu option owner split records the current child owners for `MenuBarOptions`,
  `BeginMenuOptions`, `BeginSubmenuOptions`, and `MenuItemOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/menus/tab.rs` now owns `TabBarOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/menus/tooltip.rs` now owns `TooltipOptions`, including
  popper placement, estimated size, window margin, delay overrides, hoverable-content policy, and
  test id.
- `ecosystem/fret-ui-kit/src/imui/options/menus.rs` is now a module/re-export index, preserving
  existing public paths through `imui::options` and the root `imui` facade exports.
- `tools/gate_imui_workstream_source.py` now requires the split menu-option owners and forbids
  popup/menu/tab/tooltip option bodies from drifting back into root `menus.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_tooltip_smoke --test
  imui_combo_smoke --no-fail-fast`: pass; 3 tests.
- `cargo nextest run -p fret-imui interaction_menu_tabs popup_hover --no-fail-fast`: pass; 39
  tests, 142 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- The focused `fret-ui-kit` and `fret-imui` runs waited on package-cache/build-directory locks,
  then completed successfully in the same runs.

## Tooltip Overlay Owner-Split Evidence - 2026-05-26

Claim verified: IMUI tooltip pointer-open gating and panel composition moved into focused private
owners without changing the public tooltip facade, hover/dismissal behavior, rich-content builder,
or text-tooltip helper path.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/trigger.rs` now owns non-touch pointer-move open
  gating, last-pointer model updates, pointer-transit buffer checks, and the redraw trigger for the
  first pointer-move open.
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay/panel.rs` now owns concrete tooltip panel
  placement, popover/border chrome, tooltip semantics/test id wiring, and rich-content column
  facade assembly.
- `ecosystem/fret-ui-kit/src/imui/tooltip_overlay.rs` keeps trigger id validation, tooltip event
  model setup, interaction floating-bounds calculation, open/update scheduling, open-model sync,
  dismiss request handling, hoverable-content tracking, and `request_tooltip(...)` orchestration.
- `tools/gate_imui_workstream_source.py` now requires the split tooltip owners and rejects
  pointer-transit internals, pointer-open state writes, panel chrome/composition, and tooltip
  semantics from drifting back into the root tooltip orchestration file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_tooltip_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-ui-kit --features imui --lib tooltip_overlay::tests
  --no-fail-fast`: pass; 3 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- An earlier shared-target `tooltip_overlay::tests` run timed out after 244 seconds while cargo was
  waiting behind package/build locks. The same focused lib test passed when rerun serially.

## Child-Region Resize Owner-Split Evidence - 2026-05-26

Claim verified: IMUI child-region resize handle and drag-response ownership moved into a focused
private owner without changing the public child-region facade, response surface, scroll
forwarding, framed chrome, or root/content test-id behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/child_region/resize.rs` now owns resize axis layout, handle
  constants, pointer-region drag wiring, enabled/min/max response writes, and drag start/stop edge
  reconciliation for both X and Y resize handles.
- `ecosystem/fret-ui-kit/src/imui/child_region.rs` keeps child-region content building, scroll-area
  configuration, framed chrome, viewport/content/root test-id routing, and resize stack assembly.
- `tools/gate_imui_workstream_source.py` now requires the split resize owner and rejects resize
  drag-state, handle constants, axis layout, and pointer-region drag wiring from drifting back into
  the root child-region composition file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke
  --no-fail-fast`: pass; 3 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- The focused `fret-ui-kit` build/test commands waited briefly on package-cache/build-directory
  locks, then completed successfully in the same runs.

## Selectable Keyboard Owner-Split Evidence - 2026-05-26

Claim verified: IMUI selectable keyboard policy moved into a focused private owner without changing
selectable activation, popup close, context-menu request, popup arrow navigation, or public
selectable APIs.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/selectable_controls/keyboard.rs` now owns selectable activation
  shortcuts, popup close-on-activate updates, ContextMenu/Shift+F10 context-menu requests, and
  inherited popup-menu arrow/Home/End focus navigation.
- `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs` originally kept label identity
  normalization, pressable/a11y assembly, pointer activation, response population, and visual row
  composition after the keyboard split. 2026-05-27 follow-up: `selectable_controls.rs` now keeps
  label identity, a11y wiring, selected/highlighted state reads, and row visual assembly while
  `selectable_controls/behavior.rs` owns concrete pressable behavior.
- `tools/gate_imui_workstream_source.py` now requires `selectable_controls/keyboard.rs`, forbids
  keyboard/nav internals from drifting back into the root selectable file, and keeps the existing
  visual owner guard intact.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --no-fail-fast`: pass; 1 test.
- `cargo nextest run -p fret-imui interaction_shortcuts_selectable interaction_drag_multi_select
  models_combo --no-fail-fast`: pass; 11 tests, 170 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.

Gate note:

- The focused `fret-imui` nextest run waited on package-cache/build-directory locks before
  compiling; after the wait, the selected tests completed successfully in the same run.

## Floating Surface Kinds/State Owner-Split Evidence - 2026-05-26

Claim verified: IMUI floating-surface drag-kind and state records moved into focused private owners
without changing floating-area, floating-window, drag, resize, activation, collapse, or z-order
behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/floating_surface/kinds.rs` now owns floating-area drag-kind ids,
  floating-window resize-kind ids, resize-handle tags, and transient activation/collapse event
  keys.
- `ecosystem/fret-ui-kit/src/imui/floating_surface/state.rs` now owns
  `FloatingWindowChromeResponse`, `FloatingAreaState`, and `FloatWindowState`, including all
  floating-window test-id fields and resize/drag pointer state.
- `ecosystem/fret-ui-kit/src/imui/floating_surface.rs` keeps floating-area composition,
  pointer-region drag-surface wiring, layer registration, and private re-exports only.
- `tools/gate_imui_workstream_source.py` now requires the split floating-surface owner files and
  forbids drag-kind constants, resize-kind functions, and floating state records from drifting back
  into the root `floating_surface.rs` file.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass; 25 tests, 156 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_perf_guard_smoke --no-fail-fast`:
  pass; 5 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- The first parallel nextest attempt timed out after 4 minutes while cargo/rustc processes were
  still waiting/running under package-cache and target-dir contention. After those processes exited
  naturally, the same focused nextest gates passed when rerun serially.

## Container Options Owner-Split Evidence - 2026-05-26

Claim verified: IMUI container/layout option records moved into focused private owners without
changing public option names, fields, defaults, or top-level re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/containers/flow.rs` originally owned
  `HorizontalOptions`, `ItemFlowOptions`, `SameLineOptions`, `DummyOptions`, `SpacingOptions`,
  `IndentOptions`, `VerticalOptions`, `GridOptions`, and the private IMUI layout-token default
  helpers after the root container split. 2026-05-27 follow-up: `flow.rs` is now only the flow
  option module/re-export index; the private `flow/*` owners carry the concrete records and token
  defaults.
- `ecosystem/fret-ui-kit/src/imui/options/containers/scroll.rs` now owns `ScrollOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers/list_box.rs` now owns `ListBoxOptions`,
  keeping list-box policy as layout/scroll/diagnostics semantics only.
- `ecosystem/fret-ui-kit/src/imui/options/containers/child_region.rs` is the child-region option
  re-export hub. The 2026-05-28 child-region option owner split records the current child owners
  for `ChildRegionChrome`, `ChildRegionOptions`, `ChildRegionResizeXOptions`, and
  `ChildRegionResizeYOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs` is now a module/re-export index,
  preserving the existing public paths through `imui::options` and the root `imui` facade exports.
- `tools/gate_imui_workstream_source.py` now requires the split container-option owners and
  forbids flow, scroll, list-box, and child-region option bodies from drifting back into root
  `containers.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --test
  imui_virtual_list_smoke --no-fail-fast`: pass; 4 tests.
- `cargo nextest run -p fret-imui composition --no-fail-fast`: pass; 37 tests, 144 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- The focused `fret-ui-kit` and `fret-imui` runs waited on package-cache/build-directory locks,
  then completed successfully in the same runs.

## Collection Options Owner-Split Evidence - 2026-05-26

Claim verified: IMUI collection option records moved into focused private owners without changing
public table, table-column, row/cell, or virtual-list option names and defaults.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/collections/table_column.rs` now owns
  `TableColumnWidth`, `TableColumnResizeOptions`, `TableSortDirection`, `TableColumnPin`, and
  `TableColumn`, including identity inference, visibility, sorting, resizing, and pin helpers.
- `ecosystem/fret-ui-kit/src/imui/options/collections/table.rs` is the table-option re-export
  hub. The 2026-05-28 table option owner split records the current child owners for
  `TableOptions`, `TableRowOptions`, and `TableCellOptions`.
- `ecosystem/fret-ui-kit/src/imui/options/collections/virtual_list.rs` now owns
  `VirtualListOptions`, including viewport, measurement, cache, gap, scroll-margin, known-height,
  and handle defaults.
- `ecosystem/fret-ui-kit/src/imui/options/collections.rs` is now a 9-line module/re-export index,
  preserving the existing public paths through `imui::options`, `imui::options::controls`, and the
  root `imui` facade exports.
- `tools/gate_imui_workstream_source.py` now points the dedicated table-column accessor-first gate
  at the table-column owner and forbids table-column/table/virtual-list bodies from drifting back
  into root `collections.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --test
  imui_virtual_list_smoke --no-fail-fast`: pass; 10 tests.
- `cargo nextest run -p fret-imui composition --no-fail-fast`: pass; 37 tests, 144 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

## Text Options Owner-Split Evidence - 2026-05-26

Claim verified: IMUI text-control option records moved into focused private owners without changing
public option names, fields, defaults, or top-level re-export paths.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/options/controls/text/filters.rs` now owns `InputTextFilters`,
  named filter helpers, decimal/scientific character policy, and `InputTextCustomFilter`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/input.rs` is the current input-text option
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/input/mode.rs` owns `InputTextMode`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/input/options.rs` owns `InputTextOptions`,
  including text-field semantics and command-policy defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/picker.rs` is the current picker option
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/picker/filter.rs` owns
  `InputTextPickerFilter` and its matching policy.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/picker/options.rs` owns
  `InputTextPickerOptions`, including default popup sizing and picker behavior flags.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea.rs` is the current textarea option
  re-export hub.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea/submit_key.rs` owns
  `TextAreaSubmitKey`.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea/options.rs` owns
  `TextAreaOptions`, including multiline submit/cancel defaults.
- `ecosystem/fret-ui-kit/src/imui/options/controls/text.rs` is now a 9-line module/re-export index,
  preserving the existing public paths through `imui::options`, `imui::options::controls`, and the
  root `imui` facade exports.
- `tools/gate_imui_workstream_source.py` now requires the split text-option owners and forbids
  filter/input/picker/textarea bodies from drifting back into the root `text.rs` file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib text_controls::tests --no-fail-fast`:
  pass; 3 tests, 685 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_textarea_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-imui models_text --no-fail-fast`: pass; 29 tests, 152 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- `cargo nextest run -p fret-ui-kit --features imui --lib text_controls::tests --no-fail-fast`
  initially waited on a package-cache file lock, then completed successfully in the same run.

## Widget Response Owner-Split Evidence - 2026-05-26

Claim verified: IMUI widget response records moved into focused private owners without changing
public response type names, accessors, or composition behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/widgets/open.rs` now owns
  `DisclosureResponse` and `ComboResponse`, including their sealed trigger/open/toggled storage and
  read-only accessors.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/text_picker.rs` now owns
  `InputTextPickerResponse` and its input/open/picked accessors.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/tabs.rs` now owns `TabBarResponse` and
  `TabTriggerResponse`.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/table.rs` now owns `TableResponse`,
  `TableHeaderResponse`, and `TableColumnResizeResponse`, including resize drag readout and width
  clamp helpers.
- `ecosystem/fret-ui-kit/src/imui/response/widgets/virtual_list.rs` now owns
  `VirtualListResponse`.
- `ecosystem/fret-ui-kit/src/imui/response/widgets.rs` is now a 15-line module/re-export index
  beside the existing child-region owner.
- `tools/gate_imui_workstream_source.py` now requires the split widget-response owners and forbids
  response bodies, `ResponseExt`, `DragResponse`, table sort policy, and virtual-list handle
  storage from drifting back into the root `widgets.rs` file.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `cargo nextest run -p fret-imui composition --no-fail-fast`: pass; 37 tests, 144 skipped.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass.
- `git diff --check`: pass.

Gate note:

- `cargo nextest run -p fret-imui composition --no-fail-fast` initially waited on a package-cache
  file lock, then completed successfully in the same run.

## Container Methods Owner-Split Evidence - 2026-05-26

Claim verified: IMUI facade container-method dispatch moved into focused private owners without
changing public facade method names or collection/layout behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/flow.rs` now owns item-flow,
  same-line, dummy, spacing, and indent dispatch.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/layout.rs` now owns horizontal,
  vertical, grid, scroll, and child-region dispatch.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections.rs` now owns
  list-box, table, and virtual-list dispatch.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/menu_tabs.rs` now owns menu-bar
  and tab-bar dispatch.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods.rs` is now a 22-line re-export
  index that keeps the existing `container_methods::...` call sites stable.
- `tools/gate_imui_workstream_source.py` now requires the split owners and forbids collection,
  layout, and layout-sugar bodies from drifting back into the root container-methods file.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui composition --no-fail-fast`: pass; 37 tests, 144 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; 473 dedicated directories, 47 standalone
  markdown files.
- `git diff --check`: pass.

## Slider Controls Owner-Split Evidence - 2026-05-26

Claim verified: IMUI slider semantics, pointer/key interaction, and visual track/value badge
assembly moved into focused private owners without changing the public slider facade surface.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/slider_controls/a11y.rs` now owns slider semantics value,
  numeric range, step, and jump decoration.
- `ecosystem/fret-ui-kit/src/imui/slider_controls/interaction.rs` now owns interaction handler
  clearing, active/lifecycle model lookup, and pointer/keyboard owner dispatch. Pointer and
  keyboard behavior were split deeper into `interaction/pointer.rs` and `interaction/keyboard.rs`
  on 2026-05-27.
- `ecosystem/fret-ui-kit/src/imui/slider_controls/visual.rs` now owns track/fill geometry,
  progress calculation, caption text, and value badge assembly.
- `ecosystem/fret-ui-kit/src/imui/slider_controls.rs` keeps label identity parsing, option
  normalization, response population, and final element assembly.
- `tools/gate_imui_workstream_source.py` now requires the slider sub-owners and forbids semantics,
  pointer/key handler bodies, and visual `ContainerProps` assembly from drifting back into the root
  slider file.

Focused gates:

- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui slider --no-fail-fast`: pass; 2 tests, 179 skipped.
- `cargo nextest run -p fret-imui composition --no-fail-fast`: pass; 37 tests, 144 skipped.
- `cargo nextest run -p fret-imui label_identity --no-fail-fast`: pass; 7 tests, 174 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; 473 dedicated directories, 47 standalone
  markdown files.
- `git diff --check`: pass.

## Interaction Runtime Drag Owner-Split Evidence - 2026-05-26

Claim verified: IMUI drag runtime internals moved into focused private owners without changing
pressable drag, pointer-region drag/resize, active-item, or long-press behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/active_item.rs` now owns active-item
  mark/clear helpers.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/long_press_timer.rs` now owns long-press
  timer arm/cancel for pressable drag.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/pointer_region.rs` now owns
  pointer-region drag setup, thresholded movement, cancellation, pointer capture release, and
  finish handling.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/response.rs` now owns `DragResponse`
  population, delta/total tracking, and drag transient reads.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag.rs` keeps drag-kind/threshold helpers
  and the pressable drag move/down/up state machine.
- `tools/gate_imui_workstream_source.py` now requires the drag sub-owners and forbids active-item,
  pointer-region, long-press timer, and response bodies from drifting back into the root drag
  runtime file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass.
- `cargo nextest run -p fret-imui interaction_drag --no-fail-fast`: pass; 8 tests, 173 skipped.
- `cargo nextest run -p fret-imui interaction_press --no-fail-fast`: pass; 9 tests, 172 skipped.
- `cargo nextest run -p fret-imui floating --no-fail-fast`: pass; 25 tests, 156 skipped.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Interaction Runtime Hover Owner-Split Evidence - 2026-05-26

Claim verified: IMUI hover runtime internals moved into focused private owners without changing
hovered-query, shared-delay, active-item block, or long-press behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay.rs` now owns
  window-scoped shared hover delay state, shared delay timers, clear timer handling, and shared
  delay readout.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/timers.rs` now owns deterministic
  per-element hover timer token derivation.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/long_press.rs` now owns long-press
  timer emission into the existing `KEY_LONG_PRESSED` transient.
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover.rs` keeps the exported hover query
  helpers, active-item block read, local delay state accumulation, hook installation, and response
  readout.
- `tools/gate_imui_workstream_source.py` now requires the hover sub-owners and forbids shared
  delay/timer/long-press bodies from drifting back into the root hover runtime file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 tests, 160 skipped, with
  the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-imui interaction_press --no-fail-fast`: pass; 9 tests, 172 skipped,
  with the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests, with the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Table Render Helper Owner-Split Evidence - 2026-05-26

Claim verified: table render assembly no longer owns shared cell helpers, palette resolution, or
column test-id suffix policy; public table API names and behavior stay unchanged.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/table_controls/cell.rs` now owns shared table cell layout,
  padding, empty-cell, and cell-child packing helpers.
- `ecosystem/fret-ui-kit/src/imui/table_controls/palette.rs` now owns theme-to-table-palette
  resolution.
- `ecosystem/fret-ui-kit/src/imui/table_controls/test_ids.rs` now owns column test-id suffixing.
- `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` keeps table assembly,
  hidden-column handling, header/body response collection, and root table wrapping only.
- `tools/gate_imui_workstream_source.py` now requires the helper owners and forbids helper bodies
  from drifting back into `render.rs`.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_controls::tests --no-fail-fast`:
  pass; 7 tests, 681 skipped, with the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`:
  pass; 9 tests.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Debug-Draw Options Owner-Split Evidence - 2026-05-26

Claim verified: public debug draw options/style/vertex types moved out of
`debug_draw_controls.rs` without changing debug draw API names or behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options.rs` now owns `DebugDrawOptions`,
  interaction options, stroke style, rounding flags, image/svg options, and mesh vertex helper
  types.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` re-exports those public types and keeps
  draw-list state plus helper orchestration.
- `tools/gate_imui_workstream_source.py` now requires the options owner, root re-export shape, and
  crate-local visibility for style/vertex helpers used by paint modules.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests
  --no-fail-fast`: pass; 38 tests, 650 skipped, with the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Debug-Draw Response Owner-Split Evidence - 2026-05-26

Claim verified: `DebugDrawResponse` moved out of `debug_draw_controls.rs` without changing the
public debug draw response surface or debug draw behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls/response.rs` now owns `DebugDrawResponse`
  storage, constructor, and accessors.
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` re-exports `DebugDrawResponse` and keeps
  debug draw options, draw-list/style types, and helper orchestration.
- `tools/gate_imui_workstream_source.py` now points the opaque-output check at the response owner,
  requires the root re-export, and forbids the response body from drifting back into the root file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_debug_draw_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-ui-kit --features imui --lib debug_draw_controls::tests
  --no-fail-fast`: pass; 38 tests, 650 skipped, with the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

Gate note:

- The first two `debug_draw_controls::tests` runs timed out while shared cargo work continued in
  the background. Those processes were allowed to finish naturally; the same command was then rerun
  with a longer timeout and passed.

## Menu-Family Menu Owner-Split Evidence - 2026-05-26

Claim verified: top-level IMUI menu open/close orchestration moved out of
`menu_family_controls.rs` without changing facade menu behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu.rs` now owns
  `begin_menu_with_options(...)`, including trigger wiring, menubar active-menu policy updates,
  popup open/close requests, disabled cleanup, and `DisclosureResponse` population.
- `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs` keeps menubar policy state, menu-bar
  element construction, module wiring, and tests.
- `tools/gate_imui_workstream_source.py` now requires the menu owner, requires the root forwarding
  shape, and forbids menu orchestration from drifting back into the root file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --lib menu_family_controls::tests
  --no-fail-fast`: pass; 1 test, 687 skipped.
- `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast`: pass; 18 tests, 163
  skipped, with the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

Gate note:

- The first `fret-imui interaction_menu_tabs` run timed out while shared cargo work continued in
  the background. The process was allowed to finish naturally; the same command was then rerun and
  passed.

## Core-State Owner-Split Evidence - 2026-05-26

Claim verified: `ResponseExt` core response/id/enabled behavior moved out of the root hover
response owner without changing public response accessors or model/popup behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/hover/core_state.rs` now owns core
  `fret_authoring::Response`, id, enabled, clicked, changed, rect, hover, press, and focus
  mutators/accessors.
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs` still owns core/id/enabled storage, but no
  longer owns core-state behavior bodies.
- `tools/gate_imui_workstream_source.py` now requires the core-state owner and forbids those method
  bodies from drifting back into the root response storage file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass; 6 tests, 175 skipped,
  with the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 tests, 160 skipped, with
  the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

Gate note:

- The first `fret-imui models_controls` run timed out while shared cargo work continued in the
  background. The process was allowed to finish naturally; the same command was then rerun and
  passed.

## Hover-State Owner-Split Evidence - 2026-05-26

Claim verified: `ResponseExt` raw hover/nav/delay state behavior moved out of the root hover
response owner without changing popup/tooltip hovered-query behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/hover/hover_state.rs` now owns raw pointer-hover,
  popup-barrier hover, hover-delay, active-item block, and nav-highlight mutators plus read-only
  accessors.
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs` still owns the hover state storage fields, but
  no longer owns hover state behavior bodies.
- `tools/gate_imui_workstream_source.py` now requires the hover-state owner and forbids those
  method bodies from drifting back into the root response storage file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 tests, 160 skipped, with
  the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.

## Press/Context Owner-Split Evidence - 2026-05-26

Claim verified: `ResponseExt` press/context signal behavior moved out of the root hover response
owner without changing public press/context accessors or popup/menu behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/hover/press_context.rs` now owns secondary-click,
  double-click, long-press, hold, context-menu, pointer-click, pointer-modifier, and clear helpers
  plus read-only accessors.
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs` still owns the signal storage fields, but no
  longer owns press/context behavior bodies.
- `tools/gate_imui_workstream_source.py` now requires the press/context owner and forbids those
  method bodies from drifting back into the root response storage file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 tests, 160 skipped, with
  the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `git diff --check`: pass.

Gate note:

- The first `fret-imui popup_hover` run timed out while shared cargo work continued in the
  background. The process was allowed to finish naturally; the same command was then rerun and
  passed.

## Lifecycle Owner-Split Evidence - 2026-05-26

Claim verified: `ResponseExt` lifecycle signal behavior moved out of the root hover response owner
without changing public lifecycle accessors or model-control behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/hover/lifecycle.rs` now owns lifecycle signal mutators,
  merge helpers, clearing, and read-only accessors for activation, deactivation, edits, and
  deactivate-after-edit.
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs` still owns the lifecycle storage fields, but
  no longer owns lifecycle behavior bodies.
- `tools/gate_imui_workstream_source.py` now requires the lifecycle owner and forbids lifecycle
  method bodies from drifting back into the root response storage file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass; 6 tests, 175 skipped,
  with the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `git diff --check`: pass.

Gate note:

- The first parallel `fret-ui-kit` and `fret-imui` nextest runs timed out while shared cargo work
  was still compiling. Both processes were allowed to finish naturally; the same commands were then
  rerun serially and passed.

## Hover Query Owner-Split Evidence - 2026-05-26

Claim verified: IMUI hovered query flags and query policy moved out of the `ResponseExt` storage
owner without changing the public hover API or popup/tooltip hover behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/response/hover/flags.rs` now owns `ImUiHoveredFlags`, bitwise
  composition, and the ImGui-style flag constants.
- `ecosystem/fret-ui-kit/src/imui/response/hover/query.rs` now owns `hovered_like_imgui(...)` and
  `is_hovered(...)`, including tooltip expansion, popup/active-item overrides, stationary delay,
  and shared-delay behavior.
- `ecosystem/fret-ui-kit/src/imui/response/hover.rs` stays focused on `ResponseExt` storage,
  crate-local mutators, public accessors, and drag convenience helpers.
- `tools/gate_imui_workstream_source.py` now requires the new flags/query owners, requires the
  root module forwarding shape, and forbids the flag/query implementation from drifting back into
  the storage owner.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke
  --no-fail-fast`: pass; 2 tests.
- `cargo nextest run -p fret-imui popup_hover --no-fail-fast`: pass; 21 tests, 160 skipped, with
  the same pre-existing `fret-ui` warnings.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `git diff --check`: pass.

Gate note:

- The first `fret-imui popup_hover` run timed out while compiling behind shared cargo/rustc work;
  the orphaned cargo process was allowed to finish naturally, then the same command was rerun and
  passed.

## Boolean Visual Owner-Split Evidence - 2026-05-26

Claim verified: IMUI checkbox/radio/switch visual chrome moved out of behavior files without
changing the public boolean-control surface.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/boolean_controls/visual.rs` owns checkbox badges, radio
  indicators, switch state badges, and shared boolean label text.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs` keeps checkbox/radio label identity,
  pressable behavior, shortcut/context-menu handling, model reads/updates, and `ResponseExt`
  population.
- `ecosystem/fret-ui-kit/src/imui/boolean_controls/switch.rs` keeps switch active-trigger behavior,
  shortcut handling, model updates, and response population while delegating badge/label rendering
  to the shared visual owner.
- `tools/gate_imui_workstream_source.py` now requires the visual owner and rejects checkbox/radio
  badge or switch state-badge rendering from drifting back into behavior files.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `git diff --check`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test
  imui_adapter_seam_smoke --no-fail-fast`: pass; 4 tests.
- `cargo nextest run -p fret-imui models_controls --no-fail-fast`: pass; 6 tests.

Gate note:

- The `fret-imui models_controls` run waited on external package-cache/build-directory locks before
  compiling, but completed successfully.

## Disclosure Spec Owner-Split Evidence - 2026-05-26

Claim verified: IMUI disclosure spec construction and option normalization moved out of
`disclosure_controls.rs` without changing the public collapsing-header or tree-node surface.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/spec.rs` owns private `DisclosureKind`,
  `DisclosureSpec`, option-to-spec normalization, tree level clamping, test-id routing, and
  leaf/children classification.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs` keeps pressable behavior,
  keyboard/context-menu activation, model/toggle wiring, content mounting, and
  `DisclosureResponse` population.
- `ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual.rs` consumes the private spec owner
  for a11y and visual policy without owning model construction.
- `tools/gate_imui_workstream_source.py` now requires the split owner and rejects
  `DisclosureKind` / `DisclosureSpec` definitions from drifting back into the root behavior file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `git diff --check`: pass.
- `cargo nextest run -p fret-ui-kit --features imui --lib disclosure_controls::tests
  --no-fail-fast`: pass; 6 tests.

Gate note:

- A first `cargo nextest run -p fret-ui-kit --features imui disclosure_controls::tests
  --no-fail-fast` attempt timed out because it omitted `--lib` and started compiling broader test
  targets. The corrected lib-only command above passed.

## Text Picker Owner-Split Evidence - 2026-05-26

Claim verified: IMUI input-text picker candidate visibility and keyboard state reconciliation moved
out of `text_picker_controls.rs` without changing the public completion/history picker surface.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/candidates.rs` owns candidate filtering,
  `max_items`, exact-match hiding, and open-when-empty visibility decisions.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard.rs` now also owns active-source
  cleanup and pending keyboard pick extraction through `reconcile_picker_keyboard_state(...)`.
- `ecosystem/fret-ui-kit/src/imui/text_picker_controls.rs` keeps input/popup composition,
  selectable item rendering, model update-on-pick, and `InputTextPickerResponse` merging.
- `tools/gate_imui_workstream_source.py` now requires the split owners and rejects candidate
  filtering / keyboard reconciliation from drifting back into the root picker file.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo fmt -p fret-ui-kit --check`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\gate_imui_workstream_source.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `git diff --check`: pass.
- `CARGO_TARGET_DIR=target-codex-text-picker cargo nextest run -p fret-imui models_text_picker
  --no-fail-fast`: pass; 6 tests.

Gate note:

- The first shared-target `cargo nextest run -p fret-imui models_text_picker --no-fail-fast`
  attempt was interrupted after timeout, leaving an orphan `rustc`. A second shared-target attempt
  hit MSVC `LNK1120` unresolved externals from the polluted incremental artifact. The same test set
  passed from the isolated `target-codex-text-picker` target directory.

## Button Visual Owner-Split Evidence - 2026-05-26

Claim verified: IMUI button visual/layout/accessibility ownership moved out of
`button_controls.rs` without changing the public button facade or action/response behavior.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` owns button variant layout,
  `PressableA11y` construction, arrow glyph/label mapping, and button chrome/content assembly.
- `ecosystem/fret-ui-kit/src/imui/button_controls.rs` keeps label identity scoping, enabled/action
  gating, keyboard shortcut/context-menu handling, command dispatch, transient events, and
  `ResponseExt` population.
- `tools/gate_imui_workstream_source.py` now requires the split owner and rejects the old visual
  helpers from returning to the root button behavior file.
- Gate repair while proving this slice: `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` now routes
  `DropdownMenuLabel` text through `decl_text::text_menu_group_label(...)`, matching the existing
  IMUI source-policy gate and the ContextMenu/Menubar label owner pattern.

Focused gates:

- `cargo fmt -p fret-ui-kit`: pass.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing `fret-ui` warnings
  for `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo fmt -p fret-ui-kit -p fret-ui-shadcn --check`: pass.
- `cargo check -p fret-ui-shadcn --lib`: pass with the same pre-existing `fret-ui` warnings.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --no-fail-fast`:
  pass; 1 test.
- `cargo nextest run -p fret-ui-shadcn --lib
  dropdown_menu_label_element_uses_shared_menu_group_text_role --no-fail-fast`: pass; 1 test
  after compiling the large shadcn lib test binary.
- `python -m py_compile tools\gate_imui_workstream_source.py`: pass.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`:
  pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `python tools\gate_imui_workstream_source.py`: pass.
- `git diff --check`: pass.

## Worktree Convergence Evidence - 2026-05-26

Claim verified: the dirty `main` IMUI work and the dirty
`imui-imgui-editor-grade-refactor` worktree were checkpointed, merged into `main`, conflict-resolved
by topic, and verified with focused gates before continuing development from `main`.

Checkpoints:

- `d078e25122 refactor(imui): checkpoint gap closure convergence slices`
- `05727e284b refactor(imui): checkpoint editor-grade convergence worktree`

Merge resolution evidence:

- `facade_writer.rs` keeps the editor-grade owner split and routes ListBox helpers through
  `container_methods.rs`.
- `facade_writer/image_items.rs` remains a dedicated owner from the main checkpoint.
- `fret-ui-kit::imui` option exports include both checkpoint surfaces.
- `fret-imui` composition tests import `ScrollHandle` only where needed.
- `gate_imui_workstream_source.py` carries the merged source-policy checks.
- `docs/workstreams/README.md` reflects the reconciled catalog count of 445 dedicated directories.

Fresh gates run for this convergence:

- `git diff --cached --check`: pass.
- `git diff --check`: pass.
- `python -m py_compile tools\gate_imui_workstream_source.py tools\diag_gate_imui_p2_devtools_first_open.py tools\diag_gate_imui_product_chain.py`: pass.
- `python tools\check_workstream_catalog.py`: pass; validated 445 dedicated directories and 47
  standalone markdown files.
- `python tools\gate_imui_workstream_source.py`: pass.
- `cargo fmt --check -p fret-ui-kit -p fret-imui -p fret-plot -p fret-ui-editor -p fret-examples -p fret-demo -p fretboard-dev -p fret-devtools -p fret-devtools-mcp`: pass.
- `cargo fmt --check -p fret-imui -p fret-ui-kit`: pass after removing the unused harness
  `ScrollHandle` re-export.
- `cargo check -p fret-ui-kit --features imui --lib`: pass with pre-existing warnings for
  `unexpected_cfgs` on `unstable-retained-bridge` and `dead_code` on
  `current_effective_opacity`.
- `cargo check -p fret-plot`: pass with the same `fret-ui` warnings and existing dead-code
  warnings for axis-lock helpers.
- `cargo check -p fret-plot --features imui`: pass with the same warnings.
- `cargo check -p fret-ui-editor --features imui`: pass.
- `cargo check -p fret-demo --bin imui_editor_workbench_demo`: pass with existing warnings from
  `fret-ui`, `fret-plot`, and `fret-chart`.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_image_item_smoke --test imui_table_smoke --test imui_debug_draw_smoke --test imui_child_region_smoke --test imui_virtual_list_smoke --no-fail-fast`: pass; 16 tests.
- `cargo nextest run -p fret-ui-editor --features imui --no-fail-fast`: pass; 189 tests.
- `CARGO_TARGET_DIR=target-codex-merge cargo test -p fret-examples --test imui_editor_workbench_golden_path_surface -- --nocapture`: pass; 2 tests.
- `CARGO_TARGET_DIR=target-codex-merge cargo test -p fret-imui --lib layout_collections -- --nocapture`: pass; 28 tests.
- `CARGO_TARGET_DIR=target-codex-merge cargo test -p fret-imui --lib models_text_picker -- --nocapture`: pass; 6 tests.
- `CARGO_TARGET_DIR=target-codex-merge cargo test -p fret-imui --lib item_pointer -- --nocapture`: pass; 5 tests.

Gate notes:

- Broad `cargo nextest run -p fret-examples --test imui_editor_workbench_golden_path_surface` and
  `cargo nextest run -p fret-imui --no-fail-fast` attempts timed out while waiting on heavy
  workspace compilation/build locks, with no test failure output. The same claims were rechecked
  with isolated `target-codex-merge` focused cargo test runs.
- One attempted command targeted `fret-imui` with `fret-ui-kit` test names and failed because those
  test targets do not belong to that package; it is not a code failure.

## Evidence Anchors

- Current lane:
  - `docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-imgui-gap-closure-v1/DESIGN.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P0_CURRENT_SOURCE_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLEANUP_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLOSEOUT_AUDIT_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P2_GOLDEN_PATH_PROMOTION_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_DESIGN_SURFACE_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_PORTING_SUGAR_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_CHILD_REGION_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_COLLECTION_HELPER_READINESS_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/TODO.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/MILESTONES.md`
  - `docs/workstreams/imui-imgui-gap-closure-v1/WORKTREE_CONVERGENCE_PLAN_2026-05-26.md`
- Current Fret IMUI source:
  - `ecosystem/fret-imui/src/lib.rs`
  - `ecosystem/fret-imui/src/frontend.rs`
  - `ecosystem/fret-imui/src/tests/mod.rs`
  - `ecosystem/fret-imui/src/tests/harness/mod.rs`
  - `ecosystem/fret-imui/src/tests/harness/events.rs`
  - `ecosystem/fret-imui/src/tests/harness/floating_scenes.rs`
  - `ecosystem/fret-imui/src/tests/harness/frames.rs`
  - `ecosystem/fret-imui/src/tests/harness/host.rs`
  - `ecosystem/fret-imui/src/tests/harness/hover_scenes.rs`
  - `ecosystem/fret-imui/src/tests/harness/lookup.rs`
  - `ecosystem/fret-imui/src/tests/harness/services.rs`
  - `ecosystem/fret-imui/src/tests/composition/mod.rs`
  - `ecosystem/fret-imui/src/tests/floating/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_drag/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_menu_tabs/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_press/mod.rs`
  - `ecosystem/fret-imui/src/tests/interaction_shortcuts/mod.rs`
  - `ecosystem/fret-imui/src/tests/label_identity/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_combo/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_controls/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_text_area/mod.rs`
  - `ecosystem/fret-imui/src/tests/models_text_picker/mod.rs`
  - `ecosystem/fret-imui/src/tests/popup_hover/mod.rs`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/declarative/text.rs`
  - `ecosystem/fret-ui-kit/src/declarative/file_tree.rs`
  - `ecosystem/fret-ui-kit/src/declarative/table.rs`
  - `ecosystem/fret-ui-kit/src/imui/control_chrome.rs`
  - `ecosystem/fret-ui-kit/src/imui/boolean_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/boolean_controls/switch.rs`
  - `ecosystem/fret-ui-kit/src/imui/boolean_controls/visual.rs`
  - `ecosystem/fret-ui-kit/src/imui/disclosure_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/disclosure_controls/spec.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/menu_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/selectable_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/separator_text_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/header/trigger.rs`
  - `ecosystem/fret-ui-kit/src/imui/table_controls/header/resize.rs`
  - `ecosystem/fret-plot/Cargo.toml`
  - `ecosystem/fret-plot/src/lib.rs`
  - `ecosystem/fret-plot/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/list_box_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/options/containers.rs`
  - `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`
  - `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/basic_items.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs`
  - `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`
  - `ecosystem/fret-ui-kit/src/imui/image_item_controls.rs`
  - `ecosystem/fret-ui-kit/src/imui/response/widgets.rs`
  - `ecosystem/fret-ui-kit/tests/imui_image_item_smoke.rs`
  - `ecosystem/fret-ui-editor/src/imui.rs`
  - `ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs`
  - `ecosystem/fret-ui-editor/src/primitives/input_group.rs`
  - `ecosystem/fret-ui-editor/src/primitives/popup_list.rs`
  - `ecosystem/fret-ui-editor/src/primitives/readout.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_group.rs`
  - `ecosystem/fret-ui-editor/src/composites/property_row.rs`
  - `ecosystem/fret-ui-editor/src/controls/field_status.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/copy.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/numeric.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/options.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview.rs`
  - `ecosystem/fret-ui-editor/src/controls/color_edit/popup/tooltip.rs`
  - `ecosystem/fret-ui-editor/src/controls/drag_value.rs`
  - `ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs`
  - `ecosystem/fret-ui-editor/src/controls/enum_select.rs`
  - `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs`
  - `ecosystem/fret/src/lib.rs`
  - `apps/fret-cookbook/src/lib.rs`
- Closed image item and child resize follow-ons:
  - `docs/workstreams/imui-image-item-proof-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-image-item-proof-v1/DESIGN.md`
  - `docs/workstreams/imui-image-item-proof-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-resize-y-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-resize-y-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-child-region-resize-x-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-child-region-resize-x-v1/EVIDENCE_AND_GATES.md`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/WORKSTREAM.json`
  - `docs/workstreams/imui-selectable-highlight-policy-v1/EVIDENCE_AND_GATES.md`
- Current proof surfaces:
  - `apps/fret-cookbook/README.md`
  - `apps/fret-cookbook/EXAMPLES.md`
  - `docs/examples/README.md`
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
  - `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
  - `apps/fret-cookbook/examples/imui_editor_controls_basics.rs`
  - `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
  - `apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs`
  - `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - `tools/gate_imui_editor_collection_source.py`
  - `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`
  - `apps/fret-examples/src/workspace_shell_demo.rs`
  - `ecosystem/fret-ui-kit/src/imui/multi_select.rs`
  - `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs`
  - `ecosystem/fret-ui-kit/src/recipes/imui_drag_preview.rs`
  - `apps/fret-examples/src/editor_notes_demo.rs`
  - `apps/fret-examples/src/docking_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`
  - `apps/fret-examples/src/container_queries_docking_demo.rs`
  - `apps/fret-examples/src/form_demo.rs`
  - `apps/fret-examples/src/sonner_demo.rs`
  - `apps/fret-examples/src/echarts_demo.rs`
  - `apps/fret-examples/tests/docking_demo_surface.rs`
  - `apps/fret-examples/tests/docking_arbitration_surface.rs`
  - `apps/fret-examples/tests/container_queries_docking_surface.rs`
  - `apps/fret-examples/tests/form_demo_surface.rs`
  - `apps/fret-examples/tests/sonner_demo_surface.rs`
  - `apps/fret-examples/tests/echarts_demo_surface.rs`
  - `apps/fret-ui-gallery/src/driver/toaster.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/sidebar/app_sidebar.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/command/behavior_demos.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/command/composable_shell.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/borders.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/card.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/disabled.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/focusable_disabled.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/multiple.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/showcase.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/accordion/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/flex_1_items.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/full_width_items.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/outline.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/spacing.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle_group/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/disabled.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/label.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/outline.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/toggle/with_text.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/button/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tabs/icons.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tabs/parts.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tabs/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/controlled_state.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/file_tree.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/collapsible/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/rich_content.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/alert_dialog/small_with_media.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/basic.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/children.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/demo.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/positioning.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/rtl.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/sides.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/trigger_delays.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/hover_card/usage.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/popover/align.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/tooltip/keyboard_shortcut.rs`
  - `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/mvp/gates.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
  - `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
  - `ecosystem/fret-ui-ai/src/elements/mod.rs`
  - `ecosystem/fret-ui-ai/src/surface_policy_tests.rs`
  - `tools/gate_imui_workstream_source.py`
  - `tools/diag_gate_imui_product_chain.py`
  - `tools/diag_gate_imui_p2_devtools_first_open.py`
- Prior status:
  - `docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  - `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md`
- Dear ImGui reference:
  - `repo-ref/imgui/imgui.h`
  - `repo-ref/imgui/imgui.cpp`
  - `repo-ref/imgui/imgui_draw.cpp`
  - `repo-ref/imgui/imgui_demo.cpp`
  - `repo-ref/imgui/docs/BACKENDS.md`

## P3 Public Surface Catalog Gates

Use these for the current public-surface catalog note:

```powershell
python tools/audit_crate.py --crate fret-imui
python tools/audit_crate.py --crate fret-ui-kit
python tools/audit_crate.py --crate fret-ui-editor
python tools/audit_crate.py --crate fret
rg -n "pub mod imui|pub use fret_imui|pub use fret_ui_kit::imui|pub mod kit|pub mod editor|pub mod docking|pub mod prelude" ecosystem/fret/src/lib.rs
cargo nextest run -p fret root_surface_exposes_explicit_imui_module readme_and_rustdoc_expose_imui_as_explicit_optional_surface --no-fail-fast
cargo check -p fret --no-default-features --features imui
```

## P3 Component Surface Catalog Gates

Use these for the current component-surface catalog note:

```powershell
rg --files ecosystem/fret-ui-kit/src/imui ecosystem/fret-ui-kit/tests
rg -n "pub use debug_draw_controls|pub use options|pub use response|pub use tab_family_controls::ImUiTabBar|pub use table_controls" ecosystem/fret-ui-kit/src/imui.rs
rg -n "fn (text|text_wrapped|button|small_button|arrow_button|checkbox_model|radio|switch_model|slider_f32_model|combo|combo_model|selectable|multi_selectable|tree_node|collapsing_header|child_region|virtual_list|table|tab_bar|open_popup|begin_popup|tooltip|drag_source|drop_target|debug_draw)" ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer
rg -n "pub fn (text_field|checkbox|color_edit|drag_value|numeric_input|slider|enum_select|property_grid|gradient_editor|inspector_panel)" ecosystem/fret-ui-editor/src/imui.rs
rg -n "Widgets: Text|Widgets: Main|Widgets: Combo Box|Widgets: Trees|Widgets: Selectables|Widgets: List Boxes|Widgets: Data Plotting|Widgets: Menus|Tooltips|Popups, Modals|Tables|Tab Bars|Drag and Drop|Debug Utilities" repo-ref/imgui/imgui.h
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --test imui_table_smoke --test imui_disclosure_smoke --test imui_textarea_smoke --test imui_drag_drop_smoke --test imui_virtual_list_smoke --test imui_debug_draw_smoke --test imui_tooltip_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast
```

Run evidence:

- 2026-05-14: made `FloatingAreaContext` externally opaque in
  `ecosystem/fret-ui-kit/src/imui/floating_options.rs`. The facade still hands callers a context
  with `id()`, `position()`, and `drag_kind()` accessors, but external code can no longer construct
  or mutate area identity / drag-kind fields. `tools/gate_imui_workstream_source.py` rejects public
  fields from returning.
- 2026-05-14: made `FloatingAreaResponse` / `FloatingWindowResponse` accessor-first too. Floating
  response identity, geometry, drag, resize, and collapse state now stay behind methods instead of
  public fields, and the floating tests use `resp.id()` instead of `resp.area.id`.
- 2026-05-16: refreshed the public `window(...)` posture in
  `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`. The source docs now describe the current
  in-window floating surface instead of deferring z-order/focus arbitration to a future work item.
  Evidence anchors: `ecosystem/fret-imui/src/tests/floating/movement_z_order.rs` covers
  bring-to-front hit-test order, `input_modes.rs` covers focus-on-click vs activation plus
  no-inputs / pointer-pass-through behavior, and `window_options.rs` covers close, resize, and
  collapse policy.
- 2026-05-16: narrowed the table advanced-gap wording in
  `docs/workstreams/imui-imgui-gap-closure-v1/P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`.
  `TableOptions::striped` remains the existing alternating row-background policy, with proof in
  `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`,
  `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`, and
  `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`.
- 2026-05-16: landed the explicit table row/cell background override slice. Evidence anchors:
  `TableRowOptions::background`, `TableCellOptions::background`,
  `ImUiTableRow::cell_with_options(...)`, `ImUiTableRow::cell_text_with_options(...)`,
  `ecosystem/fret-ui-kit/tests/imui_table_smoke.rs`, and
  `ecosystem/fret-imui/src/tests/composition/layout_collections.rs`.
  `table_helper_applies_explicit_row_and_cell_background_overrides` proves that explicit cell
  backgrounds paint after explicit row backgrounds.
- 2026-05-16: introduced `text_table_cell(...)` as the first shared table-cell text role and wired
  `ImUiTableRow::cell_text(...)` through it instead of bare paragraph text. Gate:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  table_cell_text_uses_compact_single_line_truncation --no-fail-fast`.
- 2026-05-16: routed sortable and plain IMUI table header labels through
  `text_table_cell(...)` too. Header labels now share compact single-line ellipsis semantics with
  body cells instead of default word wrapping. Gate: `cargo nextest run -p fret-ui-kit --features
  imui --lib table_header_label_uses_shared_table_cell_text_role --no-fail-fast`.
- 2026-05-17: routed sortable IMUI table header sort indicators through `text_chrome_glyph(...)`
  instead of bare `cx.text(...)`. Header labels remain table-cell text, while sort arrows are now
  fixed chrome glyphs with single-line clip semantics. Gate: `cargo nextest run -p fret-ui-kit
  --features imui --lib table_sort_indicator_uses_shared_chrome_glyph_text_role
  --no-fail-fast`.
- 2026-05-16: added static table column visibility through `TableColumn::hidden()` and
  `TableColumn::with_visible(bool)`. Hidden columns still consume author-submitted row cells in
  declared column order, but they do not render header/body cells and do not emit header responses.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  hidden_table_columns_do_not_render_header_body_or_response --no-fail-fast`, `cargo nextest run -p
  fret-ui-kit --features imui --test imui_table_smoke table_column_visibility_helpers_compile
  --no-fail-fast`, and `cargo nextest run -p fret-imui
  table_helper_skips_hidden_columns_in_header_and_body --no-fail-fast`.
- 2026-05-17: added runtime table-column visibility state through
  `ImUiTableColumnVisibilityState`. The helper stays in `fret-ui-kit::imui`, keeps storage opaque,
  applies stable-id overrides to `TableColumn` lists before render, and intentionally leaves
  persistence, freeze panes, and durable column storage outside the model helper. Gates: `cargo
  nextest run -p fret-ui-kit --features imui --lib
  visibility_state_applies_runtime_overrides_by_stable_column_id
  visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility
  visibility_state_toggle_uses_current_override_or_default_visibility --no-fail-fast`, `cargo
  nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_visibility_state_applies_runtime_visibility_by_column_id --no-fail-fast`, `cargo
  nextest run -p fret-imui table_helper_applies_runtime_column_visibility_state
  table_helper_skips_hidden_columns_in_header_and_body --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-17: added a table column visibility menu-item bridge through
  `table_column_visibility_menu_item(...)`. The helper stays in `fret-ui-kit::imui`, reuses the
  existing checkbox menu-item behavior, updates `ImUiTableColumnVisibilityState`, and remains
  usable for custom popup/menu surfaces even after the default header context-menu helper below.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`,
  `cargo nextest run -p fret-imui table_column_visibility_menu_item_updates_visibility_state
  --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: added a table column visibility menu-items group helper through
  `table_column_visibility_menu_items(...)`. The helper filters to stable-id, human-labeled
  columns, clones caller-owned item options, returns opaque/accessor-first per-column responses,
  and feeds both custom popup surfaces and the default header context-menu helper below. Gates:
  `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`,
  `cargo nextest run -p fret-imui
  table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns
  --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: added table header context-menu request reporting for sortable and plain headers
  through a shared private header trigger surface in `fret-ui-kit::imui`. Sortable headers still
  own button-like primary activation/click lifecycle, while plain headers only expose context-menu
  request signals. `TableHeaderResponse::response()` now reports right-click context-menu requests
  with a pointer anchor, plus keyboard requests from the ContextMenu key and Shift+F10; the helper
  below consumes this response signal for the default visibility menu surface. 2026-05-27
  follow-up: `trigger.rs` keeps header trigger props and sortable visuals while
  `trigger/behavior.rs` owns active-trigger installation and response population. Gates: `cargo
  nextest run -p fret-imui table_plain_header_left_click_does_not_activate_or_click
  table_plain_header_reports_context_menu_request_from_keyboard_without_clicking
  table_column_visibility_header_context_menu_opens_from_plain_header
  table_sortable_header_reports_context_menu_request --no-fail-fast`, `cargo nextest run -p
  fret-imui interaction_press interaction_menu_tabs --no-fail-fast`, and `cargo nextest run -p
  fret-ui-kit --features imui --test imui_response_contract_smoke --test imui_table_smoke
  --no-fail-fast`.
- 2026-05-17: added automatic table header visibility-menu wiring through
  `table_column_visibility_header_context_menu(...)`. The helper stays in `fret-ui-kit::imui`,
  scans `TableResponse` header responses from both sortable and plain headers for context-menu
  requests, opens a popup menu with the existing popup policy, renders
  `table_column_visibility_menu_items(...)`, and returns an opaque/accessor-first response.
  `TableColumnVisibilityHeaderContextMenuOptions` exposes popup and menu-item policy instead of
  hard-coding placement/sizing. Callers still own when to apply `ImUiTableColumnVisibilityState` to
  their columns; persistence, freeze panes, and old columns API shape were still separate
  follow-ons at that historical point.
  Gates: `cargo nextest run -p fret-imui
  table_column_visibility_header_context_menu_opens_from_plain_header
  table_column_visibility_header_context_menu_opens_and_updates_state
  table_column_visibility_menu_items_update_shared_visibility_state_and_filter_columns
  table_sortable_header_reports_context_menu_request --no-fail-fast`, `cargo nextest run -p
  fret-ui-kit --features imui --test imui_table_smoke --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-17: added the narrow persistence seam for runtime table-column visibility through
  `TableColumnVisibilitySnapshot` and `TableColumnVisibilityEntry`. The public data shape
  serializes stable column ids as `id` and visible flags as `visible`, while
  `ImUiTableColumnVisibilityState::snapshot()`, `from_snapshot(...)`, and
  `replace_from_snapshot(...)` keep runtime storage opaque and caller-owned. Restore ignores empty
  ids and duplicate entries use last-entry-wins. This does not add file storage, schema registry,
  freeze panes, or a mutable table runtime to `fret-imui`. Gates: `cargo nextest run -p
  fret-ui-kit --features imui --lib visibility_state_snapshot_roundtrips_stable_column_ids
  visibility_state_snapshot_restore_ignores_empty_ids_and_last_entry_wins --no-fail-fast`,
  `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_visibility_snapshot_api_compiles_and_roundtrips
  table_column_visibility_snapshot_entries_are_public_data_shape --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-17: added IMUI table column pinning as the first narrow freeze-pane seam. `TableColumn`
  now exposes `TableColumn::pinned_left()`, `TableColumn::pinned_right()`, and `with_pin(...)`;
  `TableOptions` accepts an optional `horizontal_scroll` handle. The helper render path partitions visible header/body cells
  into left/center/right groups and keeps frozen left/right cells outside the shared center X-scroll
  region. This uses `fret-ui` scroll mechanics and does not add table-state storage to `fret-imui`.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`,
  `cargo nextest run -p fret-ui-kit --features imui --test imui_table_smoke
  table_column_pinning_helpers_compile --no-fail-fast`, `cargo nextest run -p fret-imui
  table_helper_pins_left_and_right_columns_while_center_columns_scroll --no-fail-fast`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: added the first accessor-first cleanup for `TableColumn`. New read accessors cover
  `header()`, `id()`, `width()`, `visible()`, `is_sortable()`, `sort_direction()`,
  `resize_options()`, and `pin()`. `ImUiTableColumnVisibilityState::apply_to_columns(...)` now uses
  a crate-local visibility mutator, while table rendering, visibility menu policy, `fret-imui`
  composition tests, and public smoke tests use read accessors instead of direct field reads. This
  prepared the private-field hardening follow-up below. Gates: `cargo nextest run -p fret-ui-kit --features imui --test
  imui_table_smoke table_column_helpers_compile table_column_visibility_helpers_compile
  table_resizable_column_api_compiles table_sortable_header_api_compiles --no-fail-fast`, `cargo
  nextest run -p fret-ui-kit --features imui --lib
  visibility_state_applies_runtime_overrides_by_stable_column_id
  visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`, `cargo
  nextest run -p fret-imui table_helper_pins_left_and_right_columns_while_center_columns_scroll
  table_column_visibility_menu_item_updates_visibility_state --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: completed the `TableColumn` private-field hardening follow-up. The struct is now
  builder/accessor-first instead of a public option-data bag; fields are private, render and
  visibility-policy internals keep crate-local `header_arc(...)`, `id_arc(...)`, and
  `set_visible_for_policy(...)` seams, and `tools/gate_imui_workstream_source.py` rejects the old
  public field shape from returning. Gates: `cargo nextest run -p fret-ui-kit --features imui
  --test imui_table_smoke table_column_helpers_compile table_column_visibility_helpers_compile
  table_resizable_column_api_compiles table_sortable_header_api_compiles --no-fail-fast`, `cargo
  nextest run -p fret-ui-kit --features imui --lib
  visibility_state_applies_runtime_overrides_by_stable_column_id
  visibility_state_leaves_unlisted_and_unidentified_columns_at_declared_visibility
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`, `cargo
  nextest run -p fret-imui table_helper_pins_left_and_right_columns_while_center_columns_scroll
  table_column_visibility_menu_item_updates_visibility_state --no-fail-fast`, and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: refreshed the current table-gap wording after the visibility snapshot,
  freeze-pane pinning, and `TableColumn` private-field slices landed. These are no longer current table gaps in this workstream.
  Future table work should only open app/editor storage/schema policy or concrete Dear ImGui
  table-runtime parity lanes with a new proof and gate. Gates:
  `python tools/gate_imui_workstream_source.py`, `python -m py_compile
  tools/gate_imui_workstream_source.py`, and `git diff --check`.
- 2026-05-16: introduced `text_control_readout(...)` as the shared compact control-readout text
  role. The UI Gallery code-editor toolbar keeps its doc-layout helper, but that helper now
  delegates to `fret-ui-kit::declarative::text::text_control_readout(...)`, so dense status/readout
  text no longer carries app-local wrap/overflow policy. Gate: `cargo nextest run -p fret-ui-kit
  --features imui --lib control_readout_text_uses_muted_compact_single_line_truncation
  --no-fail-fast`.
- 2026-05-16: introduced `text_button_label(...)` as the shared compact button-label text role and
  routed IMUI `control_text(...)` through it. IMUI buttons and pill-style control labels now keep
  single-line truncation instead of inheriting word-wrap text semantics. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib button_label_text_uses_medium_single_line_truncation
  imui::control_chrome::tests::imui_control_text_uses_shared_button_label_role --no-fail-fast`.
- 2026-05-16: introduced `text_code_block(...)` as the shared code-block text role beside the
  existing inline/wrapping `text_code_wrap(...)`. The UI Gallery docs scaffold now uses the shared
  role for scrollable code blocks instead of constructing monospace `TextProps` locally. Gate:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  prose_variants_and_code_wrap_install_semantic_inherited_overrides --no-fail-fast`.
- 2026-05-16: introduced `text_paragraph(...)` and `text_paragraph_break_words(...)` as the stable
  semantic paragraph role names over the existing `text_prose(...)` helpers. This closes the first
  text-role vocabulary pass without breaking shadcn/Tailwind-oriented naming. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  prose_variants_and_code_wrap_install_semantic_inherited_overrides --no-fail-fast`.
- 2026-05-17: added a shared text-role layout gate in
  `fret-ui-kit::declarative::text`. The gate uses a wrapping fake text service plus
  `UiTree::layout_all(...)` to prove the base single-line roles (`text_control_readout(...)`,
  `text_button_label(...)`, `text_table_cell(...)`, and `text_code_block(...)`) stay one measured
  line under narrow resize, while `text_paragraph(...)` measures as multiple lines. Gate:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  base_single_line_text_roles_stay_single_line_under_narrow_layout
  paragraph_text_role_measures_multiple_lines_under_narrow_layout --no-fail-fast`.
- 2026-05-17: introduced `text_compact_paragraph(...)` as the shared dense wrapping paragraph role
  for editor/IMUI body copy. IMUI `bullet_text(...)` labels and
  `UiWriterImUiFacadeExt::text_wrapped(...)` now route through it, preserving explicit wrapping
  while moving fill-width/min-width-zero layout policy out of local `TextProps`. Gate: `cargo
  nextest run -p fret-ui-kit --features imui --lib
  compact_paragraph_text_uses_wrapping_fill_width_layout
  bullet_text_uses_shared_compact_paragraph_role imui_text_wrapped_is_explicit_wrapping_text
  --no-fail-fast`.
- 2026-05-18: routed IMUI `tooltip_text(...)` / `tooltip_text_with_options(...)` body copy through
  a private `tooltip_body_text(...)` helper backed by `text_compact_paragraph(...)`. This keeps the
  convenience tooltip path on dense wrapping body/help text, while rich-content `tooltip(...)`
  closures remain caller-owned. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  tooltip_body_text_uses_compact_paragraph_role --no-fail-fast`.
- 2026-05-16: routed IMUI tab triggers and menubar triggers through the shared
  `text_button_label(...)` role. This keeps button-like trigger labels single-line and truncating
  while leaving menu item/selectable row labels out of the button-label role. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  imui::tab_family_controls::tests::tab_trigger_visual_uses_button_label_text_role
  imui::menu_family_controls::tests::menu_trigger_visual_uses_button_label_text_role
  --no-fail-fast`.
- 2026-05-16: introduced `text_list_row_label(...)` as the shared dense list/command-row label
  role and routed IMUI menu items, selectables, and disclosure/tree rows through it. The role uses
  regular `text-sm` styling with fill-width, `min-width: 0`, single-line ellipsis semantics so row
  labels truncate instead of wrapping or increasing row height under resize. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  list_row_label_text_uses_fill_width_single_line_truncation
  menu_item_label_text_uses_shared_list_row_text_role
  selectable_row_label_uses_shared_list_row_text_role
  tree_row_label_uses_shared_list_row_text_role --no-fail-fast`.
- 2026-05-16: routed IMUI menu shortcut labels through the existing `text_control_readout(...)`
  role as muted compact auxiliary readouts. This keeps shortcut text single-line, shrinkable, and
  ellipsis-truncated without adding a menu-specific shortcut role. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib menu_item_shortcut_text_uses_shared_control_readout_role
  menu_item_label_text_uses_shared_list_row_text_role --no-fail-fast`.
- 2026-05-17: routed IMUI menu checkbox/radio indicators and submenu chevrons through the shared
  `text_chrome_glyph(...)` role instead of bare `cx.text(...)`. Menu glyph chrome now follows the
  same single-line clip contract as disclosure indicators, and the source gate rejects the old
  indicator text paths. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  menu_item_indicator_text_uses_shared_chrome_glyph_role --no-fail-fast`.
- 2026-05-17: introduced `text_section_chrome_label(...)` as the shared compact section/chrome
  label role and routed IMUI `separator_text` labels through it. Separator labels no longer carry
  local `TextProps` policy or default word wrapping; they stay single-line, shrinkable, and
  ellipsis-truncated under resize. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  section_chrome_label_text_uses_single_line_truncation --no-fail-fast`.
- 2026-05-17: introduced `text_chrome_title(...)` as the shared medium, fill-width chrome title
  role and routed floating window title-bar text through shared chrome text helpers. Resizable
  floating titles keep fill, grow, shrink, `min-width: 0`, and ellipsis behavior; non-resizable
  titles reuse `text_section_chrome_label(...)` instead of local `TextProps`. Gate: `cargo nextest
  run -p fret-ui-kit --features imui --lib
  chrome_title_text_fills_width_without_main_axis_growth
  section_chrome_label_text_uses_single_line_truncation --no-fail-fast`.
- 2026-05-17: introduced `text_chrome_glyph(...)` as the shared compact fixed-slot chrome glyph
  role and routed disclosure/tree indicators through it. Indicator glyphs now stay single-line and
  clipped inside fixed chrome slots without owning local `TextProps` policy in disclosure controls.
  Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  chrome_glyph_text_uses_fixed_slot_single_line_clip
  disclosure_indicator_uses_shared_chrome_glyph_text_role --no-fail-fast`.
- 2026-05-17: routed `list_from_strings(...)` row text through the shared text-role vocabulary.
  Leading list glyphs now use `text_chrome_glyph(...)`, row labels use
  `text_list_row_label(...)`, and trailing shortcut/readout text uses
  `text_control_readout(...)`. This closes a generic fixed-row compatibility-helper path without
  adding an IMUI ListBox API or moving policy into `fret-imui`. Gates: `cargo nextest run -p
  fret-ui-kit --features imui --lib list_from_strings_uses_shared_single_line_text_roles
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: routed the default retained tree row renderer through the shared text-role
  vocabulary too. Tree labels now use `text_list_row_label(...)` and toggle glyphs use
  `text_chrome_glyph(...)` instead of the local `crate::ui::text(...).truncate()` / bare
  `cx.text(...)` path. Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  default_tree_row_label_uses_shared_list_row_text_role tree_toggle_glyph_uses_shared_chrome_glyph_text_role
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: routed floating-window close button glyphs through the same
  `text_chrome_glyph(...)` role via `floating_window_close_glyph_text(...)`. This keeps fixed
  title-bar action chrome on the shared single-line clip contract instead of bare `cx.text(...)`
  default wrapping semantics. Gate: `cargo nextest run -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role --no-fail-fast`.
- 2026-05-17: introduced `text_control_label(...)` as the shared compact control-label text role
  and routed `control_chrome::fill_text(...)` through it. Checkbox/radio/switch labels plus
  combo/slider captions keep their fill, grow, shrink, `min-width: 0`, and ellipsis behavior
  without owning local `TextProps` policy inside IMUI chrome. Gate: `cargo nextest run -p
  fret-ui-kit --features imui --lib control_label_text_uses_fill_width_single_line_truncation
  imui_fill_text_is_single_line_and_shrinkable imui_control_text_uses_shared_button_label_role
  --no-fail-fast`.
- 2026-05-18: hardened `tools/gate_imui_workstream_source.py` so direct `TextProps`
  construction under `fret-ui-kit::imui` fails the source gate unless it is moved into shared text
  roles with explicit evidence. The check now covers both `TextProps::new(...)` and
  `TextProps { ... }` struct literals, matching the editor allowlist scanner and closing the
  remaining source-gate bypass for local IMUI text policy.
- 2026-05-17: removed the last IMUI direct text constructor exception by routing
  `UiWriterImUiFacadeExt::text(...)` through the shared `text_section_chrome_label(...)` role.
  Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  imui_text_item_is_single_line_and_shrinkable imui_text_wrapped_is_explicit_wrapping_text
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_input_value_text(...)` in `fret-ui-editor` input-group primitives
  and routed drag-value plus axis-drag-value scrub readouts through it. The helper keeps editor
  numeric value text fill-width, `min-width: 0`, shrinkable, single-line, and ellipsis-truncated
  while preserving editor-specific density and chrome policy outside `fret-imui`. Gates: `cargo
  nextest run -p fret-ui-editor editor_input_value_text_is_single_line_and_shrinkable --no-fail-fast`
  and `cargo nextest run -p fret-ui-editor drag_value axis_drag_value --no-fail-fast`.
- 2026-05-19: moved the remaining input-group text props constructors out of
  `primitives/input_group.rs` and into `primitives/readout.rs` as
  `editor_input_segment_text_props(...)`, `editor_input_value_text_props(...)`, and
  `editor_axis_marker_text_props(...)`. `input_group` now consumes editor text roles instead of
  owning local `TextProps` policy, and the direct editor `TextProps` allowlist no longer includes
  `input_group.rs`. Gates: `cargo nextest run -p fret-ui-editor
  editor_input_segment_text_keeps_fixed_segment_line_box
  editor_input_value_text_props_are_single_line_and_shrinkable
  editor_axis_marker_text_keeps_fixed_centered_line_box
  editor_input_value_text_is_single_line_and_shrinkable --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: moved `FieldStatusBadge` label text policy into
  `editor_status_badge_text_props(...)` in the editor readout primitive layer. The control keeps its
  compact centered badge label, single-line ellipsis, and palette behavior without owning local
  `TextProps` policy. Gate: `cargo nextest run -p fret-ui-editor
  editor_status_badge_text_uses_compact_single_line_readout_role
  error_badge_palette_keeps_short_visible_label loading_badge_palette_uses_short_label
  loading_badge_palette_stays_darker_than_editor_foreground --no-fail-fast`.
- 2026-05-17: introduced `editor_inline_error_text_props(...)` for compact single-line editor error
  readouts and routed both `ColorEdit` root errors and popup numeric errors through it. The role is
  destructive-color aware through caller-supplied color, fill-width, `min-width: 0`, and ellipsis
  truncated; wrapping validation prose remains a separate explicit control policy. Gate: `cargo
  nextest run -p fret-ui-editor editor_inline_error_text_is_single_line_and_shrinkable
  editor_preview_caption_text_is_single_line_and_shrinkable
  editor_tooltip_readout_text_is_single_line_and_shrinkable
  numeric_readout_formats_rgb_hsv_and_optional_alpha
  color_tooltip_lines_match_imgui_hex_rgb_hsv_preview_text --no-fail-fast`.
- 2026-05-17: introduced `editor_validation_message_text_props(...)` for editor validation prose
  that is allowed to wrap and grow height. `NumericInput` inline validation messages now use that
  explicit role instead of local `TextProps`, while `tools/gate_imui_workstream_source.py` freezes
  direct `TextProps` construction under `fret-ui-editor/src` to the editor primitive owners
  (`input_group`, `popup_list`, and `readout`). Gates: `cargo nextest run -p fret-ui-editor
  editor_validation_message_text_wraps_and_shrinks --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_preview_caption_text_props(...)` and
  `editor_tooltip_readout_text_props(...)` for color side-preview captions and color tooltip
  numeric lines. These stay in the editor readout primitive layer instead of being folded into
  popup-list rows, preserving the semantic distinction while removing local `TextProps` policy from
  `color_edit/popup/preview.rs` and `color_edit/popup/tooltip.rs`.
- 2026-05-17: introduced shared transform-label text helpers in
  `ecosystem/fret-ui-editor/src/primitives/readout.rs` and routed `TransformEdit` section badges,
  section headings, and inline link/uniform checkbox labels through them. The control no longer
  owns local `TextProps` literals for compact labels, and those labels now consistently stay
  single-line, `min-width: 0` where they need to shrink, and ellipsis/clip constrained under
  resize. Gate: `cargo nextest run -p fret-ui-editor
  editor_section_badge_text_is_single_line_centered_badge_label
  editor_section_heading_text_is_single_line_and_shrinkable
  editor_inline_control_label_text_is_single_line_and_shrinkable
  transform_edit_axis_outcome_exposes_read_only_signals --no-fail-fast`.
- 2026-05-17: introduced `editor_popup_list_row_text_props(...)` and
  `editor_popup_empty_text_props(...)` in `ecosystem/fret-ui-editor/src/primitives/popup_list.rs`.
  `EnumSelect` trigger/row/empty text and `TextAssistField` row/empty text now use shared editor
  helpers instead of local `TextProps` or `TextProps::new(...)`, closing another default
  word-wrap path under resize. Color-edit copy menu rows and popup option captions now reuse the
  same popup-list family through start-aligned, centered-row, and fixed-caption variants while
  leaving preview labels and tooltip lines as separate semantics. Gate: `cargo nextest run -p
  fret-ui-editor
  popup_list_row_text_is_single_line_and_shrinkable
  popup_empty_text_is_single_line_and_shrinkable
  popup_list_centered_row_text_keeps_row_fill_and_center_alignment
  popup_list_option_caption_text_keeps_fixed_caption_line_box
  enum_select_item_test_id_segment_is_stable_ascii empty_label_is_inline_only
  color_copy_entries_match_imgui_copy_as_payloads
  popup_options_default_to_imgui_like_hue_bar_surface --no-fail-fast`.
- 2026-05-19: centralized the popup-list text props family into
  `ecosystem/fret-ui-editor/src/primitives/readout.rs`. `popup_list.rs` now owns only popup-list
  geometry, row state, and palette helpers; popup row, empty-state, centered-row, and fixed-caption
  text roles are part of the shared editor text role owner. The source gate now removes
  `popup_list.rs` from the direct editor `TextProps` allowlist and forbids `TextProps`,
  `TextWrap`, `TextOverflow`, and typography policy from returning there. Gate:
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_property_group_header_text_props(...)`,
  introduced `editor_property_row_reset_glyph_text_props(...)`. This slice also
  introduced `editor_inspector_panel_title_text_props(...)` in
  `ecosystem/fret-ui-editor/src/primitives/readout.rs`, then routed `PropertyGroup` header labels,
  `PropertyRow` reset glyphs, and `InspectorPanel` titles through those shared roles. This removes
  local inspector chrome/default text policy from the composites while keeping fixed-row text
  single-line, shrinkable where needed, and line-height constrained under resize. The
  `InspectorPanel` layout gate renders a narrow header with toolbar siblings and proves the title
  remains one measured line. Gate:
  `cargo nextest run -p fret-ui-editor
  editor_property_group_header_text_is_single_line_and_shrinkable
  editor_property_row_reset_glyph_text_keeps_fixed_button_line_box
  editor_inspector_panel_title_text_is_single_line_and_shrinkable
  inspector_panel_title_stays_single_line_when_header_is_narrow --no-fail-fast`.
- 2026-05-17: introduced `editor_property_row_label_text_props(...)` and the property-grid row
  context `label_text(...)` convenience path for fixed inspector label chrome. `PropertyRow` label
  slots now clamp their own line box to the editor row height, so accidental bare/default label
  text cannot wrap and grow a fixed-height row under resize. `GradientEditor`, eager
  `PropertyGrid`, and virtualized grid smoke usage now route first-party labels through the helper,
  while arbitrary custom label elements remain possible for fully custom rows. Gate:
  `cargo nextest run -p fret-ui-editor
  editor_property_row_label_text_is_single_line_and_shrinkable
  row_label_slot_keeps_fixed_line_box_when_label_text_wraps_under_narrow_layout --no-fail-fast`.
- 2026-05-17: migrated the `imui_editor_proof_demo` property-grid labels to
  `row_cx.label_text(...)` so the selected editor-grade proof teaches the fixed-label role instead
  of relying on `PropertyRow`'s container fallback for bare `cx.text(...)`. The source gate now
  requires representative proof labels (`Name`, `Typed numeric`, `Blend slider`, `Transform`) to
  use `label_text(...)` and forbids those labels from returning to `|cx| cx.text(...)` in property
  label slots. Gates: `python tools/gate_imui_workstream_source.py` and
  `cargo check -p fret-demo --bin imui_editor_proof_demo`.
- 2026-05-19: extended the `imui_editor_proof_demo` text-role proof from property rows/readouts to
  the main IMUI proof chrome. The demo now uses `proof_imui_section_text(...)`,
  `proof_imui_readout_text(...)`, and `proof_imui_compact_paragraph_text(...)` backed by shared
  `text_section_chrome_label(...)`, `text_control_readout(...)`, and
  `text_compact_paragraph(...)` instead of `fret_ui_kit::ui::text(...).text_xs()` /
  `.font_semibold()` local styling. The focused source test and workstream source gate reject the
  old headline, hint, parity intro/state hint, and editor label paths. Gates:
  `cargo nextest run -p fret-examples --test imui_editor_proof_text_roles_surface
  imui_editor_proof_main_fixed_text_uses_shared_roles --no-fail-fast`,
  `cargo check -p fret-demo --bin imui_editor_proof_demo`, and
  `python tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out while it contended with a parallel
  `cargo check` for Cargo package-cache locks; after the background Cargo/Rustc processes exited,
  the same focused nextest command passed. `cargo fmt --check -p fret-examples` and
  `python -m py_compile tools\gate_imui_workstream_source.py` also passed before the final gate
  run. `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast`,
  `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`, and
  `git diff --check` passed.
- 2026-05-17: migrated the `workspace_shell_demo` editor rail to teach the shared text roles too.
  Rail command buttons now use `text_button_label(...)`, property labels use `row_cx.label_text(...)`,
  and compact property values route through `text_control_readout(...)` via a local proof helper.
  Gates: `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface
  --no-fail-fast`, `cargo check -p fret-demo --bin workspace_shell_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: migrated the remaining `workspace_shell_demo` editor-rail header copy to a local
  `workspace_shell_paragraph_text(...)` helper backed by shared `text_paragraph(...)` instead of
  app-local `fret_ui_kit::ui::text(...).text_sm().text_color(...)` styling. This keeps the shell
  proof's explanatory text in the shared paragraph role while leaving compact rail values on
  `workspace_shell_readout_text(...)`. Gates: `cargo nextest run -p fret-examples --test
  workspace_shell_editor_rail_surface
  workspace_shell_demo_composes_editor_rail_through_workspace_frame_slots --no-fail-fast`,
  `cargo check -p fret-demo --bin workspace_shell_demo`, `cargo nextest run -p fret-examples
  --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface
  --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out during background compile; after
  Cargo/Rustc exited, the same focused nextest command passed. `cargo fmt --check -p
  fret-examples` also passed.
- 2026-05-17: migrated the `editor_notes_demo` inspector metadata surface to the same resize-safe
  text roles. Property-grid labels now use `row_cx.label_text(...)`, inspector subtitle and compact
  committed/outcome/draft/summary status values route through a local
  `editor_notes_readout_text(...)` helper backed by `text_control_readout(...)`, and the
  `editor_notes_editor_rail_surface` test plus the IMUI workstream source gate reject those fixed
  property-row labels/readouts drifting back to bare `cx.text(...)`. Gates:
  `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --no-fail-fast`,
  `cargo check -p fret-demo --bin editor_notes_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the `editor_notes_demo` proof text-role migration to the center preview and
  collection summary surfaces. Collection summary/status values now use
  `editor_notes_readout_text(...)`, center section labels use `editor_notes_section_text(...)`, and
  app-owned explanatory/preview text uses `editor_notes_paragraph_text(...)` instead of local
  `ui::text(...).wrap(...)` styling. The inspector `PropertyGrid` and `TextField` behavior remain
  unchanged. Gates: `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface
  editor_notes_demo_composes_shell_mounted_rails_through_workspace_frame_slots --no-fail-fast`,
  `cargo check -p fret-demo --bin editor_notes_demo`, and `python
  tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out while a background Cargo/Rustc
  compile continued; after Cargo/Rustc exited, the same focused nextest command passed. `cargo
  check -p fret-demo --bin editor_notes_demo`, `python tools/gate_imui_workstream_source.py`,
  `cargo fmt --check -p fret-examples`, `python -m json.tool
  docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json > $null`, and `git diff --check`
  passed.
- 2026-05-19: migrated the `editor_notes_device_shell_demo` compact mobile header to the same
  shared text-role vocabulary. The mobile title now uses `device_shell_section_text(...)` backed by
  `text_section_chrome_label(...)`, and the explanatory header copy uses
  `device_shell_paragraph_text(...)` backed by `text_paragraph(...)` instead of local
  `ui::text(...).text_base().font_semibold()` / `ui::text(...).text_sm().text_color(...).wrap(...)`
  styling. Gates: `cargo nextest run -p fret-examples --test editor_notes_device_shell_surface
  editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content
  --no-fail-fast`, `cargo check -p fret-demo --bin editor_notes_device_shell_demo`, `cargo nextest
  run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast`, and
  `python tools/gate_imui_workstream_source.py`.
  Verification note: the first focused nextest attempt timed out during background compile; after
  Cargo/Rustc exited, the same focused nextest command passed. `cargo fmt --check -p
  fret-examples` also passed.
- 2026-05-17: migrated the `workspace_shell_demo` dirty-close prompt title/details to the same
  role vocabulary. The title now uses `workspace_shell_section_chrome_label(...)` backed by
  `text_section_chrome_label(...)`, while reason/dirty-detail lines use
  `workspace_shell_readout_text(...)` backed by `text_control_readout(...)`. Gates:
  `cargo nextest run -p fret-examples --test workspace_shell_editor_rail_surface --no-fail-fast`,
  `cargo check -p fret-demo --bin workspace_shell_demo`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: migrated selected `imui_editor_proof_demo` proof text to shared roles without
  widening framework API. Material/advanced/global `No matches` labels now use
  `proof_empty_state_text(...)`, authoring shared-state lines use `proof_compact_readout_element(...)`
  backed by `text_control_readout(...)`, and the declarative gradient section label uses
  `proof_section_chrome_label(...)` backed by `text_section_chrome_label(...)`. Gates:
  `python tools/gate_imui_workstream_source.py` and
  `cargo check -p fret-demo --bin imui_editor_proof_demo`.
- 2026-05-18: migrated the `imui_editor_proof_demo` collection proof fixed text to the same shared
  role vocabulary without adding shared collection policy. Collection title text uses a
  proof-local section-chrome helper, compact state/status/readout lines use a proof-local helper
  backed by `text_control_readout(...)`, and inline rename explanatory copy opts into
  `text_wrapped(...)`. Focused source tests and the IMUI source gate reject those fixed collection
  paths drifting back to bare `ui.text(...)`.
- 2026-05-18: moved the UI Gallery app-facing DataTable snippets onto the shared table-cell text
  role. `apps/fret-ui-gallery/src/ui/snippets/data_table/mod.rs` now owns a narrow
  `table_cell_text(...)` helper over `text_table_cell(...)`, and the default/basic/guide/RTL/reusable
  snippets use it for fixed body cells plus fallback cells instead of bare `cx.text(...)`. Amount
  columns remain on the existing tabular numeric formatting path pending a separate numeric-cell
  role. Gates: `cargo nextest run -p fret-ui-gallery --test data_table_docs_surface
  data_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: removed empty text nodes from copyable AI AudioPlayer state markers. Local and remote
  snippets now use a `state_marker(...)` helper that mounts a zero-size `SpacerProps` child under
  generic semantics with the same diagnostics `test_id`s. This keeps non-visible test anchors out
  of text layout semantics. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_audio_player_text_role_surface audio_player_state_markers_use_non_text_spacers
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: migrated visible fixed text in copyable AI Message and Terminal snippets onto the
  shared text-role vocabulary. Message uses `text_control_readout(...)` for the compact action
  status, `text_section_chrome_label(...)` for the fixed demo title, and `text_paragraph(...)` for
  user-message prose. Terminal uses section-chrome title text, paragraph explanatory copy, and the
  non-text spacer-marker pattern for its empty-output diagnostics marker. Gates: `cargo nextest run
  -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: moved the actual `fret-ui-ai` `TerminalTitle` default label from local
  `ui::raw_text(...).wrap(None).overflow(Clip)` policy to the shared `text_chrome_title(...)`
  role. The component now keeps terminal chrome titles fill-width, `min-width: 0`, grow/shrink
  enabled, single-line, and ellipsized under narrow resize. Gate: `cargo nextest run -p
  fret-ui-ai terminal_title_label_uses_chrome_title_text_role --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: tightened `text_chrome_title(...)` to own the medium-weight chrome-title contract as
  well as fill/grow/shrink/min-width-zero/ellipsis layout, then routed the real `fret-ui-ai`
  `EnvironmentVariablesTitle` default/text paths through that shared role instead of local
  `ui::raw_text(...).wrap(None).overflow(Clip)` typography policy. Custom title children still use
  the component-owned inherited title refinement because the upstream surface is children-first.
  Gates: `cargo nextest run -p fret-ui-kit --lib
  chrome_title_text_fills_width_without_main_axis_growth
  section_chrome_label_text_uses_single_line_truncation --no-fail-fast`, `cargo nextest run -p
  fret-ui-ai environment_variables_title_text_uses_chrome_title_text_role
  environment_variables_title_children_scope_inherited_typography
  terminal_title_label_uses_chrome_title_text_role --no-fail-fast`, and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: routed `fret-ui-ai` `EnvironmentVariableName` and non-selectable
  `EnvironmentVariableValue` text through `text_code_label(...)` with component-owned foreground
  inheritance instead of local monospace `TextProps`. Revealed values intentionally stay on
  `SelectableTextProps` because that path owns text selection, not ordinary fixed chrome text. The
  environment-variable empty/custom-child marker paths now use the crate-local non-text spacer
  placeholder instead of empty `Text` nodes. Gates: `cargo nextest run -p fret-ui-ai
  environment_variable_name_and_masked_value_use_code_label_text_role
  environment_variable_value_is_selectable_only_when_shown
  environment_variable_copy_button_supports_custom_children --no-fail-fast`, `cargo check -p
  fret-ui-ai`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: added the shared `text_code_label_emphasis(...)` and
  `text_compact_paragraph_inherited(...)` text-role derivatives, then routed real `fret-ui-ai`
  `PackageInfo` defaults through them. Package names and target versions now use emphasized
  code-label text, current versions and dependency rows use regular code-label text, the
  Dependencies heading uses section-chrome text, and `PackageInfoDescription` keeps the shared
  wrapping/fill-width paragraph layout without overriding the component-owned description
  typography scope. Gates: `cargo nextest run -p fret-ui-kit --features imui --lib
  prose_variants_and_code_wrap_install_semantic_inherited_overrides
  compact_paragraph_text_uses_wrapping_fill_width_layout
  inherited_compact_paragraph_keeps_wrapping_layout_without_leaf_refinement --no-fail-fast`,
  `cargo nextest run -p fret-ui-ai package_info_default_identifier_text_uses_code_label_role
  package_info_description_scopes_inherited_description_typography
  package_info_root_children_support_docs_shaped_compound_parts --no-fail-fast`,
  `cargo check -p fret-ui-ai`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the same visible text-role migration to AI Artifact, CodeBlock, and Sandbox
  snippets. Fixed demo titles and panel labels now use section-chrome text, explanatory copy uses
  paragraph text, Artifact's closed state uses a compact control readout, and CodeBlock's
  active-language diagnostics marker now uses a zero-size generic spacer marker rather than an
  invisible empty `Text`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Queue copyable snippet. The fixed
  demo title now uses `text_section_chrome_label(...)`, explanatory copy uses
  `text_paragraph(...)`, and the action-revision diagnostics anchor uses a generic zero-size
  spacer marker instead of an empty/invisible `Text` node. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Checkpoint copyable snippet.
  Conversation and explanatory copy now use `text_paragraph(...)`, restore status uses
  `text_control_readout(...)`, the checkpoint trigger uses `text_button_label(...)`, and custom
  checkpoint icon symbols use `text_chrome_glyph(...)`. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to simple chrome/prose snippets:
  Agent, CodeBlock usage, Environment Variables, and OpenIn. Each now routes fixed demo titles
  through `text_section_chrome_label(...)` and explanatory body copy through `text_paragraph(...)`
  instead of default bare `cx.text(...)`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to selector/branch state markers.
  MessageBranch, MicSelector, and ModelSelector now use generic zero-size `SpacerProps` markers for
  diagnostics-only state anchors instead of empty `Text`, and their fixed demo title/body copy uses
  shared section-chrome/paragraph roles. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to CommitLarge, Plan,
  PromptInputActionMenu, and PromptInputTooltip. CommitLarge now uses a generic zero-size
  `SpacerProps` marker for its opened-file diagnostics anchor instead of an empty `Text`, and the
  four snippets route their outer fixed demo title/body copy through shared
  section-chrome/paragraph roles. Inner Plan prose/Button composition stays out of this slice until
  a separate semantics pass. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to PlanContent internals. Plan section
  headings now use `text_section_chrome_label(...)`, the overview body uses `text_paragraph(...)`,
  bullet rows use `text_list_row_label(...)`, and the custom Build button child uses
  `text_button_label(...)` instead of local `ui::text(...)` styling. Plan open/streaming behavior
  and `fret-ui-ai` Plan component ownership were intentionally left unchanged. Gates: `cargo
  nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_and_plan_snippets_use_shared_outer_text_roles --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to large/status snippets:
  StackTraceLarge, TestResultsLarge, Tool, and Suggestions. StackTraceLarge/TestResultsLarge
  opened/activated markers and Tool/Suggestions test markers now use generic zero-size
  `SpacerProps` anchors instead of empty `Text`; fixed outer title/body copy routes through shared
  section-chrome/paragraph roles, and Tool's fixed state-section labels use section-chrome text.
  Suggestions custom-children content stays app-owned for a later custom-content semantics pass.
  Gates: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to QueuePromptInput and Transcription.
  QueuePromptInput now uses a generic zero-size `SpacerProps` anchor for the sent-count diagnostics
  marker, routes the custom Search button child through `text_button_label(...)`, and keeps fixed
  outer title/body copy on shared section-chrome/paragraph roles. Transcription now uses generic
  zero-size spacer markers for time/active diagnostics anchors while routing fixed title/body copy
  through shared roles. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to WebPreview. State diagnostics markers
  now use generic zero-size `SpacerProps` anchors instead of empty `Text`, navigation arrow/reload
  glyphs use `text_chrome_glyph(...)`, and composable child fixed body/footer copy routes through
  shared section-chrome/paragraph roles. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Chat's outer surface. The
  prompt-nonempty diagnostics marker now uses a generic zero-size spacer, empty marker fallbacks use
  spacers instead of empty `Text`, fixed header instructions route through paragraph roles, and the
  exported markdown length readout uses `text_control_readout(...)`. Chat message body rendering
  remains app/content-owned for a separate semantics pass. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to PromptInputProvider and PromptInput
  docs. Provider sent-count diagnostics now use a generic zero-size `SpacerProps` anchor, the
  custom external-add label uses `text_button_label(...)`, and fixed outer title/body copy uses
  shared roles. PromptInput docs now routes the custom Search label through button-label text and
  fixed outer title/body copy through section-chrome/paragraph roles. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to Cursor-style PromptInput custom text.
  Command item titles now use `text_list_row_label(...)`, filenames/paths use
  `text_code_label(...)`, rules/tabs hover-card readouts use `text_control_readout(...)`, and
  custom trigger counts use `text_button_label(...)` instead of local `ui::text(...)` styling.
  PromptInput placeholders, command headings, and component-owned labels were intentionally left to
  `fret-ui-ai` component policy. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface ai_prompt_input_cursor_custom_text_uses_shared_roles
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to Shimmer demo chrome. Typography,
  duration, and elements demo labels now use `text_control_readout(...)`; the inline non-shimmer
  prefix/suffix in the elements demo uses section-chrome/control-readout roles instead of bare
  `ui::text(...)`. `Shimmer::new(...)` calls intentionally remain because Shimmer itself is the
  explicit animated text capability surface. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface ai_shimmer_demo_chrome_text_uses_shared_roles --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Reasoning, StackTrace, and
  VoiceSelector fixed chrome/readouts. Fixed outer title/body copy uses shared
  section-chrome/paragraph roles; StackTrace status and VoiceSelector selected/open diagnostics use
  `text_control_readout(...)` instead of default wrapping text. Content renderers remain owned by
  their respective AI element surfaces. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Confirmation content snippets.
  Confirmation request prose now uses `text_paragraph(...)`, inline/code payloads use
  `text_code_wrap(...)`, approval/rejection result text uses `text_control_readout(...)`, and the
  demo's fixed outer title/body copy uses shared section-chrome/paragraph roles. Confirmation state
  and button policy stay in the AI element/recipe layer. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Task content. Task item labels now
  use `text_list_row_label(...)`, attached file names use `text_code_wrap(...)`, and the demo's
  fixed outer title/body copy uses shared section-chrome/paragraph roles. Gates: `cargo nextest run
  -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Conversation instrumentation and
  custom scroll-button text. Export/message-count diagnostics now use generic semantics with
  numeric values instead of text semantics, while the custom `Latest` scroll-bottom label uses
  `text_button_label(...)`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to fixed usage snippet chrome/prose.
  Attachments usage explanatory copy now uses `text_paragraph(...)`, and StackTrace usage fixed
  title/body copy uses `text_section_chrome_label(...)` / `text_paragraph(...)`. Gates: `cargo
  nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to Message usage. User message text now
  uses `text_paragraph(...)`, the last-action marker uses `text_control_readout(...)`, and fixed
  outer title/body copy uses `text_section_chrome_label(...)` / `text_paragraph(...)`; assistant
  markdown response rendering remains owned by `MessageResponse`. Gates: `cargo nextest run -p
  fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Canvas world spike. Visible
  chrome, node copy, and debug/status readouts now use `text_section_chrome_label(...)`,
  `text_paragraph(...)`, and `text_control_readout(...)` instead of bare `cx.text(...)`. Canvas
  pan/zoom, bounds, drag/drop, and connection behavior were intentionally left unchanged. Gates:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to the Image demo. The fixed explanatory
  copy now uses `text_paragraph(...)`, and the image-ready/loading readouts use
  `text_control_readout(...)` instead of bare `cx.text(...)`. Image asset lookup and presentation
  behavior were intentionally left unchanged. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-18: extended the AI visible text-role migration to PromptInput referenced sources. The
  fixed title/body copy now uses `text_section_chrome_label(...)` and `text_paragraph(...)` instead
  of bare `cx.text(...)`; source chip state and PromptInput model behavior were intentionally left
  unchanged. Gates: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` and `python tools/gate_imui_workstream_source.py`.
- 2026-05-19: extended the AI visible text-role migration to Attachments inline hover-card details.
  App-owned attachment labels now use `text_list_row_label(...)`, and media-type values use
  `text_control_readout(...)` instead of default `ui::text(...)` builders; attachment chip,
  remove affordance, and hover-card behavior were intentionally left unchanged. Gates: `cargo
  nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_attachments_inline_hover_card_uses_shared_text_roles --no-fail-fast` and `python
  tools/gate_imui_workstream_source.py`.
- 2026-05-18: migrated the AI Artifact docs code-display status marker from an invisible
  `cx.text(...)` node to a generic zero-size semantics marker with a diagnostic label. This keeps
  the existing `ui-gallery-ai-artifact-docs-run-action` `label_contains` script contract while
  removing the hidden text from text layout. Gates: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast`, `python tools/gate_imui_workstream_source.py`, and
  `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-artifact-docs-run-action.json`.
- 2026-05-18: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` passed after the large/status snippet slice landed. `python
  tools/gate_imui_workstream_source.py`, `python -m py_compile
  tools/gate_imui_workstream_source.py`, `python -m json.tool
  docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`, and `git diff --check` also passed.
- 2026-05-18: `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  --no-fail-fast` passed after the Artifact/CodeBlock/Sandbox slice landed. `python
  tools/gate_imui_workstream_source.py`, `python -m json.tool
  docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json`, and `git diff --check` also passed.
- 2026-05-17: migrated `imui_editor_proof_demo` drag-preview cards away from a newline-joined bare
  text blob. Preview titles now use `text_section_chrome_label(...)`, optional subtitles use
  `text_control_readout(...)`, and `proof_drag_preview_card_uses_single_line_text_roles` locks the
  two-line preview as two single-line role elements. Gates: `cargo nextest run -p fret-examples
  --lib proof_drag_preview_card_uses_single_line_text_roles --no-fail-fast`, `cargo check -p
  fret-demo --bin imui_editor_proof_demo`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: introduced `editor_empty_state_text_props(...)` for compact editor empty-state
  labels and routed `GradientEditor`'s `No stops` label through
  `gradient_editor_empty_state_text(...)`. This removes another real component bare-text path while
  keeping the empty-state role in `fret-ui-editor`, not `fret-imui`. Gates: `cargo nextest run -p
  fret-ui-editor editor_empty_state_text_is_single_line_and_shrinkable
  gradient_editor_empty_state_text_is_single_line_and_shrinkable --no-fail-fast` and
  `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: removed clipping from `PropertyRow` value slots while keeping fixed label/reset/action
  chrome clipped. This is the layout-container side of the text-role contract: wrapping validation
  prose such as `NumericInput` inline errors may grow to multiple lines, so the parent value slot
  must not clip the measured line box under resize. Gates:
  `cargo nextest run -p fret-ui-editor row_value_slot_keeps_overflow_visible_for_wrapping_value_children row_value_slot_grows_to_wrapping_value_text_under_narrow_layout --no-fail-fast`
  and `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: added a layout-level `PropertyRow` resize regression for wrapping validation text.
  The test renders a narrow row with `editor_validation_message_text_props(...)`, runs the real
  `UiTree::layout_all(...)` path through public element-bounds queries, and asserts that the
  measured multi-line text bottom stays inside the value slot and row bounds. This closes the
  evidence gap between the structural overflow contract and the visual resize bug report.
- 2026-05-17: added a composition-level `PropertyGrid` resize regression for the same wrapping
  validation role. The test renders mixed single-line and wrapping rows in a narrow grid, reuses the
  same measured wrapping text service, and asserts that the wrapping row grows, pushes the
  following row down, and remains contained by the grid. Gate:
  `cargo nextest run -p fret-ui-editor property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout --no-fail-fast`.
- 2026-05-17: added `P3_TEXT_ROLE_MATRIX_2026-05-17.md` as the resize triage contract for the
  current text-role work. It names the stable base roles (`text_control_readout(...)`,
  `text_button_label(...)`, `text_paragraph(...)` / `text_paragraph_break_words(...)`,
  `text_code_block(...)` / `text_code_wrap(...)`, and `text_table_cell(...)`), maps current derived
  roles, requires wrapping paragraph/validation copy to have parent layout that accounts for
  multi-line height, and keeps `fret-imui` policy-light by rejecting a public `TextRole` enum until
  two consumers need a data-driven role value. Gate: `python tools/gate_imui_workstream_source.py`.
- 2026-05-17: UI Gallery's disabled toaster driver path now returns a spacer placeholder instead
  of an empty text node. This keeps app-shell placeholder plumbing out of the text layout/measure
  contract and prevents future resize fixes from treating invisible placeholders as text content.
  Gate: `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  gallery_driver_disabled_toaster_does_not_emit_empty_text --no-fail-fast`.
- 2026-05-17: the UI Gallery app-sidebar snippet collapsed-projects path now returns a spacer
  placeholder instead of an empty text node. This keeps the copyable sidebar recipe from teaching
  empty text as layout placeholder plumbing. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_default_app sidebar_app_collapsed_projects_do_not_emit_empty_text
  --no-fail-fast`.
- 2026-05-17: `fret-ui-ai` now owns a crate-local `empty_placeholder(...)` helper for hidden or
  missing-content AI element paths. The affected AI surfaces return spacer placeholders instead of
  empty text nodes, keeping optional AI chrome out of text layout semantics without widening
  `fret-imui`. Gate: `cargo nextest run -p fret-ui-ai
  hidden_ai_element_paths_use_non_text_placeholder --no-fail-fast`.
- 2026-05-17: UI Gallery's retained-table torture page now routes fixed row/cell text through a
  local helper backed by `fret-ui-kit::declarative::text::text_table_cell(...)`, and table state
  readouts through `doc_layout::control_readout_text(...)`. The page keeps explicit prose/diagnostic
  description copy separate, while the fixed 28px table rows no longer use bare/default text. Gate:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_table_retained_torture_uses_structured_table_debug_ids --no-fail-fast`.
- 2026-05-17: UI Gallery's DataTable torture page now follows the same role split in both retained
  and non-retained render paths. Fixed cells route through a helper backed by
  `text_table_cell(...)`, while sorting/filter/pinning status lines use
  `doc_layout::control_readout_text(...)`. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_data_table_torture_exposes_header_row_anchor
  --no-fail-fast`.
- 2026-05-17: UI Gallery's DataGrid preview now routes virtualized grid cell text through a helper
  backed by `text_table_cell(...)`, and the selected-row status line through
  `doc_layout::control_readout_text(...)`. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles
  --no-fail-fast`.
- 2026-05-17: UI Gallery's DataGrid/DataTable/Tree Torture explanatory header copy now routes
  through `doc_layout::paragraph_text(...)`, backed by shared `text_paragraph(...)`, instead of
  default `cx.text(...)`. Gates: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles
  gallery_data_table_torture_exposes_header_row_anchor
  gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast`.
- 2026-05-17: UI Gallery's Inspector Torture preview now routes fixed virtual-row property labels
  through a helper backed by `text_list_row_label(...)`, and fixed row values through
  `doc_layout::control_readout_text(...)`. Gate: `cargo nextest run -p fret-ui-gallery --test
  ui_authoring_surface_internal_previews gallery_inspector_torture_uses_fixed_row_text_roles
  --no-fail-fast`.
- 2026-05-17: UI Gallery's virtual-list torture harness now routes fixed custom row labels through
  helpers backed by `text_list_row_label(...)`, and row detail/editing readouts through
  `doc_layout::control_readout_text(...)`. The UI Kit list torture custom row renderer also routes
  item labels through the shared list-row label role. Gate: `cargo nextest run -p fret-ui-gallery
  --test ui_authoring_surface_internal_previews
  harness_virtual_list_torture_uses_fixed_row_text_roles
  harness_ui_kit_list_torture_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's retained-table, hit-test, UI Kit list, virtual-list, and view-cache
  harness headers now route explanatory copy through `doc_layout::paragraph_text(...)` and
  mode/status lines through `doc_layout::control_readout_text(...)` instead of bare `cx.text(...)`.
  Gates: `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_table_retained_torture_uses_structured_table_debug_ids
  harness_hit_test_torture_uses_header_text_roles
  harness_virtual_list_torture_uses_fixed_row_text_roles
  harness_ui_kit_list_torture_uses_fixed_row_text_roles
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's View Cache torture page now routes cached inner virtual-list row labels
  through a helper backed by `text_list_row_label(...)`. Gate: `cargo nextest run -p
  fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's View Cache torture page now routes fixed switch labels through
  `doc_layout::control_label_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run
  -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's View Cache cached Popover body copy now routes through
  `doc_layout::paragraph_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run -p
  fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_view_cache_uses_fixed_row_text_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's Tree Torture dynamic target status now routes through
  `doc_layout::control_readout_text(...)` instead of local muted/text-sm styling. Gate: `cargo
  nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast`.
- 2026-05-17: UI Gallery's overlay and menu last-action/status flags now route through
  `doc_layout::control_readout_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run
  -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_overlay_status_text_uses_control_readout_roles
  gallery_menus_last_action_uses_control_readout_role --no-fail-fast`.
- 2026-05-17: UI Gallery's overlay dialog/sheet/portal scroll filler rows now route through a
  helper backed by `text_list_row_label(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest
  run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_overlay_scroll_rows_use_list_row_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's HoverCard and Popover body copy now route through
  `doc_layout::paragraph_text(...)` instead of bare `cx.text(...)`. Gate: `cargo nextest run -p
  fret-ui-gallery --test ui_authoring_surface_internal_previews
  gallery_overlay_body_copy_uses_paragraph_roles --no-fail-fast`.
- 2026-05-17: UI Gallery nav title and settings-sheet switch captions now route through the
  driver text-role owner. The nav title uses section-chrome text, and switch captions use
  control-label text instead of local `TextProps` policy. Gate: `cargo nextest run -p
  fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery's chrome torture text-input/textarea labels now route through
  `doc_layout::control_label_text(...)`, backed by shared
  `fret-ui-kit::declarative::text::text_control_label(...)`, instead of bare `cx.text(...)`. Gate:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  page_chrome_torture_uses_control_label_roles --no-fail-fast`.
- 2026-05-17: UI Gallery's status bar now routes metric, inspector-state, and last-action text
  through `driver::text_roles::chrome_readout_text(...)`, backed by
  `fret-ui-kit::declarative::text::text_control_readout(...)`. This keeps fixed status chrome on
  the shared single-line/shrinkable readout role instead of bare/default text under resize. Gate:
  `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery driver chrome now owns `driver::text_roles` as the local adapter over
  shared kit text roles. Disabled tabs/sidebar/content placeholders route through
  `chrome_readout_text(...)`, and settings-sheet section labels route through
  `chrome_section_label(...)`, keeping fixed app-shell chrome on single-line role helpers instead
  of bare/default text. Gate: `cargo nextest run -p fret-ui-gallery --test
  code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery's `BISECT_MINIMAL_ROOT` diagnostic root now routes its placeholder through
  `driver::text_roles::chrome_readout_text(...)` instead of bare `cx.text(...)`. This keeps the
  smallest resize/debug root on the same single-line readout role as the surrounding driver chrome.
  Gate: `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery's fixed-size debug HUD now renders each metric line through
  `driver::text_roles::chrome_readout_text(...)` instead of local word-wrapping `TextProps`. Long
  debug/readout lines now stay single-line and truncate inside the HUD chrome rather than growing
  line boxes under resize. Gate: `cargo nextest run -p fret-ui-gallery --test
  code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-17: UI Gallery shell content/nav text now routes page titles through
  `text_chrome_title(...)`, page origins through `text_control_readout(...)`, and sidebar group
  headings through `text_section_chrome_label(...)`. This removes local `TextProps` policy from
  fixed app-shell chrome without widening `fret-imui`. Gate: `cargo nextest run -p
  fret-ui-gallery --test code_editor_control_readout_surface
  code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast`.
- 2026-05-16: tightened `UiWriterImUiFacadeExt::text(...)` to match Dear ImGui's default
  `Text()` posture: single-line, shrinkable, `min-width: 0`, and ellipsis-truncated under resize.
  Added `UiWriterImUiFacadeExt::text_wrapped(...)` as the explicit wrapping path for explanatory
  prose, and routed first-party editor/workspace proof prose through it. Gates: `cargo nextest run
  -p fret-ui-kit --features imui --lib imui_text_item_is_single_line_and_shrinkable
  imui_text_wrapped_is_explicit_wrapping_text --no-fail-fast` and `cargo check -p fret-examples`.
- 2026-05-17: `UiWriterImUiFacadeExt::text(...)` now delegates to
  `text_section_chrome_label(...)`, removing the former local `TextProps` construction while
  keeping the same single-line resize contract. Gate: `cargo nextest run -p fret-ui-kit
  --features imui --lib imui_text_item_is_single_line_and_shrinkable
  imui_text_wrapped_is_explicit_wrapping_text --no-fail-fast`.
- 2026-05-16: tightened `control_chrome::fill_text(...)`, the shared path for boolean labels,
  combo preview/captions, and slider captions, to fill, shrink, `min-width: 0`, and truncate instead
  of word-wrapping inside compact control chrome. Gates: `cargo nextest run -p fret-ui-kit
  --features imui --lib imui_fill_text_is_single_line_and_shrinkable
  imui_control_text_uses_shared_button_label_role --no-fail-fast`, `cargo nextest run -p
  fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --no-fail-fast`, and
  `cargo nextest run -p fret-ui-kit --features imui --lib
  input_text_model_uses_compact_imui_chrome_without_focus_ring
  textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast`.
- 2026-05-17: `control_chrome::fill_text(...)` now delegates to the shared
  `text_control_label(...)` role instead of keeping that layout policy local to IMUI chrome.
- 2026-05-14: made `DisclosureResponse` / `ComboResponse` accessor-first for trigger/open/toggle
  state too. Public callers now read trigger details through `response()` and semantic helpers, the
  response types no longer expose external `Default` construction, and
  `tools/gate_imui_workstream_source.py` rejects public fields from returning.
- 2026-05-14: made `InputTextPickerResponse` accessor-first for input/open/pick state. Text picker
  tests now use `picked()` / `picked_index()` and the source gate rejects public picker response
  fields and public default construction from returning.
- 2026-05-14: made `TabBarResponse` / `TabTriggerResponse` accessor-first for selection and trigger
  state. Existing tab tests already use `selected_id()`, `selected_changed()`, `trigger(...)`, and
  trigger edge helpers; the source gate now rejects public tab response fields and public
  `TabBarResponse` default construction from returning.
- 2026-05-14: made `VirtualListResponse` accessor-first too. Public callers keep using `handle()`
  and `rendered_range()`, while the response's scroll handle and rendered-range storage are
  crate-local and guarded by the IMUI source gate.
- 2026-05-14: made `TableResponse` / `TableHeaderResponse` /
  `TableColumnResizeResponse` accessor-first for header metadata, sort state, resize bounds, and
  drag state. Public table tests now read `column_index()`, `sortable()`, `sort_direction()`,
  `resize()`, `min_width()`, and `max_width()` instead of response fields; the source gate rejects
  public table response fields and default construction from returning.
- 2026-05-14: made `DragResponse` accessor-first for started/dragging/stopped/delta/total state.
  Internal response assemblers can still populate drag state, while external callers stay on
  `started()`, `dragging()`, `stopped()`, `delta()`, and `total()` or the higher-level
  `ResponseExt` drag helpers.
- 2026-05-14: made `DragSourceResponse` / `DropTargetResponse` accessor-first too. Active source
  construction and empty target construction are crate-local, smoke tests now validate accessor
  usage through helper-returned responses, and the source gate rejects public drag/drop response
  fields or default construction from returning.
- 2026-05-14: made `ResponseExt` aggregate drag state accessor-first too. Public callers now read
  the aggregate response through `drag()`, `drag_started()`, `dragging()`, `drag_stopped()`,
  `drag_delta()`, and `drag_total()`, while `populate_pressable_drag_response(...)` and disabled
  sanitization use crate-local mutators. `tools/gate_imui_workstream_source.py` rejects a public
  `ResponseExt.drag` field or direct `ResponseExt` drag field reads from returning.
- 2026-05-14: made `ResponseExt` press/context-menu derived signal storage private too. Public
  callers keep using `secondary_clicked()`, `double_clicked()`, `long_pressed()`,
  `press_holding()`, `context_menu_requested()`, `context_menu_anchor()`, `pointer_clicked()`, and
  `pointer_click_modifiers()`, while `item_behavior`, disclosure headers, and disabled sanitization
  write through crate-local setters/clear helpers. `tools/gate_imui_workstream_source.py` rejects
  public fields or direct runtime writes from returning.
- 2026-05-14: made `ResponseExt` lifecycle edge storage private too. Public callers keep using
  `activated()`, `deactivated()`, `edited()`, and `deactivated_after_edit()`, while lifecycle
  runtime assembly plus combo/text-picker edit merging use crate-local set/merge helpers.
  `tools/gate_imui_workstream_source.py` rejects public lifecycle fields or direct runtime writes
  from returning.
- 2026-05-14: made `ResponseExt` raw hover, hover-delay, active-item block, and nav-highlight
  storage private too. Public callers and tests use `pointer_hovered_raw()`,
  `pointer_hovered_raw_below_barrier()`, the hover-delay accessors, `hover_blocked_by_active_item()`,
  and `nav_highlighted()`, while pressable/disclosure response assembly uses crate-local setters.
  Disabled sanitization still clears only nav highlight so `ALLOW_WHEN_DISABLED` raw-hover queries
  keep working.
- 2026-05-14: made `ResponseExt.enabled` storage private too. Public/demo/test callers use
  `enabled()`, while disabled sanitization and text controls use crate-local `set_enabled(...)`.
- 2026-05-14: made `ResponseExt.id` storage private too. Public/demo/test callers use `id()`, while
  pressable, disclosure, and text-control response assemblers use crate-local `set_id(...)`.
  Routing consumers in combo, menu/popup, tooltip, drag/drop, tab focus, text-picker, facade
  focusable recording, and the editor proof now consume identity through accessors. `ResponseExt.core`
  intentionally remains public in this slice because it is the broader shared
  `fret_authoring::Response` compatibility surface and needs a separate contract audit before
  accessor-only migration. The same compile sweep also migrated stale `apps/fret-examples-imui`
  demo reads for previously sealed floating/disclosure/combo/table response fields back onto their
  accessor APIs.
- 2026-05-14: made `ResponseExt.core` storage private too after the separate contract audit. Public
  callers keep the shared authoring-response bridge through `core()` and `from_core(...)`, plus
  focused signal accessors such as `rect()`, `focused()`, `hovered()`, and `pressed()`. Runtime
  pressable/disclosure/text/disabled/combo/multi-select/text-picker assembly writes through
  crate-local core setters and merge helpers, and `tools/gate_imui_workstream_source.py` rejects
  public `core` field access or direct `.core` reads/writes from returning in the covered IMUI
  surfaces.
- 2026-05-14: made emitted adapter seam records read-only as well. `AdapterSignalRecord` and
  `AdapterSignalMetadata` now expose constructor/accessor APIs while `report_adapter_signal(...)`
  remains the canonical emission path. `AdapterSeamOptions` stays as a public-field input options
  bag because callers need to provide a reporter and focus-restore target ergonomically.
- 2026-05-14: made `DragValueCoreResponse` accessor-first in `fret-ui-editor`. Scrub response
  storage for `dragging`, `hovered`, `pressed`, and `focused` is now private, `DragValueCore`
  constructs the response internally, and `DragValue` / `AxisDragValue` consume those signals
  through read-only accessors. The response no longer exposes external default construction. The
  source gate rejects public fields, stale direct field reads, and public default construction.
  Focused gates passed locally: `cargo check -p fret-ui-editor --tests`,
  `cargo nextest run -p fret-ui-editor drag_value --no-fail-fast`,
  `cargo check -p fret-demo --bin imui_editor_proof_demo`, `python tools/gate_imui_workstream_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
- 2026-05-14: made `DebugDrawResponse` accessor-first in `fret-ui-kit`. Response and summary
  storage are private, external default construction is gone, and public callers use
  `response()`, `list_summary()`, and `command_summaries()`. Cookbook and smoke tests now read the
  response through accessors, and the source gate rejects the old public-field shape.
- 2026-05-14: made `DebugDrawCommandSummary` and `DebugDrawListSummary` accessor-first as well.
  Command kind/channel/clip metadata and aggregate list counters remain readable through explicit
  accessors, but storage, default construction, and final-clip-depth mutation stay internal to
  `debug_draw_controls`. Cookbook, smoke tests, and debug-draw owner tests now use accessor reads,
  and the source gate rejects public summary fields or external default construction from returning.
- 2026-05-14: hardened `tools/gate_imui_workstream_source.py` with an opaque-output-struct check
  for the sealed IMUI response/context/summary records. The gate now parses each listed public
  output struct body and fails on any externally public field, so new response-surface cleanup does
  not depend only on per-field string markers.
- 2026-05-14: made `VecEditAxisOutcome` and `TransformEditAxisOutcome` accessor-first in
  `fret-ui-editor`. Axis edit events still carry the same section/axis/session-close values, but
  storage and construction are internal to the editor controls. The editor proof reads them through
  `section()`, `axis()`, and `outcome()`, and the source gate covers the records with the reusable
  opaque-output-struct check.
- 2026-05-14: hardened the opaque-output gate again so it scans the `fret-imui`,
  `fret-ui-editor`, and `fret-ui-kit::imui` source roots for public output-style structs by suffix.
  New public `*Response`/`*Outcome`/`*Summary`/`*Signal`/`*Record`/`*Context` records must now be
  explicitly registered in `tools/gate_imui_workstream_source.py` before the field-opacity check
  can pass.
- 2026-05-14: made editor `ColorEdit` event/request/payload records accessor-first too.
  `ColorEditPaletteSlotDrop`, `ColorEditEyedropperRequest`, and `ColorEditDragDropPayload` now keep
  storage private while exposing callback reads through explicit accessors. The opaque-output gate
  suffix scan now includes `*Request`, `*Payload`, and `*Drop` records in the IMUI/editor source
  roots. Focused gates passed locally: `cargo nextest run -p fret-ui-editor color_edit
  --no-fail-fast`, `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke
  --test imui_surface_policy --no-fail-fast`, and `python tools/gate_imui_workstream_source.py`.
- 2026-05-14: extended the opaque public-struct gate from output records to shared public state
  helpers by scanning `*State` names and registering `ImUiMultiSelectState`. This keeps future
  IMUI/editor state helpers accessor-first by default while leaving explicit options/input bags
  outside the catalog.

## P3 Design Surface Readiness Gates

Use these for the current design/theme readiness note:

```powershell
rg -n "EditorThemePresetV1|ImguiLikeDense|install_editor_theme_preset_v1|reapply_installed_editor_theme_preset_v1" ecosystem/fret-ui-editor/src/theme.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
rg -n "component\\.imui\\.disabled_alpha|imui_text_input_style_from_theme|input_text_model_uses_compact_imui_chrome_without_focus_ring|textarea_model_uses_compact_imui_chrome_without_focus_ring|hovered_like_imgui|ImUiHoveredFlags" ecosystem/fret-ui-kit/src/imui
rg -n "ShowStyleEditor|ImGuiStyle|PushStyleColor|PushStyleVar|StyleColorsDark|StyleColorsLight|StyleColorsClassic" repo-ref/imgui/imgui.h repo-ref/imgui/imgui_demo.cpp
cargo nextest run -p fret-ui-editor default_preset_keeps_existing_editor_patch_baseline imgui_like_dense_preset_overrides_density_and_field_chrome installed_preset_can_be_reapplied_after_base_theme_reset --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui input_text_model_uses_compact_imui_chrome_without_focus_ring textarea_model_uses_compact_imui_chrome_without_focus_ring --no-fail-fast
```

Run evidence:

- 2026-05-14: deleted the unused `apply_editor_theme_patch_v1` compatibility wrapper from
  `ecosystem/fret-ui-editor/src/theme.rs`. In-tree callers already use
  `apply_editor_theme_preset_v1(...)` or `install_editor_theme_preset_v1(...)`, and
  `tools/gate_imui_workstream_source.py` rejects the old wrapper from returning.

## P3 Porting Sugar Readiness Gates

Use these for the current porting-sugar readiness note:

```powershell
rg -n "SameLine|PushItemWidth|SetNextItemWidth|CalcItemWidth|PushID|##|###" repo-ref/imgui/imgui.h repo-ref/imgui/imgui.cpp repo-ref/imgui/imgui_demo.cpp
rg -n "row\\(|horizontal\\(|horizontal_with_options|row_with|id_source|test_id|push_id" ecosystem/fret-imui/src/frontend.rs ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer ecosystem/fret-ui-kit/src/imui/options/containers.rs apps/fret-cookbook/examples/imui_action_basics.rs apps/fret-cookbook/examples/imui_editor_controls_basics.rs apps/fret-examples/src/imui_editor_proof_demo.rs
rg -n "row_cx\\.row_options" apps/fret-examples/src/imui_editor_proof_demo.rs apps/fret-examples/src/editor_notes_demo.rs
cargo check -p fret-demo --bin imui_editor_proof_demo
```

Run evidence:

- 2026-05-14: removed proof-surface calls that manually passed
  `PropertyRow::new().options(row_cx.row_options.clone())` in
  `apps/fret-examples/src/imui_editor_proof_demo.rs` and `apps/fret-examples/src/editor_notes_demo.rs`.
  Default property-row policy now stays centralized in `PropertyGridRowCx::row_with(...)` instead
  of leaking into app/proof code.
- 2026-05-14: `tools/gate_imui_workstream_source.py` now rejects `row_cx.row_options` in both
  proof surfaces so future cleanup cannot regress into ad hoc row-policy wiring.
- 2026-05-14: `cargo check -p fret-demo --bin imui_editor_proof_demo`,
  `cargo check -p fret-demo --bin editor_notes_demo`,
  `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`, `python tools/gate_imui_facade_teaching_source.py`,
  and `git diff --check` passed locally.
- 2026-05-14: `PropertyGridVirtualizedRowCx` now mirrors `PropertyGridRowCx` with `row(...)` and
  `row_with(...)`, so virtualized property-grid callers can keep row policy centralized instead of
  copying `row_cx.row_options` into each row. The adapter smoke now uses the helper, and
  `tools/gate_imui_workstream_source.py` rejects that manual copy from returning there.
- 2026-05-14: deleted the unused public `PropertyGridRow` wrapper from
  `ecosystem/fret-ui-editor/src/composites/property_grid.rs` and its composite re-export. The
  editor row authoring surface now has one canonical grid policy path:
  `PropertyGridRowCx::row(...)` / `row_with(...)`, with custom raw rows still using
  `PropertyRow` directly. `tools/gate_imui_workstream_source.py` rejects the redundant wrapper and
  re-export from returning.
- 2026-05-14: made `PropertyGridRowCx` and `PropertyGridVirtualizedRowCx` opaque row contexts
  instead of exposing `row_options` as a public field. External authoring now keeps the row policy
  path on `row(...)` / `row_with(...)`; editor-internal composites that need row-local policy
  patches use crate-local `row_options()`. The source gate rejects public `row_options` /
  `density` fields from returning.
- 2026-05-14: made `InspectorPanelCx` opaque as well: callers get `density()`, `query()`,
  `is_query_empty()`, and `matches(...)` instead of public `query_lower` implementation state. The
  source gate rejects the old public context fields from returning.

## P3 Child Region Readiness Gates

Use these for the current child-region readiness note:

```powershell
cargo nextest run -p fret-imui child_region --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_child_region_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui child_region_resize --no-fail-fast
cargo check -p fret-demo --bin workspace_shell_demo
```

## P3 Collection Helper Readiness Gates

Use these for the current collection-helper readiness note:

```powershell
python tools/gate_imui_editor_collection_source.py
cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast
```

- 2026-05-14: `ImUiMultiSelectState` now exposes explicit storage operations instead of public
  `selected`/`anchor` fields. The proof collection uses `selected()`, `anchor()`,
  `first_selected()`, `selected_count()`, `is_empty()`, `clear()`, `new(...)`, and `single(...)`;
  `tools/gate_imui_workstream_source.py` rejects the public-field shape and direct proof-side field
  mutation from returning.
- 2026-05-14: moved visible-order selection repair into
  `ImUiMultiSelectState::from_ordered_selection(...)`. This keeps the storage helper aligned with
  Dear ImGui's `ImGuiSelectionBasicStorage` direction without copying `BeginMultiSelect` /
  `EndMultiSelect` runtime ownership into `fret-imui`; the proof collection no longer carries a
  local `proof_collection_normalize_selection(...)`.
- 2026-05-14: refreshed `P3_COLLECTION_HELPER_READINESS_2026-05-06.md` to keep multi-select
  request/IO vocabulary candidate-only. `tools/gate_imui_workstream_source.py` now rejects
  `BeginMultiSelect`/`EndMultiSelect`-style runtime names from the current `fret-ui-kit::imui`
  storage helper.
- 2026-05-14: the reusable opaque public-struct gate now includes `*State` and covers
  `ImUiMultiSelectState`, so the collection helper storage contract cannot regress to public
  `selected` / `anchor` fields by slipping outside the previous output-record catalog. Focused
  gates passed locally: `python tools/gate_imui_workstream_source.py`,
  `python tools/check_workstream_catalog.py`, `cargo nextest run -p fret-imui interaction_drag
  --no-fail-fast`, `cargo nextest run -p fret-ui-kit --features imui --test imui_selectable_smoke
  --test imui_sortable_recipe_smoke --test imui_drag_preview_smoke --no-fail-fast`, and
  `git diff --check`.

## P3 Execution Priority Review Gates

Use these when changing the P3 execution-priority read:

```powershell
python tools/diag_gate_imui_product_chain.py
python tools/audit_crate.py --crate fret-imui
python tools/audit_crate.py --crate fret-ui-kit
python tools/audit_crate.py --crate fret-ui-editor
python tools/audit_crate.py --crate fret
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/check_workstream_catalog.py
git diff --check
```

## P4 Performance Alignment Review Gates

Use these when changing the Dear ImGui / Zed / egui performance interpretation:

```powershell
rg -n "FrameArenaScratch|bounds-tree|diag perf|diag stats|renderer churn|Zed|egui" docs/workstreams/imui-imgui-gap-closure-v1/P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md
python tools/diag_gate_imui_product_chain.py --only discovery
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
git diff --check
```

Run evidence:

- 2026-05-15: registered `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md` in
  `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, and this evidence file so performance discipline
  remains part of the active Dear ImGui gap read.
- The review keeps perf work in `diag-perf-attribution-v1`, `ui-perf-zed-smoothness-v1`, and the
  product-chain docking perf gate. It explicitly rejects turning egui's full-layout-every-frame
  model or Dear ImGui's widget breadth into a `fret-imui` runtime/API widening target.
- 2026-05-23: `docs/workstreams/editor-canvas-paint-replay-slice-v1/` closed after the r59 Windows
  RTX4090 target-machine pass. Evidence:
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/summary.json`,
  `target/fret-diag/editor-paint-contract-validate-20260523-r59-attrib/summary.json`,
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/artifact-verification.summary.json`,
  and
  `target/fret-diag/editor-paint-contract-validate-20260523-r59/editor-paint-contract-closeout.summary.json`.
  The closeout retained `owner=canvas-paint-replay`, kept checked-in baselines unchanged, and
  confirms that current smoothness pressure belongs to editor paint / perf owner lanes rather than
  `fret-imui` helper growth.

## P3 Product Chain Gate

Use this gate before treating a Dear ImGui-class gap as widget/API breadth. It validates the current
product chain's discovery path, promoted script/suite inputs, and source gates across the generic
IMUI cookbook path, editor-controls cookbook path, editor proof, and workspace shell:

```powershell
python tools/diag_gate_imui_product_chain.py
```

Run evidence:

- 2026-05-14: `python tools/diag_gate_imui_product_chain.py` passed locally.
- The run validated cookbook example discovery, native demo discovery, tool-app discovery,
  campaign doctor output, all promoted product-chain diagnostics scripts, and both IMUI source
  gates.
- This is not a replacement for launched GUI gates. It is the first-open maintenance gate that keeps
  the cross-app product path visible before opening a new helper or widget follow-on.

## Runtime Boundary Gate

Use this gate to protect the core architectural constraint for this lane: `fret-imui` stays a thin
immediate facade over `fret-authoring` + `fret-ui`; policy-heavy IMUI helpers stay in
`fret-ui-kit::imui`, editor controls stay in `fret-ui-editor`, and docking/workspace behavior stays
outside the runtime facade.

```powershell
python tools/gate_imui_workstream_source.py
python tools/audit_crate.py --crate fret-imui
python tools/check_layering.py
```

Run evidence:

- 2026-05-14: `python tools/audit_crate.py --crate fret-imui` passed and reported direct runtime
  dependencies only on `fret-authoring` and `fret-ui`; `fret-ui-kit` remains dev-only for focused
  behavior tests.
- 2026-05-14: `python tools/check_layering.py` passed.
- 2026-05-14: `python tools/gate_imui_workstream_source.py` now validates
  `ecosystem/fret-imui/Cargo.toml` `[dependencies]` and rejects policy/runtime drift such as
  `fret-ui-kit`, `fret-ui-editor`, `fret-docking`, `fret-workspace`, `fret-ui-shadcn`, `winit`, or
  `wgpu` in the runtime dependency section.

## IMUI Maintainer Test Ownership / Table Layout Gate

Use this gate when changing menu/tab interaction ownership, table cell semantics, or IMUI layout
mechanics that can accidentally make diagnostics bounds diverge from visual column layout.

```powershell
cargo nextest run -p fret-imui composition --no-fail-fast
cargo nextest run -p fret-imui floating --no-fail-fast
cargo nextest run -p fret-imui interaction_drag --no-fail-fast
cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast
cargo nextest run -p fret-imui interaction_press --no-fail-fast
cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast
cargo nextest run -p fret-imui label_identity --no-fail-fast
cargo nextest run -p fret-imui models_combo --no-fail-fast
cargo nextest run -p fret-imui models_controls --no-fail-fast
cargo nextest run -p fret-imui models_text_area --no-fail-fast
cargo nextest run -p fret-imui models_text_picker --no-fail-fast
cargo nextest run -p fret-imui popup_hover --no-fail-fast
cargo nextest run -p fret-imui table_helper_applies_explicit_row_and_cell_background_overrides --no-fail-fast
cargo nextest run -p fret-imui table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast
cargo nextest run -p fret-imui --no-fail-fast
python tools/gate_imui_workstream_source.py
git diff --check
```

Run evidence:

- 2026-05-14: moved the shared IMUI test host, fake text/path/svg/material service, event
  dispatch helpers, geometry helpers, and floating overlay harness from
  `ecosystem/fret-imui/src/tests/mod.rs` into the shared IMUI test harness.
  `mod.rs` now only imports the harness and owns the test-module index.
- 2026-05-14: split the shared IMUI test harness into owner modules under
  `ecosystem/fret-imui/src/tests/harness/`: `services`, `host`, `frames`, `events`, `lookup`,
  `hover_scenes`, and `floating_scenes`.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/composition.rs` file into
  `mount_smoke`, `control_geometry`, and `layout_collections` test owners under
  `ecosystem/fret-imui/src/tests/composition/`.
- 2026-05-14: `cargo nextest run -p fret-imui composition --no-fail-fast` passed locally with 19
  tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/floating.rs` file into
  `movement_z_order`, `input_modes`, `layer_dismissal`, and `window_options` test owners under
  `ecosystem/fret-imui/src/tests/floating/`.
- 2026-05-14: `cargo nextest run -p fret-imui floating --no-fail-fast` passed locally with 25
  tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_drag.rs` file
  into `multi_select`, `collection_drag`, `drag_core`, `drag_preview`, and `sortable` test owners
  under `ecosystem/fret-imui/src/tests/interaction_drag/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_drag --no-fail-fast` passed locally with
  8 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_menu_tabs.rs`
  file into `menu_activation`, `submenu_hover`, `submenu_shortcuts`, and `tabs` test owners under
  `ecosystem/fret-imui/src/tests/interaction_menu_tabs/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_menu_tabs --no-fail-fast` passed
  locally with 18 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_press.rs`
  file into `click_edges`, `lifecycle`, `context_menu`, and `press_hold` test owners under
  `ecosystem/fret-imui/src/tests/interaction_press/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_press --no-fail-fast` passed
  locally with 9 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/interaction_shortcuts.rs`
  file into `command_metadata`, `button_shortcuts`, `selectable_shortcuts`, and `disclosure_tree`
  test owners under `ecosystem/fret-imui/src/tests/interaction_shortcuts/`.
- 2026-05-14: `cargo nextest run -p fret-imui interaction_shortcuts --no-fail-fast` passed
  locally with 10 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/label_identity.rs` file
  into `visible_suffixes`, `model_controls`, `explicit_ids`, and `table_headers` test owners
  under `ecosystem/fret-imui/src/tests/label_identity/`.
- 2026-05-14: `cargo nextest run -p fret-imui label_identity --no-fail-fast` passed locally
  with 7 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_combo.rs` file into
  `combo_model` and `combo_direct` test owners under
  `ecosystem/fret-imui/src/tests/models_combo/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_combo --no-fail-fast` passed locally with 11
  tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_controls.rs` file
  into `checkbox`, `switch`, and `slider` test owners under
  `ecosystem/fret-imui/src/tests/models_controls/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_controls --no-fail-fast` passed locally
  with 6 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_text_area.rs`
  file into `modes`, `commands`, `model_changed`, and `lifecycle` test owners under
  `ecosystem/fret-imui/src/tests/models_text_area/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_text_area --no-fail-fast` passed locally
  with 8 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/models_text_picker.rs`
  file into `completion_popup`, `history_popup`, `completion_keyboard`, `history_keyboard`, and
  `empty_keyboard` test owners under `ecosystem/fret-imui/src/tests/models_text_picker/`.
- 2026-05-14: `cargo nextest run -p fret-imui models_text_picker --no-fail-fast` passed
  locally with 6 tests.
- 2026-05-14: split the former single `ecosystem/fret-imui/src/tests/popup_hover.rs` file into
  `context_basics`, `hover_flags`, `item_keyboard`, `item_pointer`, and `lifecycle_modal` test
  owners under `ecosystem/fret-imui/src/tests/popup_hover/`.
- 2026-05-14: `cargo nextest run -p fret-imui popup_hover --no-fail-fast` passed locally with 21
  tests.
- 2026-05-14: `cargo nextest run -p fret-imui
  table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast` passed
  locally after moving table body-cell test semantics to layout-transparent
  `SemanticsDecoration`.
- 2026-05-16: `cargo nextest run -p fret-imui
  table_helper_applies_explicit_row_and_cell_background_overrides
  table_helper_keeps_header_and_body_columns_aligned_and_clips_long_cells --no-fail-fast` passed
  locally with 2 tests, proving explicit row/cell background paint order and preserving the
  existing table layout semantics gate.
- 2026-05-14: `cargo nextest run -p fret-imui --no-fail-fast` passed locally with 163 tests.
- 2026-05-14: `python tools/gate_imui_workstream_source.py` and `git diff --check` passed locally.

## P3 Diagnostics / DevTools First-Open Gate

Use these gates for the current Dear ImGui-class diagnostics discoverability read. The discovery
gate verifies the first-open DevTools/tool-app discovery index and repo-owned campaign preflight
that a maintainer should find before opening the GUI/MCP branch. `--discovery-only` is the
cold-start entry and may still build `fretboard-dev`; use `--discovery-only --reuse-built` for the
fast drift check when the binary is already present and the goal is to validate the discovery path
without hiding it behind a large Rust build. The launched gate verifies the shared CLI-first path
that DevTools GUI and MCP consume later: direct script run, named bundle capture, latest bundle
resolution through `script.result.json:last_bundle_dir`, bundle compare, campaign execution,
`diag summarize`, and `diag dashboard`. The gate writes `gate.progress.jsonl` in launched mode so
an outer timeout still leaves the last reached stage.

```powershell
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only
python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built
python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke
python tools/diag_gate_imui_p2_devtools_first_open.py --reuse-built --out-dir target/imui-p2-devtools-first-open-smoke
```

Run evidence:

- 2026-05-14: `python tools/diag_gate_imui_p2_devtools_first_open.py --reuse-built --out-dir target/imui-p2-devtools-first-open-smoke-reuse-final --timeout-ms 240000 --poll-ms 50` passed
  locally. Run root:
  `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728`.
- 2026-05-14: `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728/gate.progress.jsonl`
  records the full first-open path: tool-app discovery, campaign doctor preflight, direct
  `todo-baseline` run, latest resolution, compare, campaign run, summarize, dashboard, and final
  `gate.pass`.
- 2026-05-14: direct script result
  `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728/direct/sessions/1778726646873-112152/script.result.json`
  reports `stage=passed`, `run_id=1778726649588`, and `last_bundle_dir=1778726650016-todo-after-remove`.
- 2026-05-14: campaign summary
  `target/imui-p2-devtools-first-open-smoke-reuse-final/1778726646728/campaign/campaigns/devtools-first-open-smoke/1778726650543/regression.summary.json`
  reports `items_total=1`, `passed=1`, and zero deterministic/flaky/tooling/timeout failures.
- 2026-05-14: `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only` passed
  locally. The gate now validates `fretboard-dev list tool-apps` human and JSON output, the
  `docs/diagnostics-first-open.md` first-open anchor, the DevTools GUI and MCP launch/docs/gate
  entries, and `fretboard-dev diag doctor campaigns --json` with `ok=true`.
- 2026-05-16: `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`
  passed locally. This is the preferred quick drift check for the first-open discovery surface when
  `target/debug/fretboard-dev.exe` already exists; the non-`--reuse-built` discovery form remains
  the cold-start entry that also proves the maintainer build path.
- 2026-05-16: DevTools GUI first-open posture now stays summary-first: the header renders
  `First-open Next Actions`, `Evidence & Results` defaults to `Guide`, and the full first-open /
  dogfood / demo-metrics-debug / gate-command reference panels remain available from that guide
  surface. Focused gates passed locally:
  `cargo nextest run -p fret-devtools devtools_first_open_next_action_lines_prioritize_stateful_workflow devtools_first_open_lines_surface_canonical_paths devtools_dogfood_workflow_lines_surface_ui_gallery_loop devtools_demo_metrics_debug_lines_surface_canonical_routes devtools_gate_command_lines_surface_first_class_gates --no-fail-fast`
  and `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
- 2026-05-21: DevTools demo/metrics/debug discovery now has a shared CLI/JSON route, not only a
  GUI guide panel. `fretboard-dev list tool-apps` prints `route: demo-metrics-debug`, and
  `fretboard-dev list tool-apps --json` exposes the same route under `first_open_routes` with
  grouped editor demo, metrics, and debug commands, including `diag trace <bundle-or-dir> --json`.
  Focused gates passed locally for this slice:
  `cargo fmt -p fretboard-dev --check`,
  `cargo fmt -p fretboard-dev -p fret-devtools -p fret-devtools-mcp --check`,
  `cargo nextest run -p fretboard-dev tool_apps_list_names_first_open_routes tool_apps_json_value_exposes_stable_machine_readable_shape --no-fail-fast`,
  `cargo nextest run -p fret-devtools devtools_demo_metrics_debug_lines_surface_canonical_routes --no-fail-fast`,
  `cargo nextest run -p fret-devtools-mcp mcp_first_open_resource_text_surfaces_imui_product_chain --no-fail-fast`,
  `cargo build -p fretboard-dev -p fret-devtools -p fret-devtools-mcp`,
  `python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py tools/diag_gate_imui_product_chain.py tools/gate_imui_workstream_source.py`,
  `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`,
  `python tools/diag_gate_imui_product_chain.py --only discovery --reuse-built`,
  `python tools/gate_imui_workstream_source.py`, and `git diff --check`.
- 2026-05-14: tightened the first-open discovery gate so `docs/diagnostics-first-open.md` must link
  maintainers from aggregate `skipped_policy` outcomes to the policy-skip / capability-provenance
  checklist, while preserving the distinction between `capability_source` and
  `capabilities_check_path`. Focused gates passed locally:
  `python -m py_compile tools/diag_gate_imui_p2_devtools_first_open.py` and
  `python tools/diag_gate_imui_p2_devtools_first_open.py --discovery-only --reuse-built`.
- 2026-05-14: a full launched rerun with
  `python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke-2026-05-14-discovery-gate --timeout-ms 240000`
  exceeded the local 10 minute command timeout before returning a result. A later short diagnostic
  rerun showed the last recorded step was `cargo build -p fret-demo --bin todo_demo`, so the gate
  now writes stage progress and supports `--reuse-built` to separate build cost from diagnostics
  path verification.
- 2026-05-14: `python tools/diag_gate_imui_p2_devtools_first_open.py --out-dir target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun --timeout-ms 240000` passed locally.
- Direct script result:
  `target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun/1778714082990/direct/sessions/1778714086733-135148/script.result.json`
  reports `stage=passed`, `run_id=1778714090298`, and `last_bundle_dir=1778714090682-todo-after-remove`.
- Campaign summary:
  `target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun/1778714082990/campaign/campaigns/devtools-first-open-smoke/1778714091193/regression.summary.json`
  reports `items_total=1`, `passed=1`, and zero deterministic/flaky/tooling/timeout failures.
- Suite summary:
  `target/imui-p2-devtools-first-open-smoke-2026-05-14-gap-refresh-rerun/1778714082990/campaign/campaigns/devtools-first-open-smoke/1778714091193/script-results/01-tools-diag-scripts-tooling-todo-todo-baseline.json/suite.summary.json`
  reports `status=passed` and `stage_counts.passed=1`.

## P0 Gates

Run these after doc edits in the first slice:

```powershell
python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
python tools/gate_imui_facade_teaching_source.py
python tools/gate_imui_workstream_source.py
rustfmt --edition 2024 --check apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs
rustfmt --edition 2024 --check apps/fret-examples/src/workspace_shell_demo.rs
rustfmt --edition 2024 --check apps/fret-examples/src/imui_editor_proof_demo.rs apps/fret-examples/src/imui_editor_proof_demo/collection.rs
cargo check -p fret-examples-imui
cargo check -p fret-demo --bin workspace_shell_demo
cargo check -p fret-demo --bin imui_editor_proof_demo
git diff --check
```

## Focused Code Gates For First Implementation Slice

Use these once the lane moves from audit/docs into code cleanup:

```powershell
cargo nextest run -p fret-imui --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast
```

## User-Usable Golden Path Gates

Use these to validate the current editor-panel proof surface:

```powershell
python tools/gate_imui_editor_collection_source.py
cargo check -p fret-demo --bin imui_editor_proof_demo
rg -n "imui_editor_proof_demo|state, command actions|command/action dispatch" apps/fret-cookbook/README.md apps/fret-cookbook/EXAMPLES.md docs/examples/README.md
```

## Runnable Proof Surfaces

```powershell
cargo run -p fret-cookbook --features cookbook-imui --example imui_action_basics
cargo run -p fret-cookbook --features cookbook-imui --example imui_debug_draw_basics
cargo run -p fret-cookbook --features cookbook-imui --example imui_editor_controls_basics
cargo run -p fret-demo --bin imui_editor_proof_demo
cargo run -p fret-demo --bin workspace_shell_demo
cargo run -p fret-demo --bin docking_arbitration_demo
```

## Gate Interpretation

- Passing source gates proves the current teaching/doc surfaces remain within the intended owner
  split. It does not prove Dear ImGui parity.
- Passing focused crate tests proves current helper behavior did not regress. It does not justify
  widening public APIs.
- Public helper widening still needs a separate follow-on, two proof surfaces, and a focused gate.

## Latest Focused Results

2026-05-16 teaching-comment cleanup:

- `python tools/gate_imui_facade_teaching_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-examples-imui` passed.
- `git diff --check` passed.

2026-05-17 property-row wrapping value layout gate:

- `cargo nextest run -p fret-ui-editor row_value_slot_grows_to_wrapping_value_text_under_narrow_layout --no-fail-fast` passed.

2026-05-17 property-grid wrapping value layout gate:

- `cargo nextest run -p fret-ui-editor property_grid_keeps_rows_separated_when_value_text_wraps_under_narrow_layout row_value_slot_grows_to_wrapping_value_text_under_narrow_layout row_value_slot_keeps_overflow_visible_for_wrapping_value_children --no-fail-fast` passed.

2026-05-17 editor-notes proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test editor_notes_editor_rail_surface --test editor_notes_device_shell_surface --no-fail-fast` passed.
- `cargo check -p fret-demo --bin editor_notes_demo` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 shared text-role layout gate:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib base_single_line_text_roles_stay_single_line_under_narrow_layout paragraph_text_role_measures_multiple_lines_under_narrow_layout --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 generic list text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib list_from_strings_uses_shared_single_line_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 generic tree text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib default_tree_row_label_uses_shared_list_row_text_role tree_toggle_glyph_uses_shared_chrome_glyph_text_role --no-fail-fast` passed.

2026-05-17 file tree text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib file_tree_row_icon_uses_shared_chrome_glyph_text_role file_tree_row_label_uses_shared_list_row_text_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 retained table text-role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib retained_table_text_uses_shared_table_cell_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 examples table proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin table_demo --bin table_stress_demo` passed.
- `cargo nextest run -p fret-examples --test table_demo_surface table_demo_keeps_fixed_table_text_on_roles --test table_stress_demo_surface table_stress_demo_keeps_fixed_table_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 datatable proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test datatable_demo_surface datatable_demo_keeps_fixed_table_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 virtual-list stress proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin virtual_list_stress_demo` passed.
- `cargo nextest run -p fret-examples --test virtual_list_stress_demo_surface virtual_list_stress_demo_keeps_fixed_row_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 IMUI virtual-list fixed-row clip slice:

- `cargo fmt --package fret-imui --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib fixed_virtual_list_rows_clip_content_to_row_height known_virtual_list_rows_clip_content_to_known_row_height measured_virtual_list_rows_keep_content_overflow_visible_for_measurement --no-fail-fast` passed.
- `cargo nextest run -p fret-imui virtual_list_fixed_rows_clip_oversized_row_content --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 retained tree fixed-row clip slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --check --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib retained_tree_fixed_rows_clip_to_row_height retained_tree_known_rows_clip_to_row_height retained_tree_measured_rows_keep_overflow_visible_for_measurement retained_tree_fixed_rows_mount_as_clip_boundaries retained_tree_measured_rows_do_not_force_row_clip --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 retained file-tree fixed-row clip slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --check --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib file_tree_retained_row_layout_clips_to_row_height file_tree_retained_rows_mount_as_clip_boundaries --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 retained table fixed-row clip slice:

- `cargo fmt --package fret-ui-kit` passed.
- `cargo fmt --check --package fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_fixed_body_row_layout_clips_to_row_height table_measured_body_row_layout_keeps_overflow_visible_for_measurement table_virtualized_retained_fixed_rows_mount_as_clip_boundaries table_virtualized_retained_measured_rows_do_not_force_row_clip --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_virtualized_retained_nested_pressable_remains_hittable_when_pointer_row_selection_disabled table_virtualized_retained_pointer_row_selection_policy_list_like table_virtualized_retained_colpin_alignment_gate_measured_rows_do_not_shrink_width table_virtualized_retained_colpin_alignment_gate_across_pin_resize_and_overflow --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 fixed-row clip milestone traceability slice:

- `MILESTONES.md` now records the IMUI virtual-list, retained tree, retained file-tree, and
  retained/eager table fixed-row clip results.
- `tools/gate_imui_workstream_source.py` now checks those milestone anchors so the row-owner clip
  contract does not disappear from the lane summary.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\check_workstream_catalog.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 canvas datagrid stress proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin canvas_datagrid_stress_demo` passed.
- `cargo nextest run -p fret-examples --test canvas_datagrid_stress_demo_surface canvas_datagrid_stress_demo_keeps_header_text_on_readout_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 date picker proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test date_picker_demo_surface date_picker_demo_keeps_fixed_chrome_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 components gallery table and overlay proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test components_gallery_surface components_gallery_table_torture_uses_text_roles components_gallery_chrome_and_controls_use_text_roles components_gallery_overlay_text_uses_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 markdown proof chrome text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test markdown_demo_surface markdown_demo_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 markdown image placeholder text-role slice:

- Source gap before fix: `markdown_demo` still kept image placeholder text in local direct
  `TextProps`, including the clickable placeholder path. These placeholders can contain long image
  URLs, so they should not use a fixed single-line role, but they also should not bypass the shared
  text-role contract.
- `markdown_demo_image_placeholder_text(...)` now routes placeholder copy through
  `decl_text::text_paragraph_break_words(...)` and preserves the demo-owned muted foreground via
  `inherit_foreground(...)`. The optional `Pressable` activation/semantics wrapper remains local.
- `apps/fret-examples/tests/markdown_demo_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove `markdown_demo` from the
  residual direct text allowlist.
- First post-fix focused `cargo nextest` runs for `markdown_demo_surface` and
  `text_role_residual_surface` timed out while background Cargo/Rustc compilation continued. Retried
  after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test markdown_demo_surface
  markdown_demo_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `git diff --check` passed.

2026-05-17 fret-examples residual bare text capability slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 fret-examples residual direct TextProps capability slice:

- `assets_demo` image/SVG stats and `image_heavy_memory_demo` image-memory stats now route through
  `text_control_readout(...)` instead of local direct `TextProps` construction.
- `text_role_residual_surface` now counts direct `TextProps { ... }` struct literals as well as
  `cx.text(...)` and `TextProps::new(...)`, limiting remaining direct text construction to explicit
  text/IME/rendering capability proofs.
- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 form proof header text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test form_demo_surface form_demo_header_status_uses_control_readout_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 sonner proof header text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test sonner_demo_surface sonner_demo_header_text_uses_fixed_chrome_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 echarts proof title text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test echarts_demo_surface echarts_demo_chart_titles_use_section_chrome_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 virtual row fallback text removal slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib missing_tree_virtual_row_placeholder_is_not_text missing_file_tree_virtual_row_placeholder_is_not_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-17 gallery disabled toaster placeholder slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app gallery_driver_disabled_toaster_does_not_emit_empty_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery app-sidebar collapsed placeholder slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app sidebar_app_collapsed_projects_do_not_emit_empty_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 fret-ui-ai empty placeholder helper slice:

- `cargo fmt -p fret-ui-ai` passed.
- `cargo nextest run -p fret-ui-ai hidden_ai_element_paths_use_non_text_placeholder --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery status-bar readout role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery driver chrome text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery driver chrome label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery minimal-root text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery debug-HUD text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery shell content/nav text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test code_editor_control_readout_surface code_editor_header_state_readouts_use_single_line_control_readout --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery editor preview text role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 code-view editor preview prose slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews editor_code_view_header_uses_paragraph_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 text editor/conformance header prose slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews editor_text_conformance_headers_use_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 code-editor IME gate button-label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews code_editor_mvp_internal_helpers_prefer_ui_child_over_anyelement --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-17 docking arbitration body/readout text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test docking_arbitration_surface docking_arbitration_demo_keeps_body_and_state_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-demo --bin docking_arbitration_demo` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-17 docking/container-query panel text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo check -p fret-demo --bin container_queries_docking_demo --bin docking_demo` passed.
- `cargo nextest run -p fret-examples --test container_queries_docking_surface container_queries_docking_demo_keeps_fixed_panel_text_on_roles --test docking_demo_surface docking_demo_keeps_panel_text_on_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-17 gallery retained-table torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_table_retained_torture_uses_structured_table_debug_ids --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery data-table torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_data_table_torture_exposes_header_row_anchor --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 data-table snippet table-cell text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test data_table_docs_surface data_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 table snippet table-cell text-role slice:

- Added `text_table_cell_emphasis(...)` in `fret-ui-kit::declarative::text` as a medium-weight
  derivative of the shared table-cell role. It keeps the same single-line, shrinkable,
  min-width-zero, ellipsis layout contract as `text_table_cell(...)`.
- UI Gallery ordinary Table snippets now share directory-local `table_cell_text(...)` and
  `table_cell_text_emphasis(...)` helpers. Demo/Usage/Footer/RTL/Actions and fixed body cells in
  Children no longer mount bare `ui::text(...)` inside fixed table cells. The rich
  `table_head_children(...)` / `table_caption_children(...)` sample text remains intentionally
  scoped to the children-API follow-up.
- `cargo nextest run -p fret-ui-kit --features imui --lib table_cell_emphasis_text_keeps_single_line_truncation_and_medium_weight --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-gallery --test table_docs_surface table_snippets_keep_fixed_cell_text_on_table_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 checkbox table-cell text-role slice:

- Red repro: `cargo nextest run -p fret-ui-gallery --test checkbox_table_action_first_surface
  checkbox_table_snippet_keeps_fixed_cell_text_on_table_role --no-fail-fast` failed before the fix
  because `checkbox/table.rs` was missing the local table-cell text helper.
- `apps/fret-ui-gallery/src/ui/snippets/checkbox/table.rs` keeps the existing
  `cx.actions().models::<act::ToggleAllRows>` / `.action(act::ToggleAllRows)` select-all flow, and
  now routes member/role fixed table cells through a local helper backed by
  `text_table_cell(...)` instead of `ui::text(id)` / `ui::text(role)`.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test checkbox_table_action_first_surface --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 typography table-cell text-role slice:

- Red repro: `cargo nextest run -p fret-ui-gallery --test typography_docs_surface
  typography_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` failed before the
  fix because `typography/mod.rs` was missing the shared table-cell text helper.
- `apps/fret-ui-gallery/src/ui/snippets/typography/table.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/typography/demo.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/typography/rtl.rs` now route fixed table body cells through
  a typography-local helper backed by `text_table_cell(...)`. Typography prose, headings, and rich
  inline-link examples are intentionally unchanged.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test typography_docs_surface typography_table_snippets_keep_fixed_cell_text_on_table_role --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed with `PYTHONIOENCODING=utf-8`.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery data-grid text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery data paragraph text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_data_grid_uses_table_cell_text_roles gallery_data_table_torture_exposes_header_row_anchor gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery inspector torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_inspector_torture_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery virtual-list torture text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_virtual_list_torture_uses_fixed_row_text_roles harness_ui_kit_list_torture_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery harness header text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_table_retained_torture_uses_structured_table_debug_ids harness_hit_test_torture_uses_header_text_roles harness_virtual_list_torture_uses_fixed_row_text_roles harness_ui_kit_list_torture_uses_fixed_row_text_roles harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery view-cache list text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery view-cache control-label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery view-cache popover body slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews harness_view_cache_uses_fixed_row_text_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery tree torture status text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_tree_torture_uses_control_readout_for_status_text --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery overlay status text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_status_text_uses_control_readout_roles gallery_menus_last_action_uses_control_readout_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery overlay scroll-row text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_scroll_rows_use_list_row_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery overlay body prose slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_body_copy_uses_paragraph_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-17 gallery chrome torture control-label slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews page_chrome_torture_uses_control_label_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 tooltip body text role slice:

- `cargo fmt -p fret-ui-kit` passed.
- `cargo nextest run -p fret-ui-kit --features imui --lib tooltip_body_text_uses_compact_paragraph_role --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 collection proof text-role slice:

- `cargo fmt -p fret-examples` passed.
- `cargo nextest run -p fret-examples --test imui_editor_collection_text_roles_surface imui_editor_proof_collection_fixed_text_uses_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test imui_editor_collection_select_all_surface imui_editor_proof_demo_keeps_collection_select_all_app_owned_and_explicit --test imui_editor_collection_rename_surface imui_editor_proof_demo_keeps_collection_inline_rename_app_owned_and_explicit --no-fail-fast` passed.
- `cargo check -p fret-examples` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 child-region auto-height gate slice:

- `cargo fmt -p fret-imui` passed.
- `cargo nextest run -p fret-imui child_region_without_height_constraint_auto_sizes_to_content --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-20 child-region auto-width gate slice:

- `cargo nextest run -p fret-imui child_region_without_width_constraint_auto_sizes_to_content --no-fail-fast` passed.

2026-05-20 menu item row-anchor and shared-role slice:

- `cargo nextest run -p fret-ui-kit --features imui --lib menu_item_root_pressable_owns_visible_row_children menu_item_label_text_uses_shared_list_row_text_role menu_item_shortcut_text_uses_shared_control_readout_role menu_item_indicator_text_uses_shared_chrome_glyph_role --no-fail-fast` passed.
- `cargo nextest run -p fret-imui interaction_press::lifecycle::menu_item interaction_menu_tabs::menu_activation interaction_menu_tabs::submenu_hover --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.

2026-05-18 AI AudioPlayer state-marker text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_audio_player_text_role_surface audio_player_state_markers_use_non_text_spacers --no-fail-fast` timed out at 120s while waiting on/building the workspace.
- Retried with a longer timeout: `cargo nextest run -p fret-ui-gallery --test ai_audio_player_text_role_surface audio_player_state_markers_use_non_text_spacers --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` failed because the source test matched rustfmt-sensitive one-line `text_paragraph(...)` calls.
- After making the source test check role calls and text payloads separately, `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Queue visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Command snippet chrome text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  command_snippet_chrome_text_uses_shared_roles --no-fail-fast` failed because the view-runtime
  closure already receives an `ElementContext`, so the attempted `cx.elements()` bridge was wrong.
- After switching the view-runtime snippet to pass the closure `cx` directly, the focused gate
  compiled but failed on a rustfmt-sensitive import marker. The source test now checks the stable
  role import and `IntoUiElementInExt` markers separately.
- A subsequent focused run timed out at 300s during compilation. Retried with a longer timeout:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  command_snippet_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.

2026-05-18 gallery Accordion trigger text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  accordion_snippet_trigger_text_uses_button_label_role --no-fail-fast` passed.
- First `python tools/gate_imui_workstream_source.py` failed because the source marker required
  single-line `decl_text::text_button_label(cx, ...)` calls while rustfmt expanded several long
  labels. After loosening the marker to the stable helper call, the gate passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery ToggleGroup item text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_group_snippet_item_text_uses_button_label_role --no-fail-fast` timed out at 300s while a
  background `rustc` compile continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_group_snippet_item_text_uses_button_label_role --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Toggle item text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_snippet_item_text_uses_button_label_role --no-fail-fast` timed out at 300s while waiting
  on/building the focused test binary.
- Retried after background compilation finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  toggle_snippet_item_text_uses_button_label_role --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Button children text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  button_children_snippet_text_uses_button_label_role --no-fail-fast` timed out at 300s while
  background compilation continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  button_children_snippet_text_uses_button_label_role --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Tabs custom text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  tabs_snippet_custom_text_uses_shared_roles --no-fail-fast` timed out at 300s while background
  compilation continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  tabs_snippet_custom_text_uses_shared_roles --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Collapsible text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  collapsible_snippet_text_uses_shared_roles --no-fail-fast` timed out at 300s while background
  compilation continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  collapsible_snippet_text_uses_shared_roles --no-fail-fast` passed.
- After adding `collapsible/rtl.rs` and `collapsible/demo.rs` to the same source test, the final
  long-timeout run of
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  collapsible_snippet_text_uses_shared_roles --no-fail-fast` passed.
- First `python tools/gate_imui_workstream_source.py` failed on rustfmt-sensitive markers in
  `collapsible/usage.rs` and later `collapsible/demo.rs`; after splitting those markers into helper
  calls and text payloads, the gate passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Checkpoint visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI simple chrome visible snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
  timed out at 300s while a background `rustc` compile continued.
- Retried after the compile finished: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI selector/branch marker text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
  timed out at 300s while a background `rustc` compile continued.
- Retried after the compile finished: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI prompt/plan/commit-large text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- First `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast`
  timed out at 300s while a background `rustc` compile continued.
- Retried after the compile finished: `cargo nextest run -p fret-ui-gallery --test
  ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed after replacing a rustfmt-sensitive
  CommitLarge marker check with stable split markers.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI queue-prompt/transcription text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI WebPreview text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Chat text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI PromptInput provider/docs text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI chrome/readout text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Confirmation content text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Task content text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Conversation instrumentation text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI usage snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Message usage text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Canvas world spike text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Image demo text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI PromptInput referenced sources text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Artifact code-display status-marker slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `python -m json.tool tools/diag-scripts/ui-gallery/ai/ui-gallery-ai-artifact-docs-run-action.json` passed.
- `git diff --check` passed.

2026-05-18 AI ChainOfThought composable text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI TestResults composable text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Workflow snippet text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI Suggestions/reasoning/transcript text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 AI custom-children text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ai_visible_text_role_surface` passed.
- `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Sidebar snippet chrome text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  sidebar_snippet_chrome_text_uses_shared_roles --no-fail-fast` timed out at 184s while a
  background `rustc` compile continued.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  sidebar_snippet_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery AlertDialog custom text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  alert_dialog_snippet_custom_text_uses_shared_roles --no-fail-fast` failed before test execution:
  `rustc-LLVM ERROR: IO failure on output stream: no space on device`.
- `cargo clean -p fret-ui-gallery` removed 2726 generated build files, freeing about 6.2GiB.
- `python tools/gate_imui_workstream_source.py` passed.
- Retried after cleaning generated build artifacts:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  alert_dialog_snippet_custom_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery HoverCard text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- First `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  hover_card_snippet_text_uses_shared_roles --no-fail-fast` failed before test execution:
  `rustc-LLVM ERROR: IO failure on output stream: no space on device`.
- `cargo clean -p fret-ui-gallery`, `cargo clean -p fret-examples`,
  `cargo clean -p fret-examples-imui`, and `cargo clean -p fret-demo` removed generated build
  artifacts; free space recovered from under 1GiB to about 32GiB.
- Retried after cleaning generated build artifacts:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  hover_card_snippet_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Popover align text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  popover_align_snippet_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-18 gallery Tooltip keyboard shortcut text-role slice:

- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  tooltip_keyboard_shortcut_text_uses_shared_role --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs/workstreams/imui-imgui-gap-closure-v1/WORKSTREAM.json` passed.
- `git diff --check` passed.

2026-05-19 gallery Dialog scroll-row text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  dialog_scroll_row_text_uses_shared_roles --no-fail-fast` failed before the fix because
  `dialog/scrollable_content.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/dialog/scrollable_content.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/dialog/sticky_footer.rs` route scroll filler rows through
  `text_list_row_label(...)` instead of `ui::raw_text(format!(...))`.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  dialog_scroll_row_text_uses_shared_roles --no-fail-fast` timed out while contending with the
  parallel `cargo check` package-cache lock; Cargo/Rustc processes were allowed to exit.
- Retried after the lock cleared:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  dialog_scroll_row_text_uses_shared_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.

2026-05-19 extras marquee perf fixed title text-role slice:

- Source gap before fix: `apps/fret-examples/src/extras_marquee_perf_demo.rs` used local
  `ui::text(...).font_semibold()` for the fixed perf-probe title. The demo is a marquee animation
  perf surface, not a text rendering capability probe.
- `marquee_perf_title_text(...)` now routes that title through
  `decl_text::text_section_chrome_label(...)`, keeping fixed probe chrome single-line and
  shrinkable under resize.
- `apps/fret-examples/tests/extras_marquee_perf_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the role mapping and forbid the old local title
  styling from returning.
- The first post-fix `cargo nextest run -p fret-examples --test
  extras_marquee_perf_demo_surface extras_marquee_perf_demo_keeps_title_on_chrome_role
  --no-fail-fast` exposed a missing `AnyElement` import; after the import was added, a later
  `nextest` attempt timed out while background Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test extras_marquee_perf_demo_surface
  extras_marquee_perf_demo_keeps_title_on_chrome_role --no-fail-fast` passed.
- `cargo check -p fret-demo --bin extras_marquee_perf_demo` passed.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 effect reference chrome text-role slice:

- Source gap before fix: `custom_effect_v3_demo.rs`, `postprocess_theme_demo.rs`, and
  `liquid_glass_demo.rs` still used local `TextProps` for fixed effect overlay/header/card titles.
  These are ordinary chrome/readout slots around renderer/effect proofs, not text-rendering
  capability probes.
- `custom_effect_v3_demo` now uses `overlay_label_text(...)` backed by
  `decl_text::text_section_chrome_label(...)`. `postprocess_theme_demo` uses
  `postprocess_title_text(...)` for the fixed header title and `postprocess_readout_text(...)` for
  the explanatory header readout. `liquid_glass_demo` uses section-chrome helpers for both overlay
  pill labels and card titles.
- App-owned foreground inheritance and absolute/header/card container geometry remain local; the
  shrink/single-line text semantics now come from shared roles.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove these three demos from the
  residual direct text allowlist.
- First post-fix focused `cargo nextest` runs for `custom_effect_overlay_text_surface` and
  `text_role_residual_surface` timed out while background Cargo/Rustc compilation continued. Retried
  after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v3_and_effect_reference_chrome_use_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.

2026-05-19 custom effect web template overlay/control text-role slice:

- Source gap before fix: `custom_effect_v2_identity_web_demo.rs`,
  `custom_effect_v2_lut_web_demo.rs`, and `custom_effect_v2_glass_chrome_web_demo.rs` remained in
  the residual direct-text allowlist for fixed Web template overlay/control labels. These were
  ordinary demo chrome/readouts, not text-rendering capability probes.
- Starter and LUT templates now use `overlay_label_text(...)` backed by
  `text_section_chrome_label(...)` for their badges and `overlay_readout_text(...)` backed by
  `text_control_readout(...)` for unsupported-state and keyboard-hint text. Their absolute hints
  position containers around shared readout text instead of constructing local `TextProps`.
- The glass/chrome template now routes slider row names through `text_control_label(...)`, values
  through `text_control_readout(...)`, and the unsupported-state message through the same readout
  helper. App-owned foreground colors are preserved through `inherit_foreground(...)`.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove the three Web template demos
  from the residual direct text allowlist.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v2_web_templates_use_shared_text_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v2_web_overlay_readouts_use_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo check -p fret-examples --lib --target wasm32-unknown-unknown` passed.
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown` passed with existing unrelated
  `fret-platform-native` clipboard dead-code warnings.
- `cargo check -p fret-examples --lib` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.

2026-05-19 custom effect web overlay/readout text-role slice:

- Source gap before fix: `custom_effect_v2_web_demo.rs` still used three local `TextProps`
  constructions for fixed overlay/readout text: the unsupported-state message, the WebGPU badge
  label, and the bottom keyboard hint. These are ordinary UI chrome/readouts, not text-rendering
  capability probes.
- The demo now uses `overlay_label_text(...)` backed by `text_section_chrome_label(...)` for the
  badge and `overlay_readout_text(...)` backed by `text_control_readout(...)` for the unsupported
  state and keyboard hint. App-owned foreground colors are preserved with `inherit_foreground(...)`.
- The bottom hint keeps its absolute positioning by wrapping the shared readout role in a positioned
  container, avoiding a regression back to local text layout policy.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove `custom_effect_v2_web_demo`
  from the residual direct text allowlist.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v2_web_overlay_readouts_use_shared_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v1_v2_overlay_labels_use_shared_chrome_role --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-examples --lib --target wasm32-unknown-unknown` passed.
- `cargo check -p fret-demo-web --target wasm32-unknown-unknown` passed with existing unrelated
  `fret-platform-native` clipboard dead-code warnings.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 GenUI demo visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/genui_demo.rs` used local
  `ui::text(...).text_sm()` builders for action queue lines, JSON state/spec/schema panes, prompt
  text, spec issue lines, toolbar switch labels, count/readout values, stream status text, and
  stream guidance. It also used `ui::text("")` as an empty spacer in the editor panel.
- `genui_code_line_text(...)` now routes queue/state/validation/spec/schema/prompt pane lines
  through `decl_text::text_code_block(...)`, keeping code/log content on the code-text role
  instead of ordinary prose.
- `genui_readout_text(...)` now routes fixed toolbar labels, count/status/issue lines, patch-only
  status, and stream summaries through `decl_text::text_control_readout(...)`.
- `genui_paragraph_text(...)` routes stream guidance through `decl_text::text_compact_paragraph(...)`;
  the old empty text spacer was removed rather than converted to another invisible text node.
- `apps/fret-examples/tests/genui_demo_surface.rs` and `tools/gate_imui_workstream_source.py` guard
  the role mapping and forbid local `ui::text(...)`, `.text_sm()`, and the empty text spacer from
  returning.
- First post-fix `cargo nextest run -p fret-examples --test genui_demo_surface
  genui_demo_keeps_tool_text_on_roles --no-fail-fast` timed out while background Cargo/Rustc
  compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test genui_demo_surface
  genui_demo_keeps_tool_text_on_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with the existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.

2026-05-19 gallery NavigationMenu link-label text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_custom_link_labels_use_shared_button_label_role --no-fail-fast` failed before
  the fix because `navigation_menu/demo.rs` was missing the shared `decl_text` import and custom
  link labels still used `cx.text(label)`.
- `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/rtl.rs` now route custom icon/text link
  labels through `text_button_label(...)`.
- Card title/body line-clamp text remains deliberately out of this slice because it likely needs a
  separate list/card description role decision instead of a mechanical button-label migration.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_custom_link_labels_use_shared_button_label_role --no-fail-fast` passed.
- `cargo check -p fret-ui-gallery --test navigation_menu_docs_surface` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 compact paragraph line-clamp text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  compact_paragraph_line_clamp_text_uses_two_line_clamped_layout --no-fail-fast` failed before the
  fix because `text_compact_paragraph_line_clamp(...)` did not exist.
- Red repro:
  `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_list_item_copy_uses_shared_title_and_clamped_paragraph_roles --no-fail-fast`
  first timed out while a background Cargo compile continued, then failed before the fix because
  `navigation_menu/demo.rs` list-item titles/descriptions still used local `TextProps` policy.
- `ecosystem/fret-ui-kit/src/declarative/text.rs` now provides
  `text_compact_paragraph_line_clamp(...)` as a paragraph-family dense clamp helper with fill width,
  min-width-zero flex behavior, `max-height` derived from the theme line height, word wrapping, and
  ellipsis overflow.
- `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/docs_demo.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/navigation_menu/rtl.rs` now route ordinary list-item titles
  through `text_button_label(...)` and descriptions through
  `text_compact_paragraph_line_clamp(..., 2)`. Featured home-card brand copy remains explicit
  visual styling outside this slice.
- `cargo nextest run -p fret-ui-kit --features imui --lib
  compact_paragraph_line_clamp_text_uses_two_line_clamped_layout --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface
  navigation_menu_list_item_copy_uses_shared_title_and_clamped_paragraph_roles --no-fail-fast`
  passed.
- `cargo fmt -p fret-ui-kit -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-kit --features imui --lib` passed.
- `cargo check -p fret-ui-gallery --test navigation_menu_docs_surface` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Kbd custom-copy text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  kbd_snippets_route_custom_copy_through_shared_text_roles --no-fail-fast` failed before the fix
  because `kbd/demo.rs` did not import shared text roles and still rendered the `+` key separator
  through `ui::text("+")`.
- `apps/fret-ui-gallery/src/ui/snippets/kbd/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/kbd/rtl.rs` now route `+` separators through
  `text_chrome_glyph(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/kbd/group.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/kbd/tooltip.rs` now route inline helper/tooltip copy
  through `text_control_readout(...)` instead of local `ui::text(...).text_sm()` policy.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  kbd_snippets_route_custom_copy_through_shared_text_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Table children custom text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  table_children_snippet_routes_custom_text_through_shared_roles --no-fail-fast` failed before the
  fix because `table/children.rs` did not import shared text roles and still rendered custom header
  child text through `ui::text(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/table/children.rs` now routes custom header child text
  through `super::table_cell_text(...)` and caption prose through
  `decl_text::text_paragraph(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  table_children_snippet_routes_custom_text_through_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First `python tools\gate_imui_workstream_source.py` attempt failed because the older table
  source check still required the intentional `ui::text("Status ")` / `ui::text("Amount ")` /
  `ui::text("(USD)")` exception. The gate was updated to require the new table-cell markers and
  forbid the old bare text markers.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Separator menu text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test separator_docs_surface
  separator_menu_snippet_routes_section_copy_through_shared_roles --no-fail-fast` failed before the
  fix because `separator/menu.rs` still used local `Theme` plus `ui::text(...).fixed_line_box_px`
  policy.
- `apps/fret-ui-gallery/src/ui/snippets/separator/menu.rs` now routes section titles through
  `text_section_chrome_label(...)` and descriptions through `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test separator_docs_surface
  separator_menu_snippet_routes_section_copy_through_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test separator_docs_surface` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Item slotted text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  item_snippets_route_slotted_copy_through_shared_text_roles --no-fail-fast` failed before the fix
  because `item/dropdown.rs` still used local `ui::text("Select").text_sm()` for a custom trigger
  child.
- `apps/fret-ui-gallery/src/ui/snippets/item/dropdown.rs` now routes the custom trigger child
  through `text_button_label(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/item/gallery.rs` now routes the download header through
  `text_section_chrome_label(...)` and issue-number side columns through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  item_snippets_route_slotted_copy_through_shared_text_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Spinner amount readout text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  spinner_item_amount_text_uses_control_readout_role --no-fail-fast` failed before the fix because
  `spinner/demo.rs` did not import shared text roles and still rendered the payment amount through
  `ui::text("$100.00")`.
- `apps/fret-ui-gallery/src/ui/snippets/spinner/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/spinner/rtl.rs` now route item amount/status values through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  spinner_item_amount_text_uses_control_readout_role --no-fail-fast` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery AvatarStack direction label text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_avatar_stack_direction_labels_use_shared_chrome_text_role --no-fail-fast` failed
  before the fix because `shadcn_extras/avatar_stack.rs` did not import shared text roles and still
  rendered direction labels through `ui::text("LTR").font_medium()`.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/avatar_stack.rs` now routes LTR/RTL
  direction labels through `text_section_chrome_label(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_avatar_stack_direction_labels_use_shared_chrome_text_role --no-fail-fast` timed out
  without a capturable result while Cargo/Rustc processes continued; those processes were allowed
  to finish before retrying.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_avatar_stack_direction_labels_use_shared_chrome_text_role --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Kanban card title text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_kanban_card_titles_use_shared_button_label_role --no-fail-fast` failed before the
  fix because `shadcn_extras/kanban.rs` did not import shared text roles and still rendered card
  titles through local `ui::text(item.name.clone()).font_medium().truncate()` policy.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/kanban.rs` now routes the app-owned Kanban
  card title slot through `text_button_label(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_kanban_card_titles_use_shared_button_label_role --no-fail-fast` timed out without a
  capturable result while Cargo/Rustc processes continued; those processes were allowed to finish
  before retrying.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_kanban_card_titles_use_shared_button_label_role --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 Shadcn Extras AnnouncementTitle composable-title slice:

- Decision: do not mechanically rewrite
  `AnnouncementTitle::new([cx.text("Shadcn Extras landed in Fret")])` at the gallery call site.
  Upstream `repo-ref/kibo/packages/announcement/index.tsx` keeps `AnnouncementTitle` children-first
  while applying `truncate` on the title container, so Fret keeps the composable surface and moves
  the resize contract into `fret-ui-shadcn`.
- `ecosystem/fret-ui-shadcn/src/extras/announcement.rs` now gives `AnnouncementTitle` a
  shrinkable/min-width-zero clipped title container, scopes medium `text-sm` inherited typography,
  and recursively forces nested text children to single-line ellipsis.
- `apps/fret-ui-gallery/src/ui/snippets/shadcn_extras/announcement.rs` intentionally keeps the raw
  composable title call; the gallery source test and IMUI workstream gate prevent it from being
  mistaken for an unowned fixed-row text role.
- First `cargo nextest run -p fret-ui-shadcn
  announcement_title_keeps_composable_children_on_truncated_title_contract --no-fail-fast`
  attempts timed out while cold Cargo/Rustc compilation continued in the background; those
  processes were allowed to finish before retrying. A later unscoped nextest attempt failed during
  test-list enumeration with Windows `os error 740` from an unrelated
  `extras_relative_time_auto_update` integration-test executable, so the component gate was scoped
  to `--lib`.
- `cargo nextest run -p fret-ui-shadcn --lib
  announcement_title_keeps_composable_children_on_truncated_title_contract --no-fail-fast` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  shadcn_extras_announcement_title_keeps_composable_title_surface --no-fail-fast` passed after an
  earlier cold-compile timeout was allowed to finish.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `PYTHONIOENCODING=utf-8 python tools\gate_imui_workstream_source.py` passed.

2026-05-19 gallery ContextMenu trigger text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` failed before the fix
  because `context_menu/demo.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/context_menu/demo.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/basic.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/submenu.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/shortcuts.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/groups.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/icons.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/checkboxes.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/radio.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/destructive.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/sides.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/context_menu/rtl.rs` now route dashed trigger copy through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- A parallel post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` attempt printed PASS but
  hit the tool timeout while contending with Cargo locks from `cargo check`.
- Retried after Cargo locks cleared:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  context_menu_trigger_copy_uses_shared_readout_text_role --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery ScrollArea visible text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface
  scroll_area_snippets_route_visible_text_through_shared_roles --no-fail-fast` failed before the fix
  because `scroll_area/demo.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/scroll_area/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/rtl.rs` route fixed tag/RTL rows through
  `text_list_row_label(...)` and headings through `text_section_chrome_label(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/scroll_area/usage.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/compact_helper.rs` route body copy through
  `text_paragraph(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/scroll_area/horizontal.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/scroll_area/nested_scroll_routing.rs` route figure/card
  captions through `text_control_readout(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface
  scroll_area_snippets_route_visible_text_through_shared_roles --no-fail-fast` timed out while a
  background Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test scroll_area_docs_surface
  scroll_area_snippets_route_visible_text_through_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test scroll_area_docs_surface` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Drawer goal/diagnostics text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_remaining_custom_text_uses_shared_roles --no-fail-fast` failed before the fix because
  `drawer/demo.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/demo.rs` and
  `apps/fret-ui-gallery/src/ui/snippets/drawer/rtl.rs` now route goal readout and unit labels
  through `text_control_readout(...)`.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/nested.rs` now routes guidance copy through
  `text_paragraph(...)`; `apps/fret-ui-gallery/src/ui/snippets/drawer/outside_press.rs` routes
  deterministic probe description through `text_paragraph(...)` and activation count through
  `text_control_readout(...)`.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_remaining_custom_text_uses_shared_roles --no-fail-fast` passed.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.
- `python tools\gate_imui_workstream_source.py` passed after tightening the source gate to use
  stable rustfmt-resistant anchors for the multiline readout call.

2026-05-19 gallery Drawer scroll/side text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_scroll_and_side_body_text_uses_shared_roles --no-fail-fast` failed before the fix because
  `drawer/scrollable_content.rs` was missing the shared `decl_text` role import.
- `apps/fret-ui-gallery/src/ui/snippets/drawer/scrollable_content.rs` now routes scroll rows
  through `text_list_row_label(...)`, and `apps/fret-ui-gallery/src/ui/snippets/drawer/sides.rs`
  routes side body copy through `text_paragraph(...)`.
- `cargo fmt -p fret-ui-gallery` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_scroll_and_side_body_text_uses_shared_roles --no-fail-fast` timed out while a background
  Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  drawer_scroll_and_side_body_text_uses_shared_roles --no-fail-fast` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Pagination text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_pagination_page_number_helpers_use_shared_button_label_role --no-fail-fast` failed
  before the fix because pagination page-number helpers still used no-context
  `ui::text(...).tabular_nums()` builders and the RTL snippet used bare
  `cx.text(to_arabic_numerals(...))`.
- `apps/fret-ui-gallery/src/ui/snippets/pagination/demo.rs`,
  `compact_builder.rs`, `custom_text.rs`, `simple.rs`, `usage.rs`, `routing.rs`, `extras.rs`, and
  `rtl.rs` now route page labels through `decl_text::text_button_label(...)` via a context-bound
  helper. `extras.rs` also routes Fret-specific explanatory copy through `text_paragraph(...)`.
- `ecosystem/fret-ui-shadcn/src/pagination.rs` routes `PaginationPrevious` and `PaginationNext`
  visible labels through shared `text_button_label(...)` instead of bare `cx.text(...)`.
- `cargo fmt -p fret-ui-gallery -p fret-ui-shadcn` passed.
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_pagination_page_number_helpers_use_shared_button_label_role --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- First
  `cargo nextest run -p fret-ui-shadcn pagination_root_is_w_full_and_labeled
  pagination_content_and_item_emit_list_semantics pagination_link_active_stamps_selected
  pagination_link_without_action_keeps_enabled_visual_chrome pagination_disabled_link_wraps_in_opacity
  --no-fail-fast` attempt timed out at 304s while a background Cargo/Rustc compile continued.
- The timed-out Cargo/Rustc process group from that attempt was stopped after it continued compiling
  without a capturable result.
- `cargo check -p fret-ui-shadcn --lib` passed as the direct compile gate for the changed shadcn
  Pagination implementation.
- `cargo fmt --check -p fret-ui-gallery -p fret-ui-shadcn` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 gallery Carousel status/readout text-role slice:

- Red repro:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_carousel_status_readouts_use_shared_control_readout_role --no-fail-fast` failed before
  the fix because `carousel/api.rs` was missing the shared `decl_text` role import and still built
  status text through ad-hoc `TextProps`.
- `apps/fret-ui-gallery/src/ui/snippets/carousel/api.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/carousel/events.rs`,
  `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_stop_on_focus.rs`, and
  `apps/fret-ui-gallery/src/ui/snippets/carousel/plugin_autoplay_stop_on_last_snap.rs` now route
  centered diagnostic/status lines through `text_control_readout(...)` instead of local
  word-wrapping `TextProps` blocks.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_carousel_status_readouts_use_shared_control_readout_role --no-fail-fast` timed out
  without a capturable result; no background Cargo/Rustc processes remained afterward.
- `cargo fmt -p fret-ui-gallery` passed.
- `cargo check -p fret-ui-gallery --test ui_authoring_surface_default_app` passed.
- Retried after the compile/check path completed:
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app
  selected_carousel_status_readouts_use_shared_control_readout_role --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI Attachments inline hover-card text-role slice:

- Source gap before fix: `attachments_inline.rs` did not import shared text roles and still
  rendered hover-card attachment labels/media types through default `ui::text(...)` builders. This
  slice added the focused source test and gate marker with the implementation change.
- `apps/fret-ui-gallery/src/ui/snippets/ai/attachments_inline.rs` now routes hover-card attachment
  labels through `decl_text::text_list_row_label(...)` and media-type values through
  `decl_text::text_control_readout(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_attachments_inline_hover_card_uses_shared_text_roles --no-fail-fast` timed out while a
  background Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_attachments_inline_hover_card_uses_shared_text_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI PlanContent text-role slice:

- Source gap before fix: `plan_demo.rs` outer title/body already used shared roles, but inner
  PlanContent still rendered section headings, bullet rows, and the custom Build button child
  through local `ui::text(...)` styling. The overview body also carried local `text_sm().wrap(...)`
  paragraph policy instead of the shared paragraph role.
- `apps/fret-ui-gallery/src/ui/snippets/ai/plan_demo.rs` now routes inner section headings through
  `decl_text::text_section_chrome_label(...)`, overview body copy through
  `decl_text::text_paragraph(...)`, bullet rows through `decl_text::text_list_row_label(...)`, and
  the custom Build child through `decl_text::text_button_label(...)`.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_and_plan_snippets_use_shared_outer_text_roles --no-fail-fast` timed out while a
  background Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- Retried after the compile finished; the test failed once because the source-test marker expected
  the runtime-concatenated long paragraph string while `include_str!` sees Rust's split source
  literal. The marker was corrected to source-level stable fragments.
- Final focused test:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_and_plan_snippets_use_shared_outer_text_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI PromptInput cursor custom-text slice:

- Source gap before fix: `prompt_input_cursor_demo.rs` still used local `ui::text(...)` styling for
  command item labels, filenames/paths, hover-card rules text, trigger counts, and the tabs footer
  readout. Those are fixed chrome/readout/identifier roles rather than free-form text capability
  probes.
- `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_cursor_demo.rs` now routes those custom
  child text nodes through shared list-row, code-label, control-readout, section-chrome, and
  button-label roles.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_input_cursor_custom_text_uses_shared_roles --no-fail-fast` timed out while a background
  Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- `cargo fmt -p fret-ui-gallery` passed and applied the expected rustfmt wrapping.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_prompt_input_cursor_custom_text_uses_shared_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI Shimmer demo chrome text-role slice:

- Source gap before fix: Shimmer typography/duration/elements demos still used local
  `ui::text(...)` styling for demo labels and inline non-shimmer text. These nodes are gallery
  chrome/readout text, not the Shimmer text capability itself.
- `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer_typography_demo.rs`,
  `shimmer_duration_demo.rs`, and `shimmer_elements_demo.rs` now route demo labels through
  `decl_text::text_control_readout(...)`; the inline non-shimmer prefix/suffix in
  `shimmer_elements_demo.rs` uses section-chrome/control-readout roles. `Shimmer::new(...)` calls
  remain as the explicit animated text capability surface.
- First post-fix
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_shimmer_demo_chrome_text_uses_shared_roles --no-fail-fast` timed out while a background
  Cargo/Rustc compile continued; Cargo/Rustc processes were allowed to exit.
- `cargo fmt -p fret-ui-gallery` passed and applied the expected rustfmt wrapping.
- Retried after the compile finished:
  `cargo nextest run -p fret-ui-gallery --test ai_visible_text_role_surface
  ai_shimmer_demo_chrome_text_uses_shared_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-ui-gallery` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.
- `rg -n "ui::text\(|cx\.text\(" apps\fret-ui-gallery\src\ui\snippets\ai -g "*.rs"` returned no
  matches after this slice; remaining AI snippet text rendering is component-owned surfaces such as
  `Shimmer::new(...)` or other explicit AI text-capability APIs.

2026-05-19 imui node-graph compatibility title text-role slice:

- Source gap before fix: `apps/fret-examples/src/imui_node_graph_demo.rs` is explicitly a retained
  bridge compatibility proof for `fret-node`, but its fixed title still used local
  `fret_ui_kit::ui::text("imui node-graph compatibility proof").font_semibold()` styling. That
  made the compatibility demo teach an ad-hoc text policy under resize instead of the shared text
  role vocabulary.
- `apps/fret-examples/src/imui_node_graph_demo.rs` now routes the title through
  `compat_section_text(...)`, backed by `decl_text::text_section_chrome_label(...)`. The demo still
  imports the IMUI writer trait through `fret_imui::prelude::UiWriter` and keeps the retained
  bridge posture explicit; no direct `fret_authoring` dependency or new `fret-imui` API was added.
- `apps/fret-examples/tests/imui_node_graph_demo_surface.rs`,
  `tools/gate_imui_facade_teaching_source.py`, and `tools/gate_imui_workstream_source.py` now guard
  the compatibility-only wording, shared section role, and absence of the old local title styling.
- `tools/gate_imui_facade_teaching_source.py` no longer uses the stale exact-count check for
  `) -> fret_ui::element::AnyElement {` in `imui_editor_proof_demo.rs`. The gate now checks the
  actual proof-local shared text role helpers and role calls, so adding legitimate role helpers does
  not force the proof back to a worse shape.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_facade_teaching_source.py tools\gate_imui_workstream_source.py`
  passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-demo --features node-graph-demos-legacy --bin imui_node_graph_demo` passed
  with pre-existing `private_interfaces` warnings from node-graph legacy/domain driver visibility.
- First post-fix `cargo nextest run -p fret-examples --features node-graph-demos-legacy --test
  imui_node_graph_demo_surface imui_node_graph_demo_keeps_compat_title_on_shared_role
  --no-fail-fast` timed out while background Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --features node-graph-demos-legacy --test
  imui_node_graph_demo_surface imui_node_graph_demo_keeps_compat_title_on_shared_role
  --no-fail-fast` passed, with the same pre-existing node-graph legacy/domain
  `private_interfaces` warnings.

2026-05-19 embedded viewport chrome/readout text-role slice:

- Source gap before fix: `apps/fret-examples/src/embedded_viewport_demo.rs` used local
  `ui::text(...)` children for fixed ToggleGroup viewport-size labels and local
  `ui::text(format!(...)).text_sm()` builders for compact target/click/input status lines. Under
  narrow resize those are button/readout chrome, not paragraph text.
- `embedded_viewport_button_label_text(...)` now routes the size labels through
  `decl_text::text_button_label(...)`, and `embedded_viewport_readout_text(...)` routes target,
  click, and last-input status through `decl_text::text_control_readout(...)`.
- `apps/fret-examples/tests/embedded_viewport_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the shared roles and forbid the old local
  `ui::text(...)` builders for those fixed chrome/readout slots.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test embedded_viewport_demo_surface
  embedded_viewport_demo_keeps_fixed_chrome_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-demo --bin embedded_viewport_demo` passed.

2026-05-19 window hit-test probe fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/window_hit_test_probe_demo.rs` had a fixed 44px
  header title and compact diagnostic/status lines built with local
  `ui::text(...).font_semibold().text_sm()`, `ui::text(...).font_monospace().text_sm()`, and
  `ui::text(status).text_sm()` policy. These are fixed chrome/readout slots under resize, not
  paragraph text.
- `window_hit_test_title_text(...)` now routes the header through
  `decl_text::text_section_chrome_label(...)`, `window_hit_test_code_label_text(...)` routes the
  logical-window diagnostic identifier through `decl_text::text_code_label(...)`, and
  `window_hit_test_readout_text(...)` routes status through `decl_text::text_control_readout(...)`.
- `apps/fret-examples/tests/window_hit_test_probe_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- `cargo fmt --check -p fret-examples` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- First post-fix `cargo nextest run -p fret-examples --test window_hit_test_probe_demo_surface
  window_hit_test_probe_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test window_hit_test_probe_demo_surface
  window_hit_test_probe_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-examples --test window_hit_test_probe_demo_surface` passed.

2026-05-19 launcher utility window fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/launcher_utility_window_demo.rs` used local
  `ui::text(...)` builders for the frameless-window drag title, effective-style diagnostic line,
  status readout, and resize-handle glyph. These are fixed chrome/readout/glyph slots under
  resize, not paragraph text.
- `launcher_utility_title_text(...)` now routes the drag title through
  `decl_text::text_section_chrome_label(...)`, `launcher_utility_code_label_text(...)` routes the
  effective-style diagnostic through `decl_text::text_code_label(...)`,
  `launcher_utility_readout_text(...)` routes status through `decl_text::text_control_readout(...)`,
  and `launcher_utility_glyph_text(...)` routes the resize arrow through
  `decl_text::text_chrome_glyph(...)`.
- `apps/fret-examples/tests/launcher_utility_window_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- First post-fix `cargo nextest run -p fret-examples --test launcher_utility_window_demo_surface
  launcher_utility_window_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test launcher_utility_window_demo_surface
  launcher_utility_window_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.

2026-05-19 launcher utility window materials fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/launcher_utility_window_materials_demo.rs` used
  local `ui::text(...)` builders for the material-window title, effective material/style diagnostic
  line, and status readout. These are fixed chrome/readout slots under resize, not paragraph text.
- `launcher_utility_materials_title_text(...)` now routes the title through
  `decl_text::text_section_chrome_label(...)`, `launcher_utility_materials_code_label_text(...)`
  routes the effective-style diagnostic through `decl_text::text_code_label(...)`, and
  `launcher_utility_materials_readout_text(...)` routes status through
  `decl_text::text_control_readout(...)`.
- `apps/fret-examples/tests/launcher_utility_window_materials_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- First post-fix `cargo nextest run -p fret-examples --test
  launcher_utility_window_materials_demo_surface
  launcher_utility_window_materials_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while
  background Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test launcher_utility_window_materials_demo_surface
  launcher_utility_window_materials_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.

2026-05-19 API workbench lite fixed text-role slice:

- Source gap before fix: `apps/fret-examples/src/api_workbench_lite_demo.rs` used local
  `ui::text(...)` builders and a `ThemeSnapshot` parameter on `shell_frame(...)` for app title,
  sidebar labels, first-contact copy, active base URL, and history loading/error/empty states.
  These are fixed app chrome/readout/identifier or paragraph slots, not ad-hoc text policy.
- `api_workbench_section_text(...)`, `api_workbench_readout_text(...)`,
  `api_workbench_code_label_text(...)`, and `api_workbench_paragraph_text(...)` now bridge the
  app-facing `AppRenderContext` to shared declarative text roles through `cx.elements()`.
- The app title/sidebar labels route through `decl_text::text_section_chrome_label(...)`,
  first-contact copy routes through `decl_text::text_paragraph(...)`, the active base URL routes
  through `decl_text::text_code_label(...)`, and history loading/error/empty states route through
  `decl_text::text_control_readout(...)`.
- The old `shell_frame` `ThemeSnapshot` parameter was removed because it only existed to support
  local muted text color policy.
- `apps/fret-examples/tests/api_workbench_lite_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard those role mappings and forbid the old local fixed
  text policy from returning.
- First post-fix `cargo nextest run -p fret-examples --test api_workbench_lite_demo_surface
  api_workbench_lite_demo_keeps_fixed_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test api_workbench_lite_demo_surface
  api_workbench_lite_demo_keeps_fixed_text_on_roles --no-fail-fast` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-demo --bin api_workbench_lite_demo` passed.

2026-05-19 hello counter status/help text-role slice:

- Source gap before fix: `apps/fret-examples/src/hello_counter_demo.rs` used local text policy for
  the compact status line and the step help copy. Those are resize-sensitive readout/body slots,
  not text-rendering capability probes.
- `hello_counter_status_text(...)` now routes the status line through
  `decl_text::text_control_readout(...)`, and `hello_counter_paragraph_text(...)` routes the step
  help copy through `decl_text::text_paragraph(...)`.
- The large `ui::text(count.to_string()).text_size_px(Px(72.0))` counter display intentionally
  remains local for this slice. It is a visual display value, not a compact control readout; forcing
  it onto the existing compact role would encode the wrong semantics before a dedicated large
  readout/display-number role exists.
- `apps/fret-examples/tests/hello_counter_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the shared status/help roles, forbid the old local
  status/help builders, and preserve the explicit large counter display exception.
- First post-fix `cargo nextest run -p fret-examples --test hello_counter_demo_surface
  hello_counter_demo_keeps_status_and_help_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test hello_counter_demo_surface
  hello_counter_demo_keeps_status_and_help_text_on_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-examples --lib` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 simple todo visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/simple_todo_demo.rs` used local
  `ui::text(...)` builders for the title summary, empty-state body copy, footer remaining-count
  readout, and todo row labels. This is an ordinary app proof, not a text rendering capability
  surface, so fixed/list/chrome text should not own ad-hoc wrap/truncate policy.
- `simple_todo_readout_text(...)` now routes summary/footer status through
  `decl_text::text_control_readout(...)`, `simple_todo_compact_paragraph_text(...)` routes the
  empty-state copy through `decl_text::text_compact_paragraph(...)`, and
  `simple_todo_row_label_text(...)` routes todo row labels through
  `decl_text::text_list_row_label(...)`.
- Row done/active foreground remains app-owned state policy via `inherit_foreground(...)`; the
  shared role owns the single-line shrink/truncate layout semantics.
- `apps/fret-examples/tests/simple_todo_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` now guard the helper mapping and require
  `simple_todo_demo.rs` to stay free of `ui::text(...)` residuals.
- `cargo nextest run -p fret-examples --test simple_todo_demo_surface
  simple_todo_demo_keeps_visible_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo --bin simple_todo_demo` passed.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 todo demo visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/todo_demo.rs` still used local `ui::text(...)`
  builders for title, status, progress readouts, empty-state labels, filter labels, and active row
  labels, plus local `ui::rich_text(...)` layout policy for completed rows with strikethrough. This
  is a responsive app proof, not a text-rendering capability surface.
- `ecosystem/fret-ui-kit/src/declarative/text.rs` now exposes
  `text_list_row_label_attributed(...)`, an attributed-text variant of the shared list-row label
  role. It keeps fill-width, shrinkable, single-line ellipsis semantics while allowing row labels
  to carry per-span decoration such as strikethrough.
- `todo_demo` now routes title/status/progress/empty/filter/row labels through local helpers backed
  by shared `decl_text` roles: chrome title, control readout, compact paragraph, button label,
  list-row label, and attributed list-row label. Done/active foreground and strikethrough remain
  app state/decoration policy; fixed-row text layout is role-owned.
- `apps/fret-examples/tests/todo_demo_surface.rs` and `tools/gate_imui_workstream_source.py` guard
  the role mapping and require `todo_demo.rs` to stay free of local `ui::text(...)`,
  `ui::rich_text(...)`, and `typography::` text layout policy.
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the
  attributed list-row derived role and focused gate.
- `cargo nextest run -p fret-ui-kit --lib
  attributed_list_row_label_text_uses_fill_width_single_line_truncation --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test todo_demo_surface
  todo_demo_keeps_visible_text_on_roles --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo --bin todo_demo` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 async playground visible text-role slice:

- Source gap before fix: `apps/fret-examples/src/async_playground_demo.rs` used local
  `ui::text(...)` builders for fixed app chrome, catalog item labels, inspector status/policy
  readouts, policy switch labels, query parameter guidance, query identifiers, and result body
  text. Several of those slots also carried local muted color, weight, or truncation policy.
- `async_chrome_title_text(...)`, `async_section_text(...)`, `async_list_row_text(...)`,
  `async_readout_text(...)`, `async_code_label_text(...)`, and
  `async_compact_paragraph_text(...)` now route the proof through shared declarative text roles.
  Fixed chrome/control rows stay single-line and shrinkable; result/guidance body copy uses the
  compact paragraph role where wrapping is intentional.
- The query panel helper chain no longer takes `ThemeSnapshot` just to style text. Theme ownership
  remains where it still controls panel/card/background chrome; text resize semantics are owned by
  the role helpers.
- `apps/fret-examples/tests/async_playground_demo_surface.rs` and
  `tools/gate_imui_workstream_source.py` guard the role mapping and forbid the old local
  `ui::text(...)`, weight/truncate, muted text-color, and redundant theme-parameter patterns from
  returning.
- First post-fix `cargo nextest run -p fret-examples --test async_playground_demo_surface
  async_playground_demo_keeps_visible_text_on_roles --no-fail-fast` timed out while background
  Cargo/Rustc compilation continued.
- Retried after Cargo/Rustc exited:
  `cargo nextest run -p fret-examples --test async_playground_demo_surface
  async_playground_demo_keeps_visible_text_on_roles --no-fail-fast` passed.
- `cargo fmt --check -p fret-examples` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 residual builder-text capability gate tightening:

- Source gap before fix: `apps/fret-examples/tests/text_role_residual_surface.rs` counted
  `cx.text(...)`, `TextProps::new(...)`, and direct `TextProps { ... }` residuals, but missed
  builder-style `ui::text(...)` / `ui::rich_text(...)` calls. That left the proof-app text-role
  contract weaker than the current resize-semantics policy.
- `text_role_residual_surface` now counts `ui::text(...)` and `ui::rich_text(...)` too. The
  remaining `ui::text(...)` entries in `fret-examples` are explicitly documented as
  capability/display payloads: `hello_counter_demo`'s large numeric display and
  `hello_world_compare_demo`'s GPUI/Fret comparison title.
- `docs/workstreams/imui-imgui-gap-closure-v1/P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the
  current allowed residual classes, including text/IME/conformance/rendering probes and the two
  display/performance payload exceptions.
- `tools/gate_imui_workstream_source.py` now requires the stricter residual test shape so the
  gate cannot drift back to missing builder-style text construction.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `git diff --check` passed.

2026-05-19 query detail raw-text role cleanup:

- Source gap before fix: `query_demo.rs` and `query_async_tokio_demo.rs` used `ui::raw_text(...)`
  for query detail rows. `text_role_residual_surface` did not count `ui::raw_text(...)`,
  `ui::text_block(...)`, or all `cx.text_props(...)` calls, so proof apps could bypass the text-role
  contract even after builder-style `ui::text(...)` started being counted.
- `query_readout_text(...)`, `query_readout_text_with_color(...)`, and `query_data_text(...)` now
  route the query detail rows through shared `decl_text` roles. Status/error/duration/retry lines
  use `text_control_readout(...)`; fetched data values use `text_code_label(...)`; destructive or
  muted foreground remains app-owned state policy through `inherit_foreground(...)`.
- `imui_editor_proof_demo` deleted the old `EditorCompactReadoutStyle::text_props(...)` readout
  helper path and now uses the shared control-readout role for those proof readouts.
- `text_role_residual_surface` now counts `ui::raw_text(...)`, `ui::text_block(...)`, and all
  `cx.text_props(...)` calls in addition to the earlier residual counters.
- `apps/fret-examples/tests/query_demo_surface.rs` and `tools/gate_imui_workstream_source.py` guard
  the query role mapping and forbid the old raw-text policy from returning.
- First post-fix parallel nextest runs for `query_demo_surface` and `text_role_residual_surface`
  timed out while background Cargo/Rustc compilation continued. Retried after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test query_demo_surface
  query_demos_keep_detail_text_on_roles --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo nextest run -p fret-examples proof_drag_preview_card_uses_single_line_text_roles
  --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `git diff --check` passed.

2026-05-19 custom effect overlay chrome text-role slice:

- Source gap before fix: `custom_effect_v1_demo.rs` and `custom_effect_v2_demo.rs` were still in the
  direct-text residual allowlist for a single fixed overlay pill label. Those labels are ordinary
  fixed chrome over an effect preview, not text-rendering capability probes.
- Both demos now use `custom_effect_label_text(...)`, backed by
  `decl_text::text_section_chrome_label(...)`, and preserve the white overlay foreground with
  `inherit_foreground(...)`. Custom effect ABI/runtime ownership is unchanged.
- `apps/fret-examples/tests/custom_effect_overlay_text_surface.rs`,
  `apps/fret-examples/tests/text_role_residual_surface.rs`, and
  `tools/gate_imui_workstream_source.py` guard the migration and remove the two demos from the
  residual direct text allowlist.
- First post-fix `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v1_v2_overlay_labels_use_shared_chrome_role --no-fail-fast` timed out while
  background Cargo/Rustc compilation continued. Retried after Cargo/Rustc exited.
- `cargo fmt --check -p fret-examples` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo nextest run -p fret-examples --test custom_effect_overlay_text_surface
  custom_effect_v1_v2_overlay_labels_use_shared_chrome_role --no-fail-fast` passed.
- `cargo nextest run -p fret-examples --test text_role_residual_surface
  remaining_bare_text_in_fret_examples_is_explicit_capability_surface --no-fail-fast` passed.
- `cargo check -p fret-examples --lib` passed.
- `cargo check -p fret-demo` passed with an existing unrelated warning in
  `apps/fret-demo/src/bin/wgpu_hello_world_control.rs` about an unused `Result`.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `git diff --check` passed.

2026-05-19 AI Agent text-role and Accordion role-preservation slice:

- Source gap before fix: the real `fret-ui-ai` `Agent` component still carried local `TextProps`
  for header names, section labels, instruction body copy, and output labels; tool trigger
  descriptions used bare `cx.text(...)`. The shadcn `AccordionTrigger` recipe then unconditionally
  rewrote text children to wrapping trigger text, so a caller-supplied shared role could still lose
  its single-line resize contract.
- `AgentHeader` now routes the name through `decl_text::text_chrome_title(...)`;
  `AgentInstructions`, `AgentTools`, and `AgentOutput` route fixed labels through
  `text_section_chrome_label(...)`; instruction body copy uses `text_compact_paragraph(...)`; and
  `AgentTool` trigger descriptions use `text_list_row_label(...)`.
- `AccordionTrigger` now applies its wrapping trigger defaults only to bare text children. Children
  that already carry inherited text-role metadata keep their role-owned style, wrap, overflow, and
  inherited metadata; hover underline conversion also preserves that metadata.
- `tools/gate_imui_workstream_source.py` now guards the Agent role mapping and the Accordion
  role-preservation markers.
- `cargo fmt --check -p fret-ui-ai -p fret-ui-shadcn` passed.
- `cargo nextest run -p fret-ui-ai agent_header_label_uses_chrome_title_text_role
  agent_default_text_uses_shared_resize_roles
  agent_tools_multiple_uncontrolled_renders_label_and_item_text --no-fail-fast` passed.
- Unbounded `cargo nextest run -p fret-ui-shadcn ...` listed all package test binaries and hit an
  unrelated `extras_relative_time_auto_update` elevation manifest (`os error 740`). Retried with
  the intended lib-only scope.
- `cargo nextest run -p fret-ui-shadcn --lib
  accordion_trigger_label_defaults_do_not_force_foreground_color
  accordion_trigger_label_defaults_preserve_shared_text_role_contracts
  accordion_trigger_hover_underline_preserves_shared_text_role_metadata
  accordion_trigger_label_defaults_keep_bare_text_wrapping_policy --no-fail-fast` passed.
- `cargo check -p fret-ui-ai --lib` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-19 shadcn Table role-preservation slice:

- Source gap before fix: `TableCell` / `TableHead` could still rewrite shared text-role children
  by setting leaf `TextStyle`, `TextWrap::None`, and `TextOverflow::Clip` as recipe defaults. That
  meant a caller could correctly use `text_table_cell(...)` and still lose its single-line
  ellipsis resize contract under shadcn table composition.
- `apply_table_cell_text_defaults(...)`, `apply_table_inherited_text_style(...)`,
  `apply_table_footer_inherited_style(...)`, and `apply_table_head_inherited_style(...)` now treat
  inherited text-role metadata as a protected role scope. Recipe defaults continue to apply to bare
  text, but shared role children keep their role-owned style, wrap, overflow, and inherited
  metadata.
- `table_cell_preserves_shared_text_role_contracts` and
  `table_head_children_preserve_shared_text_role_contracts` cover the role-child path; the existing
  bare-text tests remain in the same focused run to prove table defaults still apply where intended.
- `tools/gate_imui_workstream_source.py` now guards the Table role-preservation helper shape and
  tests so the recipe cannot drift back to unconditional leaf typography rewrites.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib table_cell_preserves_shared_text_role_contracts
  table_head_children_preserve_shared_text_role_contracts
  table_applies_text_sm_defaults_to_unstyled_text_cells
  table_head_children_apply_header_typography_to_plain_text --no-fail-fast` failed because both
  role-preservation tests observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib table_cell_preserves_shared_text_role_contracts
  table_head_children_preserve_shared_text_role_contracts
  table_applies_text_sm_defaults_to_unstyled_text_cells
  table_head_children_apply_header_typography_to_plain_text --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-27 IMUI multi-select state owner split:

- Claim verified: `ImUiMultiSelectState` storage, ordered-selection normalization, anchor repair,
  and crate-local selection mutation helpers moved from `multi_select.rs` into
  `multi_select/state.rs` without changing the public collection helper API.
- `ecosystem/fret-ui-kit/src/imui/multi_select.rs` now keeps model hook, selectable response wiring,
  click-modifier policy, and response changed reporting.
- `tools/gate_imui_workstream_source.py` now covers `multi_select/state.rs` in the opaque-struct
  catalog and rejects the state body or selection normalization from drifting back into the root.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit`,
  `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo nextest run -p fret-ui-kit --features imui --lib multi_select::tests --no-fail-fast`,
  `cargo fmt -p fret-ui-kit --check --verbose`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\check_workstream_catalog.py`, and `git diff --check`.
- A first `cargo nextest run -p fret-ui-kit --features imui --lib multi_select::tests
  --no-fail-fast` attempt timed out after 124s while background Cargo/rustc processes kept running.
  Those timeout remnants were stopped before rerunning the same filter with `TMP`/`TEMP` pointed at
  `.fret/tmp`; the rerun passed 6/6 tests.

2026-05-27 IMUI virtual-list runtime/row owner split:

- Claim verified: runtime option projection and row mechanics moved out of
  `virtual_list_controls.rs` without changing the facade virtual-list API or fixed/known/measured
  row clipping semantics.
- `virtual_list_controls/runtime.rs` now owns runtime option projection, measure-mode repair, and
  viewport layout.
- `virtual_list_controls/row.rs` now owns row packing, row test-id suffixing, row-height
  resolution, striped row chrome, and fixed-height overflow clipping.
- `virtual_list_controls.rs` keeps keyed list assembly, focus child mounting, render-range
  tracking, scroll-handle ownership, and list-level semantics.
- Focused gates passed:
  `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `git diff --check`,
  `cargo fmt -p fret-ui-kit`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo nextest run -p fret-ui-kit --features imui --lib virtual_list_controls::tests --no-fail-fast`,
  `cargo fmt -p fret-ui-kit --check --verbose`, and `python tools\check_workstream_catalog.py`.

2026-05-25 IMUI table render/body/header owner split:

- Source gap before fix: `ecosystem/fret-ui-kit/src/imui/table_controls.rs` still owned table
  render policy, row/cell layout helpers, pinned row grouping, horizontal scroll wrapping, sortable
  header behavior, and resize interaction. That kept independent table owners coupled in one large
  IMUI implementation file.
- `ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` now owns table assembly, test-id
  suffixing, palette resolution, and shared cell layout/packing helpers.
- `ecosystem/fret-ui-kit/src/imui/table_controls/body.rs` now owns prepared table cells, pinned row
  grouping, horizontal center-scroll wrapping, and table cell wrapping.
- `ecosystem/fret-ui-kit/src/imui/table_controls/header.rs`,
  `table_controls/header/trigger.rs`, and `table_controls/header/resize.rs` now own sortable/plain
  header behavior, text-role helpers, active-trigger wiring, and column resize interaction.
- No public IMUI table surface changed; the root `table_controls.rs` keeps only table authoring
  collection plus `ImUiTable` / `ImUiTableRow` facade wiring.
- `tools/gate_imui_workstream_source.py` now checks the root/render/body/header module boundaries
  and keeps the horizontal-scroll/header text-role/hidden-column source markers covered.
- Focused gates passed:
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo nextest run -p fret-ui-kit --features imui --lib
  table_header_label_uses_shared_table_cell_text_role
  table_sort_indicator_uses_shared_chrome_glyph_text_role
  hidden_table_columns_do_not_render_header_body_or_response
  horizontal_scroll_option_wraps_unpinned_header_and_body_center_groups --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.
- Related gate drift repair: `apps/fret-ui-gallery/src/ui/previews/pages/harness/ui_kit_list_torture.rs`
  now carries the expected scroll-boundary paragraph text required by the IMUI source gate. The
  focused gallery authoring-surface test passed with
  `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_internal_previews
  harness_ui_kit_list_torture_uses_fixed_row_text_roles --no-fail-fast`.

2026-05-25 IMUI plot adapter proof:

- Source gap before fix: the old retained IMUI plot facade had been deleted correctly, but there was
  no narrow opt-in authoring adapter for current declarative `fret-plot` panels. That kept plot
  authoring pressure unresolved without a safe dependency boundary.
- `ecosystem/fret-plot/Cargo.toml` now exposes `imui = ["ui", "dep:fret-authoring"]`; default
  features remain empty.
- `ecosystem/fret-plot/src/imui.rs` adds thin `UiWriter` helpers that call the existing declarative
  plot panel constructors and immediately `ui.add(element)`.
- `fret-imui` stays thin and does not depend on `fret-plot`; `fret-ui-kit::imui` also does not
  depend on `fret-plot`.
- `tools/gate_imui_workstream_source.py` now freezes the opt-in adapter boundary and forbids plot
  dependencies from the IMUI facade/kit layers.
- Focused gates passed:
  `cargo fmt --check -p fret-ui-kit -p fret-plot`,
  `cargo check -p fret-plot`,
  `cargo check -p fret-plot --features imui`,
  `cargo nextest run -p fret-plot imui_adapter_stays_opt_in_and_declarative_only --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/check_workstream_catalog.py`,
  `python -m py_compile tools\gate_imui_workstream_source.py`, and `git diff --check`.
- `Cargo.lock` now records `fret-authoring` in the `fret-plot` dependency list because the opt-in
  `fret-plot/imui` feature uses `dep:fret-authoring`.

2026-05-25 IMUI ListBox container proof:

- Red run before fix: `cargo nextest run -p fret-imui
  list_box_container_stamps_semantics_scroll_and_hosts_selectables --no-fail-fast` failed because
  `fret-ui-kit::imui` did not expose `ListBoxOptions` or `list_box_with_options`.
- `ecosystem/fret-ui-kit/src/imui/options/containers.rs` now defines `ListBoxOptions` with only
  layout, scroll, label, multiselectable semantics, and diagnostics ids.
- `ecosystem/fret-ui-kit/src/imui/list_box_controls.rs` now owns the private semantic scroll host
  and stamps `SemanticsRole::ListBox` without owning selection, filtering, command package, or
  active-descendant policy.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` and
  `facade_writer/container_wrappers.rs` expose `list_box` / `list_box_with_options` for root writer
  and nested facade usage.
- `ecosystem/fret-imui/src/tests/composition/layout_collections.rs` proves ListBox semantics,
  forwarded scroll/test ids, vertically stacked selectable rows, selected child semantics, real
  vertical scroll range, and no container-owned active descendant.
- Focused gate passed:
  `cargo nextest run -p fret-imui
  list_box_container_stamps_semantics_scroll_and_hosts_selectables --no-fail-fast`.

2026-05-25 IMUI facade basic-items owner split:

- Source gap before fix: `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` still carried the trait
  default bodies for basic text, wrapped text, bullet text, plain separators, and separator text,
  even though similar focused wrapper clusters already lived under `facade_writer/`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/basic_items.rs` now owns those private default
  implementations and delegates text-role policy to `declarative::text`, bullet text policy to
  `bullet_text_controls`, and section separator policy to `separator_text_controls`.
- The public `UiWriterImUiFacadeExt` method names and signatures stayed in
  `facade_writer.rs`; the root trait now only forwards basic-item calls to the owner module.
- `tools/gate_imui_workstream_source.py` now requires the root-forwarding shape and the
  `basic_items.rs` owner markers while forbidding response/focusable policy from drifting into the
  basic-item owner.

2026-05-26 IMUI facade image-items owner split:

- Source gap before fix: `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` still carried the trait
  default body for image-button normalization even though interactive image item policy already
  lived in `image_item_controls.rs`.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/image_items.rs` now owns the private
  `image_item_with_options` and `image_button_with_options` facade forwarding helpers.
- `image_items.rs` keeps only forwarding/default-normalization logic. It does not own pressable
  chrome, response population, or interaction policy; those stay in `image_item_controls.rs`.
- `tools/gate_imui_workstream_source.py` now requires the root forwarding shape and the
  `image_items.rs` owner markers while forbidding pressable/chrome policy from drifting into the
  facade owner.

2026-05-26 IMUI facade command-presentation owner split:

- Source gap before fix: `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` still resolved command
  presentation for `button_command_with_options` and `menu_item_command_with_options` directly in
  the root public trait body.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` now owns the button command
  presentation/default-enabled forwarding path.
- `ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs` now owns the menu command
  presentation/default-enabled/default-shortcut forwarding path.
- `facade_writer.rs` remains the public `UiWriterImUiFacadeExt` method roster and forwards both
  command helpers to the private owner modules. Public method names and signatures stayed unchanged.
- `tools/gate_imui_workstream_source.py` now requires the root forwarding shape, requires the owner
  module command-presentation markers, and forbids command-presentation lookup from drifting back
  into the root trait hub.
- Validation:
  - `cargo fmt --check -p fret-ui-kit`: pass.
  - `python tools\gate_imui_workstream_source.py`: pass.
  - `git diff --check`: pass.
  - `cargo check -p fret-ui-kit --features imui --lib`: pass with the existing `fret-ui`
    `unstable-retained-bridge` unexpected-cfg and `current_effective_opacity` dead-code warnings.
  - `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`: pass; 6 tests.
- Gate note: one first attempt used a nonexistent `imui_menu_smoke` test target and failed during
  test-target selection; the real owner-adjacent smoke run above passed.

2026-05-26 worktree convergence plan:

- Situation before convergence: `F:/SourceCodes/Rust/fret` is on `main` at `09e568ed`, ahead of
  `origin/main` by six shadcn/parity commits, while
  `F:/SourceCodes/Rust/fret-worktrees/imui-imgui-editor-grade-refactor` is at merge-base
  `901aa6bdfd` with a larger dirty IMUI implementation set.
- The integration strategy is recorded in
  `docs/workstreams/imui-imgui-gap-closure-v1/WORKTREE_CONVERGENCE_PLAN_2026-05-26.md`.
- Review outcome: use `main` as the history/integration base, but resolve IMUI content by topic.
  Prefer the IMUI worktree for the completed facade owner split, layout sugar, canonical workbench,
  Demo/Metrics/Debug, style/theme picker, and source-gate closeout coverage. Keep identical
  `fret-plot/imui` and most table owner split files as-is. Treat `facade_writer/image_items.rs` as a
  follow-up unless completed with gate and workstream evidence before checkpointing. The image-items
  slice was completed with gate and evidence coverage before checkpointing.
- Planned convergence checkpoints:
  1. checkpoint closed `main` slices while excluding incomplete `image_items.rs` unless completed,
  2. checkpoint the IMUI worktree,
  3. merge the IMUI branch into `main`,
  4. run the focused gates listed in the convergence plan.


2026-05-24 floating resize snapshot owner split:

- Source gap before fix: `floating_window_on_area.rs` still enumerated each
  `FloatWindowResizeHandle`, looked up resize drag kinds, read active drag snapshots, and derived
  the chrome `resizing` signal from a tuple before calling the resize owner.
- `floating_window_resize.rs` now owns `FloatingWindowResizeSnapshot` and
  `current_resize_snapshot(...)`, keeping handle discovery and drag snapshot shape in the same
  owner as `prepare_resize_state(...)`, `resize_stack_element(...)`, and the resize handle elements.
- `FloatingWindowResizeStateOutput` now carries the collapsed-aware `resizing` signal, so
  `floating_window_on_area.rs` consumes resize owner outputs instead of knowing tuple fields.
- `tools/gate_imui_workstream_source.py` now requires the snapshot helper/owner shape and forbids
  resize handle enumeration, resize-kind lookup, and direct drag snapshot reads from returning to
  `floating_window_on_area.rs`.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

2026-05-24 floating resize handle layout helper:

- Source gap before fix: `resize_handle_element(...)` still repeated the same cursor/inset/size
  assembly across all eight resize handles.
- `resize_handle_layout(...)` now owns that repeated mapping, and
  `resize_handle_element(...)` only consumes the helper output before wiring the pointer-region
  behavior.
- `tools/gate_imui_workstream_source.py` now requires the helper and forbids the old inline
  `match handle` layout assembly from returning to `resize_handle_element(...)`.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo test -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

2026-05-24 floating resize drag application helper:

- Source gap before fix: `prepare_resize_state(...)` still owned the handle-driven size/position
  mutation loop directly.
- `apply_resize_drag(...)` now owns that mutation loop, and `prepare_resize_state(...)` keeps only
  snapshot selection, collapse checks, and pixel snapping.
- `tools/gate_imui_workstream_source.py` now requires the helper and the delegated call site, so
  the drag mutation block cannot quietly move back into `prepare_resize_state(...)`.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo test -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

2026-05-24 floating shell owner split:

- Source gap before fix: `floating_window_on_area.rs` still owned the window frame, title-bar
  container, clipped body, blocker, and resize stack assembly even after the resize snapshot/state
  logic had been extracted.
- `floating_window_shell.rs` now owns that remaining frame/container composition and consumes the
  prepared title row, content, resize size, resize flags, and handle ids.
- `floating_window_on_area.rs` now only wires the prepared owner outputs together and no longer
  builds the shell container tree inline.
- `tools/gate_imui_workstream_source.py` now requires the shell helper and forbids the remaining
  frame/container assembly from returning to `floating_window_on_area.rs`.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo test -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

2026-05-24 floating shell props helper:

- Source gap before fix: `floating_window_shell_element(...)` still built window frame, shell
  column, title-bar container, and clipped-body props inline even after the shell composition moved
  out of `floating_window_on_area.rs`.
- `floating_window_shell.rs` now keeps those construction details behind
  `window_frame_props(...)`, `shell_column_props(...)`, `title_bar_container_props(...)`, and
  `clipped_body_props(...)`; the shell element only composes already-prepared owner outputs.
- `tools/gate_imui_workstream_source.py` now requires those private helpers, so the shell helper
  cannot drift back into one large inline property block unnoticed.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo test -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

2026-05-24 floating title-bar props helper:

- Source gap before fix: `floating_window_title_bar_row(...)` still built row layout,
  drag-surface `PointerRegionProps`, and close-button accessibility/size props inline while also
  owning keyboard, double-click, movement, and close behavior orchestration.
- `floating_window_title_bar_props.rs` now owns `title_bar_row_props(...)`,
  `title_bar_drag_surface_props(...)`, `title_bar_drag_surface_layout(...)`, and
  `title_bar_close_button_props(...)`. `floating_window_title_bar.rs` consumes those helpers while
  retaining the title/close behavior wiring and the shared close-glyph text role.
- `tools/gate_imui_workstream_source.py` now requires the title-bar props helper module and forbids
  the old inline prop builders from returning to `floating_window_title_bar.rs`.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo test -p fret-ui-kit --features imui --lib
  floating_window_close_glyph_uses_shared_chrome_glyph_text_role`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

2026-05-24 floating content props helper:

- Source gap before fix: `floating_window_content_element(...)` still built the scroll layout,
  content container props, and surface layout inline after the content/blocker orchestration moved
  out of `floating_window_on_area.rs`.
- `floating_window_content_props.rs` now owns `content_surface_layout(...)`,
  `content_scroll_layout(...)`, and `content_container_props(...)`; the content element consumes
  those helpers while keeping the pointer/focus orchestration and the public IMUI surface stable.
- `tools/gate_imui_workstream_source.py` now requires the content props helper module and forbids
  the old inline layout/property builders from returning to `floating_window_content.rs`.
- Focused gates passed:
  `cargo fmt -p fret-ui-kit --check`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo nextest run -p fret-imui floating --no-fail-fast`,
  `python tools/gate_imui_workstream_source.py`,
  `python tools/gate_imui_facade_teaching_source.py`,
  `python tools/check_workstream_catalog.py`, and `git diff --check`.

## P5 Fearless Refactor Execution

- Internal owner split landed for floating-window resize/chrome orchestration:
  `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` now owns the resize-handle
  interaction logic, while `ecosystem/fret-ui-kit/src/imui/floating_window_on_area.rs` keeps the
  high-level render flow.
- The public IMUI surface did not change during the split.
- Validation passed:
  - `cargo test -p fret-ui-kit --features imui --lib floating_window_close_glyph_uses_shared_chrome_glyph_text_role`
  - `cargo nextest run -p fret-imui floating --no-fail-fast`
  - `cargo check -p fret-ui-kit --features imui --lib`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/gate_imui_facade_teaching_source.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`
- Notes:
  - `cargo check` still reports the pre-existing `fret-ui` dead-code warning for
    `current_effective_opacity`; this lane did not change that code.

- 2026-05-24 title-bar owner split:
  `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs` now owns the floating-window
  title-row and close-button orchestration, while `floating_window_on_area.rs` keeps the shell,
  content, and resize stack. The split keeps the public IMUI surface unchanged and keeps the close
  glyph on the shared chrome-glyph text role.
  Validation passed:
  - `cargo test -p fret-ui-kit --features imui --lib floating_window_close_glyph_uses_shared_chrome_glyph_text_role`
  - `cargo nextest run -p fret-imui floating --no-fail-fast`
  - `cargo check -p fret-ui-kit --features imui --lib`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/gate_imui_facade_teaching_source.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

- 2026-05-24 content/blocker owner split:
  `ecosystem/fret-ui-kit/src/imui/floating_window_content.rs` now owns the content
  scroll/focus wrapper, and `ecosystem/fret-ui-kit/src/imui/floating_window_blocker.rs` now owns
  the input-blocking overlay. `floating_window_on_area.rs` keeps only the shell that wires title,
  content, blocker, and resize stack together, and the public IMUI surface stayed unchanged.
  Validation passed:
  - `cargo test -p fret-ui-kit --features imui --lib floating_window_close_glyph_uses_shared_chrome_glyph_text_role`
  - `cargo nextest run -p fret-imui floating --no-fail-fast`
  - `cargo check -p fret-ui-kit --features imui --lib`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/gate_imui_facade_teaching_source.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

- 2026-05-24 resize-stack owner split:
  `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` now owns the
  body/blocker/resize-handle stack assembly through `resize_stack_element(...)` and the internal
  `FloatingWindowResizeHandleTestIds` bundle. `floating_window_on_area.rs` no longer enumerates
  resize handles directly; it passes the clipped body, blocker, resize flags, activation policy,
  and handle test ids into the resize owner.
  Validation passed:
  - `cargo fmt -p fret-ui-kit --check`
  - `cargo test -p fret-ui-kit --features imui --lib floating_window_close_glyph_uses_shared_chrome_glyph_text_role`
  - `cargo nextest run -p fret-imui floating --no-fail-fast`
  - `cargo check -p fret-ui-kit --features imui --lib`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/gate_imui_facade_teaching_source.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

- 2026-05-24 resize-state owner split:
  `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs` now owns the resize-state
  clamp/snap/update logic through `prepare_resize_state(...)`. `floating_window_on_area.rs` only
  threads the resulting state into the shell, chrome, and resize-stack owner.
  Validation passed:
  - `cargo fmt -p fret-ui-kit --check`
  - `cargo test -p fret-ui-kit --features imui --lib floating_window_close_glyph_uses_shared_chrome_glyph_text_role`
  - `cargo nextest run -p fret-imui floating --no-fail-fast`
  - `cargo check -p fret-ui-kit --features imui --lib`
  - `python tools/gate_imui_workstream_source.py`
  - `python tools/gate_imui_facade_teaching_source.py`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check`

2026-05-19 shadcn ItemTitle role-preservation slice:

- Source gap before fix: `ItemTitle::new_children(...)` intentionally patched child text leaves to
  apply the title slot contract. That is correct for bare text and ordinary rich text, but it also
  overwrote shared role children such as `text_chrome_title(...)`, even though those already carry
  a title-family role with single-line ellipsis semantics.
- `patch_item_title_text_style_recursive(...)` now delegates to a role-scope-aware helper. It still
  patches bare `Text`, `StyledText`, and `SelectableText` children so custom ItemTitle content gets
  the title fallback, but it skips any subtree that already carries inherited text-role metadata.
- `item_title_children_patch_bare_text_with_title_typography` and the existing
  `item_title_children_patch_rich_text_with_title_typography` cover the strong slot fallback;
  `item_title_children_preserve_shared_text_role_contracts` proves shared title-role children keep
  `style: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the ItemTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  item_title_children_patch_bare_text_with_title_typography
  item_title_children_preserve_shared_text_role_contracts
  item_title_children_patch_rich_text_with_title_typography --no-fail-fast` failed because the
  role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  item_title_children_patch_bare_text_with_title_typography
  item_title_children_preserve_shared_text_role_contracts
  item_title_children_patch_rich_text_with_title_typography --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn SelectLabel menu-group text role slice:

- Source gap before fix: `SelectLabel` rendered fixed select/listbox group labels with local
  `ui::text(...)` sizing, muted foreground, and `nowrap()` policy inside the overlay row renderer.
  That kept menu/select group labels outside the shared text-role vocabulary and made the same
  fixed-row resize policy likely to be duplicated by ContextMenu, DropdownMenu, and Menubar.
- `fret-ui-kit::declarative::text::text_menu_group_label(...)` now owns the muted `text-xs`,
  fill-width, shrinkable, single-line ellipsis contract for non-interactive menu/listbox group
  headings. It is a derived role, not a control readout.
- `SelectLabel` now consumes `decl_text::text_menu_group_label(...)` while keeping its existing
  entry data model cloneable. This avoids pushing move-only `AnyElement` children into
  `SelectEntry` just to fix text policy.
- `menu_group_label_text_uses_muted_xs_single_line_truncation` proves the role contract; existing
  Select tests keep label/separator focus positions and group semantics in the focused run; and
  `tools/gate_imui_workstream_source.py` guards the role helper plus the Select consumption path.
- Red run before fix:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  menu_group_label_text_uses_muted_xs_single_line_truncation --no-fail-fast` failed because
  `text_menu_group_label(...)` did not exist.
- Post-fix focused runs passed:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  menu_group_label_text_uses_muted_xs_single_line_truncation --no-fail-fast`; and
  `cargo nextest run -p fret-ui-shadcn --lib
  select_label_and_separator_do_not_affect_positions_or_initial_focus
  select_group_renders_group_semantics_node --no-fail-fast`.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-kit --features imui --lib` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn menu-family group-label text role slice:

- Source gap before fix: `DropdownMenuLabel`, `ContextMenuLabel`, and `MenubarLabel` still rendered
  non-interactive menu group headings with local `ui::text(...)` sizing, foreground, line-height,
  and `nowrap()` policy. That duplicated the `SelectLabel` fixed-row text contract and left three
  menu-family surfaces open to resize drift.
- Each menu component now has a local `*_label_element(...)` helper that owns the row container
  padding/inset shape while delegating the text leaf to
  `fret-ui-kit::declarative::text::text_menu_group_label(...)`. Menu item labels, shortcuts,
  submenu chevrons, and icon/indicator foreground policy remain menu-owned.
- Focused unit tests prove all three helpers produce shared role text leaves with no leaf-local
  `TextStyle`/color, fill-width shrinkable layout, `TextWrap::None`, and ellipsis overflow:
  `dropdown_menu_label_element_uses_shared_menu_group_text_role`,
  `context_menu_label_element_uses_shared_menu_group_text_role`, and
  `menubar_label_element_uses_shared_menu_group_text_role`.
- `tools/gate_imui_workstream_source.py` now guards the DropdownMenu, ContextMenu, and Menubar
  helper/import/test shape and rejects the old local group-label `ui::text(...)` builders from
  returning.
- First focused run timed out during compilation after 244 seconds. No cargo/rustc process was
  killed; after the background compile completed naturally, the same focused run was repeated and
  passed.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  dropdown_menu_label_element_uses_shared_menu_group_text_role
  context_menu_label_element_uses_shared_menu_group_text_role
  menubar_label_element_uses_shared_menu_group_text_role --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn CommandGroup heading text role slice:

- Source gap before fix: `CommandGroup::heading(...)` rendered command/listbox group headings in
  both `CommandList` and `CommandPalette` through local `ui::text(...)` builders with command-local
  heading typography, muted foreground, and `nowrap()` policy. That duplicated the same fixed-row
  group-label contract already used by Select and menu-family labels.
- `ecosystem/fret-ui-shadcn/src/command.rs` now uses a private
  `command_group_heading_element(...)` helper for both heading render paths. The helper owns the
  row padding/container shape and delegates the text leaf to
  `fret-ui-kit::declarative::text::text_menu_group_label(...)`.
- This intentionally keeps `CommandItem` label/highlight rendering command-owned. The slice only
  moves non-interactive group heading text policy onto the shared role and indirectly covers
  combobox, native select, and data-table recipe heading consumers through `CommandGroup::heading`.
- `command_group_heading_uses_shared_menu_group_text_role` proves the heading text leaf has no
  leaf-local `TextStyle`/color, uses fill-width/min-width-zero/shrink layout, and stays
  `TextWrap::None` with ellipsis overflow. `tools/gate_imui_workstream_source.py` guards the
  helper/import/test shape and rejects the old command-local heading text helper from returning.
- Planned focused gates for this slice:
  `cargo nextest run -p fret-ui-shadcn --lib
  command_group_heading_uses_shared_menu_group_text_role --no-fail-fast`, `cargo check -p
  fret-ui-shadcn --lib`, `cargo fmt --check -p fret-ui-shadcn`, `python -m py_compile
  tools\gate_imui_workstream_source.py`, `python -m json.tool
  docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`, `python
  tools\gate_imui_workstream_source.py`, and `git diff --check`.
- Verification note: `cargo fmt --check -p fret-ui-shadcn` initially reported the new test block
  formatting around `command_group_heading_uses_shared_menu_group_text_role`; `cargo fmt -p
  fret-ui-shadcn` normalized the block, after which `cargo fmt --check -p fret-ui-shadcn` passed.
  The first focused `cargo nextest run -p fret-ui-shadcn --lib
  command_group_heading_uses_shared_menu_group_text_role --no-fail-fast` attempt timed out while
  Cargo/Rustc still held build locks. No cargo/rustc process was killed; after the background
  compile chain exited naturally, the same focused nextest command passed. `cargo check -p
  fret-ui-shadcn --lib`, `python -m py_compile tools\gate_imui_workstream_source.py`, `python -m
  json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`, `python
  tools\gate_imui_workstream_source.py`, and `git diff --check` passed.

2026-05-20 shared status-message / CommandEmpty text role slice:

- Source gap before fix: shadcn `CommandEmpty` and `CommandLoading` rendered non-interactive
  empty/loading status messages through local `ui::text(...)` builders with duplicated
  text-sm/line-height/muted foreground and `nowrap()` policy. This kept command status messages
  outside the shared resize role vocabulary even after group headings moved to
  `text_menu_group_label(...)`.
- `fret-ui-kit::declarative::text::text_status_message(...)` now owns the muted `text-sm`,
  shrinkable, single-line ellipsis role for non-interactive empty/loading/status messages. It is a
  derived role distinct from `text_menu_group_label(...)` (`text-xs` group headings) and
  `text_control_readout(...)` (`text-xs` compact auxiliary values).
- `CommandEmpty` and `CommandLoading` keep their existing `py-6` container and centered row shape,
  but their text leaves now consume `decl_text::text_status_message(...)`. Command item labels and
  highlighted query spans remain command-owned.
- `status_message_text_uses_muted_sm_single_line_truncation` proves the shared role contract.
  `command_empty_and_loading_use_shared_status_message_text_role` proves both shadcn command status
  consumers use shared-role text leaves with no leaf-local `TextStyle`/color and with
  shrink/nowrap/ellipsis behavior.
- Focused gates passed: `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn`; `cargo nextest run
  -p fret-ui-kit --features imui --lib status_message_text_uses_muted_sm_single_line_truncation
  --no-fail-fast`; `cargo nextest run -p fret-ui-shadcn --lib
  command_empty_and_loading_use_shared_status_message_text_role --no-fail-fast`; `cargo check -p
  fret-ui-kit --features imui --lib`; `cargo check -p fret-ui-shadcn --lib`; `python -m
  py_compile tools\gate_imui_workstream_source.py`; `python -m json.tool
  docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`; `python
  tools\gate_imui_workstream_source.py`; and `git diff --check`.
- Verification note: the first focused shadcn nextest attempt timed out during background
  compilation. No cargo/rustc process was killed; after the compile chain exited naturally, the
  same command passed.

2026-05-20 shadcn DataTable toolbar text role slice:

- Source gap before fix: `DataTableToolbar` still owned local `ui::text(...)` /
  `ui::raw_text(...)` policy for fixed toolbar text: faceted-trigger labels, faceted option
  labels, count chips, clear/reset action labels, and selected-count readouts. Those are fixed-row
  control texts, so resize should truncate them through shared roles instead of allowing local
  builder policy to drift.
- `data_table_toolbar_button_label(...)`, `data_table_toolbar_option_label(...)`, and
  `data_table_toolbar_readout(...)` now keep that recipe-local semantic split while delegating to
  `text_button_label(...)`, `text_list_row_label(...)`, and `text_control_readout(...)`. The
  helper names are intentionally local to the recipe; `fret-imui` stays policy-light and no new
  public text-role enum was added.
- `data_table_toolbar_fixed_text_uses_shared_roles` proves these helper outputs keep leaf
  `style`/`color` empty, carry inherited text-role typography, stay shrinkable with
  `min-width: 0`, and use single-line ellipsis. The readout path also proves muted foreground is
  inherited.
- Focused gates passed: `cargo fmt --check -p fret-ui-shadcn`; `cargo nextest run -p
  fret-ui-shadcn --lib data_table_toolbar_fixed_text_uses_shared_roles --no-fail-fast`; `cargo
  check -p fret-ui-shadcn --lib`; `python -m py_compile tools\gate_imui_workstream_source.py`;
  `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`;
  `python tools\gate_imui_workstream_source.py`; and `git diff --check`.
- Verification note: focused shadcn nextest timed out twice while Cargo/Rustc still held build
  locks. No cargo/rustc process was killed; after each compile chain exited naturally, the same
  focused nextest command passed.

2026-05-20 inherited text feature + DataTable tabular readout slice:

- Source gap before fix: the remaining DataTable pagination footer used local
  `ui::text(...).tabular_nums()` builders because the shared inherited text-role refinement could
  not express OpenType numeric features. That left a resize-sensitive fixed footer row outside the
  shared control-readout contract.
- `TextStyleRefinement` now carries subtree-default OpenType feature settings. `TextStyle::refine`,
  `TextStyleRefinement::merge`, `text_style_refinement_fingerprint(...)`, and
  `fret-ui-kit` typography bridge helpers preserve those feature settings so passive text
  measurement/cache and role helpers see the same numeric shaping policy.
- `text_control_readout_tabular(...)` and `text_control_readout_tabular_emphasis(...)` are narrow
  control-readout variants, not new stable text-role categories. They keep single-line ellipsis,
  `min-width: 0`, inherited role typography, and tabular numeric features; the emphasis variant
  adds the medium page-summary weight.
- `DataTablePagination` now consumes recipe-local `data_table_pagination_readout(...)` and
  `data_table_pagination_summary(...)` helpers backed by those shared variants, closing the last
  local pagination footer text policy without widening `fret-imui`.
- Focused gates passed: `cargo check -p fret-core`; `cargo check -p fret-ui`; `cargo check -p
  fret-ui-kit`; `cargo check -p fret-ui-shadcn`; `cargo nextest run -p fret-core --lib
  text_style_refinement_merges_font_features_in_parent_child_order
  text_style_refine_applies_inherited_font_features_after_leaf_defaults --no-fail-fast`; `cargo
  nextest run -p fret-ui --lib inherited_text_style_features_affect_passive_text_measurement
  inherited_text_style_fingerprint_tracks_feature_overrides --no-fail-fast`; `cargo nextest run -p
  fret-ui-kit --lib control_readout_tabular_text_uses_muted_single_line_truncation
  control_readout_tabular_emphasis_text_uses_medium_single_line_truncation --no-fail-fast`; `cargo
  nextest run -p fret-ui-shadcn --lib data_table_toolbar_fixed_text_uses_shared_roles
  --no-fail-fast`; `python -m py_compile tools\gate_imui_workstream_source.py`; `python -m
  json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`; `python
  tools\gate_imui_workstream_source.py`; `rustfmt --edition 2024 --check
  crates/fret-core/src/text/mod.rs crates/fret-ui/src/text/props.rs
  crates/fret-ui/src/declarative/tests/text_style_inheritance.rs
  ecosystem/fret-ui-kit/src/typography.rs ecosystem/fret-ui-kit/src/declarative/text.rs
  ecosystem/fret-ui-shadcn/src/data_table_recipes.rs`; and `git diff --check`.
- Verification note: broader package-level `cargo fmt --check -p ...` and the first
  `fret-ui-shadcn` focused nextest attempt timed out while Cargo/Rustc still held build locks. No
  cargo/rustc process was killed; after each compile chain exited naturally, focused checks were
  rerun and passed.

2026-05-20 tabular control-readout resize gate slice:

- Source gap before fix: `text_control_readout_tabular(...)` and
  `text_control_readout_tabular_emphasis(...)` had role-local structure tests, but they were not
  included in the shared narrow-layout single-line role gate or the text-role matrix's derived-role
  catalog.
- `base_single_line_text_roles_stay_single_line_under_narrow_layout` now asserts both tabular
  control-readout variants stay one measured line under a narrow resize probe. This keeps numeric
  DataTable/page/footer readouts on the same anti-wrap contract as ordinary readouts, button
  labels, table cells, and code labels.
- `P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the variants as control-readout derivatives
  rather than a sixth stable role, and `tools/gate_imui_workstream_source.py` guards their
  narrow-layout assertions.
- Focused gates passed: `cargo nextest run -p fret-ui-kit --lib
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast`; `cargo fmt
  --check -p fret-ui-kit`; `python tools\gate_imui_workstream_source.py`; `python -m py_compile
  tools\gate_imui_workstream_source.py`; `python -m json.tool
  docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json`; and `git diff --check`.

2026-05-19 shadcn EmptyTitle children-role slice:

- Source gap before fix: `EmptyTitle` only accepted a string payload, so empty-state title slots
  could not carry caller-supplied shared title/chrome role children. That kept empty states outside
  the same explicit-role contract used by overlay, card, item, and field title slots.
- `EmptyTitle` now exposes `new_children(...)`. The string path preserves the existing shadcn
  empty-title defaults, including centered balanced text; the children path patches bare `Text`,
  `StyledText`, and `SelectableText` children with empty-title typography/foreground/centered
  balance fallback, but skips subtrees that already carry inherited text-role metadata.
- `empty_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `empty_title_children_preserve_shared_text_role_contracts` proves a `text_chrome_title(...)`
  child keeps `style: None`, `color: None`, ellipsis overflow, and role metadata; and
  `empty_description_scopes_inherited_text_style` proves description scopes are unchanged.
- `tools/gate_imui_workstream_source.py` now guards the EmptyTitle children API, scoped helper, and
  tests.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  empty_title_children_patch_rich_text_with_title_typography
  empty_title_children_preserve_shared_text_role_contracts --no-fail-fast` failed because
  `EmptyTitle::new_children(...)` did not exist.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  empty_title_children_patch_rich_text_with_title_typography
  empty_title_children_preserve_shared_text_role_contracts
  empty_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn FieldTitle children-role slice:

- Source gap before fix: `FieldTitle` only accepted a string payload, so field/property-panel title
  slots could not carry caller-supplied shared title/chrome role children. That kept field titles
  outside the same explicit-role contract now used by dialog, sheet, popover, alert, card, and item
  titles.
- `FieldTitle` now exposes `new_children(...)`. The string path preserves existing field-title
  defaults and `w-fit` behavior; the children path patches bare `Text`, `StyledText`, and
  `SelectableText` children with field-title typography/foreground/alignment, but skips subtrees
  that already carry inherited text-role metadata.
- `field_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `field_title_children_preserve_shared_text_role_contracts` proves a `text_chrome_title(...)`
  child keeps `style: None`, `color: None`, ellipsis overflow, role metadata, and its own
  fill-width layout; `field_title_and_plain_label_approximate_upstream_w_fit_defaults` keeps the
  existing bare-title `w-fit` contract in the focused run; and
  `field_description_scopes_inherited_text_style` proves description scopes are unchanged.
- `tools/gate_imui_workstream_source.py` now guards the FieldTitle children API, scoped helper, and
  tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  field_title_children_patch_rich_text_with_title_typography
  field_title_children_preserve_shared_text_role_contracts
  field_title_and_plain_label_approximate_upstream_w_fit_defaults
  field_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn PopoverTitle children-role slice:

- Source gap before fix: `PopoverTitle` only accepted a string payload, so popover/panel titles
  could not carry caller-supplied shared title/chrome role children. That left another overlay
  surface behind the title-slot contract now used by dialog, sheet, alert, card, and item titles.
- `PopoverTitle` now exposes `new_children(...)`. The string path preserves existing shadcn
  popover-title defaults; the children path patches bare `Text`, `StyledText`, and
  `SelectableText` children with popover-title typography/foreground, but skips subtrees that
  already carry inherited text-role metadata.
- `popover_title_children_patch_rich_text_with_title_typography` proves the strong fallback
  remains; `popover_title_children_preserve_shared_text_role_contracts` proves a
  `text_chrome_title(...)` child keeps `style: None`, `color: None`, ellipsis overflow, and role
  metadata; and `popover_description_scopes_inherited_text_style` stays in the focused run to prove
  the existing description scope is unchanged.
- `tools/gate_imui_workstream_source.py` now guards the PopoverTitle children API, scoped helper,
  and tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  popover_title_children_patch_rich_text_with_title_typography
  popover_title_children_preserve_shared_text_role_contracts
  popover_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn SheetTitle children-role slice:

- Source gap before fix: `SheetTitle` only accepted a string payload, so sheet titles could not
  carry caller-supplied shared title/chrome role children. That left sheet composition behind the
  dialog/title-slot contract now used by `DialogTitle`, `AlertDialogTitle`, `CardTitle`, and
  `AlertTitle`.
- `SheetTitle` now exposes `new_children(...)`. The string path preserves existing shadcn
  sheet-title defaults; the children path patches bare `Text`, `StyledText`, and `SelectableText`
  children with sheet-title typography/foreground, but skips subtrees that already carry inherited
  text-role metadata.
- `sheet_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `sheet_title_children_preserve_shared_text_role_contracts` proves a `text_chrome_title(...)`
  child keeps `style: None`, `color: None`, ellipsis overflow, and role metadata; and
  `sheet_description_scopes_inherited_text_style` stays in the focused run to prove the existing
  description scope is unchanged.
- `tools/gate_imui_workstream_source.py` now guards the SheetTitle children API, scoped helper, and
  tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  sheet_title_children_patch_rich_text_with_title_typography
  sheet_title_children_preserve_shared_text_role_contracts
  sheet_description_scopes_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn DialogTitle children-role slice:

- Source gap before fix: `DialogTitle` only accepted a string payload, so callers could not provide
  a shared title/chrome role child at all. That made dialog titles an exception to the emerging
  component contract: recipes may own bare-text defaults, but caller-supplied role children must be
  expressible and protected.
- `DialogTitle` now mirrors the other title slots with `new_children(...)`. The text path preserves
  the existing shadcn dialog-title defaults; the children path patches bare `Text`, `StyledText`,
  and `SelectableText` children with dialog-title typography/foreground, but skips subtrees that
  already carry inherited text-role metadata.
- `dialog_title_children_patch_rich_text_with_title_typography` proves the strong fallback remains;
  `dialog_title_children_preserve_shared_text_role_contracts` proves a
  `text_chrome_title(...)` child keeps `style: None`, `color: None`, ellipsis overflow, role
  metadata, and heading semantics; `dialog_description_children_scope_inherited_text_style` stays
  in the focused run to prove the existing description scope is unchanged.
- `tools/gate_imui_workstream_source.py` now guards the DialogTitle children API, scoped helper, and
  tests.
- Focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  dialog_title_children_patch_rich_text_with_title_typography
  dialog_title_children_preserve_shared_text_role_contracts
  dialog_description_children_scope_inherited_text_style --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn AlertDialogTitle role-preservation slice:

- Source gap before fix: `AlertDialogTitle` recursively wrote dialog-title typography, foreground,
  wrapping, and clipping into descendant text leaves. That kept bare dialog titles ergonomic, but it
  also overwrote shared role children such as `text_chrome_title(...)`, even though those already
  carry role-owned title-family resize semantics.
- `patch_alert_dialog_text_style_recursive(...)` now delegates to a role-scope-aware helper. It
  still patches bare `Text`, `StyledText`, and `SelectableText` children so custom
  AlertDialogTitle content gets the shadcn dialog-title fallback, but it skips any subtree that
  already carries inherited text-role metadata.
- `alert_dialog_title_children_patch_rich_text_with_title_typography` keeps the strong title-slot
  fallback; `alert_dialog_description_children_scope_rich_text_with_description_typography` stays
  in the focused run to prove the existing description scope is unchanged; and
  `alert_dialog_title_children_preserve_shared_text_role_contracts` proves shared title-role
  children keep `style: None`, `color: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the AlertDialogTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_dialog_title_children_preserve_shared_text_role_contracts
  alert_dialog_title_children_patch_rich_text_with_title_typography
  alert_dialog_description_children_scope_rich_text_with_description_typography --no-fail-fast`
  failed because the role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_dialog_title_children_preserve_shared_text_role_contracts
  alert_dialog_title_children_patch_rich_text_with_title_typography
  alert_dialog_description_children_scope_rich_text_with_description_typography --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn AlertTitle role-preservation slice:

- Source gap before fix: `AlertTitle` recursively wrote title typography into every descendant
  text leaf. That kept bare alert titles ergonomic, but it also overwrote shared role children such
  as `text_chrome_title(...)`, even though those already carry title-family single-line ellipsis
  semantics.
- `patch_alert_text_style_recursive(...)` now delegates to a role-scope-aware helper. It still
  patches bare `Text`, `StyledText`, and `SelectableText` children so custom AlertTitle content gets
  the shadcn alert-title fallback, but it skips any subtree that already carries inherited text-role
  metadata.
- `alert_title_children_patch_rich_text_with_title_typography` and
  `alert_title_children_preserve_interactive_spans_under_title_scope` keep the strong slot
  fallback; `alert_title_children_preserve_shared_text_role_contracts` proves shared title-role
  children keep `style: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the AlertTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_title_children_preserve_shared_text_role_contracts
  alert_title_children_patch_rich_text_with_title_typography
  alert_title_children_preserve_interactive_spans_under_title_scope --no-fail-fast` failed because
  the role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  alert_title_children_preserve_shared_text_role_contracts
  alert_title_children_patch_rich_text_with_title_typography
  alert_title_children_preserve_interactive_spans_under_title_scope --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn CardTitle role-preservation slice:

- Source gap before fix: `CardTitle::new_children(...)` intentionally patched child text leaves to
  apply the card-title slot contract. That is correct for bare text and ordinary rich text, but it
  also overwrote shared role children such as `text_chrome_title(...)`, even though those already
  carry title-family single-line ellipsis semantics.
- `patch_card_title_text_style_recursive(...)` now delegates to a role-scope-aware helper. It still
  patches bare `Text`, `StyledText`, and `SelectableText` children so custom CardTitle content gets
  the shadcn card-title fallback, but it skips any subtree that already carries inherited text-role
  metadata.
- `card_title_children_patch_bare_text_with_title_typography` and the existing
  `card_title_children_patch_rich_text_with_title_typography` cover the strong slot fallback;
  `card_title_children_preserve_shared_text_role_contracts` proves shared title-role children keep
  `style: None`, ellipsis overflow, and role metadata.
- `tools/gate_imui_workstream_source.py` now guards the CardTitle helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  card_title_children_patch_bare_text_with_title_typography
  card_title_children_preserve_shared_text_role_contracts
  card_title_children_patch_rich_text_with_title_typography --no-fail-fast` failed because the
  role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  card_title_children_patch_bare_text_with_title_typography
  card_title_children_preserve_shared_text_role_contracts
  card_title_children_patch_rich_text_with_title_typography --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn CardDescription children role-preservation gate:

- Source gap before gate: `CardDescription::new_children(...)` scoped description typography on the
  children container, but it did not have a focused regression proving caller-supplied shared
  description/body roles keep their leaf wrap/overflow and inherited role metadata.
- `card_description_children_preserve_shared_text_role_contracts` now proves a
  `text_compact_paragraph_line_clamp(...)` child keeps `style: None`, role metadata,
  word wrapping, and ellipsis overflow when mounted through `CardDescription::new_children(...)`.
- This stays in `fret-ui-shadcn` as recipe/slot policy. It does not add a new text role or widen
  `fret-imui`.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  card_description_children_preserve_shared_text_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command passed.

2026-05-20 shadcn Sheet/Popover description children role-preservation gate:

- Source gap before gate: `SheetDescription` and `PopoverDescription` were text-only leaves, so
  callers could not hand a composed shared paragraph role tree into the description slot without
  wrapping around the component boundary.
- `sheet_description_children_preserve_shared_text_role_contracts` and
  `popover_description_children_preserve_shared_text_role_contracts` now prove a
  `text_compact_paragraph(...)` child keeps `style: None`, wrap, fill-width layout, and inherited
  role metadata when mounted through the description slots.
- This widens the shadcn recipe/API surface only where the component already had a description slot;
  it does not add a new `fret-imui` text role or move policy into `fret-imui`.
- Focused `cargo nextest run -p fret-ui-shadcn --lib
  sheet_description_children_preserve_shared_text_role_contracts
  popover_description_children_preserve_shared_text_role_contracts --no-fail-fast` passed after the
  new children lanes landed.

2026-05-20 shadcn existing description children role-preservation gate:

- Source gap before gate: `AlertDescription`, `DialogDescription`, `AlertDialogDescription`, and
  `ItemDescription` already exposed `new_children(...)`, but their tests only covered inherited
  typography, rich/selectable text, or container layout. They did not prove shared paragraph-role
  children kept their own wrap/layout/overflow metadata under description composition.
- `alert_description_children_preserve_shared_text_role_contracts`,
  `dialog_description_children_preserve_shared_text_role_contracts`,
  `alert_dialog_description_children_preserve_shared_text_role_contracts`, and
  `item_description_children_preserve_shared_text_role_contracts` now prove shared paragraph/body
  roles keep `style: None`, `color: None`, role-owned wrapping/overflow, fill-width/min-width
  layout, and inherited role metadata when passed through the existing description children lanes.
- This is a gate-only contract slice for existing shadcn recipe surfaces. It does not add a new text
  role, widen `fret-imui`, or change description rendering policy.
- Focused `cargo nextest run -p fret-ui-shadcn --lib
  alert_description_children_preserve_shared_text_role_contracts
  dialog_description_children_preserve_shared_text_role_contracts
  alert_dialog_description_children_preserve_shared_text_role_contracts
  item_description_children_preserve_shared_text_role_contracts --no-fail-fast` passed.

2026-05-20 shadcn ButtonGroupText children role-preservation gate:

- Source gap before gate: `ButtonGroupText::new_children(...)` already passed custom children
  through, but the test suite only proved inline child count/layout. It did not prove a
  caller-supplied shared button-label role kept its own single-line shrink/ellipsis contract under
  button-group chrome composition.
- `button_group_text_children_preserve_shared_button_label_role_contracts` now proves a
  `text_button_label(...)` child keeps `style: None`, `color: None`, no wrapping, ellipsis
  overflow, zero minimum width, flex shrink, and inherited role metadata when mounted through
  `ButtonGroupText::new_children(...)`.
- This is a gate-only contract slice for an existing shadcn recipe slot. Component-owned
  `ButtonGroupText::new(...)` label styling remains local chrome policy, and this does not add a
  new text role or widen `fret-imui`.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  button_group_text_children_preserve_shared_button_label_role_contracts --no-fail-fast` timed out
  while Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally,
  the same focused command passed.
- Focused `cargo nextest run -p fret-ui-shadcn --lib
  button_group_text_children_preserve_shared_button_label_role_contracts --no-fail-fast` passed.

2026-05-20 shadcn TabsTrigger role-preservation slice:

- Source gap before fix: `TabsTrigger` recursively wrote trigger typography and foreground into
  descendant passive text leaves whenever `style` / `color` were empty. Shared text role helpers
  intentionally keep leaf `style` and `color` empty and carry typography through
  `inherited_text_style`, so trigger children built with `text_button_label(...)` lost their
  role-owned single-line shrink/ellipsis contract.
- `apply_trigger_inherited_style(...)` now treats `inherited_text_style` as a protected role scope
  and carries that scope through descendants. Bare trigger text still receives the shadcn trigger
  typography/foreground fallback; shared role children keep leaf `style: None`, `color: None`,
  no-wrap, ellipsis overflow, zero minimum width, shrink, and inherited role metadata. Trigger
  foreground still flows through the content root's inherited foreground instead of being stamped
  onto role-owned text leaves.
- `tabs_trigger_applies_default_style_to_bare_label_text` proves the bare-text fallback remains;
  `tabs_trigger_children_preserve_shared_button_label_role_contracts` proves a shared button-label
  role survives `TabsItem::trigger_children(...)`.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  tabs_trigger_applies_default_style_to_bare_label_text
  tabs_trigger_children_preserve_shared_button_label_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command passed.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  tabs_trigger_applies_default_style_to_bare_label_text
  tabs_trigger_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn Toggle/ToggleGroup role-preservation slice:

- Source gap before fix: `Toggle` and `ToggleGroupItem` recursively wrote their resolved
  foreground into descendant passive text leaves. That kept bare custom text ergonomic, but it
  overwrote shared text role children such as `text_button_label(...)`, replacing the role-owned
  leaf color/inherited typography path that controls single-line shrink/ellipsis under narrow
  editor chrome.
- `apply_toggle_inherited_style(...)` and `apply_item_inherited_style(...)` now treat
  `inherited_text_style` as a protected role scope and propagate that scope through descendants.
  Bare text children still receive toggle/toggle-group foreground; shared role children keep
  `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width, shrink, and
  inherited role metadata. Foreground still flows through the content root's inherited foreground.
- `toggle_children_apply_foreground_to_bare_text` and
  `toggle_group_item_children_apply_foreground_to_bare_text` prove the bare-text fallback remains.
  `toggle_children_preserve_shared_button_label_role_contracts` and
  `toggle_group_item_children_preserve_shared_button_label_role_contracts` prove shared
  button-label roles survive explicit toggle/toggle-group children.
- Two early focused runs timed out while Cargo/Rustc was still compiling. No process was killed;
  after Cargo/Rustc exited naturally, the split focused runs passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  toggle_children_apply_foreground_to_bare_text
  toggle_children_preserve_shared_button_label_role_contracts --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  toggle_group_item_children_apply_foreground_to_bare_text
  toggle_group_item_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn Badge role-preservation slice:

- Source gap before fix: `Badge` recursively wrote its resolved variant foreground into leading
  and trailing child text leaves. That preserved color inheritance for bare text, but it overwrote
  shared text role children such as `text_button_label(...)`, replacing the role-owned leaf color
  path that controls single-line shrink/ellipsis in dense badge chrome.
- `apply_badge_inherited_fg(...)` now treats `inherited_text_style` as a protected role scope and
  carries that scope through descendants. Bare child text still receives the badge foreground; shared
  role children keep `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width,
  shrink, and inherited role metadata. Badge foreground still flows through the content root's
  inherited foreground.
- `badge_children_apply_foreground_to_bare_text` proves the bare-text fallback remains, and
  `badge_children_preserve_shared_button_label_role_contracts` proves a shared button-label role
  survives `Badge::leading_children(...)`.
- Early focused runs timed out while Cargo/Rustc was still compiling. No process was killed; after
  Cargo/Rustc exited naturally, the split focused runs passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  badge_children_apply_foreground_to_bare_text --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  badge_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn Button children role-preservation gate:

- Source gap before gate: `Button` already used `current_color::scope_children(...)` for custom
  content instead of recursively rewriting text leaves, but the test suite did not prove
  caller-supplied shared button-label roles kept their leaf text contract through the full
  `children(...)` override path or the `leading_children(...)` / `trailing_children(...)` inline
  slot path.
- `button_children_preserve_shared_button_label_role_contracts` now proves a `text_button_label(...)`
  child keeps `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width, shrink,
  inherited role metadata, and inherited foreground through `Button::children(...)`.
- `button_inline_children_preserve_shared_button_label_role_contracts` proves the same contract for
  `Button::leading_children(...)` and `Button::trailing_children(...)`, while the existing inline
  ordering test continues to cover label preservation and RTL order flipping.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  button_children_preserve_shared_button_label_role_contracts
  button_inline_children_preserve_shared_button_label_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  split focused runs passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  button_children_preserve_shared_button_label_role_contracts --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  button_inline_children_preserve_shared_button_label_role_contracts --no-fail-fast`.

2026-05-20 shadcn TooltipContent role-preservation slice:

- Source gap before fix: `TooltipContent` recursively wrote tooltip foreground, tooltip text style,
  and max-width/min-width hints into descendant text leaves whenever those leaf props were empty.
  Shared text-role helpers intentionally keep leaf `style` and `color` empty while carrying
  typography through `inherited_text_style`, so rich tooltip content built with
  `text_control_readout(...)` could lose its role-owned single-line shrink/ellipsis contract under
  narrow overlay chrome.
- `apply_tooltip_inherited_defaults(...)` now delegates to a scoped helper that treats
  `inherited_text_style` as a protected role scope and propagates that scope through descendants.
  Bare tooltip text still receives tooltip `text-xs`, foreground, and max-width defaults; shared
  role children keep `style: None`, `color: None`, no-wrap, ellipsis overflow, zero minimum width,
  shrink, and inherited role metadata.
- `TooltipContent` now stamps tooltip foreground as inherited foreground on the content root. This
  preserves shadcn `text-background` behavior for shared text roles without mutating role-owned text
  leaves.
- `tooltip_content_applies_default_style_to_bare_text` proves the bare-text fallback remains.
  `tooltip_content_preserves_shared_control_readout_role_contracts` proves a shared control-readout
  role survives `TooltipContent::new(...)` rich content.
- First combined focused runs timed out while Cargo/Rustc was still compiling. No process was
  killed; after Cargo/Rustc exited naturally, the combined focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  tooltip_content_applies_default_style_to_bare_text
  tooltip_content_preserves_shared_control_readout_role_contracts --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-20 shadcn BreadcrumbList role-preservation slice:

- Source gap before fix: primitive `BreadcrumbList` applied list-level muted foreground and
  breadcrumb typography by writing direct `Text` leaf props for each custom child. Shared text-role
  helpers intentionally keep leaf `style` and `color` empty while carrying typography through
  `inherited_text_style`, so a primitive breadcrumb list child built with `text_button_label(...)`
  could lose its role-owned single-line shrink/ellipsis contract under breadcrumb composition.
- `apply_breadcrumb_list_text_style_defaults(...)` now treats `inherited_text_style` as a protected
  role scope and only applies breadcrumb typography to bare text leaves. `BreadcrumbList` now stamps
  the muted foreground through `current_color::scope_children(...)`, so visual color remains
  breadcrumb-list policy without mutating role-owned text leaves.
- `breadcrumb_list_applies_default_style_to_bare_text` proves the bare loose-text fallback remains.
  `breadcrumb_list_preserves_shared_button_label_role_contracts` proves a shared button-label role
  survives primitive `BreadcrumbList::into_element(...)` children.
- First focused run timed out while Cargo/Rustc was still compiling. No process was killed; after
  Cargo/Rustc exited naturally, the combined focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  breadcrumb_list_applies_default_style_to_bare_text
  breadcrumb_list_preserves_shared_button_label_role_contracts --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.

2026-05-20 shadcn AnnouncementTitle role-preservation slice:

- Source gap before fix: raw extras `AnnouncementTitle` owned the correct clipped title container
  and bare-text single-line ellipsis fallback, but its recursive title contract rewrote every
  descendant passive text leaf. Shared text-role helpers intentionally keep leaf `style` and
  `color` empty while carrying typography through `inherited_text_style`, so a custom title child
  built with `text_button_label(...)` could lose its role-owned leaf contract under resize.
- `apply_announcement_title_text_contract_recursive(...)` now delegates to a scoped helper that
  treats `inherited_text_style` as a protected role scope. Title typography is applied to bare text
  leaves instead of the title root, so shared role children are not polluted by parent inherited
  style merging. Bare `cx.text(...)` title children still receive the announcement-title
  single-line ellipsis contract, while shared role children keep `style: None`, `color: None`,
  no-wrap, ellipsis overflow, zero minimum width, shrink, and inherited role metadata.
- `announcement_title_keeps_composable_children_on_truncated_title_contract` proves the existing
  bare/composable title contract remains. `announcement_title_preserves_shared_button_label_role_contracts`
  proves a shared button-label role survives `AnnouncementTitle::new(...)` children.
- First focused `cargo nextest run -p fret-ui-shadcn --lib
  announcement_title_keeps_composable_children_on_truncated_title_contract
  announcement_title_preserves_shared_button_label_role_contracts --no-fail-fast` timed out while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command passed.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-20 shadcn SidebarGroupLabel resize text-role slice:

- Source gap before fix: `SidebarGroupLabel` rendered its default fixed-height label with local
  `ui::text(...).text_size_px(...).line_height_px(...).font_medium().text_color(...).wrap(TextWrap::Word)`.
  The outer chrome row is fixed at 32px, so narrow sidebars could wrap group labels into multiple
  lines and let text exceed the row bottom, matching the resize failure mode reported in the UI.
- `text_menu_group_label(...)` now carries the complete shadcn menu/group label role:
  `text-xs font-medium`, fill-width, shrink, zero minimum width, no-wrap, and ellipsis overflow.
  It remains muted by default for menu/select/command surfaces.
- `SidebarGroupLabel` now consumes `decl_text::text_menu_group_label(...)` for its default label
  while overriding inherited foreground with the sidebar-specific 70% foreground. The `as_child`
  custom-child path remains unchanged.
- `menu_group_label_text_uses_muted_medium_xs_single_line_truncation` proves the shared role
  semantics. `sidebar_group_label_uses_shared_menu_group_text_role` proves the fixed sidebar label
  uses that role and keeps sidebar foreground ownership without leaf style/color writes.
- `cargo nextest run -p fret-ui-kit --lib
  menu_group_label_text_uses_muted_medium_xs_single_line_truncation --no-fail-fast` passed.
- First `cargo nextest run -p fret-ui-shadcn --lib
  sidebar_group_label_uses_shared_menu_group_text_role --no-fail-fast` timed out at 304s while
  Cargo/Rustc was still compiling. No process was killed; after Cargo/Rustc exited naturally, the
  same focused command was rerun with a longer timeout and passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-20 shadcn SidebarMenuBadge compact tabular readout slice:

- Source gap before fix: `SidebarMenuBadge` is an upstream fixed `h-5 min-w-5 text-xs font-medium
  tabular-nums` counter slot, but the Fret port still rendered it with sidebar-local
  `ui::text(...).text_size_px(...).line_height_px(...).font_medium().text_color(...).nowrap()`.
  That duplicated text policy in the sidebar recipe and missed the shared readout role's
  shrink/min-width-0/ellipsis resize contract.
- `text_control_readout_compact_tabular_emphasis(...)` now exists as a derived control-readout
  role, not a new base role. It applies inherited `text-xs`, medium weight, `tnum`, no-wrap,
  shrink, zero minimum width, and ellipsis overflow for fixed badge/counter slots.
- `SidebarMenuBadge` now consumes that derived readout role and overrides inherited foreground with
  `sidebar.foreground`, while keeping the badge chrome (`h-5`, `min-w-5`, padding, rounded, inline
  end placement) recipe-owned.
- `compact_tabular_emphasis_readout_uses_xs_medium_single_line_truncation` proves the derived role.
  `sidebar_menu_badge_uses_shared_compact_tabular_readout_role` proves the sidebar badge no longer
  writes leaf style/color and keeps upstream `text-xs font-medium tabular-nums` semantics through
  inherited text metadata.
- `cargo nextest run -p fret-ui-kit --lib
  compact_tabular_emphasis_readout_uses_xs_medium_single_line_truncation
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast` passed.
- `cargo nextest run -p fret-ui-shadcn --lib
  sidebar_menu_badge_uses_shared_compact_tabular_readout_role --no-fail-fast` passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-20 shadcn SidebarMenuButton/SubButton fill button-label slice:

- Source gap before fix: default `SidebarMenuButton` and `SidebarMenuSubButton` labels lived in
  fixed-height sidebar rows but still hand-rolled local `ui::text(...).w_full().min_w_0().flex_1()
  .basis_0().text_size_px(...).font_weight(...).line_height_px(...).truncate()` policy. The
  collapsed sidebar tooltip label also used local `wrap(TextWrap::Word)` / `TextOverflow::Clip`.
  That duplicated role policy in the sidebar recipe and left the small-row `text-xs` shadcn axis
  outside the shared button-label family.
- `text_button_label_fill(...)` and `text_button_label_compact_fill(...)` now exist as derived
  button-label roles. They add fill/grow/basis-zero layout to the button-label resize contract,
  while the compact variant owns `text-xs font-medium` for small trigger rows.
- `SidebarMenuButton` and `SidebarMenuSubButton` default labels now consume those derived roles
  through a small sidebar helper and inherit sidebar foreground. `size=sm` maps to the compact
  role, default/lg and md rows map to the regular fill role, and collapsed tooltip labels use
  `text_button_label(...)` instead of local wrapping text. Sidebar still owns row chrome,
  foreground state, collapse opacity, RTL ordering, and tooltip placement.
- `fill_button_label_text_uses_growing_single_line_truncation` and
  `compact_fill_button_label_text_uses_xs_growing_single_line_truncation` prove the derived role
  contracts. `sidebar_menu_button_label_uses_shared_fill_button_role` and
  `sidebar_menu_sub_button_label_uses_shared_fill_button_role` prove sidebar default labels keep
  fill/grow/min-width-0/no-wrap/ellipsis semantics and the correct `text-sm` vs `text-xs` inherited
  typography.
- `cargo nextest run -p fret-ui-kit --lib
  fill_button_label_text_uses_growing_single_line_truncation
  compact_fill_button_label_text_uses_xs_growing_single_line_truncation
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast` passed.
- First `cargo nextest run -p fret-ui-shadcn --lib
  sidebar_menu_button_label_uses_shared_fill_button_role
  sidebar_menu_sub_button_label_uses_shared_fill_button_role
  sidebar_menu_button_default_content_reorders_in_rtl
  sidebar_menu_sub_button_default_content_reorders_in_rtl --no-fail-fast` timed out while Cargo was
  still compiling. No process was killed; after Cargo exited naturally the same focused command was
  rerun. The first rerun exposed the expected test update for sub-button row signatures after the
  label became a shared role, and the post-fix run passed.
- `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn` passed.
- `python tools\gate_imui_facade_teaching_source.py` passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.

2026-05-20 inherited-axis + shadcn Button default-label role slice:

- Source gap before fix: shadcn `Button` default labels still used a local
  `ui::text(...).text_size_px(...).fixed_line_box_px(...).nowrap().text_color(...)` builder inside
  fixed-height button chrome. Migrating that path directly to `text_button_label(...)` exposed a
  mechanism gap: `TextStyleRefinement` carried OpenType features but not variable font axes, so
  existing `Button::label_font_axis(...)` could not stay on the inherited-role path.
- `TextStyleRefinement` now carries variable font axes as subtree defaults. Merge/refine,
  passive-text measurement, cache fingerprints, and `fret-ui-kit` typography refinements all carry
  axes alongside features.
- `Button` default labels now render through a small helper backed by
  `decl_text::text_button_label(...)`. Button-owned font, feature, axis, explicit weight,
  foreground, and label `test_id` suffix behavior are layered through inherited text/foreground
  metadata instead of leaf-local style/color. The default text size follows the shared shadcn
  `text-sm font-medium whitespace-nowrap` baseline, with the existing Fret `xs/icon-xs` extension
  mapped to the shared `text-xs` preset.
- Focused proof:
  `text_style_refinement_merges_font_axes_in_parent_child_order`,
  `text_style_refine_applies_inherited_font_axes_after_leaf_defaults`,
  `inherited_text_style_axes_affect_passive_text_measurement`,
  `inherited_text_style_fingerprint_tracks_axis_overrides`,
  `composable_refinement_keeps_font_features_and_axes`,
  `button_default_label_uses_shared_button_label_role`,
  `button_default_label_keeps_font_feature_and_axis_overrides_on_role`, and
  `button_default_label_keeps_weight_override_on_role`.

2026-05-20 shadcn CalendarDayButton shared text-role slice:

- Source gap before fix: `CalendarDayButton` day numbers and optional supporting text rendered with
  Calendar-local `ui::label(...).text_size_px(...).line_height_px(...).text_color(...).nowrap()`
  builders inside fixed-size day cells. That duplicated the single-line resize contract in the
  calendar recipe and left the same day-cell text policy outside the shared role/gate vocabulary.
- `calendar_day_label_text(...)` now backs day numbers with `decl_text::text_button_label(...)`,
  layered with Calendar-owned normal weight, foreground, and center alignment through inherited
  text/foreground metadata. `calendar_day_supporting_text(...)` backs optional supporting text with
  `decl_text::text_control_readout(...)`, keeping the auxiliary value in the readout family while
  preserving Calendar-owned center alignment and opacity behavior.
- `Calendar` and `CalendarRange` both feed the same `day_text_style` into
  `calendar_day_button_children(...)`, so single-date and range day cells share the role contract.
  Calendar still owns day-cell chrome, selected/today/range foregrounds, disabled opacity, and
  test-id/ARIA semantics.
- `calendar_day_button_text_uses_shared_roles` proves day and supporting text leaves keep
  `style: None`, `color: None`, fill width, shrink, `min-width: 0`, no-wrap, ellipsis, center
  alignment, inherited text style, and inherited foreground. Existing supporting-text visibility
  tests still prove out-of-month supporting text remains absent.
- Focused gates passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  calendar_day_button_text_uses_shared_roles
  calendar_day_button_supporting_text_renders_only_for_in_month_days --no-fail-fast`,
  `cargo nextest run -p fret-ui-shadcn --lib calendar_range --no-fail-fast`, and
  `cargo check -p fret-ui-shadcn --lib`.

2026-05-20 shadcn menu item label shared text-role slice:

- Source gap before fix: DropdownMenu, ContextMenu, and Menubar overlay item labels still rendered
  local `ui::text(label).text_size_px(...).font_weight(...).nowrap()` leaves inside fixed menu rows.
  That duplicated the resize contract already assigned to `text_list_row_label(...)`, and it made
  menu rows another place where text policy could drift from the IMUI/editor row vocabulary.
- `text_list_row_label(...)` and its attributed variant now use the same fill/grow/basis-zero
  single-line layout expected by dense row labels. The existing tests now assert grow and zero
  basis instead of only checking fill/shrink/min-width-0.
- `menu_text::menu_item_label(...)` backs shadcn menu-family item labels with
  `decl_text::text_list_row_label(...)` and layers menu-owned resolved typography plus foreground
  through inherited text/foreground metadata. DropdownMenu, ContextMenu, and Menubar overlay rows
  consume that helper. Menubar's top-level trigger remains out of this slice because it is
  button-like trigger text, not an overlay item row.
- DropdownMenu no longer wraps entire rows in icon `currentColor` after labels moved to inherited
  foreground. Icon/custom/trailing subtrees still receive icon foreground, while label foreground
  remains the menu item's resolved state foreground.
- Focused gates passed:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  list_row_label_text_uses_fill_width_single_line_truncation
  attributed_list_row_label_text_uses_fill_width_single_line_truncation --no-fail-fast`,
  `cargo nextest run -p fret-ui-shadcn --lib
  menu_item_label_uses_shared_list_row_role_with_menu_refinement
  dropdown_menu_label_element_uses_shared_menu_group_text_role
  context_menu_label_element_uses_shared_menu_group_text_role
  menubar_label_element_uses_shared_menu_group_text_role --no-fail-fast`, and
  `cargo check -p fret-ui-shadcn --lib`.
- Source/format gates passed: `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`,
  `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn`, and `git diff --check`.

2026-05-20 shadcn CalendarMultiple shared text-role slice:

- Source gap before fix: `CalendarMultiple` still rendered multiple-selection day numbers with a
  local `ui::label(day_text).text_size_px(...).line_height_px(...).font_medium().w_full()
  .text_align(...).text_color(...).nowrap()` builder, even though `Calendar` and `CalendarRange`
  had already moved day-cell text into the shared button-label/readout role helper.
- `CalendarMultiple` now reuses `calendar_day_button_children(...)` with the same normal-weight
  `day_text_style` as the single-date/range calendar day cells. Multiple selection still owns
  fixed cell chrome, selected/today foregrounds, disabled opacity, and selection updates; the day
  number text contract is no longer component-local.
- `calendar_multiple_day_text_uses_shared_role` proves multiple calendar day text keeps leaf
  `style: None`, `color: None`, fill width, shrink, `min-width: 0`, no-wrap, ellipsis, center
  alignment, inherited text style, and inherited foreground.
- `tools/gate_imui_workstream_source.py` now requires the CalendarMultiple helper/test shape and
  forbids the old local `ui::label(day_text.clone())` fixed-style block from returning.
- Focused gates passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  calendar_multiple_day_text_uses_shared_role
  calendar_multiple_nav_buttons_render_svg_icons --no-fail-fast`,
  `cargo check -p fret-ui-shadcn --lib`,
  `cargo fmt --check -p fret-ui-shadcn`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-20 shadcn CalendarHijri shared text-role slice:

- Source gap before fix: `CalendarHijri` kept a separate day-cell text implementation that created
  `TextProps::new(day_text)` directly, installed a local fixed line box and foreground, and clipped
  overflow inside fixed-size day cells. This duplicated the day-cell text contract already shared
  by single, range, and multiple Gregorian calendars.
- `hijri_day_cell(...)` now reuses `calendar_day_button_children(...)` with normal-weight inherited
  typography and Persian-digit day labels. Hijri still owns RTL visual order, Gregorian-date test
  ids, fixed cell chrome, outside-month foreground, selected foreground, and selection updates.
- `calendar_hijri_day_text_uses_shared_role` proves the selected Hijri day text keeps leaf
  `style: None`, `color: None`, fill width, shrink, `min-width: 0`, no-wrap, ellipsis, center
  alignment, inherited text style, and inherited foreground.
- `tools/gate_imui_workstream_source.py` now requires the CalendarHijri helper/test shape and
  forbids the old day-cell `TextProps::new(Arc::clone(&day_text))` style block from returning.
- Focused gates passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  calendar_hijri_day_text_uses_shared_role
  calendar_hijri_day_cells_render_stable_test_ids --no-fail-fast`,
  `cargo fmt -p fret-ui-shadcn`,
  `cargo fmt --check -p fret-ui-shadcn`,
  `cargo check -p fret-ui-shadcn --lib`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-20 shadcn Kbd/ShortcutHint keycap text-role slice:

- Source gap before fix: `Kbd` and `ShortcutHint` both live in fixed `h-5` keycap/hint chrome but
  still rendered text through local `ui::label(...).fixed_line_box_px(...).line_box_in_bounds()`
  builders with leaf-local style/color. That kept the keycap resize contract outside the shared
  text-role vocabulary and allowed future fixed-chrome key labels to drift from no-wrap/ellipsis
  behavior.
- `fret-ui-kit::declarative::text::text_keycap_label(...)` now owns the compact keycap label role:
  `text-xs font-medium`, shrink, `min-width: 0`, no-wrap, and ellipsis. It is a narrow derived
  role for fixed key chrome, not a new `fret-imui` runtime surface.
- `Kbd` and `ShortcutHint` now consume `text_keycap_label(...)` and layer their shadcn
  `component.kbd.text_px` / `component.kbd.line_height` typography plus foreground through
  inherited text/foreground metadata. Keycap chrome, tooltip-slot colors, icon escape hatches, and
  hint-row layout remain recipe-owned.
- Focused gates passed:
  `cargo nextest run -p fret-ui-kit --features imui --lib
  keycap_label_text_uses_xs_medium_single_line_truncation
  base_single_line_text_roles_stay_single_line_under_narrow_layout --no-fail-fast` and
  `cargo nextest run -p fret-ui-shadcn --lib
  kbd_defaults_match_shadcn_constraints_and_typography
  shortcut_hint_label_uses_shared_keycap_role --no-fail-fast`.
- Environment note: the first `fret-ui-shadcn` run failed before compiling crate code because
  `C:\Users\Frankorz\AppData\Local\Temp` had no free space (`os error 112`). The successful retry
  set `TMP`/`TEMP` to `F:\SourceCodes\Rust\fret\.fret\tmp`; Cargo still warned that the global
  cache last-use database on C: was full, but both focused tests passed.
- Source/format gates passed:
  `cargo fmt -p fret-ui-kit -p fret-ui-shadcn`,
  `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn`,
  `cargo check -p fret-ui-kit --features imui --lib`,
  `cargo check -p fret-ui-shadcn --lib`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-20 shadcn NativeSelect shared text-role slice:

- Source gap before fix: NativeSelect trigger selected/placeholder text and popover option labels
  were still component-local `ui::text(...)` / `ui::label(...)` fixed-line builders. They had
  no-wrap styling, but the resize contract lived in the recipe instead of the shared text-role
  vocabulary, making select trigger/listbox rows easy to drift under future layout changes.
- `native_select_trigger_text(...)` now backs the trigger value with
  `decl_text::text_control_label(...)`; `native_select_item_text(...)` backs option labels with
  `decl_text::text_list_row_label(...)`. Both layer NativeSelect/Command typography and state
  foreground through inherited metadata, leaving leaf `style` and `color` empty.
- NativeSelect still owns trigger chrome, placeholder vs selected foreground, listbox command
  selection, check icon visibility, popover placement, and RTL ordering. No new `fret-imui` API or
  runtime text role was added.
- `native_select_trigger_and_item_text_use_shared_resize_roles` proves both helper paths keep
  fill/grow/shrink/basis-zero, `min-width: 0`, no-wrap, ellipsis, inherited text style, and
  inherited foreground.
- `tools/gate_imui_workstream_source.py` now requires the helper/test shape and forbids the old
  local trigger/item text builders from returning.
- Focused Rust gates initially exposed a local partial `1.92` toolchain install (`rustup show`
  reported a missing manifest and the toolchain bin directory lacked `rustc.exe`). A non-destructive
  `rustup toolchain install 1.92 --profile default` repaired the pinned toolchain.
- Focused gates passed on the pinned `1.92` toolchain:
  `cargo fmt --check -p fret-ui-shadcn`,
  `cargo check -p fret-ui-shadcn --lib`, and
  `cargo nextest run -p fret-ui-shadcn --lib
  native_select_trigger_and_item_text_use_shared_resize_roles --no-fail-fast`.
- Passed: `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`, `python tools\gate_imui_facade_teaching_source.py`,
  and `git diff --check`.

2026-05-20 shadcn Combobox shared text-role slice:

- Source gap before fix: default Combobox trigger selected/placeholder text and non-search popover
  option labels still used component-local `ui::label(...)` fixed-line builders. This duplicated
  the same resize-sensitive text contract fixed in NativeSelect and left Combobox trigger/listbox
  defaults easy to drift from the shared role vocabulary.
- `combobox_trigger_text(...)` now backs default trigger labels with
  `decl_text::text_control_label(...)`; `combobox_item_text(...)` backs non-search option labels
  with `decl_text::text_list_row_label(...)`. Both layer Combobox/Command typography and state
  foreground through inherited metadata, leaving leaf `style` and `color` empty.
- Combobox still owns trigger chrome, placeholder vs selected foreground, inline addons,
  clear/chevron buttons, popover/drawer policy, search-enabled `CommandPalette` behavior, custom
  item content, and RTL ordering. No new `fret-imui` API or runtime text role was added.
- `combobox_trigger_and_item_text_use_shared_resize_roles` proves both helper paths keep
  fill/grow/shrink/basis-zero, `min-width: 0`, no-wrap, ellipsis, inherited text style, inherited
  foreground, and the trigger-label test id hook.
- `tools/gate_imui_workstream_source.py` now requires the helper/test shape and forbids the old
  local trigger/item default text builders from returning.
- Focused gates passed:
  `cargo fmt --check -p fret-ui-shadcn`,
  `cargo check -p fret-ui-shadcn --lib`,
  `cargo nextest run -p fret-ui-shadcn --lib
  combobox_trigger_and_item_text_use_shared_resize_roles --no-fail-fast`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-21 shadcn ComboboxChips shared text-role slice:

- Source gap before fix: ComboboxChips still hand-built empty-trigger placeholder text and selected
  chip pill labels with local `ui::label(...).text_size_px(...).font_weight(...).truncate()`
  builders. Placeholder text needs the same fill/grow control-label contract as other combo
  triggers, but chip labels need a compact non-growing role so the pill chrome can shrink without
  becoming a row/control label.
- `text_chip_label(...)` is now the shared compact chip/tag/inline-badge text role in
  `fret-ui-kit::declarative::text`. It owns `text-xs font-medium`, no-wrap, shrink,
  `min-width: 0`, and ellipsis, but deliberately leaves width/flex-grow/basis at the non-growing
  defaults.
- `combobox_chips_placeholder_text(...)` backs empty trigger placeholder text with
  `decl_text::text_control_label(...)`; `combobox_chip_label_text(...)` backs selected chip labels
  with `decl_text::text_chip_label(...)`. ComboboxChips still owns trigger and chip chrome, remove
  button behavior, selected-value lookup, popover/search policy, chip row wrapping, and RTL order.
  No new `fret-imui` API or runtime text role was added.
- `chip_label_text_uses_xs_medium_non_growing_single_line_truncation` proves the shared role keeps
  `width: auto`, `flex-grow: 0`, `flex-basis: auto`, shrink, `min-width: 0`, no-wrap, ellipsis,
  and inherited typography. `combobox_chips_placeholder_and_chip_text_use_shared_resize_roles`
  proves the recipe uses fill/grow control-label semantics for placeholders and non-growing chip
  semantics for selected pill labels while layering shadcn typography and foreground through
  inherited metadata.
- `tools/gate_imui_workstream_source.py` now requires the helper/test shape and forbids the old
  local ComboboxChips placeholder/chip text builders from returning.
- Focused gates passed:
  `cargo fmt --check -p fret-ui-kit -p fret-ui-shadcn`,
  `cargo check -p fret-ui-kit --lib`,
  `cargo check -p fret-ui-shadcn --lib`,
  `cargo nextest run -p fret-ui-kit --lib
  chip_label_text_uses_xs_medium_non_growing_single_line_truncation --no-fail-fast`,
  `cargo nextest run -p fret-ui-shadcn --lib
  combobox_chips_placeholder_and_chip_text_use_shared_resize_roles --no-fail-fast`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-21 shadcn Badge default-label shared text-role slice:

- Source gap before fix: Badge default labels still hand-built local
  `ui::text(...).text_size_px(...).fixed_line_box_px(...).text_color(...)` leaves. That duplicated
  the compact chip/tag text contract already introduced for selected ComboboxChips pill labels and
  made the link-hover underline path easy to convert back to direct leaf style/color ownership.
- `badge_label_text(...)` now backs default Badge labels with `decl_text::text_chip_label(...)` and
  layers Badge-owned font, OpenType feature, weight, and foreground/currentColor behavior through
  inherited text/foreground metadata. Badge still owns variant chrome, icon sizing, link/action
  semantics, hover underline, foreground scoping, leading/trailing child fallback styling, and RTL
  order.
- `apply_badge_hover_underline(...)` now preserves `AnyElement` layout-transparent metadata when it
  converts passive text to styled text for link-hover underline, so the shared label role does not
  disappear only on the hovered link path.
- `badge_default_label_uses_shared_chip_label_role` proves the default label keeps auto width,
  non-growing shrink/min-width-zero, no-wrap, ellipsis, empty leaf style/color, inherited medium
  typography, and inherited foreground. The font/feature and weight override tests prove Badge
  refinements stay layered on the inherited role instead of reverting to leaf-local text styles.
  `badge_hover_underline_preserves_default_label_role_metadata` locks the link-hover conversion
  path.
- `tools/gate_imui_workstream_source.py` now requires the helper/test shape and forbids the old
  local Badge default label builder from returning.
- Focused gates passed:
  `cargo fmt -p fret-ui-shadcn --check`,
  `cargo check -p fret-ui-shadcn --lib`,
  `cargo nextest run -p fret-ui-shadcn --lib
  badge_default_label_uses_shared_chip_label_role
  badge_default_label_keeps_font_and_feature_overrides_on_role
  badge_default_label_keeps_weight_override_on_role
  badge_hover_underline_preserves_default_label_role_metadata
  badge_leading_icon_and_label_follow_variant_fg --no-fail-fast`,
  `python -m py_compile tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_workstream_source.py`,
  `python tools\gate_imui_facade_teaching_source.py`, and `git diff --check`.

2026-05-19 shadcn NavigationMenuLink role-preservation slice:

- Source gap before fix: `NavigationMenuLink` recursively wrote link typography and foreground
  directly into descendant text leaves. That kept bare link text ergonomic, but it overwrote shared
  role children such as `text_button_label(...)`, replacing the role-owned typography path that
  controls single-line ellipsis under narrow navigation content.
- `apply_link_inherited_style(...)` now treats `inherited_text_style` as a protected role scope and
  only writes link default typography into bare text. Link foreground is stamped as inherited
  foreground on the child root, so visual state remains link-owned without mutating text-role leaf
  props. The existing default-icon behavior remains intact: default icons opt out of inheriting the
  link foreground.
- `navigation_menu_link_applies_default_style_to_bare_text` proves the fallback still applies to
  bare text; `navigation_menu_link_preserves_shared_text_role_contracts` proves shared button-label
  text keeps `style: None`, ellipsis overflow, and role metadata; and the existing
  `navigation_menu_link_default_icons_do_not_inherit_current_color` test stays in the focused gate.
- `tools/gate_imui_workstream_source.py` now guards the NavigationMenuLink helper/test shape and
  forbids the old leaf foreground write from returning.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  navigation_menu_link_applies_default_style_to_bare_text
  navigation_menu_link_preserves_shared_text_role_contracts --no-fail-fast` failed because the
  role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  navigation_menu_link_applies_default_style_to_bare_text
  navigation_menu_link_preserves_shared_text_role_contracts
  navigation_menu_link_default_icons_do_not_inherit_current_color --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.

2026-05-19 shadcn DataTable role-preservation slice:

- Source gap before fix: `DataTable` wrapped body-cell renderers with
  `apply_default_text_style_recursive(...)`, which recursively wrote a table `TextStyle` into leaf
  text when `props.style` was empty. Shared text-role helpers intentionally keep leaf `style` empty
  and carry typography through `inherited_text_style`, so body cells built with
  `text_table_cell(...)` could lose their role-owned typography/overflow contract.
- `apply_default_text_style_recursive(...)` now delegates to a scoped helper that treats
  `inherited_text_style` as a protected role scope. Bare body text still receives the default table
  style; caller-supplied shared text roles retain their leaf style, wrap, overflow, and role
  metadata.
- `data_table_default_text_style_applies_to_bare_body_text` proves the ergonomic fallback remains;
  `data_table_default_text_style_preserves_shared_text_role_contracts` proves shared table-cell
  text roles survive the wrapper.
- `tools/gate_imui_workstream_source.py` now guards the DataTable helper/test shape.
- Red run before fix:
  `cargo nextest run -p fret-ui-shadcn --lib
  data_table_default_text_style_applies_to_bare_body_text
  data_table_default_text_style_preserves_shared_text_role_contracts --no-fail-fast` failed because
  the role-preservation test observed a leaf `style`.
- Post-fix focused run passed:
  `cargo nextest run -p fret-ui-shadcn --lib
  data_table_default_text_style_applies_to_bare_body_text
  data_table_default_text_style_preserves_shared_text_role_contracts --no-fail-fast`.
- `cargo fmt --check -p fret-ui-shadcn` passed.
- `cargo check -p fret-ui-shadcn --lib` passed.
- `python -m py_compile tools\gate_imui_workstream_source.py` passed.
- `python -m json.tool docs\workstreams\imui-imgui-gap-closure-v1\WORKSTREAM.json > $null`
  passed.
- `python tools\gate_imui_workstream_source.py` passed.
- `git diff --check` passed.
