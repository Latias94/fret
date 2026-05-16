# ImUi Dear ImGui Gap Closure v1 - TODO

Status: Active
Last updated: 2026-05-16

## P0 - Source Baseline

- [x] Create the dedicated `imui-imgui-gap-closure-v1` workstream.
- [x] Refresh the current-source audit from Fret source and `repo-ref/imgui`.
- [x] Mark `docs/workstreams/standalone/imui-imgui-parity-audit-v2.md` as partially superseded for
      current gap reads.
- [x] Wire the new lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Run the P0 doc/source gates listed in `EVIDENCE_AND_GATES.md`.
      Result: `json shape`, `workstream catalog`, `imui facade teaching source`,
      `imui workstream source`, `git diff --check`, and `cargo check -p fret-examples-imui`
      all pass.

## P1 - Fearless Cleanup / Deletion Candidates

- [x] Audit public teaching imports for stale direct `fret_imui::` or `fret_ui_kit::imui::`
      default-path examples.
      First slice landed: `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs` now routes
      `TableSortDirection` through the app-facing `fret::imui::kit` facade, and
      `tools/gate_imui_facade_teaching_source.py` forbids the stale direct kit import from
      returning there.
      Second slice landed: `apps/fret-examples/src/workspace_shell_demo.rs` now routes pane-proof
      IMUI option types through `fret::imui::kit`, and both IMUI source gates forbid direct
      `fret_ui_kit::imui` imports from returning to that default pane-first proof.
      Third slice landed: `apps/fret-examples/src/imui_editor_proof_demo.rs` and its
      `collection.rs` module now route golden-proof IMUI option/state types through `fret::imui`,
      while recipe-layer imports stay explicit.
      2026-05-16 cleanup: `imui_interaction_showcase_demo` now describes its current root
      `fret::imui` teaching surface instead of the historical direct `fret_imui` control-flow
      wording, and the facade teaching source gate rejects that stale wording from returning.
- [x] Identify duplicate helper aliases that can be deleted behind a source-policy gate.
      Audit result: historical duplicate aliases are already deleted from active source,
      `imui::adapters` is contract-only, and `*_with_options(...)` helpers are canonical explicit
      option entry points rather than compatibility aliases. P1 closes with no further delete.
- [x] Check whether `fret-ui-editor::imui` remains a pure adapter over declarative editor controls.
      Audit result: current `ecosystem/fret-ui-editor/src/imui.rs` remains a thin adapter around
      declarative editor controls/composites via `into_element(...)`; no code refactor is needed in
      that crate for this P1 pass.
- [x] Check large `fret-ui-kit::imui` implementation files for owner splits that can be performed
      without public API changes.
      Audit result: `debug_draw_controls.rs` was split in the dedicated
      `imui-debug-draw-owner-split-v1` follow-on and that lane is now closed. Future additive debug
      draw capabilities still need separate follow-ons.
      2026-05-13 rebase: `imui-kit-owner-split-v1`,
      `imui-facade-disclosure-owner-split-v1`, and
      `imui-facade-text-model-owner-split-v1` are also closed. The subsequent
      `imui-facade-boolean-wrapper-owner-split-v1` and
      `imui-facade-value-model-owner-split-v1` lanes are closed as well. The later
      `imui-facade-container-wrapper-owner-split-v1` lane is closed too.
      2026-05-14 rebase: `imui-facade-floating-popup-owner-split-v1` lane is closed too. These
      lanes moved focused facade wrappers, trait default implementation bodies, and shared
      pressable response assembly into private owners without public API or runtime contract
      widening.

## P2 - User-Usable Golden Path

- [x] Pick the smallest runnable proof that should teach a complete editor panel.
      Result: `apps/fret-examples/src/imui_editor_proof_demo.rs` plus the demo-local
      `collection.rs` module is the selected proof surface; `cargo check -p fret-demo --bin
      imui_editor_proof_demo` passes.
- [x] Verify the proof includes state, command/action dispatch, editor controls, menu/popup, and
      diagnostic-friendly `test_id`s.
      Result: the proof carries named demo state, explicit action handlers, popup/menu dispatch,
      and stable `test_id` / `viewport_test_id` anchors; the collection source-guard tests now
      pass under `cargo nextest run -p fret-examples --test ...`.
- [x] Promote missing cookbook/docs references only after the proof runs and source gates pass.
      Result: `apps/fret-cookbook/README.md`, `apps/fret-cookbook/EXAMPLES.md`, and
      `docs/examples/README.md` now point from the focused IMUI cookbook lessons to
      `cargo run -p fret-demo --bin imui_editor_proof_demo` as the heavier editor-panel proof,
      without reclassifying it as a boring-ladder cookbook example.

## P3 - Dear ImGui-Class Follow-On Candidates

The catalog order below is a readiness/review order, not an implementation queue. Current execution
priority stays: product/golden workflow coherence, runner/backend multi-window hand-feel,
diagnostics/DevTools discoverability, then proof-led helper/API widening. See
`P3_EXECUTION_PRIORITY_REVIEW_2026-05-06.md`.

Readiness order for the next locally testable review slices:

1. Public facade/API catalog: keep the app-facing `fret::imui` lane explicit and freeze owner
   rules before adding more public helpers.
   Current readiness audit: `P3_PUBLIC_SURFACE_CATALOG_2026-05-06.md`. Keep `fret-imui`
   policy-light, put generic policy-heavy helpers in `kit`, editor controls in `editor`, docking
   helpers in `docking`, and continue teaching apps through `fret::imui`.
   2026-05-14 cleanup: `FloatingAreaContext` now exposes accessors instead of public identity /
   drag-kind fields, so callers can read the facade-created context without constructing invalid
   contexts.
   2026-05-14 follow-up: floating responses now follow the same accessor-first shape; tests use
   `resp.id()` rather than reaching through `resp.area.id`.
   2026-05-14 response follow-up: disclosure and combo responses now keep trigger/open/toggle
   fields crate-local, remove external default construction, and expose read-only `response()`,
   `open()`, and `toggled()` methods for callers.
   2026-05-14 text-picker follow-up: `InputTextPickerResponse` now keeps input/open/pick storage
   crate-local, removes external default construction, and exposes read-only `response()`, `open()`,
   `picked()`, and `picked_index()` accessors.
   2026-05-14 tab follow-up: tab-bar aggregate and tab-trigger responses now follow the same
   accessor-first shape; tab response storage stays crate-local and `TabBarResponse` no longer
   exposes external default construction.
   2026-05-14 virtual-list follow-up: `VirtualListResponse` now keeps its scroll handle and
   rendered-range storage crate-local while preserving read-only `handle()` and
   `rendered_range()` accessors.
   2026-05-14 table follow-up: table aggregate/header/resize responses now expose header metadata
   and resize bounds through accessors instead of public fields; response storage stays crate-local.
   2026-05-14 drag follow-up: `DragResponse` now keeps drag edge/delta storage crate-local while
   preserving read-only drag accessors and higher-level `ResponseExt` helpers.
   2026-05-14 drag/drop follow-up: source and target drag/drop responses now keep their storage and
   construction paths crate-local; public code reads helper-returned responses through accessors.
   2026-05-14 response drag-state follow-up: `ResponseExt` now keeps its aggregate drag response
   crate-local too. Public callers read drag state through `drag()`, `drag_started()`,
   `dragging()`, `drag_stopped()`, `drag_delta()`, and `drag_total()`, while internal assemblers use
   crate-local mutators.
   2026-05-14 press/context follow-up: `ResponseExt` press and context-menu derived signals now
   keep storage private. Public callers stay on `secondary_clicked()`, `double_clicked()`,
   `long_pressed()`, `press_holding()`, `context_menu_requested()`, `context_menu_anchor()`,
   `pointer_clicked()`, and `pointer_click_modifiers()`, while runtime assemblers use crate-local
   setters.
   2026-05-14 lifecycle follow-up: `ResponseExt` lifecycle edge storage is private as well. Public
   callers stay on `activated()`, `deactivated()`, `edited()`, and `deactivated_after_edit()`;
   runtime lifecycle, combo, text-picker, and disabled paths use crate-local set/merge/clear helpers.
   2026-05-14 hover/nav follow-up: raw hover, hover-delay, active-item block, and nav-highlight
   storage is private. Public callers and tests use accessor methods; runtime pressable/disclosure
   assemblers use crate-local setters, while disabled sanitization only clears nav highlight.
   2026-05-14 enabled follow-up: `ResponseExt.enabled` storage is private too. Public/demo/test
   callers use `enabled()`, while disabled sanitization and text controls use crate-local
   `set_enabled(...)`. `ResponseExt.id` is now sealed as a routing-identity surface too: public
   callers use `id()`, while response assemblers use crate-local `set_id(...)`. `ResponseExt.core`
   is now accessor-only as well: public code can still round-trip the shared
   `fret_authoring::Response` through `core()` / `from_core(...)`, while runtime assembly writes
   through crate-local core setters. The adapter signal record is now read-only too: adapter seam
   inputs keep builder-friendly public options, but emitted `AdapterSignalRecord` /
   `AdapterSignalMetadata` values expose identity, response, and metadata through accessors.
   2026-05-14 editor drag-value follow-up: `DragValueCoreResponse` now keeps drag/hover/press/focus
   storage private and no longer exposes external default construction. `DragValueCore` still owns
   response construction, while editor controls read visual state through `dragging()`, `hovered()`,
   `pressed()`, and `focused()`.
   2026-05-14 debug-draw follow-up: `DebugDrawResponse` now keeps response and summary storage
   private, removes external default construction, and exposes the underlying interaction response
   through `response()`.
   2026-05-14 debug-draw summary follow-up: `DebugDrawCommandSummary` and
   `DebugDrawListSummary` now keep diagnostic storage private as well. Public callers read command
   kind/channel/clip/count metrics through explicit accessors instead of copying fields.
   2026-05-14 source-gate follow-up: the IMUI workstream source gate now carries a reusable
   opaque-output-struct check for sealed response/context/summary records, so public output fields
   cannot return by simply changing field names.
   2026-05-14 editor axis-outcome follow-up: `VecEditAxisOutcome` and
   `TransformEditAxisOutcome` now keep section/axis/outcome storage private. Public proof code reads
   axis edit events through explicit accessors.
   2026-05-14 output-catalog gate follow-up: the IMUI workstream source gate now scans the
   `fret-imui`, `fret-ui-editor`, and `fret-ui-kit::imui` source roots for new public
   `*Response`/`*Outcome`/`*Summary`/`*Signal`/`*Record`/`*Context` structs and fails unless they
   are explicitly covered by the opaque-output-struct check.
   2026-05-14 editor color-event follow-up: `ColorEditPaletteSlotDrop`,
   `ColorEditEyedropperRequest`, and `ColorEditDragDropPayload` now follow the same accessor-first
   rule. The opaque-output catalog now covers `*Request`, `*Payload`, and `*Drop` records in the
   IMUI/editor scan roots.
   2026-05-14 state-catalog gate follow-up: the source gate now treats public `*State` structs as
   opaque IMUI structs too and registers `ImUiMultiSelectState`, so future shared state helpers
   cannot expose mutable storage by bypassing the one-off multi-select field checks.
2. Component surface catalog: keep the widget/component gap read source-backed before opening
   implementation follow-ons.
   Current readiness audit: `P3_COMPONENT_SURFACE_CATALOG_2026-05-06.md`. Current coverage is broad
   enough for the active editor proof; list-box, plot, image-item, style-editor, advanced-table, and
   child-flag mirrors remain behavior-specific candidates, not a broad widget backlog.
   2026-05-16 follow-on result: `imui-image-item-proof-v1` closed after adding standalone
   response-bearing image item / image button authoring in `fret-ui-kit::imui`, using Fret
   `ImageId` and `ImageProps` without introducing Dear ImGui texture-ID runtime state or widening
   `fret-imui`.
   2026-05-16 selectable highlight result: `imui-selectable-highlight-policy-v1` closed after
   adding a Dear ImGui-style forced-highlight policy to `SelectableOptions` and splitting the
   input-text picker active-candidate visual from selected semantics.
   2026-05-16 floating posture refresh: `window(...)` source docs now describe the current
   in-window floating surface instead of deferring z-order/focus arbitration to a future work item.
   Current `fret-imui` floating tests already cover bring-to-front hit-test order,
   focus-on-click vs activation, no-inputs / pointer-pass-through policy, close, resize, and
   collapse; OS-window tear-out and multi-viewport parity stay in docking/runner lanes.
   2026-05-16 table gap wording refresh: the component catalog now distinguishes existing
   `TableOptions::striped` alternating row backgrounds from the explicit per-row/per-cell
   background override axis. Do not open a broad `imui-table-advanced-flags-v1` lane just because
   the old wording said "row background targets."
   2026-05-16 table background/text role follow-up: explicit per-row/per-cell background overrides
   now live in `TableRowOptions::background` / `TableCellOptions::background`, and
   `ImUiTableRow::cell_text(...)` routes through the shared `text_table_cell(...)` helper instead
   of bare paragraph text. Remaining advanced-table candidates are freeze panes, column visibility,
   and old columns API shape.
   2026-05-16 control readout text role follow-up: `text_control_readout(...)` now lives beside
   `text_table_cell(...)` in `fret-ui-kit::declarative::text`, and the UI Gallery code-editor
   toolbar readouts route through that shared role instead of carrying app-local text layout policy.
   Continue converging text into a small stable role set before adding more per-surface helpers.
   2026-05-16 button label text role follow-up: `text_button_label(...)` now owns compact
   single-line button-label text in `fret-ui-kit::declarative::text`, and IMUI control chrome uses
   it for button/pill labels instead of word-wrapping control text.
   2026-05-16 code block text role follow-up: `text_code_block(...)` now owns the shared
   monospace, horizontal-scroll-friendly code text role, and the UI Gallery docs scaffold no longer
   hand-rolls code-block `TextProps`.
3. Design surface readiness: keep Dear ImGui-style density as an opt-in token/preset outcome, not a
   mutable runtime style stack.
   Current readiness audit: `P3_DESIGN_SURFACE_READINESS_2026-05-06.md`. `ImguiLikeDense` plus
   editor tokens cover the active proof; a style editor, preset selector, or visual gate should be a
   narrow follow-on with evidence.
   2026-05-14 cleanup: the unused `apply_editor_theme_patch_v1` compatibility wrapper was deleted;
   apps and tests now stay on explicit preset entry points.
4. Porting sugar readiness: `SameLine`/item-width/label-ID helpers only if two proof surfaces pay
   the same tax. Current proof surfaces already keep most of that tax local with
   `PropertyGrid`, `row_with`, `horizontal_with_options`, `child_region_with_options`, and
   explicit `id_source` / `test_id` wiring.
   Current readiness audit: `P3_PORTING_SUGAR_READINESS_2026-05-06.md`. Do not widen sugar until a
   second surface repeats the same pattern; do not copy Dear ImGui's string-label parser or
   stack/next-item width grammar into Fret by default.
   2026-05-14 cleanup: the unused `PropertyGridRow` wrapper was deleted so property-grid row
   authoring stays on `PropertyGridRowCx::row(...)` / `row_with(...)` plus raw `PropertyRow` for
   genuinely custom rows.
   2026-05-14 follow-up: eager and virtualized grid row contexts now keep `row_options`
   crate-local, so proof/app code cannot drift back to copying row policy fields by default.
   2026-05-14 inspector follow-up: `InspectorPanelCx` now hides `query_lower` and exposes query
   behavior through methods.
5. Diagnostics/devtools readiness: define a Fret equivalent of Demo/Metrics/Debug discoverability.
   Follow-on: `docs/workstreams/standalone/diag-devtools-gui-refresh-v1.md` and
   `docs/workstreams/diag-fearless-refactor-v2/README.md` keep the GUI
   productization / first-open workflow on the existing diagnostics-consumer lane.
   2026-05-16 first-open gate wording follow-up: the IMUI gap lane now distinguishes the
   cold-start `--discovery-only` entry from the fast `--discovery-only --reuse-built` drift check,
   so diagnostics discoverability failures are not conflated with large Rust build cost.
6. Collection helper readiness: keep app-owned until both proof surfaces require one helper.
   Current readiness audit: `P3_COLLECTION_HELPER_READINESS_2026-05-06.md`. The collection proof is
   editor-grade, but most behavior remains app-owned; the already-extracted shared pieces are
   `ImUiMultiSelectState`, sortable row recipes, and drag-preview recipes. `fret-node` is useful
   comparison evidence but not a second IMUI collection proof because it owns graph-specific
   node/edge/group semantics.
   2026-05-14 multi-select storage follow-up: `ImUiMultiSelectState` now keeps selection and anchor
   storage private behind explicit accessors/constructors. The collection proof remains app-owned,
   but it can no longer bypass the shared selection-storage API by mutating public fields.
   2026-05-14 ordered-selection follow-up: collection order normalization moved from the proof app
   into `ImUiMultiSelectState::from_ordered_selection(...)`, keeping the Dear ImGui-style storage
   helper in `fret-ui-kit::imui` while avoiding a monolithic `fret-imui` multi-select runtime.
   2026-05-14 request-vocabulary audit: `P3_COLLECTION_HELPER_READINESS_2026-05-06.md` now records
   that this storage extraction is not enough evidence to add `BeginMultiSelect`/`EndMultiSelect`
   or an `ImUiMultiSelectIO` runtime surface.
   2026-05-14 state-catalog gate follow-up: `ImUiMultiSelectState` is now covered by the reusable
   opaque-struct catalog because shared collection state is part of the public policy-layer
   contract, not a freely mutable data bag.
7. Child-region depth: reopen only with a concrete `BeginChild()`-style behavior target.
   Current readiness audit: `P3_CHILD_REGION_READINESS_2026-05-06.md`. Fret already covers
   keyed scrollable child areas, chrome, scroll handles, nested shell panes, and app-owned
   collection behavior. The closed `imui-child-region-resize-y-v1` and
   `imui-child-region-resize-x-v1` follow-ons now cover axis-specific manual resize with
   app-owned size state; auto-resize, visibility-return, and nav-flattening remain
   behavior-specific candidates. Do not open a generic `BeginChild()` flag-mirror lane.
8. Multi-window parity: continue in `docking-multiwindow-imgui-parity`.
9. Performance alignment: keep Dear ImGui-class smoothness pressure in the dedicated perf lanes and
   product-chain perf gates, not in a broad widget/API backlog.
   Current review: `P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`. The useful comparison axis is
   Zed-style attribution and reuse discipline plus egui-style integration/repaint clarity; do not
   treat egui's full-layout-every-frame model as an IMUI architecture target.

These slices should stay Windows/Web-verifiable first; Linux-specific validation is not a gate for
opening the slice.

## Closeout

- [x] Add a closeout audit once the first cleanup/refactor slice lands and gates pass.
      Result: `P1_CLOSEOUT_AUDIT_2026-05-06.md` closes P1 cleanup while leaving this lane active for
      P2/P3 sequencing.
