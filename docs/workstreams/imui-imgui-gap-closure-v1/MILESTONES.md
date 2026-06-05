# ImUi Dear ImGui Gap Closure v1 - Milestones

Status: Active
Last updated: 2026-06-05

2026-06-05 Fret-ImUi popup hover-flags proof owner-split result:
`ecosystem/fret-imui/src/tests/popup_hover/hover_flags.rs` now keeps only child-owner routing for
the popup hover-flags proof surface. `hover_flags/disabled_scope.rs` owns disabled underlay
blocking and AllowWhenDisabled hover proof, `tooltip_delay.rs` owns tooltip stationary/delay hover
proof, `popup_blocking.rs` owns AllowWhenBlockedByPopup underlay hit-test proof,
`active_item.rs` owns AllowWhenBlockedByActiveItem proof while another item is active, and
`shared_delay.rs` owns shared versus local hover-delay timer proof. No `fret-ui-kit::imui` hover
runtime implementation, `fret-imui` public facade, or hover flag API changed.

2026-06-05 Fret-ImUi popup item-keyboard proof owner-split result:
`ecosystem/fret-imui/src/tests/popup_hover/item_keyboard.rs` now keeps only child-owner routing for
the popup item keyboard proof surface. `item_keyboard/keyboard_open.rs` owns context-menu keyboard
open, first-item focus, and Escape focus-restore proof; `arrow_nav.rs` owns popup item ArrowUp /
ArrowDown focus navigation proof; `shortcuts.rs` owns focused-popup-item shortcut scoping plus
`shortcut_repeat` opt-in proof; and `checkbox_semantics.rs` owns menu-item checkbox checked-state
semantics proof. No `fret-ui-kit::imui` popup/menu runtime implementation, `fret-imui` public
facade, or popup/menu option API changed.

2026-06-05 Fret-ImUi floating movement/z-order proof owner-split result:
`ecosystem/fret-imui/src/tests/floating/movement_z_order.rs` now keeps only child-owner routing for
the floating movement, window-control, and z-order proof surface. `movement_z_order/movement.rs`
owns title-bar and floating-area drag movement proof, `window_controls.rs` owns window response
position/size reporting plus close-button and Escape close proof, and `z_order.rs` owns floating
area and floating layer bring-to-front hit-test order proof. No `fret-ui-kit::imui` floating
runtime implementation, `fret-imui` public facade, or floating option API changed.

2026-06-05 Fret-ImUi control-geometry proof owner-split result:
`ecosystem/fret-imui/src/tests/composition/control_geometry.rs` now keeps only shared geometry
helpers and child-owner routing for the control-geometry proof surface. `control_geometry/variants.rs`
owns small/arrow/invisible-button and radio mount/bounds proof, `base_controls.rs` owns base
control hover/focus/pressed/value/selected bounds stability proof, `menu_tabs.rs` owns menu and tab
trigger hover/focus/press/open/selection bounds stability proof, and `disabled.rs` owns
enabled-to-disabled bounds stability proof across text, button, selection, menu, submenu, and tab
controls. No `fret-ui-kit::imui` control runtime implementation, `fret-imui` public facade, or
control option API changed.

2026-06-05 Fret-ImUi combo-model proof owner-split result:
`ecosystem/fret-imui/src/tests/models_combo/combo_model.rs` now keeps only child-owner routing for
the combo-model proof surface. `combo_model/selection.rs` owns changed-on-pick, selected model
projection, lifecycle edit, and deactivated-after-edit proof, `popup.rs` owns popup Escape
close/focus restore and popup scope/test-id override proof, and `shortcuts.rs` owns focused-trigger
shortcut scoping and `shortcut_repeat` opt-in proof. No `fret-ui-kit::imui` combo-model runtime
implementation, `fret-imui` public facade, or combo-model option API changed.

2026-06-05 Fret-ImUi combo-direct proof owner-split result:
`ecosystem/fret-imui/src/tests/models_combo/combo_direct.rs` now keeps only child-owner routing for
the direct combo proof surface. `combo_direct/lifecycle.rs` owns popup Escape close/focus restore
and open-session edge reporting proof, `shortcuts.rs` owns focused-trigger shortcut scoping and
`shortcut_repeat` opt-in proof, and `selection.rs` owns selectable-row commit, selected
preview/model projection, and close-after-pick proof. No `fret-ui-kit::imui` combo runtime
implementation, `fret-imui` public facade, or combo option API changed.

2026-06-05 Fret-ImUi menu-activation proof owner-split result:
`ecosystem/fret-imui/src/tests/interaction_menu_tabs/menu_activation.rs` now keeps only child-owner
routing for the menu-activation proof surface. `menu_activation/command_activation.rs` owns command
item activation and close-after-command proof, `shortcuts.rs` owns focused-trigger shortcut
scoping, shortcut-open focus restore, and `shortcut_repeat` opt-in proof, and
`keyboard_navigation.rs` owns ArrowDown open/focus plus horizontal ArrowLeft / ArrowRight
top-level switching. No `fret-ui-kit::imui` menu runtime implementation, `fret-imui` public facade,
or menu option API changed.

2026-06-05 Fret-ImUi submenu-hover proof owner-split result:
`ecosystem/fret-imui/src/tests/interaction_menu_tabs/submenu_hover.rs` now keeps only child-owner
routing for the submenu-hover proof surface. `submenu_hover/nested.rs` owns nested submenu
semantics and command activation, `top_level.rs` owns delayed top-level hover switching,
`open_delay.rs` owns submenu pointer-entry open-delay, `sibling_switch.rs` owns sibling submenu
hover switching, and `grace_corridor.rs` owns grace/safe corridor tests plus its local
geometry/timer helpers. No `fret-ui-kit::imui` menu runtime implementation, `fret-imui` public
facade, or menu option API changed.

2026-06-05 Fret-ImUi floating input-mode proof owner-split result:
`ecosystem/fret-imui/src/tests/floating/input_modes.rs` now keeps only child-owner routing for the
floating input-mode proof surface. `input_modes/activation.rs` owns activate-on-click,
focus-on-click, and resize-handle activation tests; `input_modes/no_inputs.rs` owns
`inputs_enabled=false`, no-input focus traversal, no-input underlay hit-testing, and click-through
focus-skip tests; `input_modes/passthrough.rs` owns pointer-pass-through hit-testing plus
nav-highlight/hover-query behavior. No `fret-ui-kit::imui` runtime implementation, `fret-imui`
public facade, or floating option API changed.

2026-06-05 IMUI plot adapter cookbook teaching surface result:
`apps/fret-cookbook/examples/imui_plot_basics.rs` now provides the first-party opt-in plot adapter
teaching path. The lesson hosts `fret_plot::imui::line_plot_panel(...)` under
`fret::imui::imui_raw(...)`, wires caller-owned `PlotState` and `PlotOutput`, stamps stable
`cookbook.imui_plot_basics.*` readouts, and sets an explicit plot canvas size for the cookbook
host. `apps/fret-cookbook/Cargo.toml` owns the `cookbook-imui-plot` feature, and
`apps/fretboard/src/demos.rs` auto-enables it for cookbook launching. The cookbook/docs/source
gates freeze this as an adapter proof without adding any `fret-plot` dependency to `fret-imui` or
`fret-ui-kit::imui`.

2026-06-05 Fret Plot declarative drag-output test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root axes/grid,
primary axis-label smoke tests, and child test-owner routing.
`ecosystem/fret-plot/src/declarative/tests/drag_output.rs` owns right-axis Y-line, X-line,
right-axis point, and right-axis rect drag-output publication regressions, including update/end
phase and mapped coordinate assertions. No plot implementation, public panel prop, optional
`fret-plot/imui` adapter routing, primary axes/grid test, or primary axis-label test changed.

2026-06-05 Fret Plot declarative right-axis paint test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root axes/grid,
primary axis-label, draggable-output regression suites, and child test-owner routing.
`ecosystem/fret-plot/src/declarative/tests/right_axis.rs` owns custom right-axis formatter labels,
right-axis series bounds, and Right2/Right3 series bounds projection regressions. No plot
implementation, public panel prop, optional `fret-plot/imui` adapter routing, primary axes/grid
test, primary axis-label test, or draggable-output test changed.

2026-06-05 Fret Plot declarative view/pan test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root axes/grid,
right-axis paint, draggable-output regression suites, and child test-owner routing.
`ecosystem/fret-plot/src/declarative/tests/view_pan.rs` owns caller-controlled view-bounds
publication, pan gesture mutation, X/Y pan-lock, and both-axis pan no-op regressions. No plot
implementation, public panel prop, optional `fret-plot/imui` adapter routing, axes/grid test,
right-axis paint test, or draggable-output test changed.

2026-06-05 Fret Plot declarative legend test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root axes/grid,
right-axis paint, controlled view/pan, and draggable-output regression suites, plus child
test-owner routing. `ecosystem/fret-plot/src/declarative/tests/legend.rs` owns legend paint,
swatch visibility toggle, label pin/unpin, shift-solo restore, and hover-emphasis regressions. No
plot implementation, public panel prop, optional `fret-plot/imui` adapter routing, right-axis paint
test, pan test, or draggable-output test changed.

2026-06-05 Fret Plot declarative series paint test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root axes/grid,
legend, pan, and draggable-output regression suites, plus child test-owner routing.
`ecosystem/fret-plot/src/declarative/tests/series_paint.rs` owns line, area, stems, histogram,
bars, candlestick, error-bars, shaded, heatmap, and histogram2d paint regressions. No plot
implementation, public panel prop, optional `fret-plot/imui` adapter routing, axes/grid test,
legend test, pan test, or draggable-output test changed.

2026-06-05 Fret Plot declarative query/box selection test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root pan and
draggable-output regression suites, and child test-owner routing.
`ecosystem/fret-plot/src/declarative/tests/query_box_selection.rs` owns query drag state/output,
box-zoom view updates, active/persisted selection rectangles, and query/zoom tooltip regressions.
No plot implementation, public panel prop, optional `fret-plot/imui` adapter routing, pan test, or
draggable-output test changed.

2026-06-05 Fret Plot declarative overlay paint test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root
selection/drag-output regression suites, and child test-owner routing.
`ecosystem/fret-plot/src/declarative/tests/overlays.rs` owns reference-line, draggable-line,
draggable-shape, text, tag, image, and right-axis overlay paint regressions. No plot
implementation, public panel prop, optional `fret-plot/imui` adapter routing, drag-output test, or
query/box selection test changed.

2026-06-05 Fret Plot declarative wheel-zoom test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps shared harness state, root non-wheel
regression suites, and child test-owner routing. `ecosystem/fret-plot/src/declarative/tests/wheel_zoom.rs`
owns controlled view-bounds zoom, Shift/Ctrl axis-only zoom, axis-region zoom routing, X/Y zoom
locks, and both-axis lock no-op tests. No plot implementation, public panel prop, optional
`fret-plot/imui` adapter routing, or runtime behavior changed.

2026-06-05 Fret Plot declarative cursor-readout test owner-split result:
`ecosystem/fret-plot/src/declarative/tests.rs` now keeps the shared `TestHost`, `FakeServices`,
scene helpers, remaining root regression suites, and `mod cursor_readout;` routing.
`ecosystem/fret-plot/src/declarative/tests/cursor_readout.rs` owns cursor output publication,
mouse cursor readout chrome/text, per-series readout rows, right-axis formatter readout, and
linked-cursor precedence tests. No plot implementation, public panel prop, optional `fret-plot/imui`
adapter routing, or runtime behavior changed.

2026-06-05 collection proof browser input-runtime owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection/browser_scope.rs` now keeps child-region
mounting, browser/content/scope test IDs, asset-grid owner mounting, and marquee overlay rendering.
`apps/fret-examples/src/imui_editor_proof_demo/collection/browser_scope/input_runtime.rs` owns
pointer-region props, keyboard handler installation, Primary+Wheel zoom routing, background
context-menu anchor publication, box-select pointer down/move/up/cancel handling, pointer capture,
selection projection, and active-id clearing/update behavior. Child-region IDs, scroll binding,
zoom, background context menu, box-select, keyboard dispatch, asset-grid rendering, and the
app-owned/no-public-helper boundary remain unchanged.

2026-06-05 collection proof selection-command sub-owner result:
`apps/fret-examples/src/imui_editor_proof_demo/collection/selection/commands.rs` is now a light
hub that re-exports `delete.rs` and `duplicate.rs`. `selection/commands/delete.rs` owns
`ProofCollectionDeleteResult`, Delete/Backspace matching, delete state transitions, next-active
refocus, and delete command tests. `selection/commands/duplicate.rs` owns
`ProofCollectionDuplicateResult`, Primary+D matching, copy-suffix generation, duplicate insertion,
visible-order copy reselect, and duplicate command tests. Existing command buttons, keyboard
dispatch, and context-menu call sites still import through `collection::selection`, while the
source/surface gates now freeze delete/refocus and duplicate/copy-suffix as separate workflow
owners.

2026-06-05 collection proof selection-command owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection/selection.rs` now keeps visible-order
projection, selected-asset lookup, active-id fallback, select-all, context-menu selection, and
keyboard navigation, while delegating duplicate/delete command transitions through
`selection/commands.rs`. `selection/commands.rs` owns `ProofCollectionDeleteResult`,
`ProofCollectionDuplicateResult`, Delete/Backspace and Primary+D shortcut matching,
duplicate/delete state transitions, copy-suffix generation, and command-transition unit tests.
Public call sites still import through
`collection::selection`, so command buttons, keyboard dispatch, and context menu routing remain
unchanged while the source/surface gates now freeze duplicate/delete transitions separately from
the selection-navigation owner.

2026-06-05 collection proof browser-scope owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now delegates the browser region
through `render_collection_browser_scope(...)` after computing visible assets, keys, active asset,
rename readiness, and layout readouts. `apps/fret-examples/src/imui_editor_proof_demo/collection/
browser_scope.rs` owns `ProofCollectionBrowserScopeModels`, `ProofCollectionBrowserScopeState`,
child-region options, scroll handle binding, pointer-region construction, keyboard handler
installation, primary-wheel zoom model/scroll updates, background pointer box-select transitions,
background context-menu anchor publication, marquee overlay mounting, and asset-grid owner
mounting. Child-region IDs, scroll binding, pointer semantics, keyboard dispatch, zoom behavior,
box-select projection, and asset-grid tile behavior remain unchanged.

2026-06-05 collection proof asset-grid owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps the browser child region,
pointer/wheel/box-select scope, and marquee overlay mounting, then delegates tile-grid rendering
through `render_collection_asset_grid(...)`. `apps/fret-examples/src/imui_editor_proof_demo/
collection/asset_grid.rs` owns `ProofCollectionAssetGridModels`, `ProofCollectionAssetGridState`,
grid construction, tile selectable/context-menu trigger routing, active focus target capture,
inline-rename field mounting/outcome routing, drag-source/ghost wiring, rendered-item capture, and
tile metadata/path readouts. Grid/tile/inline-rename/ghost labels and test IDs remain unchanged, and
the source/surface gates now freeze the asset-grid workflow separately from the root browser owner.

2026-06-05 collection proof command-buttons owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps readouts and delegates the
explicit Duplicate/Rename/Delete button row through `render_collection_command_buttons(...)`.
`apps/fret-examples/src/imui_editor_proof_demo/collection/command_buttons.rs` owns
`ProofCollectionCommandButtonModels`, `ProofCollectionCommandButtonState`, duplicate/rename/delete
button construction, duplicate/delete state-transition routing, inline-rename startup routing, app
model writeback, and command-status publication. Button labels/test IDs, enabled-state policy,
duplicate/rename/delete behavior, and the app-owned/no-public-helper boundary remain unchanged, and
the source/surface gates now freeze explicit command buttons separately from the root render owner.

2026-06-05 collection proof keyboard handler owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps pointer-region focus/
capture behavior and delegates scope key handling through `install_collection_keyboard_handler(...)`.
`apps/fret-examples/src/imui_editor_proof_demo/collection/keyboard.rs` owns
`ProofCollectionKeyboardHandlerModels`, visible-asset/key projection, IME and active-rename
suppression, Delete/Backspace/F2/Primary+A/Primary+D/Arrow/Home/End routing, selection/rename
state-transition calls, command-status model writes, and `host.notify(...)` dispatch. Shortcut
behavior, model writes, focus handoff, and the app-owned/no-public-helper boundary remain
unchanged, and the source/surface gates now freeze keyboard event dispatch separately from the root
render owner.

2026-06-05 collection proof context-menu owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps tile/background
context-menu trigger handling and delegates the popup workflow through
`render_collection_context_menu(...)`. `apps/fret-examples/src/imui_editor_proof_demo/collection/context_menu.rs`
owns `ProofCollectionContextMenuModels`, popup open-at handling, menu item construction,
selection readout mounting, duplicate/delete state-transition routing, inline-rename startup
routing, dismiss entry wiring, and command-status model writes. Popup anchor handoff, shortcut
labels, duplicate/rename/delete/dismiss behavior, and the app-owned/no-public-helper boundary
remain unchanged, and the source/surface gates now freeze context-menu popup workflow separately
from the root render owner.

2026-06-05 collection proof drag/drop owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps drag source installation,
drag preview ghost/card mounting, drop-target routing, delivered-payload model writes, and visible
drop-status readout. `apps/fret-examples/src/imui_editor_proof_demo/collection/drag_drop.rs` owns
`ProofCollectionDragPayload`, selected-set payload formation, single-asset fallback, preview
title/subtitle projection, drop status projection, and drag/drop unit tests. Selected-set drag,
unselected single-asset fallback, preview/delivered status text, and the app-owned mutation
boundary remain unchanged, and the source/surface gates now freeze drag/drop payload projection
separately from the root render owner.

2026-06-05 collection proof box-select owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps the background pointer
down/move/up/cancel hooks, focus/capture/release behavior, selection model writes, and marquee
overlay mounting. `apps/fret-examples/src/imui_editor_proof_demo/collection/box_select.rs` owns
`ProofCollectionRenderedItem`, `ProofCollectionBoxSelectSession`,
`ProofCollectionBoxSelectState`, hit-test projection, append/replace selection projection, active
marquee rect projection, and box-select unit tests. Background drag thresholding, visible-order
selection, append-vs-replace behavior, and the app-owned/no-public-helper boundary remain
unchanged, and the source/surface gates now freeze box-select state separately from the root render
owner.

2026-06-05 collection proof inline rename owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` now keeps the inline `TextField`
mount inside the active asset tile plus render call sites for explicit, keyboard, and context-menu
rename entry points. `apps/fret-examples/src/imui_editor_proof_demo/collection/rename.rs` owns
`ProofCollectionRenameSession`, `ProofCollectionRenameCommit`, inline focus timer state, begin/
commit helpers, focus sync, focus restore, and rename unit tests. F2, explicit rename, context-menu
rename, label trimming, stable asset ids/order, status text, and focus handoff behavior remain
unchanged, and the source/surface gates now freeze rename workflow state separately from the root
render owner.

2026-06-05 collection proof selection owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection.rs` keeps proof rendering, box-select
pointer hooks, drag/drop preview wiring, and app model writes.
`apps/fret-examples/src/imui_editor_proof_demo/collection/selection.rs` owns
`ProofCollectionKeyboardState`, delete/duplicate result records, visible-order projection,
active-id resolution, select-all, keyboard navigation, context-menu selection, delete refocus, and
duplicate copy-suffix state transitions. `models.rs` and `readouts.rs` import the selection owner
instead of reaching back through the root module, and source/surface gates now treat selection as a
separate demo-local owner without widening any shared IMUI helper.

2026-06-03 IMUI begin-menu trigger-flow owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu.rs` now keeps enabled/state setup, popup
request/body routing, disabled-popup cleanup, and final `DisclosureResponse` shaping.
`menu_family_controls/menu/trigger_flow.rs` owns begin-menu trigger mounting, row-open reads,
active-trigger synchronization, menubar open-menu snapshot/reconciliation, and enabled
click-toggle policy. Begin-menu identity, shortcut activation, menubar hover/sibling switching,
submenu interaction, and public IMUI menu facade behavior remain unchanged, and the source gate
freezes trigger-flow policy separately from the popup/response owner.

2026-06-03 IMUI table header sort-label owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header/labels.rs` now keeps visible-label parsing,
header content-box chrome, header label text-role routing, and narrow re-exports only.
`table_controls/header/labels/sort.rs` owns sortable detection, sort glyph projection,
sort-indicator text construction, and sortable header a11y-label wording. Header row rendering,
sortable/plain wrapper call paths, resize-handle placement, and public table facade behavior remain
unchanged, and the source gate freezes sort policy separately from the label/chrome hub.

2026-06-03 IMUI tooltip panel element owner-split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/panel.rs` now keeps root-name scoping,
trigger-anchor fallback, environment outer-bounds lookup, popper placement, and delegation only.
`tooltip_overlay/panel/element.rs` owns the named tooltip panel element, panel-id model writeback,
popover chrome attachment, rich-content column facade assembly, and tooltip semantics/test-id
decoration. Public tooltip facade behavior, hover/dismissal runtime, request orchestration, and
text-tooltip helpers remain unchanged, and the source gate freezes placement and element assembly
as separate owners.

2026-06-03 canonical workbench persistent action-strip result:
`apps/fret-examples/src/imui_editor_workbench_demo.rs` now owns an
`ImUiEditorWorkbenchView` host instead of mounting `EditorNotesDemoView` directly as the app route.
The host delegates the editor-notes workflow to `self.notes.render(cx)` and keeps a persistent
Demo/Metrics/Debug quick-action strip with stable `imui-editor-workbench.*` test IDs. The strip
surfaces the primary workbench command, supporting proof command, metrics stats command, debug
trace command, and the Wayland real-host handoff command while leaving execution in DevTools and
fretboard. `imui_editor_workbench_golden_path_surface` and the IMUI source gate now freeze the
host route, so the canonical workbench cannot regress back to a bare route alias.

2026-06-03 canonical workbench copy-affordance result:
The persistent quick-action strip is no longer display-only. It now includes stable
`imui-editor-workbench.action.copy-selected-command` and
`imui-editor-workbench.action.copy-command-bundle` controls plus an
`imui-editor-workbench.action-copy-status` readout. The workbench can copy either the currently
selected Demo/Metrics/Debug command or the full command bundle through the existing runtime
`Effect::ClipboardWriteText` boundary while keeping execution with DevTools and fretboard. The
editor workflow remains mounted under `imui-editor-workbench.workflow`.

2026-06-03 Demo/Metrics/Debug first-open shared contract result:
`apps/fret-first-open/src/lib.rs` now owns the static `demo_metrics_debug` first-open route
contract, including route docs/owner metadata plus the shared demo, metrics, debug, handoff, and
action command descriptors. `apps/fretboard/src/demos.rs`, `apps/fret-devtools/src/native.rs`,
`apps/fret-devtools/src/demo_metrics_debug/actions.rs`, `apps/fret-devtools-mcp/src/native.rs`,
and `apps/fret-examples/src/imui_editor_workbench_demo.rs` now alias the shared owner instead of
repeating local string bags. The source gate and workbench golden-path surface test now freeze the
extraction so first-open discovery and the canonical workbench stop drifting independently.

2026-06-03 IMUI product-chain first-open shared workflow result:
`apps/fret-first-open/src/lib.rs` now owns the static `product_workflow` contract for
`imui-product-chain`, including default, focused, launched, suite, docs, and expected artifact
fields. `apps/fretboard/src/demos.rs`, `apps/fret-devtools/src/native.rs`, and
`apps/fret-devtools-mcp/src/native.rs` now alias that shared owner instead of carrying separate
local workflow constants. Discovery-line, GUI, and MCP projection remain in their existing owners;
only the first-open workflow contract moved.

2026-06-03 IMUI ResponseExt type owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover.rs` now keeps the hover response module declarations
and public re-exports only. `response/hover/types.rs` owns the `ResponseExt` field record with
`hover`-scoped field visibility, while `core_state.rs`, `hover_state.rs`, `lifecycle.rs`,
`press_context.rs`, `drag_accessors.rs`, and `query.rs` keep their existing impl responsibilities.
Public `ResponseExt` / `ImUiHoveredFlags` paths, core response accessors, hover flag queries,
lifecycle and press-context signals, drag accessors, and facade behavior remain unchanged, and the
source gate plus workstream manifest freeze the new type owner.

2026-06-03 IMUI table row-group horizontal flex owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/row_groups/layout.rs` now keeps only semantic
row-group entrypoints for outer, fill, pinned, and scroll-content table row groups.
`row_groups/layout/horizontal.rs` owns the shared horizontal flex chrome, including `FlexProps`
assembly, gap token resolution, zero padding, start justification, stretch alignment, and no-wrap
policy. Table row/header layout, pinned-column ordering, horizontal scroll wrapping, public IMUI
table APIs, and `fret-imui` table facade behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split across all repeated row-group source
checks.

2026-06-03 IMUI virtual-list row metrics/children owner-split result:
`ecosystem/fret-ui-kit/src/imui/virtual_list_controls/row.rs` now keeps only virtual-list row
wrapper chrome, fixed-height clipping, striped background, and list-item semantics.
`virtual_list_controls/row/metrics.rs` owns row height and row test-id projection, while
`virtual_list_controls/row/children.rs` owns empty/single/multi-child row packing. Fixed, known,
and measured virtual-list row behavior remains unchanged, and the source gate freezes the split.

2026-06-03 editor AxisDragValue typing branch owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps keyed state lookup,
current value reads, mode/test-id/theme projection, scrub owner routing, typing owner routing, and
final dual-surface mounting only. `controls/axis_drag_value/element/typing_element.rs` owns typing
branch orchestration across draft/error local model allocation, hidden/active typing layout
selection, typing TextInput mount, focus sync/handoff, key handler installation, draft-change error
clearing, and typing frame assembly. Scrub behavior, typing behavior, public `AxisDragValue`
options, and editor/IMUI adapter APIs remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-03 editor NumericInput joined field/frame owner-split result:
`ecosystem/fret-ui-editor/src/controls/numeric_input/element.rs` now keeps keyed runtime state
projection, duplicate-affix preparation, inline error composition, and outer root layout only.
`controls/numeric_input/element/field.rs` owns joined frame assembly, frame invalid/typing semantic
overrides, prefix/suffix affix mounting, text-entry owner invocation, and trailing error icon
composition. Draft/error model reads, duplicate-affix suppression, TextInput mount behavior,
focus-target capture, inline error layout, and public `NumericInput` options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

## M7 - P1 Active Teaching Drift Guard

Exit criteria:

- Keep active first-party IMUI teaching surfaces on the app-facing `fret::imui` facade.
- Permit direct low-level crate references only in owning-crate tests, compatibility/reference
  surfaces, recipes, and historical workstream evidence.
- Keep source gates aligned with the current owner split of proof helpers.

2026-06-03 active teaching direct-crate drift-guard result:
`tools/gate_imui_facade_teaching_source.py` now has an explicit active teaching path set covering
the cookbook IMUI lessons, product workbench/proof surfaces, examples docs, cookbook docs,
`ecosystem/fret` README, and the root README. Those paths reject direct `fret_imui::` and
`fret_ui_kit::imui::` teaching imports. The gate also now validates
`apps/fret-examples/src/imui_editor_proof_demo/proof_helpers.rs` as the owner of proof readout/text
helpers and outliner readout helpers instead of expecting stale definitions in the root proof demo.

## M6 - Continuing IMUI Owner-Split Pressure

Exit criteria:

- Continue reducing large `fret-ui-kit::imui` implementation files after worktree convergence.
- Keep public IMUI facade method names, options, responses, and behavior stable.
- Move policy sub-owners behind private modules and freeze the split with source gates.
- Run focused compile/test/source gates for each slice.

2026-06-03 IMUI child-region scroll carrier/chrome owner-split result:
`ecosystem/fret-ui-kit/src/imui/child_region/scroll/types.rs` now owns the child-region scroll input
carrier records with visibility limited to the child-region subtree, and
`ecosystem/fret-ui-kit/src/imui/child_region/scroll/chrome.rs` owns framed child-region scroll
chrome. `ecosystem/fret-ui-kit/src/imui/child_region/scroll.rs` keeps scroll-area builder
orchestration, child IMUI content mounting, scroll axis/show-scrollbar layout, scroll-handle
forwarding, viewport/root/content test-id routing, and element landing. Public
`child_region_with_options` behavior remains unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-03 IMUI debug-draw round-command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command/round.rs` now owns
circle/ngon/ellipse debug-draw command payload variants, including filled circle, filled ngon, and
filled ellipse payloads.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` keeps Bezier, mesh,
clip, media, text, and the `Round(DebugDrawRoundCommand)` wrapper plus existing wrappers. Public
`ImUiDebugDrawList` circle/ngon/ellipse APIs, command summaries, round point-count projection, path
paint dispatch, media dispatch filtering, residual shape dispatch, and debug-draw response APIs
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 IMUI debug-draw linear-command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command/linear.rs` now owns
line/poly/rect/quad/triangle debug-draw command payload variants, including convex/concave polygon
fills and multi-color rect fill.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` keeps round,
Bezier, mesh, clip, media, text, and the `Linear(DebugDrawLinearCommand)` wrapper. Public
`ImUiDebugDrawList` linear geometry APIs, command summaries, point/vertex/index/triangle counts,
path paint dispatch, residual multi-color rect paint, media dispatch filtering, and debug-draw
response APIs remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 IMUI debug-draw clip-command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command/clip.rs` now owns
push/pop clip-stack debug-draw command payload variants.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` keeps geometry,
mesh, media, text, and the `Clip(DebugDrawClipCommand)` wrapper. Public `ImUiDebugDrawList`
clip APIs, command summaries, clip-depth/clip-rect projection, paint clip push/pop behavior,
media dispatch filtering, residual shape paint dispatch, and debug-draw response APIs remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 DevTools Demo/Metrics/Debug action-catalog owner-split result:
`apps/fret-devtools/src/demo_metrics_debug/actions.rs` now owns the Demo/Metrics/Debug action
catalog, per-action copy command ids, action command bundle text, metadata lines, and
selected-bundle readiness projection.
`apps/fret-devtools/src/demo_metrics_debug.rs` keeps the always-visible route line projection,
workflow readiness/status/result/artifact handoff lines, runtime state reads, panel assembly,
action-row UI, and thin root functions used by `native.rs`. Existing route behavior remains
unchanged, and the DevTools source gates now read both owners.

2026-06-03 IMUI debug-draw mesh-command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command/mesh.rs` now owns
triangle mesh and image triangle mesh debug-draw command payload variants.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` keeps geometry,
clip, media, text, and the `Mesh(DebugDrawMeshCommand)` wrapper. Public
`ImUiDebugDrawList` mesh APIs, command summaries, paint dispatch, debug-draw response APIs,
media image/SVG behavior, and text behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 IMUI debug-draw media-command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command/media.rs` now owns
raster, rounded-image, and SVG debug-draw command payload variants.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` keeps geometry,
clip, image mesh, text, and the `Media(DebugDrawMediaCommand)` wrapper. Public
`ImUiDebugDrawList` image/SVG APIs, command summaries, paint dispatch, debug-draw response APIs,
image mesh behavior, and text behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 docking declarative target-resolution owner-split result:
`ecosystem/fret-docking/src/dock/declarative/drag_resolve/target.rs` now owns layout snapshot
lookup, dock-bounds projection, tab-width/tab-scroll preparation, theme-derived hint sizing,
docking policy lookup, candidate diagnostics collection, dragged-tab lookup, and
`resolve_dock_drop_target(...)`. `ecosystem/fret-docking/src/dock/declarative/drag_resolve.rs`
keeps hover/drop lifecycle branching, tear-off handoff, drop-intent dispatch, effect application,
diagnostics capture/publication, debug tracing, and drag allow checks. Public docking APIs,
diagnostics payloads, previous-hover latching, and target-resolution behavior remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 docking declarative drop-intent preparation owner-split result:
`ecosystem/fret-docking/src/dock/declarative/drag_resolve/drop_intent.rs` now owns panel/tabs
payload-to-intent preparation, `DockPanelDropDrag` / `DockTabsDropDrag` construction, declarative
tear-off allow checks, and default floating rect fallback from last panel sizes.
`ecosystem/fret-docking/src/dock/declarative/drag_resolve.rs` keeps target resolution,
`apply_dock_drop_intent(...)`, diagnostics capture/publication, debug tracing, hover-time
auto-scroll routing, and panel/tabs drag allow checks. Public docking APIs, diagnostics payloads,
and drop-intent behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-03 docking declarative drag-hover auto-scroll owner-split result:
`ecosystem/fret-docking/src/dock/declarative/drag_resolve/hover_autoscroll.rs` now owns hover-time
tab-bar drag auto-scroll gating, target tab stack lookup, tab-bar geometry projection,
`declarative_apply_tab_bar_drag_auto_scroll(...)`, and tab scroll synchronization.
`ecosystem/fret-docking/src/dock/declarative/drag_resolve.rs` keeps hover/drop target resolution,
drop intent effect projection, tear-off handoff, diagnostics capture/publication, debug tracing,
and panel/tabs drag allow checks. Public docking APIs, diagnostics payloads, and tab-drag
auto-scroll behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-03 docking declarative drag-resolve diagnostics owner-split result:
`ecosystem/fret-docking/src/dock/declarative/drag_resolve/diagnostics.rs` now owns drag hover/drop
diagnostics capture, graph stats/signature capture, `DockDropResolveDiagnostics` construction,
hover update/clear side effects needed for diagnostics, and `WindowInteractionDiagnosticsStore`
publication. `ecosystem/fret-docking/src/dock/declarative/drag_resolve.rs` keeps hover/drop target
resolution, drop intent effect projection, tab-bar auto-scroll, tear-off handoff, debug tracing,
and panel/tabs drag allow checks. Public docking APIs and diagnostics payloads remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 docking declarative begin-drag owner-split result:
`ecosystem/fret-docking/src/dock/declarative/drag_resolve/begin_drag.rs` now owns panel/tabs
cross-window drag-session startup and drag payload construction.
`ecosystem/fret-docking/src/dock/declarative/drag_resolve.rs` keeps internal drag hover/drop
resolution, drop intent effect projection, tab-bar auto-scroll, tear-off handoff, diagnostics
publication, and panel/tabs drag allow checks. Public docking APIs, drag inversion flags,
grab-offset propagation, tab active-index capture, and tear-off payload defaults remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor theme preset picker ListBox render owner-split result:
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker/render/listbox.rs` now owns
ListBox semantics, header text, preset iteration, and picker container chrome.
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker/render.rs` keeps
`EditorThemePresetPickerRenderInput` and the build entry that delegates to the ListBox owner.
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker/render/row.rs` remains the
ListBoxOption row chrome owner, and `render/row/behavior.rs` remains the selected-preset activation
owner. Public editor/IMUI facade APIs, render input shape, preset replay behavior, and density
status labels remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 IMUI facade core identity owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/facade_core/identity.rs` now owns
`ImUiFacade::id`, `ImUiFacade::push_id`, and `ImUiFacade::for_each_keyed`.
`ecosystem/fret-ui-kit/src/imui/facade_writer/facade_core.rs` keeps storage, focus capture,
`cx_mut`, `add`, and the `UiWriter` implementation, while
`facade_core/disabled_scope.rs` remains the disabled-scope behavior owner. Public facade method
names, child facade construction, runtime preparation, build-focus capture, and disabled-scope
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 docking drop resolve diagnostics owner-split result:
`ecosystem/fret-docking/src/dock/drop_resolve/diagnostics.rs` now owns resolved target diagnostics,
preview diagnostics, and resolve diagnostics payload construction.
`ecosystem/fret-docking/src/dock/drop_resolve.rs` is now a pure private module and re-export hub
for diagnostics, intent, target, and floating-hit owners. Public docking APIs, target resolution,
drop-intent projection, floating hit testing, and diagnostics payloads remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 docking drop resolve intent owner-split result:
`ecosystem/fret-docking/src/dock/drop_resolve/intent.rs` now owns panel/tab drop intent projection,
in-window float intent projection, tear-off request gating, effect emission, invalidate-layout
toggling, and debug intent labels. `ecosystem/fret-docking/src/dock/drop_resolve.rs` now keeps
diagnostics orchestration and re-exports the intent owner API for existing declarative callers.
Public docking APIs, panel/tab drop intents, effect projection, invalidation behavior, target
resolution, debug labels, and diagnostics payloads remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 docking drop resolve target owner-split result:
`ecosystem/fret-docking/src/dock/drop_resolve/target.rs` now owns dock drop target resolution:
layout-map projection, tab-bar insert target resolution, inner/outer hint-pad target picking,
float/empty dock-space target classification, previous-hover latching, inverted docking, and
policy allow checks. `ecosystem/fret-docking/src/dock/drop_resolve.rs` now keeps drop-intent
projection, effect application, and diagnostics orchestration while re-exporting
`resolve_dock_drop_target(...)` for existing declarative callers. Public docking APIs, target
classification, tab insert resolution, hint picking, drop intents, effect projection, and
diagnostics payloads remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-03 docking drop resolve floating-hit owner-split result:
`ecosystem/fret-docking/src/dock/drop_resolve/floating_hit.rs` now owns floating-window close,
title-bar, and body hit classification plus floating layout-context projection used by dock drop
target resolution. `ecosystem/fret-docking/src/dock/drop_resolve.rs` keeps target resolution,
drop-intent projection, effect application, and diagnostics orchestration. Public docking APIs,
floating title-bar center-drop projection, tab-bar insert resolution, policy checks, drop intents,
effect projection, and diagnostics payloads remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 plot area/shaded/stems props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` is now a pure builder-owner facade that declares
private owners and re-exports public prop records while
`ecosystem/fret-plot/src/declarative/props/area.rs`,
`ecosystem/fret-plot/src/declarative/props/shaded.rs`, and
`ecosystem/fret-plot/src/declarative/props/stems.rs` own `AreaPlotPanelProps`,
`ShadedPlotPanelProps`, and `StemsPlotPanelProps` construction plus
output/state/style/axis-label/axis-scale/step-mode builder methods. Public prop type names,
builder signatures/defaults, panel entrypoints, optional IMUI adapter routing, paint/event owners,
output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 plot histogram2d props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private histogram2d, heatmap,
candlestick, bars, histogram, error-bars, and line builder owners, re-exports public prop records,
and keeps remaining plot prop builders while
`ecosystem/fret-plot/src/declarative/props/histogram2d.rs` owns `Histogram2DPlotPanelProps`
construction, the `style.heatmap_show_colorbar = true` default, and
output/state/style/axis-label/axis-scale/step-mode builder methods. Public prop type names,
builder signatures/defaults, panel entrypoints, optional IMUI adapter routing, paint/event owners,
output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 plot heatmap props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private heatmap, candlestick, bars,
histogram, error-bars, and line builder owners, re-exports public prop records, and keeps remaining
plot prop builders plus the histogram2d colorbar default while
`ecosystem/fret-plot/src/declarative/props/heatmap.rs` owns `HeatmapPlotPanelProps` construction,
the `style.heatmap_show_colorbar = true` default, and
output/state/style/axis-label/axis-scale/step-mode builder methods. Public prop type names,
builder signatures/defaults, panel entrypoints, optional IMUI adapter routing, paint/event owners,
output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot candlestick props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private candlestick, bars,
histogram, error-bars, and line builder owners, re-exports public prop records, and keeps remaining
plot prop builders plus heatmap colorbar defaults while
`ecosystem/fret-plot/src/declarative/props/candlestick.rs` owns `CandlestickPlotPanelProps`
construction and output/state/style/axis-label/axis-scale/step-mode builder methods. Public prop
type names, builder signatures/defaults, panel entrypoints, optional IMUI adapter routing,
paint/event owners, output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot bars props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private bars, histogram,
error-bars, and line builder owners, re-exports public prop records, and keeps remaining plot prop
builders plus heatmap colorbar defaults while `ecosystem/fret-plot/src/declarative/props/bars.rs`
owns `BarsPlotPanelProps` construction and output/state/style/axis-label/axis-scale/step-mode
builder methods. Public prop type names, builder signatures/defaults, panel entrypoints, optional
IMUI adapter routing, paint/event owners, output publication, and plot model projection remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot histogram props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private histogram, error-bars, and
line builder owners, re-exports public prop records, and keeps remaining plot prop builders plus
heatmap colorbar defaults while `ecosystem/fret-plot/src/declarative/props/histogram.rs` owns
`HistogramPlotPanelProps` construction and output/state/style/axis-label/axis-scale/step-mode
builder methods. Public prop type names, builder signatures/defaults, panel entrypoints, optional
IMUI adapter routing, paint/event owners, output publication, and plot model projection remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot error-bars props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private error-bars and line builder
owners, re-exports public prop records, and keeps remaining plot prop builders plus heatmap colorbar
defaults while `ecosystem/fret-plot/src/declarative/props/error_bars.rs` owns
`ErrorBarsPlotPanelProps` construction and output/state/style/axis-label/axis-scale/step-mode
builder methods. Public prop type names, builder signatures/defaults, panel entrypoints, optional
IMUI adapter routing, paint/event owners, output publication, and plot model projection remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot line props builder owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now declares the private line builder owner,
re-exports public prop records, and keeps remaining plot prop builders plus heatmap colorbar
defaults while `ecosystem/fret-plot/src/declarative/props/line.rs` owns `LinePlotPanelProps`
construction and output/state/style/axis-label/axis-scale/step-mode builder methods. Public prop
type names, builder signatures/defaults, panel entrypoints, optional IMUI adapter routing,
paint/event owners, output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot line/area/stems command owner-split result:
`ecosystem/fret-plot/src/declarative/commands.rs` is now a thin command projection hub that keeps
shared line/area path keys and re-exports private command owners while
`ecosystem/fret-plot/src/declarative/commands/line_area.rs` owns area fill closure, stems baseline
projection, and step pre/post expansion. Line/area/stems painter dispatch, style/color/draw-order
behavior, public panel props, panel entrypoints, optional IMUI adapter routing, paint/event owners,
output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot shaded command owner-split result:
`ecosystem/fret-plot/src/declarative/commands.rs` now re-exports shaded command entrypoints and
keeps shared line/area keys plus non-shaded command builders while
`ecosystem/fret-plot/src/declarative/commands/shaded.rs` owns shaded lower path-key projection,
sorted-series cursor interpolation, segment splitting, viewport x filtering, fallback aligned-series
projection, upper/lower stroke commands, and fill band closure. Shaded painter dispatch,
style/color/draw-order behavior, public panel props, panel entrypoints, optional IMUI adapter
routing, paint/event owners, output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot error-bars command owner-split result:
`ecosystem/fret-plot/src/declarative/commands.rs` now re-exports the error-bars command entrypoint
and keeps shared path keys plus non-error-bars command builders while
`ecosystem/fret-plot/src/declarative/commands/error_bars.rs` owns x/y cap command construction,
marker shape command construction, marker radius gating, and slice-vs-indexed series data
iteration. Error-bars painter dispatch, stroke style, color, public panel props, panel entrypoints,
optional IMUI adapter routing, paint/event owners, output publication, and plot model projection
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot bar/histogram command owner-split result:
`ecosystem/fret-plot/src/declarative/commands.rs` now re-exports bar and histogram command
entrypoints and keeps shared path keys plus non-bar/histogram command builders while
`ecosystem/fret-plot/src/declarative/commands/bar_histogram.rs` owns histogram bin closed-rect
command construction and grouped/stacked bar baseline closed-rect command construction. Paint
style/color/draw-order dispatch, public panel props, panel entrypoints, optional IMUI adapter
routing, paint/event owners, output publication, and plot model projection remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot candlestick command owner-split result:
`ecosystem/fret-plot/src/declarative/commands.rs` now re-exports candlestick command entrypoints
and keeps shared line/area/shaded keys plus non-candlestick command builders while
`ecosystem/fret-plot/src/declarative/commands/candlestick.rs` owns candlestick down-body path-key
projection, wick/body command construction, rectangle body closure, and device point budgeting.
Candlestick painter dispatch, colors, draw order, public panel props, panel entrypoints, optional
IMUI adapter routing, paint/event owners, output publication, and plot model projection remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot prop records owner-split result:
`ecosystem/fret-plot/src/declarative/props.rs` now re-exports public plot panel prop records and
keeps builder methods plus heatmap colorbar defaults while
`ecosystem/fret-plot/src/declarative/props/records.rs` owns the public `*PlotPanelProps` record
definitions. Public type names, field visibility, builder behavior, panel entrypoints, optional
IMUI adapter routing, paint/event owners, output publication, and plot model projection remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot box zoom interaction owner-split result:
`ecosystem/fret-plot/src/declarative/interaction.rs` now keeps legend event routing, pointer event
snapshots, selection overlay records, shared mouse-button helpers, and child interaction re-exports
while `ecosystem/fret-plot/src/declarative/interaction/box_zoom.rs` owns box zoom session state,
modifier expansion, active-selection updates, axis-lock filtering, clamp/sanitize handling, and
view-bound update routing. Paint owners, output publication, active-selection rendering, public
panel props, plot model projection, optional IMUI adapter routing, and retained-free boundaries
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot query interaction owner-split result:
`ecosystem/fret-plot/src/declarative/interaction.rs` now keeps legend event routing and
re-exports query entrypoints while
`ecosystem/fret-plot/src/declarative/interaction/query.rs` owns query drag session state, query
selection overlay updates, query rectangle data projection, and `PlotState::query` update routing.
Paint owners, output publication, active-selection rendering, public panel props, plot model
projection, optional IMUI adapter routing, and retained-free boundaries remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot pan interaction owner-split result:
`ecosystem/fret-plot/src/declarative/interaction.rs` now keeps legend event routing and re-exports
pan entrypoints while
`ecosystem/fret-plot/src/declarative/interaction/pan.rs` owns pan session state, pointer-drag
routing, axis-lock filtering, and scaled pan view-bound projection. Paint owners, output
publication, public panel props, plot model projection, optional IMUI adapter routing, and
retained-free boundaries remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-02 plot wheel zoom interaction owner-split result:
`ecosystem/fret-plot/src/declarative/interaction.rs` now keeps legend event routing and re-exports
wheel zoom entrypoints while
`ecosystem/fret-plot/src/declarative/interaction/wheel.rs` owns wheel region detection,
modifier-to-axis selection, axis-lock filtering, clamp/sanitize handling, and view-bound update
projection. Paint owners, output publication, public panel props, plot model projection, optional
IMUI adapter routing, and retained-free boundaries remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot draggable interaction owner-split result:
`ecosystem/fret-plot/src/declarative/interaction.rs` now keeps legend event routing and re-exports
draggable interaction entrypoints while
`ecosystem/fret-plot/src/declarative/interaction/draggable.rs` owns draggable overlay hit-testing,
drag-session mutation, multi-axis drag transform selection, and `PlotDragOutput` projection. Paint
owners, output publication, public panel props, plot model projection, optional IMUI adapter
routing, and retained-free boundaries remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot annotation overlay helper owner-split result:
`ecosystem/fret-plot/src/declarative/overlays.rs` is now an overlay re-export hub while
`ecosystem/fret-plot/src/declarative/overlays/annotation.rs` owns shared annotation token
resolution, annotation label formatting, text-box emission, tag marker boxes, and plot-bound
clamping for tag, text, and draggable-label owners. Reference-line, draggable-shape, image, tag,
text, draggable-label, panel orchestration, event routing, public panel props, plot model
projection, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot draggable-shape overlay paint owner-split result:
`ecosystem/fret-plot/src/declarative/overlays.rs` now keeps overlay re-exports while delegating
draggable point and rectangle projection to
`ecosystem/fret-plot/src/declarative/overlays/draggable_shapes.rs`. The private draggable-shape
owner owns point/rect transform projection, right-axis view-bound routing, style fallback colors,
and direct quad emission. Reference-line, image, draggable-label, tag, text, panel orchestration,
event routing, public panel props, plot model projection, and optional IMUI adapter routing remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 plot reference-line overlay paint owner-split result:
`ecosystem/fret-plot/src/declarative/overlays.rs` now keeps overlay re-exports while delegating
infinite-line and draggable-line rectangle projection to
`ecosystem/fret-plot/src/declarative/overlays/reference_lines.rs`. The private reference-line owner
owns x/y reference line transform projection, right-axis view-bound routing, style crosshair
fallback, and filled-rect emission. Image, draggable-label, tag, text, panel orchestration, event
routing, public panel props, plot model projection, and optional IMUI adapter routing remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor AxisDragValue typing focus owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps keyed scrub/typing
orchestration, input and frame routing, and key-handler installation while delegating typing focus
lifecycle to `ecosystem/fret-ui-editor/src/controls/axis_drag_value/element/typing_focus.rs`. The
private focus owner preserves focus-driven return-to-scrub behavior, shared numeric focus sync,
focus-handoff timer arming, last-draft refresh while unfocused, and draft-change error clearing.
Typing key registration order, scrub/typing frame routing, public AxisDragValue behavior, and IMUI
adapter routing remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor ColorEdit caller-keying owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element.rs` now keeps public `ColorEdit`,
state/model setup, input/swatch construction, delivered-drop application, overlay requests, test-id
owner handoff, and root layout orchestration while delegating root keyed mounting to
`ecosystem/fret-ui-editor/src/controls/color_edit/element/keying.rs`. The private keying owner
captures the color model id, preserves explicit `id_source` precedence, preserves `#[track_caller]`
callsite fallback behavior, and mounts through the existing keyed element assembly. Public
`ColorEdit` behavior, popup routing, drag/drop routing, and IMUI adapter routing remain unchanged,
and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-02 editor ColorEdit root test-id owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element.rs` now keeps public `ColorEdit`,
caller-keyed root mounting, state/model setup, input/swatch construction, delivered-drop
application, overlay requests, and root layout orchestration while delegating child test-id
derivation to `ecosystem/fret-ui-editor/src/controls/color_edit/element/test_ids.rs`. The private
test-id owner preserves explicit child test-id precedence and root-test-id fallback suffixes for
input, swatch, popup, tooltip, copy-menu, and eyedropper ids. Public `ColorEdit` behavior, popup
request routing, drag/drop routing, and IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit alpha preview gradient/thumb owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/alpha/preview.rs` now keeps only
the horizontal and vertical alpha preview stack entrypoints, checkerboard layering, and child
ordering. `alpha/preview/gradient.rs` owns horizontal/vertical alpha gradient grids, alpha-step
projection, and `color_from_rgb_preserving_alpha(...)` use; `alpha/preview/thumb.rs` owns
horizontal/vertical thumb overlays, marker chrome, shared horizontal spacer routing, and vertical
spacer behavior. Alpha bar pressable behavior, alpha coordinate mapping, popup picker composition,
and public ColorEdit / IMUI facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py`
plus `imui_surface_policy` freeze the child-owner split.

2026-06-05 editor ColorEdit alpha bar surface owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/alpha/bar.rs` now keeps the
horizontal/vertical alpha bar pressable entrypoints, slider a11y props, pointer capture/release,
and alpha mutation routing only. `alpha/bar/surface.rs` owns focused border/ring resolution,
clipped frame chrome, 1px padding, and horizontal/vertical preview stack mounting. Horizontal and
vertical alpha bar import paths, pointer lifecycle, alpha coordinate mapping, preview visuals, and
public ColorEdit / IMUI facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py`
plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit alpha bar pointer owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/alpha/bar.rs` now keeps the
horizontal/vertical alpha bar pressable entrypoints, slider a11y props, and delegation to the
private pointer and surface owners. `alpha/bar/pointer.rs` owns horizontal/vertical pointer
down/move/up handler installation, left-button gating, pointer capture/release, and alpha mutation
routing through the existing alpha interaction owner. Alpha bar import paths, slider a11y values,
focused surface rendering, alpha coordinate mapping, preview visuals, and public ColorEdit / IMUI
facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py` plus
`imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit hue-wheel picker pointer owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/hue_wheel_picker.rs` now keeps the
hue-wheel picker pressable entrypoint, slider a11y props/value, focused border/ring surface, and
`hue_wheel_canvas(...)` mounting. `hue_wheel_picker/pointer.rs` owns drag-target storage,
horizontal/vertical local-position use, pointer down/move/up handler installation, left-button
gating, capture/release cleanup, target hit testing, and `apply_hue_wheel_position(...)` routing
through the existing HSV mutation owner. Hue-wheel picker import paths, drag target semantics,
HSV mutation behavior, focused surface rendering, canvas painting, and public ColorEdit / IMUI
facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py` plus
`imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit SV preview grid/thumb owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/sv/preview.rs` now keeps only the
SV preview stack entrypoint and child ordering. `sv/preview/grid.rs` owns `SV_PICKER_STEPS`, grid
track construction, saturation/value cell projection, `unit_from_step(...)`, and
HSV-to-color rendering. `sv/preview/thumb.rs` owns the saturation/value thumb overlay, vertical
spacer, marker chrome, and shared horizontal thumb spacer reuse. SV preview stack ordering, grid
color projection, thumb geometry, popup picker composition, and public ColorEdit / IMUI facade APIs
remain unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the
split.

2026-06-05 editor ColorEdit hue-bar preview gradient/thumb owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/hue_bar/preview.rs` now keeps only
the hue-bar preview stack entrypoint and child ordering. `hue_bar/preview/gradient.rs` owns
`HUE_BAR_STEPS`, vertical grid track construction, hue-step cell projection, and HSV-to-color
rendering. `hue_bar/preview/thumb.rs` owns the vertical thumb overlay, spacer, and marker chrome.
Hue preview stack ordering, hue-step color projection, thumb geometry, popup picker composition,
and public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit fill-preview checkerboard owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview/fill.rs` now keeps the
fill-preview stack entrypoint, alpha-preview mode dispatch, shared fill layout, absolute overlay
layout, and opaque/alpha visibility color helpers. `preview/fill/checkerboard.rs` owns the
checkerboard grid, grid track construction, cell parity color policy, and light/dark checkerboard
token use. Checkerboard/overlay ordering, half-alpha preview composition, alpha-preview reuse by
the popup alpha picker, and public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit side-preview cell owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/preview/side.rs` now keeps side-preview
stack assembly, current/original cell ordering, original child routing, and side/root test-id
propagation only. `preview/side/cell.rs` owns current cell construction, shared preview cell
content/layout, caption text-role styling, swatch dimensions, preview stack mounting, and formatted
a11y value projection. `preview/side/original.rs` continues to own restore activation and now
reuses the cell owner for shared layout/content. Current/original ordering, original restore
behavior, side-preview dimensions, preview a11y values, and public ColorEdit / IMUI facade APIs
remain unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the
split.

2026-06-05 editor ColorEdit picker option card owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/options/picker.rs` now keeps picker option
row composition, Hue Bar / Hue Wheel option ordering, row-level test-id derivation, and selection
routing only. `options/picker/card.rs` owns the picker radio-card pressable, radio a11y state,
runtime picker writeback, selected/disabled palette, thumbnail mounting, caption text, card sizing,
and redraw request. Hue Bar / Hue Wheel runtime mutation, thumbnail reuse, caption text role,
option-card sizing, and public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit drag-source handler owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/drag_drop/source.rs` now keeps only drag
threshold token resolution, finite/non-negative fallback policy, and the re-export for
`install_color_drag_source(...)`. `drag_drop/source/handlers.rs` owns drag kind derivation,
pointer down/move/up handler installation, cross-window and same-window drag startup, active drag
store updates, delivered-drop insertion, cancel cleanup, threshold-exceeded math, and
skip-activate behavior. Drag threshold semantics, cross-window source behavior, same-window source
behavior, delivery recording, and public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit keyed frame owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element.rs` now keeps only the public
`ColorEdit` type, constructor/options builder, `into_element(...)`, and private keyed delegation.
`element/frame.rs` owns keyed frame assembly: local state model reads, editor density and popup
padding resolution, current color/hex projection, drag/drop store setup and pruning, test-id
projection, input/swatch construction, delivered-drop application, popup/tooltip/copy overlay
requests, and root layout handoff. Public ColorEdit APIs, caller-keyed identity behavior,
swatch/input semantics, drag/drop delivery, overlay routing, and IMUI facade APIs remain unchanged,
and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit drag/drop store owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/drag_drop.rs` now keeps drag/drop root routing,
source and store re-exports, target hover updates, delivered-drop application, and payload
conversion only. `drag_drop/store.rs` owns the global store model, `ColorDragDropStore`,
`ActiveColorDrag`, `DeliveredColorDrop`, store allocation, stale active-session pruning, and
delivered-drop tick retention. Store model identity, active drag session tracking, delivered-drop
retention, source handler record construction, drop-target hover behavior, delivered payload
application, palette slot payload conversion, and public ColorEdit / IMUI facade APIs remain
unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit drag/drop delivery owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/drag_drop.rs` now keeps source/store/delivery
re-exports and target hover updates only. `drag_drop/delivery.rs` owns delivered-drop tick
validation, delivered payload removal, root model/draft/error writeback, RGB/RGBA alpha
preservation, and palette slot payload conversion. Drop-target hover behavior, source handler
records, store model records, delivered-drop application semantics, palette slot conversion, and
public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit picker popup layout owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs` now keeps module declarations,
picker child re-exports, picker constants, shared thumb spacer/border helpers, and shared HSV color
application only. `picker/layout.rs` owns `hsv_picker(...)`, `hsv_hue_wheel_picker(...)`,
SV/hue/wheel/alpha child mounting, derived test-id routing, and the horizontal picker `FlexProps`
row. HSV Hue Bar / Hue Wheel picker selection, alpha visibility, shared color application, picker
layout metrics, and public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit auxiliary policy record owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/options.rs` now keeps the public
`ColorEditOptions` aggregate, root Debug/Default assembly, palette/history/callback fields, popup
re-exports, and policy record re-exports only. `options/policies.rs` owns
`ColorEditAlphaPreview`, `ColorEditDragDropOptions`, `ColorEditTooltipOptions`,
`ColorEditCopyOptions`, and their defaults. Public option type names, default values,
`ColorEditOptions` field layout, popup options, palette/history callbacks, and public ColorEdit /
IMUI facade APIs remain unchanged, and the workstream manifest, source gate, and
`imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit HSV model owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/model.rs` now keeps hex parse/format plus
hue-wheel, numeric, and HSV owner re-export routing only. `model/hsv.rs` owns `HsvColor`,
RGB/HSV conversion, alpha-preserving color conversion, SV/hue local-position mapping, unit/hue
sanitization, step projection, and picker a11y text helpers. Hex parsing/formatting, numeric tests,
hue-wheel model ownership, picker interaction behavior, and public ColorEdit / IMUI facade APIs
remain unchanged, and the workstream manifest, source gate, and `imui_surface_policy` freeze the
split.

2026-06-05 editor ColorEdit swatch element owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/swatch.rs` now keeps module declarations,
`ColorEditSwatchArgs`, and the `color_swatch(...)` re-export only. `swatch/element.rs` owns the
pressable root, focus ring and a11y props, activation hook installation, context-menu hook
orchestration, drag-source installation, drop-target hover updates, visual child mounting,
test-id routing, and swatch a11y value assignment. Activation behavior, context-menu behavior,
drag/drop behavior, visual state projection, and public ColorEdit / IMUI facade APIs remain
unchanged, and the workstream manifest, source gate, and `imui_surface_policy` freeze the split.

2026-06-02 editor VecEdit caller-keying owner-split result:
`ecosystem/fret-ui-editor/src/controls/vec_edit/model.rs` now keeps the public Vec2/Vec3/Vec4 model
records, constructors, builder-style setters, and presentation affix adoption while delegating
caller-keyed `into_element(...)` routing to
`ecosystem/fret-ui-editor/src/controls/vec_edit/model/keying.rs`. The private keying owner captures
Vec2/Vec3/Vec4 model-id tuples, preserves explicit `id_source` precedence and
`#[track_caller]` callsite fallback behavior, and mounts through the existing keyed element
entrypoints. Public VecEdit APIs, keyed element assembly, axis ordering, and layout semantics remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor EnumSelect overlay panel owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay.rs` now keeps open/filter/reveal state
preparation, placement policy, close-focus policy, and dismiss request assembly while delegating
anchored popup panel composition to
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay/panel.rs`. The private panel owner
contains anchored props, popup panel chrome, search/list column layout, search test-id mounting,
list viewport test-id derivation, and root list test-id mounting. Filter, list viewport, empty row,
selected-row reveal, row rendering, and dismiss behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor TextAssistField keyboard owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field/element.rs` now keeps public control
construction, controller/expanded-state/semantics preparation, field/panel/empty layout, and
overlay routing while delegating root key-handler installation to
`ecosystem/fret-ui-editor/src/controls/text_assist_field/element/keyboard.rs`. The private keyboard
owner contains input-owned text-assist key policy forwarding, query/dismissed-query/active-id model
handoff, and keyboard acceptance routing through `accept_text_assist_match(...)`. Public
`TextAssistField` APIs, panel row activation, overlay routing, accept semantics, and redraw behavior
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor ColorEdit swatch activation owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/swatch.rs` now keeps swatch
pressable/chrome/drag/drop/tooltip orchestration while delegating popup activation to
`ecosystem/fret-ui-editor/src/controls/color_edit/swatch/activation.rs`. The private activation
owner contains visible-content gating, original-reference capture, popup open toggling, copy-menu
closing, and redraw requests. Context-menu routing, drag/drop hooks, tooltip state, preview chrome,
pressable registration, and public `ColorEdit` behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor GradientEditor preview paint owner-split result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor/preview.rs` now keeps preview state,
pressable assembly, and pointer down/move/up stop mutation while delegating canvas painting to
`ecosystem/fret-ui-editor/src/composites/gradient_editor/preview/paint.rs`. The private paint owner
contains gradient vector projection, stop clamping/fallback stops, preview quad painting, marker
geometry, active marker resolution, and marker painting. Canvas layout, drag behavior, public
gradient editor APIs, and IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-02 editor ColorEdit popup body layout owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/body.rs` now keeps popup model reads,
effective option resolution, and picker/numeric/swatches/eyedropper element creation while
delegating popup content ordering, picker-plus-side-preview row layout, and width selection to
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/body/layout.rs`. Popup defaults, runtime
picker options, side-preview visibility, standalone alpha-bar behavior, and public `ColorEdit`
options remain unchanged, and `tools/gate_imui_workstream_source.py` plus
`imui_surface_policy` freeze the new body/layout owner boundary.

2026-06-02 fret-imui composition region-container test owner-split result:
`ecosystem/fret-imui/src/tests/composition/layout_collections.rs` now keeps the layout, porting
sugar, table, virtual-list, separator, and bullet-text composition coverage while delegating
ChildRegion/ListBox region-container coverage to
`ecosystem/fret-imui/src/tests/composition/layout_collections/region_containers.rs`. Test names and
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the parent router plus
region-container child owner boundary.

2026-06-02 IMUI begin-submenu open-policy read owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu/open_policy.rs` now keeps
clicked-trigger reconciliation, stale close cleanup, and popup close/open dispatch while delegating
submenu `open_value` readback to `submenu/open_policy/read.rs`. Submenu hover/shortcut behavior,
sibling switching, popup anchoring, and begin-submenu response semantics remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the read owner boundary.

2026-06-02 IMUI debug-draw residual shape paint owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes.rs` now keeps draw-order/key setup
and path-vs-residual dispatch while delegating filled-rect, vertex-color rect, triangle mesh,
image-triangle mesh, text paint, and exhaustive no-op residual routing to
`paint_shapes/residual.rs`. Path command routing, media paint dispatch, clip balancing, draw-list
recording, and public debug-draw summaries remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the new root/residual boundary.

2026-06-02 IMUI begin-menu active-trigger child owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy/active_trigger.rs`
now keeps guard/orchestration and the reconcile re-export while delegating current-trigger
group-active readback to `active_trigger/read.rs` and `MenubarActiveTrigger` plus row-open
activation writes to `active_trigger/activate.rs`. Menubar open-menu synchronization, post-trigger
reconciliation, and begin-menu response semantics remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 IMUI begin-menu open-request bridge owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu.rs` now keeps trigger mounting, popup body
mounting, disabled cleanup routing, and final `DisclosureResponse` assembly while delegating
open-request resolution bridging to `menu_family_controls/menu/open.rs`. The private open owner
contains resolve-open-request, menubar active-trigger activation, and trigger-rect
`ui.open_popup_at(...)` dispatch. Menubar hover/open behavior, popup body mounting, disabled-popup
cleanup, and public IMUI menu response semantics remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 IMUI floating-area drag-state commit owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/area/drag_state.rs` now keeps drag snapshot
discovery, scale-factor lookup, prepared output assembly, and final-state re-export routing while
delegating drag-state commit to `floating_surface/area/drag_state/commit.rs` and final placement
readback to `floating_surface/area/drag_state/final_state.rs`. The commit owner contains
initial/test-id state construction, drag delta application, device-pixel snapping, and
last-drag-position cleanup. Floating-area dragging/position response semantics, test-id overrides,
and child-window resize feedback remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-01 IMUI table-column visibility mutation owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/state/overrides.rs` now keeps construction
and read-side override queries while delegating `set_visible`, `show`, `hide`, `toggle`, `remove`,
and `clear` to private `table_column_visibility/state/mutation.rs`. Empty-id filtering,
last-entry-wins behavior, snapshot restoration, table-column application, and public
`ImUiTableColumnVisibilityState` methods remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 IMUI input-text picker input-root child owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/input.rs` now keeps input-root request/result
shapes, text input mounting, and fill-width root container construction while delegating
ComboBox assistive semantics to `text_picker_controls/input/semantics.rs` and focused-input keyboard
handler gating/candidate forwarding to `text_picker_controls/input/keyboard.rs`. Completion/history
picker behavior, popup-open state, active-descendant semantics, root sizing, keyboard navigation
conditions, and public IMUI options/responses remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 IMUI text picker input-root type owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/input/types.rs` now owns
InputTextPickerInputRootRequest and BuiltInputTextPickerInputRoot data shapes. `input.rs` keeps text
input mounting, response capture, root container construction, and keyboard install. Completion/
history picker behavior, popup-open forwarding, assistive semantics, root fill sizing, and public
IMUI options/responses remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-01 editor NumericInput affix segment owner-split result:
`ecosystem/fret-ui-editor/src/controls/numeric_input/element.rs` now keeps joined field/frame
orchestration, input owner invocation, and error owner invocation while delegating prefix/suffix
segment chrome to private `controls/numeric_input/element/affix.rs`. The affix owner contains muted
text color resolution, frame text-px/padding usage, prefix/suffix test-id routing, and a11y label
stamping. Prefix/suffix duplicate suppression, segment order, trailing error icon composition,
text-entry mounting, and public `NumericInput` options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor NumericInput text-entry mounting owner-split result:
`ecosystem/fret-ui-editor/src/controls/numeric_input/element.rs` now keeps keyed field/frame
assembly, affix routing, and error owner invocation while delegating TextInput props construction,
focus-target capture, focus-state synchronization, last-draft tracking, key handler installation,
and draft/error cleanup to private `controls/numeric_input/element/input.rs`. TextInput enabled,
focusable, placeholder, test-id, invalid a11y state, joined input chrome, editor numeric text style,
selection behavior, keyboard commit/cancel behavior, affix rendering, and public `NumericInput`
options remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor NumericInput error presentation owner-split result:
`ecosystem/fret-ui-editor/src/controls/numeric_input/element.rs` now keeps keyed field assembly,
affix rendering, draft/focus sync, and keyboard handler wiring while delegating trailing error icon
and inline validation text rendering to private `controls/numeric_input/element/error.rs`. The
error owner contains error display-mode gating, invalid border/foreground color resolution,
validation-message text role routing, error icon/test-id stamping, inline error test-id/a11y label
routing, and source text size/line-height adoption. Public `NumericInput` constructors, options,
keyboard behavior, draft/error model ownership, and affix behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor PropertyGroup element owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_group.rs` now keeps the public
`PropertyGroup` builder/toggle API and delegates header/content/root construction to private
`composites/property_group/element.rs`. The element owner contains theme metric/color resolution,
collapsed model reads and toggle mutation, disclosure icon choice, hover/press header chrome,
header action slot mounting, content visibility, test-id routing, root flex decoration, and outer
panel chrome. Public `PropertyGroupOptions`, toggle callback behavior, collapsed defaults, and
builder import paths remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor VecEdit element assembly owner-split result:
`ecosystem/fret-ui-editor/src/controls/vec_edit/element.rs` now keeps Vec2/Vec3/Vec4 keyed
entrypoints and maps concrete axis fields to axis descriptors while delegating shared element
assembly to `controls/vec_edit/element/assembly.rs`. The private assembly owner contains layout
plan resolution, per-axis id/test-id derivation, axis color mapping, root flex chrome, axis group
mounting, numeric format/parse/validate forwarding, outcome forwarding, and root test-id
decoration. Public Vec2/Vec3/Vec4 constructors/builders, options, axis reset behavior, and layout
semantics remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextField buffered actions owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/buffered.rs` now keeps buffered state, focus
transition planning, draft model allocation, model-to-draft sync, focus/timer orchestration, blur
dispatch, and multiline commit shortcut classification while delegating commit/cancel finalizers to
`controls/text_field/buffered/actions.rs`. The private actions owner contains pending-blur
clearing, clear-state reset, model/draft commit and cancel finalizers, draft-controller finalizers,
outcome emission, submit-command dispatch, and redraw requests. Public `TextFieldDraftController`
and `TextField` options remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-01 editor TextField entry owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element.rs` now keeps public construction,
callsite/model keying, joined frame/chrome orchestration, current draft sync, clear trailing
segments, and field id reporting while delegating entry mounting/session wiring to
`controls/text_field/element/entry.rs`. The private entry owner contains TextInput/TextArea
selection and mounting, input-id reporting, buffered session sync, draft-controller binding,
buffered key routing, blur commit/cancel handling, focus-selection routing, and unbuffered
multiline Escape-clear installation. Public `TextField` options, entry props/chrome semantics,
clear-button behavior, and joined frame mounting remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor AxisDragValue scrub-element owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps scrub/typing
orchestration, focus sync, typing input/key/frame routing, error clearing, and final mounted
composition while delegating `DragValueCore` scrub assembly to
`controls/axis_drag_value/element/scrub_element.rs`. The private scrub-element owner contains
scrub options, live model update wiring, commit/cancel outcome callbacks, double-click typing
handoff, focus-handoff arming, scrub id recording, scrub response state mapping, and scrub frame
owner routing. Public AxisDragValue options, typing behavior, frame visuals/test ids/reset action,
and hidden-layout semantics remain unchanged, and `tools/gate_imui_workstream_source.py` freezes
the split.

2026-06-01 editor TextAssistField option-row owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field/panel.rs` now keeps listbox semantics,
scroll wrapping, popup surface chrome, and rendered panel packaging while delegating suggestion
option-row assembly to `controls/text_assist_field/panel/row.rs`. The private row owner contains
pressable props, option activation commit wiring, active/disabled row palette selection,
item test-id derivation, listbox option a11y fields, and row text rendering. Visible-match
semantics, scroll threshold, popup chrome, and `RenderedTextAssistPanel` handoff remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextAssistField accept owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now keeps input/key orchestration,
panel routing, inline empty-label helper policy, and max-height helper policy while delegating
match acceptance to `controls/text_assist_field/accept.rs`. The private accept owner contains
query model writes, dismissed-query sync, active item-id updates, user accept callback dispatch,
and redraw requests shared by keyboard acceptance and suggestion row activation. Public
`TextAssistField` options, root key handling, panel row activation, and overlay routing remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextAssistField empty-label owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now keeps input/key orchestration,
panel routing, and inline empty-label gating while delegating empty-label rendering to
`controls/text_assist_field/empty.rs`. The private empty owner contains popup empty-text props,
muted foreground resolution, density row-height routing, and empty test-id mounting. Inline gating,
panel routing, overlay routing, and public `TextAssistField` options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextField escape-clear owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element.rs` now keeps TextInput/TextArea
assembly, buffered key routing, focus-selection sync, blur handling, and clear-button composition
while delegating unbuffered multiline Escape-clear key capture to
`controls/text_field/element/escape_clear.rs`. The private escape-clear owner preserves
clear-on-Escape behavior and redraw requests with a focused key-classification test. Single-line
cancel command routing, buffered commit/cancel handling, clear-button behavior, and public
`TextField` options remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextField focus-selection owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element.rs` now keeps TextInput/TextArea
assembly, buffered key routing, blur handling, and clear-button composition while delegating
text-present detection plus shared focus-selection sync to `controls/text_field/element/focus.rs`.
The private focus owner contains buffered draft vs model value precedence and the call into shared
editor text-entry focus-selection policy. Select-all-on-focus behavior, timer dispatch,
single-line/multiline focus sync, buffered behavior, and public `TextField` options remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor EnumSelect trigger owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select.rs` now keeps public control construction,
model reads, trigger-key registration, and overlay routing while delegating trigger pressable and
visual assembly to `controls/enum_select/trigger.rs`. The private trigger owner preserves
min-height fallback, a11y combobox state, focus ring geometry, activate toggle behavior, trigger
press open-change reason, text/caret layout, divider, caret icon selection, and frame chrome.
Public `EnumSelect` options, overlay routing, trigger key policy, and row behavior remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor EnumSelect trigger-key owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select.rs` now keeps public control construction,
trigger visual composition, key-handler registration, and overlay routing while delegating
trigger keyboard open/close policy to `controls/enum_select/trigger_keys.rs`. The private key
owner preserves enabled gating, Enter/NumpadEnter/Space/ArrowDown open behavior, Escape close
behavior, open-change reason updates, and redraw requests with focused key intent tests. Public
`EnumSelect` options, trigger visuals, overlay routing, and row behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor EnumSelect overlay-reveal owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay.rs` now keeps overlay request and popup
layout orchestration while delegating selected-row reveal, active-descendant scroll-into-view,
viewport test-id derivation, pending-reveal clearing, and viewport visibility math to
`controls/enum_select/overlay/reveal.rs`. Close-focus policy, filtering, row routing, popup/list
layout, selected reveal behavior, and public `EnumSelect` options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor EnumSelect overlay-filter owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay.rs` now keeps overlay request assembly,
popup/list layout, selected-row reveal, close-focus policy, and dismiss behavior while delegating
query normalization plus label/value filtering to `controls/enum_select/overlay/filter.rs`. The
private filter owner preserves trim/lowercase matching, empty-query ordering, and label/value match
coverage with focused tests. Public `EnumSelect` options, row routing, selected reveal behavior,
and overlay chrome remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor EnumSelect overlay-empty owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay.rs` now keeps overlay request assembly,
popup/list layout, search field routing, row routing, selected-row reveal, close-focus policy, and
dismiss behavior while delegating empty result rendering to `controls/enum_select/overlay/empty.rs`.
The private empty owner contains the `No matches` popup readout text, muted foreground resolution,
and row-height routing. Public `EnumSelect` options, filtered row behavior, overlay chrome, and
reveal behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor EnumSelect overlay-list owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay.rs` now keeps overlay request assembly,
anchored panel chrome, search box routing, close-focus policy, and dismiss behavior while
delegating scroll/list viewport orchestration to `controls/enum_select/overlay/list.rs`. The
private list owner contains row collection, empty-state routing, scroll handle wiring, viewport
test-id propagation, selected-row capture, and reveal dispatch. Public `EnumSelect` options,
filtered ordering, row behavior, popup/search layout, and selected-row reveal timing remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 kit IMUI debug-draw stroke-style owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/options/stroke.rs` now keeps the public
`DebugDrawStrokeStyle` record, builders, defaults, invalid dash/miter guards, and method names
while delegating visibility/path-style projection to `options/stroke/style.rs`. The private style
owner contains the V1 `PathStyle::Stroke` fast path and explicit `StrokeV2` policy projection.
Public debug-draw option exports, stroke defaults, and tests remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValue element owner-split result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps the public control API, callsite /
id-source keying wrapper, module declarations, and `DragValueOptions` re-export while delegating
keyed element composition to `controls/drag_value/element.rs`. The private element owner contains
state lookup, current value reads, mode/scrub revision reads, duplicate chrome affix suppression,
test-id derivation, scrub/input owner routing, hidden input mounting, and final mounted
composition. Public `DragValue` constructors/builders, keying behavior, options, and scrub/typing
semantics remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValue scrub-element owner-split result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps keyed control orchestration and
scrub/input owner composition while delegating `DragValueCore` scrub assembly to
`controls/drag_value/scrub_element.rs`. The private scrub-element owner contains live model update
wiring, commit/cancel callback emission, scrub layout hiding while typing, double-click typing
handoff, focus-handoff arming, scrub id recording, scrub response state mapping, and scrub frame
owner routing. Public `DragValue` options, mounted scrub/input semantics, and scrub frame visuals
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValue typing owner-split result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps keyed control orchestration, scrub
mode switching, `DragValueCore` wiring, live model updates, and scrub/input composition while
delegating typed `NumericInput` assembly to `controls/drag_value/typing.rs`. The private typing
owner contains constrained parse wrapping, validation/options forwarding, hidden typing layout
consumption, commit/cancel outcome mapping, scrub focus restore, scrub revision bumping, redraw,
and numeric text-entry focus handoff. Public `DragValue` options, scrub frame behavior, and mounted
hidden input semantics remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-01 editor AxisDragValue typing-key owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps keyed scrub/typing
orchestration, mounted text input props, focus handoff, scrub frame assembly, and typing frame
routing. The private `controls/axis_drag_value/element/typing_keys.rs` owner contains
replace-on-focus key handling plus Enter commit and Escape cancel policy, including parse/validate,
constraint application, invalid-number reporting, draft/error sync, focus restore to scrub, scrub
revision bumping, and outcome routing. Public AxisDragValue options and scrub/typing frame behavior
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor AxisDragValue typing-input owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps scrub/typing
orchestration, focus handoff, typing key handling, error clearing, and frame routing while
delegating TextInput props and mount to `controls/axis_drag_value/element/input.rs`. The private
input owner contains hidden/active enabled and focusable gating, invalid a11y state, joined input
chrome, text style routing, test-id routing, input mounting, input id capture, and focus reads.
Public AxisDragValue options, typing key behavior, focus handoff, scrub mounting, and typing frame
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TransformEdit section-control owner-split result:
`ecosystem/fret-ui-editor/src/controls/transform_edit/element.rs` now keeps linked-scale
model/sync orchestration, Column/Row layout selection, section row/column mounting, and root
test-id decoration while delegating Vec3 section-control construction to
`controls/transform_edit/element/section_control.rs`. The private section-control owner contains
per-section presentation projection, id-source/test-id routing, validation forwarding, link-scale
test-id derivation, and transform-axis outcome mapping. Public TransformEdit options,
section presentation formats/parses/chrome affixes, linked-scale sync, and layout behavior remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextField entry-props owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element.rs` now keeps keyed construction, joined
frame assembly, buffered session orchestration, focus/blur/key handlers, clear-button composition,
and entry mounting while delegating TextInput/TextArea props construction to
`controls/text_field/element/entry_props.rs`. The private entry-props owner contains joined chrome,
field style resolution, single-line assistive semantics, password mode, submit/cancel command
forwarding, and multiline min-height/stable line-box policy. Public TextField options,
single-line/multiline routing, buffered behavior, focus selection, clear behavior, and mounted entry
IDs remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor Slider interaction owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider/element.rs` now keeps keyed state lookup, current
value reads, quantization, affix/test-id routing, pressable/frame composition, NumericInput typing
composition, and focus handoff sync while delegating pointer interaction handler installation to
`controls/slider/element/interaction.rs`. The private interaction owner contains click-to-update,
drag begin/move/up, missed pointer-up cleanup, double-click typing handoff, value math updates,
redraw requests, and col-resize cursor setting. Public Slider options, frame assembly,
NumericInput typing behavior, and focus handoff behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor VecEdit options owner-split result:
`ecosystem/fret-ui-editor/src/controls/vec_edit.rs` now keeps the public Vec2/Vec3/Vec4 control hub,
constructors, builder methods, keyed entrypoints, presentation adoption, and axis exports while
re-exporting `VecEditOptions` and `VecEditLayoutVariant`. The private
`controls/vec_edit/options.rs` owner contains public option fields, layout variant, and defaults.
Public import paths, default layout/auto-stack/gap/id/test-id behavior, Vec2/Vec3/Vec4
constructors, and layout/axis assembly remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-01 editor EnumSelect options owner-split result:
`ecosystem/fret-ui-editor/src/controls/enum_select.rs` now keeps the item record, control hub,
trigger/open-key orchestration, and overlay request routing while re-exporting
`EnumSelectOptions`. The private `controls/enum_select/options.rs` owner contains public option
fields and defaults. Public import paths, default layout, placeholder/none labels, max-list-height
and diagnostics fields, keyed state identity, trigger composition, open-key policy, and overlay
routing remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor TextField buffered-key owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element.rs` now keeps keyed construction, joined
frame/input/area assembly, focus selection sync, blur handler installation, and clear affordance
composition. The private `controls/text_field/element/buffered_keys.rs` owner contains buffered
single-line and multiline commit/cancel key routing, including IME/repeat guards, single-line Enter
commit, multiline Ctrl/Cmd+Enter commit, Escape cancel, submit-command forwarding, and outcome
routing. Text input/area composition, blur behavior, clear-button behavior, draft controller
binding, and public TextField options remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-01 editor TextField clear-button owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element.rs` now keeps input/textarea assembly and
delegates clear affordance construction. The private `controls/text_field/element/clear_button.rs`
owner contains clear visibility reads, buffered draft/model clearing, buffered-state reset, and
single-line/multiline clear segment selection. Clear-button visibility, draft/model clearing,
buffered session reset, single-line vs multiline clear button chrome, a11y label, test-id routing,
and redraw behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor Checkbox options owner-split result:
`ecosystem/fret-ui-editor/src/controls/checkbox.rs` now keeps model reads, tri-state behavior,
chrome resolution, pressable activation, indicator mounting, and chrome regression routing while
re-exporting `CheckboxOptions`. The private `controls/checkbox/options.rs` owner contains option
fields and defaults. Public `CheckboxOptions` import paths, auto layout defaults,
enabled/focusable defaults, a11y/test-id fields, bool/optional-bool model behavior, tri-state
chrome, token fallback, and pressable activation behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor Checkbox chrome owner-split result:
`ecosystem/fret-ui-editor/src/controls/checkbox.rs` now keeps model reads, a11y, pressable
activation, indicator mounting, and root control assembly while delegating token fallback chrome to
`controls/checkbox/chrome.rs`. The private chrome owner contains resolved chrome colors, editor
token precedence, generic palette fallback, and the chrome token regression. Bool/optional-bool
model behavior, tri-state indicator selection, focus-ring geometry, a11y/test-id routing, checked
foreground/background semantics, and pressable activation behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor Checkbox model owner-split result:
`ecosystem/fret-ui-editor/src/controls/checkbox.rs` now keeps a11y, pressable props, indicator
mounting, and root control assembly while delegating checked-state reads and activation toggling to
`controls/checkbox/model.rs`. The private model owner contains bool/optional-bool model variants,
paint invalidation reads, optional-bool tri-state mapping, disabled activation guard, toggle
mutation, and redraw request behavior. Public bool vs optional-bool constructors, a11y routing,
focus-ring geometry, chrome resolution, and indicator mounting remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor Checkbox indicator owner-split result:
`ecosystem/fret-ui-editor/src/controls/checkbox.rs` now keeps a11y, pressable props, root control
assembly, and visual-state calculation while delegating indicator container/icon mounting to
`controls/checkbox/indicator.rs`. The private indicator owner contains tri-state icon selection,
checked/indeterminate/unchecked icon mounting, box size/radius, border width, centered icon layout,
and icon color routing. A11y routing, focus-ring geometry, model behavior, chrome resolution, and
pressable activation behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes
the split.

2026-06-01 editor PropertyRow trailing-slot owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_row/element.rs` now keeps row/column layout,
value-slot marking, reset/action visibility decisions, and reset/action child routing while
delegating the fixed-width trailing slot wrapper to `composites/property_row/slot.rs`. The private
slot owner contains shared reset/action slot chrome: fixed width, min row height, clip overflow,
zero gap/padding, horizontal end alignment, and center cross-axis alignment. Row/column layout,
reset/action visibility, reset element routing, action mounting, value-slot overflow semantics, and
test-id propagation remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor PropertyGrid test owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_grid.rs` now keeps public grid options,
row-option resolution, row-context helper methods, and root grid composition while routing
regression coverage through `#[cfg(test)] mod tests;`. The private
`composites/property_grid/tests.rs` owner contains the narrow wrapping-layout regression fixture,
including wrapping text services, measured bounds helpers, row separation assertions, and test-id
capture. Public `PropertyGridOptions`, row option defaults, row composition, wrapping value text
measurement, and test-id propagation remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor GradientEditor stop-row owner-split result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps public composition,
preview/angle/stops group orchestration, add-stop behavior, and empty-state text role helper. The
private `composites/gradient_editor/stops.rs` owner contains stop-row PropertyRow assembly,
position DragValue, ColorEdit, remove button, and row/field test-id derivation. Stop sorting, row
identity/test-id derivation, position/color editor wiring, remove action routing, row layout,
empty-state text role, preview behavior, and public gradient editor options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor GradientEditor options owner-split result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps keyed element composition,
preview/angle/stops group orchestration, add-stop behavior, and empty-state text role helper while
re-exporting options. The private `composites/gradient_editor/options.rs` owner contains public
option/action/binding records and defaults. Public re-export paths, layout defaults,
enabled/preview/angle defaults, preview/stops/add-stop test-id fields, stop binding model fields,
add/remove action callback types, preview behavior, stop-row ordering, add-stop gating, and
empty-state text role behavior remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-01 editor GradientEditor angle owner-split result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps keyed element composition,
preview/stops/add-stop orchestration, and empty-state text role helper while delegating angle row
construction to the private `composites/gradient_editor/angle.rs` owner. `show_angle` gating,
angle model routing, derived angle test id, PropertyRow slot width overrides, Angle label text
role, DragValue degrees presentation, preview behavior, stop-row ordering, add-stop gating, and
public gradient editor options remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-01 editor GradientEditor Stops group owner-split result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps keyed element composition,
model reads, preview assembly, angle row routing, and root layout while delegating Stops group
construction to the private `composites/gradient_editor/stops_group.rs` owner. Stop-row sorting,
stops group test-id propagation, add-stop max-stop gating, add-stop action routing, PropertyGrid
row-option forwarding, stop-row mounting, empty-state text role behavior, preview behavior, angle
row behavior, and public gradient editor options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor GradientEditor stop model owner-split result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps keyed element composition
and final preview / angle / Stops group / root assembly while delegating stop model reads and
derived row data to the private `composites/gradient_editor/stops_model.rs` owner. Paint
invalidation model reads, transparent color fallback, preview stop clamping, preview stop sorting,
stop-row sorting, preview drag stop-model collection, preview assembly, Stops group assembly,
angle row behavior, and public gradient editor options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor InspectorPanel search owner-split result:
`ecosystem/fret-ui-editor/src/composites/inspector_panel/element.rs` now keeps panel
metrics/header/content/root assembly and delegates search field construction to the private
`composites/inspector_panel/element/search.rs` owner. Search query trimming/lowercase matching,
header visibility, enabled/focusable routing, clear-button test ids, `MiniSearchBox` fallback,
`TextAssistField` anchored overlay routing, search assist list/empty/key/test/max-height
forwarding, and public `InspectorPanel` options remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor InspectorPanel options owner-split result:
`ecosystem/fret-ui-editor/src/composites/inspector_panel.rs` now keeps public cx/control records,
builder methods, and child-owner routing while re-exporting options. The private
`composites/inspector_panel/options.rs` owner contains public option records and defaults. Public
`InspectorPanelOptions` and `InspectorPanelSearchAssistOptions` import paths, layout defaults,
enabled/title/test-id defaults, search assist option fields, search fallback behavior, and panel
assembly behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor InspectorPanel element owner-split result:
`ecosystem/fret-ui-editor/src/composites/inspector_panel.rs` now keeps public options/cx/control
records, builder methods, and `into_element_in(...)` routing. The private
`composites/inspector_panel/element.rs` owner contains scoped panel assembly, theme/chrome
resolution, header/title/toolbar layout, search/search-assist element selection, content mounting,
and root panel chrome. Public constructors/builders, `InspectorPanelCx` accessor shape, query
trimming/lowercase matching, title text-role behavior, search assist fallback, header/content/root
test-id propagation, panel chrome token fallback, and `into_element_in(...)` routing remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor PropertyGroup header owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_group/element.rs` now keeps metric/theme
resolution, collapsed-state reads, content/root/panel assembly, and delegates header construction
to the private `composites/property_group/element/header.rs` owner. Toggle callback behavior,
collapsed model mutation/redraw routing, disclosure icon choice, enabled/collapsible gating,
hover/press header chrome, header text role, header actions slot, header test-id propagation,
content visibility, and panel chrome remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-01 editor PropertyGroup options owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_group.rs` now keeps the public group control,
collapse/toggle behavior, header/content/root assembly, and re-exports `PropertyGroupOptions`.
The private `composites/property_group/options.rs` owner contains option fields and defaults.
Public `PropertyGroupOptions` import paths, layout defaults, collapsed model/default behavior,
enabled/collapsible defaults, header/content test-id fields, header rendering, content mounting,
and toggle callback routing remain unchanged, and `tools/gate_imui_workstream_source.py` freezes
the split.

2026-06-01 editor PropertyRow element owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_row.rs` now keeps the public composite, label
helper, keying/identity wrapper, and public re-exports. The private
`composites/property_row/element.rs` owner contains row/column flex assembly, layout-query usage,
resolved-layout consumption, value-slot marking, reset/action slot mounting, and test-id
application. Public constructors/builders, explicit id-source keying, label helper behavior,
layout resolution, auto row/column switching, value-slot overflow semantics, reset/action slots,
test-id propagation, and property-row text role behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor PropertyRow row branch owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_row/element.rs` now keeps layout-query /
resolution, auto dispatch, column branch assembly, and test-id application while delegating row
branch construction to the private `composites/property_row/element/row.rs` owner. Row/column/auto
layout variant resolution, row label fixed slot width, single-line label line box, row value-slot
overflow semantics, reset/action trailing slot wiring, row min-height behavior, test-id
propagation, and public property row APIs remain unchanged. `tools/gate_imui_workstream_source.py`
freezes the split and the value-slot overflow guard now tracks the two marked slots across root
and row owners.

2026-06-01 editor PropertyRow column branch owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_row/element.rs` now keeps layout-query /
resolution, auto dispatch, row/column owner routing, and test-id application while delegating
column branch construction to the private `composites/property_row/element/column.rs` owner.
Row/column/auto layout variant resolution, column header/value stacking, header label line box,
column value-slot overflow semantics, reset/action trailing slot wiring, column stack gap
behavior, test-id propagation, and public property row APIs remain unchanged. The source gate and
value-slot overflow guard now track the two marked slots across row and column owners.

2026-06-01 editor PropertyRow options owner-split result:
`ecosystem/fret-ui-editor/src/composites/property_row.rs` now keeps the public composite, label
helper, keyed row entrypoint, row/column child assembly, value-slot marking, and reset-slot wiring
while re-exporting `PropertyRowOptions`. The private `composites/property_row/options.rs` owner
contains public option fields and defaults. Public import paths, layout defaults, slot-width
defaults, auto-stack identity/test-id fields, row/column assembly, reset slot behavior, value-slot
marking, and property-row text role behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValueCore behavior owner-split result:
`ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs` now keeps public API shape,
slot-state lookup, layout/a11y setup, current-value synchronization, and response construction.
The private `primitives/drag_value_core/behavior.rs` owner contains pressable pointer down/move/up
handler installation, Escape key capture, pointer capture/release calls, scrub delta calculation,
constraint application, and commit/cancel/live callback dispatch. Public constructors/builders,
options import paths, drag threshold crossing, unexpected pointer-stream cleanup, pointer-up
commit, Escape cancel, and response accessor behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValueCore options owner-split result:
`ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs` now keeps the public drag-to-edit
primitive entrypoint, pressable/key handler wiring, and response construction while re-exporting
`DragValueCoreOptions`. The private `primitives/drag_value_core/options.rs` owner contains public
options, defaults, theme-token resolution, finite-value sanitization, and drag-threshold clamping.
Public import paths, defaults, theme token fallback behavior, and `DragValueCore` runtime behavior
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValueCore scrub-state owner-split result:
`ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs` now keeps the public drag-to-edit
primitive entrypoint, pressable/key handler wiring, a11y/layout options, and response construction.
The private `primitives/drag_value_core/state.rs` owner contains scrub session storage,
commit/cancel state mutation, move action classification, and scrub multiplier resolution. Public
`DragValueCore` APIs, response accessors, pointer routing, Escape cancel behavior, live callbacks,
commit/cancel callbacks, modifier multipliers, and numeric constraints remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 docking declarative frame drop-hints owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` no longer owns the drop-hint helper that only
serves frame aggregation. The private `dock/declarative/frame.rs` owner now contains both
`DockSpaceElementFrame` construction and `DockDropHints` projection from hover state. Public docking
APIs, frame output construction, and drop-hint painting remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the helper boundary.

2026-06-01 docking declarative drag-resolve owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, event routing,
paint ordering, and public entrypoint functions while importing private drag-resolve owner helpers.
The private `dock/declarative/drag_resolve.rs` owner contains internal drag hover/drop resolution,
drop-intent effect projection, tab-bar auto-scroll during drag, tear-off handoff, drag diagnostics
publication, drag inversion payload flags, panel/tabs drag allow checks, and cross-window drag
session payload startup. Public docking APIs, drag/drop behavior, and diagnostics payloads remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 docking declarative drag-preview owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, paint ordering,
drop-overlay dispatch, and public entrypoint functions while importing private drag-preview owner
helpers. The private `dock/declarative/drag_preview.rs` owner contains drag ghost snapshot lookup,
drag source tab lookup, ghost title fallback, drag ghost paint preparation, and center-zone tab
insert preview title metadata. Public docking APIs, drag ghost rendering order, and tab insert
preview painting remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 docking declarative floating owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, drag/drop event
routing, layout/render wiring, and public entrypoint functions while importing private floating
owner helpers. The private `dock/declarative/floating.rs` owner contains floating hover lookup,
floating hover paint-state projection, floating chrome paint inputs, close/title-bar hit tests,
leaf-tabs selection for title-bar drags, and floating title-bar drag target resolution. Public
docking APIs and managed dock-space entrypoints remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 docking declarative geometry owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, drag/drop event
routing, layout/render wiring, and public entrypoint functions while importing private geometry
owner helpers. The private `dock/declarative/geometry.rs` owner contains declarative tab hit tests,
layout snapshot lookup, split-handle hit/min-size geometry, split-handle cursor mapping,
pixels-per-point lookup, and active viewport hit-test projection. Public docking APIs and managed
dock-space entrypoints remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-01 docking declarative tab-overflow owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, layout/render
wiring, input event routing, and public entrypoint functions while importing private overflow
owner helpers. The private `dock/declarative/overflow.rs` owner contains tab overflow menu lookup
and opening, active-row scroll positioning, menu click/close effects, menu wheel scrolling,
tab-strip wheel persistence, and tab/overflow hover projection. Public docking APIs and managed
dock-space entrypoints remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-01 docking declarative tear-off owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, drag/drop event
routing, layout/render wiring, and public entrypoint functions while importing the private tear-off
owner helpers. The private `dock/declarative/tear_off.rs` owner contains panel/tab tear-off
eligibility checks, stable out-of-bounds frame tracking, retry clearing, request-float effect
construction, default floating rect sizing, and floating bounds clamping. Public docking APIs and
managed dock-space entrypoints remain unchanged, and `tools/gate_imui_workstream_source.py` freezes
the split.

2026-06-01 docking declarative frame owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, input routing,
layout/render event wiring, and public entrypoint functions while importing the private frame
output owner. The private `dock/declarative/frame.rs` owner contains `DockSpaceElementFrame`, empty
frame construction, layout snapshot projection, cached panel sizes, tab/floating/viewport/split
paint input storage, drag ghost storage, and drop-hint derivation. Managed dock-space entrypoints
and public docking APIs remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-01 docking declarative registry owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, input routing,
layout/render assembly, and public entrypoint functions while re-exporting the registry public
surface through the existing module path. The private `dock/declarative/registry.rs` owner contains
`DockSpaceElementOptions`, `DockPanelElement`, `DockPanelElementRegistry`,
`DockPanelElementRegistryService`, `dock_panel_element`, panel collection/order, missing-panel
fallback UI, and panel-node binding helpers. Public re-export paths and managed dock-space
entrypoints remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 docking declarative tab-metrics owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps dock-space orchestration, hit testing,
input routing, paint input assembly, and public entrypoints. The private
`dock/declarative/tab_metrics.rs` owner contains tab title/glyph text preparation, measured and
fallback tab width projection, tab-bar geometry, active-tab visibility clamping, persisted tab
scroll sync, tab detail paint preparation, and drag auto-scroll insert-index updates. Public
dock-space APIs and tab/overflow behavior remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 docking declarative interaction-state owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps the managed-surface dock-space
entrypoint, panel registry, layout/render/input orchestration, and public docking APIs. The private
`dock/declarative/interaction.rs` owner contains declarative pressed close, floating/divider/panel
drag, viewport capture, tab-overflow menu, tab scroll/width, tab-hover, and floating-hover state
records plus the `DeclarativeDockInteractionService` helpers. Managed dock-space behavior and
cross-window docking call paths remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-03 docking declarative interaction drag-session owner-split result:
`ecosystem/fret-docking/src/dock/declarative/interaction.rs` now keeps the interaction service
state fields plus pressed-close, tab-overflow, tab-scroll/width, auto-scroll gate, tab-hover, and
floating-hover helpers. `ecosystem/fret-docking/src/dock/declarative/interaction/drag_sessions.rs`
owns floating drag, divider drag, pending panel/tabs drag, and viewport-capture session map
mutation/query/take helpers with visibility limited to `crate::dock::declarative`. Public
dock-space APIs and sibling `events.rs` call paths remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 desktop runner window request dispatch owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `Effect::Window(req)` to
`crates/fret-launch/src/runner/desktop/runner/window_requests.rs`.
`window_requests.rs` owns `handle_window_request_effect`, including close/create dispatch,
DockFloating create trace logging, docking post-create handling, driver `window_created` callback
ordering, request-redraw behavior, and delegation to close/geometry/style owners. Runtime behavior
and public window request surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

2026-06-04 desktop runner window metrics owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `Effect::WindowMetricsSetInsets` and `Effect::WindowMetricsSetPreferences` to
`crates/fret-launch/src/runner/desktop/runner/window_metrics.rs`. `window_metrics.rs` owns
diagnostic insets/preference override storage, `WindowMetricsService` known-state comparisons,
safe-area/occlusion/color-scheme/reduced-motion/text-scale service mutation, redraw requests, and
RAF requests. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

2026-06-04 desktop runner clipboard effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates clipboard diagnostics, clipboard write/read, and primary-selection effect branches to
`crates/fret-launch/src/runner/desktop/runner/clipboard_effects.rs`. `clipboard_effects.rs` owns
diagnostics-forced clipboard unavailable state, clipboard completion events, primary selection
capability gating, primary selection unavailable events, and platform clipboard error logging.
The source gate explicitly tracks primary selection capability gating.
Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

2026-06-04 desktop runner incoming-open effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates diagnostic incoming-open injection, incoming-open reads, limit-capped reads, and release
cleanup to `crates/fret-launch/src/runner/desktop/runner/incoming_open_effects.rs`.
`incoming_open_effects.rs` owns diagnostic payload projection, startup path payload reads,
incoming-open capability gating, unavailable-event delivery, and release cleanup. Runtime behavior
and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

2026-06-04 desktop runner file-transfer effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates external-drop read completion, file-dialog open selection/cancel, read-limit capped reads, and
release cleanup to `crates/fret-launch/src/runner/desktop/runner/file_transfer_effects.rs`.
`file_transfer_effects.rs` owns external-drop provider calls, file-dialog capability gating, native
path reads, and platform completion dispatch. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

2026-06-04 desktop runner shell effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates shell action handling, including macOS about-panel and app-hide/unhide actions,
open-url capability gating, and
share-sheet unavailable completion to `crates/fret-launch/src/runner/desktop/runner/shell_effects.rs`.
`shell_effects.rs` owns the platform menu calls, shell capability checks, native open-url dispatch,
and share-sheet completion dispatch. Runtime behavior and public effect surfaces remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

2026-06-04 desktop runner image effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates image registration, streaming RGBA8/NV12/I420 updates, and image unregister handling to
`crates/fret-launch/src/runner/desktop/runner/image_effects.rs`.
`image_effects.rs` owns image upload validation, streaming image update dispatch, YUV fallback
conversion, and redraw-on-register/unregister behavior. Runtime behavior and public effect surfaces
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

2026-06-04 desktop runner text effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates font asset injection and system-font rescan handling to
`crates/fret-launch/src/runner/desktop/runner/text_effects.rs`.
`text_effects.rs` owns `TextAddFontAssets` dispatch, `TextRescanSystemFonts` dispatch, asset request
injection, and redraw or rescan follow-up behavior. Runtime behavior and public effect surfaces
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

2026-06-04 desktop runner system-font rescan owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
drain-turn trigger while `crates/fret-launch/src/runner/desktop/runner/text_effects.rs` owns
system-font rescan async gating, startup async gating, state publication, sync/async request
handling, completed-result application, resize deferral, redraw follow-up, and pending restart behavior.
Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

2026-06-04 desktop runner IME effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates IME allow/disable, virtual-keyboard request, and cursor-area handling to
`crates/fret-launch/src/runner/desktop/runner/ime_effects.rs`. `ime_effects.rs` owns Android
soft-input forwarding, `FRET_IME_DEBUG` cursor-area logging, platform cursor-area publication, and
dirty-window propagation. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

2026-06-04 desktop runner frame effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates redraw, request-animation-frame, and diagnostic event-injection handling to
`crates/fret-launch/src/runner/desktop/runner/frame_effects.rs`. `frame_effects.rs` owns
effect-redraw RAF fallback scheduling, `EffectRequestAnimationFrame` reason recording,
injected-event scope handling, and post-injection redraw/RAF scheduling. Runtime behavior and
public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

2026-06-04 desktop runner timer effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `SetTimer` and `CancelTimer` handling to the existing
`crates/fret-launch/src/runner/desktop/runner/timers.rs` owner. `timers.rs` now owns timer effect
set/cancel dispatch next to timer firing, repeating timer re-arm, and fired-timer removal behavior.
Runtime behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

2026-06-04 desktop runner cursor effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates cursor icon application to `crates/fret-launch/src/runner/desktop/runner/cursor_effects.rs`.
`cursor_effects.rs` owns platform cursor icon application and dirty-window propagation. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

2026-06-04 desktop runner quit-app effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `Effect::QuitApp` handling to `crates/fret-launch/src/runner/desktop/runner/quit_effects.rs`.
`quit_effects.rs` owns the before-close prompt gate, dev-state geometry flush, force-close-all-windows
behavior, dispatcher shutdown, and event-loop exit. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

2026-06-04 desktop runner command effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `Effect::Command` handling to `crates/fret-launch/src/runner/desktop/runner/command_effects.rs`.
`command_effects.rs` owns window/global command context assembly, UI services selection, and driver
callback routing. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source gate.

2026-06-04 desktop runner menu effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `Effect::SetMenuBar` handling to `crates/fret-launch/src/runner/desktop/runner/menu_effects.rs`.
`menu_effects.rs` owns menu-bar caching, Windows per-window menu installation, macOS app-menu
installation, and unsupported-target no-op consumption. Runtime behavior and public effect surfaces
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

2026-06-04 desktop runner change propagation owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
calls `propagate_model_changes` / `propagate_global_changes` from
`crates/fret-launch/src/runner/desktop/runner/change_propagation.rs`. `change_propagation.rs` owns
model/global driver fan-out, platform menu keymap and command-gating sync, renderer font-family
sync, and renderer locale sync. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source gate.

2026-06-04 desktop runner driver effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
delegates `Effect::ViewportInput` / `Effect::Dock` handling to
`crates/fret-launch/src/runner/desktop/runner/driver_effects.rs`. `driver_effects.rs` owns
viewport-input driver forwarding, DockOp driver forwarding, and the existing DockFloating tear-off
log for `DockOp::RequestFloatPanelToNewWindow`. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

2026-06-04 desktop runner streaming effects owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the generic effect loop and
image update effect dispatch while delegating streaming upload preprocessing, dropped-update ack
delivery, perf snapshot/debug publication, and pending streaming redraw wakeups to
`crates/fret-launch/src/runner/desktop/runner/streaming_effects.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

2026-06-04 desktop runner effect queue owner-split result:
`crates/fret-launch/src/runner/desktop/runner/effects.rs` now keeps the fixed-point drain loop and
post-dispatch lifecycle work while delegating ordered queued effect dispatch to
`crates/fret-launch/src/runner/desktop/runner/effect_queue.rs`. Runtime behavior, effect ordering,
streaming stats mutation, dirty-window tracking, and early-exit signaling remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

2026-06-04 desktop runner wheel coalescing owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing and
`ApplicationHandler` wiring while delegating wheel coalescing math, per-axis max-abs splitting,
carried remainder behavior, and wheel-coalescing env configuration to
`crates/fret-launch/src/runner/desktop/runner/wheel_coalescing.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

2026-06-04 desktop runner redraw hitch owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing,
redraw execution, renderer frame construction, surface recovery, and `ApplicationHandler` wiring
while delegating hitch configuration, log path selection, buffered writes, logical pixel
quantization, phase tracing spans, and elapsed timing to
`crates/fret-launch/src/runner/desktop/runner/redraw_hitch.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

2026-06-04 desktop runner monitor topology owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing,
surface lifecycle entry points, redraw execution, and `ApplicationHandler` wiring while delegating
monitor enumeration, scale-factor fallback, stable monitor sorting, virtual desktop bounds
construction, and topology diagnostics publication to
`crates/fret-launch/src/runner/desktop/runner/monitor_topology.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

2026-06-04 desktop runner surface lifecycle owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing,
the `can_create_surfaces`/`destroy_surfaces` lifecycle hooks, redraw execution, and
`ApplicationHandler` wiring while delegating screenshot surface usage selection, deferred
missing-surface creation, composited-alpha surface configuration, surface bootstrap redraw/RAF
scheduling, and surface destroy cleanup to
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

2026-06-04 desktop runner wgpu adapter diagnostics owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing,
context construction, renderer construction, startup font initialization, and
`ApplicationHandler` wiring while delegating `FRET_WGPU_BACKEND` request logging, adapter identity
logging, downlevel capability warnings, init-attempt logging, and
`WgpuAdapterSelectionSnapshot` publication to
`crates/fret-launch/src/runner/desktop/runner/wgpu_adapter_diagnostics.rs`. Runtime behavior and
public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

2026-06-04 desktop runner renderer bootstrap owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing,
wgpu context construction, main-window insertion, factory surface attachment, driver
initialization, and `ApplicationHandler` wiring while delegating renderer creation, renderer
capability publication, budget configuration, startup font environment initialization,
context/renderer installation, `gpu_ready` ordering, and startup async font rescan gating to
`crates/fret-launch/src/runner/desktop/runner/renderer_bootstrap.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

2026-06-04 desktop runner factory surface attach owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps winit event routing,
mobile context construction, renderer bootstrap, driver initialization, and `ApplicationHandler`
wiring while delegating Android/iOS factory-provided main surface attachment, `SurfaceState`
construction, screenshot usage selection, composited-alpha configuration, missing context/state skip
behavior, and failed factory surface early return to
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

2026-06-04 desktop runner device event owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::device_event` trait hook while delegating cross-window pointer/device routing,
pointer-motion cursor tracking, diagnostics pointer input isolation, dock drag follow updates,
pointer-capture sync, released-outside fallback drop routing, reliable window-under-cursor skip
behavior, cached mouse-button cleanup, and DockFloating follow stop behavior to
`crates/fret-launch/src/runner/desktop/runner/device_events.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

2026-06-04 desktop runner proxy wake owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::proxy_wake_up` trait hook while delegating queued proxy event dispatch, proxy
queue drain, platform completion delivery, asset reload wake handling, Windows/macOS menu command
forwarding, macOS menu gating refresh, macOS hit-test refresh forwarding, and final effect draining
to `crates/fret-launch/src/runner/desktop/runner/event_loop.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

Marker summary: proxy queue drain.

2026-06-04 desktop runner surface lifecycle hook owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::destroy_surfaces`, `ApplicationHandler::resumed`, and
`ApplicationHandler::suspended` trait hooks while delegating destroy-surface diagnostics, surface
destroy cleanup, Android/iOS resume redraw requests, Android/iOS resume effect draining,
Android/iOS suspend state updates, Android/iOS best-effort surface drop, and suspend control-flow
wait to `crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs`. Runtime behavior and
public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

Marker summary: destroy-surface diagnostics; suspend control-flow wait.

2026-06-04 desktop runner about-to-wait control-flow owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating pending-front request processing,
timer deadline merging, dispatcher deadline merging, pending-front deadline merging, hotpatch
deadline merging, dock drag/follow polling pressure, RAF deadline scheduling, RAF flush behavior,
and final `ControlFlow::Poll` / `ControlFlow::WaitUntil` / `ControlFlow::Wait` selection to
`crates/fret-launch/src/runner/desktop/runner/event_loop.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

Marker summary: pending-front request processing; RAF flush behavior; final ControlFlow selection.

2026-06-04 desktop runner about-to-wait internal drag poll owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating scripted cursor screen position
override polling, event-loop turn polling, internal drag hover routing, diagnostic mouse-button
override polling, release-to-drop routing, `saw_left_mouse_release_this_turn` updates, dock drag
follow updates, and conditional effect draining to
`crates/fret-launch/src/runner/desktop/runner/device_events.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

Marker summary: internal drag hover routing; conditional effect draining.

2026-06-04 desktop runner about-to-wait DockFloating follow-stop owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating no-pointer-motion follow stop,
canceled drag session detection, source-window left-button fallback detection, `Instant::now()`
stop timing, non-raising stop semantics, and the pre-drain idle follow-stop check to
`crates/fret-launch/src/runner/desktop/runner/docking/follow.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

Marker summary: no-pointer-motion follow stop; non-raising stop semantics.

2026-06-04 desktop runner about-to-wait DockFloating released-outside fallback owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating macOS released-outside polling,
Windows poll-up routing, diagnostics pointer-input isolation behavior, cursor-based drop routing,
follow cleanup after Windows poll-up, and fallback-triggered effect draining to
`crates/fret-launch/src/runner/desktop/runner/docking/poll_up.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

Marker summary: released-outside fallback scheduling; platform poll-up drain.

2026-06-04 desktop runner about-to-wait turn bookkeeping owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating tick-id advancement, app tick
publication, per-turn left-release reset, turn timestamp sampling, window environment polling, and
dev-state timestamp reuse to `crates/fret-launch/src/runner/desktop/runner/event_loop.rs`. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: tick-id advancement; environment poll; dev-state timestamp reuse.

2026-06-04 desktop runner about-to-wait window turn accessibility owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating iOS keyboard bootstrap, Android/iOS
platform inset projection, diagnostic inset overrides, accessibility activation diagnostics, and
accessibility action draining to `crates/fret-launch/src/runner/desktop/runner/window_turn.rs`.
Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: window-turn platform insets; accessibility action drain; activation diagnostics.

2026-06-04 desktop runner about-to-wait mobile surface recreation owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating Android/iOS `can_create_surfaces`
gating, context-present checks, missing-surface scans, `try_create_missing_surfaces`, and
post-bootstrap effect draining to
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs`. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

Marker summary: mobile surface recreation; can-create-surfaces lifecycle gate; post-bootstrap effect drain.

2026-06-04 desktop runner about-to-wait diag screenshot poll owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating feature-gated screenshot request
polling, pending-window collection by `AppWindowId` FFI key, `EffectRedraw` request publication, and
RAF wake requests to `crates/fret-launch/src/runner/desktop/runner/diag_screenshots.rs`. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: diag screenshot polling; pending screenshot windows; EffectRedraw request; RAF wake.

2026-06-04 desktop runner about-to-wait dev-state observation owner-split result:
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` now keeps the
`ApplicationHandler::about_to_wait` trait hook while delegating desktop-only dev-state
alive-window filtering, app export, window-key snapshot iteration, logical-size projection,
outer-position sampling, and turn timestamp reuse to
`crates/fret-launch/src/runner/desktop/runner/dev_state.rs`. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

Marker summary: dev-state window observation; alive-window filtering; app-export ordering; turn timestamp reuse.

2026-06-04 desktop runner about-to-wait preamble owner-split result:
`crates/fret-launch/src/runner/desktop/runner/event_loop.rs` now keeps the pre-render effect drain,
suspended wait fast path, and monitor topology refresh while
`crates/fret-launch/src/runner/desktop/runner/app_handler.rs` delegates the preamble before the
later about-to-wait diagnostics, surface recreation, drag polling, turn bookkeeping, dev-state
observation, and dock follow/fallback hooks. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: pre-render drain; suspended wait fast path; monitor topology refresh.

2026-06-04 desktop runner monitor geometry owner-split result:
`crates/fret-launch/src/runner/desktop/runner/monitor_topology.rs` now owns the monitor rect types,
virtual desktop bounds lookup, physical monitor rect collection, target monitor selection,
visibility clamping, Windows work-area preference, mixed-DPI scale lookup, and DockFloating
outer-position settling while `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps
window/client coordinate conversion, cursor-grab placement, z-order heuristics, and window helper
tests. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: monitor geometry helpers; virtual desktop bounds; outer-position settle.

2026-06-04 desktop runner window position owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_position.rs` now owns client/screen coordinate
conversion, client-origin diagnostics, local-position projection, cursor-grab decoration math,
mixed-DPI cursor-grab estimates, window anchor/cursor placement, client-rect hit checks, and
DockFloating cursor-grab outer-position helpers while
`crates/fret-launch/src/runner/desktop/runner/window.rs` keeps platform focus/style, platform
under-cursor lookup, heuristic z-order fallback, and window runtime state. Runtime behavior and
public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

Marker summary: client/screen coordinate helpers; cursor-grab placement; client-origin diagnostics.

2026-06-04 desktop runner window under-cursor owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_under_cursor.rs` now owns platform
under-cursor lookup, macOS ordered-window matching, Windows root-HWND lookup and z-order walk
fallback, heuristic rect fallback, preferred-window exclusion, and z-order bump bookkeeping used by
DockFloating drag target identification. `crates/fret-launch/src/runner/desktop/runner/window.rs`
keeps window runtime state; the follow-up M110 split moves the remaining platform operation helpers
out too. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: platform under-cursor lookup; z-order fallback; DockFloating target identification.

2026-06-04 desktop runner window platform owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_platform.rs` now owns platform raise/focus,
Windows foreground raising, macOS ordered-front logging, opacity application, hit-test passthrough,
region hit-test fallback, background material application, and non-macOS/non-Windows fallback
behavior. `crates/fret-launch/src/runner/desktop/runner/window.rs` now keeps `WindowRuntime`,
`PendingWheelEvent`, `TimerEntry`, and `DockTearoffFollow` only after the follow-up M111
pending-front split. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: platform window operations; raise/focus; style material helpers.

2026-06-04 desktop runner window front owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_front.rs` now owns the pending-front retry
queue, `PendingFrontRequest`, `enqueue_window_front`, `process_pending_front_requests`, and
`next_pending_front_deadline`. `crates/fret-launch/src/runner/desktop/runner/window.rs` keeps
runtime state records and `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps
create/insert/destroy lifecycle helpers without pending-front retry behavior. Runtime behavior and
public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

Marker summary: pending-front retry queue; about-to-wait scheduling; DockFloating fronting.

2026-06-04 desktop runner surface alpha owner-split result:
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` now owns
`want_surface_composited_alpha_for_style` and
`configure_surface_alpha_mode_for_composited_window`, including background material implied
transparency, alpha-mode selection order, and surface reconfigure behavior. `window_lifecycle.rs`
keeps window create/insert/destroy lifecycle helpers and delegates initial surface alpha setup to
the surface owner; `window_style.rs` calls the same surface owner for style updates. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: composited-alpha surface configuration; background material implied transparency;
surface reconfigure.

2026-06-04 desktop runner window close teardown owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_close.rs` now owns `close_window`,
`force_close_window`, and `close_window_impl`, including `before_close_window` checks, dev-state
flushes, DockFloating follow cancellation, drag cleanup, webview close cleanup, window registry
removal, diagnostics cleanup, per-window service cleanup, metrics cleanup, and main-window clearing.
`crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps window create/insert
helpers without close-window teardown. Runtime behavior and public effect surfaces remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: close-window teardown; checked close; drag cleanup; diagnostics cleanup.

2026-06-04 desktop runner window insert owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_insert.rs` now owns `insert_window`,
`WindowRuntime` construction, optional surface setup, metrics bootstrap, surface config diagnostics,
environment refresh, window registry insertion, z-order bootstrap, menu registration, lifecycle
diagnostics, and initial redraw/RAF bootstrap. `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
keeps OS window creation and create-request orchestration without insertion/bootstrap behavior.
Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: window insertion bootstrap; metrics bootstrap; redraw bootstrap.

2026-06-04 desktop runner OS window create owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_os_create.rs` now owns `create_os_window`,
winit `WindowAttributes`, OS window creation, creation-time style attributes, Windows taskbar
creation attributes, macOS parent-window creation attributes, accessibility bootstrap, z-level
setup, background material setup, hit-test setup, and opacity setup.
`crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs` keeps create-request
orchestration without OS window creation. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: OS window creation; create-time style attributes; accessibility bootstrap.

2026-06-04 desktop runner window create-request owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_create_request.rs` now owns
`create_window_from_request`, driver/default spec resolution, dev-state spec projection,
DockFloating cursor/anchor placement selection, macOS hidden-create policy, macOS parent handle
selection, OS window creation delegation, WGPU surface creation, insertion delegation, open-style
diagnostics, dev-state key registration, and monitor topology refresh. The current source no
longer has a `window_lifecycle.rs` module owner. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: create-request orchestration; dev-state spec projection; no lifecycle owner.

2026-06-04 desktop runner window external drag owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_external_drag.rs` now owns
`handle_window_drag_entered`, `handle_window_drag_moved`, `handle_window_drag_dropped`, and
`handle_window_drag_left`, including external drag token allocation/reuse, path-cache updates,
physical-to-logical position mapping, Enter/Over/Drop/Leave external drag event construction,
payload-path publication, token release, and effect draining. `app_handler.rs` keeps winit event
dispatch and the existing pointer-move merge path. Runtime behavior and public effect surfaces
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: external file drag state machine; token/path cache; app-handler dispatch only.

2026-06-04 desktop runner window surface resize owner-split result:
`crates/fret-launch/src/runner/desktop/runner/surface_lifecycle.rs` now owns
`handle_window_surface_resized`, including immediate surface resize synchronization, macOS active
hit-test refresh, surface resize redraw requesting, and effect draining. `app_handler.rs` keeps the
`WindowEvent::SurfaceResized` dispatch and the existing redraw-time `pending_surface_resize`
fallback. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: surface resize event owner; immediate resize sync; app-handler dispatch only.

2026-06-04 desktop runner window pointer move owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_pointer_move.rs` now owns
`handle_window_pointer_moved`, including pointer-move platform event mapping, non-touch
screen-position sampling, macOS cursor transform calibration, DockFloating follow updates, external
drag over-event delivery, cross-window dock-drag move rerouting, dock-drag pointer-capture
synchronization, internal drag hover routing, and effect draining. `app_handler.rs` keeps only
`WindowEvent::PointerMoved` dispatch. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: pointer move event owner; dock drag move reroute; app-handler dispatch only.

2026-06-04 desktop runner window pointer button owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_pointer_button.rs` now owns
`handle_window_pointer_button`, including pointer button platform event mapping, cursor
screen-position fallback, macOS cursor transform calibration, dock-drag pointer-capture
synchronization, left mouse down/up tracking, cursor-based internal drag drop delivery,
DockFloating follow stop on left release, cross-window drag cancellation, dock-source Up/Down
rerouting, mapped event delivery, and effect draining. `app_handler.rs` keeps only
`WindowEvent::PointerButton` dispatch. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: pointer button event owner; left release drag cleanup; app-handler dispatch only.

2026-06-04 desktop runner window state events owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_state_events.rs` now owns
`handle_window_modifiers_changed`, `handle_window_theme_changed`, and
`handle_window_focus_changed`, including modifier platform event mapping, internal drag hover
rerouting, theme/environment refresh, redraw requesting, focus state updates, pressed-button reset
on focus loss, focus z-order bump, `Event::WindowFocusChanged` delivery, and macOS focus logging.
`app_handler.rs` keeps only `WindowEvent::ModifiersChanged`, `WindowEvent::ThemeChanged`, and
`WindowEvent::Focused` dispatch. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: window state event owner; focus/environment refresh; app-handler dispatch only.

2026-06-04 desktop runner window mapped events owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_mapped_events.rs` now owns
`handle_window_mapped_event`, including catchall platform event mapping, wheel-event coalescing
into `WindowRuntime::pending_wheel`, redraw requesting after coalesced wheel input, RenderDoc F12
capture requests, Escape cancellation for active cross-window dock drags, mapped event delivery,
and effect draining. `app_handler.rs` keeps only catchall dispatch plus the existing redraw-time
pending wheel drain. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: mapped window event owner; wheel coalescing catchall; app-handler dispatch only.

2026-06-04 desktop runner window moved events owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_moved_events.rs` now owns
`handle_window_moved`, including the macOS hit-test active-region check and latest mouse-location
refresh used for the `macos-hit-test-regions` guarded `WindowEvent::Moved(..)` path.
`app_handler.rs` keeps only cfg-gated moved-event dispatch. Runtime behavior and public effect
surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the
docking multiwindow source gate.

Marker summary: moved window event owner; macOS hit-test refresh; app-handler dispatch only.

2026-06-04 desktop runner window pre-dispatch events owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_pre_dispatch_events.rs` now owns
`handle_window_pre_dispatch_event`, including raw winit event delivery into the accessibility
backend and `FRET_IME_DEBUG` winit IME cached cursor-area logging. `app_handler.rs` keeps only the
pre-dispatch call before `WindowEvent` matching. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: window pre-dispatch event owner; accessibility event feed; IME debug logging;
app-handler dispatch only.

2026-06-04 desktop runner window redraw accessibility owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_accessibility.rs` now owns
`update_window_redraw_accessibility_snapshot`, including active accessibility checks, driver
semantics snapshot requests, AccessKit tree update construction, active update dispatch, and
`last_semantics_snapshot` cache maintenance. `app_handler.rs` keeps only redraw-time accessibility
dispatch after scene validation and before engine-frame recording. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

Marker summary: redraw accessibility owner; semantics snapshot cache; app-handler dispatch only.

2026-06-04 desktop runner window redraw text-input owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_text_input.rs` now owns
`apply_window_redraw_text_input_snapshot`, including `WindowTextInputSnapshotService` lookup, IME
allowed-state sync, Android soft-input forwarding, cursor-area sync, surrounding-text sync,
`FRET_IME_DEBUG` snapshot logging, and follow-up `prepare_frame`. `app_handler.rs` keeps only
cfg-gated redraw-time text-input snapshot dispatch after render and before scene validation.
Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw text-input owner; IME snapshot application; Android soft-input forwarding;
app-handler dispatch only.

2026-06-04 desktop runner window redraw renderer perf owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_renderer_perf.rs` now owns
`maybe_publish_window_redraw_renderer_perf_sample`, including `FRET_DIAG_RENDERER_PERF`,
`take_last_frame_perf_snapshot`, `RendererPerfFrameSample`, `RendererPerfFrameStore` recording, and
`driver.renderer_perf_sample`. `app_handler.rs` keeps only redraw-time renderer perf dispatch after
text diagnostics and before WGPU diagnostics. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: redraw renderer perf owner; renderer perf sample publication; app-handler dispatch
only.

2026-06-04 desktop runner window redraw WGPU hub report owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_wgpu_report.rs` now owns
`maybe_record_window_redraw_wgpu_hub_report`, including `FRET_DIAG_WGPU_REPORT`, cadence parsing,
`context.instance.generate_report`, hub count projection, and `WgpuHubReportFrameStore` recording.
`app_handler.rs` keeps only redraw-time WGPU hub report dispatch after renderer perf diagnostics
and before allocator diagnostics. Allocator report publication remains in `app_handler.rs` for a
separate owner split. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw WGPU hub report owner; hub report count publication; app-handler dispatch
only.

2026-06-04 desktop runner window redraw WGPU allocator report owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_wgpu_allocator_report.rs` now owns
`maybe_record_window_redraw_wgpu_allocator_report`, including
`FRET_DIAG_WGPU_ALLOCATOR_REPORT`, cadence parsing, top-N/max-name-byte configuration,
`context.device.generate_allocator_report`, macOS Metal allocated-size sampling, and
`WgpuAllocatorReportFrameStore` recording. `app_handler.rs` keeps only redraw-time WGPU allocator
report dispatch after hub report diagnostics and before command-buffer submission. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: redraw WGPU allocator report owner; allocator report sample publication;
app-handler dispatch only.

2026-06-04 desktop runner window redraw text diagnostics owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_text_diagnostics.rs` now owns
`WindowRedrawTextDiagnosticsMode`, `window_redraw_text_diagnostics_mode_from_env`,
`begin_window_redraw_text_diagnostics_frame`, and `publish_window_redraw_text_diagnostics`,
including `FRET_RENDER_TEXT_DEBUG`, `FRET_DIAG_DIR`, `begin_text_diagnostics_frame`, SVG text
bridge diagnostics publication, renderer text diagnostics snapshots, and the debug vs untracked
global-write policy. `app_handler.rs` keeps only redraw-time text diagnostics mode creation plus
begin/publish dispatch around render and renderer perf diagnostics. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

Marker summary: redraw text diagnostics owner; renderer text diagnostics publication;
app-handler dispatch only.

2026-06-04 desktop runner window redraw diag screenshots owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_diag_screenshots.rs` now owns
`poll_window_redraw_diag_screenshot_requests`,
`begin_window_redraw_diag_screenshot_capture`,
`finish_window_redraw_diag_screenshot_capture`,
`begin_window_redraw_bundle_screenshot_readback`, and
`finish_window_redraw_bundle_screenshot_readback`, including feature-gated diagnostic screenshot
capture polling, per-window capture begin/finish, bundle screenshot readback begin/finish, and
capture failure logging. `app_handler.rs` keeps only redraw-time dispatch plus
`context.queue.submit`, `frame.present`, and frame-id commit ordering. Runtime behavior and public
effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split
through the docking multiwindow source gate.

Marker summary: redraw diag screenshots owner; screenshot capture/readback lifecycle;
app-handler submit/present orchestration only.

2026-06-04 desktop runner window redraw pending-wheel owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_pending_wheel.rs` now owns
`handle_window_redraw_pending_wheel`, including diagnostic wheel burst injection, pending-wheel
coalescing, frame-boundary max-abs splitting, remainder carry-over redraw requests, and final wheel
event delivery. `app_handler.rs` keeps only redraw-time pending-wheel dispatch before
window-environment refresh. `wheel_coalescing.rs` continues to own math/env configuration, and
`window_mapped_events.rs` continues to own catchall mapped wheel accumulation into
`pending_wheel`. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw pending wheel owner; frame-boundary wheel drain; app-handler dispatch only.

2026-06-04 desktop runner window redraw surface-resize owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_surface_resize.rs` now owns
`handle_window_redraw_pending_surface_resize`, including `pending_surface_resize` draining,
fallback `resize_surface`, logical-size quantization, `last_delivered_window_resized`
deduplication, `Event::WindowResized`, and `Event::WindowScaleFactorChanged` delivery.
`app_handler.rs` keeps only redraw-time surface-resize dispatch before platform frame preparation.
`surface_lifecycle.rs` continues to own immediate surface resize event handling and GPU surface
synchronization. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw surface resize owner; pending surface resize fallback; app-handler dispatch
only.

2026-06-04 desktop runner window redraw frame-prepare owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_frame_prepare.rs` now owns
`prepare_window_redraw_frame`, including platform `prepare_frame`, scale-factor capture,
surface-size-to-bounds projection, logical-size quantization, and driver `gpu_frame_prepare`
dispatch. `app_handler.rs` keeps only redraw-time frame-prepare dispatch before render and
continues to own render/record/present orchestration. Runtime behavior and public effect surfaces
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: redraw frame prepare owner; platform frame preparation; app-handler prepare
dispatch only.

2026-06-04 desktop runner window redraw render owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_render.rs` now owns
`render_window_redraw_frame`, including `RedrawPhase::Render`, renderer text diagnostics frame
begin, `WinitRenderContext`, and app `driver.render(...)` dispatch. `app_handler.rs` keeps only
redraw-time render owner dispatch before text-input/accessibility/record/present orchestration and
continues to publish text diagnostics in present. Runtime behavior and public effect surfaces
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: redraw render owner; app render dispatch; app-handler render dispatch only.

2026-06-04 desktop runner window redraw record owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_record.rs` now owns
`record_window_redraw_frame`, including `RedrawPhase::Record`, scene-op count measurement, and
`driver.record_engine_frame(...)` dispatch. `app_handler.rs` keeps only redraw-time record owner
dispatch before webview sync/render-target updates/present orchestration and continues to
destructure `EngineFrameUpdate` locally. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: redraw record owner; engine frame recording; app-handler record dispatch only.

2026-06-04 desktop runner window redraw target-updates owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_target_updates.rs` now owns
`apply_window_redraw_target_updates`, including `RenderTargetUpdate::Update`,
`RenderTargetUpdate::Unregister`, `renderer.update_render_target(...)`, and
`renderer.unregister_render_target(...)`. `app_handler.rs` keeps only redraw-time target-updates
owner dispatch after webview sync and continues to own present/render-scene orchestration. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: redraw target updates owner; render-target update application; app-handler target
updates dispatch.

2026-06-04 desktop runner window redraw present-target owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_target.rs` now owns
`acquire_window_redraw_present_frame`, `WindowRedrawPresentTargetInput`,
`prepare_window_redraw_present_target`, `WindowRedrawPresentTarget`,
`FRET_DIAG_RENDERER_PERF` fallback admission for `SurfaceAcquireError::Other`, fallback
texture/view preparation, and target-view selection. `app_handler.rs` keeps clear-color resolution,
render-scene recording, diagnostics publication, command submission, `frame.present()`, surface
recovery, and hitch summary orchestration. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: redraw present target owner; surface frame acquire; renderer-perf fallback target;
app-handler present target dispatch.

2026-06-04 desktop runner window redraw render-scene owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_render_scene.rs` now owns
`record_window_redraw_render_scene`, including `WindowRedrawRenderSceneInput`,
`RedrawPhase::RenderScene`, `RenderSceneParams`, `renderer.render_scene(...)`, surface format/size
reads, target-view routing, clear color, scale factor, and UI command buffer return semantics.
`app_handler.rs` keeps only render-scene owner dispatch and continues to own text diagnostics,
renderer perf/wgpu reports, screenshots, command submission, `frame.present()`, surface recovery,
and hitch summary orchestration. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw render scene owner; UI command buffer recording; app-handler render-scene
dispatch.

2026-06-04 desktop runner window redraw present-submit owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_submit.rs` now owns
`submit_window_redraw_present_frame`, including `WindowRedrawPresentSubmitInput`,
`input.context.queue.submit(input.command_buffers)`, `WindowRedrawPresentTarget` consumption, and
`frame.present()`. `app_handler.rs` keeps only present-submit owner dispatch and continues to own
text diagnostics, renderer perf/wgpu reports, screenshots, scheduling presented-frame commit,
engine keepalive drop, surface recovery, and hitch summary orchestration. Runtime behavior and
public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split through the docking multiwindow source gate.

Marker summary: redraw present submit owner; queue submit; surface frame present; app-handler
present-submit dispatch.

2026-06-04 desktop runner window redraw present-finish owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_finish.rs` now owns
`finish_window_redraw_present_frame`, including `WindowRedrawPresentFinishInput`,
`commit_presented_frame_for_window`, `drop(input.keepalive)`,
`finish_window_redraw_diag_screenshot_capture`, and
`finish_window_redraw_bundle_screenshot_readback`. `app_handler.rs` keeps only present-finish owner
dispatch and continues to own present-target acquisition, render-scene recording, diagnostics
publication, present-submit dispatch, surface recovery, and hitch summary orchestration. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: redraw present finish owner; frame-id commit; engine keepalive release; diagnostic
screenshot finish; app-handler present-finish dispatch.

2026-06-04 desktop runner window redraw present-error owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_error.rs` now owns
`handle_window_redraw_present_error`, including
`clear_window_surface_after_present_acquire_failure`, `RenderError::SurfaceAcquireFailed`,
`RunnerFrameDriveReason::SurfaceRecoverLost`,
`RunnerFrameDriveReason::SurfaceRecoverOutdated`,
`RunnerFrameDriveReason::SurfaceRecoverTimeout`, `self.raf_windows.request(app_window)`,
`self.dispatcher.shutdown()`, `event_loop.exit()`, and `error!(?err, "render error")`.
`app_handler.rs` keeps only present-error owner dispatch after renderdoc capture end and before
hitch summary orchestration. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw present error owner; surface acquire recovery; timeout redraw retry;
out-of-memory exit; app-handler present-error dispatch.

2026-06-04 desktop runner window redraw hitch-summary owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_hitch_summary.rs` now owns
`maybe_write_window_redraw_hitch_summary`, including `WindowRedrawHitchSummaryInput`,
`RedrawHitchConfig`, total redraw elapsed calculation, threshold comparison, the existing
`redraw hitch window=...` line shape, and `write_redraw_hitch_log(&format!(...))`.
`app_handler.rs` keeps phase timing capture, present-error owner dispatch, and hitch-summary owner
dispatch. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw hitch summary owner; total redraw elapsed; hitch threshold check;
app-handler hitch-summary dispatch.

2026-06-04 desktop runner window redraw RenderDoc capture owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_renderdoc_capture.rs` now owns
`begin_window_redraw_renderdoc_capture` and `end_window_redraw_renderdoc_capture`, including
`Option<&mut RenderDocCapture>`, `begin_capture_if_requested`, the `capturing` boolean, and
`end_capture`. `app_handler.rs` keeps only RenderDoc capture begin/end owner dispatch around redraw
frame work. RenderDoc initialization stays in `render.rs`; capture request hotkey handling stays in
`window_mapped_events.rs`. Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw renderdoc capture owner; capture begin; capture end; app-handler
renderdoc-capture dispatch.

2026-06-04 desktop runner window redraw clear-color owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_clear_color.rs` now owns
`resolve_window_redraw_clear_color`, including the `RunnerWindowStyleDiagnosticsStore` lookup,
`effective_snapshot(app_window)`, `visual_transparent`, transparent
`ClearColor(wgpu::Color::TRANSPARENT)`, and configured clear-color fallback. `app_handler.rs` keeps
only clear-color owner dispatch before render-scene recording. Render-scene command recording stays
in `window_redraw_render_scene.rs`. Runtime behavior and public effect surfaces remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw clear color owner; visual transparent selection; app-handler clear-color
dispatch.

2026-06-04 desktop runner window redraw webviews owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_webviews.rs` now owns
`WindowRedrawWebViewSyncInput`, `sync_window_redraw_webviews`, and
`window_redraw_webview_snapshot`, including the `WebViewHost` gate,
`webview_has_surfaces_for_window`, cached `last_semantics_snapshot` reuse, fallback
`driver.semantics_snapshot`, and `RunnerWebViewState::sync_window` dispatch. `app_handler.rs` keeps
only webview sync owner dispatch after frame recording and before target updates. `webview.rs` still
owns request/event bridging, placement requests, stale-surface GC, and native host state. Runtime
behavior and public effect surfaces remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split through the docking multiwindow source gate.

Marker summary: redraw webviews owner; webview snapshot selection; app-handler webview sync
dispatch.

2026-06-04 desktop runner window redraw post-render diagnostics owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_post_render_diagnostics.rs` now owns
`WindowRedrawPostRenderDiagnosticsInput` and
`publish_window_redraw_post_render_diagnostics`, dispatching text diagnostics, renderer perf
samples, WGPU hub reports, and WGPU allocator reports after render-scene command recording and
before command-buffer assembly. The underlying diagnostics modules still own their environment
gates, sampling cadence, snapshot construction, report construction, and global-store writes.
Runtime behavior and public effect surfaces remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split through the docking multiwindow source
gate.

Marker summary: redraw post-render diagnostics owner; text diagnostics dispatch; renderer perf and
wgpu reports; app-handler post-render diagnostics dispatch.

2026-06-05 desktop runner window redraw present-capture command owner-split result:
`crates/fret-launch/src/runner/desktop/runner/window_redraw_present_capture_commands.rs` now owns
`WindowRedrawPresentCaptureCommandsInput`, `WindowRedrawPresentCaptureCommands`, and
`prepare_window_redraw_present_capture_commands`. The owner appends `ui_cmd` to engine command
buffers, dispatches diag screenshot capture begin, polls bundle screenshot request dirs, and
dispatches bundle screenshot readback begin. Screenshot finish, present submit, present finish, and
present error recovery remain in existing owners. Runtime behavior and public effect surfaces remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split through the docking
multiwindow source gate.

Marker summary: redraw present capture commands owner; command buffer assembly; screenshot capture
begin; bundle screenshot readback begin; app-handler present-capture dispatch.

2026-06-01 docking declarative drag-route owner-split result:
`ecosystem/fret-docking/src/dock/declarative.rs` now keeps the managed-surface dock-space
entrypoint, layout/render/input orchestration, and public docking APIs. The private
`dock/declarative/drag_route.rs` owner contains internal dock drag route anchor installation,
dock-space node registration, dock drag session-kind checks, active-window invalidation gating, and
drop-time dock drag kind detection. Public dock-space APIs and cross-window docking call paths
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 IMUI text-picker popup render owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` now keeps
session/input/open-policy orchestration only. `text_picker_controls/core/popup.rs` owns popup
request construction and render dispatch, including trigger forwarding, popup open model
forwarding, keyboard handler gating, selected candidate routing, and pending keyboard pick
forwarding. Public picker APIs, popup semantics, and response finalization remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 IMUI text-picker response finalization owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` initially kept
session/input/open-policy/popup orchestration. The 2026-06-01 popup owner follow-up moved popup
request construction and render dispatch into `text_picker_controls/core/popup.rs`.
`text_picker_controls/response.rs` owns popup-result finalization and picked-change response
merging, including changed/edited/deactivated-after-edit propagation. Public
`InputTextPickerResponse` behavior and picker popup semantics remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 IMUI debug-draw paint clip-stack owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint.rs` now keeps command iteration plus
media/shape dispatch only. `paint/clip.rs` owns clip push/pop scene-op emission, empty clip elision,
unmatched pop elision, open-depth tracking, and final clip-stack cleanup. Debug-draw command order,
media dispatch, shape dispatch, and public drawing APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 IMUI disclosure visual tests owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/tests/visual.rs` now keeps visual regression
module routing only. `tests/visual/palette.rs` owns tree-node hover palette coverage, while
`tests/visual/text_roles.rs` owns row label and disclosure indicator text-role coverage. The
shared disclosure test harness, palette assertions, text-role assertions, and public disclosure
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 IMUI switch entry render owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/switch/entry.rs` now keeps public switch model
entrypoints and label identity scoping only. `switch/entry/render.rs` owns model reads,
`PressableProps` construction, active-trigger behavior installation, field chrome, switch state
badge/label mounting, and response return. Public switch facade behavior, label identity scoping,
`SwitchOptions` a11y/test-id wiring, active-trigger semantics, and visual row output remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 window overlay toast render helper owner-split result:
`ecosystem/fret-ui-kit/src/window_overlays/render.rs` now keeps overlay render orchestration and
toast layer assembly. `window_overlays/render/toast_render.rs` owns toast viewport pause state,
part test-id derivation, icon override/glyph helpers, Sonner title/description text helpers, alpha
blending, and stack-shift state/output calculation. Toast layer request synthesis, viewport
pause/focus behavior, action/cancel/close test IDs, icon routing, stack animation, and dismissal
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 editor DragValue model/session owner-split result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps root public surface and control
orchestration. `controls/drag_value/model.rs` owns `DragValueMode` and `DragValueState`.
`controls/drag_value/session.rs` owns hidden layout, numeric-input outcome mapping, and outcome
callback emission. Hidden scrub/input mounting, double-click typing handoff, and public
`DragValue` APIs remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 editor DragValue scrub frame owner-split result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps keyed control orchestration,
mode switching, `DragValueCore` commit/cancel wiring, live model updates, and `NumericInput`
typing routing. `controls/drag_value/scrub.rs` owns scrub frame chrome, prefix/value/suffix segment
rendering, and scrub test-id stamping. Public `DragValue` options and double-click typing handoff
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-01 editor DragValue options owner-split result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps keyed control orchestration, mode
switching, `DragValueCore` wiring, live model updates, and `NumericInput` typing routing while
re-exporting `DragValueOptions`. The private `controls/drag_value/options.rs` owner contains
option fields and defaults. Public `DragValueOptions` import paths, fill-width/flex defaults,
prefix/suffix fields, shared numeric constraints, replace-all typing selection behavior,
id-source semantics, test-id routing, scrub frame behavior, and typing input routing remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 editor input-group icon segment owner-split result:
`ecosystem/fret-ui-editor/src/primitives/input_group/segments.rs` now keeps segment layout, text,
value, axis, and derived-test-id helpers plus re-exports the icon segment helper names.
`input_group/segments/icon.rs` owns icon-button chrome, clear-button routing, multiline clear-button
inset layout, and static icon slot rendering. Existing `crate::primitives::input_group::*` helper
paths remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-05-31 editor theme tests owner-split result:
`ecosystem/fret-ui-editor/src/theme.rs` now keeps public editor theme preset metadata, preset
install/replay helpers, and host theme sync helpers only, plus a `#[cfg(test)] mod tests;` route.
`ecosystem/fret-ui-editor/src/theme/tests.rs` owns the preset metadata, default/dense token patch,
installed-preset replay, and window-metrics sync regressions. `tools/gate_imui_workstream_source.py`
now gates that split so test bodies stay out of the runtime theme entry point, while
`theme/patches.rs` remains the private token patch owner.

2026-05-31 IMUI runtime boundary source-gate refresh:
`fret-imui` remains a thin policy-light authoring facade over `fret-authoring` and `fret-ui`.
`tools/gate_imui_workstream_source.py` now also freezes the public `ecosystem/fret-imui/src/lib.rs`
shape and rejects kit/editor/docking/workspace/plot/shadcn/winit/wgpu imports from the runtime
facade. Generic IMUI policy stays in `fret-ui-kit::imui`, editor controls stay in
`fret-ui-editor`, and docking/multi-window policy stays in `fret-docking` plus runner/backend
owners.

2026-05-31 IMUI virtual-list output owner-split result:
`ecosystem/fret-ui-kit/src/imui/virtual_list_controls/element.rs` keeps keyed runtime list
assembly, default scroll-handle slot state, focus child mounting, row wrapping, rendered-range
tracking, and runtime option consumption. `virtual_list_controls/element/output.rs` owns list-level
semantics decoration and `VirtualListResponse` packaging. Facade method names, keyed substrate
usage, list semantics, row behavior, and response reporting remain unchanged.

2026-05-31 IMUI floating title-bar row owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_title_bar.rs` now keeps owner routing and close-glyph
text construction. `floating_window_title_bar/row.rs` owns row composition, title text mounting,
drag-surface setup, close-button prop selection, and behavior owner calls. Title text-role
selection, drag-surface behavior hooks, close-button behavior wiring, close-glyph text-role helper,
and public floating-window facade behavior remain unchanged.

2026-05-31 IMUI bullet-text element owner-split result:
`ecosystem/fret-ui-kit/src/imui/bullet_text_controls.rs` now keeps the immediate-mode entry point
only. `bullet_text_controls/element.rs` owns bullet indicator/track layout, label semantics/test
IDs, inherited foreground, and compact paragraph mounting. Public bullet text facade behavior,
bullet indicator layout, label test-id forwarding, inherited foreground, and compact paragraph
text-role semantics remain unchanged.

2026-05-31 IMUI multi-select interaction owner-split result:
`ecosystem/fret-ui-kit/src/imui/multi_select.rs` now keeps controllable model hooks, selected-state
reads, selectable response wiring, and changed-signal propagation. `multi_select/interaction.rs`
owns `apply_click(...)` and primary modifier detection for plain, primary-modifier, and shift
selection. Selection mutation semantics, read-only state storage, and regression test routing remain
unchanged.

2026-05-31 IMUI adapter signal owner-split result:
`ecosystem/fret-ui-kit/src/imui/adapters.rs` now keeps the public adapter seam hub,
`AdapterSeamOptions`, and `report_adapter_signal(...)` only. The private
`adapters/signal.rs` owner contains `AdapterSignalMetadata`, `AdapterSignalRecord`, and
`AdapterSignalReporter`. Public `imui::adapters::*` paths, emitted signal accessors, reporter
callback shape, seam options, and signal reporting behavior remain unchanged.

2026-05-31 IMUI open response owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/widgets/open.rs` is now a thin hub that re-exports open
response child owners. `response/widgets/open/disclosure.rs` owns `DisclosureResponse` and its
`empty()` constructor plus query accessors. `response/widgets/open/combo.rs` owns `ComboResponse`
and its query accessors. Public response type names, accessor semantics, crate-visible field access,
and re-export paths remain unchanged.

2026-05-31 editor input-group segments owner-split result:
`ecosystem/fret-ui-editor/src/primitives/input_group.rs` is now a thin hub that re-exports frame,
joined-input, and segment owner APIs at the existing module path. The private
`primitives/input_group/segments.rs` owner now contains inset/segment/row/divider helpers,
icon/clear/text/value segments, derived test-id policy, axis segment composition, and axis tint
color mixing. Crate-visible input-group APIs remain unchanged.

2026-05-31 editor input-group joined owner-split result:
`ecosystem/fret-ui-editor/src/primitives/input_group.rs` now keeps segment helpers, text-role
helpers, axis segment composition, and re-exports joined-input APIs at the existing module path.
The private `primitives/input_group/joined.rs` owner now contains joined frame composition,
leading/input/trailing segment assembly, pointer pressed-state cleanup, pointer down/up/cancel
handlers, and frame override handoff. Crate-visible input-group APIs remain unchanged.

2026-05-31 editor input-group frame owner-split result:
`ecosystem/fret-ui-editor/src/primitives/input_group.rs` now keeps segment helpers, joined-input
assembly, pointer-region behavior, axis segment composition, text-role usage, and re-exports the
frame owner APIs at the existing module path. The private `primitives/input_group/frame.rs` owner
now contains `EditorInputGroupFrameOverrides`, base frame construction, min-height fallback,
semantic/bg/border override application, and `EditorWidgetVisuals` frame visual resolution.
Crate-visible input-group APIs remain unchanged.

2026-05-31 editor TransformEdit element owner-split result:
`ecosystem/fret-ui-editor/src/controls/transform_edit.rs` now keeps public options,
section/outcome records, constructors, presentation adoption, builder methods, and callsite/
id-source keying only. The private `controls/transform_edit/element.rs` owner now contains keyed
element assembly, per-section presentation projection, linked-scale model/sync orchestration,
section row/column composition, derived id/test-id routing, and root test-id decoration. Public
TransformEdit option/control APIs remain unchanged.

2026-05-31 editor VecEdit element owner-split result:
`ecosystem/fret-ui-editor/src/controls/vec_edit.rs` now keeps public VecEdit options, Vec2/Vec3/
Vec4 records, constructors, presentation adoption, builder methods, and callsite/id-source keying
only. The private `controls/vec_edit/element.rs` owner now contains keyed Vec2/Vec3/Vec4 element
assembly, layout-plan consumption, derived axis id/test-id routing, axis group order, and root
test-id decoration. Public VecEdit option/control APIs remain unchanged.

2026-05-31 editor AxisDragValue element owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs` now keeps public control construction,
presentation adoption, builder methods, and callsite/id-source keying only. The private
`controls/axis_drag_value/element.rs` owner now contains keyed scrub/typing element assembly,
focus handoff, Enter/Escape commit/cancel policy, reset segment wiring, and error icon chrome.
Public AxisDragValue options/outcome APIs remain unchanged.

2026-05-31 editor AxisDragValue typing-frame owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps keyed owner
orchestration, scrub frame composition, text input props, focus/key handling, and mode transitions.
The private `controls/axis_drag_value/element/typing.rs` owner contains typing input-group frame
composition plus axis/prefix/suffix/error/reset segments. Scrub mounting, Enter/Escape
commit/cancel behavior, focus handoff, test-id routing, invalid-state icon, reset affordance, and
public AxisDragValue options remain unchanged.

2026-05-31 editor AxisDragValue scrub-frame owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/element.rs` now keeps keyed owner
orchestration, DragValueCore wiring, double-click typing transition, and text-entry focus/key
policy. The private `controls/axis_drag_value/element/scrub.rs` owner contains scrub input-group
frame composition plus axis/value/prefix/suffix/reset segments. DragValueCore commit/cancel
routing, scrub response state mapping, test-id routing, reset affordance, and public AxisDragValue
options remain unchanged.

2026-05-31 editor proof helper owner-split result:
`apps/fret-examples/src/imui_editor_proof_demo.rs` now keeps workflow rendering, docking/window
glue, model factories, and proof orchestration, while
`apps/fret-examples/src/imui_editor_proof_demo/proof_helpers.rs` owns demo-local proof text-role
helpers, numeric presentation adapters, outcome labels, drag preview card composition, outliner
helper structs/readouts, and theme diagnostic projection. Public IMUI/editor APIs and the
collection module remain unchanged.

2026-05-31 SameLine status-drift refresh result:
`P0_CURRENT_SOURCE_AUDIT_2026-05-06.md`, the long-form TODO readiness notes, and historical
milestone summary now agree that SameLine is a narrow proven teaching-surface helper. Remaining
porting-sugar pressure stays on item-width stacks, next-item width defaults, and label-ID helpers
until a future two-surface proof justifies widening.

2026-05-31 same-line porting sugar proof result:
`apps/fret-cookbook/examples/imui_action_basics.rs` now teaches the existing closure-scoped
`ui.same_line_with_options(...)` helper for the IMUI payload button row, with a stable row
`test_id` and source-gate coverage. The P3 readiness note now treats SameLine as a narrow proven
teaching-surface helper while keeping item-width stacks, next-item width defaults, label-suffix
identity parsing, and broad Dear ImGui mutable cursor sugar out of scope.

2026-05-31 debug-draw command type hub owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types.rs` is now a thin private command-type
re-export hub, while
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` owns the private
`DebugDrawCommand` payload enum and all draw-list command variants. Command module wiring, summary
projection, draw-list recording, paint dispatch, public debug-draw summaries, and facade APIs remain
unchanged.

2026-05-31 selectable visual palette owner-split result:
`ecosystem/fret-ui-kit/src/imui/selectable_controls/visual.rs` now keeps selectable row composition
and shared list-row text-role mounting only. `selectable_controls/visual/palette.rs` owns
`SelectablePalette`, selected/hover/pressed/disabled palette fallback order, and highlighted-row
palette semantics. Public selectable behavior remains unchanged.

2026-05-31 boolean-control indicator owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/visual.rs` now keeps shared boolean label text and
indicator re-export routing only. `boolean_controls/visual/indicators.rs` owns checkbox badge,
radio ring/dot, and switch state badge chrome. Public checkbox, radio, and switch behavior remains
unchanged.

2026-06-03 boolean indicator visual child owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/visual/indicators.rs` is now a private re-export
hub only. `boolean_controls/visual/indicators/checkbox.rs` owns checkbox checked/unchecked pill
text, `radio.rs` owns radio outer/dot geometry, and `switch.rs` owns switch On/Off badge text.
Palette channel selection, shared boolean label mounting, and public checkbox/radio/switch behavior
remain unchanged.

2026-05-31 editor style/theme picker density status result:
`ecosystem/fret-ui-editor/src/theme.rs` now exposes `EditorThemePresetV1::picker_status_label()` as
stable picker metadata, and `controls/editor_theme_preset_picker/render.rs` renders compact
`24px`/`22px` density status labels in each preset row. This improves the Dear ImGui-style
style/theme picker affordance while keeping editor policy in `fret-ui-editor` and leaving
`GetStyle`, `PushStyleVar`, global style stacks, and `fret-ui-kit::imui` theme-editor policy out of
scope.

2026-05-31 selectable pressable/a11y props owner-split result:
`ecosystem/fret-ui-kit/src/imui/selectable_controls.rs` now keeps label identity, option state
reads, behavior wiring, and row visual assembly. `selectable_controls/props.rs` owns fill-width
pressable props, enabled/focusable forwarding, listbox-option role fallback, and a11y
label/test-id/selected forwarding. Public selectable behavior remains unchanged.

2026-05-31 disclosure layout props owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/layout.rs` now keeps body `ImUiFacade` mounting,
root/content composition, and test-id routing only. `disclosure_controls/layout/props.rs` owns the
fill-width/auto-height visible-overflow props, zero-gap column props, and content padding
application. Public collapsing-header/tree-node behavior remains unchanged.

2026-05-31 editor axis-drag-value session owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs` now keeps scrub/typing control
orchestration and child-owner routing. `controls/axis_drag_value/session.rs` owns hidden layout
projection, outcome callback emit, and draft/error local model allocation. Scrub/typing mounting,
focus handoff, local state identity, outcome routing, and public AxisDragValue options remain
unchanged.

2026-05-31 editor axis-drag-value ids owner-split result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs` now keeps control orchestration while
delegating scrub/typing/reset child test-id derivation to `controls/axis_drag_value/ids.rs`.
`ids/tests.rs` covers active typing gating, default scrub/typing child ids, explicit reset-id
precedence, and typing reset suffix behavior. Diagnostics naming, control routing, and public
AxisDragValue options remain unchanged.

2026-05-31 editor axis-drag-value joined input chrome reuse result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs` now reuses
`primitives::chrome::joined_text_input_style(...)` for the typing field instead of carrying a local
duplicate of transparent/borderless joined input chrome policy. Joined input transparency,
borderless chrome, focus-ring suppression, text style, typing field routing, scrub mounting, and
public AxisDragValue options remain unchanged.

2026-05-31 editor slider options model owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now re-exports `SliderOptions` from the model
owner and keeps Slider constructors/builders plus control orchestration. `controls/slider/model.rs`
owns `SliderOptions`, its default layout/value/readout/typing policy values, mode/state, hidden
layout projection, and affixed-value helper. Public option field names and defaults remain
unchanged.

2026-05-31 editor slider default text strategy owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now delegates `Slider::new` default format/parse
construction to `controls/slider/model.rs`. `model.rs` owns integer-or-three-decimal display text
and trimmed f64 parsing defaults, while `model/tests.rs` owns focused coverage for those defaults.
Presentation overrides, pointer/typing behavior, and public `SliderOptions` remain unchanged.

2026-05-31 editor slider typing adapter owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps NumericInput composition and typing
mode lifecycle only. `controls/slider/typing.rs` owns typing parse quantization and validation
adapter construction, while `typing/tests.rs` covers clamp/step quantization, unclamped range
validation, and custom-validator delegation. Focus restore, NumericInput typing mode, and public
`SliderOptions` remain unchanged.

2026-05-31 editor slider pointer state owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps pointer event wiring and rendering.
`controls/slider/pointer.rs` owns slide/typing mode resets, drag pointer begin/clear/finish, and
matching-pointer policy, while `pointer/tests.rs` covers the state transitions. Double-click typing
entry, missed-pointer-up cleanup, matching-pointer release, NumericInput commit/cancel reset, and
public `SliderOptions` remain unchanged.

2026-05-31 editor slider runtime paint chrome owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps event wiring, layout, and paint
application only. `controls/slider/chrome.rs` owns resolved runtime paint derivation from chrome
tokens, hover/pressed accent mixing, and disabled alpha attenuation, while `chrome/tests.rs` covers
token precedence plus paint state behavior. Pointer/typing behavior, rendering layout, and public
`SliderOptions` remain unchanged.

2026-05-31 editor slider geometry chrome owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now consumes resolved track/thumb geometry from
the chrome owner. `controls/slider/chrome.rs` owns slider track/thumb metric fallback, minimum track
height, thumb-at-least-track clamping, and radius derivation, while `chrome/tests.rs` covers default
and clamped geometry behavior. Pointer math, rendering layout, and public `SliderOptions` remain
unchanged.

2026-05-31 editor slider track/thumb props chrome owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps element composition order and value
display assembly only for the track row. `controls/slider/chrome.rs` owns track flex props,
left/right segment container props, and thumb container props, while `chrome/tests.rs` covers track
layout, segment shape, fixed thumb diameter, border, and radius behavior. Render order, pointer
math, value display layout, and public `SliderOptions` remain unchanged.

2026-05-31 editor slider frame owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps public Slider orchestration, keyed
state, pointer handlers, value math, resolved paint/geometry handoff, and NumericInput typing mode.
The private `controls/slider/frame.rs` owner contains input-group frame assembly, track/thumb
children, optional value display segment, readout styling, and value display test-id decoration.
Pointer event wiring, typing handoff, track/thumb render order, and public `SliderOptions` remain
unchanged.

2026-06-01 editor slider element owner-split result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps the public Slider API,
`NumericPresentation` adoption, identity keying, and child-owner routing. The private
`controls/slider/element.rs` owner contains keyed element assembly, slider state/focus-handoff
storage, pressable pointer hooks, NumericInput typing composition, focus handoff synchronization,
and frame owner invocation. Public constructors/builders, identity semantics, pointer/typing
behavior, resolved paint/geometry policy, and public `SliderOptions` remain unchanged.

2026-05-31 editor numeric-input model/session owner-split result:
`ecosystem/fret-ui-editor/src/controls/numeric_input.rs` now keeps NumericInput constructors,
builder methods, keyed control orchestration, validation message rendering, and presentation test
routing. `controls/numeric_input/model.rs` owns options, error display, outcome/type aliases, and
edit-line text-style policy; `model/tests.rs` owns line-box coverage; `session.rs` owns draft/error
local model allocation. Public option/type alias names and default selection behavior remain
unchanged.

2026-05-31 editor text-field draft-controller owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/buffered.rs` now re-exports
`TextFieldDraftController` from `buffered/controller.rs` and keeps buffered state, focus/blur
planning, session sync, commit/cancel helpers, shortcut policy, and tests. `controller.rs` owns the
public controller, private binding, commit/discard forwarding, bind/unbind, and Debug output. The
public text-field re-export path remains unchanged.

2026-05-31 editor text-field buffered tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/text_field/buffered.rs` now keeps buffered runtime state,
focus/blur/session helpers, commit/cancel helpers, and shortcut policy only. `buffered/tests.rs`
owns focus/blur plan coverage, stable line-box default coverage, and draft-controller
commit/discard/no-op behavior tests.

2026-05-31 editor text-field element child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_field.rs` now keeps the public control/options and
draft-controller re-export. `controls/text_field/element.rs` owns keyed element construction,
input/textarea assembly, buffered session wiring, clear affordance wiring, and focus-selection
handoff. Public TextField builders, option names/defaults, buffered draft behavior, clear-button
reset behavior, multiline shortcuts, password mode, assistive semantics, and IMUI adapter routing
remain unchanged.

2026-05-31 editor color-edit numeric tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs` now keeps color-edit policy, picker,
preview, drag/drop, copy, tooltip, and shared HSV assertion coverage. `color_edit/tests/numeric.rs`
owns popup numeric mode ordering, RGB/HSV readout formatting, hex/numeric parsing, alpha
preservation, and HSV conversion roundtrip coverage.

2026-05-31 editor color-edit picker tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs` now keeps color-edit policy/defaults,
drag/drop, copy, tooltip, and shared HSV assertion coverage. `color_edit/tests/picker.rs` owns SV
picker, hue bar, hue wheel, alpha bar, checkerboard, preview alpha visibility, original restore,
and a11y alpha percent coverage. Picker geometry, preview alpha policy, original-restore component
rules, and public color-edit behavior remain unchanged.

2026-05-31 editor color-edit hue-wheel model child-owner result:
`ecosystem/fret-ui-editor/src/controls/color_edit/model.rs` now keeps numeric text/parse helpers,
RGB/HSV conversion, SV/hue bar helpers, and root re-exports. `color_edit/model/hue_wheel.rs` owns
hue-wheel geometry, target selection, barycentric triangle math, cursor projection, and hue-wheel
HSV updates. Hue-wheel import paths inside `color_edit`, target hit-testing, rotated triangle
geometry, SV cursor projection, HSV update math, numeric input parsing, and picker tests remain
unchanged.

2026-05-31 editor color-edit popup policy tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs` now keeps palette/history, drag/drop,
eyedropper, tooltip/copy payload, and shared HSV assertion coverage.
`color_edit/tests/popup_policy.rs` owns popup defaults, side-preview ratio/defaults, alpha-preview
modes, tooltip/copy defaults, visible-content swatch policy, and runtime override sync coverage.
Popup policy and public color-edit behavior remain unchanged.

2026-05-31 editor color-edit drag/drop tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs` now keeps palette/history, eyedropper,
tooltip/copy payload, and shared HSV assertion coverage. `color_edit/tests/drag_drop.rs` owns
palette slot drop defaults/events and color drag/drop payload shape/application coverage. Palette
slot metadata, RGB-only palette slot semantics, local payload defaults, and COL3F/COL4F alpha rules
remain unchanged.

2026-05-31 editor color-edit palette tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs` now keeps eyedropper, tooltip/copy
payload, and shared HSV assertion coverage. `color_edit/tests/palette.rs` owns preset uniqueness,
hex formatting, default palette source, and app-owned palette/history slot coverage. Public
ColorEdit palette/history option behavior remains unchanged.

2026-05-31 editor color-edit affordance tests owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests.rs` is now a test hub with module routing
plus the shared HSV assertion helper. `color_edit/tests/affordances.rs` owns app-owned eyedropper
defaults, sample alpha application, tooltip preview text, and copy-as payload formats. Public
ColorEdit eyedropper/tooltip/copy behavior remains unchanged.

2026-05-31 menu interaction behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/interaction/behavior.rs` now owns active-trigger
installation and keyboard behavior orchestration only. `behavior/activation.rs` owns activate
handling, close-popup mutation, clicked transient recording, lifecycle instant marking, and command
dispatch. `behavior/response.rs` owns clicked transient consumption and response population.
Public menu item behavior remains unchanged.

2026-05-31 checkbox entry render owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox/entry.rs` now owns public checkbox model
entrypoints and label identity scoping only. `entry/render.rs` owns model reads, pressable props,
behavior installation, field chrome, indicator/label mounting, and response return. Public
checkbox facade behavior remains unchanged.

2026-05-31 tooltip runtime model owner-split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime.rs` now keeps trigger-id validation,
provider defaults, layout/interaction/request orchestration, and response return only.
`runtime/models.rs` owns local open/panel models, Radix trigger event models, last-pointer
tracking, dismiss handler installation, and pointer-move open gate installation. Public tooltip
facade behavior remains unchanged.

2026-05-31 virtual-list element assembly owner-split result:
`ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs` is now a thin module/re-export hub.
`virtual_list_controls/element.rs` owns keyed runtime list assembly, default scroll-handle slot
state, build-focus forwarding, list semantics, and response packaging. Public virtual-list facade
behavior remains unchanged.

2026-05-31 bullet-text test-owner split result:
`ecosystem/fret-ui-kit/src/imui/bullet_text_controls/tests.rs` is now a thin test hub.
`tests/text_role.rs` owns compact paragraph text-role coverage for bullet labels. Public
bullet-text behavior remains unchanged.

2026-05-31 drag/drop test-owner split result:
`ecosystem/fret-ui-kit/src/imui/drag_drop/tests.rs` is now a thin test hub.
`tests/source.rs` owns source no-trigger fallback coverage, while `tests/target.rs` owns target
no-trigger fallback coverage. Public drag/drop behavior remains unchanged.

2026-05-31 label-identity test-owner split result:
`ecosystem/fret-ui-kit/src/imui/label_identity/tests.rs` is now a thin test hub.
`tests/double_hash.rs` owns plain and `##` identity coverage, while `tests/triple_hash.rs` owns
`###` stable identity and precedence coverage. Public label identity behavior remains unchanged.

2026-05-31 image-item test-owner split result:
`ecosystem/fret-ui-kit/src/imui/image_item_controls/tests.rs` is now a thin test hub.
`tests/helpers.rs` owns size/opacity/UV normalization coverage, while `tests/props.rs` owns image
props fill/fit/sampling/UV coverage. Public image-item behavior remains unchanged.

2026-05-31 radio entry/props owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/radio.rs` is now a thin module/re-export hub.
`radio/entry.rs` owns label identity, behavior installation, field chrome, and visual row assembly,
while `radio/props.rs` owns `PressableProps` plus radio semantics wiring. Public radio behavior
remains unchanged.

2026-05-31 disclosure visual style owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/style.rs` is now a thin re-export hub.
`style/padding.rs` owns content padding by disclosure kind, while `style/palette.rs` owns
`DisclosurePalette` and palette resolution. Public disclosure behavior remains unchanged.

2026-05-31 input-text policy command owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/policy_commands/input.rs` now installs the focused
key handler and dispatches resolved commands only. `input/resolve.rs` owns completion/history/
undo/redo command capture, repeat gating, IME/meta/alt suppression, and key-to-command resolution.
Public input-text command behavior remains unchanged.

2026-05-31 table body-row cell-preparation owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/render/body_rows.rs` now keeps row iteration,
keying, striping, and row wrapping only. `body_rows/cells.rs` owns hidden-column filtering,
fallback empty-cell creation, default/explicit test-id precedence, and prepared-cell wrapping.
Public table rendering behavior remains unchanged.

2026-05-31 table body wrapper owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/body.rs` is now a thin hub.
`body/row.rs` owns row wrapping and grouped row chrome, while `body/cell.rs` owns cell wrapping
and semantics decoration. Public table body wrapper behavior remains unchanged.

2026-05-31 table render planning owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/render.rs` now keeps final table assembly only.
`render/plan.rs` owns visible-column scanning, horizontal scroll-handle planning, header gating,
and column test-id suffix preparation. Public table rendering behavior remains unchanged.

2026-05-31 table row-group composition owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/row_groups.rs` now dispatches only.
`row_groups/unpinned.rs` owns the no-pinned fill/scroll path, while `row_groups/pinned.rs` owns
left/center/right pinned group assembly. Public table row-group behavior remains unchanged.

2026-05-31 floating-window resize drag-application owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/drag_apply.rs` now owns delta
calculation and `last_resize_position` updates only. `drag_apply/bounds.rs` owns min/max clamps,
while `drag_apply/handles.rs` owns handle-specific size/position mutation. Public floating-window
resize drag behavior remains unchanged.

2026-05-31 floating-window resize handle-mutation owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/drag_apply/handles.rs` now dispatches
by handle family only. `handles/edge.rs` owns left/right/top/bottom edge mutation, while
`handles/corner.rs` owns corner mutation. Public floating-window resize handle behavior remains
unchanged.

2026-05-31 floating-window resize commit mutation owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/commit.rs` now keeps state
transaction, pixel snap, and output packing. `commit/mutation.rs` owns collapsed/reset/drag
lifecycle mutation. Public floating-window resize commit behavior remains unchanged.

2026-05-31 facade-writer text test-owner split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/tests.rs` is now a thin test hub.
`tests/text.rs` owns `ui.text(...)` dense single-line coverage, while `tests/wrapped.rs` owns
`ui.text_wrapped(...)` explicit wrapping coverage. Public facade writer behavior remains unchanged.

2026-05-31 virtual-list test-owner split result:
`ecosystem/fret-ui-kit/src/imui/virtual_list_controls/tests.rs` is now a thin test hub.
`tests/fixed_known.rs` owns fixed and known row clipping coverage, while `tests/measured.rs`
owns measured overflow visibility coverage. Public virtual-list behavior remains unchanged.

2026-05-31 multi-select test-owner split result:
`ecosystem/fret-ui-kit/src/imui/multi_select/tests.rs` is now a thin test hub.
`tests/clicks.rs` owns plain/primary/shift click policy coverage, while
`tests/ordered_selection.rs` owns ordered-selection normalization and anchor repair coverage.
Public multi-select behavior remains unchanged.

2026-05-31 tooltip test-owner split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/tests.rs` is now a thin test hub.
`tests/mount.rs` owns no-trigger mount behavior, `tests/text_role.rs` owns compact body text-role
coverage, and `tests/options.rs` owns default placement/delay/test-id coverage. Public tooltip
behavior remains unchanged.

2026-05-31 control-chrome test-owner split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/tests.rs` is now a thin test hub.
`tests/text_roles.rs` owns control/fill text single-line shrink coverage, while `tests/layout.rs`
owns row/stack dense layout helper coverage. Public control chrome behavior remains unchanged.

2026-05-31 text-control chrome test-owner split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/tests.rs` is now a thin test hub.
`tests/input_chrome.rs` owns input-text fixed-height chrome coverage, while
`tests/textarea_chrome.rs` owns textarea fill-width chrome coverage. Public text-control behavior
remains unchanged.

2026-05-31 selectable test-owner split result:
`ecosystem/fret-ui-kit/src/imui/selectable_controls/tests.rs` is now a thin test hub.
`tests/palette.rs` owns selected/hover/disabled palette and highlight coverage, while
`tests/row_text.rs` owns shared list-row text-role coverage for selectable rows. Public selectable
behavior remains unchanged.

2026-05-31 table-column visibility test-owner split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/tests.rs` is now a thin test hub.
`tests/state.rs` owns runtime override, snapshot, and column-application coverage, while
`tests/menu.rs` owns stable menu-column id, visible label, and test-id suffix coverage. Public
table-column visibility behavior remains unchanged.

2026-05-31 menu-control test-owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/tests.rs` is now a thin test hub.
`tests/text_roles.rs` owns label/shortcut/indicator text-role coverage, while
`tests/root.rs` owns root pressable and visible-child mounting coverage. Shared helpers and module
routing stay in the root file, and the public menu-item behavior remains unchanged.

2026-05-31 debug-draw path-builder test-owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/path_builder.rs` is now a thin test
hub. Stroke/fill command recording and invalid finished-path cleanup live in
`tests/path_builder/commands.rs`, rectangle and rounded-rectangle sampling coverage lives in
`tests/path_builder/rects.rs`, Bezier defaults live in `tests/path_builder/curves.rs`, and
circular/elliptical arc defaults live in `tests/path_builder/arcs.rs`. The public
`ImUiDebugDrawPath` behavior and source-gated path-builder coverage remain unchanged.

2026-05-31 debug-draw draw-list command test-owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list/commands.rs` is now a thin
test hub. Broad command-order coverage lives in `tests/draw_list/commands/core.rs`, triangle
mesh/image mesh coverage lives in `tests/draw_list/commands/meshes.rs`, image/SVG overlay coverage
lives in `tests/draw_list/commands/media.rs`, and concave polygon fill coverage lives in
`tests/draw_list/commands/polygons.rs`. `ImUiDebugDrawList` command recording behavior and public
debug-draw authoring APIs remain unchanged.

2026-05-31 debug-draw core command-order test-owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list/commands/core.rs` is now a
thin nested hub. `core/linear.rs` owns line/poly/rect/quad/triangle command ordering,
`core/round_curve.rs` owns circle/ngon/ellipse/Bezier ordering, `core/text.rs` owns text command
ordering, and `core/order.rs` retains the all-command aggregate order proof. Public debug-draw
behavior remains unchanged.

2026-05-31 debug-draw draw-list summary test-owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/draw_list/summaries.rs` is now a thin
test hub. `tests/draw_list/summaries/merge_order.rs` owns command-summary merge ordering,
`tests/draw_list/summaries/counts.rs` owns aggregate list-summary counts, and
`tests/draw_list/summaries/clip_stack.rs` owns effective clip-stack plus clip push/pop command
coverage.

2026-05-31 debug-draw path helper test-owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/tests/paths.rs` is now a thin test hub.
Rect/polyline/polygon/triangle/quad path closure coverage lives in `tests/paths/linear.rs`,
circle/ngon/ellipse generation and ellipse default/rotation coverage lives in `tests/paths/round.rs`,
and native quadratic/cubic Bezier command coverage lives in `tests/paths/beziers.rs`. Path helper
behavior and debug-draw scene output contracts remain unchanged.

2026-05-30 text-field buffered child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_field.rs` now keeps the public control/options and
layout orchestration only. `controls/text_field/buffered.rs` owns the draft controller, buffered
state, session planning, commit/cancel helpers, clear-button session reset, and the buffered unit
tests. Public TextField options, draft-controller API, buffered blur behavior, and
`text_field_api_smoke` coverage remain unchanged.

2026-05-30 text-assist field model child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now keeps control orchestration, panel
rendering, overlay request, key handling, and accept commits. `controls/text_assist_field/model.rs`
owns `OnTextAssistFieldAccept`, `TextAssistFieldSurface`, `TextAssistFieldOptions`,
`RenderedTextAssistPanel`, and the focused option/default tests. Public option names, default
unbuffered input policy, item test-id prefix fallback, rendered panel handoff, inline empty-label
behavior, and anchored-overlay height policy remain unchanged.

2026-05-31 text-assist field model tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field/model.rs` now keeps option/model records
and test-owner routing. `controls/text_assist_field/model/tests.rs` owns option/default coverage.
Public option names, default unbuffered input policy, item test-id prefix fallback, rendered panel
handoff, and root control orchestration remain unchanged.

2026-05-30 text-assist field overlay child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now keeps input and panel
orchestration. `controls/text_assist_field/overlay.rs` owns anchored placement, diagnostics
placement recording, dismissible popover request construction, branch registration, query
dismissal writeback, and overlay open-state model creation. Anchor fallback, popper placement,
overlay diagnostics, dismiss behavior, and local open model behavior remain unchanged.

2026-05-30 text-assist field panel child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now keeps input/key orchestration and
accept flow. `controls/text_assist_field/panel.rs` owns suggestion panel content, option rows,
scroll wrapping, listbox semantics, popup chrome, and rendered panel packaging. Visible-match
listbox semantics, active/disabled row palette, option activation, scroll threshold, popup surface
chrome, item test-id derivation, and rendered panel handoff remain unchanged.

2026-05-31 text-assist field root tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field.rs` now keeps input/key orchestration,
accept flow, helper policy, and test-owner routing. `controls/text_assist_field/tests.rs` owns
inline empty-label and anchored-overlay default-height coverage. Panel and overlay child-owner
boundaries remain unchanged.

2026-05-30 property-row reset child-owner result:
`ecosystem/fret-ui-editor/src/composites/property_row.rs` now keeps the row layout and value
orchestration plus reset delegation only. `composites/property_row/reset.rs` owns
`OnPropertyRowReset`, `PropertyRowResetOptions`, `PropertyRowReset`, and the reset pressable /
activation helpers. Row layout, value-slot growth, reset keying, and property chrome semantics
remain unchanged.

2026-05-30 property-row layout child-owner result:
`ecosystem/fret-ui-editor/src/composites/property_row.rs` now keeps the public composite,
test-facing value slot marker, and row/column child assembly. `composites/property_row/layout.rs`
owns `PropertyRowLayoutVariant`, theme-derived resolved layout/chrome metrics, auto-stack variant
selection, min-height application, and focused layout-policy tests. Public row options, row vs
column rendering, value-slot growth, fixed label line boxes, and reset/action slot mounting remain
unchanged.

2026-05-30 property-row tests child-owner result:
`ecosystem/fret-ui-editor/src/composites/property_row.rs` now keeps implementation and a thin
`mod tests;` hook. `composites/property_row/tests.rs` owns the wrapping label/value-slot regression
harness, including value-slot marker lookup, wrapping text services, and layout-query assertions.
Public row options, label line-box behavior, wrapping value growth, and value-slot overflow
contracts remain unchanged.

2026-05-31 editor field-status tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/field_status.rs` now keeps badge implementation and palette
resolution plus test-owner routing. `controls/field_status/tests.rs` owns short-label and luma
coverage. Compact badge text-role routing, status palette mixing, destructive/loading label policy,
and badge layout remain unchanged.

2026-05-31 editor chrome tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/chrome.rs` now keeps editor chrome/style resolution and
test-owner routing. `primitives/chrome/tests.rs` owns text-field/text-area chrome policy coverage.
Editor token precedence, legacy component fallback behavior, line-height policy, and focus ring
token routing remain unchanged.

2026-05-31 editor colors tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/colors.rs` now keeps semantic color helper implementation
and test-owner routing. `primitives/colors/tests.rs` owns color fallback policy coverage.
Editor-owned token precedence, legacy text-field fallback behavior, shared palette fallbacks,
invalid lane fallback, and popup/panel fallback order remain unchanged.

2026-05-31 editor density tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/density.rs` now keeps density policy implementation and
test-owner routing. `primitives/density/tests.rs` owns affordance extent coverage. Editor density
defaults, theme metric resolution, non-negative clamping, and hit-target extent policy remain
unchanged.

2026-05-31 editor edit-session tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/edit_session.rs` now keeps edit-session primitive
implementation and test-owner routing. `primitives/edit_session/tests.rs` owns dirty-state
coverage. Pre-edit capture, commit/cancel clearing, active-state reporting, and changed-from
semantics remain unchanged.

2026-05-31 editor numeric-format tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/numeric_format.rs` now keeps numeric format
implementation and test-owner routing. `primitives/numeric_format/tests.rs` owns formatting and
presentation coverage. Fixed decimal formatting, plain parsing, affix format/parse semantics,
duplicate chrome affix suppression, presentation chrome layering, and degrees helper behavior
remain unchanged.

2026-05-31 editor numeric-text-entry tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/numeric_text_entry.rs` now keeps numeric text-entry policy
implementation and test-owner routing. `primitives/numeric_text_entry/tests.rs` owns
replacement-plan coverage. Focus handoff state, replace-on-focus arming, draft/error
synchronization, paste/delete/navigation key planning, and text-insertion key detection remain
unchanged.

2026-05-31 editor numeric-value tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/numeric_value.rs` now keeps numeric constraint
implementation and test-owner routing. `primitives/numeric_value/tests.rs` owns bounds and
quantization coverage. Bound normalization, finite-step filtering, clamp ordering, quantization
origin, and scalar conversion behavior remain unchanged.

2026-05-31 editor popup-surface tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/popup_surface.rs` now keeps popup chrome implementation
and test-owner routing. `primitives/popup_surface/tests.rs` owns popup surface chrome coverage.
Overlay/inline shadow policy, popup token precedence, radius/shadow metric resolution, shadow color
fallback, and dense preset popup chrome remain unchanged.

2026-05-31 editor popup-list tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/popup_list.rs` now keeps popup-list
state/dimensions/palette policy and test-owner routing. `primitives/popup_list/tests.rs` owns
palette and height coverage. Popup-list state records, row gap/height helpers, default max-height
budget, highlight palette, disabled foreground, and text-role ownership in the readout child owner
remain unchanged.

2026-05-31 editor visuals tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/visuals.rs` now keeps editor widget visual policy and
test-owner routing. `primitives/visuals/tests.rs` owns visual-state policy coverage. Shared visual
policy, selected-frame fill/foreground behavior, disabled alpha attenuation, invalid chrome
routing, and icon-button hover overlay source remain unchanged.

2026-05-31 editor drag-value core tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/drag_value_core.rs` now keeps drag-to-edit primitive
implementation and test-owner routing. `primitives/drag_value_core/tests.rs` owns session and
response coverage. Scrub session commit/cancel semantics, response accessor privacy, and response
construction remain unchanged.

2026-05-31 editor inspector-panel tests child-owner result:
`ecosystem/fret-ui-editor/src/composites/inspector_panel.rs` now keeps panel composition and
test-owner routing. `composites/inspector_panel/tests.rs` owns the single-line title layout
regression harness. Panel composition, title text-role routing, toolbar/body slots, and layout query
coverage remain unchanged.

2026-05-31 editor gradient tests child-owner result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps gradient editor composition
and preview implementation. `composites/gradient_editor/tests.rs` owns empty-state text-role
coverage. Gradient stop composition, preview canvas behavior, empty-state copy, and editor readout
text-role routing remain unchanged.

2026-05-31 editor gradient preview child-owner result:
`ecosystem/fret-ui-editor/src/composites/gradient_editor.rs` now keeps public composition and stop
rows. `composites/gradient_editor/preview.rs` owns preview drag state, pressable pointer handlers,
gradient fill construction, and stop marker painting. Public gradient editor builders, stop
sorting, preview drag mutation, marker painting, empty-state copy, and IMUI adapter routing remain
unchanged.

2026-05-30 editor readout popup-list child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the non-popup editor readout
helpers only. `primitives/readout/popup_list.rs` owns the popup-list row, centered-row,
option-caption, and empty-state text helpers plus the focused popup-list tests. Popup row geometry,
alignment, empty-state copy, and popup-list text-role coverage remain unchanged.

2026-05-31 editor readout popup-list tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout/popup_list.rs` now keeps popup-list readout helper
implementation and test-owner routing. `primitives/readout/popup_list/tests.rs` owns popup-list
readout text-role coverage. Popup row text props, empty text props, centered row alignment, fixed
caption line boxes, and direct `TextProps` allowance for the readout child owner remain unchanged.

2026-05-30 editor readout theme-preset child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the shared non-popup readout hub and
re-exports theme preset picker text helpers. `primitives/readout/theme_preset.rs` owns the theme
picker header, row label, row status text props, and fixed-line tests. Compact header sizing, row
label/status line boxes, re-export paths, and style/theme picker rendering remain unchanged.

2026-05-31 editor readout theme-preset tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout/theme_preset.rs` now keeps theme-preset readout
helper implementation and test-owner routing. `primitives/readout/theme_preset/tests.rs` owns
theme-preset fixed-line coverage. Compact header sizing, fixed row label/status line boxes, fixed
status slot, re-export paths, and style/theme picker rendering remain unchanged.

2026-05-31 editor input-group tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/input_group.rs` now keeps joined input-group helper
implementation and test-owner routing. `primitives/input_group/tests.rs` owns value text-role layout
coverage. Joined frame helpers, segment helpers, axis marker routing, and value text
shrink/ellipsis policy remain unchanged.

2026-05-31 editor readout tests child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the non-popup readout helper hub and
child-owner re-exports. `primitives/readout/tests.rs` owns compact readout sizing and editor
text-role regression tests. Non-popup helper names, text-role layout policy, compact readout
sizing, and popup/theme-preset child-owner boundaries remain unchanged.

2026-05-31 editor readout feedback child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the shared readout hub and re-exports
feedback helpers. `primitives/readout/feedback.rs` owns status badge, inline error, and validation
message text props. Status badge, inline error, validation message layout semantics, re-export
paths, and readout regression coverage remain unchanged.

2026-05-31 editor readout property child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the shared readout hub and re-exports
property helpers. `primitives/readout/property.rs` owns property group header, inspector title,
property-row label, and reset glyph text props. Property chrome layout semantics, re-export paths,
and readout regression coverage remain unchanged.

2026-05-31 editor readout input child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the shared readout hub and re-exports
input helpers. `primitives/readout/input.rs` owns inline control label, input segment, input value,
and axis marker text props. Input/axis layout semantics, re-export paths, and readout regression
coverage remain unchanged.

2026-05-31 editor readout section child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps the shared readout hub and re-exports
section helpers. `primitives/readout/section.rs` owns section badge and section heading text props.
Transform section badge/heading layout semantics, re-export paths, and readout regression coverage
remain unchanged.

2026-05-31 editor readout surface child-owner result:
`ecosystem/fret-ui-editor/src/primitives/readout.rs` now keeps only the shared compact readout style
owner plus child-module re-exports. `primitives/readout/surface.rs` owns preview caption,
empty-state, and tooltip readout text props. Color popup preview captions, gradient empty-state
text, color tooltip readout layout semantics, re-export paths, and readout regression coverage
remain unchanged.

2026-05-30 editor vec-edit axis child-owner result:
`ecosystem/fret-ui-editor/src/controls/vec_edit.rs` now keeps Vec2/Vec3/Vec4 public control
orchestration. `controls/vec_edit/axis.rs` owns `VecEditAxis`, `VecEditAxisOutcome`, reset options,
reset action packaging, axis group rendering, and the focused axis-outcome test. Public
constructors, reset options, outcome accessors, transform-edit routing, identity derivation, and
row/column auto layout remain unchanged.

2026-05-31 editor vec-edit layout child-owner result:
`ecosystem/fret-ui-editor/src/controls/vec_edit.rs` now keeps Vec2/Vec3/Vec4 public control
orchestration and axis group composition. `controls/vec_edit/layout.rs` owns axis token color
resolution, auto-stack threshold calculation, Row/Column direction selection, grow policy, and
id-source suffix derivation, while `controls/vec_edit/layout/tests.rs` owns focused layout-policy
regressions. Public constructors, row/column auto-stack behavior, axis fallback colors, test-id
derivation, transform-edit routing, and axis group composition remain unchanged.

2026-05-30 editor transform section child-owner result:
`ecosystem/fret-ui-editor/src/controls/transform_edit.rs` now keeps TransformEdit public surface,
Vec3 composition, outcome routing, and linked-scale model/sync logic.
`controls/transform_edit/sections.rs` owns row/column section chrome, badge/heading text-role
routing, and link/uniform toggle layout. Public options, Vec3 composition, section text roles,
link-scale test IDs, row/column selection, and uniform-scale sync remain unchanged.

2026-05-30 editor transform sync child-owner result:
`ecosystem/fret-ui-editor/src/controls/transform_edit.rs` now keeps TransformEdit public surface
and Vec3 composition. `controls/transform_edit/sync.rs` owns linked-scale local model creation,
sync-slot allocation, uniform-scale projection, model writeback, and focused sync tests. Public
options, link toggle behavior, single-axis uniform projection, multi-axis edit rejection, and
near-equal threshold policy remain unchanged.

2026-05-30 editor axis-drag-value model child-owner result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs` now keeps the `AxisDragValue<T>` control
orchestration. `controls/axis_drag_value/model.rs` owns public option/reset/outcome records,
internal scrub/typing mode/state records, and the focused input text-style test. Public option
fields, reset action packaging, outcome callback aliases, focus handoff behavior, mode transitions,
and input text line-box policy remain unchanged.

2026-05-31 editor axis-drag-value tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value.rs` now keeps control orchestration and
child-owner routing. `controls/axis_drag_value/tests.rs` owns presentation format/parse/chrome-affix
coverage. `AxisDragValue::from_presentation`, NumericPresentation adoption, axis tint routing, and
model child-owner boundaries remain unchanged.

2026-05-31 editor axis-drag-value model tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/axis_drag_value/model.rs` now keeps model/type definitions
and test-owner routing. `controls/axis_drag_value/model/tests.rs` owns density line-height coverage.
Typing line-height resolution, default options, reset action packaging, outcome callback aliases,
and control routing remain unchanged.

2026-05-30 editor slider chrome child-owner result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps slider state, value flow, pointer/input
switching, and layout orchestration. `controls/slider/chrome.rs` owns token fallback, color mixing,
alpha attenuation, resolved chrome fields, and the focused chrome precedence test. Pointer/typing
behavior, value formatting, theme token precedence, hover/pressed/disabled color mixing, and public
slider options remain unchanged.

2026-05-31 editor slider chrome tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/slider/chrome.rs` now keeps slider chrome/color resolution
implementation and test-owner routing. `controls/slider/chrome/tests.rs` owns chrome precedence
coverage. Theme token precedence, fallback palette behavior, color mixing, alpha attenuation, and
slider control routing remain unchanged.

2026-05-30 editor slider value-math child-owner result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps slider state, event handling, typing
handoff, and layout orchestration. `controls/slider/value_math.rs` owns value quantization,
normalized progress, pointer-position projection, and focused value-math tests. Pointer-x mapping,
clamp/step quantization, thumb-radius compensation, track-degenerate behavior, typing fallback, and
public slider options remain unchanged.

2026-05-31 editor slider value-math tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/slider/value_math.rs` now keeps value-domain math
implementation and test-owner routing. `controls/slider/value_math/tests.rs` owns value-math
coverage. Quantization, normalized progress, thumb-radius pointer projection, degenerate track
fallback, and slider control routing remain unchanged.

2026-05-31 editor slider pointer projection value-math result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now delegates pointer-down and pointer-move local
x projection to `controls/slider/value_math.rs::value_from_slider_local_x(...)`. The value-math
owner now covers value-readout width subtraction, frame padding compensation, pointer clamping,
thumb-radius mapping, and clamp/step quantization. Pointer down/drag event flow, value display
layout, and public slider options remain unchanged.

2026-05-31 editor slider presentation tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps slider control orchestration and
child-owner routing. `controls/slider/tests.rs` owns presentation adoption coverage. Slider public
constructors, NumericPresentation adoption, duplicate chrome affix suppression, and slider
chrome/value-math child-owner boundaries remain unchanged.

2026-05-31 editor slider model child-owner result:
`ecosystem/fret-ui-editor/src/controls/slider.rs` now keeps the slider public surface, control
orchestration, pointer/input behavior, and child-owner routing. `controls/slider/model.rs` owns
`SliderMode`, `SliderState`, hidden layout projection, and affixed value composition, while
`controls/slider/model/tests.rs` owns affixed-value helper coverage. Pointer/typing behavior, focus
restore, hidden slide/input mounting, duplicate chrome affix suppression, and public slider options
remain unchanged.

2026-05-30 editor enum-select row child-owner result:
`ecosystem/fret-ui-editor/src/controls/enum_select.rs` now keeps public control/options, trigger
composition, and overlay orchestration. `controls/enum_select/row.rs` owns option-row rendering,
selection commit policy, item test-id normalization, and the focused row-policy tests. Trigger
composition, overlay dismissal, filter/search behavior, popup empty-state rendering, row chrome,
and selected-row reveal remain unchanged.

2026-05-31 editor enum-select row tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/enum_select/row.rs` now keeps option-row implementation and
test-owner routing. `controls/enum_select/row/tests.rs` owns commit-policy and item-id coverage.
Option-row rendering, popup-list row text-role routing, and overlay boundaries remain unchanged.

2026-05-30 editor enum-select overlay child-owner result:
`ecosystem/fret-ui-editor/src/controls/enum_select.rs` now keeps public control/options and trigger
composition. `controls/enum_select/overlay.rs` owns overlay request assembly, popup panel/list
layout, selected-row reveal, close-focus policy, viewport test-id derivation, and overlay helper
tests. Trigger composition, search/filter behavior, popup placement, dismissal policy, row routing,
and focus restore remain unchanged.

2026-05-31 editor enum-select overlay tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/enum_select/overlay.rs` now keeps overlay implementation and
test-owner routing. `controls/enum_select/overlay/tests.rs` owns close-focus, viewport-id, and
visibility-contract coverage. Overlay request assembly, popup panel/list layout, selected-row
reveal, viewport test-id derivation, row routing, and focus restore remain unchanged.

2026-05-30 editor theme preset picker render child-owner result:
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker.rs` now keeps preset
installation, theme resolution, and render dispatch only. `render.rs` owns listbox semantics,
header row, preset rows, and color mixing. Selected preset sync, item test IDs, activation
behavior, and theme replay semantics remain unchanged.

2026-05-31 editor theme preset picker tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker.rs` now keeps preset
installation, theme resolution, render dispatch, and test-owner routing.
`controls/editor_theme_preset_picker/tests.rs` owns listbox semantics, selected state, click
activation, and reversible preset replay coverage. Render child-owner boundaries remain unchanged.

2026-06-01 editor theme preset picker row owner-split result:
`ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker/render.rs` now keeps listbox
container semantics, preset iteration, and header text routing. The private
`controls/editor_theme_preset_picker/render/row.rs` owner contains ListBoxOption semantics,
pressable activation, row chrome, row test IDs, density status label rendering, and color mixing.
Listbox semantics, selected-state behavior, click activation, and public picker APIs remain
unchanged.

2026-05-31 editor numeric-input tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/numeric_input.rs` now keeps numeric input control
orchestration and test-owner routing. `controls/numeric_input/tests.rs` owns edit line-box and
presentation coverage. NumericInput public options, default selection behavior, validation message
routing, density-derived edit line boxes, and NumericPresentation adoption remain unchanged.

2026-05-31 editor drag-value tests child-owner result:
`ecosystem/fret-ui-editor/src/controls/drag_value.rs` now keeps drag-value control orchestration and
test-owner routing. `controls/drag_value/tests.rs` owns presentation format/parse/chrome-affix
coverage. `DragValue::from_presentation`, NumericPresentation adoption, duplicate chrome affix
suppression, scrub/typing behavior, and value text-role routing remain unchanged.

2026-05-30 textarea element and props child-owner result:
`ecosystem/fret-ui-kit/src/imui/text_controls/textarea.rs` now only owns the public wrapper and
`ResponseExt` plumbing. `text_controls/textarea/element.rs` owns lifecycle, select-all, policy
commands, and element mounting, while `text_controls/textarea/props.rs` owns `TextAreaProps` and
IMUI chrome/style resolution. Facade calls, enabled gating, focus tracking, submit command
behavior, and textarea layout semantics remain unchanged.

2026-05-30 slider entry element child-owner result:
`ecosystem/fret-ui-kit/src/imui/slider_controls/entry.rs` now owns label identity parsing,
visible-label suffix stripping, and scoped facade routing only.
`slider_controls/entry/element.rs` owns slider element construction, response population,
interaction installation, chrome resolution, and visual child mounting. Slider facade calls,
push-id scoping, enabled/disabled gating, a11y range semantics, pointer/keyboard behavior, and
response lifecycle reporting remain unchanged.

2026-05-30 virtual-list rendered-range child-owner result:
`ecosystem/fret-ui-kit/src/imui/virtual_list_controls.rs` now keeps virtual-list element assembly,
row wrapping, build-focus forwarding, runtime option resolution, and response packaging.
`virtual_list_controls/range.rs` owns first/last rendered index tracking and rendered-range
projection. Row height resolution, row test IDs, clipping semantics, and public
`VirtualListResponse` reporting remain unchanged.

2026-05-30 porting-sugar scoped layout child-owner result:
`ecosystem/fret-ui-kit/src/imui/layout_sugar/scoped.rs` is now a private hub.
`scoped/flow.rs` owns `items` and `same_line` container routing, while `scoped/indent.rs` owns
indent spacer/content composition. Item-spacing token behavior, dummy spacer sizing, content test
IDs, focus forwarding, and public porting-sugar facade behavior remain unchanged.

2026-05-30 floating-window closed response child-owner result:
`ecosystem/fret-ui-kit/src/imui/floating_window.rs` now keeps open-model reads and normal
floating-area render routing. `floating_window/closed.rs` owns the open=false sentinel response,
including the zero area id, initial position/size preservation, and inactive
`FloatingWindowResponse` flags. Normal floating-area routing, on-area chrome rendering, and public
window response behavior remain unchanged.

2026-05-30 menu item interaction parts child-owner result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` now keeps enabled/action gating,
menubar policy capture, and thin behavior forwarding. `interaction/parts.rs` owns
`MenuItemInteractionParts`, `MenuItemInteraction`, pressable prop/a11y construction, and runtime
data packaging. Menu item enabled/action gating, close-popup/action behavior, keyboard wiring,
active-trigger installation, and response semantics remain unchanged.

2026-05-30 floating-area drag snapshot child-owner result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/area/drag_state.rs` now keeps position/test-id
state reconciliation, scale-factor lookup, device-pixel snapping, and final state readback.
`drag_state/snapshot.rs` owns active drag lookup, same-window drag filtering, and drag snapshot
projection. Floating-area drag movement, snapping, test-id refresh, and `FloatingAreaResponse`
movement semantics remain unchanged.

2026-05-30 button visual content child-owner result:
`ecosystem/fret-ui-kit/src/imui/button_controls/visual.rs` now keeps `ButtonVisual`, chrome
resolution, and visible/invisible selection only. `button_controls/visual/content.rs` owns
`ButtonVisualContent`, foreground-aware centered text child construction, and empty invisible-button
content. Button chrome resolution, variant sizing, arrow glyph labels, centered text mounting, and
button response behavior remain unchanged.

2026-05-30 child-region resize handle child-owner result:
`ecosystem/fret-ui-kit/src/imui/child_region/resize/handle.rs` now keeps the pointer-region handle
element assembly, axis layout application, and handle test-id stamping. `handle/events.rs` owns the
pointer down/move/up drag callbacks, cursor request, thresholded drag movement, and pointer release
finish call. `handle/drag_state.rs` owns `ResponseExt` population plus started/stopped drag edge
tracking. Resize handle layout, enabled gating, threshold/cursor behavior, and
`ChildRegionResponse` resize drag semantics remain unchanged.

2026-05-30 floating-window on-area state child-owner result:
`ecosystem/fret-ui-kit/src/imui/floating_window_on_area/state.rs` now keeps the on-area state
preparation flow, resize snapshot/prepare calls, scale-factor lookup, and chrome response assembly.
`state/collapsed.rs` owns collapsed-model toggle/readback, while `state/position.rs` owns area
position feedback after resize. Collapsed toggles, resize state preparation, area position feedback,
and `FloatingWindowChromeResponse` semantics remain unchanged.

2026-05-30 interaction-runtime element-model child-owner result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/models/element.rs` is now a private
module/re-export hub. `element/context_menu.rs` owns context-menu anchor models,
`element/press.rs` owns long-press signal and pointer-click modifier models,
`element/lifecycle.rs` owns lifecycle session models, and `element/floating.rs` owns floating-window
collapsed models. Public interaction-runtime re-exports and model identity semantics remain
unchanged.

2026-05-30 table-column visibility menu-items child-owner result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/menu.rs` now keeps header context-menu
trigger selection and popup orchestration. `menu/items.rs` owns repeated column-menu item
composition, generated item test IDs, runtime visibility reads, and `TableColumnVisibilityMenuResponse`
aggregation, while `menu/item.rs` still owns the single checkbox item mutation. Public helper
forwarding, model updates, filtering, and header context-menu response semantics remain unchanged.

2026-05-30 input-text element child-owner result:
`ecosystem/fret-ui-kit/src/imui/text_controls/input.rs` now keeps the public input-text wrapper,
assistive-semantics re-export, and shared model-changed helper. `input/element.rs` owns
ElementContext element assembly, response lifecycle population, select-all-on-focus command
emission, input-text props mounting, and policy-command installation. Input-text facade calls,
picker assistive semantics, filters, compact chrome/style, and changed/focus response semantics
remain unchanged.

2026-05-30 submenu clear reset child-owner result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state/clear/reset.rs` is now a
private reset module/re-export hub. `reset/active.rs` owns active submenu value/trigger/geometry
clearing, `reset/pending.rs` owns pending-open value/trigger cleanup, and `reset/runtime.rs` owns
pointer-grace, close/focus/open timer, focus target, and focus retry reset cleanup. Submenu hover,
shortcut, sibling-switch, and close semantics remain unchanged.

2026-05-30 begin-menu open-policy child-owner result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/open_policy.rs` is now a private
module/re-export hub. `open_policy/toggle.rs` owns trigger-click menubar/popup toggling,
`open_policy/resolve.rs` owns open-request resolution and stale row/popup close cleanup, and
`open_policy/disabled.rs` owns disabled-popup close cleanup. Menubar open-menu synchronization,
active-trigger behavior, popup open/close semantics, and `DisclosureResponse` reporting remain
unchanged.

2026-05-30 table header-row cells child-owner result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header_row.rs` now keeps the keyed header row and
row wrapping only. `header_row/cells.rs` owns visible-header-cell assembly, sortable/plain wrapper
selection, resize response initialization, `TableHeaderResponse` collection, and prepared-cell
projection. Header visibility, sort/resize metadata, header test IDs, pinned/horizontal-scroll
wrapping, and aggregate `TableResponse` semantics remain unchanged.

2026-05-30 floating-window resize state commit output-pack result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/commit.rs` now keeps resize state
lookup, collapsed/non-drag reset, drag application, and device-pixel snapping. The new
`state/commit/output_pack.rs` owns committed state capture, handle test-id packaging, and
`FloatingWindowResizeStateOutput` construction. Resize handle IDs, size/position output, and active
`resizing` semantics remain unchanged.

2026-05-30 debug-draw path-builder arc child-owner result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/path_builder/shape_methods/arcs.rs` is now a
private module hub. `arcs/circular.rs` owns `arc_to` and `arc_to_fast`, while
`arcs/elliptical.rs` owns `elliptical_arc_to`. Method names, invalid-input handling, default
segment fallback, 12-step fast arc behavior, elliptical rotation handling, and path point storage
remain unchanged.

2026-05-30 debug-draw rounded rect child-owner result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths/rects/rounded.rs` now keeps only
rounded-rect point append orchestration. `rounded/corners.rs` owns per-corner rounding selection
and corner arc sampling, while `rounded/geometry.rs` owns rect max-point calculation. Effective
rounding clamp, fallback square points, corner sample order, and path-builder behavior remain
unchanged.

2026-05-30 debug-draw filled polygon child-owner result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled/polygons.rs` is now
a private re-export hub. `polygons/multi.rs` owns convex and concave polygon fill painting, while
`polygons/primitives.rs` owns quad and triangle fill painting plus degenerate-triangle filtering.
Filled path command generation, shared fill style, canvas path dispatch, and draw-list behavior
remain unchanged.

2026-05-30 debug-draw media dispatch child-owner result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media/dispatch.rs` is now a private
dispatch hub for media-command routing. `dispatch/raster_commands.rs` owns image/image-region/
image-quad routing, `dispatch/rounded_commands.rs` owns rounded image/region routing,
`dispatch/svg_commands.rs` owns SVG image/mask-icon routing, and `dispatch/non_media.rs` keeps an
exhaustive non-media no-op guard. Raster, rounded, and SVG paint behavior remains in the existing
paint owners.

2026-05-30 button root wrapper child-owner result:
`ecosystem/fret-ui-kit/src/imui/button_controls.rs` is now a private re-export hub for
public-in-IMUI button wrappers. `button_controls/plain.rs` owns button, small-button, arrow, and
invisible-button wrapper routing; `button_controls/actions.rs` owns action and payload-action
wrapper routing. Variant selection, push-id scoping, action payload forwarding, behavior dispatch,
and response projection remain unchanged.

2026-05-30 popup-modal layer child-owner result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layer.rs` now keeps layer input/output,
root-name mounting, stack wiring, and panel-focus handoff. `layer/backdrop.rs` owns modal barrier
construction and outside-press dismissal plumbing, while `layer/panel.rs` owns panel semantics,
child `ImUiFacade` mounting, and panel id capture. Modal root naming, layer stack layout, panel
semantics, focus handoff, and public popup modal facade behavior remain unchanged.

2026-06-03 popup-modal layer carrier owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layer.rs` now keeps modal root/backdrop/panel
assembly only. `layer/focus.rs` owns the modal focus-state factory, and `layer/types.rs` owns modal
layer input/output carrier records with visibility limited to the modal subtree. Modal root naming,
layer stack layout, panel semantics, focus handoff, and public popup modal facade behavior remain
unchanged.

2026-05-30 disclosure entry child-owner result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/entry.rs` now keeps public
collapsing-header/tree-node wrappers, label identity normalization, and aggregate
`DisclosureResponse` assembly. `entry/state.rs` owns collapsible open-model setup, open reads,
toggled detection, and enabled gating; `entry/body.rs` owns trigger/content child construction.
Public disclosure facade calls, root layout, trigger/content mounting, and response semantics remain
unchanged.

2026-05-29 facade root surface owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer.rs` now keeps the single public
`UiWriterImUiFacadeExt` trait hub plus surface macro expansion only. Scope, basic text/separator
and debug-draw, and disclosure/tree trait default method declarations now live in
`facade_writer/scope_surface.rs`, `facade_writer/basic_surface.rs`, and
`facade_writer/disclosure_surface.rs`. Existing `scope_methods.rs`, `basic_items.rs`, and
`disclosure_controls` remain the behavior owners.

2026-05-29 facade basic surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/basic_surface.rs` is now a module/re-export hub.
`basic_surface/text.rs` owns text, wrapped text, and bullet text trait forwarding;
`basic_surface/debug_draw.rs` owns debug-draw trait forwarding; and
`basic_surface/separators.rs` owns separator and separator-text trait forwarding. Public trait
method names, default option forwarding, macro expansion order, response returns, and concrete
`basic_items` behavior ownership remain unchanged.

2026-05-29 facade disclosure surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/disclosure_surface.rs` is now a module/re-export
hub. `disclosure_surface/collapsing_header.rs` owns collapsing-header trait forwarding, while
`disclosure_surface/tree_node.rs` owns tree-node trait forwarding and explicit depth guidance.
Public trait method names, stable identity docs, response returns, macro expansion order, and
concrete `disclosure_controls` behavior ownership remain unchanged.

2026-05-29 facade support sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_support.rs` is now a module/re-export hub.
`facade_support/constants.rs` owns IMUI key and timing constants, `geometry.rs` owns point and
device-pixel helpers, `runtime.rs` owns frame preparation, `state.rs` owns model-change
tracking, and `ui_writer.rs` owns the `UiWriterUiKitExt` bridge trait implementation. Public
IMUI key names, frame preparation, geometry helpers, model-change behavior, and
`UiWriterUiKitExt` re-export paths remain unchanged.

2026-05-29 facade scope method sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/scope_methods.rs` is now a module/re-export hub.
`scope_methods/push_id.rs` owns keyed child facade execution and result propagation, while
`scope_methods/disabled_scope.rs` owns disabled-scope wrapping, disabled alpha, pointer blocking,
and focus traversal gating. Public facade method names, `scope_surface` forwarding, runtime frame
preparation, keyed identity semantics, and disabled-scope behavior remain unchanged.

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

2026-05-29 facade container collection surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface/collections.rs` is now a
module/re-export hub. `collections/list_box.rs` owns ListBox trait forwarding,
`collections/grid.rs` owns grid trait forwarding, `collections/table.rs` owns Table trait
forwarding and response returns, and `collections/virtual_list.rs` owns VirtualList trait
forwarding and response returns. Public trait method names, macro expansion order, and concrete
`container_methods/*` behavior owners remain unchanged.

2026-05-29 facade container layout surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_surface/layout.rs` is now a
module/re-export hub. `layout/flow.rs` owns item-flow, same-line, dummy, spacing, and indent trait
forwarding, while `layout/groups.rs` owns horizontal and vertical group trait forwarding. Public
trait method names, macro expansion order, and concrete `container_methods/*` behavior owners
remain unchanged.

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

2026-05-29 facade container collection wrapper sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/container_wrappers/collections.rs` is now a module
hub. `collections/list_box.rs` owns inherent ListBox label/options wrappers,
`collections/table.rs` owns inherent Table wrappers and response returns, and
`collections/virtual_list.rs` owns inherent VirtualList wrappers and response returns. Public
inherent method names, build-focus forwarding, and `container_methods/*` delegation remain
unchanged.

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

2026-05-29 facade floating tooltip/drag surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/tooltip_drag.rs` is now a
module/re-export hub. `tooltip_drag/tooltip.rs` owns tooltip text and custom-content forwarding,
while `tooltip_drag/drag_drop.rs` owns typed drag source/drop target forwarding and docs. Public
trait method names, tooltip behavior, drag/drop behavior, and concrete `floating_popup/*` behavior
owners remain unchanged.

2026-05-29 facade floating popup surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/floating_surface/popup.rs` is now a module/re-export
hub. `floating_surface/popup/area.rs` owns floating layer, floating area, and area drag-surface
forwarding; `floating_surface/popup/state.rs` owns popup open-model, drop, open, anchor-open, and
close forwarding; and `floating_surface/popup/begin.rs` owns popup menu/modal begin forwarding.
The public trait expansion points in `facade_writer.rs` now call these child macros directly,
while concrete `floating_popup/*` behavior owners remain unchanged.

2026-05-29 facade floating-popup popup behavior sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup/popup.rs` is now a module/re-export
hub. `floating_popup/popup/state.rs` owns popup open-model, drop, open, anchor-open, and close
forwarding to `popup_overlay`, while `floating_popup/popup/begin.rs` owns popup menu, modal, and
context-menu begin forwarding. Public facade method names, popup state/begin behavior, and
`floating_popup.rs` re-export paths remain unchanged.

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

2026-05-29 facade menu/selection selection-combo surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/menu_selection_surface/selection_combo.rs` is now a
module/re-export hub. `selection_combo/selectables.rs` owns selectable and multi-selectable trait
forwarding, while `selection_combo/combo.rs` owns combo trait forwarding. Public trait method
names, default option forwarding, macro expansion order, response returns, focusable-recording
inherent wrapper owners, and concrete selectable/combo behavior owners remain unchanged.

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

2026-05-29 facade text model surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/model_surface/text.rs` is now a module/re-export hub.
`text/input.rs` owns input-text model trait forwarding, `text/picker.rs` owns completion/history
picker trait forwarding, and `text/textarea.rs` owns textarea trait forwarding. Public trait method
names, default option forwarding, macro expansion order, response returns, focusable-recording
inherent wrapper owners, and concrete `text_controls` / `text_picker_controls` behavior owners
remain unchanged.

2026-05-29 facade boolean-control inherent-wrapper sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/boolean_wrappers.rs` is now a module hub.
`boolean_wrappers/checkbox.rs` owns checkbox model inherent wrappers,
`boolean_wrappers/radio.rs` owns radio inherent wrappers, and `boolean_wrappers/switch.rs` owns
switch model inherent wrappers. Disabled checks, focusable recording, trait delegation paths, public
inherent method names, and the `fret-imui` thin boundary remain unchanged.

2026-05-29 facade value/combo-model inherent-wrapper sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/value_models.rs` is now a module hub.
`value_models/slider.rs` owns slider model inherent wrappers, and
`value_models/combo_model.rs` owns combo-model inherent wrappers. Disabled checks, focusable
recording, trait delegation paths, public inherent method names, and the `fret-imui` thin boundary
remain unchanged.

2026-05-29 facade text-model inherent-wrapper sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/text_models.rs` is now a module hub.
`text_models/input.rs` owns single-line input inherent wrappers, `text_models/picker.rs` owns
completion/history picker inherent wrappers, and `text_models/textarea.rs` owns textarea inherent
wrappers. Disabled/focusable checks, picker focusable calculation, trait delegation paths, public
inherent method names, and the `fret-imui` thin boundary remain unchanged.

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

2026-05-29 facade button action surface sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_surface/actions.rs` is now a module/re-export
hub. `actions/action.rs` owns action button trait forwarding, `actions/payload.rs` owns
payload-action button trait forwarding and payload bounds, and `actions/command.rs` owns
command-button trait forwarding. Public trait method names, default option forwarding, macro
expansion order, command presentation forwarding, focusable-recording inherent wrapper owners, and
concrete button/action behavior owners remain unchanged.

2026-05-29 facade button/action inherent-wrapper sub-owner result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions.rs` now keeps module wiring and the
private button-command helper re-export only. `button_actions/buttons.rs` owns
plain/small/arrow/invisible button inherent wrappers, and `button_actions/commands.rs` owns
button-command inherent wrappers. Existing `action_methods.rs` and `button_command.rs` now import
directly from the facade parent instead of relying on root hub imports. Public inherent method
names, focusable recording, command metadata lookup, action/payload action wrappers, and the
`fret-imui` thin boundary remain unchanged.

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

2026-05-30 editor theme patch owner-split result:
`ecosystem/fret-ui-editor/src/theme.rs` keeps public preset metadata plus install/replay and host
theme sync APIs. `ecosystem/fret-ui-editor/src/theme/patches.rs` owns default and ImGui-like dense
token patch construction without changing preset keys, labels, or token values.

2026-05-30 color-edit hue-wheel canvas owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs` keeps picker composition,
pointer interactions, and exported picker preview entrypoints. `picker/hue_wheel.rs` owns the
hue-wheel canvas painting and local geometry helpers used by the popup and option thumbnails.

2026-05-30 color-edit alpha bar owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs` keeps color picker composition
and alpha entrypoint re-exports. `picker/alpha.rs` owns horizontal and vertical alpha bar
previews, gradient overlays, thumb overlays, pointer update application, and alpha coordinate/a11y
helper math.

2026-05-31 color-edit alpha preview child-owner result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker/alpha.rs` now keeps horizontal and
vertical bar pressable interaction, model/draft/error mutation, and alpha helper math.
`picker/alpha/preview.rs` owns preview stacks, checkerboard-backed alpha gradients, and
horizontal/vertical thumb overlays. Horizontal/vertical alpha bars, pointer mutation, alpha
coordinate mapping, checkerboard/gradient/thumb visuals, and picker tests remain unchanged.

2026-05-30 color-edit hue bar owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs` keeps color picker composition
and shared HSV color application. `picker/hue_bar.rs` owns hue bar previews, vertical hue
gradient construction, thumb overlays, pointer update application, and hue coordinate helper
wiring.

2026-05-30 color-edit saturation/value picker owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs` keeps color picker composition
and shared HSV color application. `picker/sv.rs` owns SV preview grid construction, thumb overlay
layout, pointer update application, and SV coordinate helper wiring.

2026-05-30 color-edit hue-wheel interaction owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/picker.rs` keeps color picker composition
and shared HSV color application. `picker/hue_wheel.rs` remains the pure canvas painting owner,
while `picker/hue_wheel_picker.rs` owns hue-wheel pressable drag target tracking and HSV update
wiring.

2026-05-30 color-edit options owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` keeps public re-exports, payload/request
records, the main control renderer, and shared local models. `controls/color_edit/options.rs` owns
option records, default construction, popup runtime defaults, and runtime sync semantics.

2026-05-30 color-edit records owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` keeps public record re-exports, the main
control renderer, and shared local models. `controls/color_edit/records.rs` owns default palette
data, palette entries, typed drag/drop payload records, palette slot drop requests, and eyedropper
request/callback records.

2026-05-30 color-edit state owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` keeps public re-exports and the main control
renderer. `controls/color_edit/state.rs` owns popup, tooltip, copy-menu, reference, draft, error,
and popup runtime option local models plus runtime default sync.

2026-05-30 color-edit input owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` keeps the main control renderer and passes
input arguments into `controls/color_edit/input.rs`. The input owner owns text input props,
text-field chrome resolution, draft sync, Enter/Escape handling, parse/reset errors, and pointer
focus wrapping.

2026-05-30 color-edit swatch owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` keeps popup requests and delivered drop
application. `controls/color_edit/swatch.rs` owns the swatch pressable, activation/reference
capture, copy-menu triggers, drag source/drop hover state, tooltip hover synchronization, preview
container, frame visuals, and swatch style resolution.

2026-05-30 color-edit delivered-drop owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` passes swatch id and model context into the
drag/drop owner. `controls/color_edit/drag_drop.rs` owns delivered payload extraction,
alpha-aware payload application, formatted draft synchronization, and error clearing.

2026-05-30 color-edit layout owner-split result:
`ecosystem/fret-ui-editor/src/controls/color_edit.rs` remains the state/owner orchestration hub.
`controls/color_edit/layout.rs` owns error text rendering, root min-height fallback, vertical root
layout, horizontal swatch/input row layout, and root test-id assignment.

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
shape, focus recording, and `UiWriter` implementation; the 2026-06-03 identity follow-up moves
keyed id helpers to `facade_core/identity.rs`.

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

2026-06-03 debug-draw round path paint primitive owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/round.rs` is now a
private re-export hub over circle, ngon, and ellipse stroked round path paint branches.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/filled/round.rs` is now a
private re-export hub over circle, ngon, and ellipse filled round path paint branches. The six
child owners keep the same path command generation, stroke/fill style dispatch, canvas path
dispatch, and debug-draw behavior.

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

2026-06-03 debug-draw stroked rect/quad/triangle path paint child owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint_shapes/paths/stroked/linear/rect_quad_triangle.rs`
is now a private re-export hub only. `rect.rs`, `quad.rs`, and `triangle.rs` own the three concrete
stroked linear path paint branches, preserving path command generation, culling checks, stroke style
dispatch, and canvas path dispatch.

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

2026-05-30 table header resize props/behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header/resize/props.rs` now owns pointer-region
sizing and enabled props. `header/resize/behavior.rs` owns pointer down/move/up hooks, cursor
behavior, and resize drag response edge merging. `header/resize.rs` keeps column identity, keyed
shell, visual mounting, and test-id attachment.

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
kept trigger gates, interaction updates, open state writeback, and overlay request submission until
the later runtime-interaction split.

2026-05-30 tooltip runtime interaction owner-split result:
`ecosystem/fret-ui-kit/src/imui/tooltip_overlay/runtime/interaction.rs` now owns trigger
hover/focus gating, `TooltipInteractionConfig` construction, continuous-frame scheduling, and open
model synchronization. `tooltip_overlay/runtime.rs` keeps trigger-id validation, runtime model
creation, pointer-move gate installation, layout resolution, and overlay request submission.

2026-05-28 button-command helper owner-split result:
`ecosystem/fret-ui-kit/src/imui/facade_writer/button_actions/button_command.rs` now owns command
presentation lookup and enabled gating. `button_actions.rs` keeps the public button wrappers and
the private helper re-export.

2026-05-28 pressable drag state-machine owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/pressable.rs` now owns pressable pointer
down/move/up drag state transitions, long-press timer coordination, active item cleanup, and
drag-started/stopped transients. `interaction_runtime/drag.rs` keeps drag kind/threshold helpers
and private sub-owner re-exports.

2026-05-30 pressable drag phase child-owner split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/drag/pressable.rs` is now a private phase hub.
`pressable/down.rs` owns pointer-down active-item/timer/drag begin setup, `pressable/move_phase.rs`
owns thresholded move transitions and drag started/stopped transients, and `pressable/up.rs` owns
pointer-up active-item/timer cleanup and drag cancelation. Drag kind/threshold helpers and public
response drag state remain unchanged.

2026-05-28 drag-source payload lifecycle owner-split result:
`ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks/payload_lifecycle.rs` now owns pointer-move
active payload tracking, hovered-target preservation, and pointer-up delivery insertion.
`drag_drop/source/hooks.rs` keeps enabled gating, cross-window drag upgrade policy, and the private
payload-lifecycle delegation.

2026-05-30 drag-source payload lifecycle child-owner split result:
`ecosystem/fret-ui-kit/src/imui/drag_drop/source/hooks/payload_lifecycle.rs` is now a private hook
installer hub. `payload_lifecycle/move_hook.rs` owns active payload tracking and hovered-target
preservation, while `payload_lifecycle/up_delivery.rs` owns pointer-up target resolution and
delivered payload insertion. Cross-window drag upgrade policy and public drag/drop response
behavior remain unchanged.

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
open-menu model reads for begin-menu capture/open-policy. `capture.rs` kept `BeginMenuState`,
`MenuRenderState`, row/popup/was-open model identity, render-state writeback, and read facade
methods until the later 2026-05-30 state-carrier split moved those state bodies out.

2026-05-30 begin-menu capture state-carrier owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_state/capture/state.rs` now owns
`BeginMenuState`, `MenuRenderState`, row/open-menu read facade methods, and
`record_render_state(...)`. `capture.rs` keeps begin-menu model capture and state assembly.

2026-05-28 table builder test-id owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/builder/test_ids.rs` now owns row/cell test-id
derivation, including explicit row test-id override fallback and default `.row.*` / `.cell.*`
strings. `builder.rs` keeps public `ImUiTable` / `ImUiTableRow` methods, row/cell collection,
keyed row scopes, child mounting, and table render handoff.

2026-05-31 table-control test owner split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/tests.rs` now keeps shared table test helpers and
module routing only. `tests/header_text.rs` owns header label/sort indicator text-role coverage,
while `tests/rendering.rs` owns hidden-column header/body filtering, response filtering, and
horizontal-scroll wrapping coverage.

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

2026-05-30 disclosure trigger hook-family owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/trigger/behavior.rs` now keeps hook clearing,
context-menu anchor model handoff, installation order, and response-owner dispatch only.
`behavior/activation.rs` owns activate-click toggling, `behavior/keyboard.rs` owns activate
shortcuts plus ContextMenu/Shift+F10 requests, and `behavior/pointer.rs` owns right-click anchor
capture plus double-click transient signaling.

2026-05-31 disclosure control test owner split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/tests.rs` now keeps the shared test harness and
module routing only. `tests/entry.rs` owns collapsing-header body mounting coverage,
`tests/tree.rs` owns tree-node semantics/default coverage, and `tests/visual.rs` owns hover palette
plus tree-row/indicator text-role coverage.

2026-05-28 slider pointer value-update owner-split result:
`ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer/value_update.rs` now owns
pointer-to-value projection, clamp/snap, and changed-detection writes. `pointer.rs` keeps
pointer hook installation, active-item updates, capture/release, focus, lifecycle activation/
deactivation, and transient change emission.

2026-05-30 slider pointer hook sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/slider_controls/interaction/pointer.rs` now keeps model clone and
hook installation order only. `pointer/down.rs` owns left-button activation, capture/focus, active
item writes, initial value update, and changed transient emission. `pointer/move_handler.rs` owns
drag value updates plus lost-left-button cleanup. `pointer/up.rs` owns release/deactivation cleanup.

2026-05-28 combo trigger visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/visual.rs` now owns ComboBox trigger props,
field chrome lookup, visual children assembly, and the a11y label helper. `trigger.rs` keeps
behavior installation and visual-owner dispatch, while public combo behavior remains unchanged.

2026-05-30 combo trigger visual sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/visual.rs` is now the chrome/re-export hub
for ComboBox trigger visuals. `trigger/visual/props.rs` owns `PressableProps` construction and
a11y label derivation, while `trigger/visual/children.rs` owns the label/preview row and Open/Menu
state badge assembly. Public combo behavior remains unchanged.

2026-05-30 combo trigger behavior sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger/behavior.rs` now keeps behavior input
normalization, shared pressable item behavior installation, and owner dispatch only.
`trigger/behavior/activation.rs` owns activate click recording and keyboard lifecycle marking,
`trigger/behavior/keyboard.rs` owns activate shortcuts plus ContextMenu/Shift+F10 requests, and
`trigger/behavior/response.rs` owns pressable trigger response projection.

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
reads, candidate visibility, input-root mounting, open-policy application, and popup rendering; the
2026-06-01 follow-up moved popup-result finalization and picked-change response merging into
`text_picker_controls/response.rs`, then moved popup request/render dispatch into
`text_picker_controls/core/popup.rs`.

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

2026-05-30 submenu clear reset owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu_state/clear.rs` keeps the
public-in-menu-family clear flow. `submenu_state/clear/reset.rs` owns active submenu, pending
submenu, and runtime pointer-grace/focus/timer model resets.

2026-05-28 menu keyboard owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard.rs` is now a private module/re-export index.
`keyboard/popup.rs` owns popup menu item registration, keyboard shortcut activation,
popup-close-on-key activation, and Arrow/Home/End item focus movement. `keyboard/menubar.rs` owns
menubar horizontal-arrow close-focus suppression and primitive trigger-row horizontal switching
wiring.

2026-05-30 popup menu keyboard sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/keyboard/popup.rs` now keeps only the popup key-handler
composition point. `keyboard/popup/shortcut.rs` owns activate-shortcut repeat/IME gating, lifecycle
instant marking, popup close, clicked transient emission, and action dispatch. `keyboard/popup/nav.rs`
owns popup menu nav item registration and Arrow/Home/End roving focus movement.

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

2026-05-31 child-region resize response tests child-owner result:
`ecosystem/fret-ui-kit/src/imui/response/widgets/child_region/resize/x.rs` and `resize/y.rs` now
keep response projection plus test-owner routing. `resize/x/tests.rs` owns width clamp coverage,
while `resize/y/tests.rs` owns height clamp coverage. Public resize response re-exports,
enabled/min/max accessors, drag delta/total projection, and clamp-from-start math remain unchanged.

2026-05-28 input-text filter owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/text/filters.rs` is now a private
module/re-export index. `filters/builtin.rs` owns `InputTextFilters` plus
decimal/scientific/hex/uppercase/no-blank character filtering, and `filters/custom.rs` owns
`InputTextCustomFilter` closure storage and debug output.

2026-05-30 input-text built-in filter application owner-split result:
`ecosystem/fret-ui-kit/src/imui/options/controls/text/filters/builtin.rs` now owns
`InputTextFilters` storage and constructors. `filters/builtin/filtering.rs` owns
`filter_text(...)`, per-character filtering, and decimal/scientific character classifiers.

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
`hover/shared_delay.rs` kept hover-enter/leave shared timer policy and clear-timer handling until
the later 2026-05-30 hover-change/timer sub-owner split moved that event policy out.

2026-05-30 shared hover-delay event-policy sub-owner result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay/hover_change.rs` now owns
hover-enter/leave shared timer scheduling and clear-timer cancellation.
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/shared_delay/timer.rs` now owns
short/normal/clear timer consumption, delay-flag updates, pending timer cancellation, and notify
behavior. `shared_delay.rs` is now a private module/re-export hub.

2026-05-28 hover query delay read owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/read.rs` now owns local hover-delay
state, transient consumption, shared-delay flag reads, and `HoverQueryDelayRead` projection.
`interaction_runtime/hover.rs` keeps active-item blocking, hover-change hook installation, timer
dispatch, shared-delay delegation, and long-press delegation.

2026-05-31 hovered query pointer/delay owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/query.rs` keeps the public `hovered_like_imgui` /
`is_hovered` API and tooltip flag expansion. `response/hover/query/pointer.rs` owns nav override,
disabled-item, popup-barrier underlay, and active-item pointer gating.
`response/hover/query/delay.rs` owns stationary, short/normal delay, and shared-delay query
gating. The Dear ImGui-style `ImUiHoveredFlags` semantics remain unchanged.

2026-05-30 hover active-block/hooks child-owner split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/active_block.rs` now owns active-item
blocking reads. `interaction_runtime/hover/hooks.rs` owns hover-change and timer hook
installation, stationary/short/normal delay timers, shared-delay delegation, and long-press
delegation. `interaction_runtime/hover.rs` is now a private module/re-export hub.

2026-05-31 hover hook child-owner split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/hover/hooks.rs` now orchestrates shared-delay
model lookup, child hook installation, and delay reads only.
`interaction_runtime/hover/hook_hover_change.rs` owns pressable hover-change timer
setup/cancellation. `interaction_runtime/hover/hook_timer.rs` owns local hover-delay timer
dispatch, shared-delay timer delegation, and long-press timer delegation. Stationary/short/normal
hover timing, shared-delay behavior, and `HoverQueryDelayRead` projection remain unchanged.

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

2026-05-30 text-picker keyboard handler sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/keyboard/handler.rs` now keeps key-down
capture, repeat/IME/modifier gating, and key dispatch. `keyboard/handler/navigation.rs` owns Arrow
highlight movement through the cmdk active-index helper. `keyboard/handler/pick.rs` owns
Enter/NumpadEnter highlighted candidate commit, input model writes, popup close, pending pick
storage, and redraw.

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

2026-05-30 interaction lifecycle mutation owner-split result:
`ecosystem/fret-ui-kit/src/imui/interaction_runtime/lifecycle.rs` is now a mutation/response
re-export hub. `lifecycle/pointer_edges.rs` owns pointer down/up lifecycle edges, `lifecycle/edit.rs`
owns edit marking, and `lifecycle/instant.rs` owns inactive instant lifecycle emission.

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
registration, activation dispatch, layer child mounting, and rank sort application.

2026-05-30 floating layer layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/layer/layout.rs` now owns the absolute fill
visible-overflow layer shell and root id stamping. `floating_surface/layer.rs` now keeps marker
state, child registration, activation dispatch, child mounting, and z-order snapshot
reconciliation before delegating rank sort and layout.

2026-05-30 floating layer sort owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/layer/sort.rs` now owns z-order rank lookup,
unknown-rank fallback, and original-index stable tie-break sorting. `floating_surface/layer.rs`
keeps marker state, child registration, activation dispatch, child mounting, and z-order snapshot
reconciliation.

2026-05-30 floating resize handle edge/corner layout owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/layout.rs` now only dispatches
resize handle layout by handle family. `floating_window_resize/handles/layout/edge.rs` owns the
four 6 px edge handles and `floating_window_resize/handles/layout/corner.rs` owns the four 10 px
corner handles. `floating_window_resize/handles/pointer.rs` still composes layout, cursor, pointer
capture, and activation behavior without public IMUI API changes.

2026-05-30 floating-window resize handle pointer-events owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/pointer.rs` now owns
element/layout/cursor composition and bring-to-front handoff. `handles/pointer/events.rs` owns
pointer hook clearing, down/move/up callbacks, runtime drag begin/update/cancel, pointer capture,
cursor updates, and resize-handle activation events.

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

2026-05-30 tab-list child-owner split result:
`ecosystem/fret-ui-kit/src/imui/tab_family_controls/items/list.rs` is now a private tab-list hub.
`list/triggers.rs` owns trigger rendering, selected/first-focusable trigger tracking, and
`TabTriggerResponse` collection. `list/element.rs` owns tab-list semantics/test id, root row
layout, and h-flex trigger composition. Public tab-bar APIs and response semantics remain
unchanged.

2026-05-28 text-picker core owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core.rs` now owns input-text picker
orchestration: model reads, candidate visibility, keyboard snapshot reconciliation, input root
mounting, open-policy application, popup rendering, and initially pick response merging. The
2026-06-01 follow-up moved popup-result finalization and picked-change response merging into
`text_picker_controls/response.rs`, then moved popup request/render dispatch into
`text_picker_controls/core/popup.rs`.
`text_picker_controls.rs` is now a private module index and re-export hub for the core picker and
completion/history entry wrappers.

2026-05-30 text-picker session preparation owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/core/session.rs` now owns model reads,
candidate visibility, popup-open model lookup, enabled-scope checks, keyboard snapshot
reconciliation, popup snapshot reads, and `picker_expanded` derivation. `core.rs` keeps input-root
mounting, open-policy application, and popup rendering; the 2026-06-01 follow-up moved
popup-result finalization and picked-change response merging into `text_picker_controls/response.rs`,
then moved popup request/render dispatch into `text_picker_controls/core/popup.rs`.

2026-05-27 table header-cell owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header/cell.rs` now owns header cell layout,
resize-handle attachment, resize test-id suffixing, and header content flex wrapping.
`table_controls/header.rs` keeps sortable/plain header trigger orchestration and `BuiltHeaderCell`
response assembly.

2026-05-30 table header sortable/plain sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/header.rs` is now the labels/cell/trigger re-export
hub plus the `BuiltHeaderCell` response record. `header/sortable.rs` owns sortable-header trigger
assembly and sort visual wiring, while `header/plain.rs` owns plain-header fallback labels and
content-box assembly. Header trigger behavior, resize wrapping, table response collection, and
public table facade behavior remain unchanged.

2026-05-27 debug-draw path-family owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paths.rs` is now a private path-family
re-export hub. `paths/linear.rs` now indexes polyline/fill/primitive subowners; `paths/round.rs`
now indexes circle/ngon/ellipse subowners; `paths/beziers.rs` owns quadratic and cubic bezier path
construction. The 2026-05-28 follow-ups split the linear and round families into
`paths/linear/{polyline,fills,primitives}.rs` and
`paths/round/{circle,ngon,ellipse}.rs`.

2026-05-27 debug-draw command payload owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types.rs` is now a private command-type re-export hub.
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/types/command.rs` owns
the private `DebugDrawCommand` payload enum and all draw-list command variants.
`debug_draw_controls/commands.rs` keeps summary projection wiring plus the parent-visible command
re-export.

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
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/layout.rs` now owns body `ImUiFacade`
construction, root/content composition, and content/root test-id application. The 2026-05-31
follow-up moved content/root props into `disclosure_controls/layout/props.rs`.
`disclosure_controls.rs` keeps label identity parsing, open-model reads, trigger mounting, and
aggregate `DisclosureResponse` writes.

2026-05-30 disclosure entry owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/entry.rs` now owns collapsing-header/tree-node
entry wrappers, label identity normalization, open-model setup, trigger/content/root orchestration,
and aggregate `DisclosureResponse` writes. `disclosure_controls.rs` is now a module/re-export hub
plus test-only helper imports.

2026-05-27 text-picker entry owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/entry.rs` now owns completion/history wrapper
functions plus history filter/open normalization. `text_picker_controls.rs` keeps core picker
orchestration and re-exports the entry helpers.

2026-05-27 floating-window shell props owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_shell/props.rs` now owns frame, title-bar, shell
column, and clipped-body props. `floating_window_shell.rs` keeps shell composition, blocker
mounting, and resize-stack composition.

2026-05-30 floating-window shell props child-owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_shell/props.rs` is now a private hub.
`props/frame.rs` owns frame size/background/border/radius props, `props/body.rs` owns shell column
and clipped-body sizing/overflow/radius props, and `props/title_bar.rs` owns title-bar
clipping/padding/border/radius props. Shell composition, blocker mounting, resize-stack mounting,
and public IMUI window APIs remain unchanged.

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

2026-05-30 table render root owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_controls/render/root.rs` now owns root container props,
vertical stack mounting, optional group semantics, and root test-id forwarding.
`table_controls/render.rs` keeps palette resolution, visible-column filtering, scroll/header/body
dispatch, and final `TableResponse` aggregation.

2026-05-27 table-column visibility response owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/response.rs` now owns
`TableColumnVisibilityMenuResponse`, `TableColumnVisibilityHeaderContextMenuResponse`, and
`TableColumnVisibilityMenuItemResponse` plus their public accessors. The root
`table_column_visibility.rs` keeps options, state re-exports, public helper forwarding, and tests.

2026-05-30 table-column visibility options owner-split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/options.rs` now owns
`TableColumnVisibilityMenuOptions` and `TableColumnVisibilityHeaderContextMenuOptions`, including
header popup default sizing. The root `table_column_visibility.rs` keeps option/response/state
re-exports, public helper forwarding, and tests.

2026-05-27 control chrome palette/button/field owner-split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/chrome.rs` is now a private module index/re-export
hub. `control_chrome/chrome/palette.rs` owns `ImUiControlPalette`,
`control_chrome/chrome/button.rs` owns button theme resolution and compact button chrome props,
and `control_chrome/chrome/field.rs` owns field theme resolution plus fill-width field chrome
props.

2026-06-03 button control chrome child owner-split result:
`ecosystem/fret-ui-kit/src/imui/control_chrome/chrome/button.rs` now keeps the
`button_chrome(...)` entry only. `button/palette.rs` owns button state color fallback order, and
`button/props.rs` owns compact button container chrome props. Caller paths, theme token fallback
order, press/hover/focus semantics, and dense button chrome defaults remain unchanged.

2026-05-27 container element owner-split result:
`ecosystem/fret-ui-kit/src/imui/containers.rs` is now a private module index/re-export hub.
`ecosystem/fret-ui-kit/src/imui/containers/children.rs` owns child `ImUiFacade` mounting with build
focus propagation. `containers/linear.rs` owns horizontal/vertical flex composition,
`containers/scroll.rs` owns scroll-area construction, and `containers/grid.rs` owns grid row
batching plus keyed row assembly.

2026-05-31 container identity test owner split result:
`ecosystem/fret-ui-kit/src/imui/containers/tests/identity.rs` now keeps identity test imports and
module routing only. `identity/outer.rs` owns horizontal/vertical/grid/scroll outer-surface test-id
coverage, while `identity/viewport.rs` owns inner scroll viewport test-id coverage.

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

2026-05-30 popup-menu panel lifecycle/state owner-split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/menu/panel/state.rs` now owns popup store reads,
open/anchor validation, missing-anchor cleanup, keepalive refresh, last-panel-size desired-size
projection, and panel id writeback. `panel.rs` keeps nav-state installation, content mounting, and
`PopupMenuBuilt` assembly.

2026-05-27 checkbox behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox/behavior.rs` now owns pressable behavior
installation, activate/shortcut model toggling, context-menu key handling, transient changed reads,
and `ResponseExt` population. `checkbox.rs` keeps label identity, `CheckboxOptions` a11y wiring,
field chrome, checkbox indicator mounting, boolean label mounting, and fill-row visual assembly.

2026-05-30 checkbox behavior sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox/behavior.rs` now keeps option
normalization, shared pressable item behavior installation, and owner dispatch only.
`checkbox/behavior/activation.rs` owns click toggling, lifecycle edit marking, and changed
transient emission. `checkbox/behavior/keyboard.rs` owns activate shortcuts plus
ContextMenu/Shift+F10 requests. `checkbox/behavior/response.rs` owns changed response projection.

2026-05-30 checkbox entry/props owner split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/checkbox.rs` is now a thin module/re-export hub.
`checkbox/entry.rs` owns label identity, model reads, behavior installation, field chrome, checkbox
indicator mounting, boolean label mounting, and fill-row visual assembly. `checkbox/props.rs` owns
`PressableProps` construction plus `SemanticsRole::Checkbox`, checked-state, a11y label, and test-id
wiring.

2026-05-27 radio behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/radio/behavior.rs` now owns pressable behavior
installation, activate/shortcut click signaling, context-menu key handling, transient clicked
reads, and `ResponseExt` population. `radio.rs` keeps label identity, `RadioOptions` a11y wiring,
field chrome, radio indicator mounting, boolean label mounting, and fill-row visual assembly.

2026-05-30 radio behavior sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/radio/behavior.rs` now keeps option
normalization, shared pressable item behavior installation, and owner dispatch only.
`radio/behavior/activation.rs` owns click activation and lifecycle marking,
`radio/behavior/keyboard.rs` owns activate shortcuts plus ContextMenu/Shift+F10 requests, and
`radio/behavior/response.rs` owns clicked response projection.

2026-05-27 debug-draw media paint owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` kept
`paint_debug_draw_media_command(...)` routing for this slice. `paint/media/raster.rs` owns image,
image-region, and image-quad paint. `paint/media/rounded.rs` owns rounded image/region paint and
clip balancing. `paint/media/svg.rs` owns SVG image and mask-icon paint.

2026-05-30 debug-draw media dispatch owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media/dispatch.rs` now owns
`paint_debug_draw_media_command(...)` media command match routing and non-media no-op dispatch.
`paint/media.rs` is now only the media paint module/type hub for `MediaPaintKey`, `RasterImage`,
`RasterUvRect`, and child owner wiring.

2026-05-27 debug-draw element behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element/behavior.rs` now owns pressable
behavior installation, keyboard activation lifecycle marking, clicked transient reads, and
`ResponseExt` population. `element.rs` keeps canvas composition, fill-layout policy for interactive
canvases, cache policy, clipping, test-id routing, and debug-draw command painting.

2026-05-30 debug-draw element canvas/pressable owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/element.rs` now keeps
interactive/noninteractive element dispatch only. `element/canvas.rs` owns canvas cache policy,
fill layout, clipping, test-id routing, and command painting. `element/pressable.rs` owns
pressable props, focus-ring suppression, behavior installation, and interactive canvas embedding.

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

2026-05-30 debug-draw draw-list rect/quad/triangle owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/linear/rect_quad_triangle.rs`
is now a private module index. `rect.rs`, `quad.rs`, and `triangle.rs` own their corresponding
command recording methods.

2026-05-30 debug-draw draw-list round owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/draw_list_shapes/round.rs` is now a private
module index. `round/circle.rs` owns circle command recording, `round/ngon.rs` owns ngon command
recording, and `round/ellipse.rs` owns ellipse command recording.

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

2026-05-30 debug-draw residual summary projection owner-split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/commands/summary_projection/residual.rs` now
owns non-geometry media, clip, SVG, and text command summary dispatch. `summary_projection.rs`
keeps `summary_with_clip_state(...)`, clip-state application, and geometry/residual routing over
the unchanged private `DebugDrawCommand` discriminant.

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

2026-05-30 slider entry/props owner-split result:
`ecosystem/fret-ui-kit/src/imui/slider_controls/entry.rs` now owns label identity normalization,
push-id scoping, slider element assembly, interaction/response wiring, and final add.
`slider_controls/props.rs` owns pressable enabled/focus/layout/a11y props, and
`slider_controls.rs` is now a private module/re-export hub.

2026-05-27 begin-menu trigger behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior.rs` now owns active-trigger
behavior installation, keyboard activation lifecycle marking, activate shortcut handling, menubar
row registry/sync wiring, arrow-down/up open behavior, transient click reads, and trigger
`ResponseExt` population. `trigger.rs` keeps label identity, `PressableA11y`, pressable shell
construction, and `visual::menu_trigger_visual(...)` mounting.

2026-05-30 begin-menu trigger behavior sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/trigger/behavior.rs` now keeps input
structure, active-trigger behavior installation, menubar owner dispatch, and base owner dispatch
only. `trigger/behavior/activation.rs` owns click activation, `trigger/behavior/keyboard.rs` owns
shortcut activation, `trigger/behavior/response.rs` owns trigger response projection, and
`trigger/behavior/menubar.rs` keeps menubar-specific row behavior.

2026-05-27 switch behavior owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/switch/behavior.rs` now owns active-trigger
behavior installation, activate/shortcut model toggling, lifecycle edit marking, transient
changed/clicked reads, and `ResponseExt` population. `switch.rs` keeps label identity,
`SwitchOptions` a11y wiring, field chrome, switch state badge mounting, boolean label mounting, and
fill-row visual assembly.

2026-05-30 switch behavior sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/switch/behavior.rs` now keeps option
normalization, active-trigger behavior installation, and owner dispatch only.
`switch/behavior/activation.rs` owns click toggling, lifecycle edit marking, and clicked/changed
transient emission. `switch/behavior/keyboard.rs` owns activate shortcuts, and
`switch/behavior/response.rs` owns active-trigger response projection.

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

2026-05-30 tab trigger behavior sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/tab_family_controls/trigger/behavior.rs` now keeps input structure,
active-trigger behavior installation, and owner dispatch only. `trigger/behavior/activation.rs`
owns activate selected-model writes, `trigger/behavior/keyboard.rs` owns activate-shortcut selected
model writes, and `trigger/behavior/response.rs` owns active-trigger response projection.

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

2026-05-30 floating drag-surface child-owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/drag_surface.rs` now keeps the public entrypoint,
pointer-region shell, and bring-to-front orchestration. `drag_surface/behavior.rs` owns pointer
down/move/up drag behavior, double-click dispatch, and activation signals, while
`drag_surface/content.rs` owns setup callback invocation, key stub installation, and IMUI child
mounting.

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

2026-05-30 begin-submenu state/popup child-owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/submenu/state.rs` now owns popup-open and
was-open snapshot reads plus was-open writeback. `submenu/popup.rs` owns popup menu mounting and
disabled-popup close. `submenu.rs` keeps disabled gating, popup policy lookup, trigger creation,
open-policy reconciliation, popup delegation, and `DisclosureResponse` assembly.

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

2026-05-29 floating-window resize state sub-owner result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/initial.rs` now owns initial
`FloatWindowState` construction and stable title/close/resize test-id generation.
`floating_window_resize/state/output.rs` owns `FloatingWindowResizeStateOutput`. `state.rs` keeps
`cx.state_for(...)`, snapshot/collapse orchestration, drag application, device-pixel snapping, and
output assembly. Resize test-id strings, initial size defaults, handle packaging, and resize
behavior remain unchanged.

2026-05-30 floating-window resize state commit owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state/commit.rs` now owns
`cx.state_for(...)`, collapsed/non-drag reset policy, drag application, device-pixel snapping, state
tuple extraction, and `FloatingWindowResizeStateOutput` packaging. `state.rs` keeps the public
`prepare_resize_state(...)` parameter surface and active `resizing` derivation.

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

2026-05-30 debug-draw command summary sub-owner result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/command.rs` is now a private
re-export index. `command/kind.rs` owns `DebugDrawCommandKind`, while `command/summary.rs` owns
`DebugDrawCommandSummary` storage, accessors, construction, and channel projection.

2026-05-30 debug-draw list summary sub-owner result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/summaries/list.rs` keeps the opaque
`DebugDrawListSummary` storage shape. `list/accessors.rs` owns public getters, and
`list/mutation.rs` owns construction, final-clip-depth updates, and command inclusion aggregation.

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
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media.rs` delegates media paint behavior
to private raster, rounded, and SVG owners. Root `paint.rs` initially kept clip-stack balancing and
command-class dispatch to media vs shape painters; the 2026-06-01 follow-up moved clip-stack
handling into `paint/clip.rs`.
Debug-draw scene output and public authoring APIs remain unchanged.

2026-05-30 debug-draw media dispatch owner split result:
`ecosystem/fret-ui-kit/src/imui/debug_draw_controls/paint/media/dispatch.rs` now owns the media
command routing over image, image-region, image-quad, rounded-image, rounded-image-region, SVG
image, and SVG mask-icon commands. `paint/media.rs` only wires the child owners and shared paint
types.

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
the public entry routing and label-identity scope only until the later 2026-05-30 entry split moved
the shared implementation out. `button_controls/visual.rs` remains the layout/a11y/chrome owner.
The public button, small-button, arrow-button, invisible-button, and action-button APIs remain
unchanged.

2026-05-30 button entry owner-split result:
`ecosystem/fret-ui-kit/src/imui/button_controls/entry.rs` now owns `button_impl(...)`, label
identity parsing, visible label projection, scoped `push_id`, and delegation to
`behavior::button_pressable(...)`. `button_controls.rs` is now a wrapper hub for public-in-IMUI
button entry points.

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

2026-05-30 text-picker popup sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/popup.rs` now keeps popup mounting and
candidate iteration. `popup/keyboard.rs` owns optional popup-scoped keyboard handler installation.
`popup/types.rs` owns popup input/result data shapes. `popup/item.rs` keeps selectable candidate
rows and picked-value commits.

2026-05-27 text-picker input-root owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/input.rs` now owns picker input option/test-id
preparation, ComboBox semantics normalization, assistive semantics, root fill container
construction, text input mounting, and input-focused keyboard handler installation.
`text_picker_controls.rs` keeps candidate visibility, popup-open state reads, keyboard-state
snapshot reconciliation, popup lifecycle policy, popup rendering delegation, and final
`InputTextPickerResponse` merge. Completion/history picker behavior, active-descendant wiring,
test-id derivation, and picked response semantics remain unchanged.

2026-05-30 text-picker input options sub-owner result:
`ecosystem/fret-ui-kit/src/imui/text_picker_controls/input/options.rs` now owns
`PreparedInputTextPickerInput`, test-id fallback/`.input` suffix derivation, and TextField-to-
ComboBox role normalization. `input.rs` keeps input-root request/result shapes, assistive
semantics, root container construction, text input mounting, and keyboard handler installation.

2026-05-30 popup-store lifecycle sub-owner result:
`ecosystem/fret-ui-kit/src/imui/popup_store/lifecycle.rs` now owns stale popup cleanup during
render-generation preparation. `popup_store.rs` keeps popup store state, generation entry points,
scoped entry lookup, and explicit scope dropping.

2026-05-31 popup-store state/entry/drop owner split result:
`ecosystem/fret-ui-kit/src/imui/popup_store.rs` is now a thin private re-export hub.
`popup_store/state.rs` owns the per-window/per-id storage records and model creation,
`popup_store/entry.rs` owns render-generation marking plus scoped lookup, and
`popup_store/drop_scope.rs` owns explicit scope removal/model reset. The stale-generation cleanup
owner remains `popup_store/lifecycle.rs`.

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
radius props until the later 2026-05-30 children split moved the flex/body composition out.
`disclosure_controls/visual.rs` keeps disclosure a11y, content padding, and palette resolution.
Trigger pressable behavior, shortcut/context-menu handling, indicator glyphs, label text roles,
indentation, and row chrome remain unchanged.

2026-05-30 disclosure header children owner-split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual/header/children.rs` now owns the header
flex row, indicator slot, label text, and spacer composition. `header.rs` keeps palette lookup,
row container props, and metric lookups only. The public collapsing-header and tree-node APIs
remain unchanged.

2026-05-30 disclosure visual sub-owner split result:
`ecosystem/fret-ui-kit/src/imui/disclosure_controls/visual.rs` is now the header/a11y/style
re-export hub. `visual/a11y.rs` owns collapsing-header/tree-node `PressableA11y` construction,
while `visual/style.rs` owns content padding and palette resolution. Header-row rendering,
trigger behavior, public disclosure facade calls, and a11y/palette outcomes remain unchanged.

2026-05-26 combo trigger owner-split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/trigger.rs` now owns ComboBox pressable
construction, accessibility label derivation, shortcut activation, context-menu key handling,
trigger `ResponseExt` population, and the open/menu badge chrome. `combo_controls.rs` keeps label
identity normalization, popup open/close model wiring, popup mounting, and aggregate
`ComboResponse` open/toggled state. The public combo and combo-model facade APIs remain unchanged.

2026-05-30 combo-model owner split result:
`ecosystem/fret-ui-kit/src/imui/combo_model_controls.rs` is now a thin module/re-export hub.
`combo_model_controls/entry.rs` owns model reads, preview fallback, combo option forwarding, and
canonical combo mounting. `combo_model_controls/popup_items.rs` owns borrowed item iteration,
selectable item rows, option test-id suffixes, model updates, and popup close. `response.rs` owns
changed/edited/deactivated-after-edit response projection.

2026-05-26 boolean visual owner-split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/visual.rs` now owns shared boolean label text and
re-export routing for checkbox/radio/switch indicator chrome. The 2026-05-31 follow-up moved the
indicator chrome into `boolean_controls/visual/indicators.rs`. `boolean_controls.rs` keeps
checkbox/radio pressable orchestration, shortcut/context-menu handling, and response population,
while `boolean_controls/switch.rs` keeps switch active-trigger behavior and model updates. The
public checkbox, radio, and switch APIs remain unchanged.

2026-05-30 switch entry/props owner split result:
`ecosystem/fret-ui-kit/src/imui/boolean_controls/switch.rs` is now a thin module/re-export hub.
`switch/entry.rs` initially owned label identity, model reads, active-trigger behavior
installation, field chrome, switch state badge mounting, boolean label mounting, and fill-row
visual assembly. The 2026-05-31 follow-up moved those render/runtime concerns into
`switch/entry/render.rs`, leaving `switch/entry.rs` with public entrypoints and label identity
scoping only.
`switch/props.rs` owns `PressableProps` construction plus switch a11y label, checked state, and
test-id wiring.

2026-05-26 hover query owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/flags.rs` now owns `ImUiHoveredFlags`, while
`ecosystem/fret-ui-kit/src/imui/response/hover/query.rs` owns the ImGui-style hovered query
helpers. `response/hover.rs` keeps `ResponseExt` storage, crate-local mutators, public accessors,
and drag convenience helpers until the later 2026-05-30 drag accessor owner split moves drag
methods out. The public hover flags and `ResponseExt` API remain unchanged.

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

2026-05-30 drag accessor owner-split result:
`ecosystem/fret-ui-kit/src/imui/response/hover/drag_accessors.rs` now owns `ResponseExt` drag
mutation plus public drag read accessors. `response/hover.rs` keeps the drag storage field only.
The public `ResponseExt` API remains unchanged.

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
registration, IMUI facade content mounting, absolute area layout, no-input/pass-through gates, and
`FloatingAreaResponse` assembly.
`floating_surface.rs` keeps drag-surface pointer-region behavior, layer/kind/state re-exports, and
module wiring. Floating-area position, dragging, test-id, no-inputs, pointer pass-through, and
response semantics remain unchanged.

2026-05-29 floating-area drag/state sub-owner result:
`ecosystem/fret-ui-kit/src/imui/floating_surface/area/drag_state.rs` now owns drag snapshot
discovery, drag-position reconciliation, scale-factor snapping, test-id state updates, and final
placement readback. `floating_surface/area.rs` now only orchestrates layer registration, area
context creation, IMUI child mounting, layout shell creation, and response assembly.

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
`ecosystem/fret-ui-kit/src/imui/multi_select/state.rs` now owns `ImUiMultiSelectState` storage and
read-only accessors. 2026-05-30 follow-up:
`ecosystem/fret-ui-kit/src/imui/multi_select/state/selection.rs` now owns ordered-selection
normalization, anchor repair, crate-local mutation helpers, and `is_selected(...)`. The root
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

2026-05-30 popup modal layout child-owner split result:
`ecosystem/fret-ui-kit/src/imui/popup_overlay/modal/layout.rs` is now a private hub.
`layout/types.rs` owns modal palette resolution, dim color, and centered panel geometry.
`layout/props.rs` owns absolute layer/backdrop props, dialog semantics layout/test id, panel chrome,
and full-inset construction. Modal open/close behavior, barrier semantics, centered placement, and
test ids remain unchanged.

2026-05-26 menu-item interaction owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/interaction.rs` now owns menu item enabled/action
gating, pressable props, activation and shortcut handlers, popup menu roving focus, menubar
horizontal-arrow switching, command dispatch source metadata, and `ResponseExt` population.
`menu_controls/element.rs` kept the row panel, checkbox/radio/submenu indicators, shortcut text,
and label text until the later visual-row split; it keeps the custom `pressable_hook` insertion
point. Public menu item and command menu item facade APIs remain unchanged.

2026-05-30 menu-item visual row owner-split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/element/visual_row.rs` now owns menu item panel/row
props, checkbox/radio/submenu indicator selection, label/shortcut/submenu glyph mounting, and
shortcut test-id stamping. `menu_controls/element.rs` keeps pressable orchestration, interaction
owner wiring, response population, and the custom `pressable_hook` insertion point.

2026-05-30 menu-item visual-row child-owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_controls/element/visual_row.rs` now keeps menu-item visual row
option projection and render orchestration only. `visual_row/layout.rs` owns panel/row props, and
`visual_row/content.rs` owns checkbox/radio/submenu indicator selection, shortcut mounting, and
shortcut test-id stamping.

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
selected/highlighted state reads, behavior wiring, and row visual assembly. The 2026-05-31
follow-up moved pressable/a11y prop construction into `selectable_controls/props.rs`.

2026-05-26 textarea owner-split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/textarea.rs` now owns textarea props assembly,
lifecycle/response population, select-all-on-focus command emission, submit/cancel command policy
installation, and text-area chrome/text-style selection. `text_controls.rs` keeps input-text
assembly plus shared helper routing. Public `textarea_model(...)` and `textarea_model_with_options`
facade behavior remains unchanged.

2026-05-30 text-control style child-owner split result:
`ecosystem/fret-ui-kit/src/imui/text_controls/style.rs` now keeps input/textarea style assembly
and public text-style helper routing. `style/palette.rs` owns theme color fallback plus
selection/preedit derivation, while `style/chrome.rs` owns input padding, border, radius, and fixed
field layout. Input-text and textarea chrome behavior remains unchanged.

2026-05-26 floating-window resize state owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/state.rs` now owns active resize snapshot
lookup, drag delta application, min/max size clamping, left/top origin updates, collapse reset,
device-pixel snapping, and resize state/test-id output. `floating_window_resize.rs` is now a thin
`handles`/`state` index plus the shared resize-handle test-id record; `handles.rs` still owns
pointer-region handle rendering and drag lifecycle wiring.

2026-05-29 floating-window on-area state prep owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_on_area/state.rs` now owns resizable-layout and
resize-enabled derivation, collapse toggle/readback, scale-factor lookup, resize owner calls, area
position feedback after resize, and `FloatingWindowChromeResponse` assembly.
`floating_window_on_area.rs` now wires prepared state into title bar, content, shell, and facade
output.

2026-05-26 floating-window resize snapshot owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/snapshot.rs` now owns active resize drag
discovery and snapshot capture. `state.rs` now focuses on resize delta application, min/max
clamping, origin updates, collapse reset, device-pixel snapping, and output assembly. Public
floating-window facade behavior and internal `floating_window_resize::current_resize_snapshot(...)`
call sites remain unchanged.

2026-05-26 floating-window resize handle owner-split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/layout.rs` now owns handle geometry
while `handles/pointer.rs` owns pointer-region wiring, pointer capture, runtime drag
begin/update/cancel, cursor updates, and activation handoff. `handles.rs` now only stacks
body/blocker with the eight resize handles.

2026-05-29 floating-window resize cursor sub-owner result:
`ecosystem/fret-ui-kit/src/imui/floating_window_resize/handles/cursor.rs` now owns
handle-to-cursor mapping for all eight resize handles. `handles/layout.rs` is geometry-only, and
`handles/pointer.rs` composes cursor and layout before wiring pointer-region behavior, keeping the
handle stack free of layout, cursor, and drag details.

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
  Current porting-sugar audit result: `SameLine` is now a narrow proven teaching-surface helper
  through the existing closure-scoped layout sugar and cookbook payload-row proof. Keep item-width,
  next-item width, and label-ID sugar candidate-only until at least two proof surfaces pay the same
  authoring tax. Prefer typed Fret helpers (`horizontal_with_options`,
  `PropertyGrid::row_with`, explicit `id_source` / `test_id`) over copying Dear ImGui's mutable
  cursor, item-width stack, or label suffix parser.
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

Fret Plot declarative model projection owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/model.rs` now owns all concrete plot model projection into the
private `PlotPanelModel` records, including histogram bin projection. The declarative root keeps the
retained-free paint/event/layout core, while `declarative/panels.rs` and `declarative/props.rs`
continue to own public entrypoints and public props. This keeps the optional IMUI plot adapter
thin over declarative panels instead of rebuilding plot policy in `fret-imui` or
`fret-ui-kit::imui`.

Fret Plot declarative legend owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/legend.rs` now owns private legend painting and hit testing,
including row metrics, swatch/text painting, hover/pin highlight, and swatch/label hit testing. The
declarative root keeps event state mutation for hidden-series and pinned-series changes, so legend
interaction policy does not move into the paint/hit-test owner or the optional IMUI adapter.

Fret Plot declarative path-command owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/commands.rs` now owns private path-command projection for line,
area, shaded, stems, histogram, bars, candlestick, and error-bar series. The declarative root keeps
paint/event orchestration and imports only command projection entrypoints, so path generation stays
declarative and does not move into `fret-imui`, `fret-ui-kit::imui`, or the optional adapter.

Fret Plot declarative selection overlay owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/selection.rs` now owns query/box-zoom selection rectangle
painting and tooltip placement/text. The declarative root keeps drag/session mutation and pointer
event handling, so selection interaction policy stays out of the paint/tooltip owner and the
optional IMUI adapter.

Fret Plot declarative readout owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/readout.rs` now owns cursor and linked-cursor readout painting,
overlay placement, series row projection, and pinned-series filtering. The declarative root keeps
shared axis label formatting, event output publication, and plot state handling, so readout
presentation stays declarative and out of the optional IMUI adapter.

Fret Plot declarative axis label owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/axis_labels.rs` now owns primary and right-axis tick label
painting, including y2/y3/y4 lane offsets, text constraints, and stable canvas text keys. The
declarative root keeps grid/baseline axis painting, shared axis label formatting, data/view bounds
orchestration, event output publication, and plot state handling, so axis labels stay declarative
and out of the optional IMUI adapter.

Fret Plot declarative heatmap owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/heatmap.rs` now owns heatmap grid cell painting and default
colorbar projection, including clipping, colormap sampling, gradient steps, and min/max labels. The
declarative root keeps panel paint orchestration, event output publication, and plot state handling
while `declarative/model.rs` keeps heatmap model projection for heatmap and histogram2d plot models.

Fret Plot declarative overlay paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/overlays.rs` now owns reference lines, draggable point/rect
paint, image layer painting, draggable labels, tag overlays, and text overlays. The declarative root
keeps panel paint orchestration, draggable overlay event routing, output publication, and plot state
handling, so overlay paint stays state-free and event-free.

Fret Plot declarative image overlay paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/overlays/images.rs` now owns caller-owned `PlotImage` layer
filtering, multi-axis projection, clipping, opacity filtering, and `ImageRegion` scene emission.
`overlays.rs` re-exports image overlay painting and keeps reference lines, draggable shapes,
draggable labels, tag overlays, text overlays, and annotation text box helpers.

Fret Plot declarative draggable overlay labels paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/overlays/draggable_labels.rs` now owns draggable line and point
label projection. `overlays.rs` re-exports draggable overlay label painting and keeps shared
annotation token/text-box helpers for tag, text, and label overlay owners.

Fret Plot declarative tag overlay paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/overlays/tags.rs` now owns `TagX` and `TagY` projection.
`overlays.rs` re-exports tag overlay painting and keeps shared annotation token/text-box and marker
paint helpers for tag, text, and draggable-label owners.

Fret Plot declarative plot text overlay paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/overlays/text.rs` now owns `PlotText` placement, right-axis
anchoring, background quad, and text emission. `overlays.rs` re-exports plot text overlay painting
and keeps shared annotation token/text-box, clamp, and marker paint helpers.

Fret Plot declarative tests owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/tests.rs` now owns the declarative plot panel regression tests,
including `TestHost`, scene helpers, paint regressions, drag output regressions, and
linked-cursor/readout regressions. The implementation root keeps `#[cfg(test)] mod tests;` only.

Fret Plot declarative interaction owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/interaction.rs` now owns legend, draggable, query, box-zoom,
pan, and wheel event routing plus interaction session records. The declarative root keeps panel
assembly, paint orchestration, output publication, view/output snapshot records, shared geometry
helpers, and plot state model wiring.

Fret Plot declarative output owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/output.rs` now owns output publication, query extraction,
pointer cursor snapshots, output snapshot construction, and state/default view bounds projection.
The declarative root keeps panel assembly, paint orchestration, grid/axis painting, shared geometry
helpers, and plot state model wiring.

Fret Plot declarative grid axes owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/grid_axes.rs` now owns grid tick projection, grid line
painting, baseline axis painting, and primary-axis tick label orchestration. The declarative root
keeps panel assembly, paint orchestration, shared paint primitives, shared geometry helpers, and
plot state model wiring.

Fret Plot declarative paint primitives owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/paint_primitives.rs` now owns shared Quad helpers for vertical
lines, horizontal lines, and filled rectangles. Grid, readout, heatmap, and overlay owners import
those primitives explicitly, and the declarative root keeps panel assembly, paint orchestration,
shared geometry helpers, and plot state model wiring.

Fret Plot declarative geometry owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/geometry.rs` now owns shared inner-rect and y-axis view-bounds
projection. Axis labels, interaction, output, and overlay owners import geometry explicitly, and the
declarative root keeps panel assembly, paint orchestration, formatting helpers, series color policy,
and plot state model wiring.

Fret Plot declarative style helpers owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/style_helpers.rs` now owns axis label formatting and series
color fallback. Axis labels, readout, selection, overlays, legend, and panel paint import style
helpers explicitly, and the declarative root kept panel assembly, paint orchestration, and plot
state model wiring at that slice. The later panel-paint owner split narrows the current root role.

Fret Plot declarative panel paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/panel_paint.rs` now owns panel background, grid, heatmap,
right-axis labels, series, overlays, legend, selection/readout, and command-builder paint
orchestration at that slice. The declarative root keeps panel element assembly, event wiring,
output publication, and plot state model wiring, while the source gate keeps event/state/retained
concerns out of the paint owner. The later series-paint owner split narrows current panel paint
responsibility.

Fret Plot declarative series paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/series_paint.rs` now owns line, area, shaded, stems,
histogram, bars, candlestick, and error-bar series painting at that slice. `panel_paint.rs` keeps
background, grid, heatmap, right-axis labels, overlays, legend, selection/readout, and panel-level
paint orchestration, while the source gate keeps event/output/overlay concerns out of the series
owner. The later candlestick owner split narrows current series router responsibility.

Fret Plot declarative candlestick series paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/series_paint/candlestick.rs` now owns candlestick wick/body
command painting. `series_paint.rs` delegates candlestick drawing and keeps line, area, shaded,
stems, histogram, bars, and error-bar series routing, while the source gate keeps non-candlestick
series concerns out of the candlestick owner.

Fret Plot declarative bar and histogram series paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/series_paint/bar_histogram.rs` now owns bar and histogram
closed fill path drawing. `series_paint.rs` delegates bar/histogram drawing and keeps line, area,
shaded, stems, and error-bar series routing, while the source gate keeps non-bar/histogram concerns
out of the bar/histogram owner.

Fret Plot declarative error-bars series paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/series_paint/error_bars.rs` now owns error-bars caps and
markers stroke path drawing. `series_paint.rs` delegates error-bars drawing and keeps line, area,
shaded, and stems series routing, while the source gate keeps non-error-bars concerns out of the
error-bars owner.

Fret Plot declarative shaded series paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/series_paint/shaded.rs` now owns shaded band fill and
upper/lower stroke path drawing. `series_paint.rs` delegates shaded drawing and keeps line, area,
and stems series routing, while the source gate keeps non-shaded concerns out of the shaded owner.

Fret Plot declarative line/area/stems series paint owner split - 2026-06-02:
`ecosystem/fret-plot/src/declarative/series_paint/line_area.rs` now owns line, area-fill, and stems
stroke path drawing. `series_paint.rs` delegates all concrete series drawing and keeps axis
transform selection plus concrete series routing, while the source gate keeps command/path drawing
out of the router.

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

2026-06-01 IMUI facade export owner split result:
`ecosystem/fret-ui-kit/src/imui/exports.rs` now owns the public `fret_ui_kit::imui::*` re-export
surface. `ecosystem/fret-ui-kit/src/imui.rs` remains the module hub and shared internal-import owner
and republishes the same public surface through `pub use exports::*;`. This keeps downstream import
paths stable while removing public API catalog churn from the root implementation hub.

2026-06-01 combo popup state owner split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/state.rs` now owns popup open-state reads,
trigger-driven open/close transitions, disabled popup cleanup, toggled detection, and trigger
response flag mutation. `combo_controls.rs` keeps the higher-level combo flow: label identity,
trigger option wiring, popup body composition, and final `ComboResponse` assembly.

2026-06-01 floating-window state owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window/state.rs` now owns optional open-model reads and
chrome-to-`FloatingWindowResponse` assembly. `floating_window.rs` keeps floating-area setup,
closed-window routing, and render-in-area wiring, while `floating_window/closed.rs` remains the
closed response sentinel owner.

2026-06-01 active-trigger type owner split result:
`ecosystem/fret-ui-kit/src/imui/active_trigger_behavior/types.rs` now owns the shared
`ActiveTriggerBehavior`, `ActiveTriggerBehaviorOptions`, and `ActiveTriggerResponseInput` data
shapes. The root behavior file re-exports those types through the original private module path and
keeps behavior installation plus response delegation only.

2026-06-01 menubar policy-state owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/policy_state.rs` now owns
`ImUiMenubarPolicyState` and its open-menu/group-active/registry/close-auto-focus model handles.
`menu_family_controls.rs` re-exports the type through the original private module path and keeps
menu-bar composition plus child mounting only.

2026-06-01 image-item pressable props owner split result:
`ecosystem/fret-ui-kit/src/imui/image_item_controls/props.rs` now owns `PressableProps`
construction for image items and image buttons, including sanitized size, focusability, key
activation policy, and a11y role/label/test id propagation. `image_item_controls.rs` keeps the
identity, behavior, chrome, image props, and response wiring.

2026-06-01 selectable entry owner split result:
`ecosystem/fret-ui-kit/src/imui/selectable_controls/entry.rs` now owns the visible-label selectable
entry assembly: response initialization, enabled/focusable/selected/highlighted derivation,
`PressableProps` delegation, `pressable_with_id` mounting, behavior installation, visual row
mounting, and final response return. `selectable_controls.rs` keeps label identity parsing plus the
stable `push_id` wrapper.

2026-06-01 ListBox scroll-host and semantics owner split result:
`ecosystem/fret-ui-kit/src/imui/list_box_controls/scroll_host.rs` now owns ListBox scroll-area
composition, hosted child focus forwarding, content/root/viewport test-id wiring, scrollbar/handle
application, and final semantics attachment. `list_box_controls/semantics.rs` owns ListBox role,
label, and multiselectable semantics construction. `list_box_controls.rs` keeps keyed identity and
`ListBoxOptions` destructuring only.

2026-06-01 child-region entry owner split result:
`ecosystem/fret-ui-kit/src/imui/child_region/entry.rs` now owns child-region keyed body
orchestration: resize detection, scroll layout choice, scroll input assembly, resize-vs-scroll root
test-id routing, response initialization, and resize-stack selection. `child_region.rs` keeps the
facade-facing keyed wrapper only.

2026-06-01 floating-window in-area assembly owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_on_area/assembly.rs` now owns prepared floating
window state consumption plus title bar, content, and shell assembly for in-area windows.
`floating_window_on_area.rs` keeps only the facade-facing `with_cx_mut` wrapper, window insertion,
and chrome response return.

2026-06-01 floating-window shell body owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_shell/body.rs` now owns title/body/clipped-body
assembly, input blocker mounting, and resize-stack delegation. `floating_window_shell.rs` keeps
frame palette resolution, frame props, and the outer frame container.

2026-06-01 floating-window entry owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window/entry.rs` now owns show-with-options
orchestration: option destructuring, open checks, floating-area mounting, chrome capture, in-area
render dispatch, and final response assembly delegation. `floating_window.rs` keeps the
facade-facing helper pair only.

2026-06-01 table-column visibility model owner split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/model.rs` now owns the controllable-model
bridge for `ImUiTableColumnVisibilityState`. `table_column_visibility.rs` keeps the public
`table_column_visibility_use_model(...)` signature as a forwarding helper, plus option/response/
state re-exports and menu delegation.

2026-06-01 multi-select model owner split result:
`ecosystem/fret-ui-kit/src/imui/multi_select/model.rs` now owns the controllable-model bridge for
`ImUiMultiSelectState<K>`. `multi_select.rs` keeps the public `multi_select_use_model(...)`
signature as a forwarding helper, plus state re-export, click-policy delegation, and selectable
entry wiring.

2026-06-01 separator-text element owner split result:
`ecosystem/fret-ui-kit/src/imui/separator_text_controls/element.rs` now owns section-label element
construction: text-role label chrome, trailing border rule, row layout, and test-id decoration.
`separator_text_controls.rs` keeps visible-label identity parsing and facade insertion only.

2026-06-01 menubar root element owner split result:
`ecosystem/fret-ui-kit/src/imui/menu_family_controls/menu_bar.rs` now owns the named menubar
element, local policy models, trigger-row registry clearing, child hosting, row layout, and
`SemanticsRole::MenuBar`. `menu_family_controls.rs` keeps only module routing plus begin-menu/
submenu exports.

2026-06-01 floating-window title-bar props owner split result:
`ecosystem/fret-ui-kit/src/imui/floating_window_title_bar_props/drag_surface.rs` now owns
pointer-region layout and resizable fill/shrink behavior, while
`floating_window_title_bar_props/close_button.rs` owns close-button a11y/test-id wiring and fixed
20px sizing. `floating_window_title_bar_props.rs` keeps title-row props plus private re-exports.

2026-06-01 combo entry owner split result:
`ecosystem/fret-ui-kit/src/imui/combo_controls/entry.rs` now owns the direct combo flow: visible
label parsing, enabled/open reads, trigger option construction, popup mounting, disabled cleanup,
and `ComboResponse` open/toggled assembly. `combo_controls.rs` keeps the facade-facing helper
signature and forwards to the entry owner.

2026-06-01 image-item entry owner split result:
`ecosystem/fret-ui-kit/src/imui/image_item_controls/entry.rs` now owns the inner pressable element
body for image items/buttons: enabled/focusable derivation, pressable props, behavior installation,
chrome, image props, element insertion, and `ResponseExt` return. `image_item_controls.rs` keeps
the stable `push_id(("image-item", id), ...)` wrapper.

2026-06-01 active-trigger install owner split result:
`ecosystem/fret-ui-kit/src/imui/active_trigger_behavior/install.rs` now owns hook clearing,
active/lifecycle/context model lookup, context-menu key handling, pointer handler installation, and
`ActiveTriggerBehavior` assembly. `active_trigger_behavior.rs` keeps the stable install and
response-population entry points.

2026-06-03 editor widget visuals owner split result:
`ecosystem/fret-ui-editor/src/primitives/visuals.rs` now keeps the stable
`EditorWidgetVisuals` helper facade, icon-button hover helpers, and public free-function wrappers.
Private owners now carry the reusable policy details: `visuals/color_math.rs` owns alpha scaling
and color interpolation, `visuals/invalid.rs` owns invalid foreground/border/background fallback,
`visuals/frame.rs` owns input-like frame state projection, and `visuals/selection.rs` owns
selection/toggle-like frame state projection. Public editor/IMUI adapter APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor numeric text-entry owner split result:
`ecosystem/fret-ui-editor/src/primitives/numeric_text_entry.rs` now keeps the stable numeric
text-entry facade, `NumericInputSelectionBehavior` re-export, focus/handoff helper re-exports, and
the replace-key handler used by numeric input, slider, axis drag value, and drag value controls.
`numeric_text_entry/focus.rs` owns focus state, focus handoff timers, draft/error resync, and
draft-change error clearing. `numeric_text_entry/replace.rs` owns `NumericReplacementPlan`,
replace-on-first-edit key classification, paste handling, delete consumption, and insertion-key
classification. Public editor/IMUI adapter APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor VecEdit model record owner split result:
`ecosystem/fret-ui-editor/src/controls/vec_edit/model.rs` now keeps the stable public VecEdit model
hub, Vec2/Vec3/Vec4 re-exports, and the focused presentation regression test. The public model
records moved into dedicated private owners: `model/vec2.rs` owns `Vec2Edit`, `model/vec3.rs` owns
`Vec3Edit`, and `model/vec4.rs` owns `Vec4Edit`. Each owner carries its record fields,
constructor, presentation helper, validation/reset/option builders, and `into_element(...)`
delegation through the existing private keying owner. Public editor/IMUI adapter APIs remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor TextAssistField body assembly owner split result:
`ecosystem/fret-ui-editor/src/controls/text_assist_field/element.rs` now keeps the public
`TextAssistField` record, constructors/builders, and callsite/id-source keyed routing only.
`controls/text_assist_field/element/body.rs` owns the keyed body assembly: controller projection,
expanded-state sync, panel rendering, input-owned text-assist semantics, TextField assistive
semantics wiring, inline/overlay panel selection, empty-label fallback, root layout, and keyboard
handler installation through the existing private keyboard owner. Public editor/IMUI adapter APIs
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor ColorEdit popup body section owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/body.rs` now keeps popup argument reads,
current/reference model projection, runtime option resolution, popup chrome/container assembly,
width selection, and layout mounting. `controls/color_edit/popup/body/sections.rs` owns picker,
side-preview, picker-options, eyedropper, numeric row, history swatch, preset swatch, and
standalone alpha-bar section construction. The popup layout owner remains
`controls/color_edit/popup/body/layout.rs`, and `imui_surface_policy` now checks the sections owner
for the real preset-palette source anchors. Public editor/IMUI adapter APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor ColorEdit hue-wheel model owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/model/hue_wheel.rs` now keeps only the stable
public hue-wheel model re-export hub. `model/hue_wheel/geometry.rs` owns `HueWheelGeometry`,
finite-size sanitization, and wheel/triangle radius projection. `model/hue_wheel/triangle.rs` owns
`HueWheelTriangle`, rotated-triangle projection, SV cursor projection, barycentric math, and
closest-point helpers. `model/hue_wheel/interaction.rs` owns `HueWheelDragTarget`, pointer hit
testing, and position-to-HSV mapping. Public editor/IMUI adapter APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor slider typing-input owner split result:
`ecosystem/fret-ui-editor/src/controls/slider/element.rs` now keeps keyed slider state and
focus-handoff lookup, current value reads, layout switching, pressable slider assembly, and
child-owner routing only. `controls/slider/element/typing_input.rs` owns NumericInput
construction, parse/validate adapter wiring, trailing-icon validation error policy, commit/cancel
reset plus slider focus restore, and focus handoff sync. Public editor/IMUI adapter APIs remain
unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor TextField entry branch owner split result:
`ecosystem/fret-ui-editor/src/controls/text_field/element/entry.rs` now keeps only the shared
entry args plus multiline-vs-single-line routing. `controls/text_field/element/entry/multiline.rs`
owns the TextArea props call, mount, buffered multiline session/key/blur wiring, focus-selection
routing, and unbuffered Escape-clear installation. `controls/text_field/element/entry/single_line.rs`
owns the TextInput props call, mount, buffered single-line session/key/blur wiring,
submit-command-aware key mode, and focus-selection routing. Public `TextField` options and
editor/IMUI adapter APIs remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-03 editor ColorEdit swatch visual owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/swatch.rs` now keeps swatch pressable
registration, popup activation/context-menu orchestration, drag source/drop hover hooks, and
visual child-owner routing only. `controls/color_edit/swatch/visual.rs` owns tooltip-open
synchronization, `EditorWidgetVisuals` frame projection, clipped preview container assembly, and
`color_preview_stack(...)` mounting. Public `ColorEdit` behavior and editor/IMUI adapter APIs
remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 editor TransformEdit section chrome row/column owner split result:
`ecosystem/fret-ui-editor/src/controls/transform_edit/sections.rs` now keeps only the section
chrome owner hub and private row/column module routing. `controls/transform_edit/sections/row.rs`
owns horizontal badge chrome plus the row Link toggle, while
`controls/transform_edit/sections/column.rs` owns column heading chrome plus the Uniform toggle
row. Section text roles, link-scale test IDs, Column/Row layout selection, Vec3Edit composition,
uniform-scale sync, public `TransformEdit` options, and editor/IMUI adapter APIs remain unchanged,
and `tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 table-column visibility item-response owner split result:
`ecosystem/fret-ui-kit/src/imui/table_column_visibility/response.rs` now keeps aggregate menu
response and header context-menu response ownership plus the stable public re-export for
`TableColumnVisibilityMenuItemResponse`. The item response storage moved to
`table_column_visibility/response/item.rs`, where it owns opaque fields, public accessors, clicked/
changed forwarding, and crate-local construction. Menu item aggregation now uses the constructor
instead of field literals, and `tools/gate_imui_workstream_source.py` freezes the new owner.

2026-06-03 Fret Plot line model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-line model records, shared
bounds helpers, and the stable public re-export for `LineSeries` / `LinePlotModel`.
`ecosystem/fret-plot/src/models/line.rs` owns line series fields/builders, line plot model records,
default and caller-supplied bounds construction, and primary/Y2/Y3/Y4 data-bound projection. Public
`crate::models::{LineSeries, LinePlotModel}` imports, line chart builder routing, declarative line
plot panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 Fret Plot stems model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-stems model records, shared
bounds helpers, and the stable public re-export for `StemsSeries` / `StemsPlotModel`.
`ecosystem/fret-plot/src/models/stems.rs` owns stems series fields/builders, baseline policy,
stems plot model records, and primary/Y2/Y3/Y4 baseline-expanded data-bound projection. Public
`crate::models::{StemsSeries, StemsPlotModel}` imports, declarative stems plot panels, and optional
IMUI adapter routing remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-03 Fret Plot scatter model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-scatter model records, shared
bounds helpers, and the stable public re-export for `ScatterSeries` / `ScatterPlotModel`.
`ecosystem/fret-plot/src/models/scatter.rs` owns scatter series fields/builders, marker radius and
marker shape policy, scatter plot model records, and primary/Y2/Y3/Y4 data-bound projection. Public
`crate::models::{ScatterSeries, ScatterPlotModel}` imports, declarative model imports, and optional
IMUI adapter routing remain unchanged, and `tools/gate_imui_workstream_source.py` freezes the
split.

2026-06-03 Fret Plot area model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-area model records, shared
bounds helpers, and the stable public re-export for `AreaSeries` / `AreaPlotModel`.
`ecosystem/fret-plot/src/models/area.rs` owns area series fields/builders, fill alpha and baseline
policy, area plot model records, caller-supplied bounds construction, and primary/Y2/Y3/Y4
baseline-expanded data-bound projection. Public `crate::models::{AreaSeries, AreaPlotModel}`
imports, declarative area plot panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 Fret Plot shaded model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-shaded model records, shared
bounds helpers, and the stable public re-export for `ShadedSeries` / `ShadedPlotModel`.
`ecosystem/fret-plot/src/models/shaded.rs` owns shaded series fields/builders, upper/lower band
bounds union policy, shaded plot model records, caller-supplied bounds construction, and
primary/Y2/Y3/Y4 data-bound projection. Public `crate::models::{ShadedSeries, ShadedPlotModel}`
imports, declarative shaded plot panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 Fret Plot error-bars model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-error-bars model records,
shared bounds helpers, and the stable public re-export for `ErrorBar`, `ErrorBarsSeries`, and
`ErrorBarsPlotModel`. `ecosystem/fret-plot/src/models/error_bars.rs` owns error-bar payloads,
error-bars series fields/builders, cap/marker policy, error-bars plot model records, and
primary/Y2/Y3/Y4 error-expanded data-bound projection. Public
`crate::models::{ErrorBar, ErrorBarsSeries, ErrorBarsPlotModel}` imports, declarative error-bars
plot panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 Fret Plot candlestick model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-candlestick model records,
shared bounds helpers, and the stable public re-export for `OhlcPoint`, `CandlestickSeries`, and
`CandlestickPlotModel`. `ecosystem/fret-plot/src/models/candlestick.rs` owns OHLC payloads,
close-series adapter storage, candlestick series fields/builders, candlestick plot model records,
candle-width bounds construction, and the focused bounds unit test. Public
`crate::models::{OhlcPoint, CandlestickSeries, CandlestickPlotModel}` imports, declarative
candlestick plot panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 Fret Plot bars model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-bar model records, shared
bounds helpers, and the stable public re-export for `BarSeries`, `CategoryBarSeries`, and
`BarsPlotModel`. `ecosystem/fret-plot/src/models/bars.rs` owns bar/category payloads, bar series
fields/builders, grouped and stacked category helpers, bars plot model records, and
baseline-aware primary/Y2/Y3/Y4 data-bound projection. Public
`crate::models::{BarSeries, CategoryBarSeries, BarsPlotModel}` imports, declarative bars plot
panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-03 Fret Plot histogram model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes, non-histogram model records,
shared bounds helpers, and the stable public re-export for `HistogramSeries` and
`HistogramPlotModel`. `ecosystem/fret-plot/src/models/histogram.rs` owns histogram sample payloads,
bin/range/gap/fill series builders, histogram plot model records, and bins-backed
primary/Y2/Y3/Y4 data-bound projection. Public
`crate::models::{HistogramSeries, HistogramPlotModel}` imports, declarative histogram plot panels,
and optional IMUI adapter routing remain unchanged, and `tools/gate_imui_workstream_source.py`
freezes the split.

2026-06-03 Fret Plot heatmap grid-value model owner split result:
`ecosystem/fret-plot/src/models.rs` now keeps shared plot axes and shared series-data bounds
helpers only, plus stable public re-exports for `HeatmapPlotModel` and `Histogram2DPlotModel`.
`ecosystem/fret-plot/src/models/heatmap.rs` owns both grid-value model records, grid shape/value
storage, finite min/max fallback, sanitized data bounds, and row-major `value_at(...)` lookup.
Public `crate::models::{HeatmapPlotModel, Histogram2DPlotModel}` imports, declarative heatmap and
histogram2d plot panels, and optional IMUI adapter routing remain unchanged, and
`tools/gate_imui_workstream_source.py` freezes the split.

2026-06-05 editor ColorEdit popup sections child-owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/body/sections.rs` now keeps popup section
type records, child module declarations, and the stable `color_popup_body_sections(...)`
re-export. Private child owners now carry the section-specific construction:
`sections/picker.rs` owns picker-shape and standalone alpha-bar selection, `sections/actions.rs`
owns picker-options/eyedropper/numeric section construction, `sections/preview.rs` owns
side-preview construction, and `sections/swatches.rs` owns history/preset swatch construction.
Popup chrome, layout ordering, runtime picker overrides, side-preview restore behavior, swatch
drag/drop hooks, numeric rows, public ColorEdit APIs, and IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the split.

2026-06-05 editor ColorEdit popup request owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup.rs` now keeps popup module routing and the
stable `request_popup_overlay(...)` re-export. The new
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/request.rs` owns visible-content gating,
draft/error model setup, overlay id and presence creation, popper placement, anchored props,
pointer-region wrapping, dismissible menu request flags, close-on-window-focus/resize policy, and
close-auto-focus restore to the swatch. Popup body assembly remains in `popup/body.rs`, public
ColorEdit APIs and IMUI facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py`
plus `imui_surface_policy` freeze the hub/request split.

2026-06-05 editor ColorEdit popup sections assembly owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/body/sections.rs` now keeps section type
records, child module declarations, and the stable `color_popup_body_sections(...)` re-export. The
new `ecosystem/fret-ui-editor/src/controls/color_edit/popup/body/sections/assembly.rs` owns section
call sequencing, picker/options/preview/eyedropper/numeric/history/preset/standalone-alpha routing,
`has_side_preview`, and final `ColorPopupContentArgs` assembly. Popup body width policy, child
section behavior, public ColorEdit APIs, and IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the hub/assembly split.

2026-06-05 editor ColorEdit frame affordance owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame.rs` now keeps keyed frame
orchestration, local state/model setup, child construction, delivered-drop application, overlay
requests, and root layout. The new
`ecosystem/fret-ui-editor/src/controls/color_edit/element/affordance.rs` owns popup visible-content
merging, drag/drop tooltip/copy/eyedropper enablement, and swatch enabled/focusable derivation.
Public ColorEdit APIs and IMUI facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py`
plus `imui_surface_policy` freeze the frame/affordance split.

2026-06-05 editor ColorEdit picker test owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests/picker.rs` now keeps only child module
routing. `tests/picker/bars.rs` owns SV/hue/alpha bar local-position mapping,
`tests/picker/hue_wheel.rs` owns hue-wheel ring target coverage,
`tests/picker/hue_wheel_triangle.rs` owns barycentric SV triangle coverage, and
 `tests/picker/preview_alpha.rs` owns alpha-preserving HSV edits, checkerboard stability,
preview-alpha policy, original restore behavior, and alpha a11y text coverage. Assertions,
production ColorEdit code, public ColorEdit APIs, and IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the picker test hub split.

2026-06-05 editor ColorEdit popup policy test owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests/popup_policy.rs` now keeps only child
module routing. `tests/popup_policy/defaults.rs` owns popup option defaults, side-preview default
and ratio coverage, alpha preview variants, and tooltip/copy defaults.
`tests/popup_policy/visibility.rs` owns popup visible-content predicate coverage, and
`tests/popup_policy/runtime.rs` owns runtime override synchronization, hidden-picker enforcement,
and disabled options-surface policy. Assertions, production ColorEdit code, public ColorEdit APIs,
and IMUI facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py` plus
`imui_surface_policy` freeze the popup policy test hub split.

2026-06-05 editor ColorEdit numeric test owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/tests/numeric.rs` now keeps only child module
routing. `tests/numeric/modes.rs` owns popup numeric mode ordering, `tests/numeric/hex.rs` owns
RGB/RGBA hex parsing, alpha-preserving preset conversion, and numeric readout coverage,
`tests/numeric/input.rs` owns RGB/HSV text input parsing and rejection coverage, and
`tests/numeric/conversion.rs` owns primary-color, grayscale, and palette roundtrip HSV conversion
coverage. Assertions, production ColorEdit code, public ColorEdit APIs, and IMUI facade APIs remain
unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the numeric
test hub split.

2026-06-05 editor ColorEdit numeric model owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/model/numeric.rs` now keeps only child module
routing and stable re-exports. `model/numeric/mode.rs` owns `ColorNumericInputMode`, mode metadata,
and popup numeric input mode lists. `model/numeric/text.rs` owns RGB/HSV readout formatting.
`model/numeric/parse.rs` owns RGB/HSV numeric text parsing, channel/unit validation, and
alpha-preserving color conversion routing. Public ColorEdit APIs, IMUI facade APIs, and existing
`model::{ColorNumericInputMode, color_numeric_text, parse_color_numeric_input, ...}` imports remain
unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the numeric
model hub split.

2026-06-05 editor ColorEdit drag-source phase owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/drag_drop/source/handlers.rs` now keeps only
child module routing, `install_color_drag_source(...)`, drag kind derivation, and threshold
distance math. `handlers/down.rs` owns left-button drag startup, cross-window/same-window begin
calls, and stale active-session cleanup. `handlers/move_phase.rs` owns pointer move routing,
thresholded drag activation, dragging/canceled phase updates, active store writes, cancel cleanup,
and redraw. `handlers/up.rs` owns left-button release handling, delivered-drop insertion, drag
cleanup, redraw, and skip-activate behavior. Drag-source installation, drag threshold semantics,
delivery recording, and public ColorEdit / IMUI facade APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the phase split.

2026-06-05 editor ColorEdit keyed frame overlay owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame.rs` now keeps keyed frame
orchestration, local state/model setup, theme density/popup padding resolution, current color/hex
projection, drag/drop store setup, test-id projection, input/swatch construction, delivered-drop
application, and root layout handoff. The new
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame/overlays.rs` owns
`ColorEditFrameOverlayArgs`, `request_color_edit_frame_overlays(...)`, popup request routing,
tooltip request routing, copy-menu request routing, popup runtime option handoff, callback
forwarding, and popup/tooltip/copy/eyedropper test-id forwarding. Popup, tooltip, copy-menu,
eyedropper, palette/history, drag/drop, runtime popup option, and public ColorEdit / IMUI facade
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy`
freeze the frame/overlay split.

2026-06-05 editor ColorEdit keyed frame children owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame.rs` now delegates input/swatch
construction through `ColorEditFrameChildrenArgs` and keeps keyed frame orchestration, local
state/model setup, theme density/popup padding resolution, current color/hex projection, drag/drop
store setup, test-id projection, affordance resolution, delivered-drop application, overlay routing,
and root layout handoff. The new
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame/children.rs` owns
`ColorEditFrameChildren`, `color_edit_frame_children(...)`, `ColorEditInputArgs` assembly,
`ColorEditSwatchArgs` assembly, input/swatch test-id forwarding, and swatch affordance forwarding.
Hex input behavior, swatch behavior, popup/tooltip/copy model forwarding, reference model
forwarding, drag/drop store forwarding, test-id routing, and public ColorEdit / IMUI facade
behavior remain unchanged, and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy`
freeze the frame/children split.

2026-06-05 editor ColorEdit keyed frame setup owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame.rs` is now a keyed frame
orchestration owner: it delegates setup/runtime snapshot construction through
`color_edit_frame_setup(...)`, child construction through `color_edit_frame_children(...)`, overlay
routing through `request_color_edit_frame_overlays(...)`, and still owns delivered-drop application
plus root layout handoff. The new
`ecosystem/fret-ui-editor/src/controls/color_edit/element/frame/setup.rs` owns
`ColorEditFrameSetup`, local state model allocation, editor density and popup padding resolution,
current color/hex projection, drag/drop store setup and pruning, drag threshold resolution, root
test-id derivation, popup runtime option synchronization, palette/history/eyedropper presence
projection, and `ColorEditFrameAffordances` resolution. Local model identity, theme token fallback,
drag/drop setup, runtime popup option behavior, affordance gates, and public ColorEdit / IMUI
facade behavior remain unchanged, and `tools/gate_imui_workstream_source.py` plus
`imui_surface_policy` freeze the frame/setup split.

2026-06-05 editor ColorEdit public records child-owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/records.rs` is now a public record re-export hub:
it declares private palette, drag/drop, and eyedropper child modules and keeps the existing public
`ColorEdit` record type names available through the same root exports. The new
`records/palette.rs` owns `ColorEditPaletteEntry`, default palette constants, and
`default_color_edit_palette(...)`; `records/drag_drop.rs` owns
`ColorEditDragDropComponents`, `ColorEditDragDropPayload`, `ColorEditPaletteSlotDrop`, and
`OnColorEditPaletteSlotDrop`; and `records/eyedropper.rs` owns
`ColorEditEyedropperRequest` plus `OnColorEditEyedropper`. Public import paths, default palette
contents, drag/drop payload component behavior, palette slot drop conversion, eyedropper sample
alpha policy, and IMUI facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py`
plus `imui_surface_policy` freeze the records hub/child split.

2026-06-05 editor ColorEdit popup swatch slot child-owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/popup/swatches/slot.rs` now keeps preset swatch
pressable/root orchestration, focus/a11y props, drag-source install, drop-target hover updates,
test-id routing, and final a11y value assignment only. The new `slot/activation.rs` owns preset
activation writeback into the color model, hex draft, error state, popup open state, and redraw.
The new `slot/delivery.rs` owns delivered color drop retrieval, `ColorEditPaletteSlotDrop`
construction, app callback dispatch, and redraw. The new `slot/visual.rs` owns selected/drop-over
preview container chrome, clipped fill layout, border/ring selection, and preview stack mounting.
Preset activation semantics, drag/drop payload publication, palette slot drop callback behavior,
preview rendering, a11y value formatting, and public ColorEdit / IMUI facade APIs remain unchanged,
and `tools/gate_imui_workstream_source.py` plus `imui_surface_policy` freeze the slot child-owner
split.

2026-06-05 editor ColorEdit popup option types/runtime owner split result:
`ecosystem/fret-ui-editor/src/controls/color_edit/options/popup.rs` now keeps
`ColorEditPopupOptions`, visible-content predicates, alpha/picker option gates, runtime defaults
construction, runtime override application, and stable re-exports only. The new
`options/popup/types.rs` owns `ColorEditPopupPicker`, `ColorEditPopupNumericInputs`,
`ColorEditPopupSidePreview`, their defaults, and the side-preview helper predicates. The new
`options/popup/runtime.rs` owns `ColorEditPopupRuntimeOptions` plus `sync_defaults(...)`.
Picker/numeric/side-preview defaults, popup visible-content behavior, picker runtime override
semantics, runtime default synchronization, public option type names, and public ColorEdit / IMUI
facade APIs remain unchanged, and `tools/gate_imui_workstream_source.py` plus
`imui_surface_policy` freeze the popup options child-owner split.

2026-06-05 supporting editor proof workbench shell owner split result:
`apps/fret-examples/src/imui_editor_proof_demo.rs` now keeps proof rendering, editor-control
composition, authoring parity sections, and supporting proof routing while delegating dock/window
shell behavior to `apps/fret-examples/src/imui_editor_proof_demo/workbench_shell.rs`. The new
owner carries the dock panel registry, dock test-id derivation, dock graph ensure/reset policy,
single-window floating fallback graph, auxiliary-window bootstrap service, window-create specs,
dock lifecycle callbacks. The same slice also moves the local `slotmap::Key` import to the
`fret-launch` diag screenshot owner that calls `AppWindowId::data()`. The canonical
`imui_editor_workbench_demo` product route, supporting dense proof rendering, dock graph defaults,
auxiliary window behavior, and public IMUI/editor APIs remain unchanged, and
`tools/gate_imui_workstream_source.py` plus
`imui_editor_proof_workbench_shell_surface` freeze the shell owner boundary.

2026-06-05 DevTools Demo/Metrics/Debug workflow projection owner split result:
`apps/fret-devtools/src/demo_metrics_debug.rs` now keeps route assembly, action-row UI, panel state
reads, and stable public re-exports for the Demo/Metrics/Debug route. The new
`apps/fret-devtools/src/demo_metrics_debug/workflow.rs` owns workflow readiness, workflow status,
workflow result-action, and workflow artifact-action line projection for the always-visible
Demo/Metrics/Debug DevTools surface. Route line ordering, action metadata ownership, copy command
IDs, workflow readiness reasons, result/artifact command strings, panel button enablement, DevTools
native imports, and first-open route metadata remain unchanged, and
`tools/gate_imui_workstream_source.py` plus the existing `fret-devtools` Demo/Metrics/Debug tests
freeze the workflow child-owner boundary.

2026-06-05 DevTools product/first-open gate owner realignment result:
`tools/diag_gate_imui_product_chain.py` and
`tools/diag_gate_imui_p2_devtools_first_open.py` now validate the same shared first-open owners
used by the product code: `fret_first_open::product_workflow::*` for product-chain workflow
metadata, `fret_first_open::demo_metrics_debug::*` for Demo/Metrics/Debug route/action metadata,
and `apps/fret-devtools/src/demo_metrics_debug/workflow.rs` for workflow readiness/status/result/
artifact line projection. The focused discovery gates no longer require local duplicated workflow
strings, route IDs, action struct fields, or workflow projection text in the old route assembly
owner.

2026-06-05 collection proof surface-test owner realignment result:
`apps/fret-examples/tests/imui_editor_collection_*_surface.rs` now treats the collection proof
surface as the composed demo-local owner set: `collection.rs`, `collection/geometry.rs`, and
`collection/readouts.rs`. This keeps box-select, command package, context-menu, delete, keyboard,
rename, select-all, text-role, and zoom tests aligned with the existing child-owner split while
still rejecting shared-helper widening. The keyboard surface now checks the
`ImUiMultiSelectState::first_selected()` accessor instead of the removed direct selected-field
access, and `tools/gate_imui_editor_collection_source.py` now validates the same composed owner
bundle.

2026-06-05 collection proof model owner split result:
`apps/fret-examples/src/imui_editor_proof_demo/collection/models.rs` now owns collection proof
state/model slot registration: selection, stored assets, reverse order, box-select state, keyboard
state, zoom extent, scroll handle, context-menu anchor, rename session/draft/focus/status, command
status, and drop status. `collection.rs` keeps render assembly, pointer/key/menu handling, drag
preview/drop policy, inline rename behavior, and command execution. The split leaves app-owned
collection proof behavior unchanged while reducing the root owner and making model slot drift
visible in `tools/gate_imui_editor_collection_source.py`.
