# Code Editor Ecosystem v1 — TODO Tracker

Status: Active (workstream tracker)
Last updated: 2026-01-27

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
- [ ] Review ADR 0195 and confirm web strategy:
  - hidden textarea bridge,
  - `beforeinput` + `composition*` translation,
  - proxy mode (no full document mirroring).
- [ ] Add 1–3 evidence anchors per ADR (file paths / tests) once implementation starts.

---

## M1 — Web IME Bridge (wasm baseline)

### DOM element lifecycle

- [~] Create the hidden textarea element (currently global; TODO per-window + canvas overlay attachment).
- [x] Define focus/blur rules and map them to `Effect::ImeAllow`.
- [x] Define best-effort caret anchoring and map it to `Effect::ImeSetCursorArea`.

### Event translation

- [x] Translate `compositionstart/update/end` to `Event::Ime` (preedit/commit).
- [x] Translate `beforeinput`/`input` to `Event::TextInput` for committed insertions.
- [x] Filter control characters from `TextInput` (ADR 0012).
- [~] Implement command-path suppression to avoid “command executes + DOM inserts text” (shortcut suppression landed; keep auditing edge cases).

### UTF-16 ↔ UTF-8 conversion

- [x] Implement deterministic conversion + clamping utilities.
- [x] Add tests for mixed-script and emoji sequences (byte offsets remain valid).

### Observability (debug-only)

- [ ] Counters: last `inputType`, whether suppressed, last composing state.
- [ ] Counters: last caret-rect anchor and whether positioning was attempted.

### Harness

- [x] Add a web harness/demo that exercises:
  - preedit updates,
  - commit,
  - backspace/arrows,
  - no double-insert on `compositionend`.

---

## M2 — Word Boundaries and Click Selection

### Mode seam

- [x] Define `TextBoundaryMode` and wire it into window-scoped `InputContext`.
- [x] Implement override stack service (push/pop token) for focused surfaces/overlays.
- [x] Default mode is `UnicodeWord` unless overridden.
- [x] Allow focused text input regions to override the mode (mechanism-only).

### Command semantics

- [x] Ensure `text.move_word_*` and `text.select_word_*` consult the active mode.
- [x] Ensure double-click selects word and triple-click selects logical line (ADR 0151 + ADR 0194).
- [ ] Ensure composing selection operates on display text (ADR 0071).

### Tests

- [~] Unicode word boundaries: Latin/CJK/emoji (seed tests added; expand coverage).
- [~] Identifier boundaries: underscores, digits, mixed scripts, punctuation (seed tests added; expand coverage).
- [~] Double/triple click selection under scroll offsets and transforms (existing SelectableText tests; add mode coverage and TextInput/TextArea click selection).

Evidence anchors:

- `crates/fret-runtime/src/input.rs` (`InputContext.text_boundary_mode`, `TextBoundaryMode`)
- `crates/fret-runtime/src/window_text_boundary_mode.rs` (`WindowTextBoundaryModeService`)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps.text_boundary_mode_override`)
- `crates/fret-ui/src/declarative/mount.rs` (mounts focused override into the runtime tree)
- `crates/fret-ui/src/tree/dispatch.rs` / `crates/fret-ui/src/tree/paint.rs` (publishes focused override in `InputContext`)
- `crates/fret-ui/src/text_edit.rs` (Unicode/identifier segmentation + tests)
- `crates/fret-ui/src/text_input/widget.rs` / `crates/fret-ui/src/text_area/widget.rs` / `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs` (integration)

---

## M3 — Editor Surface MVP (native first, windowed)

### Windowed surface model

- [x] Choose the v1 surface implementation:
  - paint-driven windowed surface (preferred), or
  - VirtualList rows (only if composability is required early).
- [x] Define overscan policy and scroll stability expectations.

### Text preparation + caching

- [x] Prepare text per visible display row only (no monolithic document blob).
- [x] Define row cache keys and budgets (viewport-bounded, LRU-ish).
- [ ] Ensure theme-only changes remain paint-only (no reshaping).

### Input/IME integration

- [x] Inline preedit rendering (best-effort overlay for v1).
- [x] Caret rect reporting for `ImeSetCursorArea` (native; best-effort).
- [x] Provide a mechanism-only text input region for custom surfaces (no internal buffer).

### Harness

- [x] Add a UI Gallery page for the editor MVP (manual interaction harness).
- [x] Add a “scroll stability / no stale paint” torture harness entry (ui-gallery style).

Evidence anchors:

- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditor`, row painting + selection/caret + IME)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps`, `ElementKind::TextInputRegion`)
- `crates/fret-ui/src/declarative/host_widget/event/text_input_region.rs` (IME/TextInput forwarding)
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs` (`on_pointer_up`/`on_pointer_cancel`)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_MVP`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_TORTURE`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_torture`)

---

## M4 — Buffer Model + Undo Hooks

- [~] Choose v1 buffer structure (rope / piece table / hybrid) (seed `TextBuffer` exists; internal structure decision pending).
- [x] Lock edit op vocabulary (insert/delete/replace) in UTF-8 byte indices.
- [x] Lock transaction hooks (begin/update/commit/cancel) compatible with ADR 0136.
- [~] Lock document identity (URI-like) for multi-document workflows.

Evidence anchors:

- `ecosystem/fret-code-editor-buffer/src/lib.rs` (`TextBuffer`, `Edit`, `TextBufferTransaction`, `TextBufferTx`, `apply_in_transaction`, `rollback_transaction`)
- `ecosystem/fret-code-editor-buffer/src/lib.rs` (`DocId`, `DocUri`, `TextBuffer::uri`, `TextBuffer::set_uri`)
- `ecosystem/fret-code-editor/src/lib.rs` (`UndoGroupKind`, `UndoGroup`, `apply_and_record_edit`, `UndoHistory::record_or_coalesce`)

---

## M5 — Syntax Highlighting (incremental + visible-window materialization)

- [ ] Define semantic token schema (highlight ids independent of theme colors).
- [ ] Incremental update strategy (best-effort; visible window prioritized).
- [ ] Materialize spans only for visible rows.
- [ ] Theme changes update paint-only styles without reshaping.

---

## M6 — Display Map Expansion (wrap/fold/inlay) (optional v1 → v2)

- [ ] Soft wrap with stable coordinate mapping (buffer ↔ display ↔ pixels).
- [ ] Fold regions + placeholders without breaking caret/selection.
- [ ] Inlays (injected display fragments) without mutating the underlying buffer.

---

## M7 — Retained Host / Composable Rows (only if required)

- [ ] Decide whether we need composable per-row subtrees (embedded widgets, rich gutters).
- [ ] If yes, adopt the retained host direction (ADR 0192) so window boundary crossings do not force parent rerenders.
