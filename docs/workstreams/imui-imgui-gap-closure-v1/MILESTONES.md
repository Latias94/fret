# ImUi Dear ImGui Gap Closure v1 - Milestones

Status: Active
Last updated: 2026-05-20

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
  Current performance-alignment review result: `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
  belongs in the active gap lane's evidence set. Keep runtime smoothness work in
  `diag-perf-attribution-v1`, `ui-perf-zed-smoothness-v1`, and the product-chain docking perf gate;
  do not use Dear ImGui/egui performance pressure as a reason to widen `fret-imui` or start a
  broad widget/API backlog.
