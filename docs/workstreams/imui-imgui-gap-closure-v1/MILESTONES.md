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
  semantics. Freeze panes, runtime hideable-column policy, and old columns API remain
  advanced-table candidates.
  2026-05-16 table header text result: sortable and plain table header labels also use
  `text_table_cell(...)`, preserving the same compact single-line ellipsis semantics as body cells.
  2026-05-16 static table column visibility result: `TableColumn::hidden()` and
  `TableColumn::with_visible(bool)` now cover author-declared hidden columns without copying Dear
  ImGui's mutable table runtime. Hidden columns still consume submitted row cells in declared order
  but skip header/body rendering and header responses; runtime hideable columns, persistence, and
  header-menu policy stay candidate-only.
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
