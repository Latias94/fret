# Code Editor Ecosystem v1 - TODO Tracker

Status: Active (workstream tracker)
Last updated: 2026-02-04

This is the checkbox tracker companion to:

- `docs/workstreams/code-editor-ecosystem-v1.md`

Normative contracts:

- `docs/adr/0193-code-editor-ecosystem-v1.md`
- `docs/adr/0194-text-navigation-and-word-boundaries-v1.md`
- `docs/adr/0195-web-ime-and-text-input-bridge-v1.md`

Legend:

- [ ] pending
- [~] in progress
- [x] done
- [!] blocked / needs decision

---

## M0 — Contracts Locked

- [ ] Review ADR 0193 and confirm crate split and v1 baseline (windowed surface first).
- [ ] Review ADR 0194 and confirm the preferred seam:
  - window-scoped `InputContext.text_boundary_mode` + override stack.
- [x] Review ADR 0195 and confirm web strategy:
  - hidden textarea bridge,
  - `beforeinput` + `composition*` translation,
  - proxy mode (no full document mirroring).
- [x] Add 1–3 evidence anchors per ADR (file paths / tests) in ADR 0193/0194/0195.

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
- [~] Implement command-path suppression to avoid “command executes + DOM inserts text” (shortcut suppression landed; keep auditing edge cases).

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
- [~] Ensure composing selection operates on display text (ADR 0071) (v1 policy: cancel inline preedit deterministically on selection/navigation; caret rect respects preedit cursor) (TextInput display→base hit-test mapping fixed; tests added for TextInput/TextArea double-click cancel; CodeEditor click selection cancel).

### Tests

- [~] Unicode word boundaries: Latin/CJK/emoji (seed tests added; expand coverage).
- [~] Identifier boundaries: underscores, digits, mixed scripts, punctuation (seed tests added; expand coverage).
- Note: expanded coverage in `crates/fret-ui/src/text_edit.rs` (mixed Latin/CJK/emoji; identifier punctuation).
- [~] Double/triple click selection under scroll offsets and transforms (existing SelectableText tests; add mode coverage and TextInput/TextArea click selection).
  - Done: selectable text double-click respects `WindowTextBoundaryModeService` under `render_transform` and `Scroll` offset.
  - Done: text input + text area double-click respect `WindowTextBoundaryModeService` under `render_transform` and `Scroll` offset.

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
- [~] Define row cache keys and budgets (viewport-bounded, LRU-ish) (row text + syntax spans are bounded; text system cache/telemetry alignment pending).
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
  - Paint correctness: `windowed_rows_surface` now anchors row rects at the canvas bounds origin to avoid “left clipped / prefixes missing”.
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
- [~] Lock document identity (URI-like) for multi-document workflows.

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
- [~] Incremental update strategy (best-effort; visible window prioritized) (implemented: bounded line-window invalidation via `BufferDelta`, and far-row cache key shifting when line count changes; see `invalidate_syntax_row_cache_for_delta` + `syntax_cache_invalidation_*` tests under `syntax-rust`).
- [x] Materialize spans only for visible rows.
- [x] Expose a UI Gallery toggle for manual validation.
- [x] Theme changes update paint-only styles without reshaping.

---

## M6 — Semantics (a11y) and selection state

- [~] Define semantics role for the editor surface (current baseline: `TextInputRegion` emits `SemanticsRole::TextField`).
- [~] Ensure selection and composition ranges follow ADR 0071 rules (baseline: app-provided UTF-8 ranges within an app-provided value; code editor handles `SetTextSelection` best-effort within its windowed value).
- [x] Decide whether to expose visible-row-only semantics or a stub/viewport role for v1 (documented in workstream; v1 chooses stub/viewport semantics).

---

## M7 — Diagnostics and perf attribution

- [x] Add bundle-friendly counters (v1 baseline):
  - visible rows + overscan (windowed surfaces),
  - editor-local cache hits/misses (row text + syntax).
- [x] Ensure windowed surface window telemetry is exported in diagnostics snapshots (align with ADR 0190).
- [x] Export editor/IME harness state into diagnostics snapshots (ui-gallery app snapshot + web IME bridge snapshot; enables “single artifact” repros).
- [ ] Add renderer-level churn counters (next):
  - text blob churn and glyph atlas pressure (likely from renderer/canvas caches).

---

## M8 — Display Map Expansion (wrap/fold/inlay) (optional v1 → v2)

- [~] Soft wrap with stable coordinate mapping (buffer ↔ display ↔ pixels) (column-based baseline; pixel-accurate caret/selection/hit-test is migrating to renderer caret stops).
- [ ] Fold regions + placeholders without breaking caret/selection.
- [ ] Inlays (injected display fragments) without mutating the underlying buffer.

---

## M9 — Retained Host / Composable Rows (only if required)

- [ ] Decide whether we need composable per-row subtrees (embedded widgets, rich gutters).
- [ ] If yes, adopt the retained host direction (ADR 0192) so window boundary crossings do not force parent rerenders.
