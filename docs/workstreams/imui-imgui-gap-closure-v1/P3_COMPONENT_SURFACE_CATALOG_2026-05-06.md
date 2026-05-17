# P3 Component Surface Catalog - 2026-05-06

Status: component surface audit; partially superseded by closed proof lanes
Last updated: 2026-05-17

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
`table_column_visibility_menu_items(...)`; automatic header context-menu popup wiring,
persistence, freeze panes, and old columns API remain candidate-only.

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
| Tree / disclosure | `collapsing_header`, `tree_node`, explicit `TreeNodeOptions::level` | Covered with Fret-native explicit identity/depth; do not copy implicit indent/ID stacks |
| Child windows / scrolling | `child_region`, `scroll`, `virtual_list`, `ChildRegionResize{X,Y}Options` | Covered for keyed scrollable panes and manual axis resize; Dear ImGui child flags such as auto-resize, clipping-return, and nav flattening stay behavior-specific candidates |
| In-window floating windows / overlay areas | `floating_layer`, `floating_area`, `window`, `window_with_options`, `FloatingWindowOptions` | Covered for in-window drag, z-order hit-testing, focus/input policy, close, resize, and collapse; OS-window tear-out / multi-viewport parity stays in docking/runner lanes |
| Menus / menu bars / popups / modals | `menu_bar`, `begin_menu`, `begin_submenu`, menu items, `open_popup`, `begin_popup_menu`, context menu helpers, modal helpers | Covered at policy layer; dismissal/focus policy stays in ecosystem |
| Tooltips | `tooltip_text`, `tooltip`, `TooltipOptions` | Covered enough for current response-driven usage |
| Tabs | `tab_bar`, `ImUiTabBar`, `tab_item`, response reporting | Covered for current shell/editor proofs |
| Tables | `table`, `ImUiTable`, `TableColumn`, `TableColumn::hidden`, `TableColumn::with_visible`, `ImUiTableColumnVisibilityState`, `table_column_visibility_menu_item`, `table_column_visibility_menu_items`, `TableOptions::striped`, `TableRowOptions::background`, `TableCellOptions::background`, sort/resize/header responses, virtual-list support | Covered for basic/sort/resize/striped-row, static column visibility, runtime hideable-column helper proof, header menu-item/group composition, and explicit row/cell background proof paths; remaining advanced table flags should be split by proof |
| Drag and drop | response-driven `drag_source` / `drop_target` with typed payloads | Covered with Fret-native response style; do not copy begin/end mutable payload grammar |
| Draw list / images | `debug_draw`, `ImUiDebugDrawList`, paths, channels, mesh, image/SVG variants | Strong local coverage; keep feature growth in debug-draw follow-ons |
| Color edit / picker | `fret-ui-editor::ColorEdit` through `fret::imui::editor::color_edit` | Covered as editor-control policy, not generic kit vocabulary |
| Property editor | `PropertyGroup`, `PropertyGrid`, `InspectorPanel`, vector/transform controls via editor adapters | Covered through editor composites |

## Candidate-Only Gaps

These are real Dear ImGui-class areas, but current source evidence does not justify immediate
public helper widening:

1. **ListBox as a named widget**
   - Dear ImGui treats list boxes as a thin child-window/selectable convenience.
   - Fret already has `child_region`, `selectable`, `multi_selectable`, and `virtual_list`.
   - Open only if two proof surfaces repeat the same list-box chrome/selection setup.
2. **PlotLines / PlotHistogram**
   - Fret has plot/chart ecosystem directions outside IMUI.
   - Start with an app or cookbook proof using existing plot/chart crates before adding an IMUI
     wrapper.
3. **Style editor / style selector**
   - Dear ImGui exposes these as built-in tools.
   - Fret should use theme/editor tooling and diagnostics/devtools lanes; do not freeze a generic
     style editor API from this audit.
4. **Advanced table flags**
   - Sorting, resize handles, alternating row backgrounds, explicit per-row/per-cell background
     override targets, and a narrow runtime hideable-column helper already have proof.
   - Automatic header context-menu popup wiring, freeze panes, persistence, and old columns API
     should stay narrow follow-ons.
5. **Child-region flag mirrors beyond manual resize**
   - `ResizeY` and `ResizeX` now have closed proof lanes.
   - Auto-resize, nav flattening, and clipping-return behavior still need behavior-specific proof
     and gates.

## Source-Backed Facts

- `ecosystem/fret-ui-kit/src/imui.rs` owns the large policy-heavy re-export surface: options,
  response types, debug draw, floating/window helpers, virtual-list types, tables, tabs, and
  multi-select state.
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
  section composition for callers that own a popup/menu surface. Do not treat Dear ImGui `RowBg`
  or visibility parity as wholly missing; the remaining table axes are automatic header
  context-menu popup wiring, freeze panes, persistence, and old columns API shape, which still need
  narrow proofs.
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

- `imui-list-box-proof-v1`
- `imui-plot-adapter-proof-v1`
- `imui-table-advanced-flags-v1`
- `imui-child-region-auto-resize-v1`
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
