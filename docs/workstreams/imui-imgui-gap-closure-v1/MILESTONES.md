# ImUi Dear ImGui Gap Closure v1 - Milestones

Status: Active
Last updated: 2026-05-29

## M6 - Continuing IMUI Owner-Split Pressure

Exit criteria:

- Continue reducing large `fret-ui-kit::imui` implementation files after worktree convergence.
- Keep public IMUI facade method names, options, responses, and behavior stable.
- Move policy sub-owners behind private modules and freeze the split with source gates.
- Run focused compile/test/source gates for each slice.

2026-05-29 facade root surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` now keeps the single public
`UiWriterImUiFacadeExt` trait hub plus surface macro expansion only. Scope, basic text/separator
and debug-draw, and disclosure/tree trait default method declarations now live in
`facade_writer/scope_surface.rs`, `facade_writer/basic_surface.rs`, and
`facade_writer/disclosure_surface.rs`. Existing `scope_methods.rs`, `basic_items.rs`, and
`disclosure_controls` remain the behavior owners.

2026-05-29 facade container surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
`UiWriterImUiFacadeExt` trait hub, but item-flow, same-line, dummy/spacing/indent, layout groups,
menu/tab bars, ListBox, grid, table, virtual-list, scroll, and child-region trait default method
declarations now live in `facade_writer/container_surface.rs` and are expanded into the public
trait. Existing `container_methods/*` owners still carry the concrete layout/container behavior.

2026-05-29 facade container surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface.rs` is now a module/re-export hub.
`container_surface/layout.rs` owns item-flow, same-line, spacing, indent, horizontal, and vertical
forwarding; `container_surface/menu_tabs.rs` owns menu-bar and tab-bar forwarding;
`container_surface/collections.rs` owns ListBox, grid, table, and virtual-list forwarding; and
`container_surface/regions.rs` owns scroll and child-region forwarding. The public trait expansion
points in `facade_writer.rs` now call these child macros directly, while concrete
`container_methods/*` behavior owners remain unchanged.

2026-05-29 facade container layout-method sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/layout.rs` is now a module/re-export
hub. `layout/linear.rs` owns horizontal/vertical forwarding, `layout/grid_scroll.rs` owns grid and
scroll forwarding, and `layout/child_region.rs` owns child-region forwarding plus response return.
The public facade methods, build-focus forwarding, element routing, and `container_methods`
re-export paths remain unchanged.

2026-05-29 facade container collection-method sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/collections.rs` is now a
module/re-export hub. `collections/list_box.rs` owns ListBox option normalization and forwarding to
`list_box_controls::list_box_element`, `collections/table.rs` owns Table forwarding and response
return from `table_controls::table_element`, and `collections/virtual_list.rs` owns VirtualList
forwarding and response return from `virtual_list_controls::virtual_list_element`. The public
facade methods, build-focus forwarding, element routing, and `container_methods` re-export paths
remain unchanged.

2026-05-29 facade container flow-method sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/flow.rs` is now a module/re-export
hub. `flow/sequences.rs` owns item-flow and same-line forwarding to the scoped layout-sugar
elements, `flow/spacers.rs` owns dummy and spacing forwarding, and `flow/indent.rs` owns indent
forwarding. The public facade methods, build-focus forwarding, porting-sugar layout routing, and
`container_methods` re-export paths remain unchanged.

2026-05-29 facade container menu/tab-method sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_methods/menu_tabs.rs` is now a
module/re-export hub. `menu_tabs/menu.rs` owns menu-bar forwarding to
`menu_family_controls::menu_bar_element`, and `menu_tabs/tabs.rs` owns tab-bar forwarding plus
`TabBarResponse` return from `tab_family_controls::tab_bar_element`. The public facade methods,
build-focus forwarding, menu/tab routing, and `container_methods` re-export paths remain unchanged.

2026-05-29 facade container wrapper ListBox routing result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/collections.rs` now delegates
inherent `list_box_with_options(...)` to `container_methods::list_box_with_options(...)` instead of
constructing `list_box_controls::list_box_element` directly. This keeps ListBox concrete routing in
the collection method owner and makes inherent ListBox, Table, and VirtualList wrappers follow the
same delegation shape.

2026-05-29 facade floating surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
`UiWriterImUiFacadeExt` trait hub, but floating layer/area, popup open/drop/begin,
tooltip, drag/drop, and in-window floating-window trait default method declarations now live in
`facade_writer/floating_surface.rs` and are expanded into the public trait. Existing
`floating_popup/*` owners still delegate to the concrete floating, popup, tooltip, drag/drop, and
window behavior modules.

2026-05-29 facade floating surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface.rs` is now a module/re-export hub.
`floating_surface/popup.rs` owns floating layer/area and popup open/drop/begin trait forwarding,
`floating_surface/tooltip_drag.rs` owns tooltip and drag/drop forwarding, and
`floating_surface/window.rs` owns in-window floating-window forwarding. The public trait expansion
points in `facade_writer.rs` and the concrete `floating_popup/*` behavior owners remain unchanged.

2026-05-29 facade floating popup surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup.rs` is now a module/re-export
hub. `floating_surface/popup/area.rs` owns floating layer, floating area, and area drag-surface
forwarding; `floating_surface/popup/state.rs` owns popup open-model, drop, open, anchor-open, and
close forwarding; and `floating_surface/popup/begin.rs` owns popup menu/modal begin forwarding.
The public trait expansion points in `facade_writer.rs` now call these child macros directly,
while concrete `floating_popup/*` behavior owners remain unchanged.

2026-05-29 facade menu/selection surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
`UiWriterImUiFacadeExt` trait hub, but menu item, begin menu/submenu, selectable,
multi-selectable, combo, and context-menu trait default method declarations now live in
`facade_writer/menu_selection_surface.rs` and are expanded into the public trait. Existing
`menu_items.rs`, `selection_combo.rs`, and `floating_popup/*` owners still carry inherent wrappers
and underlying behavior.

2026-05-29 facade menu/selection surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface.rs` is now a module/re-export
hub. `menu_selection_surface/menu_items.rs` owns menu item forwarding,
`menu_selection_surface/menu_family.rs` owns begin menu/submenu forwarding,
`menu_selection_surface/selection_combo.rs` owns selectable, multi-selectable, and combo
forwarding, and `menu_selection_surface/context_popup.rs` owns context-menu popup forwarding. The
public trait expansion points in `facade_writer.rs` now call these child macros directly, while
the existing behavior/inherent wrapper owners remain unchanged.

2026-05-29 facade selectable/combo inherent-wrapper sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/selection_combo.rs` is now a module hub.
`selection_combo/selectables.rs` owns selectable and multi-selectable inherent wrappers, including
disabled checks and focusable recording, while `selection_combo/combo.rs` owns direct combo inherent
wrappers with the same focusable recording behavior. The public inherent method names, trait
delegation paths, and `fret-imui` thin boundary remain unchanged.

2026-05-29 facade model surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
`UiWriterImUiFacadeExt` trait hub, but checkbox/radio/switch, slider/combo model, input text
model, input text picker model, history text picker model, and textarea model trait default method
declarations now live in `facade_writer/model_surface.rs` and are expanded into the public trait.
Existing `boolean_wrappers.rs`, `value_models.rs`, and `text_models.rs` inherent wrapper owners
still record focusable state and delegate through the public trait.

2026-05-29 facade model surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface.rs` is now a module/re-export hub.
`model_surface/boolean.rs` owns checkbox/radio/switch model forwarding,
`model_surface/value_combo.rs` owns slider and combo-model forwarding, and `model_surface/text.rs`
owns input text, input text picker/history, and textarea forwarding. The public trait expansion
points in `facade_writer.rs` now call these child macros directly, while `boolean_wrappers.rs`,
`value_models.rs`, and `text_models.rs` remain the focusable-recording inherent wrapper owners.

2026-05-29 facade button surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` keeps the single public
`UiWriterImUiFacadeExt` trait hub, but button, small/arrow/invisible button, image item/button,
action button, payload action button, and button-command trait default method declarations now live
in `facade_writer/button_surface.rs` and are expanded into the public trait. The existing
`button_actions.rs` / `button_actions/*` inherent wrapper owners still record focusable state, and
`image_items.rs` still owns image-button option normalization.

2026-05-29 facade button surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface.rs` is now a module/re-export hub.
`button_surface/plain.rs` owns plain, small, arrow, and invisible button forwarding;
`button_surface/images.rs` owns image item/button forwarding; and `button_surface/actions.rs` owns
action button, payload action button, and button-command forwarding. The public trait expansion
points in `facade_writer.rs` now call these child macros directly, while `button_actions.rs` /
`button_actions/*` and `image_items.rs` remain the behavior/inherent wrapper owners.

2026-05-29 item behavior pointer hook owner-split result:
`ecosystem/fret-ui-kit/src/imui/item_behavior/install.rs` now keeps hook clearing, model capture,
and behavior assembly. `item_behavior/install/pointer_down.rs` owns lifecycle activation and drag
start preparation, `pointer_move.rs` owns drag-threshold move handling, and `pointer_up.rs` owns
lifecycle deactivation, drag finish, context-menu transients, pointer-click modifier capture, and
double-click transients.

2026-05-29 table response owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/widgets/table.rs` now keeps the `TableResponse`
aggregation and header lookup methods. `response/widgets/table/header.rs` owns
`TableHeaderResponse` accessors, and `response/widgets/table/resize.rs` owns
`TableColumnResizeResponse` drag/width projection accessors.

2026-05-29 debug-draw root owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` now stays a thin module/re-export hub.
`debug_draw_controls/draw_list.rs` owns `ImUiDebugDrawList` and channel-split state, while
`debug_draw_controls/facade.rs` owns `debug_draw_with_options(...)` list capture, summary
projection, element mounting, and `DebugDrawResponse` assembly.

2026-05-29 popup-overlay root owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay.rs` now keeps menu/modal entrypoint delegation and
re-exports private owners. `popup_overlay/state.rs` owns popup open/drop/open-at/close state
mutations, and `popup_overlay/context_menu.rs` owns context-menu anchor fallback and menu
delegation.

2026-05-29 tab-bar item-method owner-split result:
`ecosystem/fret-ui-kit/src/imui/tab_family_controls/item_methods.rs` now owns `ImUiTabBar`
`tab_item`, `tab_item_with_options`, `begin_tab_item`, and `begin_tab_item_with_options`
builder methods. `tab_family_controls.rs` keeps `ImUiTabBar` storage and `tab_bar_element`
assembly.

2026-05-29 floating-area option/context owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_options/area.rs` is now a public re-export hub.
`floating_options/area/options.rs` owns `FloatingAreaOptions`, and
`floating_options/area/context.rs` owns opaque `FloatingAreaContext` storage and accessors.

2026-05-29 floating-window option owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_options/window.rs` is now a public re-export hub.
`floating_options/window/behavior.rs` owns `FloatingWindowOptions`,
`floating_options/window/resize.rs` owns `FloatingWindowResizeOptions`, and
`floating_options/window/options.rs` owns `WindowOptions` defaults and builder methods.

2026-05-29 editor theme preset picker option owner-split result:
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker.rs` keeps picker rendering and
behavior. `editor_theme_preset_picker/options.rs` owns `EditorThemePresetPickerOptions` defaults
while the public controls re-export and IMUI adapter remain unchanged.

2026-05-29 leaf control option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/selection.rs`,
`ecosystem/fret-ui-kit/src/imui/options/controls/tab.rs`, and
`ecosystem/fret-ui-kit/src/imui/options/controls/value.rs` are now public re-export hubs.
`selection/options.rs` owns `SelectableOptions`, `tab/options.rs` owns `TabItemOptions`, and
`value/slider.rs` owns `SliderOptions`.

2026-05-29 input-text-picker option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/text/picker.rs` is now a public re-export hub.
`picker/filter.rs` owns `InputTextPickerFilter`, including matching policy, and
`picker/options.rs` owns `InputTextPickerOptions`, including default popup sizing and picker flags.

2026-05-28 textarea option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/text/textarea.rs` is now a public re-export hub.
`textarea/submit_key.rs` owns `TextAreaSubmitKey`, and `textarea/options.rs` owns
`TextAreaOptions`.

2026-05-28 input-text option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/text/input.rs` is now a public re-export hub.
`input/mode.rs` owns `InputTextMode`, and `input/options.rs` owns `InputTextOptions`.

2026-05-28 popup option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/menus/popup.rs` is now a public re-export hub.
`popup/menu.rs` owns `PopupMenuOptions`, and `popup/modal.rs` owns `PopupModalOptions`.

2026-05-28 misc option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/misc.rs` is now a public re-export hub.
`misc/drag_source.rs` owns `DragSourceOptions`, `misc/drop_target.rs` owns `DropTargetOptions`,
`misc/separator_text.rs` owns `SeparatorTextOptions`, and `misc/bullet_text.rs` owns
`BulletTextOptions`.

2026-05-28 spacer flow option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/containers/flow/spacer.rs` is now a private re-export hub.
`flow/spacer/dummy.rs` owns `DummyOptions`, `flow/spacer/spacing.rs` owns `SpacingOptions`, and
`flow/spacer/indent.rs` owns `IndentOptions`.

2026-05-28 linear flow option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/containers/flow/linear.rs` is now a private re-export hub.
`flow/linear/horizontal.rs` owns `HorizontalOptions`, and `flow/linear/vertical.rs` owns
`VerticalOptions`.

2026-05-28 inline flow option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/containers/flow/inline.rs` is now a private re-export hub.
`flow/inline/item_flow.rs` owns `ItemFlowOptions`, and `flow/inline/same_line.rs` owns
`SameLineOptions`.

2026-05-28 child-region option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/containers/child_region.rs` is now a public re-export hub.
`child_region/chrome.rs` owns `ChildRegionChrome`, `child_region/options.rs` owns
`ChildRegionOptions`, and `child_region/resize.rs` owns `ChildRegionResizeXOptions` /
`ChildRegionResizeYOptions`.

2026-05-28 table option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/collections/table.rs` is now a public re-export hub.
`table/root.rs` owns `TableOptions`, `table/row.rs` owns `TableRowOptions`, and `table/cell.rs`
owns `TableCellOptions`.

2026-05-28 menu option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/menus/menu.rs` is now a public re-export hub.
`menu/bar.rs` owns `MenuBarOptions`, `menu/begin.rs` owns `BeginMenuOptions` and
`BeginSubmenuOptions`, and `menu/item.rs` owns `MenuItemOptions`.

2026-05-28 combo control option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/combo.rs` is now a public re-export hub.
`combo/direct.rs` owns `ComboOptions`, while `combo/model.rs` owns `ComboModelOptions` and the
default placeholder text.

2026-05-28 boolean control option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/boolean.rs` is now a public re-export hub.
`boolean/checkbox.rs`, `boolean/radio.rs`, and `boolean/switch.rs` own the three option structs
and their default values.

2026-05-28 disclosure control option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/disclosure.rs` is now a public re-export hub.
`disclosure/collapsing_header.rs` owns `CollapsingHeaderOptions`, and
`disclosure/tree_node.rs` owns `TreeNodeOptions`.

2026-05-28 button/image control option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/button_image.rs` is now a public re-export hub.
`button_image/button.rs` owns button direction/variant/options and `button_image/image.rs` owns
image-item variant/options plus builder methods.

2026-05-28 debug-draw rect path owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/rects.rs` is now a private re-export
hub. `rects/plain.rs` owns plain closed rect path commands and `rects/rounded.rs` owns rounded-rect
point generation plus corner arc sampling.

2026-05-28 debug-draw linear path owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/linear.rs` is now a private re-export
hub. `linear/polyline.rs` owns stroke point requirements and polyline commands, `linear/fills.rs`
owns convex/concave fill forwarding, and `linear/primitives.rs` owns triangle/quad path
construction.

2026-05-28 debug-draw round path owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/round.rs` is now a private re-export
hub. `round/circle.rs` owns circle cubic path construction, `round/ngon.rs` owns regular polygon
path construction, and `round/ellipse.rs` owns ellipse path validation and rotation sampling.

2026-05-28 debug-draw path sampling owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/sampling.rs` is now a private re-export
hub. `sampling/segments.rs` owns default segment fallback, `sampling/arcs.rs` owns circular and
elliptical arc point sampling, and `sampling/beziers.rs` owns quadratic/cubic Bezier point
interpolation.

2026-05-28 debug-draw geometry helper owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/geometry.rs` is now a private re-export hub.
`geometry/finite.rs` owns point/UV/vertex finite checks, `geometry/rects.rs` owns rect checks and
rounding clamp rules, and `geometry/triangles.rs` owns triangle degeneracy/drawability, indexed
triangle lookup, and sequential index generation.

2026-05-28 facade menu-item inherent wrapper owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items/item_methods.rs` now owns plain,
checkbox/radio, and action menu item inherent wrappers. `facade_writer/menu_items.rs` keeps
begin-menu/submenu inherent wrappers and command menu item wiring.

2026-05-28 menu dispatch entry variant owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries/checked.rs` now owns
checkbox/radio entry wrappers and checked-state semantics, while
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries/action.rs` owns action
entry forwarding. `dispatch/entries.rs` keeps plain menu-item routing, shared implementation
forwarding, pressable-hook entry routing, and private re-exports.

2026-05-28 facade button-action inherent wrapper owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions/action_methods.rs` now owns
`action_button`, `action_button_with_options`, `action_payload_button`, and
`action_payload_button_with_options` inherent wrappers. `facade_writer/button_actions.rs` keeps
ordinary button wrappers and command button wiring.

2026-05-28 text-picker core input-root owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core/input_root.rs` now owns prepared
input-root request construction, root mounting, response extraction, and popup item test-id base
forwarding. `text_picker_controls/core.rs` keeps model/candidate/keyboard/open-policy/popup/
response orchestration.

2026-05-28 facade-core disabled-scope owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/facade_core/disabled_scope.rs` now owns
`ImUiFacade::disabled_scope` behavior. `facade_writer/facade_core.rs` keeps the facade storage
shape, focus recording, keyed id helpers, and `UiWriter` implementation.

2026-05-28 table builder row/cell owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/builder/row_methods.rs` now owns `row` /
`row_with_options` row collection and keyed row scopes, while
`ecosystem/fret-ui-kit/src/imui/table_controls/builder/cell_methods.rs` owns `cell` /
`cell_with_options` / `cell_text` / `cell_text_with_options` child mounting and cell packing.
`table_controls/builder.rs` keeps built row/cell data shapes and `build_table_rows`.

2026-05-28 child-region resize-stack owner-split result:
`ecosystem/fret-ui-kit/src/imui/child_region/resize_stack.rs` now owns resize handle test-id
derivation, X/Y handle creation, stack layout/style projection, children ordering, and resizable
root test-id stamping. `child_region.rs` keeps option normalization, scroll owner dispatch,
response aggregation, and the non-resizable vs resizable branch.

2026-05-28 child-region scroll owner-split result:
`ecosystem/fret-ui-kit/src/imui/child_region/scroll.rs` now owns scroll-area builder
construction, content mounting, framed chrome, handle forwarding, viewport test-id routing, and
non-resizable root test-id stamping. `child_region.rs` keeps resize option detection, resize handle
assembly, stack layout/test-id routing, and response aggregation.

2026-05-28 popup-modal request owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/request.rs` now owns modal overlay id/root-name
construction and `OverlayRequest::modal` submission. `popup_overlay/modal.rs` keeps open-state
gating, layout/dismiss/layer owner dispatch, and the final request input assembly.

2026-05-28 popup-modal state owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/state.rs` now owns modal open-model lookup,
is-open reads, and keep-alive generation writeback. `popup_overlay/modal.rs` keeps dismiss policy
creation, overlay identity/root naming, layout owner dispatch, layer owner dispatch, overlay
request assembly, and final focus target selection.

2026-05-28 popup-modal layer owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layer.rs` now owns modal layer/root mounting,
barrier construction, panel semantics mounting, facade child rendering, focus-state construction,
and panel focus handoff. `popup_overlay/modal.rs` keeps open-state gating, keep-alive writeback,
dismiss policy creation, overlay request assembly, and final focus target selection.

2026-05-28 popup-modal dismiss request policy owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/dismiss.rs` now owns modal
`OnDismissRequest` policy for Escape, optional outside press, and default prevention.
`popup_overlay/modal.rs` keeps open-state gating, keep-alive writeback, layer/panel assembly,
overlay request assembly, and focus initialization.

2026-05-28 button pressable props/a11y owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/behavior/props.rs` now owns `PressableProps`
construction, focusable gating, variant layout, and button a11y metadata.
`button_controls/behavior.rs` keeps chrome owner dispatch, behavior owner dispatch, response
projection dispatch, and visual resolution.

2026-05-28 button pressable response projection owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/behavior/response.rs` now owns button clicked
transient consumption and shared `PressableItemResponseInput` projection.
`button_controls/behavior.rs` keeps pressable props/chrome assembly, activation/keyboard owner
dispatch, and visual assembly.

2026-05-28 button pressable activation behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/behavior/activation.rs` now owns pressable
activate-hook installation, keyboard activation lifecycle marking, clicked transient recording,
action dispatch, and notify. `button_controls/behavior.rs` keeps pressable props/chrome assembly,
keyboard owner dispatch, response population, and visual assembly.

2026-05-28 button pressable keyboard behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/behavior/keyboard.rs` now owns focused button
activate-shortcut handling and keyboard context-menu requests. `button_controls/behavior.rs` keeps
pressable props/chrome assembly, action activation, response population, and visual assembly.

2026-05-28 debug-draw round path-command dispatch owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/round.rs` now only
dispatches to stroked and filled round command owners. `round/stroked.rs` owns circle/ngon/ellipse
stroke paint routing, and `round/filled.rs` owns circle/ngon/ellipse fill routing.

2026-05-28 debug-draw linear path-command dispatch owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands/linear.rs` now only
dispatches to stroked and filled linear command owners. `linear/stroked.rs` owns line/polyline/rect/
quad/triangle stroke paint routing, and `linear/filled.rs` owns convex/concave polygon, quad-fill,
and triangle-fill routing.

2026-05-28 debug-draw geometry summary projection owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry.rs` now
dispatches only across geometry family owners. `geometry/linear.rs`, `geometry/mesh.rs`,
`geometry/round.rs`, and `geometry/beziers.rs` own the concrete summary point/vertex/index/triangle
counts for their command families.

2026-05-28 debug-draw stroked linear path painter owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear/line_poly.rs`
now owns line and polyline stroke painting, and
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear/rect_quad_triangle.rs`
owns rect, quad, and triangle stroke painting. `stroked/linear.rs` keeps private re-exports.

2026-05-28 debug-draw list summary classification owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/list/classification.rs` now owns
command-kind to list summary class mapping. `summaries/list.rs` keeps aggregate counters, public
accessors, final clip-depth writeback, and include-time counter updates.

2026-05-28 text-picker popup item owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/popup/item.rs` now owns selectable candidate
rows, item test-id derivation, active element writeback, model update, popup close, and click pick
result. `popup.rs` keeps popup lifetime, keyboard handler installation, and aggregate pick result
merging.

2026-05-28 table header resize grip visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header/resize/visual.rs` now owns resize grip color,
disabled alpha, and visual dimensions. `header/resize.rs` keeps pointer-region drag setup, cursor
behavior, response writeback, drag response edge merging, and test-id attachment.

2026-05-28 debug-draw filled path painter owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled/polygons.rs` now owns
convex/concave/quad/triangle fill painting, and `paint_shapes/paths/filled/round.rs` owns
circle/ngon/ellipse fill painting. `filled.rs` keeps the shared fill style and private re-exports.

2026-05-28 disclosure header metrics owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/header/metrics.rs` now owns indicator
glyph selection, tree indentation padding, and header border edges. `visual/header.rs` keeps
palette lookup, row element composition, glyph/text rendering, and spacer layout.

2026-05-28 menu-item command helper owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items/command.rs` now owns command presentation
lookup, enabled gating, and shortcut fallback for menu command items. `menu_items.rs` keeps the
public menu item wrappers, focusable recording, and the private helper re-export.

2026-05-28 tooltip runtime layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime/layout.rs` now owns anchor bounds,
measured/estimated panel sizing, and floating bounds calculation. `tooltip_overlay/runtime.rs`
keeps trigger gates, interaction updates, open state writeback, and overlay request submission.

2026-05-28 button-command helper owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions/button_command.rs` now owns command
presentation lookup and enabled gating. `button_actions.rs` keeps the public button wrappers and
the private helper re-export.

2026-05-28 pressable drag state-machine owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/pressable.rs` now owns pressable pointer
down/move/up drag state transitions, long-press timer coordination, active item cleanup, and
drag-started/stopped transients. `interaction_runtime/drag.rs` keeps drag kind/threshold helpers
and private sub-owner re-exports.

2026-05-28 drag-source payload lifecycle owner-split result:
`ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks/payload_lifecycle.rs` now owns pointer-move
active payload tracking, hovered-target preservation, and pointer-up delivery insertion.
`drag_drop/source/hooks.rs` keeps enabled gating, cross-window drag upgrade policy, and the private
payload-lifecycle delegation.

2026-05-28 table-column visibility menu-item owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu/item.rs` now owns single checkbox item
rendering, visible-state reads, model mutation, and changed/edited response flags. `menu.rs` keeps
header context-menu orchestration, item group composition, identity/test-id filtering, and the
private item-owner re-export.

2026-05-28 menubar active-trigger reconcile owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy/active_trigger/reconcile.rs`
now owns close-after-render reconciliation, popup close restoration, and open-menu/group-active
cleanup. `active_trigger.rs` keeps active-trigger open-menu sync, activation, and the private
re-export.

2026-05-28 begin-menu capture read owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/capture/read.rs` now owns bool and
open-menu model reads for begin-menu capture/open-policy. `capture.rs` keeps `BeginMenuState`,
`MenuRenderState`, row/popup/was-open model identity, render-state writeback, and read facade
methods.

2026-05-28 table builder test-id owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/builder/test_ids.rs` now owns row/cell test-id
derivation, including explicit row test-id override fallback and default `.row.*` / `.cell.*`
strings. `builder.rs` keeps public `ImUiTable` / `ImUiTableRow` methods, row/cell collection,
keyed row scopes, child mounting, and table render handoff.

2026-05-28 selectable popup-nav owner-split result:
`ecosystem/fret-ui-kit/src/imui/selectable_controls/keyboard/popup_nav.rs` now owns inherited
popup menu nav item registration plus Arrow/Up/Home/End focus movement. `keyboard.rs` keeps
selectable shortcut activation, popup close-on-shortcut, and context-menu key handling.

2026-05-28 menu-family trigger menubar owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior/menubar.rs` now owns
menubar trigger-row registration, state sync, patient-click timer wiring, toggle-on-activate, and
ArrowDown/ArrowUp open support. `behavior.rs` keeps base active-trigger behavior, shortcut
activation, click transient recording, and active-trigger response population.

2026-05-28 disclosure trigger response owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger/behavior/response.rs` now owns
trigger-response projection, hover query hook attachment, active-item hover blocking, and
response sanitization. `behavior.rs` keeps pressable/key/pointer hook installation and delegates
the projection to the dedicated response owner.

2026-05-28 slider pointer value-update owner-split result:
`ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer/value_update.rs` now owns
pointer-to-value projection, clamp/snap, and changed-detection writes. `pointer.rs` keeps
pointer hook installation, active-item updates, capture/release, focus, lifecycle activation/
deactivation, and transient change emission.

2026-05-28 combo trigger visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/visual.rs` now owns ComboBox trigger props,
field chrome lookup, visual children assembly, and the a11y label helper. `trigger.rs` keeps
behavior installation and visual-owner dispatch, while public combo behavior remains unchanged.

2026-05-28 popup-menu overlay request owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/request.rs` now owns popup-menu open model
lookup, trigger fallback, auto-focus target construction, focus-outside submenu preservation,
menubar close-auto-focus suppression, submenu pointer-move handler installation, modal flag
forwarding, and final overlay request submission. `menu.rs` keeps overlay id/root creation,
popup/menubar policy lookup, panel build orchestration, and request owner dispatch.

2026-05-28 text-picker keyboard preparation owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core/keyboard_state.rs` now owns text-picker
keyboard model creation, enabled/empty/exact-match reconciliation, active source projection,
pending keyboard pick projection, and active descendant element projection. `core.rs` keeps model
reads, candidate visibility, input-root mounting, open-policy application, popup rendering, and
pick response merging.

2026-05-28 child-region resize handle owner-split result:
`ecosystem/fret-ui-kit/src/imui/child_region/resize/handle.rs` now owns the shared pointer-region
resize handle, drag-kind setup, drag threshold handling, pointer down/move/up hooks, drag response
projection, started/stopped edge synthesis, and handle test-id stamping. `resize.rs` keeps X/Y
entry points and response option/min/max wiring, while `resize/axis.rs` remains the layout/cursor
owner.

2026-05-28 submenu state owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state.rs` is now a private
module/re-export index. `submenu_state/clear.rs` owns submenu clear behavior, including pending
open cleanup, pointer-grace cleanup, timer cleanup, and focus retry reset. `submenu_state/select.rs`
owns submenu selection writes and pending/open-timer cleanup.

2026-05-28 menu keyboard owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard.rs` is now a private module/re-export index.
`keyboard/popup.rs` owns popup menu item registration, keyboard shortcut activation,
popup-close-on-key activation, and Arrow/Home/End item focus movement. `keyboard/menubar.rs` owns
menubar horizontal-arrow close-focus suppression and primitive trigger-row horizontal switching
wiring.

2026-05-28 interaction-runtime models owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/models.rs` is now a private module/re-export
index. `models/element.rs` owns element-scoped context-menu anchor, long-press, pointer-click
modifier, lifecycle-session, and collapsed-window stores. `models/window.rs` owns the
window-scoped active-item store. `models/scope.rs` owns disabled-scope depth. `models/state.rs`
owns the public long-press, lifecycle, and active-item state shapes.

2026-05-28 input-text props owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/input/props.rs` now owns
`InputTextAssistiveSemantics`, `TextInputProps` construction, built-in/custom insert filters,
password-mode projection, accessibility metadata, placeholder/submit/cancel forwarding, and compact
IMUI input chrome/style. `input.rs` keeps model reads, response lifecycle, select-all effect
dispatch, element mounting, and policy-command installation.

2026-05-28 text policy command owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/policy_commands.rs` is now a private module/re-export
index. `policy_commands/input.rs` owns input completion/history/undo/redo key-down dispatch, and
`policy_commands/textarea.rs` owns textarea submit/cancel key-down capture dispatch. Input and
textarea model assembly still call the same internal policy helpers through the unchanged text
control module surface.

2026-05-28 table-column visibility state owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/state.rs` now owns only the state/override
storage shape and public snapshot re-export. `state/overrides.rs` owns runtime override
mutation/query, `state/snapshot_io.rs` owns snapshot conversion/restoration, and
`state/columns.rs` owns `TableColumn` application.

2026-05-28 child-region resize response owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize.rs` is now a private
module/re-export index. `resize/x.rs` owns width-axis response projection and tests, while
`resize/y.rs` owns height-axis response projection and tests. `ChildRegionResponse` still re-exports
both public response types from the same public IMUI surface.

2026-05-28 input-text filter owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/text/filters.rs` is now a private
module/re-export index. `filters/builtin.rs` owns `InputTextFilters` plus
decimal/scientific/hex/uppercase/no-blank character filtering, and `filters/custom.rs` owns
`InputTextCustomFilter` closure storage and debug output.

2026-05-28 debug-draw draw-list image-authoring owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list/images.rs` is now a private module
index. `images/mesh.rs` owns image triangle-mesh command recording, `images/raster.rs` owns image,
image-region, and image-quad command recording, and `images/rounded.rs` owns rounded image/region
command recording.

2026-05-28 debug-draw paint-helper owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_helpers.rs` is now a private
module/re-export index. `paint_helpers/media.rs` owns opacity/UV validation plus raster image scene
ops, `paint_helpers/meshes.rs` owns vertex-color and image triangle mesh scene ops, and
`paint_helpers/rounded.rs` owns rounded-corner visibility/projection.

2026-05-28 debug-draw path-builder shape-method owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods.rs` is now a private
module index. `shape_methods/rects.rs` owns rect and rounded-rect point appending,
`shape_methods/beziers.rs` owns quadratic/cubic Bezier sampling, and `shape_methods/arcs.rs` owns
circular, fast 12-step, and elliptical arc sampling. `path_builder.rs` still owns path authoring
storage plus stroke/fill dispatch.

2026-05-28 shared hover-delay state owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay/state.rs` now owns
`ImUiSharedHoverDelayState`, `ImUiSharedHoverDelayStore`, `model_for_window`, and `delay_flags`.
`hover/shared_delay.rs` keeps hover-enter/leave shared timer policy and clear-timer handling.

2026-05-28 hover query delay read owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/read.rs` now owns local hover-delay
state, transient consumption, shared-delay flag reads, and `HoverQueryDelayRead` projection.
`interaction_runtime/hover.rs` keeps active-item blocking, hover-change hook installation, timer
dispatch, shared-delay delegation, and long-press delegation.

2026-05-28 layout sugar scoped/spacer owner-split result:
`ecosystem/fret-ui-kit/src/imui/layout_sugar/scoped.rs` now owns item-flow, same-line, and indent
container composition. `ecosystem/fret-ui-kit/src/imui/layout_sugar/spacers.rs` owns dummy/spacing
spacer construction and default IMUI spacing token projection. `layout_sugar.rs` is now a private
module/re-export index.

2026-05-28 text-picker keyboard handler owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard/handler.rs` now owns key-down capture,
Arrow/Enter navigation and pick handling, repeat/IME/modifier gating, model writes, and popup close.
`text_picker_controls/keyboard.rs` keeps keyboard pick/state/snapshot storage and reconciliation.
Focused gates passed: `cargo fmt -p fret-ui-kit`, `cargo check -p fret-ui-kit --features imui --lib`,
`cargo nextest run -p fret-imui models_text_picker --no-fail-fast`, source gate, catalog, and
`git diff --check`.

2026-05-28 menu routing dispatch owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch/entries.rs` now owns
public-in-IMUI menu-item entry wrappers plus semantics/action selection.
`menu_controls/routing/dispatch/core.rs` owns no-op pressable hook and identity-to-mount dispatch.
`menu_controls/routing/dispatch.rs` is now a private module/re-export index.

2026-05-28 facade support slider math owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_support/slider_math.rs` now owns
`slider_step_or_default`, `slider_normalize_range`, `slider_clamp_and_snap`, and
`slider_value_from_pointer`. `facade_support.rs` keeps writer bridge support, transient keys,
runtime frame prep, device-pixel snapping, point arithmetic, and model-change detection.

2026-05-28 drag source hook owner-split result:
`ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks.rs` now owns enabled/cross-window policy,
pointer-down cross-window promotion, pointer-move active payload publication, and pointer-up
delivery insertion. `drag_drop/source.rs` keeps trigger validation, payload boxing, store model
lifecycle/pruning, drag-kind selection, hook owner dispatch, and source response projection.

2026-05-28 interaction lifecycle response owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle/response.rs` now owns
transient-to-response population, active-state lifecycle frame diffing, edited-state stamping, and
activated/deactivated merge application. `interaction_runtime/lifecycle.rs` keeps pointer-down/up
lifecycle mutation, instant edit mutation, lifecycle edit mutation, and private re-exports for
callers.

2026-05-28 tooltip overlay request owner-split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/request.rs` now owns panel child construction,
tooltip overlay request creation, trigger binding, dismiss close-request signaling, optional
hoverable-content pointer tracker installation, and request submission.
`tooltip_overlay/runtime.rs` keeps trigger-id validation, event/open models, pointer-move open gate
installation, hover/focus update gates, interaction updates, panel-size/anchor projection, and
open-state synchronization.

2026-05-28 floating area layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/area/layout.rs` now owns absolute area layout
props, `interactivity_gate_props` selection for `no_inputs`, `hit_test_gate_props` selection for
hit-test passthrough, and the container fallback. `floating_surface/area.rs` keeps layer child
registration, drag snapshot/state reconciliation, child mounting, final test-id stamping, and
`FloatingAreaResponse` construction.

2026-05-28 floating layer z-order owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/layer/z_order.rs` now owns
`FloatWindowLayerZOrder`, z-order membership, bring-to-front reordering, missing-window pruning,
and rank snapshot projection. `floating_surface/layer.rs` keeps layer marker state, child
registration, activation dispatch, layer child mounting, rank sort application, and absolute fill
layout.

2026-05-28 shared item behavior install owner-split result:
`ecosystem/fret-ui-kit/src/imui/item_behavior/install.rs` now owns pressable pointer hook clearing,
active-item/long-press/lifecycle model capture, and assembly. Later pointer-hook sub-owner splits
move down/move/up transient bodies into `item_behavior/install/*`.
`item_behavior.rs` keeps shared data shapes plus install/response re-exports.

2026-05-28 facade floating/popup owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs` is now a private module/re-export
index. `floating_popup/floating.rs` owns floating layer/area forwarding, `popup.rs` owns popup
open/close/menu/modal/context forwarding, `tooltip.rs` owns tooltip forwarding,
`drag_drop_facade.rs` owns drag/drop forwarding, and `window.rs` owns floating-window forwarding.

2026-05-28 button behavior action owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/behavior/action.rs` now owns `ButtonAction`, action
payload storage, command dispatch source recording, pending payload recording, and final action
dispatch. `button_controls/behavior.rs` keeps pressable props, shortcut/context-menu handlers,
enabled gating, lifecycle marking, response population, and visual resolution.

2026-05-28 button visual a11y/variant owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/visual/a11y.rs` now owns button `PressableA11y`
construction, `SemanticsRole::Button`, custom label fallback, and arrow a11y labels.
`button_controls/visual/variant.rs` owns variant sizing plus arrow glyph selection.
`button_controls/visual.rs` keeps `ButtonVisual`, `ButtonVisualContent`, chrome resolution, and
visible/invisible content assembly.

2026-05-28 tab item list/panel owner-split result:
`ecosystem/fret-ui-kit/src/imui/tab_family_controls/items/list.rs` now owns tab-list semantics,
trigger rendering, selected/first-focusable trigger tracking, and `TabTriggerResponse`
collection. `items/panel.rs` owns selected tab-panel semantics and panel child mounting.
`items.rs` keeps `BuiltTabItem`, selected-model normalization, build-focus propagation, final
column assembly, and `TabBarResponse` construction.

2026-05-28 text-picker core owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` now owns input-text picker
orchestration: model reads, candidate visibility, keyboard snapshot reconciliation, input root
mounting, open-policy application, popup rendering, and pick response merging.
`text_picker_controls.rs` is now a private module index and re-export hub for the core picker and
completion/history entry wrappers.

2026-05-27 table header-cell owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header/cell.rs` now owns header cell layout,
resize-handle attachment, resize test-id suffixing, and header content flex wrapping.
`table_controls/header.rs` keeps sortable/plain header trigger orchestration and `BuiltHeaderCell`
response assembly.

2026-05-27 debug-draw path-family owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs` is now a private path-family
re-export hub. `paths/linear.rs` now indexes polyline/fill/primitive subowners; `paths/round.rs`
now indexes circle/ngon/ellipse subowners; `paths/beziers.rs` owns quadratic and cubic bezier path
construction. The 2026-05-28 follow-ups split the linear and round families into
`paths/linear/{polyline,fills,primitives}.rs` and
`paths/round/{circle,ngon,ellipse}.rs`.

2026-05-27 debug-draw command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types.rs` now owns the private
`DebugDrawCommand` payload enum and all draw-list command variants. `debug_draw_controls/commands.rs`
keeps summary projection wiring plus the parent-visible command re-export.

2026-05-27 table builder owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/builder.rs` now owns `ImUiTable` /
`ImUiTableRow`, built row/cell records, row/cell test-id derivation, child `ImUiFacade`
mounting, and `cell_text(...)` table-cell text routing. `table_controls.rs` keeps only module
wiring, public table builder re-exports, `table_element(...)`, and final render dispatch.

2026-05-27 input-text element owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/input.rs` now owns input-text model element assembly,
assistive semantics, response lifecycle population, select-all command emission, input filters,
policy-command installation, and compact input chrome/style selection. `text_controls.rs` is now a
private focus/input/policy/style/textarea module index and re-export hub.

2026-05-27 menu-item routing dispatch owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing/dispatch.rs` now owns public menu-item entry
wrappers, checkbox/radio/action role selection, noop-hook routing, and identity-to-mount dispatch.
`menu_controls/routing.rs` is now a private dispatch/identity/mount module index and re-export hub.

2026-05-27 disclosure layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/layout.rs` now owns content container
composition, body `ImUiFacade` construction, root column layout, and content/root test-id
application. `disclosure_controls.rs` keeps label identity parsing, open-model reads, trigger
mounting, and aggregate `DisclosureResponse` writes.

2026-05-27 text-picker entry owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/entry.rs` now owns completion/history wrapper
functions plus history filter/open normalization. `text_picker_controls.rs` keeps core picker
orchestration and re-exports the entry helpers.

2026-05-27 floating-window shell props owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_shell/props.rs` now owns frame, title-bar, shell
column, and clipped-body props. `floating_window_shell.rs` keeps shell composition, blocker
mounting, and resize-stack composition.

2026-05-27 menu-item routing mount owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing/mount.rs` now owns final menu-item element
mounting, `ResponseExt::default()` initialization, final `ui.add(...)`, and response return.
`menu_controls/routing.rs` keeps public dispatch, checkbox/radio/action role selection,
noop-hook routing, and label identity scoping.

2026-05-27 table body-row owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/render/body_rows.rs` now owns keyed body row
assembly, hidden-column filtering, fallback empty-cell insertion, body cell wrapping, striped row
selection, and body row wrapping. `table_controls/render.rs` keeps palette, visible-column,
scroll/header decisions, root chrome, semantics, and final `TableResponse` assembly.

2026-05-27 table-column visibility response owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/response.rs` now owns
`TableColumnVisibilityMenuResponse`, `TableColumnVisibilityHeaderContextMenuResponse`, and
`TableColumnVisibilityMenuItemResponse` plus their public accessors. The root
`table_column_visibility.rs` keeps options, state re-exports, public helper forwarding, and tests.

2026-05-27 control chrome palette/button/field owner-split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/chrome.rs` is now a private module index/re-export
hub. `control_chrome/chrome/palette.rs` owns `ImUiControlPalette`,
`control_chrome/chrome/button.rs` owns button theme resolution and compact button chrome props,
and `control_chrome/chrome/field.rs` owns field theme resolution plus fill-width field chrome
props.

2026-05-27 container element owner-split result:
`ecosystem/fret-ui-kit/src/imui/containers.rs` is now a private module index/re-export hub.
`ecosystem/fret-ui-kit/src/imui/containers/children.rs` owns child `ImUiFacade` mounting with build
focus propagation. `containers/linear.rs` owns horizontal/vertical flex composition,
`containers/scroll.rs` owns scroll-area construction, and `containers/grid.rs` owns grid row
batching plus keyed row assembly.

2026-05-27 flow option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/containers/flow.rs` is now a private module/re-export
index. `flow/spacing.rs` owns the IMUI layout-token defaults, `flow/inline.rs` is the current
inline option re-export hub, `flow/inline/item_flow.rs` owns item-flow options,
`flow/inline/same_line.rs` owns same-line options, `flow/linear.rs` is the current linear option
re-export hub, `flow/linear/horizontal.rs` owns horizontal options, `flow/linear/vertical.rs` owns
vertical options, `flow/spacer.rs` is the current spacer option re-export hub,
`flow/spacer/dummy.rs` owns dummy options, `flow/spacer/spacing.rs` owns spacing options,
`flow/spacer/indent.rs` owns indent options, and `flow/grid.rs` owns grid options.

2026-05-27 popup-menu panel owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel.rs` keeps popup open/anchor lifecycle
reads, keepalive updates, nav-state installation, panel id storage, and `PopupMenuBuilt` assembly.
`panel/layout.rs` owns popper placement, menu semantics layout, panel palette/chrome, and column
props. `panel/content.rs` owns popup/menubar policy provider nesting plus IMUI child mounting.

2026-05-27 checkbox behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox/behavior.rs` now owns pressable behavior
installation, activate/shortcut model toggling, context-menu key handling, transient changed reads,
and `ResponseExt` population. `checkbox.rs` keeps label identity, `CheckboxOptions` a11y wiring,
field chrome, checkbox indicator mounting, boolean label mounting, and fill-row visual assembly.

2026-05-27 radio behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/radio/behavior.rs` now owns pressable behavior
installation, activate/shortcut click signaling, context-menu key handling, transient clicked
reads, and `ResponseExt` population. `radio.rs` keeps label identity, `RadioOptions` a11y wiring,
field chrome, radio indicator mounting, boolean label mounting, and fill-row visual assembly.

2026-05-27 debug-draw media paint owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` keeps
`paint_debug_draw_media_command(...)` routing. `paint/media/raster.rs` owns image, image-region, and
image-quad paint. `paint/media/rounded.rs` owns rounded image/region paint and clip balancing.
`paint/media/svg.rs` owns SVG image and mask-icon paint.

2026-05-27 debug-draw element behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element/behavior.rs` now owns pressable
behavior installation, keyboard activation lifecycle marking, clicked transient reads, and
`ResponseExt` population. `element.rs` keeps canvas composition, fill-layout policy for interactive
canvases, cache policy, clipping, test-id routing, and debug-draw command painting.

2026-05-27 table row-group owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/row_groups.rs` keeps
`wrap_pinned_table_row_groups(...)` orchestration. `row_groups/split.rs` owns pinned-cell
classification, `row_groups/layout.rs` owns horizontal row flex wrappers, and
`row_groups/scroll.rs` owns center horizontal scroll wrapping.

2026-05-27 debug-draw draw-list linear owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear.rs` is now a private
module index. `linear/line_poly.rs` owns line, polyline, convex polygon fill, and concave polygon
fill command recording. `linear/rect_quad_triangle.rs` owns rect, quad, triangle, and filled
variants.

2026-05-27 table-column method-family owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/collections/table_column.rs` keeps the public
`TableColumn` storage shape and primitive re-exports.
`table_column/construction.rs`, `identity.rs`, `visibility.rs`, `sorting.rs`, `resize.rs`, and
`pinning.rs` now own the corresponding `TableColumn` impl method families without changing method
names or chainability.

2026-05-27 drag/drop store owner-split result:
`ecosystem/fret-ui-kit/src/imui/drag_drop/store.rs` is now a private re-export index.
`drag_drop/store/state.rs` owns the shared drag/drop model and active/delivered payload records.
`store/lifecycle.rs` owns global model creation and stale session/delivery pruning.
`store/source_response.rs` owns source response projection, while `store/target_payloads.rs` owns
typed active/delivered payload lookup for drop targets.

2026-05-27 debug-draw summary projection owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/geometry.rs` now
owns point/vertex/index/triangle-count summaries for geometric commands.
`summary_projection/clip_state.rs` owns push/pop/current clip rect and clip-depth updates.
`summary_projection.rs` keeps the public-in-debug-draw entry point plus media/text/clip command
routing.

2026-05-27 canonical workbench teaching-doc refresh result:
`docs/examples/README.md`, `apps/fret-cookbook/README.md`, and
`apps/fret-cookbook/EXAMPLES.md` now say `imui_editor_workbench_demo` mounts the editor-notes
workflow directly. The same docs keep `imui_editor_proof_demo` as the supporting dense panel /
explicit stable identity proof, and `tools/gate_imui_facade_teaching_source.py` freezes the
current wording.

2026-05-27 P0/P2 canonical workbench status refresh result:
`P0_CURRENT_SOURCE_AUDIT_2026-05-06.md` and `TODO.md` now name
`cargo run -p fret-demo --bin imui_editor_workbench_demo` as the canonical product-facing editor
workbench route. `imui_editor_proof_demo`, `workspace_shell_demo`, and docking demos remain
supporting proof surfaces, while current workbench verification points at
`imui_editor_workbench_golden_path_surface`.

2026-05-27 debug-draw path-command family owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands.rs` is now a thin
family router. `path_commands/linear.rs` owns line/polyline/polygon/rect/quad/triangle dispatch,
`path_commands/round.rs` owns circle/ngon/ellipse dispatch, and `path_commands/beziers.rs` owns
quadratic/cubic bezier dispatch.

2026-05-27 debug-draw path-shape dispatch owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/path_commands.rs` now owns
line/polyline/polygon, rect-outline, quad, triangle, circle, ngon, ellipse, and bezier dispatch into
the path paint owners. `paint_shapes.rs` keeps draw-order/key setup, filled rect routing, mesh
routing, text routing, and ignored media/clip command routing.

2026-05-27 menu-item routing owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing.rs` now owns menu item entry dispatch,
checkbox/radio/action semantic role selection, `##/###` label identity parsing, item-id scoping,
response assembly, and final element insertion. `menu_controls.rs` is now a thin module/re-export
index that wires routing, element, interaction, keyboard, visual, and tests owners.

2026-05-27 menu-item routing identity owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/routing/identity.rs` now owns
`parse_label_identity(...)`, visible label extraction, and `menu-item-label` `push_id` scoping.
`menu_controls/routing.rs` keeps public menu item dispatch, semantic role selection, response
assembly, and final element insertion. Public menu item labels and stable-id behavior remain
unchanged.

2026-05-27 P3 component catalog refresh result:
`P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md` no longer treats ListBox, plot adapter, or
style/theme preset picker as open candidate-only gaps. The current map now records ListBox as a
kit-owned container proof, plot as an optional `fret-plot/imui` adapter, and style/theme editing as
editor-owned preset tooling surfaced through the canonical workbench.

2026-05-27 active-trigger behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/active_trigger_behavior/keyboard.rs` now owns ContextMenu and
Shift+F10 request handling. `active_trigger_behavior/pointer.rs` owns primary active-item pointer
lifecycle, focus request policy, and secondary-click anchor signaling.
`active_trigger_behavior/response.rs` owns context-menu response fields, hover query hookup, and
shared pressable response population. `active_trigger_behavior.rs` keeps handler clearing, model
lookup, option/input structs, and owner dispatch.

2026-05-27 tooltip runtime owner-split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime.rs` now owns trigger-id validation,
tooltip event model setup, interaction bounds calculation, open/update scheduling, open-model sync,
dismiss request handling, hoverable-content tracking, and `request_tooltip(...)` orchestration.
`tooltip_overlay.rs` is now a thin module index that wires the runtime, trigger, panel, text, and
tests owners without carrying tooltip policy code.

2026-05-27 slider pointer/keyboard interaction owner-split result:
`ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer.rs` now owns pointer
down/move/up capture, active-item set/clear, pointer value projection, pointer model mutation, and
pointer lifecycle edit signals. `slider_controls/interaction/keyboard.rs` owns enabled keyboard
gating, arrow/page/home/end value edits, snapping, and keyboard lifecycle edit signals.
`interaction.rs` keeps handler clearing, active/lifecycle model lookup, shared range input, and
owner dispatch.

2026-05-27 begin-menu trigger behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior.rs` now owns active-trigger
behavior installation, keyboard activation lifecycle marking, activate shortcut handling, menubar
row registry/sync wiring, arrow-down/up open behavior, transient click reads, and trigger
`ResponseExt` population. `trigger.rs` keeps label identity, `PressableA11y`, pressable shell
construction, and `visual::menu_trigger_visual(...)` mounting.

2026-05-27 switch behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/switch/behavior.rs` now owns active-trigger
behavior installation, activate/shortcut model toggling, lifecycle edit marking, transient
changed/clicked reads, and `ResponseExt` population. `switch.rs` keeps label identity,
`SwitchOptions` a11y wiring, field chrome, switch state badge mounting, boolean label mounting, and
fill-row visual assembly.

2026-05-27 disclosure trigger behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger/behavior.rs` now owns pressable
callback installation, activate shortcut/context-menu key handling, pointer down/up hooks,
hover-delay reads, context-menu anchor reporting, enabled sanitization, and trigger `ResponseExt`
population. `trigger.rs` keeps pressable props, a11y, header visual mounting, collapsible trigger
controls, and test-id application.

2026-05-27 tab-family selected-model normalization owner-split result:
`ecosystem/fret-ui-kit/src/imui/tab_family_controls/items/selection.rs` now owns selected model
reads, current-tab validity checks, default-selected fallback, first-enabled fallback, and model
correction writes. `tab_family_controls/items.rs` keeps `BuiltTabItem`, trigger response
aggregation, focus fallback, tab-list/panel assembly, and final `TabBarResponse` construction.

2026-05-27 tab trigger behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger/behavior.rs` now owns active-trigger
behavior installation, keyboard lifecycle marking, selected-model writes, activate-shortcut
handling, clicked transient reads, and `ResponseExt` population. `trigger.rs` now keeps tab trigger
props, collection a11y, keyed trigger assembly, and visual mounting.

2026-05-27 control chrome palette owner-split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/chrome.rs` now owns `ImUiControlPalette`, button
theme color resolution, field theme color resolution, and compact button/field container chrome.
`control_chrome.rs` keeps style constants, owner module wiring, and private re-exports for chrome,
layout, and text helpers.

2026-05-27 floating options owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_options.rs` is now a thin re-export index.
`floating_options/window.rs` owns `FloatingWindowResizeOptions`, `FloatingWindowOptions`,
`WindowOptions`, defaults, and builder methods. `floating_options/area.rs` owns
`FloatingAreaOptions`, `FloatingAreaContext`, area defaults, and context accessors. The opaque
context source gate now follows the area owner.

2026-05-27 floating drag-surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/drag_surface.rs` now owns
`floating_area_drag_surface_element(...)`, pointer-region wiring, double-click dispatch,
activation event recording, pointer drag move/up handling, setup callback invocation, and IMUI child
mounting. `floating_surface.rs` is now a module index/re-export hub for area, drag-surface, kinds,
layer, and state owners.

2026-05-27 drag response source/target owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/drag/source.rs` now owns `DragSourceResponse` storage,
inactive/new constructors, and source read accessors.
`ecosystem/fret-ui-kit/src/imui/response/drag/target.rs` now owns `DropTargetResponse<T>` storage,
empty construction, preview/delivered payload and position accessors, source id reads, and session
reads. `response/drag.rs` keeps generic `DragResponse` edge/motion storage plus source/target
re-exports.

2026-05-27 combo trigger behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/behavior.rs` now owns activate handling,
activate-shortcut handling, context-menu shortcut handling, transient events, and `ResponseExt`
population. `trigger.rs` keeps pressable props, ComboBox a11y, chrome/pill visual assembly, and
a11y label derivation.

2026-05-27 child-region resize axis owner-split result:
`ecosystem/fret-ui-kit/src/imui/child_region/resize/axis.rs` now owns X/Y handle width/height
constants, axis keys, resize cursors, and absolute handle layout. `resize.rs` keeps handle entry
points, response writes, pointer-region drag lifecycle wiring, and drag edge merging.

2026-05-27 child-region resize response owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize.rs` now owns
`ChildRegionResizeXResponse`, `ChildRegionResizeYResponse`, drag/min/max accessors, width/height
clamping helpers, and clamping tests. `child_region.rs` keeps aggregate `ChildRegionResponse`
storage, aggregate accessors, and resize response re-exports.

2026-05-27 text-picker pick-response merge owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/response.rs` now owns selected-value re-read,
`model_value_changed_for(...)` lookup, and `ResponseExt` merge writes for picked completion/history
candidates. `text_picker_controls.rs` keeps input/popup orchestration and final response assembly.

2026-05-27 text-picker open-policy owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/open_policy.rs` now owns popup open/panel-id
snapshot reads, expanded-state calculation, empty/exact-match close policy, and open-on-focus
anchoring. `text_picker_controls.rs` keeps completion/history entry points, input/popup
orchestration, keyboard reconciliation, and response assembly.

2026-05-27 debug-draw media summary projection owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/media.rs`
now owns image triangle mesh, image rect/region/rounded, image quad, and SVG rect summary
assembly. `summary_projection.rs` keeps clip-stack tracking and non-media command projection.

2026-05-27 begin-menu active-trigger open-policy owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy/active_trigger.rs`
now owns active-trigger open-menu synchronization, post-trigger menubar reconciliation, and
`MenubarActiveTrigger` group-active writes. `open_policy.rs` keeps trigger-click toggling,
open-request resolution, disabled-popup cleanup, and the private owner re-export.

2026-05-27 begin-submenu trigger/open-policy owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu/trigger.rs` now owns submenu
menu-item trigger assembly, submenu expanded semantics, shortcut forwarding, and
`sub_trigger::wire(...)` geometry hints.
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu/open_policy.rs` owns clicked-trigger
submenu-state reconciliation, stale-open cleanup, and popup open/close anchoring. `submenu.rs`
keeps public begin-submenu orchestration, state reads, popup mounting, and `DisclosureResponse`
assembly.

2026-05-27 table-column primitive option owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/collections/table_column/primitives.rs` now owns
`TableColumnWidth`, `TableColumnResizeOptions`, `TableSortDirection`, `TableColumnPin`, width
constructors, and default resize limits. `table_column.rs` keeps the `TableColumn` builder,
identity inference, accessor-first seams, and visibility/sort/resize/pin policy methods.

2026-05-27 floating-window resize drag-apply owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/drag_apply.rs` now owns resize drag
delta calculation, min/max clamping, left/top origin reconciliation, all eight handle branches,
and `last_resize_position` advancement. `state.rs` keeps lifecycle state lookup, collapsed/non-drag
reset policy, device-pixel snapping, resize output assembly, and handle test-id packaging.

2026-05-27 table-column visibility snapshot owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/state/snapshot.rs` now owns
`TableColumnVisibilitySnapshot`, `TableColumnVisibilityEntry`, serde derives, public data fields,
and snapshot/entry accessors. `state.rs` keeps runtime override storage, mutation helpers, snapshot
restore/apply orchestration, and column visibility policy application. The root IMUI re-export
surface and serde payload shape remain unchanged.

2026-05-27 table-column visibility menu identity owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu/identity.rs` now owns stable menu
column id extraction, visible menu label parsing, and generated test-id suffix slugs. `menu.rs`
keeps header context-menu composition, menu item/group rendering, model updates, and response
population. Public table-column visibility helpers and test-id behavior remain unchanged.

2026-05-27 debug-draw summary owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries.rs` is now a thin re-export index.
`summaries/command.rs` owns `DebugDrawCommandKind` plus per-command summary storage/accessors, and
`summaries/list.rs` owns aggregate list summary counters and classification. The public
`DebugDrawCommandSummary` / `DebugDrawListSummary` accessor-first contract remains unchanged.

2026-05-27 facade container-wrapper owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers.rs` is now a thin module index.
Flow wrappers live in `container_wrappers/flow.rs`, layout wrappers live in
`container_wrappers/layout.rs`, collection wrappers live in `container_wrappers/collections.rs`,
and menu/tab wrappers live in `container_wrappers/menu_tabs.rs`. `ImUiFacade` method names and
forwarding behavior remain unchanged, while the wrapper owner structure now mirrors the existing
`container_methods` split.

2026-05-27 debug-draw options sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options.rs` is now a thin module/re-export
index. Root canvas and interaction options live in `options/root.rs`, stroke style/path conversion
lives in `options/stroke.rs`, rounded-corner flags live in `options/round_corners.rs`, image/svg
option bags live in `options/media.rs`, and mesh vertices live in `options/vertex.rs`. The public
debug draw API and root `debug_draw_controls` re-export surface remain unchanged.

2026-05-27 debug-draw path-builder shape-method owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods.rs` now owns rect,
Bezier, arc, fast-arc, and elliptical-arc authoring methods plus their sampling/sanitization calls.
`path_builder.rs` keeps the path type, point-list basics, stroke/fill command recording, and
point-count/empty accessors. The public `ImUiDebugDrawPath` API remains unchanged.

2026-05-27 debug-draw paint media owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` now owns image, image-region,
image-quad, rounded-image, rounded-image-region, SVG image, and SVG mask-icon painting. Root
`paint.rs` keeps clip-stack balancing and command-class dispatch to media vs shape painters.
Debug-draw scene output and public authoring APIs remain unchanged.

2026-05-26 button visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` now owns button variant layout,
accessibility labels, arrow glyph/label mapping, and visual chrome/content assembly.
`button_controls.rs` keeps immediate pressable orchestration, keyboard shortcut/context-menu
handling, action dispatch, and response population. The public IMUI button APIs remain unchanged.
The same verification pass repaired the existing DropdownMenuLabel source-policy drift by routing
that shadcn label through the shared `text_menu_group_label(...)` role.

2026-05-26 button behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/behavior.rs` now owns button action payload
storage, command gating, pressable construction, shortcut/context-menu handling, command dispatch
source metadata, payload forwarding, and `ResponseExt` population. `button_controls.rs` now keeps
the public entry routing and label-identity scope only, while `button_controls/visual.rs` remains
the layout/a11y/chrome owner. The public button, small-button, arrow-button, invisible-button, and
action-button APIs remain unchanged.

2026-05-27 control chrome text owner-split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/text.rs` now owns compact control text helpers,
caption color routing, and pill badge chrome. At this slice, `control_chrome.rs` still kept style
constants, `ImUiControlPalette`, button/field chrome, row/stack layout props, and test module
wiring; the later 2026-05-27 chrome owner split moved palette/button/field chrome out too.
Existing `control_chrome::control_text`, `fill_text`, `caption_text`, and `pill` call paths remain
unchanged through the private root re-export.

2026-05-27 control chrome layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/layout.rs` now owns shared IMUI row/stack flex
helper props. At this slice, `control_chrome.rs` still kept style constants, `ImUiControlPalette`,
button/field chrome, text helper re-exports, and test module wiring; the later 2026-05-27 chrome
owner split moved palette/button/field chrome out too. Existing `fill_row_props`,
`centered_row_props`, and `fill_stack_props` call paths keep row direction, fill-width behavior,
gap tokens, justification, and alignment.

2026-05-26 text-picker owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/candidates.rs` now owns candidate filtering,
`max_items`, exact-match hiding, and open-when-empty visibility decisions.
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard.rs` now also owns keyboard active
source reconciliation and pending keyboard pick extraction. `text_picker_controls.rs` keeps the
input/popup composition, selectable item rendering, command-free model updates, and response
merging. The public input-text completion/history APIs remain unchanged.

2026-05-26 text-picker popup owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/popup.rs` now owns popup mounting,
popup-scoped keyboard handler installation, candidate selectable rows, active-element
synchronization, clicked candidate commits, popup close, and picked-result reporting.
`text_picker_controls.rs` keeps input composition, assistive semantics, open/close policy,
candidate/keyboard snapshots, and final `InputTextPickerResponse` merge. The public completion and
history picker APIs remain unchanged.

2026-05-27 text-picker input-root owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/input.rs` now owns picker input option/test-id
preparation, ComboBox semantics normalization, assistive semantics, root fill container
construction, text input mounting, and input-focused keyboard handler installation.
`text_picker_controls.rs` keeps candidate visibility, popup-open state reads, keyboard-state
snapshot reconciliation, popup lifecycle policy, popup rendering delegation, and final
`InputTextPickerResponse` merge. Completion/history picker behavior, active-descendant wiring,
test-id derivation, and picked response semantics remain unchanged.

2026-05-26 disclosure spec owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/spec.rs` now owns the private
`DisclosureKind` / `DisclosureSpec` model, option normalization, level clamping, test-id routing,
and leaf/children classification. `disclosure_controls.rs` keeps immediate pressable behavior,
keyboard/context-menu handling, open-model updates, content mounting, and `DisclosureResponse`
population. The public collapsing-header and tree-node APIs remain unchanged.

2026-05-26 disclosure trigger owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger.rs` now owns header pressable
construction, shortcut activation, context-menu key/right-click handling, double-click signaling,
hover-delay reads, enabled sanitization, and trigger `ResponseExt` population.
`disclosure_controls.rs` keeps label identity normalization, spec/open-model setup, content
mounting, and aggregate open/toggled response state. The public collapsing-header and tree-node
APIs remain unchanged.

2026-05-27 disclosure header-row visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/header.rs` now owns header row
container/flex assembly, indicator glyph mounting, label text mounting, row padding, border, and
radius props. `disclosure_controls/visual.rs` keeps disclosure a11y, content padding, and palette
resolution. Trigger pressable behavior, shortcut/context-menu handling, indicator glyphs, label
text roles, indentation, and row chrome remain unchanged.

2026-05-26 combo trigger owner-split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger.rs` now owns ComboBox pressable
construction, accessibility label derivation, shortcut activation, context-menu key handling,
trigger `ResponseExt` population, and the open/menu badge chrome. `combo_controls.rs` keeps label
identity normalization, popup open/close model wiring, popup mounting, and aggregate
`ComboResponse` open/toggled state. The public combo and combo-model facade APIs remain unchanged.

2026-05-26 boolean visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/visual.rs` now owns checkbox badges, radio
indicators, switch state badges, and shared boolean label text. `boolean_controls.rs` keeps
checkbox/radio pressable orchestration, shortcut/context-menu handling, and response population,
while `boolean_controls/switch.rs` keeps switch active-trigger behavior and model updates. The
public checkbox, radio, and switch APIs remain unchanged.

2026-05-26 hover query owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/flags.rs` now owns `ImUiHoveredFlags`, while
`ecosystem/fret-ui-kit/src/imui/response/hover/query.rs` owns the ImGui-style hovered query
helpers. `response/hover.rs` keeps `ResponseExt` storage, crate-local mutators, public accessors,
and drag convenience helpers. The public hover flags and `ResponseExt` API remain unchanged.

2026-05-26 lifecycle owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/lifecycle.rs` now owns the `ResponseExt`
lifecycle signal mutators, merge helpers, clearing, and read-only accessors for activation,
deactivation, edits, and deactivate-after-edit. `response/hover.rs` keeps lifecycle storage but no
longer owns lifecycle method bodies. The public `ResponseExt` API remains unchanged.

2026-05-26 press/context owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/press_context.rs` now owns the `ResponseExt`
secondary-click, double-click, long-press, hold, context-menu, pointer-click, pointer-modifier, and
clear helpers plus read-only accessors. `response/hover.rs` keeps storage for those signals only.
The public `ResponseExt` API remains unchanged.

2026-05-26 hover-state owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/hover_state.rs` now owns the `ResponseExt` raw
pointer-hover, popup-barrier hover, hover-delay, active-item block, and nav-highlight mutators plus
read-only accessors. `response/hover.rs` keeps the hover state storage fields only. The public
`ResponseExt` API remains unchanged.

2026-05-26 core-state owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/core_state.rs` now owns the `ResponseExt`
core-response, id, enabled, clicked, changed, rect, hover, press, and focus mutators/accessors.
`response/hover.rs` keeps core/id/enabled storage only. The public `ResponseExt` API remains
unchanged.

2026-05-26 interaction-runtime hover owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay.rs` now owns window-scoped
shared hover delay state, clear timers, and shared timer transitions. `hover/timers.rs` owns
deterministic per-element hover timer token derivation, and `hover/long_press.rs` owns long-press
timer emission. `interaction_runtime/hover.rs` keeps the exported hover query helpers, active-item
block read, local delay state accumulation, and response readout. The public hover and long-press
behavior remains unchanged.

2026-05-26 interaction-runtime drag owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/active_item.rs` now owns active-item
set/clear helpers, `drag/long_press_timer.rs` owns long-press arm/cancel, `drag/pointer_region.rs`
owns pointer-region drag/resize lifecycle, and `drag/response.rs` owns pressable drag response
population. `interaction_runtime/drag.rs` keeps drag-kind/threshold helpers and the pressable drag
state machine. Pressable drag, floating-window resize/move, active-item blocking, and long-press
behavior remain unchanged.

2026-05-27 floating-area composition owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/area.rs` now owns floating-area layer
registration, drag snapshot application, state/test-id updates, IMUI facade content mounting,
absolute area layout, no-input/pass-through gates, and `FloatingAreaResponse` assembly.
`floating_surface.rs` keeps drag-surface pointer-region behavior, layer/kind/state re-exports, and
module wiring. Floating-area position, dragging, test-id, no-inputs, pointer pass-through, and
response semantics remain unchanged.

2026-05-26 menu-family menu owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu.rs` now owns top-level
`begin_menu_with_options(...)` menu open/close orchestration, trigger wiring, menubar active-menu
policy updates, and popup mounting. `menu_family_controls.rs` keeps menubar policy state,
menu-bar element construction, module wiring, and tests. The public facade menu API remains
unchanged.

2026-05-26 debug-draw response owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/response.rs` now owns `DebugDrawResponse`
storage and accessors. The opaque-output source gate follows the new owner, while
`debug_draw_controls.rs` re-exports the public surface. The public debug draw response API remains
unchanged.

2026-05-26 debug-draw options owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options.rs` now owns the public debug draw
options, stroke/rounding/image/svg options, and mesh vertex helper types. `debug_draw_controls.rs`
re-exports those types; later owner splits moved draw-list state to `draw_list.rs` and facade entry
glue to `facade.rs`. The public debug draw API remains unchanged.

2026-05-27 multi-select state owner-split result:
`ecosystem/fret-ui-kit/src/imui/multi_select/state.rs` now owns `ImUiMultiSelectState`,
ordered-selection normalization, anchor repair, and crate-local mutation helpers. The root
`multi_select.rs` keeps model hook, selectable response wiring, click-modifier policy, and response
changed reporting, so collection helper state remains accessor-first without broadening the public
surface.

2026-05-27 virtual-list runtime/row owner-split result:
`ecosystem/fret-ui-kit/src/imui/virtual_list_controls/runtime.rs` now owns runtime option projection
and list viewport layout. `virtual_list_controls/row.rs` owns row packing, row test-id suffixing,
row-height resolution, striped row chrome, and fixed-height clipping. The root
`virtual_list_controls.rs` keeps keyed list assembly, focus child mounting, render-range tracking,
and list-level semantics.

2026-05-26 popup-menu policy/panel owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/policy.rs` now owns menu navigation state,
popup submenu policy state, and root submenu synchronization. `popup_overlay/menu/panel.rs` now
owns popper placement, menu semantics, nav-state installation, panel chrome, IMUI child mounting,
and focus-target extraction. `popup_overlay/menu.rs` keeps begin-popup orchestration, menubar
policy lookup, dismiss/focus hooks, and overlay request dispatch. Popup/menu/submenu public facade
behavior remains unchanged. The 2026-05-27 popup-menu panel follow-up above splits the concrete
layout/chrome/content bodies behind `panel.rs`.

2026-05-27 popup modal layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layout.rs` now owns modal palette resolution,
centered panel geometry, absolute layer/backdrop props, dialog semantics layout, and panel chrome
props. `popup_overlay/modal.rs` keeps popup store reads, keepalive generation, Escape and outside
press dismissal policy, focus handoff, IMUI facade content mounting, and `OverlayRequest::modal`
assembly. Modal open/close behavior, barrier semantics, centered placement, and test ids remain
unchanged.

2026-05-26 menu-item interaction owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` now owns menu item enabled/action
gating, pressable props, activation and shortcut handlers, popup menu roving focus, menubar
horizontal-arrow switching, command dispatch source metadata, and `ResponseExt` population.
`menu_controls/element.rs` keeps the row panel, checkbox/radio/submenu indicators, shortcut text,
label text, and custom `pressable_hook` insertion point. Public menu item and command menu item
facade APIs remain unchanged.

2026-05-26 menu-item keyboard owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard.rs` now owns item-local activate shortcut
handling, popup menu roving focus, menubar close-auto-focus suppression, and horizontal-arrow menu
switching. `menu_controls/interaction.rs` keeps enabled/action gating, pressable props, activation
dispatch, and response population. Public menu item, command menu item, submenu, and menubar
keyboard behavior remain unchanged.

2026-05-27 menu-item behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/interaction/behavior.rs` now owns active-trigger
installation, activate-handler popup close/click signaling, command dispatch source metadata,
clicked transient draining, keyboard owner wiring, and `ResponseExt` population.
`menu_controls/interaction.rs` keeps menu-item interaction structs, enabled/action gating,
pressable prop construction, and thin forwarding call sites for element/keyboard users. Public menu
item, command menu item, submenu, and menubar behavior remain unchanged.

2026-05-27 selectable behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/selectable_controls/behavior.rs` now owns pressable behavior
installation, activate-handler popup close/click signaling, keyboard owner delegation, transient
clicked reads, and `ResponseExt` population. `selectable_controls.rs` keeps label identity,
`SelectableOptions` a11y wiring, selected/highlighted state reads, and row visual assembly.

2026-05-26 textarea owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/textarea.rs` now owns textarea props assembly,
lifecycle/response population, select-all-on-focus command emission, submit/cancel command policy
installation, and text-area chrome/text-style selection. `text_controls.rs` keeps input-text
assembly plus shared helper routing. Public `textarea_model(...)` and `textarea_model_with_options`
facade behavior remains unchanged.

2026-05-26 floating-window resize state owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` now owns active resize snapshot
lookup, drag delta application, min/max size clamping, left/top origin updates, collapse reset,
device-pixel snapping, and resize state/test-id output. `floating_window_resize.rs` is now a thin
`handles`/`state` index plus the shared resize-handle test-id record; `handles.rs` still owns
pointer-region handle rendering and drag lifecycle wiring.

2026-05-26 floating-window resize snapshot owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/snapshot.rs` now owns active resize drag
discovery and snapshot capture. `state.rs` now focuses on resize delta application, min/max
clamping, origin updates, collapse reset, device-pixel snapping, and output assembly. Public
floating-window facade behavior and internal `floating_window_resize::current_resize_snapshot(...)`
call sites remain unchanged.

2026-05-26 floating-window resize handle owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/layout.rs` now owns handle geometry
and resize cursors, while `handles/pointer.rs` owns pointer-region wiring, pointer capture,
runtime drag begin/update/cancel, cursor updates, and activation handoff. `handles.rs` now only
stacks body/blocker with the eight resize handles.

2026-05-27 begin-menu state owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state.rs` now owns begin-menu state
capture, row/popup/was-open models, menubar open-menu synchronization, active trigger state writes,
open-request resolution, disabled-popup cleanup, and render-state recording. `menu.rs` now keeps
begin-menu flow orchestration, trigger mounting, popup mounting, and final `DisclosureResponse`
assembly.

2026-05-27 begin-menu state capture owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/capture.rs` now owns
`BeginMenuState`, `MenuRenderState`, row/popup/was-open model capture, row/open-menu reads, and
render-state recording. `menu_state.rs` now focuses on menubar open-menu mutation, active-trigger
synchronization, open-request resolution, and disabled-popup cleanup.

2026-05-27 begin-menu open-policy owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy.rs` now owns menubar
open-menu synchronization, active-trigger writes, trigger-click toggling, open-request resolution,
and disabled-popup cleanup. `menu_state.rs` is now a thin capture/open-policy module index.

2026-05-27 table header row owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header_row.rs` now owns the keyed header row,
visible header cell assembly, sortable/plain wrapper selection, resize response initialization,
`TableHeaderResponse` collection, and header row wrapping. `table_controls/render.rs` keeps table
palette, visible-column, horizontal-scroll, and header-presence decisions plus body rows, root
chrome, semantics, and final `TableResponse` assembly. Public IMUI table APIs remain unchanged.

2026-05-27 table header label owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header/labels.rs` now owns visible-label parsing,
sort-indicator text, sortable a11y labels, header content boxes, and header label text. The root
`header.rs` keeps sortable/plain header-cell assembly and resize-handle wrapping while re-exporting
the same `header::visible_header_label`, `header::column_is_sortable`,
`header::table_header_label_text`, and `header::table_sort_indicator_text` call surface.

2026-05-27 table row-group owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/row_groups.rs` now owns pinned-cell splitting,
left/center/right row-group assembly, horizontal center-scroll wrapping, and the shared horizontal
flex primitive. `table_controls/body.rs` keeps `PreparedTableCell`, `TablePalette`, row
semantics/background selection, and cell wrapping. Public IMUI table APIs remain unchanged.

2026-05-27 pressable item response owner-split result:
`ecosystem/fret-ui-kit/src/imui/item_behavior/response.rs` now owns shared pressable item response
population: transient signal reads, context anchor/modifier reads, drag response merging, hover
query hook installation, and final `ResponseExt` population. `item_behavior.rs` keeps pressable
hook installation, active-item/long-press/lifecycle/context-menu models, pointer-up transient
emission, and the existing `item_behavior::populate_pressable_item_response(...)` re-exported call
surface. Public IMUI widget APIs remain unchanged.

## M5 - Worktree Convergence

Exit criteria:

- Dirty IMUI changes in `main` and `imui-imgui-editor-grade-refactor` are checkpointed before
  integration.
- `main` remains the integration base and the only continuation branch.
- Overlapping `fret-ui-kit::imui`, `fret-imui`, demo, workstream, and source-gate changes are
  merged by topic with no unresolved conflict markers.
- Focused convergence gates pass or have an explicit recorded reason for a narrower substitute.

Result: done on 2026-05-26. The main checkpoint is
`d078e25122 refactor(imui): checkpoint gap closure convergence slices`; the editor-grade worktree
checkpoint is `05727e284b refactor(imui): checkpoint editor-grade convergence worktree`; the merge
resolution keeps the editor-grade facade/container/listbox organization, preserves the main
image-item owner split, and records the verification evidence in `EVIDENCE_AND_GATES.md`.

## M0 - Current Source Baseline

Exit criteria:

- The lane exists with the minimum doc set.
- The current source audit names current Fret capabilities and Dear ImGui comparison anchors.
- The old standalone parity audit has a status note explaining how to read it.
- Repo trackers point to this lane for current gap-closure sequencing.
- P0 doc/source gates are run.
  Result: done on 2026-05-06.

## M1 - Fearless Cleanup Candidate Selection

Exit criteria:

- One cleanup/delete candidate is chosen from current source evidence.
- The candidate has a named owner crate/doc surface.
- The candidate has a focused gate and a rollback-free delete/refactor plan.
- Any closed historical lane remains closed unless a narrower follow-on is created.
  Current first candidate selected: teaching-surface cleanup for `imui_shadcn_adapter_demo`.
  The first code slice routes `TableSortDirection` through `fret::imui::kit` and extends the
  teaching-source gate so the stale direct `fret_ui_kit::imui::TableSortDirection` import cannot
  return to that default-path example.
  The second code slice routes `workspace_shell_demo` pane-proof option types through
  `fret::imui::kit` and extends both IMUI source gates so direct `fret_ui_kit::imui` imports cannot
  return to that default pane-first proof.
  The third code slice routes `imui_editor_proof_demo` and its `collection.rs` module through the
  app-facing `fret::imui` facade for IMUI option/state types while keeping recipe-layer imports
  explicit. `fret-ui-editor::imui` was audited and remains a thin declarative-control adapter.
  2026-05-16 teaching-comment cleanup: `imui_interaction_showcase_demo` now names the root
  `fret::imui` lane instead of stale direct-`fret_imui` control-flow wording, with a source-gate
  marker preventing that drift from returning.

## M2 - First Cleanup/Refactor Slice

Exit criteria:

- The first P1 slice lands.
- Public teaching surfaces still prefer the app-facing `fret::imui` path.
- `fret-imui` stays policy-light.
- `fret-ui-editor::imui` stays a thin adapter.
- Focused gates pass.
  Result: done for the P1 import cleanup pass on 2026-05-06. Remaining P1 item is duplicate helper
  alias deletion, if source evidence finds a real alias worth removing.
  P1 closeout result: no additional alias delete is justified. The debug draw owner cleanup was
  split into `imui-debug-draw-owner-split-v1` and that follow-on is now closed.

## M3 - User-Usable Golden Path

Exit criteria:

- A single runnable proof teaches a realistic editor panel path.
- It combines immediate authoring, editor controls, actions/commands, popup/menu behavior, and
  diagnostic hooks.
- Cookbook/docs point at that proof without promoting historical smoke demos as the default path.
  Current proof surface: `apps/fret-examples/src/imui_editor_proof_demo.rs` with the demo-local
  `collection.rs` module. The proof has been gated with focused collection source tests plus
  `cargo check -p fret-demo --bin imui_editor_proof_demo`; cookbook/docs promotion now points from
  the focused IMUI cookbook lessons to the heavier `fret-demo` proof without turning it into a
  boring-ladder cookbook example.

## M4 - Follow-On Split

Exit criteria:

- Remaining Dear ImGui-class gaps are split into narrow lanes with owner, repro, gate, and evidence.
- This lane remains the source-audit and priority map, not a dumping ground for all implementation.
  Current public-surface audit result: keep the existing owner split. `fret-imui` stays
  policy-light, apps teach the opt-in `fret::imui` lane, and policy-heavy widgets remain under
  `fret::imui::kit` / `editor` / `docking`. New public helpers need two proof surfaces plus a
  focused gate before they become default authoring vocabulary.
  2026-05-14 cleanup result: `FloatingAreaContext` is now externally opaque, with read-only
  accessors for facade-created area id, position, and drag kind.
  2026-05-14 follow-up result: floating responses now expose area/window state through accessors
  instead of public fields.
  2026-05-14 response follow-up result: disclosure and combo responses now expose trigger state
  through `response()`, keep trigger/open/toggle storage crate-local, and no longer allow external
  default construction.
  2026-05-14 text-picker follow-up result: `InputTextPickerResponse` now exposes input state and
  pick results through accessors while keeping its storage crate-local.
  2026-05-14 tab follow-up result: tab-bar aggregate and tab-trigger responses now keep selection /
  trigger storage crate-local while preserving `selected_id()`, `selected_changed()`, `trigger(...)`,
  and trigger edge accessors.
  2026-05-14 virtual-list follow-up result: `VirtualListResponse` now keeps scroll handle and
  rendered-range storage crate-local while retaining `handle()` and `rendered_range()`.
  2026-05-14 table follow-up result: table aggregate/header/resize responses now keep metadata and
  drag storage crate-local while exposing explicit header and resize accessors.
  2026-05-14 drag follow-up result: `DragResponse` now keeps edge/delta storage crate-local while
  preserving read-only drag accessors and `ResponseExt` helper methods.
  2026-05-14 drag/drop follow-up result: `DragSourceResponse` and `DropTargetResponse` now keep
  storage and construction paths crate-local, while public callers use helper-returned response
  accessors.
  2026-05-14 response drag-state follow-up result: `ResponseExt` now keeps aggregate drag response
  storage crate-local as well; public code stays on `drag()` and the higher-level drag edge/motion
  helpers, while internal response assemblers populate it through crate-local mutators.
  2026-05-14 press/context follow-up result: `ResponseExt` press and context-menu derived signal
  storage is private, with runtime assembly routed through crate-local setters and public use routed
  through accessors.
  2026-05-14 lifecycle follow-up result: `ResponseExt` activation/edit/deactivation edge storage
  is private too; runtime lifecycle assembly and combo/text-picker edit merging use crate-local
  helpers while public callers keep accessor-only reads.
  2026-05-14 hover/nav follow-up result: `ResponseExt` raw hover, hover-delay, active-item block,
  and nav-highlight storage is private too; pressable/disclosure assembly uses crate-local setters
  and tests read through accessors.
  2026-05-14 enabled follow-up result: `ResponseExt.enabled` storage is private too; public readers
  use `enabled()` and runtime/text-control assembly uses crate-local `set_enabled(...)`. `core`
  stayed out of that slice pending a separate contract audit.
  2026-05-14 identity follow-up result: `ResponseExt.id` storage is private too; public readers use
  `id()` and response assembly uses crate-local `set_id(...)`.
  2026-05-14 shared-response follow-up result: `ResponseExt.core` storage is private too; public
  readers use `core()`, `from_core(...)`, and the small signal accessors (`rect()`, `focused()`,
  `hovered()`, `pressed()`, `clicked()`, `changed()`), while response assembly uses crate-local core
  setters. This preserves the shared `fret_authoring::Response` compatibility surface without
  keeping public field mutation.
  2026-05-14 adapter-seam follow-up result: emitted adapter signal records are read-only now.
  `AdapterSignalRecord` and `AdapterSignalMetadata` are constructed by the seam and expose
  `identity()`, `response()`, `metadata()`, `rect()`, and `focus_restore_target()` accessors, while
  `AdapterSeamOptions` remains a public-field input options bag.
  2026-05-14 editor drag-value follow-up result: `DragValueCoreResponse` storage is private too and
  external default construction is gone. `DragValueCore` owns construction through a crate-local
  constructor, and editor controls consume scrub visual state through read-only accessors instead of
  copying response fields.
  2026-05-14 debug-draw follow-up result: `DebugDrawResponse` now follows the same accessor-first
  rule. Interaction and summary storage are private, helper construction is internal, and public
  callers use `response()`, `list_summary()`, and `command_summaries()`.
  2026-05-14 debug-draw summary follow-up result: `DebugDrawCommandSummary` and
  `DebugDrawListSummary` are accessor-first too. Debug-draw diagnostic metrics remain public to
  read, but construction and mutation stay internal to the list/response pipeline.
  2026-05-14 source-gate hardening result: `tools/gate_imui_workstream_source.py` now has a
  reusable opaque-output-struct check for the sealed IMUI response/context/summary records instead
  of relying only on one-off public-field marker strings.
  2026-05-14 editor axis-outcome follow-up result: vector and transform axis edit outcome records
  are accessor-first too, so proof/app code can observe edit events without constructing or
  mutating invalid section/axis/outcome triples.
  2026-05-14 output-catalog gate result: the source gate now auto-discovers new public IMUI/editor
  output-style structs by suffix and requires every match to be registered in the opaque-output
  catalog before field-opacity checks run.
  2026-05-14 editor color-event result: color-edit event/request/payload records are accessor-first
  too. `ColorEditPaletteSlotDrop`, `ColorEditEyedropperRequest`, and
  `ColorEditDragDropPayload` keep storage private while preserving callback/event reads through
  explicit methods.
  2026-05-14 state-catalog gate result: the opaque public-struct catalog now scans `*State` names
  and registers `ImUiMultiSelectState`, so shared state helpers cannot reintroduce public storage
  outside the accessor/constructor contract.
  Current component-surface audit result: do not open a broad widget-backlog lane. The current
  `fret-ui-kit::imui` surface already covers the editor-proof path across controls, text,
  disclosure, menus/popups/tooltips, tabs, tables, drag/drop, child regions, virtual lists, and
  debug draw. The 2026-05-13 owner splits changed private file ownership for focused facade wrappers
  and pressable response assembly, including the container wrapper owner split, but did not widen
  the public surface. List-box, plotting, image item, style-editor, advanced-table, and child-flag
  work should be narrow proof-led follow-ons.
  2026-05-16 image-item follow-on result: `imui-image-item-proof-v1` is the narrow proof lane for
  response-bearing image item / image button authoring. It is intentionally a `fret-ui-kit::imui`
  additive helper over existing `ImageId` / `ImageProps`, not a `fret-imui` runtime texture stack.
  Current design-surface audit result: keep imgui-class density as an opt-in editor token/preset
  outcome. `EditorThemePresetV1::ImguiLikeDense` is sufficient for the active proof; do not copy
  Dear ImGui's mutable style stack or make a generic style editor without visual/tooling proof.
  2026-05-14 cleanup result: the unused `apply_editor_theme_patch_v1` compatibility wrapper was
  deleted; explicit preset entry points remain the only editor theme patch authoring path.
  Current porting-sugar audit result: keep `SameLine` / item-width / label-ID sugar candidate-only
  until at least two proof surfaces pay the same authoring tax. Prefer typed Fret helpers
  (`horizontal_with_options`, `PropertyGrid::row_with`, explicit `id_source` / `test_id`) over
  copying Dear ImGui's mutable cursor, item-width stack, or label suffix parser.
  2026-05-14 cleanup result: the unused public `PropertyGridRow` wrapper was deleted so the grid
  row authoring surface stays on the canonical `PropertyGridRowCx::row(...)` / `row_with(...)`
  path instead of growing a second row-policy wrapper.
  2026-05-14 follow-up result: both eager and virtualized grid row contexts are now opaque; row
  options stay crate-local while external callers use row helpers instead of public fields.
  2026-05-17 text-role follow-up result: eager and virtualized grid row contexts now expose
  `label_text(...)` for fixed inspector row labels. `PropertyRow` also clamps label slot line boxes
  to the editor row height, so default/bare label text cannot wrap and grow fixed inspector rows
  under resize.
  2026-05-17 proof teaching follow-up result: `imui_editor_proof_demo` property-grid labels now
  use `row_cx.label_text(...)`, and the source gate rejects representative proof labels if they
  return to bare `|cx| cx.text(...)` label slots.
  2026-05-19 editor proof main text result: `imui_editor_proof_demo` main proof chrome and
  explanatory copy now use proof-local IMUI helpers backed by shared section-chrome,
  control-readout, and compact paragraph roles instead of local `fret_ui_kit::ui::text(...)`
  styling.
  2026-05-19 workspace shell paragraph text result: the remaining editor-rail header copy now uses
  `workspace_shell_paragraph_text(...)` backed by shared `text_paragraph(...)` instead of local
  `fret_ui_kit::ui::text(...).text_sm().text_color(...)` styling.
  2026-05-19 editor notes center/collection text result: `editor_notes_demo` collection summary
  and center preview text now use local helpers backed by shared readout, section, and paragraph
  roles instead of local `ui::text(...).wrap(...)` styling.
  2026-05-19 editor notes device shell text result: `editor_notes_device_shell_demo` compact
  mobile header title/body copy now uses device-shell local helpers backed by shared
  section-chrome and paragraph roles instead of local `ui::text(...)` styling.
  2026-05-19 editor popup-list text ownership result: popup list row, empty, centered-row, and
  fixed-caption text props now live with the shared editor text roles in
  `ecosystem/fret-ui-editor/src/primitives/readout.rs`. `popup_list.rs` keeps popup-list geometry,
  state, and palette policy only, and the source gate rejects direct text props/wrap policy there.
  2026-05-14 inspector follow-up result: `InspectorPanelCx` now exposes query behavior through
  methods and keeps `query_lower` private.
  2026-05-16 child-region resize result: `imui-child-region-resize-y-v1` and
  `imui-child-region-resize-x-v1` are the closed proof lanes for axis-specific manual child-region
  resize. Height/width state stays app-owned through response helpers, and broader child-region
  behavior such as auto-resize, clipping-return, or nav-flattening remains candidate-only.
  2026-05-18 child-region auto-height result: a focused `fret-imui` composition gate now proves the
  Fret-native AutoResizeY-equivalent posture: width-constrained child regions with no explicit
  height auto-size to measured content and push following siblings down. This keeps the current
  layout contract explicit without adding a Dear ImGui `AutoResizeY` flag mirror.
  2026-05-20 child-region auto-width result: a focused `fret-imui` composition gate now proves the
  matching Fret-native AutoResizeX-equivalent posture: height-constrained child regions with no
  explicit width auto-size to measured content and push following siblings right. This keeps the
  current layout contract explicit without adding a Dear ImGui `AutoResizeX` flag mirror.
  2026-05-16 selectable highlight result: `imui-selectable-highlight-policy-v1` is the closed proof
  lane for forced selectable highlight visuals. Keyboard-active picker rows now use highlighted
  policy instead of selected semantics, while broader selectable flags remain candidate-only.
  2026-05-16 floating posture refresh result: the public `window(...)` docs no longer describe
  z-order/focus arbitration as future work. `fret-ui-kit::imui` owns the in-window floating policy
  knobs through `WindowOptions` / `FloatingWindowOptions`, and `fret-imui` floating tests cover
  bring-to-front hit-test order, focus-on-click vs activation, no-inputs / pointer-pass-through,
  close, resize, and collapse. Multi-window / viewport parity remains outside this helper.
  2026-05-16 table gap wording result: the component catalog now treats alternating row backgrounds
  as covered through `TableOptions::striped` and narrowed the remaining table gap read before the
  row/cell override proof landed.
  2026-05-16 table override/text role result: explicit row/cell background override policy landed
  in `TableRowOptions::background` and `TableCellOptions::background`, with scene-paint proof that
  cell overrides paint after row overrides. `ImUiTableRow::cell_text(...)` now uses the shared
  `text_table_cell(...)` role helper, so default table text no longer inherits paragraph wrapping
  semantics. At that point, freeze panes, visibility persistence, and old columns API shape were
  still advanced-table candidates; later results below narrow or close those axes.
  2026-05-16 table header text result: sortable and plain table header labels also use
  `text_table_cell(...)`, preserving the same compact single-line ellipsis semantics as body cells.
  2026-05-16 static table column visibility result: `TableColumn::hidden()` and
  `TableColumn::with_visible(bool)` now cover author-declared hidden columns without copying Dear
  ImGui's mutable table runtime. Hidden columns still consume submitted row cells in declared order
  but skip header/body rendering and header responses; runtime hideable-column policy is covered
  by the follow-up state/helper chain below, while persistence stays candidate-only.
  2026-05-17 runtime table column visibility result: `ImUiTableColumnVisibilityState` now covers
  runtime stable-id visibility overrides as a policy-layer helper in `fret-ui-kit::imui`. It
  produces an adjusted `TableColumn` list and reuses the existing hidden-column render contract.
  Header menu policy is now covered by the helper chain below; persistence, freeze panes, and old
  columns API shape were still candidate-only at that historical point. A `fret-imui` composition
  gate proves the helper can drive table rendering while the runtime facade remains policy-light.
  2026-05-17 table visibility menu-item result: `table_column_visibility_menu_item(...)` now
  bridges `TableColumn`, existing checkbox menu item behavior, and
  `ImUiTableColumnVisibilityState`. Callers can still own a custom menu surface; the default header
  context-menu surface is covered by the helper below. Persistence, freeze panes, and old columns
  API shape were still candidate-only at that historical point.
  2026-05-17 table visibility menu-items group result:
  `table_column_visibility_menu_items(...)` now covers the repeated "show/hide columns" menu
  section for stable-id, human-labeled columns. The helper returns opaque/accessor-first item
  responses and feeds the header context-menu helper below without moving popup/menu policy into
  `fret-imui`.
  2026-05-17 table header trigger surface result: sortable and plain table headers now share the
  same private header trigger surface. Sortable headers keep button-like primary activation, while
  plain headers expose context-menu request signals without reporting left-click click/activation
  lifecycle.
  2026-05-27 table header trigger behavior owner-split result:
  `table_controls/header/trigger/behavior.rs` now owns active-trigger behavior installation,
  sortable keyboard activation lifecycle marking, clicked transient draining for plain headers, and
  `ResponseExt` population. `trigger.rs` keeps header trigger props, a11y/key-activation policy,
  keyed surface assembly, and sortable header visual construction.
  2026-05-17 table header visibility menu wiring result:
  `table_column_visibility_header_context_menu(...)` now bridges `TableResponse` header context
  requests from both sortable and plain headers, popup placement, and column visibility menu items.
  It returns an opaque/accessor-first response, exposes popup/menu policy through
  `TableColumnVisibilityHeaderContextMenuOptions`, keeps the visibility model caller-owned, and
  left persistence, freeze panes, and old columns API shape candidate-only at that historical point.
  2026-05-17 table visibility snapshot result:
  `TableColumnVisibilitySnapshot` and `TableColumnVisibilityEntry` now close the narrow
  persistence seam for runtime column visibility without introducing a table-state runtime.
  `ImUiTableColumnVisibilityState::snapshot()`, `from_snapshot(...)`, and
  `replace_from_snapshot(...)` round-trip stable column ids and visible flags through a serde data
  shape. Empty ids are ignored on restore and duplicate ids use last-entry-wins; applications still
  own storage, schema placement, and when to apply restored state. Later entries below close the
  freeze-pane seam and old API-shape cleanup.
  2026-05-17 table column pinning result: `TableColumn::pinned_left()` and
  `TableColumn::pinned_right()` now cover the first IMUI freeze-pane slice without copying Dear
  ImGui's table runtime. The helper render path splits visible header/body cells into frozen
  left/right groups plus a shared-scroll center group, keeps scroll state caller-owned when a
  `horizontal_scroll` handle is supplied, and falls back to an element-local scroll handle inside
  `fret-ui-kit::imui` when pinned columns need one. `fret-imui` stays a thin composition facade.
  Old columns API shape was still candidate-only at that historical point.
  2026-05-18 table column API-shape first-pass result: `TableColumn` now exposes accessor-first
  reads for its public option data, and table rendering, column-visibility policy, `fret-imui`
  composition tests, and public smoke tests use those accessors. This reduces the teaching/API
  pressure from direct field reads and prepared the private-field cleanup below.
  2026-05-18 table column private-field hardening result: after auditing current Fret call sites,
  `TableColumn` is no longer a public field bag. The fields are private, public callers stay on
  builder/accessor methods, internal render/policy code uses crate-local Arc/mutator seams, and the
  IMUI workstream source gate now fails if the old public fields return.
  2026-05-18 current advanced-table gap read: visibility snapshot/restore, freeze-pane pinning, and
  the old `TableColumn` public field-bag API shape are closed or narrowed. Remaining broad table
  work should not reopen them as current gaps; only app/editor storage/schema policy or a concrete
  Dear ImGui table-runtime parity proof should open a new lane.
  2026-05-16 control readout text role result: `text_control_readout(...)` now sits in
  `fret-ui-kit::declarative::text` beside `text_table_cell(...)`. The UI Gallery code-editor
  readouts still use the doc-layout app helper, but that helper delegates to the shared role, so
  toolbar/status readouts get muted compact single-line truncation without app-local text policy.
  2026-05-16 button label text role result: `text_button_label(...)` now gives IMUI button/pill
  labels a shared compact single-line truncation role instead of the previous word-wrapping
  `control_text(...)` behavior.
  2026-05-16 code block text role result: `text_code_block(...)` now shares the monospace
  single-line code-block text contract, and UI Gallery docs code blocks no longer hand-roll their
  own `TextProps`.
  2026-05-16 paragraph text role result: `text_paragraph(...)` and
  `text_paragraph_break_words(...)` now close the first shared text-role vocabulary pass while
  keeping `text_prose(...)` as the shadcn/Tailwind-compatible name.
  2026-05-17 compact paragraph text result: `text_compact_paragraph(...)` now owns dense wrapping
  body copy for editor/IMUI panels. IMUI `bullet_text(...)` labels and `text_wrapped(...)` route
  through that shared role instead of carrying local fill-width/min-width-zero `TextProps` policy.
  2026-05-18 tooltip body text result: IMUI `tooltip_text(...)` default body copy now routes
  through a private `tooltip_body_text(...)` seam backed by `text_compact_paragraph(...)`, so
  convenience tooltips wrap as dense body/help text instead of inheriting single-line chrome text
  from `ui.text(...)`.
  2026-05-18 collection proof text-role result: `imui_editor_proof_demo` collection fixed chrome
  and readouts now use proof-local helpers over shared section-chrome/control-readout roles. Inline
  rename explanatory copy is the explicit wrapping path, while collection state, asset metadata,
  context-menu selection, and drop-status text stay single-line and shrinkable under resize.
  2026-05-16 trigger label reuse result: IMUI tab triggers and menubar triggers now reuse
  `text_button_label(...)`; selectable/menu item row labels stayed out of that role because they
  are command/list rows, not button labels.
  2026-05-16 list row text role result: `text_list_row_label(...)` is now the shared dense row
  label role for menu items, selectables, and tree/disclosure rows. It preserves regular `text-sm`
  styling with fill-width, min-width-zero, single-line ellipsis behavior, so row labels do not wrap
  or grow row height under resize.
  2026-05-20 menu item row-anchor result: `menu_item_element_with_pressable_hook_inner(...)`
  now makes the root pressable own the visible row children instead of using an absolute overlay
  sibling. The menu item's `test_id`, focus, click, and keyboard behavior stay on the same row box
  while the visible row layout remains a single pressable tree.
  2026-05-16 menu shortcut readout reuse result: IMUI menu shortcut labels now reuse
  `text_control_readout(...)` as muted compact auxiliary readouts, keeping shortcut text inside the
  stable control-readout role instead of adding another menu-specific text policy.
  2026-05-17 section chrome label text result: `text_section_chrome_label(...)` now owns compact
  separator/section chrome labels in `fret-ui-kit::declarative::text`. IMUI `separator_text`
  labels use that shared role, so section chrome stays single-line, shrinkable, and ellipsis-based
  under resize instead of inheriting default word wrapping.
  2026-05-17 chrome title text result: `text_chrome_title(...)` now owns medium, fill-width
  floating window title-bar text. Resizable floating titles keep fill/grow/min-width-zero behavior,
  while non-resizable titles reuse the section/chrome label role instead of local `TextProps`.
  2026-05-17 chrome glyph text result: `text_chrome_glyph(...)` now owns compact fixed-slot
  chrome glyph text in `fret-ui-kit::declarative::text`. Disclosure/tree indicators use that
  shared role, so glyph-only chrome stays single-line and clipped without local `TextProps`.
  2026-05-18 text role source-gate hardening result: `tools/gate_imui_workstream_source.py` now
  freezes direct `TextProps` construction under `fret-ui-kit::imui` behind an explicit allowlist,
  including both `TextProps::new(...)` and `TextProps { ... }` struct literals. Future compact
  IMUI text policy additions must go through the shared role vocabulary or an intentional gate
  update with evidence.
  2026-05-16 IMUI text item resize result: `UiWriterImUiFacadeExt::text(...)` now mirrors Dear
  ImGui's default `Text()` semantics by staying single-line and shrinkable with ellipsis under
  resize. `text_wrapped(...)` is the explicit wrapping escape hatch, and the editor/workspace proof
  prose that should wrap now opts into it.
  2026-05-16 control chrome fill-text result: shared checkbox/radio/switch labels, combo preview
  text, and slider captions now use single-line shrink/ellipsis layout through
  `control_chrome::fill_text(...)`, removing another fixed-height control wrapping path.
  2026-05-17 control label text result: `text_control_label(...)` now owns the compact control
  label role in `fret-ui-kit::declarative::text`, and `control_chrome::fill_text(...)` delegates
  to that shared role instead of carrying local `TextProps` policy.
  2026-05-17 editor input value text result: `editor_input_value_text(...)` now owns the shared
  numeric scrub-readout text role for drag-value and axis-drag-value controls. The role stays in
  `fret-ui-editor` because it depends on editor density/chrome policy, but it now has the same
  resize-safe fill, `min-width: 0`, shrink, single-line, and ellipsis behavior expected by IMUI
  editor panels.
  2026-05-19 editor input-group text ownership result: `input_group.rs` no longer carries local
  direct `TextProps` policy for joined-control text segments, numeric value text, or axis markers.
  Those fixed-line editor text roles now live in `readout.rs`, and the source gate rejects direct
  `TextProps` construction from the input-group composition layer.
  2026-05-17 editor status badge text result: `FieldStatusBadge` no longer hand-rolls badge label
  `TextProps`; it uses `editor_status_badge_text_props(...)` from the editor readout primitive
  layer, preserving compact centered ellipsis text while keeping role policy reusable.
  2026-05-17 editor inline error text result: `ColorEdit` root errors and popup numeric errors now
  share `editor_inline_error_text_props(...)`, so compact destructive readouts are single-line,
  shrinkable, and owned by the editor readout primitive layer instead of duplicated per surface.
  2026-05-17 editor validation message text result: `NumericInput` inline validation messages now
  use `editor_validation_message_text_props(...)`, keeping the explicit wrapping validation role in
  the editor primitive layer. `tools/gate_imui_workstream_source.py` also freezes direct
  `TextProps` construction under `fret-ui-editor/src` to the named primitive owners, so new editor
  controls cannot bypass the text-role vocabulary by adding local text literals.
  2026-05-17 transform label text result: `TransformEdit` section badges, section headings, and
  inline link/uniform labels now reuse editor readout text helpers. The control no longer carries
  local compact-label `TextProps`, so future resize fixes happen in the shared editor text role
  layer instead of per-transform branches.
  2026-05-17 popup list text result: enum-select and text-assist popup/list empty labels now reuse
  shared popup-list text helpers. This removes the `TextProps::new(...)` default word-wrap path
  from those editor assist surfaces while keeping the behavior in `fret-ui-editor`, not
  `fret-imui`. Color-edit copy menu rows and popup option captions now use the same popup-list text
  family through explicit aligned variants, while preview/tooltip text stays separate.
  2026-05-17 color preview/tooltip text result: color side-preview captions and tooltip numeric
  lines now use dedicated editor readout helpers. This keeps readout/caption semantics out of
  popup-list rows while eliminating the remaining local `TextProps` in those color surfaces.
  2026-05-17 property chrome text result: property-group header labels, property-row reset glyphs,
  and inspector panel titles now reuse editor readout helpers. Fixed inspector chrome text no
  longer hand-rolls local/default `TextProps`, so header, title, and reset glyph behavior remains
  single-line and line-box constrained under resize. A narrow-header `InspectorPanel` layout gate
  proves long titles stay one measured line beside toolbar siblings.
  2026-05-17 text role matrix result: `P3_TEXT_ROLE_MATRIX_2026-05-17.md` now records the stable
  base role vocabulary for future resize triage: control readout, button label, paragraph, code
  text, and table cell text. The matrix treats wrapping paragraph/validation copy as an explicit
  multi-line layout contract, keeps fixed chrome/control text single-line by default, and avoids a
  public `TextRole` enum until a data-driven role value has at least two consumers.
  2026-05-17 property-row value overflow result: `PropertyRow` now keeps its value slot overflow
  visible so wrapping validation/prose children can contribute and paint multi-line height under
  resize. The fixed label/reset/action chrome slots remain clipped. This closes the first concrete
  layout-container fix from the text-role matrix without moving policy into `fret-imui`.
  2026-05-17 property-row wrapping layout result: the overflow fix now has a layout-level gate.
  A narrow `PropertyRow` with `editor_validation_message_text_props(...)` runs through
  `UiTree::layout_all(...)`, then public element-bounds queries assert that the wrapped validation
  text is contained by the value slot and row bounds instead of painting past the row bottom.
  2026-05-17 property-grid wrapping layout result: the container fix is now covered at the
  inspector composition level too. A narrow `PropertyGrid` with single-line rows before/after a
  wrapping validation row proves that the wrapping row grows and pushes following rows down instead
  of preserving a fixed-row-height assumption.
  2026-05-17 generic list text result: the compatibility `list_from_strings(...)` helper now uses
  the shared text-role vocabulary for fixed virtual-list rows. Leading status glyphs use
  `text_chrome_glyph(...)`, labels use `text_list_row_label(...)`, and trailing shortcut text uses
  `text_control_readout(...)`, with a focused structure test and source-gate marker preventing the
  old bare `cx.text(...)` row path from returning.
  2026-05-17 generic tree text result: the default retained tree row renderer now routes row labels
  through `text_list_row_label(...)` and toggle glyphs through `text_chrome_glyph(...)`, while
  leaving custom tree row renderers free to own their own content policy.
  2026-05-17 file tree text result: `file_tree_view_retained_v0(...)` now routes fixed row icons
  through `text_chrome_glyph(...)` and file labels through `text_list_row_label(...)` instead of
  hand-rolling `ui::text(...).truncate()` inside the fixed-height row.
  2026-05-17 retained table text result: `table_virtualized_retained_v0(...)` and grouped table
  row text now route retained header labels, grouped row labels, and aggregation values through
  `text_table_cell(...)` instead of bare `cx.text(...)` inside fixed table cells.
  2026-05-17 examples table proof text result: `table_demo` and `table_stress_demo` now teach the
  same shared text-role vocabulary as the retained/data-grid proof surfaces. Their long status
  headers use `text_control_readout(...)`, while table headers and body cells use
  `text_table_cell(...)`, with source tests preventing the old bare `cx.text(header/label/text)`
  paths from returning.
  2026-05-17 datatable proof text result: `datatable_demo` now teaches the same contract at the
  shadcn data-table layer. The compact selected/sort status readout uses
  `text_control_readout(...)`, and body cells use `text_table_cell(...)`; source gates reject the
  old bare `cx.text(...)` cell/readout paths.
  2026-05-17 virtual-list stress proof text result: `virtual_list_stress_demo` now teaches the
  same fixed-row text contract for virtualized lists. The scroll/state header uses
  `text_control_readout(...)`, visible row labels use `text_list_row_label(...)`, and source gates
  reject the old bare `cx.text(header)` / row-label text path.
  2026-05-17 canvas datagrid stress proof text result: `canvas_datagrid_stress_demo` now routes its
  compact retained-canvas grid stats header through `text_control_readout(...)`, and source gates
  reject the old bare `cx.text(header)` path above the fixed grid slot.
  2026-05-17 date picker proof text result: `date_picker_demo` now demonstrates the role split in
  one surface: status readouts use `text_control_readout(...)`, switch captions use
  `text_control_label(...)`, and keyboard instructions use `text_paragraph(...)`; source gates
  reject the old bare status/label/instruction `cx.text(...)` paths.
  2026-05-17 form proof text result: `form_demo` now routes its header submit/valid/dirty/status
  readout through `text_control_readout(...)`, and source gates reject the old bare fixed-header
  `cx.text(Arc::from(format!(...)))` path.
  2026-05-17 sonner proof text result: `sonner_demo` now routes its fixed demo title through
  `text_section_chrome_label(...)` and promise/last-action status through
  `text_control_readout(...)`; source gates reject the old bare title/status `cx.text(...)` paths.
  2026-05-17 echarts proof text result: `echarts_demo` chart titles now route through
  `text_section_chrome_label(...)`; source gates reject the old bare chart-title `cx.text(...)`
  path.
  2026-05-17 components gallery table proof text result: the runnable `components_gallery`
  retained table torture path now routes fixed cell renderers through `text_table_cell(...)` and
  the table explanation through `text_paragraph(...)`. The same proof now routes top chrome,
  tree-status, theme-control, color-swatch, and control-state text through shared chrome, label,
  and readout roles. Overlay body copy now routes through paragraph text and overlay last-action
  status routes through `text_control_readout(...)`; source gates reject the old bare table-cell,
  header, fixed control, and overlay proof `cx.text(...)` paths.
  2026-05-17 markdown proof chrome text result: `markdown_demo` now keeps fixed title, preview
  description, and toolbar state text on shared section-chrome, paragraph, and control-readout
  roles. 2026-05-19 markdown image placeholder result: image placeholder copy now uses the shared
  paragraph-break-words role with app-owned muted foreground, removing the last `markdown_demo`
  direct `TextProps` residual while keeping Markdown body rendering surface-owned.
  2026-05-17 residual bare text capability result: a focused `fret-examples` source test now keeps
  remaining bare `cx.text(...)` / `TextProps::new(...)` paths limited to explicit text/IME
  capability proofs. This prevents both accidental fixed-chrome regressions and mechanical
  migration of surfaces that intentionally test text/input rendering.
  2026-05-17 gallery retained-table torture text result: the UI Gallery retained-table torture page
  now uses `text_table_cell(...)` for fixed table cells and `control_readout_text(...)` for table
  state readouts, so the visible retained-table stress surface no longer teaches bare fixed-cell
  text under resize.
  2026-05-17 gallery data-table torture text result: the UI Gallery DataTable torture page now
  routes fixed cells through `text_table_cell(...)` in both retained and non-retained render paths,
  and table sorting/filter/pinning status lines through `control_readout_text(...)`.
  2026-05-18 data-table snippet table-cell text result: the copyable DataTable snippets now route
  fixed status/name/email/CPU/memory/fallback cells through a directory-local helper backed by
  `text_table_cell(...)`. The amount columns intentionally keep their existing tabular numeric text
  styling until numeric table-cell semantics are split as a separate role.
  2026-05-19 table snippet table-cell text result: ordinary copyable Table snippets now route
  fixed body/footer/action cells through directory-local helpers backed by shared table-cell roles.
  `text_table_cell_emphasis(...)` preserves medium first-column emphasis without reintroducing
  app-local `ui::text(...).font_weight(...)`; the later children-API follow-up below closes the
  remaining custom header/caption text exception.
  2026-05-19 table children custom text result: the explicit children-API table snippet now routes
  rich header child text through `table_cell_text(...)` and caption copy through
  `text_paragraph(...)`, keeping slotted table examples on shared roles without changing table
  recipe internals.
  2026-05-19 checkbox table-cell text result: the checkbox table snippet keeps the action-first
  select-all model/action surface while moving member/role fixed cells to a local helper backed by
  `text_table_cell(...)`, closing another fixed-row bare text escape without changing checkbox
  recipe behavior.
  2026-05-19 typography table-cell text result: Typography table samples in standalone, demo, and
  RTL snippets now share a local helper backed by `text_table_cell(...)`, preserving typography
  prose/rich-link behavior while keeping table rows on fixed-cell text semantics.
  2026-05-18 AI AudioPlayer state-marker result: the copyable AudioPlayer local/remote snippets
  now use zero-size `SpacerProps` children under generic semantics for state-only diagnostics
  markers, instead of mounting empty `Text` nodes for non-visible test anchors.
  2026-05-18 AI visible text-role result: the Message and Terminal copyable snippets now use shared
  text roles for fixed demo titles, explanatory prose, and compact action status instead of visible
  bare `cx.text(...)`; the Terminal empty-output marker also moved to a non-text spacer anchor.
  2026-05-19 AI Terminal title text-role result: the `fret-ui-ai` `TerminalTitle` default label now
  uses `text_chrome_title(...)`, giving the real component the same fill-width, `min-width: 0`,
  grow/shrink, single-line ellipsis contract as other chrome titles.
  2026-05-19 AI EnvironmentVariables title text-role result: `text_chrome_title(...)` now also owns
  medium chrome-title weight, and the `fret-ui-ai` `EnvironmentVariablesTitle` default/text paths
  route through that shared role instead of local raw-text title policy. Custom title children stay
  on the component-owned inherited title refinement because the upstream surface is children-first.
  2026-05-19 AI EnvironmentVariables code-label result: environment variable names and
  non-selectable values now reuse `text_code_label(...)` for fixed identifier slots, while revealed
  values stay on `SelectableTextProps` for the explicit selection capability surface. Empty
  custom-child/diagnostics markers no longer use empty `Text` nodes.
  2026-05-19 AI PackageInfo code/paragraph result: shared text roles now cover the PackageInfo
  defaults without local `TextProps`. Package names and target versions use
  `text_code_label_emphasis(...)`; current versions, dependency names, and dependency versions use
  `text_code_label(...)`; the Dependencies heading uses `text_section_chrome_label(...)`; and
  descriptions use `text_compact_paragraph_inherited(...)` so component-owned description tokens
  still win while the leaf owns the shared wrapping/fill-width resize contract.
  2026-05-19 AI Agent text-role/accordion-boundary result: shared text roles now cover real Agent
  chrome/content defaults without local `TextProps`. Agent header names use chrome-title text,
  section labels use section-chrome text, instructions use compact paragraph text, and tool trigger
  descriptions use list-row text. The shadcn Accordion trigger defaulting path now preserves
  caller-supplied text role style/wrap/overflow metadata instead of forcing every text child back
  to wrapping trigger text.
  2026-05-18 AI visible text-role result 2: Artifact, CodeBlock, and Sandbox snippets now reuse the
  same shared text roles for visible fixed chrome/prose, and CodeBlock's active-language marker no
  longer mounts an invisible empty `Text` element.
  2026-05-18 AI Queue text-role result: the Queue copyable snippet now uses section-chrome text for
  the fixed demo title, paragraph text for explanatory copy, and a generic zero-size spacer marker
  for action-revision diagnostics instead of bare or invisible `cx.text(...)`.
  2026-05-18 AI Checkpoint text-role result: the Checkpoint copyable snippet now routes
  conversation/prose text through paragraph roles, restore status through control-readout text,
  checkpoint trigger text through button-label text, and custom checkpoint icon symbols through
  chrome-glyph text.
  2026-05-18 AI simple chrome text-role result: Agent, CodeBlock usage, Environment Variables, and
  OpenIn snippets now route fixed demo titles through section-chrome text and explanatory body copy
  through paragraph text instead of default bare `cx.text(...)`.
  2026-05-18 AI selector/branch marker result: MessageBranch, MicSelector, and ModelSelector
  snippets now use generic zero-size spacer markers for state-only diagnostics anchors instead of
  empty `Text`, and their demo titles/body copy route through shared section-chrome/paragraph roles.
  2026-05-18 AI prompt/plan/commit-large text-role result: CommitLarge now keeps its opened-file
  diagnostics anchor out of text layout semantics with a generic zero-size spacer marker, and
  CommitLarge, Plan, PromptInputActionMenu, and PromptInputTooltip route their outer fixed
  title/body text through shared section-chrome/paragraph roles.
  2026-05-18 AI large/status text-role result: StackTraceLarge, TestResultsLarge, Tool, and
  Suggestions now keep fixed outer chrome/prose on shared roles. StackTraceLarge/TestResultsLarge
  diagnostics anchors and the Tool/Suggestions test markers now use generic zero-size spacers
  instead of empty `Text`; Tool's fixed state-section labels use section-chrome text.
  2026-05-18 AI queue-prompt/transcription text-role result: QueuePromptInput now keeps its
  sent-count diagnostics anchor out of text layout semantics, routes the custom Search button child
  through `text_button_label(...)`, and keeps fixed outer title/body copy on shared roles.
  Transcription now uses generic zero-size spacer markers for time/active diagnostics anchors while
  routing fixed title/body copy through shared section-chrome/paragraph roles.
  2026-05-18 AI WebPreview text-role result: WebPreview state diagnostics now use generic
  zero-size spacer markers instead of empty `Text`, navigation glyphs use `text_chrome_glyph(...)`,
  and composable child fixed body/footer copy uses shared section-chrome/paragraph roles.
  2026-05-18 AI Chat text-role result: Chat's prompt-nonempty diagnostics marker now uses a
  generic zero-size spacer, empty marker fallbacks use spacers instead of empty `Text`, fixed header
  instructions use paragraph roles, and exported markdown length uses control-readout text. Chat
  message body rendering stays app/content-owned for a separate semantics pass.
  2026-05-18 AI PromptInput provider/docs text-role result: PromptInputProvider now keeps
  sent-count diagnostics out of text layout semantics, routes the custom external-add label through
  `text_button_label(...)`, and keeps fixed outer title/body copy on shared roles. PromptInput docs
  now routes the custom Search label through button-label text and fixed outer title/body copy
  through section-chrome/paragraph roles.
  2026-05-19 AI PromptInput cursor custom-text result: Cursor-style PromptInput custom command
  rows, file/path labels, rules popover copy, tabs footer readout, and trigger counts now use
  shared list-row/code/readout/button text roles instead of local `ui::text(...)` styling.
  2026-05-18 AI chrome/readout text-role result: Reasoning, StackTrace, and VoiceSelector now route
  fixed outer title/body copy through shared section-chrome/paragraph roles; StackTrace and
  VoiceSelector compact status/diagnostics readouts use `text_control_readout(...)` instead of
  default wrapping text.
  2026-05-18 AI Confirmation content text-role result: Confirmation request/body snippets now route
  prose through `text_paragraph(...)`, inline/code payloads through `text_code_wrap(...)`,
  approval/rejection result text through `text_control_readout(...)`, and the demo's fixed outer
  title/body copy through shared section-chrome/paragraph roles.
  2026-05-18 AI Task content text-role result: Task item labels now route through
  `text_list_row_label(...)`, attached file names through `text_code_wrap(...)`, and the demo's
  fixed outer title/body copy through shared section-chrome/paragraph roles.
  2026-05-18 AI Conversation instrumentation text-role result: Conversation export/message-count
  diagnostics now use generic semantics instead of text semantics, and the custom scroll-bottom
  label uses the shared `text_button_label(...)` role.
  2026-05-18 AI usage snippet text-role result: Attachments usage explanatory copy now uses the
  shared paragraph role, and StackTrace usage fixed title/body copy uses shared
  section-chrome/paragraph roles.
  2026-05-18 AI Message usage text-role result: Message usage user text now uses paragraph text,
  the last-action marker uses control-readout text, and fixed outer title/body copy uses
  section-chrome/paragraph roles.
  2026-05-18 AI Canvas world spike text-role result: the canvas spike visible chrome, node copy,
  and debug/status readouts now use shared section-chrome, paragraph, and control-readout roles
  instead of bare `cx.text(...)`.
  2026-05-18 AI Image demo text-role result: fixed explanatory copy now uses shared paragraph text,
  and the image-ready/loading readouts use shared control-readout text instead of bare
  `cx.text(...)`.
  2026-05-18 AI PromptInput referenced sources text-role result: fixed referenced-sources
  title/body copy now uses shared section-chrome and paragraph roles instead of bare
  `cx.text(...)`.
  2026-05-19 AI Attachments inline hover-card text-role result: hover-card attachment labels now
  use shared list-row text and media-type values use shared control-readout text instead of
  default `ui::text(...)`, keeping inline attachment overlay details single-line/shrinkable under
  resize without changing `fret-ui-ai` attachment behavior.
  2026-05-18 AI Artifact code display status-marker result: the docs action status now preserves
  its diagnostic label on a generic zero-size semantics marker instead of an invisible bare text
  element.
  2026-05-18 AI ChainOfThought composable text-role result: composed header, step-label, and
  description child text now uses shared section-chrome and paragraph roles instead of bare
  `cx.text(...)`.
  2026-05-18 AI TestResults composable text-role result: custom summary/progress/status/name and
  duration child text now uses shared readout/list-row roles instead of bare `cx.text(...)`.
  2026-05-18 AI Workflow snippet text-role result: workflow fixed chrome, panel copy,
  node-content sample copy, footer labels, and click readouts now use shared text roles instead of
  bare `cx.text(...)`.
  2026-05-18 AI Suggestions/reasoning/transcript text-role result: suggestions custom children,
  reasoning hook status, transcript torture header copy, and chat exported-status marker now use
  shared text roles or generic marker semantics instead of bare/default text surfaces.
  2026-05-19 AI Shimmer demo chrome text result: Shimmer demo labels and inline non-shimmer text
  now use shared readout/section roles instead of local `ui::text(...)` styling, while
  `Shimmer::new(...)` remains the explicit animated text capability surface.
  2026-05-18 AI custom-children text-role result: environment variables, package info, inline
  citations, persona, and sources custom-child snippets now use shared roles for app-owned visible
  text; `text_code_label(...)` covers fixed-height package/env/dependency identifiers.
  2026-05-19 AI PlanContent text-role result: Plan's inner section headings, paragraph body,
  bullet rows, and custom Build button child now use shared section-chrome, paragraph, list-row,
  and button-label roles instead of local `ui::text(...)` styling.
  2026-05-17 gallery data-grid text result: the UI Gallery DataGrid preview now routes virtualized
  grid cells through `text_table_cell(...)` and the selected-row status line through
  `control_readout_text(...)`.
  2026-05-17 gallery data paragraph text result: DataGrid/DataTable/Tree Torture explanatory
  header copy now routes through `paragraph_text(...)`, backed by shared `text_paragraph(...)`,
  instead of default `cx.text(...)`.
  2026-05-17 gallery inspector torture text result: the UI Gallery Inspector Torture virtual rows
  now route fixed property labels through `text_list_row_label(...)` and fixed values through
  `control_readout_text(...)`.
  2026-05-17 gallery virtual-list torture text result: the UI Gallery virtual-list harness now
  routes fixed custom row labels through `text_list_row_label(...)`, detail/editing readouts
  through `control_readout_text(...)`, and the UI Kit list torture custom row renderer through the
  shared list-row label role.
  2026-05-17 gallery harness header text result: retained-table, hit-test, UI Kit list,
  virtual-list, and view-cache harness headers now route explanatory copy through
  `paragraph_text(...)` and mode/status lines through `control_readout_text(...)` instead of bare
  `cx.text(...)`.
  2026-05-17 gallery view-cache list text result: the UI Gallery View Cache torture page now routes
  cached inner virtual-list row labels through `text_list_row_label(...)`.
  2026-05-17 gallery view-cache control-label result: fixed switch labels now route through
  `control_label_text(...)` instead of bare `cx.text(...)`.
  2026-05-17 gallery view-cache popover body result: the cached Popover body copy now routes
  through `paragraph_text(...)` instead of bare `cx.text(...)`.
  2026-05-17 gallery tree torture status text result: the UI Gallery Tree Torture dynamic target
  status now routes through `control_readout_text(...)` instead of local text-sm/muted styling.
  2026-05-17 gallery overlay status text result: overlay and menu last-action/status flags now
  route through `control_readout_text(...)` instead of bare `cx.text(...)`.
  2026-05-17 gallery overlay scroll-row text result: dialog/sheet/portal scroll filler rows now
  route through `text_list_row_label(...)` instead of bare `cx.text(...)`.
  2026-05-17 gallery overlay body prose result: HoverCard and Popover body copy now route
  through `paragraph_text(...)` instead of bare `cx.text(...)`.
  2026-05-17 gallery chrome torture control-label result: fixed text-input/textarea labels now
  route through `control_label_text(...)`, backed by shared `text_control_label(...)`, instead of
  bare `cx.text(...)`.
  2026-05-17 virtual row fallback result: tree and file-tree virtualizer out-of-range fallback
  paths now return spacer placeholders instead of empty text nodes, so fixed-row helpers no longer
  create meaningless `Text` elements for missing rows.
  2026-05-17 gallery disabled toaster placeholder result: the UI Gallery disabled toaster driver
  path now returns a spacer placeholder instead of an empty text node, keeping app-shell placeholder
  plumbing outside text layout semantics.
  2026-05-17 gallery app-sidebar collapsed placeholder result: the copyable app-sidebar snippet now
  uses a spacer placeholder for collapsed project groups instead of `cx.text("")`.
  2026-05-17 fret-ui-ai empty placeholder result: AI element hidden/missing-content fallbacks now
  share a crate-local spacer helper instead of returning empty text nodes.
  2026-05-17 gallery status-bar readout result: UI Gallery status-bar metric, inspector, and
  last-action text now use `driver::text_roles::chrome_readout_text(...)` backed by
  `text_control_readout(...)`, so fixed status chrome no longer teaches bare wrapping text under
  resize.
  2026-05-17 gallery driver chrome text result: UI Gallery driver chrome now owns a tiny
  `text_roles` module over the shared kit text roles. Disabled pane placeholders route through
  control readouts, and settings sheet section labels route through section chrome labels instead
  of bare/default text.
  2026-05-17 gallery driver chrome label result: the nav title now routes through section-chrome
  text, and settings-sheet switch captions route through control-label text instead of local
  `TextProps` policy.
  2026-05-17 gallery minimal-root text result: the UI Gallery `BISECT_MINIMAL_ROOT` placeholder now
  uses the driver chrome-readout text role instead of bare/default text, keeping even diagnostic
  resize-bisect roots on the shared single-line text posture.
  2026-05-17 gallery debug-HUD text result: fixed-size debug HUD lines now use the shared driver
  chrome-readout role instead of local word-wrapping `TextProps`, keeping long diagnostic metrics
  single-line/truncated inside HUD chrome.
  2026-05-17 gallery shell content/nav text result: page header title/source and sidebar group
  headings now use shared chrome/readout roles instead of hand-rolled `TextProps` in the gallery
  app shell.
  2026-05-18 gallery sidebar snippet chrome text result: copyable Sidebar examples now keep
  body/fallback prose on shared paragraph roles and status/debug lines on shared control-readout
  text, so sidebar docs no longer teach bare wrapping `cx.text(...)` in fixed example chrome.
  2026-05-18 gallery command snippet chrome text result: copyable Command examples now keep
  last-action/count/active-value status on shared control-readout roles, short subsection headings
  on section-chrome text, and desktop-only/prose copy on paragraph text. The retained
  active-descendant snippet stays outside this migration because it intentionally exercises the
  command text-input capability surface.
  2026-05-18 gallery Accordion trigger text result: copyable Accordion examples now keep trigger
  labels on the shared button-label role, so these button-like rows truncate instead of teaching
  default wrapping text under resize. The slice stays in UI Gallery snippets/tests/gates and does
  not change shadcn component internals.
  2026-05-18 gallery ToggleGroup item text result: copyable ToggleGroup examples now keep ordinary
  item captions on the shared button-label role, covering text-only, icon+text, RTL, spacing,
  full-width, and flex-1 item snippets. The custom weight-card snippet keeps its local visual sample
  typography outside this default-role migration.
  2026-05-18 gallery Toggle item text result: copyable Toggle examples now keep text-only and
  icon+text captions on the shared button-label role, and the label-association pressed-state line
  on the shared control-readout role.
  2026-05-18 gallery Button children text result: the custom command-menu child label in the
  copyable Button children snippet now uses the shared button-label role instead of bare
  `cx.text(...)`.
  2026-05-18 gallery Tabs custom text result: icon+label custom tab triggers now use the shared
  button-label role, and usage-panel prose uses paragraph text; built-in tab label recipe paths
  remain recipe-owned.
  2026-05-18 gallery Collapsible text result: trigger labels, controlled-state readout, panel
  prose, repository identifiers, and file-tree row labels now use shared text roles instead of bare
  `cx.text(...)` / `ui::text(...)` / raw typography.
  2026-05-18 gallery AlertDialog custom text result: rich-content body copy now uses paragraph
  text, rich-content action child labels use button-label text, and small/RTL custom title/body
  children use section-chrome/paragraph roles. The rich attributed-title path stays as intentional
  text capability evidence.
  2026-05-18 gallery HoverCard text result: app-owned HoverCard title/date/body/positioning copy
  now uses shared section-chrome, paragraph, control-readout, and button-label roles instead of
  raw/default text builders.
  2026-05-18 gallery Popover align text result: the align preview body labels now use paragraph
  text instead of bare `cx.text(...)`.
  2026-05-18 gallery Tooltip keyboard shortcut text result: the custom shortcut tooltip label now
  uses control-readout text instead of bare `cx.text(...)`.
  2026-05-19 gallery Kbd custom-copy text result: Kbd demo/RTL separator glyphs now use shared
  chrome-glyph text, while group/tooltip helper copy uses shared control-readout text. The keycap
  text policy stays inside `fret-ui-shadcn::Kbd`, so this remains a caller-composition cleanup
  rather than a shadcn recipe rewrite.
  2026-05-19 gallery Separator menu text result: the menu snippet's section helper now uses
  shared section-chrome and control-readout roles instead of local `ui::text(...)` line-box/color
  policy, keeping separator copy resize-safe while leaving the Separator primitive leaf-shaped.
  2026-05-19 gallery Item slotted text result: Item dropdown trigger copy, download header copy,
  and issue number side columns now use shared button-label, section-chrome, and control-readout
  roles. Recipe-owned `ItemTitle` / `ItemDescription` rendering stays inside `fret-ui-shadcn`.
  2026-05-19 gallery Spinner amount readout result: Spinner item amount/status values in LTR and
  RTL snippets now use shared control-readout text, keeping fixed item rows from teaching local
  `ui::text(...).text_sm()` / `cx.text(...)` value builders.
  2026-05-19 gallery AvatarStack direction label result: Shadcn Extras avatar-stack direction
  labels now use shared section-chrome text instead of local `ui::text(...).font_medium()`, keeping
  the fixed demo chrome on resize-safe roles while leaving raw extras recipes unchanged.
  2026-05-19 gallery Kanban card title result: Shadcn Extras Kanban app-owned card titles now use
  shared button-label text instead of local `ui::text(item.name).font_medium().truncate()` policy,
  while raw Kanban drag/drop and column recipes remain unchanged.
  2026-05-19 Shadcn Extras AnnouncementTitle result: the announcement title copy is intentionally
  not rewritten at the gallery call site. `AnnouncementTitle` remains a composable children-first
  raw extras component, while `fret-ui-shadcn` now applies the upstream-style `truncate` contract
  at the recipe owner: shrink/min-width-zero title container, clipped overflow, inherited medium
  text-sm title style, and single-line ellipsis for nested text children.
  2026-05-19 gallery Dialog scroll-row text result: scrollable-content and sticky-footer dialog
  filler rows now use shared list-row label text instead of `ui::raw_text(format!(...))`, keeping
  the scroll proof rows on a fixed-row text role without changing shadcn dialog recipes.
  2026-05-19 gallery Drawer scroll/side text result: drawer scroll filler rows now use shared
  list-row label text, side body examples use paragraph text, and the scroll helper was renamed
  from `paragraph_block` to `scroll_rows` to match the fixed-row role.
  2026-05-19 gallery Drawer goal/diagnostics text result: demo/RTL goal numbers and unit labels
  now use shared control-readout text, nested drawer guidance uses paragraph text, and outside-press
  probe copy/status no longer emits bare `ui::text`.
  2026-05-19 gallery ScrollArea visible text result: demo/RTL fixed scroll rows now use shared
  list-row text, headings use section-chrome text, horizontal/nested captions use control-readout
  text, and usage/compact prose uses paragraph text instead of local text styling.
  2026-05-19 gallery ContextMenu trigger text result: dashed context-region trigger copy across the
  ContextMenu snippets now uses shared control-readout text, removing the duplicated local
  muted/text-sm policy while preserving pointer-aware wording and trigger geometry.
  2026-05-19 gallery Pagination text result: copyable Pagination page labels now use shared
  button-label text via a context-bound helper, RTL page labels share that path, extras explanatory
  copy uses paragraph text, and `fret-ui-shadcn` Previous/Next visible labels use the same shared
  button-label role instead of bare `cx.text(...)`.
  2026-05-19 gallery Carousel status/readout text result: API/events/autoplay diagnostic status
  lines now use shared control-readout text instead of local word-wrapping `TextProps` blocks,
  while centered placement stays in snippet layout (`h_flex + justify_center`) instead of becoming
  a new text role.
  2026-05-19 gallery NavigationMenu link-label text result: custom icon/text NavigationMenu link
  labels now use the shared button-label role in docs, demo, and RTL snippets. The line-clamped
  card title/body text remains intentionally out of scope for a separate derived-role decision.
  2026-05-19 compact paragraph line-clamp result: `text_compact_paragraph_line_clamp(...)` adds a
  shared dense paragraph-family clamp contract, and ordinary NavigationMenu list-item
  titles/descriptions now use shared button-label plus clamped paragraph roles instead of local
  line-clamp `TextProps`.
  2026-05-17 gallery editor preview text result: code-editor, Markdown, and Web IME preview
  headers now use paragraph text for prose, control readout text for fixed status/debug values,
  and button label text for custom pointer-region actions. The slice keeps editor-proof resize
  text semantics in gallery/doc-layout helpers and the shared kit role vocabulary, not in
  `fret-imui`.
  2026-05-18 code-view editor preview prose result: the UI Gallery code-view torture header now
  uses `doc_layout::paragraph_text(...)` for explanatory copy instead of bare `cx.text(...)`,
  keeping scrollable code/text preview prose on the same paragraph role as the other editor
  preview headers.
  2026-05-18 text editor/conformance header prose result: the UI Gallery text
  editor/conformance headers now use paragraph text for explanatory copy and a control-readout role
  for the BiDi sample-list heading. Explicit text capability probes remain on direct
  `TextProps`, `SelectableTextProps`, and canvas text paths.
  2026-05-17 code-editor IME gate button-label result: the MVP IME gate action labels now use
  `doc_layout::button_label_text(...)`, and focused source/test guards prevent those custom
  pointer-region buttons from drifting back to bare `cx.text(...)` under fixed row chrome.
  2026-05-17 docking arbitration text-role result: `docking_arbitration_demo` now uses a local
  paragraph helper for Popover body copy and a local readout helper backed by
  `text_control_readout(...)` for state/debug status lines, keeping the docking proof on shared
  text roles without moving policy into `fret-imui`.
  2026-05-17 docking/container-query panel text result: `docking_demo` and
  `container_queries_docking_demo` now use local helpers over shared list-row, control-readout, and
  button-label text roles for fixed panel text. This closes the simple docking demo resize escape
  hatch while leaving docking topology/policy ownership unchanged.
  2026-05-19 imui node-graph compatibility title result: `imui_node_graph_demo` keeps the
  retained-bridge node-graph proof explicitly compatibility-only, but its fixed title is now a
  section-chrome text role through `compat_section_text(...)`. The slice removes local
  `fret_ui_kit::ui::text(...).font_semibold()` title styling without adding node-graph policy to
  `fret-imui`.
  2026-05-19 embedded viewport chrome text result: `embedded_viewport_demo` now keeps viewport size
  ToggleGroup labels on shared button-label text and status lines on shared control-readout text.
  This removes local fixed-chrome `ui::text(...).text_sm()` policy from the embedded viewport proof
  without changing the embedded RenderTarget/input-forwarding interop path.
  2026-05-19 window hit-test probe text result: `window_hit_test_probe_demo` keeps its multi-window
  hit-test repro logic unchanged while moving fixed header text to section chrome, logical window
  identifiers to code-label text, and status to control-readout text. This removes the old local
  `ui::text(...).text_sm()` policy from another resize-sensitive probe surface.
  2026-05-19 launcher utility window text result: `launcher_utility_window_demo` keeps its
  frameless utility-window proof behavior unchanged while moving the drag-region title to
  section-chrome text, the effective style diagnostic to code-label text, status to
  control-readout text, and the resize handle arrow to chrome-glyph text. This removes local
  fixed-window chrome/readout/glyph text styling without adding new `fret-imui` API.
  2026-05-19 launcher utility window materials text result:
  `launcher_utility_window_materials_demo` keeps its material request/diagnostics proof behavior
  unchanged while moving the fixed title to section-chrome text, the effective material/style
  diagnostic to code-label text, and status to control-readout text. This completes the paired
  utility-window chrome text cleanup without moving window-material policy into `fret-imui`.
  2026-05-19 API workbench lite text result: `api_workbench_lite_demo` keeps its request,
  mutation, query, and persisted-history proof behavior unchanged while moving app/sidebar chrome,
  paragraph copy, base-URL identifiers, and history status states onto shared text roles. The slice
  also removes the now-redundant `shell_frame` theme snapshot parameter, because text color policy
  is no longer owned locally by that proof surface.
  2026-05-19 hello counter text result: `hello_counter_demo` keeps its action/state proof behavior
  unchanged while moving the status line to control-readout text and the step help copy to paragraph
  text. The large numeric counter display remains an explicit visual display value until a
  dedicated large-readout role exists.
  2026-05-19 simple todo text result: `simple_todo_demo` keeps its typed action/list proof behavior
  unchanged while moving app-owned visible text to control-readout, compact paragraph, and list-row
  roles. The done/active row foreground stays app state policy, but row layout no longer relies on
  local `ui::text(...)` truncation.
  2026-05-19 todo demo text result: `todo_demo` keeps its responsive/stateful todo proof behavior
  unchanged while moving app-owned visible text to title, readout, compact paragraph, button-label,
  and list-row roles. Completed row strikethrough now uses an attributed list-row label helper, so
  the row decoration no longer requires local `ui::rich_text(...)` layout policy.
  2026-05-19 async playground text result: `async_playground_demo` keeps its async query/cache proof
  behavior unchanged while moving app-owned visible text to chrome-title, section-chrome, list-row,
  control-readout, code-label, and compact-paragraph roles. The query helper call chain no longer
  carries `ThemeSnapshot` just to color fixed readouts, so resize behavior is role-owned rather
  than locally styled.
  2026-05-19 GenUI demo text result: `genui_demo` keeps its catalog/editor/runtime validation
  behavior unchanged while moving tool text to code-block, control-readout, and compact-paragraph
  roles. JSON/spec/schema/prompt panes now use code text, fixed toolbar/issue/status values use
  readout text, stream help uses paragraph text, and the empty text spacer is gone.
  2026-05-19 extras marquee perf text result: `extras_marquee_perf_demo` keeps its marquee
  animation/perf probe unchanged while moving the fixed title to the section-chrome text role.
  2026-05-19 residual bare text gate tightening result: `text_role_residual_surface` now counts
  `ui::text(...)` and `ui::rich_text(...)` residuals too, so ordinary proof apps cannot bypass the
  text-role contract by using the builder-style text facade. The remaining builder-style residuals
  are explicit capability/display payloads: the large numeric counter display and the GPUI/Fret
  hello-world comparison title.
  2026-05-19 query detail text result: `query_demo` and `query_async_tokio_demo` no longer use
  `ui::raw_text(...)` for query detail rows. Status/error/timing/retry diagnostics now use shared
  control-readout text, fetched data uses code-label text, and error foreground remains app-owned
  state styling. `imui_editor_proof_demo` also removed its old direct editor-style readout
  `TextProps` construction in favor of the shared control-readout role.
  2026-05-19 custom effect overlay text result: `custom_effect_v1_demo` and
  `custom_effect_v2_demo` keep their explicit custom-effect/runtime ownership, but their fixed
  overlay pill labels now use shared section-chrome text with inherited white foreground instead
  of local `TextProps`.
  2026-05-19 custom effect web overlay text result: `custom_effect_v2_web_demo` keeps its WebGPU
  effect ownership unchanged while moving the unsupported-state readout, badge label, and keyboard
  hint to shared text roles. The absolute keyboard hint now positions a container around
  control-readout text instead of constructing local `TextProps`.
  2026-05-19 custom effect web template text result: `custom_effect_v2_identity_web_demo`,
  `custom_effect_v2_lut_web_demo`, and `custom_effect_v2_glass_chrome_web_demo` keep their WebGPU
  template behavior unchanged while moving fixed overlay/control text out of local `TextProps`.
  Starter/LUT badges use section-chrome text, hints/status use readout text, and glass/chrome
  slider names/values use control-label/readout roles.
  2026-05-19 effect reference chrome text result: `custom_effect_v3_demo`,
  `postprocess_theme_demo`, and `liquid_glass_demo` keep their renderer/effect proof behavior
  unchanged while moving fixed overlay/header/card titles out of local `TextProps`. The remaining
  effect chrome uses shared section-chrome/control-readout roles with app-owned foreground and
  container geometry.
  2026-05-19 shadcn Table role-preservation result: `TableCell` and `TableHead` now preserve
  caller-supplied shared text roles instead of rewriting their leaf typography or overflow. Bare
  text children still receive table defaults, so recipe ergonomics remain intact while role-owned
  resize semantics survive nested shadcn composition.
  2026-05-19 shadcn DataTable role-preservation result: the virtualized DataTable body-cell
  default text-style wrapper now skips shared text-role scopes. This keeps data-table ergonomics
  for bare cell renderers while preserving role-owned typography and ellipsis for callers that
  already supply `text_table_cell(...)`.
  2026-05-19 shadcn NavigationMenuLink role-preservation result: custom link content now keeps
  shared button-label text roles intact. Link foreground is stamped as inherited foreground, while
  link typography remains a bare-text fallback instead of a recursive leaf override.
  2026-05-19 shadcn ItemTitle role-preservation result: ItemTitle keeps its strong title-slot
  fallback for bare/rich text, while shared title-role children keep their role-owned style and
  ellipsis contract under item composition.
  2026-05-19 shadcn CardTitle role-preservation result: CardTitle keeps the shadcn title fallback
  for bare/rich card-title children, while shared title/chrome roles remain protected role scopes
  and keep their single-line ellipsis contract under card composition.
  2026-05-20 shadcn CardDescription children role-preservation result: the composable
  CardDescription children lane now has a focused gate proving shared description/body roles keep
  their role-owned wrap/overflow and inherited metadata under card composition.
  2026-05-20 shadcn Sheet/Popover description children role-preservation result: SheetDescription
  and PopoverDescription now have composable children lanes, with focused gates proving shared
  paragraph/body roles keep role-owned wrap/layout and inherited metadata under overlay description
  composition.
  2026-05-20 shadcn existing description children role-preservation result: AlertDescription,
  DialogDescription, AlertDialogDescription, and ItemDescription now have focused gates proving
  their existing children lanes preserve shared paragraph/body roles under description composition.
  2026-05-19 shadcn AlertTitle role-preservation result: AlertTitle keeps the shadcn title fallback
  for bare/rich alert-title children, while shared title/chrome roles remain protected role scopes
  and keep their single-line ellipsis contract under alert composition.
  2026-05-19 shadcn AlertDialogTitle role-preservation result: AlertDialogTitle keeps the shadcn
  dialog-title fallback for bare/rich title children, while shared title/chrome roles remain
  protected role scopes under alert-dialog composition.
  2026-05-19 shadcn DialogTitle children-role result: DialogTitle now has a composable
  `new_children(...)` path. Bare/rich title children still receive dialog-title defaults, while
  shared title/chrome roles keep their own style, foreground, wrap, and overflow contracts.
  2026-05-19 shadcn SheetTitle children-role result: SheetTitle now has a composable
  `new_children(...)` path. Bare/rich title children still receive sheet-title defaults, while
  shared title/chrome roles keep their own style, foreground, wrap, and overflow contracts.
  2026-05-19 shadcn PopoverTitle children-role result: PopoverTitle now has a composable
  `new_children(...)` path. Bare/rich title children still receive popover-title defaults, while
  shared title/chrome roles keep their own style, foreground, wrap, and overflow contracts.
  2026-05-19 shadcn FieldTitle children-role result: FieldTitle now has a composable
  `new_children(...)` path. Bare/rich title children still receive field-title defaults and w-fit
  layout behavior, while shared title/chrome roles keep their own layout and ellipsis contracts.
  2026-05-19 shadcn EmptyTitle children-role result: EmptyTitle now has a composable
  `new_children(...)` path. Bare/rich empty-state title children still receive empty-title
  defaults, while shared title/chrome roles keep their own ellipsis contracts.
  2026-05-20 shadcn SelectLabel menu-group text result: `text_menu_group_label(...)` now owns the
  muted fixed-row menu/select group-label role, and SelectLabel consumes it instead of local
  overlay text sizing/nowrap policy.
  2026-05-20 shadcn menu-family group-label text result: DropdownMenu, ContextMenu, and Menubar
  label rows now route their non-interactive group heading text through
  `text_menu_group_label(...)`. The helper reuse keeps fixed menu rows on one shared resize
  contract while preserving menu-owned item label rendering and icon foreground policy.
  2026-05-20 shadcn CommandGroup heading text result: Command/Listbox group headings now route
  through `text_menu_group_label(...)` via a command-local helper. Combobox, native select, and
  data-table recipes benefit through `CommandGroup::heading(...)`, while command row
  label/highlight rendering stays command-owned.
  2026-05-20 shared status-message text result: `text_status_message(...)` now covers muted
  `text-sm` non-interactive empty/loading/status messages, and shadcn `CommandEmpty` /
  `CommandLoading` use it for their fixed command-surface status text.
  2026-05-20 shadcn DataTable toolbar text result: DataTable toolbar faceted trigger labels,
  faceted option labels, option counts, clear/reset action labels, and selected-count readouts now
  consume shared button-label, list-row-label, and control-readout text roles. Pagination footer
  page/selected summaries now also consume tabular control-readout variants after
  `TextStyleRefinement` gained inherited OpenType feature support, closing the local
  `ui::text(...).tabular_nums()` footer escape without adding a sixth stable text role.
  2026-05-20 tabular readout resize-gate result: those tabular control-readout variants are now
  part of the shared narrow-layout single-line role gate and the text-role matrix derived-role
  catalog, so footer/page readouts remain explicitly protected against resize wrapping.
  2026-05-20 shadcn ButtonGroupText children role-preservation result: the existing composable
  ButtonGroupText children lane now has a focused gate proving caller-supplied button-label roles
  keep their single-line shrink/ellipsis contract under button-group chrome composition.
  2026-05-20 shadcn TabsTrigger role-preservation result: TabsTrigger now preserves
  caller-supplied button-label roles in trigger children by treating inherited text styles as
  protected role scopes, while bare trigger labels still receive the shadcn tabs fallback.
  2026-05-20 shadcn Toggle/ToggleGroup role-preservation result: Toggle and ToggleGroupItem now
  preserve caller-supplied button-label roles in explicit children while retaining the foreground
  fallback for bare custom text children.
  2026-05-20 shadcn Badge role-preservation result: Badge now preserves caller-supplied
  button-label roles in leading/trailing children while retaining foreground fallback for bare
  child text.
  2026-05-20 shadcn Button role-preservation result: Button now has focused gates proving
  caller-supplied button-label roles survive both full custom content and inline leading/trailing
  slot composition.
  2026-05-20 shadcn TooltipContent role-preservation result: TooltipContent now preserves
  caller-supplied control-readout roles in rich tooltip content while retaining tooltip-owned
  typography/foreground fallback for bare text. Tooltip foreground now flows as inherited
  foreground from the content root instead of being forced into shared role text leaves.
  2026-05-20 shadcn BreadcrumbList role-preservation result: BreadcrumbList now preserves
  caller-supplied button-label roles in primitive list children while retaining breadcrumb
  typography fallback for bare loose text. List-level muted foreground now flows as inherited
  foreground instead of being forced into text leaves.
  2026-05-20 shadcn AnnouncementTitle role-preservation result: raw extras AnnouncementTitle now
  preserves caller-supplied button-label roles while retaining the recipe-owned clipped title
  container and bare-text single-line ellipsis fallback. Title typography is applied to bare text
  leaves instead of the root, avoiding inherited-style merging into shared roles.
  2026-05-20 shadcn SidebarGroupLabel resize result: fixed-height sidebar group labels now consume
  the shared menu-group text role instead of local wrapping text builders. The role carries
  `text-xs font-medium`, fill/shrink/min-width-0, no-wrap, and ellipsis semantics, while sidebar
  still owns its muted foreground. Narrow sidebars therefore truncate the label instead of letting
  wrapped text exceed the 32px chrome row.
  2026-05-20 shadcn SidebarMenuBadge resize result: fixed sidebar menu badges now consume a
  compact tabular emphasis readout role (`text-xs font-medium tabular-nums`) instead of local
  sidebar-only text sizing. The badge keeps its `h-5 min-w-5` chrome and sidebar foreground, while
  the text role owns no-wrap, shrink, min-width-0, and ellipsis behavior under resize.
  2026-05-20 shadcn SidebarMenuButton/SubButton label result: default sidebar menu and sub-menu
  labels now consume fill-width button-label role variants instead of sidebar-local text builders.
  The main default/lg labels use `text_button_label_fill(...)`, small main/sub labels use
  `text_button_label_compact_fill(...)`, and collapsed tooltip labels use `text_button_label(...)`.
  This keeps upstream truncate behavior in the shared button-label family while leaving sidebar
  chrome, foreground, collapse motion, RTL ordering, and tooltip placement recipe-owned.
  2026-05-20 inherited-axis + shadcn Button default-label result: `TextStyleRefinement` now carries
  variable font axes as subtree defaults alongside OpenType features, with merge/refine,
  measurement, cache-fingerprint, and typography bridge gates. shadcn `Button` default labels now
  consume the shared button-label role instead of local `ui::text(...).fixed_line_box_px(...)`
  builders, while preserving label font, feature, axis, weight, foreground, and `test_id` suffix
  behavior through inherited text/foreground metadata.
  2026-05-20 shadcn CalendarDayButton text-role result: Calendar day numbers and optional
  supporting text now consume shared button-label/readout role helpers instead of local
  `ui::label(...).line_height_px(...).nowrap()` builders. Calendar still owns fixed day-cell
  chrome, center alignment, range/selected/today foreground, and disabled opacity, while both
  single and range calendar day cells share the same single-line shrink/ellipsis text contract.
  2026-05-20 shadcn CalendarMultiple text-role result: multiple-selection calendar day numbers now
  consume the same `calendar_day_button_children(...)` helper as single/range day cells instead of
  carrying a local `ui::label(day_text).text_size_px(...).line_height_px(...).font_medium()`
  builder. Multiple selection still owns selection updates and cell chrome, while shared text roles
  own no-wrap, shrink, min-width-zero, ellipsis, and inherited typography/foreground.
  2026-05-20 shadcn CalendarHijri text-role result: Hijri day numbers now consume the same shared
  day-cell helper instead of direct `TextProps::new(day_text)` fixed-line/clipped text. Hijri keeps
  RTL order, Persian digits, Gregorian-date test ids, and selection chrome; shared roles own the
  fixed-cell text resize contract.
  2026-05-20 shadcn Kbd/ShortcutHint keycap text-role result: fixed keycap/hint chrome now consumes
  `text_keycap_label(...)` instead of local `ui::label(...).fixed_line_box_px(...)` builders. Kbd
  and ShortcutHint still own shadcn `component.kbd.*` typography refinements, foreground, tooltip
  slot colors, icon children, and row layout; the shared role owns no-wrap, shrink,
  min-width-zero, and ellipsis.
  2026-05-20 shadcn menu item label text-role result: shared `text_list_row_label(...)` now truly
  owns fill/grow/basis-zero row-label layout, and DropdownMenu, ContextMenu, and Menubar overlay
  item labels consume a shared shadcn menu item label helper instead of local
  `ui::text(...).text_size_px(...).nowrap()` builders. Menu recipes still own row chrome,
  destructive/disabled/focused foregrounds, shortcut/trailing slots, and icon currentColor
  inheritance.
  2026-05-20 shadcn NativeSelect text-role result: NativeSelect selected/placeholder trigger text
  now consumes `text_control_label(...)`, and option rows consume `text_list_row_label(...)`,
  with shadcn NativeSelect/Command typography and state foreground layered through inherited
  metadata. NativeSelect keeps trigger chrome, popover/listbox behavior, check icons, placeholder
  state, and RTL ordering recipe-owned; the shared roles own the resize-critical single-line,
  fill/grow/shrink, min-width-zero, and ellipsis contracts.
  2026-05-20 shadcn Combobox text-role result: default Combobox selected/placeholder trigger text
  now consumes `text_control_label(...)`, and non-search option rows consume
  `text_list_row_label(...)`, with Combobox/Command typography and state foreground layered
  through inherited metadata. Combobox keeps trigger chrome, inline addons, clear/chevron buttons,
  popover/drawer policy, search-enabled CommandPalette behavior, custom item content, and RTL
  ordering recipe-owned; the shared roles own the default-label single-line, fill/grow/shrink,
  min-width-zero, and ellipsis contracts.
  2026-05-21 shadcn ComboboxChips text-role result: empty-trigger placeholder text now consumes
  `text_control_label(...)`, while selected chip pill labels consume `text_chip_label(...)`. The new
  shared chip role owns compact medium no-wrap/min-width-zero/ellipsis without fill/grow, so chip
  chrome can shrink safely without behaving like a row/control label. ComboboxChips keeps
  trigger/chip chrome, remove actions, popover/search policy, selected-value mapping, wrapping chip
  layout, and RTL ordering recipe-owned.
  2026-05-21 shadcn Badge default-label result: Badge default labels now consume
  `text_chip_label(...)` instead of local fixed-line `ui::text(...)` builders. Badge layers font,
  feature, weight, foreground/currentColor, and link-hover underline behavior through inherited
  metadata, while the shared chip role owns no-wrap, min-width-zero, shrink, ellipsis, and
  non-growing inline-badge text layout.
  2026-05-18 IMUI virtual-list fixed-row clip result: fixed/known-height `fret-ui-kit::imui`
  virtual-list rows now mount as fixed-height `Overflow::Clip` row containers, while measured rows
  stay auto-height/visible so runtime measurement still works.
  2026-05-18 retained tree fixed-row clip result: retained tree rows now align their pressable row
  owner with the virtualizer contract: fixed/known rows clip at the configured row height, and
  measured rows keep visible overflow for variable-height content.
  2026-05-18 retained file-tree fixed-row clip result: `file_tree_view_retained_v0(...)` now clips
  at the retained pressable row owner instead of relying only on inner row content containers.
  2026-05-18 retained table fixed-row clip result: retained/eager table body rows now share the
  same owner-side contract through `table_body_row_layout(...)`; fixed rows clip to row height and
  measured rows keep measurement-friendly overflow.
  Current collection-helper audit result: keep collection behavior app-owned until a second IMUI
  proof repeats the same request/box-select/selection-repair shape. `fret-node` remains domain
  evidence, not an API-freezing proof surface.
  2026-05-14 multi-select storage result: `ImUiMultiSelectState` is still the shared policy-layer
  storage helper, but callers now use `new`/`single` plus read-only selection and anchor accessors
  instead of constructing or clearing public fields directly.
  2026-05-14 ordered-selection result: visible-order selection repair is now a
  `fret-ui-kit::imui` storage operation instead of duplicated proof-app logic.
  2026-05-14 request-vocabulary audit result: keep request/IO multi-select API candidate-only until
  another first-party proof repeats the same selection request shape.
  2026-05-14 state-catalog gate result: `ImUiMultiSelectState` is now guarded by the reusable
  opaque-struct check, not only by narrow string markers.
  Current execution-priority review result: treat the P3 catalog notes as readiness maps, not an
  implementation queue. Product/golden workflow coherence, runner/backend multi-window hand-feel,
  and diagnostics/DevTools discoverability remain higher-value Dear ImGui-grade closure work than
  blind widget/API mirroring.
  2026-05-16 first-open gate wording result: diagnostics/DevTools entrypoint discovery now has an
  explicit cold-start gate and an explicit `--reuse-built` drift check. This keeps DevTools
  discoverability source-backed without treating Rust build latency as a GUI/MCP product failure.
  2026-05-21 demo/metrics/debug discovery result: `fretboard-dev list tool-apps` and its JSON form
  now expose the `demo-metrics-debug` first-open route with grouped demo, metrics, and debug
  commands, including trace drill-down. This moves the Dear ImGui-style Demo/Metrics/Debug
  entrypoint into the shared CLI/JSON discovery surface instead of leaving it discoverable only
  inside the DevTools GUI guide.
  Current performance-alignment review result: `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
  belongs in the active gap lane's evidence set. Keep runtime smoothness work in
  `diag-perf-attribution-v1`, `ui-perf-zed-smoothness-v1`, and the product-chain docking perf gate;
  do not use Dear ImGui/egui performance pressure as a reason to widen `fret-imui` or start a
  broad widget/API backlog.
  2026-05-23 performance refresh result: `editor-canvas-paint-replay-slice-v1` is now closed after
  the r59 Windows RTX4090 target-machine validation, attribution validation, artifact verification,
  and closeout. The retained owner is `canvas-paint-replay`, checked-in baselines stay unchanged,
  and the lesson remains an editor-paint/perf-lane owner split rather than an IMUI runtime/API
  widening.

## M5 - Fearless Refactor Execution

Exit criteria:

- The first internal owner split lands without changing the public IMUI surface.
- Floating-window behavior stays covered by focused smoke tests and source gates.
- The next cleanup slice is chosen from fresh evidence, not stale parity notes.

2026-05-24 title-bar owner split result: the floating-window title-row / close-button composition
now lives in `ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs`, leaving
`floating_window_on_area.rs` as the shell that frames the chrome, content, and resize stack.
The public IMUI surface stayed stable, and the existing floating smoke + source gates were reused
to validate the split.

2026-05-24 content/blocker owner split result: the floating-window content scroll/focus wrapper
now lives in `ecosystem/fret-ui-kit/src/imui/floating_window_content.rs`, and the input-blocking
overlay moved to `ecosystem/fret-ui-kit/src/imui/floating_window_blocker.rs`. The main
`floating_window_on_area.rs` shell now just wires title, content, blocker, and resize stack
together, with the public IMUI surface still unchanged.

2026-05-24 resize-stack owner split result: `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs`
now owns the body/blocker/resize-handle stack assembly through an internal
`resize_stack_element(...)` helper. `floating_window_on_area.rs` now only passes the clipped body,
blocker, resize flags, activation policy, and handle test ids into the resize owner.

2026-05-24 resize-state owner split result: the resize clamp/snap/update logic now lives in
`prepare_resize_state(...)` inside `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs`.
`floating_window_on_area.rs` no longer owns the `FloatWindowState` clamp/snap loop or the resize
handle test-id tuple assembly.

2026-05-24 resize-snapshot owner split result: the resize owner now also owns active resize handle
discovery through `current_resize_snapshot(...)`. `floating_window_on_area.rs` no longer enumerates
`FloatWindowResizeHandle` values, reads drag runtime snapshots directly, or derives the chrome
`resizing` signal from tuple-shaped resize state.

2026-05-24 shell owner split result: the remaining floating-window frame/container composition now
lives in `ecosystem/fret-ui-kit/src/imui/floating_window_shell.rs`. `floating_window_on_area.rs`
no longer owns the title-bar container, clipped body, blocker, or resize stack assembly.

2026-05-24 resize-handle layout helper result: the repeated cursor/inset/size mapping now lives in
`resize_handle_layout(...)` inside `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs`.
`resize_handle_element(...)` now just consumes that helper and keeps the pointer-region wiring.

2026-05-24 resize-drag application helper result: the handle-driven size/position mutation now
lives in `apply_resize_drag(...)` inside `ecosystem/fret-ui-kit/src/imui/floating_window_resize.rs`.
`prepare_resize_state(...)` now keeps the snapshot/collapse/snap orchestration only.

2026-05-24 shell props helper result: `window_frame_props(...)`, `shell_column_props(...)`,
`title_bar_container_props(...)`, and `clipped_body_props(...)` now own the frame/container property
construction inside `ecosystem/fret-ui-kit/src/imui/floating_window_shell.rs`. The public shell
helper now only composes the prepared title row, content, blocker, and resize stack.

2026-05-24 title-bar props helper result:
`ecosystem/fret-ui-kit/src/imui/floating_window_title_bar_props.rs` now owns title-row layout,
drag-surface layout, and close-button accessibility/size props. `floating_window_title_bar.rs`
keeps keyboard/click behavior orchestration plus the close-glyph text-role helper.

2026-05-27 title-bar behavior owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_title_bar/behavior.rs` now owns double-click
collapse event recording, title-bar Escape close key behavior, close-button activation wiring, and
model update/notify calls. `floating_window_title_bar.rs` now keeps row composition, title text-role
selection, close-button prop selection, and close-glyph text construction.

2026-05-24 content props helper result: `ecosystem/fret-ui-kit/src/imui/floating_window_content_props.rs`
now owns content surface layout, scroll layout, and container props. `floating_window_content.rs`
keeps the pointer/focus orchestration and consumes the prepared content owner outputs.

2026-05-27 content behavior owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_content/behavior.rs` now owns content-surface
pointer-region wrapping, focusable key stub installation, background-click focus requests,
activate-on-click event recording, and float-layer bring-to-front delegation.
`floating_window_content.rs` now keeps content scroll/container composition and IMUI child mounting.

2026-05-25 table render/body/header owner split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` now owns table assembly, test-id suffixing,
palette resolution, and shared cell helpers. `table_controls/body.rs` owns prepared cells, pinned row
grouping, horizontal center-scroll wrapping, and cell wrapping. `table_controls/header.rs` plus
`header/{trigger,resize}.rs` own sortable/plain header behavior and resize interaction. The root
`table_controls.rs` keeps authoring collection and row/cell facade wiring only. The public IMUI
table API stayed stable.

2026-05-26 table render helper owner split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/cell.rs` now owns shared table cell layout, padding,
empty-cell, and cell-child packing helpers. `table_controls/palette.rs` owns theme-to-table-palette
resolution, and `table_controls/test_ids.rs` owns column test-id suffixing. `render.rs` keeps table
assembly, hidden-column handling, header/body response collection, and root table wrapping only.
The public IMUI table API stayed stable.

2026-05-25 plot adapter proof result:
`ecosystem/fret-plot/src/imui.rs` now provides opt-in `UiWriter` helpers that delegate to the
existing declarative plot panels. `fret-plot` default features remain empty, `fret-imui` and
`fret-ui-kit::imui` do not depend on `fret-plot`, and the retained plot bridge stays deleted.

2026-05-25 ListBox container proof result:
`ecosystem/fret-ui-kit/src/imui/list_box_controls.rs` now provides a Dear ImGui `BeginListBox`-style
semantic scroll host. `ListBoxOptions` stays container-scoped, covering layout, scroll,
diagnostics ids, label, and multiselectable semantics only. Selection rows remain ordinary
`selectable_with_options(...)` children, and the container does not own active-descendant,
filtering, command, or collection policy.

2026-05-25 facade basic-items owner split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/basic_items.rs` now owns the private
`UiWriterImUiFacadeExt` default bodies for basic text, wrapped text, bullet text, plain separators,
and separator text. `facade_writer.rs` remains the public trait hub and forwards those methods to
the owner module without changing public names or behavior.

2026-05-26 facade image-items owner split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/image_items.rs` now owns the private
`UiWriterImUiFacadeExt` default bodies for image item/button forwarding, including the
`ImageItemVariant::Button` normalization used by `image_button_with_options`. The interactive image
item policy stays in `image_item_controls.rs`; the public facade method names and signatures stay
unchanged.

2026-05-27 image-item visual owner split result:
`ecosystem/fret-ui-kit/src/imui/image_item_controls/visual.rs` now owns image item chrome
selection, image props, size sanitization, opacity normalization, and UV validation.
2026-05-27 follow-up: `image_item_controls.rs` now keeps a11y props, size props, key activation
policy for plain images, chrome mounting, and image visual assembly, while
`image_item_controls/behavior.rs` owns pressable behavior, context-menu key handling, activation
lifecycle marking, pointer-click reporting, and `ResponseExt` population.

2026-05-27 image-item behavior owner split result:
`ecosystem/fret-ui-kit/src/imui/image_item_controls/behavior.rs` now owns pressable behavior
installation, keyboard-activation lifecycle marking, context-menu key handling, transient clicked
reads, and `ResponseExt` population. `image_item_controls.rs` keeps a11y props, size props, key
activation policy for plain images, chrome mounting, and image visual assembly.

2026-05-26 facade command-presentation owner split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` now owns the button command
presentation/default-enabled forwarding path, and
`ecosystem/fret-ui-kit/src/imui/facade_writer/menu_items.rs` owns the menu command
presentation/default-enabled/default-shortcut forwarding path. The root `UiWriterImUiFacadeExt`
trait in `facade_writer.rs` still exposes the same public method roster and now only forwards these
command helpers to the owner modules. `facade_writer.rs` dropped from 984 to 971 lines after this
slice.

2026-05-26 worktree convergence decision:
`WORKTREE_CONVERGENCE_PLAN_2026-05-26.md` records the integration strategy for the dirty `main` and
`imui-imgui-editor-grade-refactor` worktrees. `main` remains the final integration base because it
already contains the six committed shadcn/parity foundation commits. IMUI content is resolved by
topic: keep identical plot/table slices, prefer the IMUI worktree's more complete facade owner split,
layout sugar, canonical workbench, Demo/Metrics/Debug, and style/theme picker work, and leave the
`main`-only `facade_writer/image_items.rs` slice for a separate evidenced follow-up unless completed
before checkpointing. The image-items slice was completed before the `main` checkpoint.
