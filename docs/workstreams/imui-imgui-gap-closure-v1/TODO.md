# ImUi Dear ImGui Gap Closure v1 - TODO

Status: Active
Last updated: 2026-05-18

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
   of bare paragraph text. At that point the still-open advanced-table axes were freeze panes,
   runtime visibility persistence, and old columns API shape; later follow-ups below narrow or close
   those axes.
   2026-05-16 table header text follow-up: sortable/plain table header labels now reuse
   `text_table_cell(...)` too, so header text follows the same single-line ellipsis role as body
   cells instead of falling back to default word wrapping.
   2026-05-17 table sort glyph follow-up: sortable header sort indicators now reuse
   `text_chrome_glyph(...)`, keeping table header labels on table-cell text while treating arrows
   as fixed chrome glyphs.
   2026-05-16 static table column visibility follow-up: `TableColumn::hidden()` and
   `TableColumn::with_visible(bool)` now provide a narrow author-declared visibility API. Hidden
   columns keep row cell submission in declared column order but skip header/body rendering and do
   not emit header responses. Runtime hideable-column policy is now covered by the
   `ImUiTableColumnVisibilityState` and header menu follow-ups below; persistence remains a
   separate follow-on.
   2026-05-17 runtime table column visibility follow-up: `ImUiTableColumnVisibilityState` now
   provides a narrow stable-id visibility override helper in `fret-ui-kit::imui`. It applies to
   `TableColumn` lists before render and reuses the existing hidden-column path; menu/popup policy
   is covered by the subsequent helpers below, while persistence, freeze panes, and old columns API
   shape were still separate follow-ons at that historical point. A `fret-imui` composition test
   proves the helper drives the existing table render path without moving the state policy into
   `fret-imui`.
   2026-05-17 table visibility menu-item follow-up: `table_column_visibility_menu_item(...)` now
   provides the checkbox menu-item bridge between `TableColumn` and
   `ImUiTableColumnVisibilityState`. It reuses existing menu item policy; automatic header
   context-menu popup wiring is now covered by the helper below, while persistence, freeze panes,
   and old columns API shape were still separate follow-ons at that historical point.
   2026-05-17 table visibility menu-items group follow-up:
   `table_column_visibility_menu_items(...)` now composes the repeated checkbox menu section for
   stable-id, human-labeled table columns and returns accessor-first per-column responses. It still
   leaves popup presentation to the helper below and keeps persistence, freeze panes, and old
   columns API shape outside `fret-imui`.
   2026-05-17 table header context-menu request follow-up: sortable and plain table header
   responses now report context-menu requests through a shared header trigger surface. Sortable
   headers keep their primary activation/click lifecycle; plain headers expose right-click,
   ContextMenu-key, and Shift+F10 requests without reporting left-click activation. That trigger
   signal is now consumed by the visibility menu helper below; persistence, freeze panes, and old
   columns API shape remain separate follow-ons.
   2026-05-17 table header visibility menu wiring follow-up:
   `table_column_visibility_header_context_menu(...)` now composes table header context-menu
   requests from both sortable and plain headers, popup placement, and
   `table_column_visibility_menu_items(...)` into a narrow accessor-first helper in
   `fret-ui-kit::imui`. `TableColumnVisibilityHeaderContextMenuOptions` exposes popup/menu policy
   knobs. Callers still own applying the visibility model to their column list, and `fret-imui`
   stays policy-light.
   2026-05-17 table visibility snapshot follow-up:
   `TableColumnVisibilitySnapshot` / `TableColumnVisibilityEntry` now provide the persistence seam
   for `ImUiTableColumnVisibilityState`. The snapshot stores only stable column ids plus visible
   flags, round-trips through serde, ignores empty ids on restore, and uses last-entry-wins for
   duplicate ids. Apps/editors still own storage, schema placement, and when to apply the restored
   state. Later entries below close the freeze-pane seam and old API-shape cleanup.
   2026-05-17 table column pinning follow-up: `TableColumn::pinned_left()` and
   `TableColumn::pinned_right()` now provide the narrow freeze-pane seam for IMUI tables. The
   render path splits visible cells into left/center/right groups and keeps left/right frozen
   outside the shared center horizontal scroll handle. This stays in `fret-ui-kit::imui`, reuses
   `fret-ui` scroll mechanics, and does not add a table-state runtime to `fret-imui`. The old
   columns API shape stayed as the separate cleanup closed below.
   2026-05-18 table column API-shape first pass: `TableColumn` now has accessor-first read methods
   for header, stable id, width, visibility, sortability, sort direction, resize options, and pin
   state. The IMUI table render path, visibility helper, `fret-imui` composition tests, and public
   smoke tests now read through those accessors instead of teaching direct field reads. This
   prepared the separate private-field cleanup below.
   2026-05-18 table column private-field cleanup: `TableColumn` fields are now private after the
   accessor-first audit found no in-repo struct-literal construction or direct public field reads.
   Public callers keep the builder/accessor surface, `fret-ui-kit::imui` keeps crate-local
   `header_arc(...)`, `id_arc(...)`, and `set_visible_for_policy(...)` seams for render/policy
   internals, and `tools/gate_imui_workstream_source.py` prevents the public field bag from
   returning. This closes the old columns API-shape table follow-on.
   2026-05-18 table gap current read: visibility snapshot/restore, freeze-pane pinning, and the old
   `TableColumn` public field-bag shape are no longer current table gaps. Broader storage/schema
   ownership and any full Dear ImGui-style mutable table runtime remain outside `fret-imui` until a
   separate proof needs them.
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
   2026-05-16 paragraph text role follow-up: `text_paragraph(...)` and
   `text_paragraph_break_words(...)` now provide semantic aliases over the existing prose helpers,
   leaving `text_prose(...)` available for shadcn/Tailwind naming while giving Fret apps a stable
   paragraph role name.
   2026-05-17 compact paragraph text follow-up: `text_compact_paragraph(...)` now owns dense
   wrapping body copy for editor/IMUI panels, and both `bullet_text(...)` labels and
   `UiWriterImUiFacadeExt::text_wrapped(...)` route through it instead of local `TextProps`.
   2026-05-18 tooltip body text follow-up: `tooltip_text(...)` / `tooltip_text_with_options(...)`
   now route their default body copy through a crate-local `tooltip_body_text(...)` helper backed
   by `text_compact_paragraph(...)`, while rich-content `tooltip(...)` closures still let callers
   choose their own text role.
   2026-05-18 collection proof text-role follow-up: `imui_editor_proof_demo` collection titles,
   status/readout lines, asset metadata/path text, context-menu selection summaries, and drop
   status now route through proof-local helpers backed by shared section-chrome and control-readout
   roles. The inline rename explanatory copy explicitly uses `text_wrapped(...)`, keeping app-owned
   collection behavior intact while avoiding fixed-row bare text under resize.
   2026-05-16 trigger label reuse follow-up: IMUI tab triggers and menubar triggers now reuse
   `text_button_label(...)` because they are button-like trigger labels.
   2026-05-16 list row text role follow-up: `text_list_row_label(...)` now owns dense
   selectable/menu/tree row label text. IMUI menu items, selectables, and disclosure/tree rows use
   the shared role, giving them fill-width, min-width-zero, single-line ellipsis semantics without
   recasting them as button labels.
   2026-05-16 menu shortcut readout reuse follow-up: menu item shortcut labels now reuse
   `text_control_readout(...)` as muted compact auxiliary readouts instead of carrying local
   nowrap/clip `TextProps` policy.
   2026-05-17 menu indicator glyph follow-up: menu checkbox/radio indicators and submenu chevrons
   now reuse `text_chrome_glyph(...)`, so glyph-only menu chrome no longer falls back to bare
   `cx.text(...)` default wrapping semantics.
   2026-05-17 section chrome label text follow-up: `text_section_chrome_label(...)` now owns
   compact separator/section chrome labels in `fret-ui-kit::declarative::text`, and IMUI
   `separator_text` labels route through it instead of local default-wrapping `TextProps`.
   2026-05-17 chrome title text follow-up: `text_chrome_title(...)` now owns fill-width floating
   window title-bar text, and floating window titles route through shared chrome text roles instead
   of local `TextProps`.
   2026-05-17 chrome glyph text follow-up: `text_chrome_glyph(...)` now owns compact fixed-slot
   chrome glyphs in `fret-ui-kit::declarative::text`, and disclosure/tree indicators route through
   it instead of keeping a local `TextProps` constructor.
   2026-05-17 floating close glyph follow-up: floating-window close button glyphs now reuse
   `text_chrome_glyph(...)` too, so fixed title-bar chrome no longer falls back to bare
   `cx.text(...)` default wrapping semantics.
   2026-05-18 text role source-gate hardening follow-up: `tools/gate_imui_workstream_source.py`
   rejects direct `TextProps` construction under `fret-ui-kit::imui`, including both
   `TextProps::new(...)` and struct-literal forms, so new compact IMUI text policy must route
   through the shared role vocabulary instead of gaining local text layout exceptions.
   2026-05-16 IMUI text item resize follow-up: `UiWriterImUiFacadeExt::text(...)` now follows Dear
   ImGui's default `Text()` posture by staying single-line, shrinkable, and ellipsis-truncated
   under narrow resize. `text_wrapped(...)` is the explicit opt-in path for explanatory copy that
   should wrap, and first-party proof prose now uses that API.
   2026-05-17 IMUI text item role cleanup: `UiWriterImUiFacadeExt::text(...)` now delegates to the
   shared `text_section_chrome_label(...)` role instead of hand-rolling `TextProps`, and the source
   gate no longer keeps an allowlist exception for IMUI default text.
   2026-05-16 control chrome fill-text follow-up: checkbox/radio/switch labels plus combo/slider
   captions now inherit the same compact single-line shrink/ellipsis posture through
   `control_chrome::fill_text(...)`, so fixed-height control chrome cannot grow rows by word
   wrapping under resize.
   2026-05-17 control label text follow-up: `text_control_label(...)` now owns compact control
   labels in `fret-ui-kit::declarative::text`, and `control_chrome::fill_text(...)` routes through
   it instead of owning local `TextProps` policy.
   2026-05-17 editor input value text follow-up: `fret-ui-editor` now shares
   `editor_input_value_text(...)` for drag-value and axis-drag-value scrub readouts. The helper
   keeps numeric value text fill-width, `min-width: 0`, shrinkable, single-line, and ellipsis
   clipped without moving editor-specific density/chrome policy into `fret-imui`.
   2026-05-17 editor status badge text follow-up: `FieldStatusBadge` now routes its compact label
   through `editor_status_badge_text_props(...)`, keeping badge text in the editor readout
   primitive layer instead of hand-rolling `TextProps` inside the control.
   2026-05-17 editor inline error text follow-up: `ColorEdit` and its numeric popup now share
   `editor_inline_error_text_props(...)` for compact single-line destructive readouts, leaving
   wrapping validation prose to controls that explicitly need multi-line errors.
   2026-05-17 editor validation message text follow-up: `NumericInput` inline validation messages
   now route through `editor_validation_message_text_props(...)` instead of owning local
   `TextProps`. The role is the explicit wrapping error/prose path for editor controls, while the
   IMUI workstream source gate now allowlists direct editor `TextProps` construction only inside
   editor text primitive owners.
   2026-05-17 transform label text follow-up: `TransformEdit` now routes section badges, section
   headings, and inline link/uniform checkbox labels through shared editor readout helpers instead
   of owning local `TextProps` policy. The helpers keep these compact control labels single-line
   and shrinkable under resize without moving editor-specific density policy into `fret-imui`.
   2026-05-17 popup list text follow-up: enum-select and text-assist popup rows plus empty-state
   labels now share `editor_popup_list_row_text_props(...)` /
   `editor_popup_empty_text_props(...)`, replacing `TextProps::new(...)` default word wrapping in
   popup/editor assist surfaces. Color-edit copy menu rows and popup option captions now reuse the
   same popup-list family through start-aligned, centered-row, and fixed-caption variants.
   2026-05-17 color preview/tooltip text follow-up: color side-preview captions and tooltip
   numeric lines now use dedicated editor readout helpers instead of local `TextProps`, keeping
   them separate from popup-list rows while preserving single-line resize behavior.
   2026-05-17 property chrome text follow-up: property-group header labels, property-row reset
   glyphs, and inspector panel titles now share editor readout helpers instead of local/default
   text policy. Fixed inspector chrome text stays single-line, min-width-zero where it shrinks, and
   line-height constrained under resize without moving editor-specific policy into `fret-imui`.
   The inspector title path also has a narrow-header layout gate with toolbar siblings, so panel
   titles cannot drift back to default word wrapping.
   2026-05-17 property-row label text follow-up: `editor_property_row_label_text_props(...)` now
   owns fixed inspector label text, and eager/virtualized property-grid row contexts expose
   `label_text(...)` as the preferred first-party label path. `PropertyRow` label slots also clamp
   their own line boxes to the editor row height, so accidental bare/default label text cannot
   reintroduce row growth under resize.
   2026-05-17 proof teaching follow-up: `imui_editor_proof_demo` property-grid labels now use
   `row_cx.label_text(...)`, and the IMUI workstream source gate forbids representative proof
   labels from drifting back to bare `|cx| cx.text(...)` label slots.
   2026-05-17 workspace shell proof text follow-up: `workspace_shell_demo` editor-rail buttons,
   property labels, and compact property values now teach the shared button-label, property-label,
   and control-readout text roles instead of bare `cx.text(...)`.
   2026-05-17 editor notes proof text follow-up: `editor_notes_demo` inspector metadata labels,
   subtitle, and compact status values now teach the same property-label and control-readout roles
   instead of relying on bare `cx.text(...)` inside fixed property rows.
   2026-05-17 workspace shell prompt text follow-up: the dirty-close prompt title/details now
   reuse the shared section-chrome and control-readout text roles instead of bare dialog text.
   2026-05-17 editor proof readout follow-up: `imui_editor_proof_demo` now routes selected
   empty-state labels, authoring shared-state readouts, and the declarative gradient section label
   through proof-local helpers backed by shared text roles instead of bare `cx.text(...)`.
   2026-05-17 drag-preview text follow-up: proof drag-preview cards now render title and subtitle
   as separate shared text-role elements instead of a newline-joined bare text blob.
   2026-05-17 gradient editor empty-state follow-up: `GradientEditor` now routes its `No stops`
   empty-state label through an editor readout primitive instead of bare/default text, keeping
   compact inspector empty states single-line and shrinkable under resize.
   2026-05-17 text role matrix follow-up: `P3_TEXT_ROLE_MATRIX_2026-05-17.md` now freezes the
   stable base role vocabulary for resize triage: control readout, button label, paragraph, code
   text, and table cell text. The matrix also classifies current derived roles, keeps wrapping text
   out of fixed-height control rows unless parents measure multi-line height, and explicitly
   rejects adding a public `TextRole` enum until two consumers need a data-driven role value.
   2026-05-17 shared text-role layout gate follow-up: the base shared text roles now have a real
   narrow-layout regression proving single-line roles stay one measured line while paragraph text
   measures multi-line height through `UiTree::layout_all(...)`.
   2026-05-17 property-row value overflow follow-up: `PropertyRow` value slots no longer force
   `Overflow::Clip`, so explicit wrapping validation/prose children such as `NumericInput` inline
   validation messages can grow with their measured multi-line height instead of painting past a
   clipped inspector row. Fixed label/reset/action chrome slots still clip themselves; this is a
   layout-container contract fix in `fret-ui-editor`, not a `fret-imui` runtime/API widening.
   2026-05-17 property-row wrapping layout follow-up: the value-slot fix now has a real layout
   gate, not only a structural overflow check. A narrow row with the editor validation-message role
   runs through `UiTree::layout_all(...)`, and public element-bounds queries prove the multi-line
   validation text is contained by the value slot and row bounds.
   2026-05-17 property-grid wrapping layout follow-up: `PropertyGrid` now has a composition-level gate
   with mixed single-line and wrapping rows. The test proves a multi-line validation
   value grows its row and pushes the following row down, so the fix is locked at the realistic
   inspector-grid composition layer instead of only the single-row container layer.
   2026-05-17 generic list text follow-up: `list_from_strings(...)`, the compatibility string-list
   helper over retained virtual lists, now routes leading glyphs, row labels, and trailing shortcuts
   through `text_chrome_glyph(...)`, `text_list_row_label(...)`, and
   `text_control_readout(...)` instead of bare `cx.text(...)`. The helper remains a generic
   declarative kit convenience, not an IMUI runtime/list-box widening.
   2026-05-17 generic tree text follow-up: the default retained tree row renderer now routes row
   labels through `text_list_row_label(...)` and toggle glyphs through `text_chrome_glyph(...)`
   instead of keeping a local truncate/text path. Custom tree row renderers still own their own
   content policy.
   2026-05-17 file tree text follow-up: `file_tree_view_retained_v0(...)`, the retained workspace
   file-tree surface, now routes fixed row icons through `text_chrome_glyph(...)` and file labels
   through `text_list_row_label(...)` instead of hand-rolling text truncation inside fixed rows.
   2026-05-17 retained table text follow-up: retained table headers plus grouped row label and
   aggregation text now route through `text_table_cell(...)`, keeping fixed table cells on the
   shared single-line ellipsis role outside the IMUI-specific table wrapper.
   2026-05-17 examples table proof text follow-up: `table_demo` and `table_stress_demo` now route
   fixed header/status readouts through `text_control_readout(...)` and fixed table header/body
   cells through `text_table_cell(...)`, so the runnable table proof surfaces no longer teach bare
   default wrapping text inside fixed table rows.
   2026-05-17 datatable proof text follow-up: `datatable_demo` now routes its compact selected/sort
   status line through `text_control_readout(...)` and its DataTable body cells through
   `text_table_cell(...)`, keeping the shadcn data-table proof aligned with the same fixed-row
   text semantics as the lower-level table demos.
   2026-05-17 virtual-list stress proof text follow-up: `virtual_list_stress_demo` now routes its
   compact scroll/state header through `text_control_readout(...)` and visible fixed-row labels
   through `text_list_row_label(...)`, so the runnable virtual-list stress surface no longer
   teaches bare default wrapping text inside fixed-height rows.
   2026-05-17 canvas datagrid stress proof text follow-up: `canvas_datagrid_stress_demo` now routes
   its compact canvas grid stats header through `text_control_readout(...)`, so the retained-canvas
   grid stress proof no longer teaches bare default wrapping text above the fixed grid slot.
   2026-05-17 date picker proof text follow-up: `date_picker_demo` now routes its compact
   open/selected/month status through `text_control_readout(...)`, switch captions through
   `text_control_label(...)`, and keyboard instructions through `text_paragraph(...)`, keeping the
   proof aligned with the fixed-chrome-vs-prose text role split.
   2026-05-17 form proof text follow-up: `form_demo` now routes its compact header
   submit/valid/dirty/status readout through `text_control_readout(...)`, keeping the runnable
   shadcn form proof from teaching bare default wrapping text inside fixed header chrome.
   2026-05-17 sonner proof text follow-up: `sonner_demo` now routes its fixed demo title through
   `text_section_chrome_label(...)` and promise/last-action status through
   `text_control_readout(...)`, keeping toast proof chrome single-line and shrinkable under
   resize.
   2026-05-17 echarts proof text follow-up: `echarts_demo` now routes its chart titles through
   `text_section_chrome_label(...)`, keeping chart scaffold labels on the shared single-line
   chrome role instead of bare default text.
   2026-05-17 components gallery table proof text follow-up: `components_gallery` now routes the
   retained table torture cell renderer through `text_table_cell(...)` and its explanatory table
   header through `text_paragraph(...)`. The same proof now routes its top chrome title, tree
   status, theme controls, color swatch labels, and control state readouts through the matching
   shared text roles. Overlay body copy now uses paragraph text, while the overlay last-action
   status uses `text_control_readout(...)`, keeping the runnable gallery proof from teaching bare
   default wrapping text in fixed cells, fixed control chrome, or overlay proof copy.
   2026-05-17 markdown proof chrome text follow-up: `markdown_demo` now routes its fixed demo
   title through `text_section_chrome_label(...)`, its explanatory preview copy through
   `text_paragraph(...)`, and its toolbar/readout state through `text_control_readout(...)`.
   Markdown body rendering, image placeholders, and text capability probes stay owned by the
   Markdown surface instead of being mechanically rewritten.
   2026-05-17 residual bare text capability follow-up: the remaining first-party
   `apps/fret-examples/src` bare `cx.text(...)` / `TextProps::new(...)` paths are now source-gated
   to explicit text/IME/rendering capability proofs only: `components_gallery` text smoke/font
   probes, `ime_smoke_demo` IME behavior instructions/status, and the explicit rendering-capability
   demos that still use direct `TextProps { ... }` constructors. New bare text in runnable proof
   apps must now either use a shared role or update the residual proof gate with a documented
   reason.
   2026-05-17 gallery retained-table torture text follow-up: the UI Gallery retained-table torture
   page now routes fixed cell text through `text_table_cell(...)` and table state/status readouts
   through `control_readout_text(...)`, leaving only explicitly multi-line/paragraph copy as bare
   prose.
   2026-05-17 gallery data-table torture text follow-up: the UI Gallery DataTable torture page now
   routes both retained and non-retained fixed cell renderers through `text_table_cell(...)`, and
   table sorting/filter/pinning status lines through `control_readout_text(...)`.
   2026-05-18 data-table snippet table-cell text follow-up: the app-facing copyable DataTable
   snippets now share a directory-local `table_cell_text(...)` helper backed by
   `text_table_cell(...)`. Default/basic/guide/RTL/reusable-component fixed cell text and fallback
   cells no longer teach bare `cx.text(...)`; amount columns keep their existing tabular numeric
   formatting until a dedicated numeric-cell role is designed.
   2026-05-18 AI AudioPlayer state-marker follow-up: copyable AudioPlayer snippets now expose
   state-only diagnostics anchors through zero-size `SpacerProps` children under generic semantics
   instead of empty `Text` nodes. This keeps test markers out of visible text layout semantics while
   preserving stable diagnostics `test_id`s.
   2026-05-18 AI visible text-role follow-up: Message and Terminal copyable snippets now route
   fixed titles through `text_section_chrome_label(...)`, explanatory prose through
   `text_paragraph(...)`, and compact Message action status through `text_control_readout(...)`.
   The Terminal empty-output marker also uses the same non-text spacer-marker pattern.
   2026-05-18 AI visible text-role follow-up 2: Artifact, CodeBlock, and Sandbox copyable snippets
   now use the same role split for fixed demo titles, explanatory prose, and compact status text.
   CodeBlock's active-language diagnostics marker also moved from an invisible empty text node to a
   zero-size generic spacer marker.
   2026-05-18 AI Queue text-role follow-up: the Queue copyable snippet now routes its fixed demo
   title through `text_section_chrome_label(...)`, explanatory copy through `text_paragraph(...)`,
   and action-revision diagnostics through a generic zero-size spacer marker instead of empty or
   invisible text.
   2026-05-18 AI Checkpoint text-role follow-up: the Checkpoint copyable snippet now uses
   paragraph text for conversation/prose, control-readout text for restore status, button-label
   text for the checkpoint trigger, and chrome-glyph text for custom checkpoint icon glyphs.
   2026-05-18 AI simple chrome text-role follow-up: Agent, CodeBlock usage, Environment Variables,
   and OpenIn copyable snippets now route fixed demo titles through `text_section_chrome_label(...)`
   and explanatory body copy through `text_paragraph(...)`.
   2026-05-18 AI selector/branch marker follow-up: MessageBranch, MicSelector, and ModelSelector
   snippets now expose state-only diagnostics anchors through generic zero-size spacer markers
   instead of empty `Text`, while their fixed demo titles/body copy use shared text roles.
   2026-05-18 AI prompt/plan/commit-large text-role follow-up: CommitLarge now routes its
   opened-file diagnostics marker through a generic zero-size spacer anchor instead of empty
   `Text`, and its fixed title/body copy uses shared section-chrome/paragraph roles. Plan,
   PromptInputActionMenu, and PromptInputTooltip now put their outer demo titles/body copy on the
   same shared roles while leaving inner plan prose and shadcn/Button composition for a separate
   semantics pass.
   2026-05-18 AI large/status snippet text-role follow-up: StackTraceLarge and TestResultsLarge
   now use generic zero-size spacer markers for opened/activated diagnostics anchors, while
   StackTraceLarge, TestResultsLarge, Tool, and Suggestions route fixed outer title/body text
   through shared section-chrome/paragraph roles. Tool's fixed state-section labels also use
   section-chrome text; Suggestions custom-children content stays app-owned for a later custom
   content semantics pass.
   2026-05-18 AI queue-prompt/transcription text-role follow-up: QueuePromptInput now routes its
   sent-count diagnostics marker through a generic zero-size spacer marker, its custom Search
   button child through `text_button_label(...)`, and fixed outer title/body copy through shared
   section-chrome/paragraph roles. Transcription now uses generic zero-size spacer markers for
   time/active diagnostics anchors while keeping fixed title/body copy on shared roles.
   2026-05-18 AI WebPreview text-role follow-up: WebPreview state diagnostics now use generic
   zero-size spacer markers instead of empty `Text`, navigation glyphs use `text_chrome_glyph(...)`,
   and composable child fixed body/footer copy uses shared section-chrome/paragraph roles.
   2026-05-18 AI Chat text-role follow-up: Chat's prompt-nonempty diagnostics marker now uses a
   generic zero-size spacer, empty marker fallbacks use spacers instead of empty `Text`, fixed
   header instructions use paragraph roles, and exported markdown length uses control-readout text.
   Chat message body rendering remains app/content-owned for a separate semantics pass.
   2026-05-18 AI PromptInput provider/docs text-role follow-up: PromptInputProvider now routes its
   sent-count diagnostics marker through a generic zero-size spacer, its custom external-add label
   through `text_button_label(...)`, and fixed outer title/body copy through shared roles.
   PromptInput docs now routes the custom Search label through button-label text and fixed outer
   title/body copy through section-chrome/paragraph roles.
   2026-05-18 AI chrome/readout text-role follow-up: Reasoning, StackTrace, and VoiceSelector now
   route fixed outer title/body copy through shared section-chrome/paragraph roles; StackTrace and
   VoiceSelector compact status/diagnostics readouts use `text_control_readout(...)` instead of
   default wrapping text.
   2026-05-18 AI Confirmation content text-role follow-up: Confirmation request/body snippets now
   route prose through `text_paragraph(...)`, inline/code payloads through `text_code_wrap(...)`,
   approval/rejection result text through `text_control_readout(...)`, and the demo's fixed outer
   title/body copy through shared section-chrome/paragraph roles.
   2026-05-18 AI Task content text-role follow-up: Task item labels now route through
   `text_list_row_label(...)`, attached file names through `text_code_wrap(...)`, and the demo's
   fixed outer title/body copy through shared section-chrome/paragraph roles.
   2026-05-18 AI Conversation instrumentation text-role follow-up: Conversation length/export
   diagnostics now use generic semantics instead of text semantics, and the custom scroll-bottom
   label routes through `text_button_label(...)`.
   2026-05-18 AI usage snippet text-role follow-up: Attachments usage explanatory copy now uses
   paragraph text, and StackTrace usage title/body copy uses section-chrome/paragraph roles.
   2026-05-18 AI Message usage text-role follow-up: Message usage now routes user message text
   through paragraph text, the last-action marker through control-readout text, and fixed outer
   title/body copy through section-chrome/paragraph roles.
   2026-05-18 AI Canvas world spike text-role follow-up: the canvas spike now routes visible
   chrome, node copy, and debug/status readouts through shared section-chrome, paragraph, and
   control-readout roles instead of bare `cx.text(...)`.
   2026-05-18 AI Image demo text-role follow-up: the image demo now routes fixed explanatory copy
   through paragraph text and status/loading readouts through shared control-readout text instead of
   bare `cx.text(...)`.
   2026-05-18 AI PromptInput referenced sources text-role follow-up: fixed referenced-sources
   title/body copy now routes through section-chrome and paragraph roles instead of bare
   `cx.text(...)`.
   2026-05-18 AI Artifact code display status-marker follow-up: the docs status marker now keeps
   the diagnostic `label_contains` contract on a generic semantics marker with a zero-size spacer
   instead of an invisible bare text element.
   2026-05-18 AI ChainOfThought composable text-role follow-up: composed header, step-label, and
   description child text now uses shared section-chrome and paragraph roles instead of bare
   `cx.text(...)`.
   2026-05-18 AI TestResults composable text-role follow-up: custom summary/progress/status/name
   and duration child text now uses shared readout/list-row roles instead of bare `cx.text(...)`.
   2026-05-18 AI Workflow snippet text-role follow-up: workflow fixed chrome, panel copy,
   node-content sample copy, footer labels, and click readouts now use shared text roles instead of
   bare `cx.text(...)`.
   2026-05-18 AI Suggestions/reasoning/transcript text-role follow-up: suggestions custom
   children, reasoning hook status, transcript torture header copy, and chat exported-status marker
   now use shared text roles or generic marker semantics instead of bare/default text surfaces.
   2026-05-18 AI custom-children text-role follow-up: environment variables, package info,
   inline citations, persona, and sources custom-child snippets now route visible app-owned text
   through shared roles, including the new single-line `text_code_label(...)` identifier role.
   2026-05-17 gallery data-grid text follow-up: the UI Gallery DataGrid preview now routes
   virtualized grid cell text through `text_table_cell(...)` and the selected-row readout through
   `control_readout_text(...)`.
   2026-05-17 gallery data paragraph text follow-up: DataGrid/DataTable/Tree Torture explanatory
   header copy now routes through `paragraph_text(...)`, backed by shared `text_paragraph(...)`,
   instead of default `cx.text(...)`.
   2026-05-17 gallery inspector torture text follow-up: the UI Gallery Inspector Torture page now
   routes fixed virtual-row property labels through `text_list_row_label(...)` and fixed row values
   through `control_readout_text(...)`.
   2026-05-17 gallery virtual-list torture text follow-up: UI Gallery virtual-list harness row
   labels now use `text_list_row_label(...)`, row detail/editing readouts use
   `control_readout_text(...)`, and the UI Kit list torture custom row renderer also routes item
   labels through `text_list_row_label(...)`.
   2026-05-17 gallery harness header text follow-up: retained-table, hit-test, UI Kit list,
   virtual-list, and view-cache harness headers now route explanatory copy through
   `paragraph_text(...)` and mode/status lines through `control_readout_text(...)` instead of bare
   `cx.text(...)`.
   2026-05-17 gallery view-cache list text follow-up: the UI Gallery View Cache torture page now
   routes its cached inner virtual-list row labels through `text_list_row_label(...)`.
   2026-05-17 gallery view-cache control-label follow-up: fixed switch labels now route through
   `control_label_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery view-cache popover body follow-up: the cached Popover body copy now routes
   through `paragraph_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery tree torture status text follow-up: the UI Gallery Tree Torture dynamic
   target status now routes through `control_readout_text(...)` instead of carrying local
   muted/text-sm styling.
   2026-05-17 gallery overlay status text follow-up: overlay and menu last-action/status flags now
   route through `control_readout_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery overlay scroll-row text follow-up: dialog/sheet/portal scroll filler rows
   now route through `text_list_row_label(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery overlay body prose follow-up: HoverCard and Popover body copy now route
   through `paragraph_text(...)` instead of bare `cx.text(...)`.
   2026-05-17 gallery chrome torture control-label follow-up: the fixed text-input/textarea
   labels now route through `control_label_text(...)`, backed by shared `text_control_label(...)`,
   instead of bare `cx.text(...)`.
   2026-05-17 virtual row fallback follow-up: tree and file-tree virtualizer out-of-range fallback
   paths now use spacer placeholders instead of empty text nodes, removing another fixed-row
   `Text` escape hatch.
   2026-05-17 gallery disabled toaster placeholder follow-up: the UI Gallery disabled toaster
   driver path now uses a spacer placeholder instead of `cx.text("")`, so app-shell disabled
   placeholder plumbing no longer creates meaningless text nodes.
   2026-05-17 gallery app-sidebar collapsed placeholder follow-up: the copyable app-sidebar snippet
   now uses a spacer placeholder for collapsed project groups instead of teaching `cx.text("")` as
   a layout placeholder.
   2026-05-17 fret-ui-ai empty placeholder follow-up: hidden/missing-content AI element fallbacks
   now use a crate-local spacer helper instead of returning empty text nodes, without widening
   `fret-imui`.
   2026-05-17 gallery status-bar readout follow-up: UI Gallery status-bar metrics,
   inspector-state, and last-action text now route through
   `driver::text_roles::chrome_readout_text(...)`, backed by `text_control_readout(...)` instead
   of bare `cx.text(...)`, keeping fixed status chrome single-line and shrinkable under resize.
   2026-05-17 gallery driver chrome text follow-up: UI Gallery driver chrome now has a small
   `driver::text_roles` owner. Disabled tabs/sidebar/content placeholders use the shared
   control-readout role, and settings-sheet section labels use the shared section-chrome role
   instead of bare `cx.text(...)`.
   2026-05-17 gallery driver chrome label follow-up: the nav title now uses section-chrome text,
   and settings-sheet switch captions use control-label text through `driver::text_roles` instead
   of local `TextProps` policy.
   2026-05-17 gallery minimal-root text follow-up: the `BISECT_MINIMAL_ROOT` diagnostic root now
   routes its placeholder readout through `driver::text_roles::chrome_readout_text(...)` instead
   of bare `cx.text(...)`, so resize bisect surfaces do not teach default wrapping text.
   2026-05-17 gallery debug-HUD text follow-up: fixed-size debug HUD lines now route through
   `driver::text_roles::chrome_readout_text(...)` instead of local word-wrapping `TextProps`, so
   long metric/readout lines truncate rather than growing HUD row height under resize.
   2026-05-17 gallery shell content/nav text follow-up: the page header title/source and sidebar
   group headings now route through shared chrome/readout roles instead of local `TextProps`,
   keeping app-shell text policy centralized while staying outside `fret-imui`.
   2026-05-18 gallery sidebar snippet chrome text follow-up: the copyable Sidebar examples now
   route card body prose and missing-content fallbacks through paragraph roles, and debug/status
   lines through `text_control_readout(...)` instead of bare `cx.text(...)`.
   2026-05-18 gallery command snippet chrome text follow-up: copyable Command examples now route
   last-action/count/active-value status through `text_control_readout(...)`, short subsection
   headings through section-chrome text, and desktop-only/prose copy through paragraph text. The
   retained active-descendant command snippet remains an explicit text-input capability surface.
   2026-05-18 gallery Accordion trigger text follow-up: copyable Accordion examples now route
   trigger labels through `text_button_label(...)` instead of bare `cx.text(...)`, keeping those
   button-like trigger rows single-line/shrinkable under resize while leaving shadcn component
   internals and upstream parity tests untouched.
   2026-05-18 gallery ToggleGroup item text follow-up: copyable ToggleGroup examples now route
   ordinary item captions through `text_button_label(...)` instead of bare `cx.text(...)` /
   `ui::text(...)`, keeping toggle buttons single-line/shrinkable under resize. The custom
   weight-card snippet remains an explicitly styled visual sample rather than a default text-role
   migration target.
   2026-05-18 gallery Toggle item text follow-up: copyable Toggle examples now route ordinary
   toggle captions through `text_button_label(...)`, and the label-association pressed-state
   marker through `text_control_readout(...)`, so these fixed button-like snippets no longer teach
   wrapping text inside toggle chrome.
   2026-05-17 gallery editor preview text follow-up: code-editor, Markdown, and Web IME preview
   headers now route explanatory copy through `paragraph_text(...)`, fixed status/debug values
   through `control_readout_text(...)`, and custom pointer-region labels through
   `button_label_text(...)`. This keeps resize-sensitive editor proof chrome on the shared
   role vocabulary without adding a public `TextRole` enum or moving policy into `fret-imui`.
   2026-05-18 code-view editor preview prose follow-up: the UI Gallery code-view torture header
   now routes its explanatory copy through `paragraph_text(...)` instead of bare `cx.text(...)`,
   keeping the scrollable code/text preview on the shared paragraph role vocabulary.
   2026-05-18 text editor/conformance header prose follow-up: the UI Gallery text
   editor/conformance headers for feature toggles, measure overlay, mixed-script fallback, outline
   stroke, selection perf, and BiDi/RTL now route explanatory copy through `paragraph_text(...)`;
   the BiDi sample-list heading uses `control_readout_text(...)`. Text capability probes remain on
   their explicit `TextProps` / `SelectableTextProps` / canvas text paths.
   2026-05-18 IMUI virtual-list fixed-row clip follow-up: fixed/known-height IMUI virtual-list
   rows now set `Overflow::Clip` on the row container, while measured rows keep visible overflow for
   runtime measurement. This closes the container side of the text-resize failure mode where a
   caller accidentally submits oversized/wrapping content into a fixed row; the fix stays in
   `fret-ui-kit::imui` and does not add a mutable list runtime to `fret-imui`.
   2026-05-18 retained tree fixed-row clip follow-up: `tree_view_retained(...)` now gives
   fixed/known-height retained tree rows an explicit fill-width, fixed-height, `Overflow::Clip`
   pressable row layout, while `Measured` retained tree rows stay auto-height/visible for runtime
   measurement. This closes the same resize-overflow class for generic editor trees without moving
   tree policy into `fret-imui`.
   2026-05-18 retained file-tree fixed-row clip follow-up:
   `file_tree_view_retained_v0(...)` now gives the retained Pressable row itself an explicit
   fill-width, fixed-height, `Overflow::Clip` layout, matching its known-height virtualizer
   contract instead of relying only on an inner content container to clip.
   2026-05-18 retained table fixed-row clip follow-up:
   `table_virtualized_retained_v0(...)` and grouped/eager table row owners now share
   `table_body_row_layout(...)`: fixed rows become fill-width fixed-height clip boundaries, while
   measured rows stay auto-height/visible for runtime measurement. This keeps table cell text and
   table row containers aligned with the same resize contract without moving table state/runtime
   policy into `fret-imui`.
   2026-05-17 code-editor IME gate button-label follow-up: code-editor MVP IME gate actions now
   route their custom pointer-region labels through `button_label_text(...)`, and both the gallery
   source test and IMUI workstream source gate forbid those fixed action labels from returning to
   bare `cx.text(...)`.
   2026-05-17 docking arbitration text-role follow-up: `docking_arbitration_demo` now routes
   Popover body copy through a local paragraph helper and state/debug status lines through a local
   readout helper backed by `text_control_readout(...)`. A source test and IMUI workstream source
   gate prevent the docking proof from returning to bare `cx.text(...)`.
   2026-05-17 docking/container-query panel text follow-up: `docking_demo` and
   `container_queries_docking_demo` now route fixed panel labels, readouts, and placeholders through
   local helpers backed by shared list-row, control-readout, and button-label text roles. Focused
   source tests plus the IMUI workstream source gate keep those resize-sensitive panel demos from
   returning to bare `cx.text(...)`.
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
   app-owned size state; the height-unconstrained child-region path now has a focused AutoResizeY
   composition gate, while visibility-return, nav-flattening, and more specific auto-resize
   behavior remain behavior-specific candidates. Do not open a generic `BeginChild()` flag-mirror
   lane.
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
