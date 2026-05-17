# ImUi Dear ImGui Gap Closure v1 - Milestones

Status: Active
Last updated: 2026-05-17

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
  2026-05-14 inspector follow-up result: `InspectorPanelCx` now exposes query behavior through
  methods and keeps `query_lower` private.
  2026-05-16 child-region resize result: `imui-child-region-resize-y-v1` and
  `imui-child-region-resize-x-v1` are the closed proof lanes for axis-specific manual child-region
  resize. Height/width state stays app-owned through response helpers, and broader child-region
  behavior such as auto-resize, clipping-return, or nav-flattening remains candidate-only.
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
  semantics. Freeze panes, persistence, header-menu policy, and old columns API remain
  advanced-table candidates.
  2026-05-16 table header text result: sortable and plain table header labels also use
  `text_table_cell(...)`, preserving the same compact single-line ellipsis semantics as body cells.
  2026-05-16 static table column visibility result: `TableColumn::hidden()` and
  `TableColumn::with_visible(bool)` now cover author-declared hidden columns without copying Dear
  ImGui's mutable table runtime. Hidden columns still consume submitted row cells in declared order
  but skip header/body rendering and header responses; runtime hideable-column policy is covered
  by the follow-up state helper below, while persistence and header-menu policy stay
  candidate-only.
  2026-05-17 runtime table column visibility result: `ImUiTableColumnVisibilityState` now covers
  runtime stable-id visibility overrides as a policy-layer helper in `fret-ui-kit::imui`. It
  produces an adjusted `TableColumn` list and reuses the existing hidden-column render contract;
  persistence, header-menu policy, freeze panes, and old columns API shape stay candidate-only. A
  `fret-imui` composition gate proves the helper can drive table rendering while the runtime
  facade remains policy-light.
  2026-05-17 table visibility menu-item result: `table_column_visibility_menu_item(...)` now
  bridges `TableColumn`, existing checkbox menu item behavior, and
  `ImUiTableColumnVisibilityState`. Callers still own where that menu is presented; automatic
  header context-menu popup wiring, persistence, freeze panes, and old columns API shape stay
  candidate-only.
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
  2026-05-16 trigger label reuse result: IMUI tab triggers and menubar triggers now reuse
  `text_button_label(...)`; selectable/menu item row labels stayed out of that role because they
  are command/list rows, not button labels.
  2026-05-16 list row text role result: `text_list_row_label(...)` is now the shared dense row
  label role for menu items, selectables, and tree/disclosure rows. It preserves regular `text-sm`
  styling with fill-width, min-width-zero, single-line ellipsis behavior, so row labels do not wrap
  or grow row height under resize.
  2026-05-16 menu shortcut readout reuse result: IMUI menu shortcut labels now reuse
  `text_control_readout(...)` as muted compact auxiliary readouts, keeping shortcut text inside the
  stable control-readout role instead of adding another menu-specific text policy.
  2026-05-17 section chrome label text result: `text_section_chrome_label(...)` now owns compact
  separator/section chrome labels in `fret-ui-kit::declarative::text`. IMUI `separator_text`
  labels use that shared role, so section chrome stays single-line, shrinkable, and ellipsis-based
  under resize instead of inheriting default word wrapping.
  2026-05-17 chrome title text result: `text_chrome_title(...)` now owns fill-width floating
  window title-bar text. Resizable floating titles keep fill/grow/min-width-zero behavior, while
  non-resizable titles reuse the section/chrome label role instead of local `TextProps`.
  2026-05-17 chrome glyph text result: `text_chrome_glyph(...)` now owns compact fixed-slot
  chrome glyph text in `fret-ui-kit::declarative::text`. Disclosure/tree indicators use that
  shared role, so glyph-only chrome stays single-line and clipped without local `TextProps`.
  2026-05-16 text role source-gate result: `tools/gate_imui_workstream_source.py` now freezes the
  remaining direct `TextProps::new(...)` constructors under `fret-ui-kit::imui` behind an explicit
  allowlist, forcing future compact text policy additions through the shared role vocabulary or an
  intentional gate update.
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
  2026-05-17 gallery retained-table torture text result: the UI Gallery retained-table torture page
  now uses `text_table_cell(...)` for fixed table cells and `control_readout_text(...)` for table
  state readouts, so the visible retained-table stress surface no longer teaches bare fixed-cell
  text under resize.
  2026-05-17 gallery data-table torture text result: the UI Gallery DataTable torture page now
  routes fixed cells through `text_table_cell(...)` in both retained and non-retained render paths,
  and table sorting/filter/pinning status lines through `control_readout_text(...)`.
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
  2026-05-17 gallery editor preview text result: code-editor, Markdown, and Web IME preview
  headers now use paragraph text for prose, control readout text for fixed status/debug values,
  and button label text for custom pointer-region actions. The slice keeps editor-proof resize
  text semantics in gallery/doc-layout helpers and the shared kit role vocabulary, not in
  `fret-imui`.
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
  Current performance-alignment review result: `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
  belongs in the active gap lane's evidence set. Keep runtime smoothness work in
  `diag-perf-attribution-v1`, `ui-perf-zed-smoothness-v1`, and the product-chain docking perf gate;
  do not use Dear ImGui/egui performance pressure as a reason to widen `fret-imui` or start a
  broad widget/API backlog.
