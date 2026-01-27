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
- [x] Provide a focused-surface override via `TextInputRegion` (code editor defaults to `Identifier`).
- [ ] Implement override stack service (push/pop token) if needed for non-focus-based policies.
- [x] Default mode is `UnicodeWord` unless overridden.

### Command semantics

- [x] Ensure `text.move_word_*` and `text.select_word_*` consult the active mode.
- [ ] Ensure double-click selects word and triple-click selects logical line (ADR 0151 + ADR 0194).
- [ ] Ensure composing selection operates on display text (ADR 0071).

### Tests

- [x] Unicode word boundaries: Latin/CJK/emoji.
- [x] Identifier boundaries: underscores, digits, mixed scripts, punctuation.
- [x] Window input context snapshots include `text_boundary_mode` and arbitration.
- [ ] Double/triple click selection under scroll offsets and transforms.

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

- [x] Inline preedit rendering.
- [x] Caret rect reporting for `ImeSetCursorArea` (native).

### Harness

- [x] Add a “scroll stability / no stale paint” torture harness entry (ui-gallery style).

---

## M4 — Buffer Model + Undo Hooks

- [ ] Choose v1 buffer structure (rope / piece table / hybrid).
- [ ] Lock edit op vocabulary (insert/delete/replace) in UTF-8 byte indices.
- [ ] Lock transaction hooks (begin/update/commit/cancel) compatible with ADR 0136.
- [ ] Lock document identity (URI-like) for multi-document workflows.

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
