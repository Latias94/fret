# P3 Component Surface Catalog - 2026-05-06

Status: component surface audit; partially superseded by closed proof lanes
Last updated: 2026-05-27

Status note (2026-05-16): this catalog remains the current component-surface gap map, but the
image-item and child-region manual-resize candidates below have since landed in narrow proof lanes:
`imui-image-item-proof-v1`, `imui-child-region-resize-y-v1`, and
`imui-child-region-resize-x-v1`. Selectable forced-highlight visuals also landed in
`imui-selectable-highlight-policy-v1`. The in-window floating-window posture has also been
refreshed: current source/tests already cover z-order hit-testing, focus-on-click vs activation,
no-inputs / pointer-pass-through policy, close, resize, and collapse behavior. The table
advanced-gap wording has also been narrowed: `TableOptions::striped` already covers alternating row
backgrounds, explicit per-row/per-cell background overrides now have a narrow proof through
`TableRowOptions::background` and `TableCellOptions::background`, and static author-declared
column visibility now has a narrow proof through `TableColumn::hidden()` /
`TableColumn::with_visible(bool)`. Runtime hideable-column policy now has a narrow helper proof
through `ImUiTableColumnVisibilityState`, and header menu-item composition has a bridge through
`table_column_visibility_menu_item(...)` plus a repeated-section helper through
`table_column_visibility_menu_items(...)`; automatic header context-menu popup wiring now has a
default helper through `table_column_visibility_header_context_menu(...)`. Runtime visibility also
has a caller-owned snapshot/restore seam through `TableColumnVisibilitySnapshot` /
`TableColumnVisibilityEntry`; storage and schema placement remain app/editor-owned. Column
pinning now has a narrow freeze-pane proof through `TableColumn::pinned_left()` /
`TableColumn::pinned_right()`, with pinned left/right groups staying outside the shared center
horizontal scroll handle. The old columns API shape has since been closed by making `TableColumn`
builder/accessor-first with private fields. Child-region auto-height and auto-width have also been narrowed.
Leaving the child-region size unconstrained on an axis now has focused composition gates proving
the AutoResizeY-equivalent layout posture and AutoResizeX-equivalent layout posture, so only always
auto-resize behavior, clipping-return, and nav flattening remain candidates.

Status note (2026-05-27): ListBox, plot adapter, and style/theme preset proof are no longer
candidate-only gaps. The ListBox container proof landed in `fret-ui-kit::imui`, the optional plot
adapter landed behind `fret-plot/imui`, and the style/theme preset picker landed in
`fret-ui-editor` plus the canonical editor workbench. Keep plot sugar and arbitrary runtime-global
style editing deferred unless a new proof-led lane shows repeated product pressure.

## Decision

Do not open a broad "finish all Dear ImGui widgets" lane. The current IMUI component surface is
already broad enough for the active editor-proof path; the remaining gaps should be split by
behavior and proof pressure.

Keep the owner split:

- generic immediate widgets and response vocabulary stay in `fret-ui-kit::imui`,
- editor-grade controls stay declarative-first in `fret-ui-editor`, with `fret-ui-editor::imui`
  exposing thin adapters,
- app/domain collection behavior stays app-owned until a second first-party IMUI proof repeats the
  same behavior,
- plotting/charting should use existing plot/chart ecosystem crates first, with an IMUI adapter
  only after a concrete proof needs it.

## Current Coverage Map

| Dear ImGui area | Current Fret surface | Verdict |
| --- | --- | --- |
| Text / separators / bullets | `text`, `text_wrapped`, `bullet_text`, `separator`, `separator_text` in `UiWriterImUiFacadeExt` | Covered for default single-line text items, explicit wrapped text, and current teaching/proof surfaces |
| Main controls | `button`, `small_button`, `arrow_button`, `invisible_button`, command/action buttons | Covered; command/action variants are Fret-native additions |
| Boolean controls | `checkbox_model`, `radio`, `switch_model` | Covered; switch is a Fret policy addition, not Dear ImGui parity debt |
| Input text / textarea | `input_text_model`, picker/history/completion helpers, `textarea_model`, input filters, undo command policy | Covered for current needs; mutable-buffer callback grammar remains intentionally absent |
| Slider / drag value | `slider_f32_model` in kit, richer typed `Slider` / `DragValue` adapters in editor | Covered through split kit/editor ownership; generic numeric breadth belongs in editor controls first |
| Combo / selectable / multi-select | `combo`, `combo_model`, `selectable`, `multi_selectable`, `SelectableOptions::highlighted`, `ImUiMultiSelectState` | Covered for current examples; full app collection helper and broader selectable flag mirrors remain candidate-only |
| List boxes | `list_box`, `list_box_with_options`, `ListBoxOptions`, list-box semantics over hosted children | Covered as a Dear ImGui `BeginListBox`-style container proof; selection, filtering, and app collection behavior stay caller-owned |
| Tree / disclosure | `collapsing_header`, `tree_node`, explicit `TreeNodeOptions::level` | Covered with Fret-native explicit identity/depth; do not copy implicit indent/ID stacks |
| Child windows / scrolling | `child_region`, `scroll`, `virtual_list`, `ChildRegionResize{X,Y}Options` | Covered for keyed scrollable panes, unconstrained-axis auto-size layout, and manual axis resize; more specific Dear ImGui child behavior such as always auto-resize, clipping-return, and nav flattening stays behavior-specific candidate work |
| In-window floating windows / overlay areas | `floating_layer`, `floating_area`, `window`, `window_with_options`, `FloatingWindowOptions` | Covered for in-window drag, z-order hit-testing, focus/input policy, close, resize, and collapse; OS-window tear-out / multi-viewport parity stays in docking/runner lanes |
| Menus / menu bars / popups / modals | `menu_bar`, `begin_menu`, `begin_submenu`, menu items, `open_popup`, `begin_popup_menu`, context menu helpers, modal helpers | Covered at policy layer; dismissal/focus policy stays in ecosystem |
| Tooltips | `tooltip_text`, `tooltip`, `TooltipOptions` | Covered enough for current response-driven usage |
| Tabs | `tab_bar`, `ImUiTabBar`, `tab_item`, response reporting | Covered for current shell/editor proofs |
| Tables | `table`, `ImUiTable`, `TableColumn`, `TableColumn::hidden`, `TableColumn::with_visible`, `TableColumn::pinned_left`, `TableColumn::pinned_right`, `ImUiTableColumnVisibilityState`, `TableColumnVisibilitySnapshot`, `table_column_visibility_menu_item`, `table_column_visibility_menu_items`, `table_column_visibility_header_context_menu`, `TableOptions::striped`, `TableOptions::horizontal_scroll`, `TableRowOptions::background`, `TableCellOptions::background`, sort/resize/header responses, virtual-list support | Covered for basic/sort/resize/striped-row, static column visibility, runtime hideable-column helper proof, caller-owned visibility snapshot/restore, header visibility-menu composition, column pinning/freeze-pane seam, explicit horizontal-scroll seam, and explicit row/cell background proof paths; remaining advanced table flags should be split by proof |
| Data plotting adapter | `fret-plot` optional `imui` feature with thin `UiWriter` adapters over declarative plot panels | Covered as an opt-in ecosystem adapter; no `fret-imui` or `fret-ui-kit::imui` plot dependency, and root `fret::imui` plot sugar stays deferred |
| Drag and drop | response-driven `drag_source` / `drop_target` with typed payloads | Covered with Fret-native response style; do not copy begin/end mutable payload grammar |
| Draw list / images | `debug_draw`, `ImUiDebugDrawList`, paths, channels, mesh, image/SVG variants | Strong local coverage; keep feature growth in debug-draw follow-ons |
| Color edit / picker | `fret-ui-editor::ColorEdit` through `fret::imui::editor::color_edit` | Covered as editor-control policy, not generic kit vocabulary |
| Property editor | `PropertyGroup`, `PropertyGrid`, `InspectorPanel`, vector/transform controls via editor adapters | Covered through editor composites |
| Style/theme preset picker | `fret-ui-editor::imui::editor_theme_preset_picker`, editor theme preset metadata, reversible preset install/reapply | Covered as editor-owned theme tooling and canonical workbench affordance; no generic runtime `ImGuiStyle` clone or mutable style stack |

## Closed / Narrowed Former Candidate Areas

These Dear ImGui-class areas were earlier candidates, but current source evidence has since closed
or narrowed them:

1. **ListBox as a named widget**
   - Closed as a kit-owned container proof through `list_box`, `list_box_with_options`, and
     `ListBoxOptions`.
   - The proof deliberately stops at hosted list-box semantics; selection/filtering/collection
     command packages remain caller-owned until repeated product pressure proves a shared helper.
2. **PlotLines / PlotHistogram**
   - Narrowed to an optional `fret-plot/imui` adapter over existing declarative plot panels.
   - Root `fret::imui` plot sugar and canonical-workbench plot adoption remain deferred until
     repeated authoring friction appears.
3. **Style editor / style selector**
   - Closed as editor-owned preset/theme tooling through `fret-ui-editor`, not as a generic
     `fret-ui-kit::imui` or runtime-global style editor.
   - Do not copy Dear ImGui's `GetStyle`, `PushStyleVar`, or mutable global style stack.

## Remaining Candidate-Only Gaps

These are still real Dear ImGui-class areas, but current source evidence does not justify immediate
public helper widening:

1. **Advanced table flags**
   - Sorting, resize handles, alternating row backgrounds, explicit per-row/per-cell background
     override targets, a narrow runtime hideable-column helper, caller-owned visibility
     snapshot/restore, default header visibility-menu wiring, and column pinning/freeze-pane seam
     already have proof.
   - The old public `TableColumn` field-bag API shape is closed; do not reopen it as a follow-on
     unless a new public construction failure appears.
2. **Child-region flag mirrors beyond manual resize**
   - `ResizeY` and `ResizeX` now have closed proof lanes.
   - Basic AutoResizeY-equivalent and AutoResizeX-equivalent layout is covered by the
     unconstrained-axis child-region composition gates.
   - Always auto-resize, nav flattening, and clipping-return behavior still need
     behavior-specific proof and gates.

## Source-Backed Facts

- `ecosystem/fret-ui-kit/src/imui.rs` owns the large policy-heavy re-export surface: options,
  response types, debug draw, floating/window helpers, virtual-list types, tables, tabs, and
  multi-select state.
- `ecosystem/fret-ui-kit/src/imui/list_box_controls.rs` owns the ListBox container proof while
  `ListBoxOptions` stays limited to layout, scroll, and diagnostics semantics.
- `ecosystem/fret-plot/src/imui.rs` owns the optional plot IMUI adapter; `fret-imui` and
  `fret-ui-kit::imui` still have no plot dependency.
- `ecosystem/fret-ui-editor/src/controls/editor_theme_preset_picker.rs` and
  `ecosystem/fret-ui-editor/src/imui.rs` own the style/theme preset picker proof, with the
  canonical workbench exposing that affordance through editor-owned adapters.
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` remains the public immediate authoring hub for
  the trait, text, popups, drag/drop, windows, and debug draw; the inherent `ImUiFacade` wrappers
  for button/actions, menu items, selection/combo, disclosure, text/value/boolean models, and
  structural containers now live in `ecosystem/fret-ui-kit/src/imui/facade_writer/`. The
  floating/popup trait default implementation bodies now live in
  `ecosystem/fret-ui-kit/src/imui/facade_writer/floating_popup.rs`.
- `window(...)` is no longer a v1 posture with z-order/focus arbitration deferred: current
  `fret-imui` floating tests cover bring-to-front hit-test order, focus-on-click independent from
  activation, no-inputs / pointer-pass-through behavior, close, resize, and collapse.
- `TableOptions::striped` is already the current alternating row-background policy, while
  `TableRowOptions::background` / `TableCellOptions::background` cover explicit per-row/per-cell
  override targets. `TableColumn::hidden()` / `TableColumn::with_visible(bool)` cover static
  author-declared visibility, `ImUiTableColumnVisibilityState` gives runtime visibility overrides
  a narrow stable-id helper before table render, and `table_column_visibility_menu_item(...)`
  plus `table_column_visibility_menu_items(...)` provide the menu checkbox bridge and repeated
  section composition. `table_column_visibility_header_context_menu(...)` provides the default
  header context-menu popup wiring, and `TableColumnVisibilitySnapshot` provides serde-friendly
  caller-owned save/restore data without moving file storage or a mutable table runtime into
  `fret-imui`. `TableColumn::pinned_left()` / `TableColumn::pinned_right()` plus
  `TableOptions::horizontal_scroll` provide the narrow freeze-pane seam without moving a mutable
  table runtime into `fret-imui`. `TableColumn` is now builder/accessor-first with private fields,
  so the old public field-bag API shape is closed. Do not treat Dear ImGui `RowBg`, visibility
  parity, column pinning, or the old columns API shape as wholly missing.
- `ecosystem/fret-ui-editor/src/imui.rs` is only a thin adapter layer that forwards editor controls
  and composites through `into_element(...)`.
- `repo-ref/imgui/imgui.h` still groups the upstream surface by Windows, Child Windows, Widgets,
  Menus, Tooltips, Popups, Tables, Tab Bars, Drag and Drop, query utilities, and Debug Utilities.
- `repo-ref/imgui/imgui_demo.cpp` remains useful for usage pressure, but Fret should not import its
  mutable stack grammar by default.

## Follow-On Threshold

Open a component-specific follow-on only when the proposal can name:

1. the exact missing component or behavior,
2. the owner layer (`kit`, `editor`, `docking`, or app/domain proof),
3. at least two proof surfaces unless it is a thin adapter over an existing declarative editor
   control,
4. one focused smoke/source gate,
5. and the Dear ImGui reference section being matched.

Suggested follow-on names:

- `imui-child-region-auto-resize-specific-v1`
- `imui-child-region-visibility-return-v1`

## Gates

Suggested audit/gate commands:

```powershell
rg --files ecosystem/fret-ui-kit/src/imui ecosystem/fret-ui-kit/tests
rg -n "pub use debug_draw_controls|pub use options|pub use response|pub use tab_family_controls::ImUiTabBar|pub use table_controls" ecosystem/fret-ui-kit/src/imui.rs
rg -n "fn (text|text_wrapped|button|small_button|arrow_button|checkbox_model|radio|switch_model|slider_f32_model|combo|combo_model|selectable|multi_selectable|tree_node|collapsing_header|child_region|virtual_list|table|tab_bar|open_popup|begin_popup|tooltip|drag_source|drop_target|debug_draw)" ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer
rg -n "pub fn (text_field|checkbox|color_edit|drag_value|numeric_input|slider|enum_select|property_grid|gradient_editor|inspector_panel)" ecosystem/fret-ui-editor/src/imui.rs
rg -n "Widgets: Text|Widgets: Main|Widgets: Combo Box|Widgets: Trees|Widgets: Selectables|Widgets: List Boxes|Widgets: Data Plotting|Widgets: Menus|Tooltips|Popups, Modals|Tables|Tab Bars|Drag and Drop|Debug Utilities" repo-ref/imgui/imgui.h
cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --test imui_table_smoke --test imui_disclosure_smoke --test imui_textarea_smoke --test imui_drag_drop_smoke --test imui_virtual_list_smoke --test imui_debug_draw_smoke --test imui_tooltip_smoke --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast
```

## Gate Results

2026-05-06 local results:

- `rg --files ecosystem/fret-ui-kit/src/imui ecosystem/fret-ui-kit/tests` passed and confirmed the
  current IMUI source/test module set.
- `rg -n "pub use debug_draw_controls|pub use options|pub use response|pub use tab_family_controls::ImUiTabBar|pub use table_controls" ecosystem/fret-ui-kit/src/imui.rs`
  passed and found the kit-level re-export anchors.
- `rg -n "fn (text|text_wrapped|button|small_button|arrow_button|checkbox_model|radio|switch_model|slider_f32_model|combo|combo_model|selectable|multi_selectable|tree_node|collapsing_header|child_region|virtual_list|table|tab_bar|open_popup|begin_popup|tooltip|drag_source|drop_target|debug_draw)" ecosystem/fret-ui-kit/src/imui/facade_writer.rs ecosystem/fret-ui-kit/src/imui/facade_writer`
  passed and found the current facade method anchors across the root file and split owner modules.
- `rg -n "pub fn (text_field|checkbox|color_edit|drag_value|numeric_input|slider|enum_select|property_grid|gradient_editor|inspector_panel)" ecosystem/fret-ui-editor/src/imui.rs`
  passed and found the editor adapter anchors.
- `rg -n "Widgets: Text|Widgets: Main|Widgets: Combo Box|Widgets: Trees|Widgets: Selectables|Widgets: List Boxes|Widgets: Data Plotting|Widgets: Menus|Tooltips|Popups, Modals|Tables|Tab Bars|Drag and Drop|Debug Utilities" repo-ref/imgui/imgui.h`
  passed and found the upstream Dear ImGui category anchors.
- `cargo nextest run -p fret-ui-kit --features imui --test imui_button_smoke --test imui_combo_smoke --test imui_table_smoke --test imui_disclosure_smoke --test imui_textarea_smoke --test imui_drag_drop_smoke --test imui_virtual_list_smoke --test imui_debug_draw_smoke --test imui_tooltip_smoke --no-fail-fast`
  passed: 14 tests run, 14 passed.
- `cargo nextest run -p fret-ui-editor --features imui --test imui_adapter_smoke --test imui_surface_policy --no-fail-fast`
  passed: 3 tests run, 3 passed.
