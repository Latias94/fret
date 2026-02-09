# Code Editor Ecosystem v1 - TODO Tracker

Status: Active (workstream tracker)
Last updated: 2026-02-09

This is the checkbox tracker companion to:

- `docs/workstreams/code-editor-ecosystem-v1.md`

Normative contracts:

- `docs/adr/0200-code-editor-ecosystem-v1.md`
- `docs/adr/0194-text-navigation-and-word-boundaries-v1.md`
- `docs/adr/0195-web-ime-and-text-input-bridge-v1.md`

Legend:

- [ ] pending
- [~] in progress
- [x] done
- [!] blocked / needs decision

---

## M0 — Contracts Locked

- [x] Review ADR 0200 and confirm crate split and v1 baseline (windowed surface first).
  - See: ADR 0200 “M0 Review Checklist (Non-Normative)”.
- [x] Review ADR 0194 and confirm the preferred seam:
  - window-scoped `InputContext.text_boundary_mode` + override stack.
  - See: ADR 0194 “M0 Review Checklist (Non-Normative)”.
- [x] Review ADR 0195 and confirm web strategy:
  - hidden textarea bridge,
  - `beforeinput` + `composition*` translation,
  - proxy mode (no full document mirroring).
- [x] Add 1–3 evidence anchors per ADR (file paths / tests) in ADR 0200/0194/0195.

---

## M1 — Web IME Bridge (wasm baseline)

### DOM element lifecycle

- [x] Create the hidden textarea element (tracked per `AppWindowId` and mounted into a per-canvas wrapper/overlay layer).
- [x] Define focus/blur rules and map them to `Effect::ImeAllow`.
- [x] Web runner: flush `Effect::ImeAllow` on pointer-down (user activation) to allow synchronous textarea focus.
- [x] Define best-effort caret anchoring and map it to `Effect::ImeSetCursorArea`.
- [x] Load bundled default fonts during web renderer adoption (avoid “first frame” tofu; keep `TextAddFonts` for user-provided fonts).

### Event translation

- [x] Translate `compositionstart/update/end` to `Event::Ime` (preedit/commit).
- [x] Translate `beforeinput`/`input` to `Event::TextInput` for committed insertions.
- [x] Filter control characters from `TextInput` (ADR 0012).
- [x] Implement command-path suppression to avoid “command executes + DOM inserts text” (shortcut suppression + ordering suppression landed; keep auditing edge cases via the web harness).

### UTF-16 ↔ UTF-8 conversion

- [x] Implement deterministic conversion + clamping utilities.
- [x] Add tests for mixed-script and emoji sequences (byte offsets remain valid).

### Observability (debug-only)

- [x] Counters: last `inputType`, whether suppressed, last composing state.
- [x] Counters: last caret-rect anchor and whether positioning was attempted.
- [x] Opt-in browser console logging for IME focus/cursor-area updates (`?ime_debug=1` / `window.__FRET_IME_DEBUG=true`).
- [x] Record a small `recent_events` ring buffer for ordering diagnostics (`beforeinput`/`input`/`composition*`/cursor area updates).
- [x] Surface `WindowTextInputSnapshotService` + `WindowInputContextService` snapshots in the UI Gallery harness panel for cross-layer debugging.
- [x] Surface `TextFontStackKey` + `TextFontFamilyConfig` + `FontCatalog` in the UI Gallery web IME harness panel for font/tofu debugging.
- [x] Add a UI Gallery “Dump layout…” button that writes a Taffy subtree dump to `.fret/taffy-dumps` for nested scroll/clip/layout debugging.

### Harness

- [x] Add a web harness/demo that exercises:
  - preedit updates,
  - commit,
  - backspace/arrows,
  - no double-insert on `compositionend`.
- [x] Validate glyph coverage (CJK/emoji) by enabling web demo font features (to avoid “tofu” squares).
- [!] Deferred: IME enable/focus is still flaky on some browsers/dev setups (activation-window timing). Keep `?demo=ui_gallery&page=web_ime_harness` as the repro surface and revisit later.
  - Triage recipe:
    - Load the harness with `?demo=ui_gallery&page=web_ime_harness&ime_debug=1` (console logs are opt-in).
    - Click the editor surface once (ensure the browser grants user activation), then check the harness panel snapshots:
      - `WindowInputContextService` (focus + `text_boundary_mode` + text-input classification)
      - `WindowTextInputSnapshotService` (preedit/selection ranges and UTF-16↔UTF-8 mapping clues)
    - If focus is flaky, confirm whether the bridge reports `enabled=1` and `textarea_has_focus=true` in the debug snapshot and inspect the recent DOM event ring buffer for ordering clues (`beforeinput`/`input`/`composition*`).

---

## M2 — Word Boundaries and Click Selection

### Mode seam

- [x] Define `TextBoundaryMode` and wire it into window-scoped `InputContext`.
- [x] Implement override stack service (push/pop token) for focused surfaces/overlays.
- [x] Default mode is `UnicodeWord` unless overridden.
- [x] Allow focused text input regions to override the mode (mechanism-only).
- [x] Allow code-editor-grade surfaces to select the mode explicitly (policy input), and expose a UI Gallery toggle.

### Command semantics

- [x] Ensure `text.move_word_*` and `text.select_word_*` consult the active mode.
- [x] Ensure double-click selects word and triple-click selects logical line (including trailing newline) (ADR 0151 + ADR 0194).
- [x] Ensure composing selection operates on display text (ADR 0071) (v1 policy: cancel inline preedit deterministically on selection/navigation; caret rect respects preedit cursor) (TextInput/TextArea double/triple-click cancel + command-driven navigation cancel; CodeEditor click selection cancel).

### Tests

- [x] Unicode word boundaries: Latin/CJK/emoji (seed tests added; expand coverage).
- [x] Identifier boundaries: underscores, digits, mixed scripts, punctuation (seed tests added; expand coverage).
- Note: expanded coverage in `crates/fret-ui/src/text_edit.rs` (mixed Latin/CJK/emoji; identifier punctuation).
- [x] Word navigation + deletion respect the active boundary mode across `SelectableText` / `TextInput` / `TextArea` (Ctrl/Alt+Arrow, Ctrl/Alt+Backspace/Delete; command path parity).
- [x] Double/triple click selection under scroll offsets and transforms (existing SelectableText tests; add mode coverage and TextInput/TextArea click selection).
  - Done: selectable text double-click respects `WindowTextBoundaryModeService` under `render_transform` and `Scroll` offset.
  - Done: text input + text area double-click respect `WindowTextBoundaryModeService` under `render_transform` and `Scroll` offset.
  - Done: text input triple-click selects logical line under `render_transform` and `Scroll` offset.
  - Done: text area triple-click selects logical line (including trailing newline) under `render_transform` and `Scroll` offset.

Evidence anchors:

- `crates/fret-runtime/src/input.rs` (`InputContext.text_boundary_mode`, `TextBoundaryMode`)
- `crates/fret-runtime/src/window_text_boundary_mode.rs` (`WindowTextBoundaryModeService`)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps.text_boundary_mode_override`)
- `crates/fret-ui/src/declarative/mount.rs` (mounts focused override into the runtime tree)
- `crates/fret-ui/src/tree/dispatch.rs` / `crates/fret-ui/src/tree/paint.rs` (publishes focused override in `InputContext`)
- `crates/fret-ui/src/text_edit.rs` (Unicode/identifier segmentation + tests)
- `crates/fret-ui/src/text_input/widget.rs` / `crates/fret-ui/src/text_area/widget.rs` / `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs` (integration)
- `crates/fret-ui/src/declarative/host_widget.rs` / `crates/fret-ui/src/text_input/bound.rs` / `crates/fret-ui/src/text_area/bound.rs` (platform text input delegation for declarative widgets)
- `crates/fret-ui/src/declarative/tests/interactions.rs` (scroll/transform double-click selection; double-click cancels IME preedit for TextInput/TextArea)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::set_text_boundary_mode`)
- `ecosystem/fret-code-editor/src/editor/tests/mod.rs` (double/triple click selection; a11y preedit window)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`, `preview_code_editor_torture` boundary mode toggle)

---

## M3 — Editor Surface MVP (native first, windowed)

### Windowed surface model

- [x] Choose the v1 surface implementation:
  - paint-driven windowed surface (preferred), or
  - VirtualList rows (only if composability is required early).
- [x] Define overscan policy and scroll stability expectations.

### Text preparation + caching

- [x] Prepare text per visible display row only (no monolithic document blob).
- [x] Define row cache keys and budgets (viewport-bounded, LRU-ish) (implemented; remaining work is tightening cache telemetry attribution across the renderer/canvas layers).
- [x] Replace the code editor monospace "cell width" heuristic with cached renderer caret stops for pointer hit-testing, caret, and selection geometry (keep the heuristic as a fallback until every backend implements caret stops).
- [x] Make vertical caret movement preserve a pixel `preferred_x` (per-row caret stops), not the last display column.
- [x] Draw selection using `TextService::selection_rects` when a row has a `TextBlobId` (fallback to caret stops / cell width).
- [x] Ensure theme-only changes remain paint-only (no reshaping).

### Input/IME integration

- [x] Inline preedit rendering (best-effort; underline + optional range highlight for v1).
- [x] Caret rect reporting for `ImeSetCursorArea` (native; best-effort).
- [x] Use renderer text caret rect metrics (caret y/height) when computing `ImeSetCursorArea` for editor-grade surfaces (fallback to row height when unavailable).
- [x] Align editor caret Y to the row text blob baseline (prevents caret drifting above glyphs on mixed-font / rich-span lines).
- [x] Provide a mechanism-only text input region for custom surfaces (no internal buffer).
- [x] Web/WASM: bind focus/key/command/pointer hooks to the `TextInputRegion` element id scope (not the outer keyed scope) so input routing attaches to the focused region.
- [x] Web/WASM: emit `ImeAllow` during pointer-down focus for editor-grade `TextInputRegion` surfaces (user-activation friendly textarea focusing).
- [x] UiTree: treat focused declarative `TextInput` / `TextArea` / `TextInputRegion` as text input when computing `focus_is_text_input` (prevents stale host-widget flags from disabling IME).

### Harness

- [x] Add a UI Gallery page for the editor MVP (manual interaction harness).
- [x] Add a “scroll stability / no stale paint” torture harness entry (ui-gallery style).
- [x] Fix the “no stale lines” torture failure (scroll-driven window changes must not show stale row text).
  - Mechanism: `ScrollProps.windowed_paint` forces view-cache rerender on scroll offset changes for windowed paint surfaces.
  - Surface glue: `windowed_rows_surface_with_pointer_region` also sets `scroll.windowed_paint = true` (the code editor uses the pointer-region variant).
  - Paint correctness: `windowed_rows_surface` now anchors row rects at the canvas bounds origin to avoid “left clipped / prefixes missing”.
  - Diagnostics gate: `tools/diag-scripts/ui-gallery-code-editor-torture-scroll-stability.json` + `--check-windowed-rows-offset-changes-min 1` (with UI Gallery view-cache enabled).
- [x] Add a soft-wrap + editing baseline gate (ui-gallery torture).
  - Script: `tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json`
  - Gates: `--check-windowed-rows-offset-changes-min 1` + `--check-pixels-changed ui-gallery-code-editor-torture-root`
- [x] Clamp windowed row hit-testing during drags (keeps selection updates continuous when the pointer leaves bounds).
- [x] Drag-to-select edge autoscroll (Zed-style scaling), including a timer-driven path so it continues while the pointer is stationary at the viewport edge.

Evidence anchors:

- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditor`, row painting + selection/caret + IME)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps`, `ElementKind::TextInputRegion`)
- `crates/fret-ui/src/declarative/host_widget/event/text_input_region.rs` (IME/TextInput forwarding)
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs` (`row_index_for_pointer` clamp, `on_timer` wiring)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_MVP`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_TORTURE`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_torture`)

---

## M4 — Buffer Model + Undo Hooks

- [x] Choose v1 buffer structure: rope (`ropey`) while preserving the UTF-8 byte-index contract.
- [x] Lock edit op vocabulary (insert/delete/replace) in UTF-8 byte indices.
- [x] Lock transaction hooks (begin/update/commit/cancel) compatible with ADR 0136.
- [x] Lock document identity (URI-like) for multi-document workflows.

Evidence anchors:

- `ecosystem/fret-code-editor-buffer/src/lib.rs` (`TextBuffer`, `Edit`, `TextBufferTransaction`, `TextBufferTx`, `apply_in_transaction`, `rollback_transaction`)
- `ecosystem/fret-code-editor-buffer/src/lib.rs` (`DocId`, `DocUri`, `TextBuffer::uri`, `TextBuffer::set_uri`)
- `ecosystem/fret-code-editor/src/lib.rs` (`UndoGroupKind`, `UndoGroup`, `apply_and_record_edit`, `UndoHistory::record_or_coalesce`)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::replace_buffer`, `CodeEditorHandle::set_text`)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::set_language`, `cached_row_syntax_spans`, `materialize_row_rich_text`)
- `ecosystem/fret-code-editor/Cargo.toml` (`syntax` / `syntax-rust` / `syntax-all`)

---

## M5 — Syntax Highlighting (incremental + visible-window materialization)

- [x] Define semantic token schema (highlight ids independent of theme colors).
- [x] Incremental update strategy (best-effort; visible window prioritized) (bounded line-window invalidation via `BufferDelta`, plus far-row cache key shifting when line count changes; validated by `invalidate_syntax_row_cache_for_delta` + `syntax_cache_invalidation_*` tests under `syntax-rust`).
- [x] Materialize spans only for visible rows.
- [x] Expose a UI Gallery toggle for manual validation.
- [x] Theme changes update paint-only styles without reshaping.

---

## M6 — Semantics (a11y) and selection state

- [x] Define semantics role for the editor surface (v1: a `TextField` node via `TextInputRegion`, plus a sibling `Viewport` node for the scrollable windowed surface).
- [x] Ensure selection and composition ranges follow ADR 0071 rules (UTF-8 byte offsets into the exported `value`; code editor handles `SetTextSelection` best-effort within its windowed value and cancels inline preedit deterministically).
- [x] Decide whether to expose visible-row-only semantics or a stub/viewport role for v1 (documented in workstream; v1 chooses stub/viewport semantics).
  - [x] Add regression gates for selection/composition invariants (including wrap and drag-selection cases):
    - `tools/diag-scripts/ui-gallery-code-editor-a11y-selection-baseline.json`
    - `tools/diag-scripts/ui-gallery-code-editor-a11y-composition-baseline.json`
    - `tools/diag-scripts/ui-gallery-code-editor-a11y-composition-drag-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (a11y selection/composition checkers + evidence JSON)

---

## M7 — Diagnostics and perf attribution

- [x] Add bundle-friendly counters (v1 baseline):
  - visible rows + overscan (windowed surfaces),
  - editor-local cache hits/misses (row text + syntax).
- [x] Ensure windowed surface window telemetry is exported in diagnostics snapshots (align with ADR 0190).
- [x] Export editor/IME harness state into diagnostics snapshots (ui-gallery app snapshot + web IME bridge snapshot; enables “single artifact” repros).
- [x] Add renderer-level churn counters:
  - Text blob churn + glyph atlas pressure are captured by the runner as a per-frame app global (`fret_core::RendererTextPerfSnapshot`) and exported into UI diagnostics bundles.
  - Evidence: `crates/fret-core/src/render_text.rs`, `crates/fret-render/src/text.rs`, `crates/fret-launch/src/runner/desktop/app_handler.rs`, `crates/fret-launch/src/runner/web.rs`, `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.

---

## M8 — Display Map Expansion (wrap/fold/inlay) (optional v1 → v2)

- [~] Soft wrap with stable coordinate mapping (buffer ↔ display ↔ pixels) (column-based baseline; pixel-accurate caret/selection/hit-test is migrating to renderer caret stops).
  - [x] Add wrap-boundary semantics regression gates (UI Gallery harness + fretboard diag):
    - `tools/diag-scripts/ui-gallery-code-editor-a11y-selection-wrap-baseline.json`
    - `tools/diag-scripts/ui-gallery-code-editor-a11y-composition-wrap-baseline.json`
    - `apps/fret-ui-gallery/src/ui.rs` (wrap gate viewports + preedit inject/clear buttons)
    - `apps/fretboard/src/diag/stats.rs` (wrap gate checkers + evidence JSON)
- [~] Fold regions + placeholders without breaking caret/selection.
  - [x] Unwrapped baseline: materialize per-line fold placeholders and map caret/selection/hit-test between buffer-local and display-local indices.
  - [x] Add a UI Gallery fixture toggle and a bundle gate that asserts the fold placeholder is observed at least once:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-folds-placeholder-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present`)
  - [x] Add a soft-wrap gate that asserts fold placeholders are visible under soft wrap:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present_under_soft_wrap`)
  - [x] Define v1 behavior for inline preedit: suppress fold placeholders while inline preedit is active, and lock it with a regression gate:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-folds-soft-wrap-inline-preedit-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_absent_under_inline_preedit`)
  - [x] Decision (v2): keep v1 behavior — suppress fold placeholders while inline preedit is active.
    - Rationale: composing fold placeholders with preedit requires fragment-based DisplayMap composition (unified buffer↔display↔a11y mapping).
    - Revisit once preedit is modeled as an injected display fragment (similar to inlays) rather than a paint-time string splice.
- [~] Inlays (injected display fragments) without mutating the underlying buffer.
  - [x] Unwrapped baseline: inject per-line inlay text and include it in the same buffer↔display mapping used by caret/selection/hit-test.
  - [x] Add a UI Gallery fixture toggle and a bundle gate that asserts the inlay fixture is observed at least once:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-inlays-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_inlays_present`)
  - [x] Add a soft-wrap gate that asserts inlays are visible under soft wrap:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_inlays_present_under_soft_wrap`)
  - [x] Define v1 behavior for inline preedit: suppress inlays while inline preedit is active, and lock it with a regression gate:
    - `tools/diag-scripts/ui-gallery-code-editor-torture-inlays-soft-wrap-inline-preedit-baseline.json`
    - `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_inlays_absent_under_inline_preedit`)
  - [x] Decision (v2): keep v1 behavior — suppress inlays while inline preedit is active.
    - Rationale: composing inlays with preedit requires fragment-based DisplayMap composition (unified buffer↔display↔a11y mapping).
    - Revisit once preedit is modeled as an injected display fragment (and the mapping surface can compose multiple fragment sources deterministically).
  - [ ] Follow-up (v2+): define a fragment-based DisplayMap composition model so fold/inlay/preedit can compose under a single mapping surface.
    - Proposed: `docs/adr/0203-code-editor-display-fragments-and-displaymap-composition-v1.md`.

---

## M9 — Retained Host / Composable Rows (only if required)

- [ ] Decide whether we need composable per-row subtrees (embedded widgets, rich gutters).
- [ ] If yes, adopt the retained host direction (ADR 0192) so window boundary crossings do not force parent rerenders.

---

## M10 — Markdown Editor v0 (source mode) (downstream validation)

This milestone is defined in `docs/workstreams/code-editor-ecosystem-v1.md` (“Downstream Milestone:
Markdown Editor v0”).

### M10.1 — Source editor shell + interaction control

- [x] Define a minimal interaction control surface for `fret-code-editor`:
  - `CodeEditorInteractionOptions` (policy surface) + input gating.
  - Evidence: `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeEditorInteractionOptions`, `CodeEditorState::set_interaction`),
    `ecosystem/fret-code-editor/src/editor/input/mod.rs` (edit/undo/redo gating).
- [x] Add a UI Gallery toggle + diag coverage for read-only behavior (typing does not mutate the buffer).
  - UI: `apps/fret-ui-gallery/src/ui.rs` (Mode: edit/read-only/disabled buttons).
  - Script: `tools/diag-scripts/ui-gallery-code-editor-torture-read-only-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_read_only_blocks_edits`).
- [x] Add Markdown syntax highlighting support (feature-gated; prefer `fret-syntax/lang-md`).
  - Evidence: `ecosystem/fret-code-editor/Cargo.toml` (`syntax-markdown`), `apps/fret-ui-gallery/Cargo.toml` (enables feature),
    `apps/fret-ui-gallery/src/ui.rs` (`handle.set_language(Some(\"markdown\"))`).
- [x] Add a UI Gallery “markdown_editor_source” page:
  - code editor configured for Markdown (soft wrap toggle),
  - split preview via `fret-markdown`.
  - Evidence: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MARKDOWN_EDITOR_SOURCE`),
    `apps/fret-ui-gallery/src/ui.rs` (`preview_markdown_editor_source`).
- [x] Add a UI Gallery toggle + diag coverage for disabled behavior on the Markdown editor page.
  - UI: `apps/fret-ui-gallery/src/ui.rs` (`ui-gallery-markdown-editor-mode-disabled`).
  - Scripts:
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-disabled-baseline.json`.
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-disabled-inject-preedit-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits`).
    - Asserts: no buffer mutations, and the disabled editor is not focused with no composition.

### M10.2 — Soft-wrap + selection/navigation consistency

- [x] Add a read-only regression gate for the Markdown editor page (typing does not mutate the buffer).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-read-only-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_read_only_blocks_edits`).
- [x] Add a soft-wrap toggle stability gate for the Markdown editor page (caret/revision remain stable).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-toggle-stability-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_toggle_stable`).
- [x] Add Markdown editor word-boundary regressions (ADR 0194; UnicodeWord baseline) using semantics selection.
  - Scripts:
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-baseline.json`
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-baseline.json`
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json`
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json`
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json`
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_word_boundary`).
- [x] Add a soft-wrap editing regression: selection mapping remains stable under wrap while editing (not just toggles).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_soft_wrap_editing_selection_wrap_stable`).
- [x] Follow-up: add a triple-click select-line baseline for the Markdown editor page (ADR 0194).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-line-boundary-triple-click-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_line_boundary_triple_click`).
- [x] Add fold/inlay decoration baselines for the Markdown editor page (ADR 0200; present under wrap; suppressed under inline preedit).
  - Scripts:
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-placeholder-baseline.json`.
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-baseline.json`.
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-inline-preedit-baseline.json`.
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-baseline.json`.
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-baseline.json`.
    - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-inline-preedit-baseline.json`.
  - Gates: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_*folds*`, `check_bundle_for_ui_gallery_markdown_editor_source_*inlays*`).
- [x] Add a folds clamp-selection regression for the Markdown editor fixture (ADR 0200).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-clamp-selection-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_folds_clamp_selection_out_of_folds`).
  - Flag: `--check-ui-gallery-markdown-editor-source-folds-clamp-selection-out-of-folds`.
- [x] Add an inlays caret-navigation regression for the Markdown editor fixture (ADR 0200).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-caret-navigation-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_inlays_caret_navigation_stable`).
  - Flag: `--check-ui-gallery-markdown-editor-source-inlays-caret-navigation-stable`.

### M10.3 — IME bridge seam validation (native + web)

- [x] Add a Markdown editor a11y composition regression (ADR 0071 range invariants; synthetic preedit injection).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-a11y-composition-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition`).
- [x] Add a soft-wrap a11y composition regression (same invariants with wrap=80 enabled).
  - Script: `tools/diag-scripts/ui-gallery-markdown-editor-source-a11y-composition-soft-wrap-baseline.json`.
  - Gate: `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_a11y_composition_soft_wrap`).
- [ ] Deferred: Web IME bridge attach baseline (ADR 0195) (best-effort; non-flaky baseline only).
  - Decision: keep as a manual harness until it can be stabilized.

### M10.4 — Diag suite / definition-of-done

- [x] Minimal diag script suite validates the Markdown editor milestone end-to-end:
  - read-only blocks edits,
  - soft-wrap toggle stability,
  - word-boundary baselines (including double-click),
  - a11y composition baseline,
  - soft-wrap editing selection-wrap stability.
