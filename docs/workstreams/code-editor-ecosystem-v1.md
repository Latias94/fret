# Code Editor Ecosystem v1 — Refactor Plan & TODO Tracker

Status: Draft (workstream document; normative contracts live in ADRs)
Last updated: 2026-01-27

This document is an implementation-focused tracker for building an editor-grade **code editor ecosystem** for Fret.
It is intentionally non-authoritative; the normative contracts are:

- `docs/adr/0193-code-editor-ecosystem-v1.md`
- `docs/adr/0194-text-navigation-and-word-boundaries-v1.md`
- `docs/adr/0195-web-ime-and-text-input-bridge-v1.md`

Non-normative upstream reference anchors (for concept mapping only; do not copy implementations):

- Zed/GPUI (copyleft; architecture reference only):
  - `repo-ref/zed/crates/editor/src/display_map.rs`
  - `repo-ref/zed/crates/editor/src/editor.rs`
  - `repo-ref/zed/crates/rope/src/rope.rs`
  - `repo-ref/zed/crates/text/src/text.rs`
- Monaco Editor (MIT; concept reference):
  - `repo-ref/monaco-editor/node_modules/monaco-editor-core/esm/vs/editor/common/model/textModel.js`
  - `repo-ref/monaco-editor/node_modules/monaco-editor-core/esm/vs/editor/common/model/pieceTreeTextBuffer/pieceTreeTextBuffer.js`
  - `repo-ref/monaco-editor/node_modules/monaco-editor-core/esm/vs/editor/common/model/pieceTreeTextBuffer/pieceTreeBase.js`

It also depends on existing locked contracts:

- Text boundary + geometry queries: `docs/adr/0006-text-system.md`, `docs/adr/0045-text-geometry-queries-hit-testing-and-caret-metrics.md`,
  `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- IME model: `docs/adr/0012-keyboard-ime-and-text-input.md`, `docs/adr/0071-text-input-multiline-composition-contract.md`
- Text editing commands: `docs/adr/0044-text-editing-state-and-commands.md`
- Cache roots + dirty views: `docs/adr/1152-cache-roots-and-cached-subtree-semantics-v1.md`, `docs/adr/0180-dirty-views-and-notify-gpui-aligned.md`
- Prepaint and multi-stream reuse direction: `docs/adr/0182-prepaint-interaction-stream-and-range-reuse.md`
- Windowed virtual surfaces and retained hosts: `docs/adr/0190-prepaint-windowed-virtual-surfaces.md`, `docs/adr/0192-retained-windowed-surface-hosts.md`
- Virtualization baseline: `docs/adr/0042-virtualization-and-large-lists.md`
- Semantics/a11y baseline: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`, `docs/adr/0085-virtualized-accessibility-and-collection-semantics.md`

---

## Scope (what we are building)

We want an ecosystem-grade code editor surface that supports:

- Large documents (100k+ lines) without unbounded UI trees.
- Editor-grade IME correctness (desktop first; wasm/mobile considered by contract).
- Deterministic selection/caret/hit-testing via the renderer text boundary.
- Stable command vocabulary (`text.*` baseline + `editor.*` extensions where needed).
- Performance explainability via cache roots + windowed surfaces (scroll-driven changes should not force full rerenders).

We are **not** building “the editor app”; we are building reusable ecosystem crates and runtime seams.

---

## Non-goals (v1)

- Collaboration/CRDT/OT.
- Full VSCode-class feature parity.
- Cross-element text selection (still out of baseline; see ADR 0152).

---

## Architectural Principles (performance-first, Fret-aligned)

1) **No CSS runtime dependency**
   - Fret remains a non-DOM UI runtime: layout and styling are `LayoutStyle` + theme tokens (ADR 0066).
   - On wasm, a **hidden DOM textarea** may require setting inline styles to function reliably for IME.
     This is a runner implementation detail, not a framework contract (ADR 0195).

2) **Windowed surface first**
   - The editor UI should be a “windowed virtual surface” (ADR 0190): bounded work per frame based on
     viewport + overscan, not document length.

3) **Line-local text blobs**
   - No monolithic `TextBlobId` for an entire document. Prepare and cache text per visible row (ADR 0193).

4) **Theme-only updates should be paint-only**
   - Syntax highlighting should be expressed as semantic tokens and materialized as paint-only spans for visible rows.

5) **Keep runtime mechanism-only**
   - Word boundary rules, code-specific behaviors, and editor defaults belong in ecosystem policy layers (ADR 0066).

---

## Target Crate Layout (v1 direction)

See ADR 0193 for the normative split. This workstream assumes:

- `ecosystem/fret-code-editor-buffer`: document model + edits + selection + undo hooks
- `ecosystem/fret-code-editor-view`: display mapping + invalidation + syntax token projection
- `ecosystem/fret-code-editor`: Fret UI integration (windowed surface + input/IME + rendering)

Notes:

- v1 may start with fewer crates for iteration speed, but the public surfaces should preserve the split.
- Code must remain original; Zed is a reference for outcomes only (license constraints).

---

## Key Runtime/Platform Seams We Must Not Miss

### A) Prepaint streams + cache-hit correctness

The editor depends on the runtime’s direction toward:

- `prepaint` as the phase that can update interaction geometry and windowed surface membership
  without forcing re-execution of an entire cache root.

Evidence and dependencies:

- ADR 0182 (Proposed) defines multi-stream recording and reuse.
- ADR 0190/0192 describe the windowing model and the retained host direction.

Workstream implication:

- If `prepaint` stream reuse is not implemented, editor window changes will either:
  - force rerender of a large cache root, or
  - produce stale interaction/semantics ranges.

### B) Word boundaries and double-click selection

We need a stable seam for “word boundary mode”:

- default: Unicode word boundaries for general UI text
- override: identifier boundaries for code editor surfaces

This must be window-scoped and explainable (ADR 0194).

### C) Web IME bridge

On wasm/mobile, reliable IME typically requires a focused DOM input element.
The bridge must remain runner-owned and preserve Fret’s event model (ADR 0195).

---

## Milestones (recommended)

Each milestone has “exit criteria” that should be demonstrably true (tests, harness pages, or diagnostics).

### M0 — Contracts locked (docs + agreement)

Exit criteria:

- ADR 0193/0194/0195 reviewed and accepted or revised with explicit decisions.
- This workstream document reflects the accepted decisions and links to evidence anchors.

### M1 — Web IME bridge spike (wasm baseline)

Exit criteria:

- A web runner can:
  - enable/disable IME via existing effects (`ImeAllow`),
  - receive preedit/commit as `Event::Ime`,
  - receive committed insertions as `Event::TextInput` without control characters.
- UTF-16 ↔ UTF-8 conversion is deterministic and clamps to char boundaries.
- A minimal interactive demo/harness exists that exercises:
  - composition update,
  - commit,
  - backspace/arrow keys (command-path),
  - and does not “double-insert” on `compositionend` + `input`.

Implemented (partial evidence):

- UTF-16 ↔ UTF-8 deterministic conversion + clamping:
  - `crates/fret-core/src/utf.rs` (unit tests included)
- Hidden textarea IME bridge (wasm32):
  - `crates/fret-platform-web/src/wasm.rs` (`WebPlatformServices` handles `Effect::ImeAllow` / `Effect::ImeSetCursorArea`)
- Harness page (web UI gallery):
  - `apps/fret-ui-gallery/src/spec.rs` (`PAGE_WEB_IME_HARNESS`)
  - `apps/fret-ui-gallery/src/ui.rs` (`preview_web_ime_harness`)
- WASM build plumbing for the harness:
  - `.cargo/config.toml` (selects `getrandom_backend="wasm_js"` for `wasm32-unknown-unknown`)
  - `crates/fret-launch/Cargo.toml` (wasm `getrandom` dependency)
  - `apps/fret-ui-gallery/Cargo.toml` (avoid native `tree-sitter` deps for wasm)

### M2 — Word boundary mode seam (runtime mechanism; ecosystem policy)

Exit criteria:

- A window-scoped `text_boundary_mode` can be set to:
  - `UnicodeWord` (default),
  - `Identifier` (code surface override).
- `TextInput`, `TextArea`, and `SelectableText` use the mode consistently for:
  - `text.move_word_*` and `text.select_word_*`,
  - double-click select-word behavior,
  - triple-click select-line behavior.

Implemented (evidence):

- `crates/fret-runtime/src/window_text_boundary_mode.rs` (`WindowTextBoundaryModeService`)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps::text_boundary_mode_override`)
- `crates/fret-ui/src/tree/dispatch.rs` (`focus_text_boundary_mode_override`)
- Tests:
  - `crates/fret-ui/src/text_edit.rs` (Unicode/Identifier boundary unit tests)
  - `crates/fret-ui/src/tree/tests/window_input_context_snapshot.rs`

### M3 — Editor surface MVP (native first, windowed)

Exit criteria:

- A windowed editor surface exists that:
  - scrolls smoothly (bounded work per frame),
  - supports caret + selection + clipboard + undo hooks,
  - integrates IME preedit and cursor-area effects (native).
- Row text is prepared and cached per visible window (no whole-doc blob).

Implemented (evidence):

- Crate split:
  - `ecosystem/fret-code-editor-buffer`
  - `ecosystem/fret-code-editor-view`
  - `ecosystem/fret-code-editor`
- Word/line selection + word navigation (Identifier mode):
  - `ecosystem/fret-code-editor-view/src/lib.rs` (`select_word_range`, `move_word_left/right`)
  - `ecosystem/fret-code-editor/src/lib.rs` (double/triple click selection + `text.move_word_*` routing)
- Undo/redo wiring (widget-owned history for MVP):
  - `ecosystem/fret-code-editor/src/lib.rs` (`edit.undo` / `edit.redo` command handling)
- Surface + caches:
  - `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditor`, row text cache, torture overlay)
  - `crates/fret-ui/src/canvas.rs` / `crates/fret-ui/src/element.rs` (`CanvasCachePolicy.shared_text`)
- Harness pages:
  - `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_TORTURE`)
  - `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_torture`)
  - `apps/fret-ui-gallery/src/docs.rs`

### M4 — Incremental highlighting (visible-window materialization)

Exit criteria:

- Syntax state updates incrementally on edits (best-effort).
- Only visible rows are materialized into spans for paint.
- Theme changes do not trigger reshaping; only paint changes.

### M5 — Display map growth (wrap/fold/inlay) (optional for v1)

Exit criteria:

- Soft wrap is supported with stable coordinate mapping (buffer ↔ display ↔ pixels).
- Fold regions and placeholders (if adopted) do not break caret/selection semantics.
- Inlays (if adopted) are represented without mutating the underlying buffer.

### M6 — Composable rows / retained host (only if needed)

Exit criteria:

- If the editor requires composable per-row subtrees (embedded widgets, rich gutters),
  adopt the retained host direction (ADR 0192) so window boundary crossings do not force parent rerenders.

---

## TODO Tracker (living checklist)

Legend:

- [ ] pending
- [~] in progress
- [x] done
- [!] blocked / needs decision

### 0) Contracts (ADRs)

- [ ] Review ADR 0193 and confirm crate split and v1 baseline (windowed surface first).
- [ ] Review ADR 0194 and confirm the preferred seam:
  - window-scoped `InputContext.text_boundary_mode` + override stack.
- [ ] Review ADR 0195 and confirm web strategy:
  - hidden textarea bridge,
  - `beforeinput` + `composition*` translation rules,
  - proxy mode (no full document mirroring).

### 1) Web runner IME bridge (ADR 0195)

- [ ] Define DOM element strategy: textarea creation, attach layer, z-order and isolation.
- [ ] Define focus lifecycle and mapping to `Effect::ImeAllow`.
- [ ] Define caret anchoring mapping to `Effect::ImeSetCursorArea` (best-effort, mobile-leaning).
- [ ] Define event translation and suppression rules:
  - `beforeinput` → `TextInput` (filtered),
  - `composition*` → `ImeEvent`,
  - command-dispatch suppression to avoid double insert.
- [ ] Implement UTF-16 ↔ UTF-8 conversion utility with deterministic clamping.
- [ ] Add debug-only counters/logging for bridge behavior (last inputType, suppression reason, focused state).
- [ ] Add a web harness page (or demo mode) dedicated to IME conformance.

### 2) Word boundaries seam (ADR 0194)

- [x] Add/standardize `TextBoundaryMode` definition (`fret-runtime` input context).
- [x] Add window-scoped snapshot for the mode (`InputContext`).
- [x] Provide a focused-surface override via `TextInputRegion` (stack-based override remains optional).
- [ ] Ensure `TextInput`, `TextArea`, `SelectableText` consult the mode for:
  - word move/select commands,
  - double-click selection,
  - triple-click line selection.
- [x] Define test cases for Unicode and identifier modes.

### 3) Windowed editor surface (ADR 0190/0193)

- [x] Choose v1 surface implementation strategy:
  - paint-driven windowed surface (stable tree, `Scroll` + `Canvas`), or
  - VirtualList-based rows (only if composability is required early).
- [x] Define row cache keys and budgets (viewport-bounded caches; shared text cache configurable).
- [x] Define selection/caret painting layers (paint-only where possible).
- [x] Define IME preedit rendering strategy (inline preedit + caret anchoring).
- [x] Add a “scroll stability / no stale paint” torture harness entry (reuse ui-gallery patterns).

### 4) Document model (buffer) and undo hooks (ADR 0193 / ADR 0136)

- [ ] Select v1 text buffer structure:
  - rope, piece table, or hybrid (document decision).
- [ ] Define edit operation vocabulary (insert/delete/replace) in UTF-8 byte indices.
- [ ] Define transaction boundaries (begin/update/commit/cancel) compatible with `fret-undo`.
- [ ] Define document identity (URI-like) and multi-document story for workspace shells.

### 5) Syntax and highlighting (ADR 0193)

- [ ] Define semantic token schema (independent of theme colors).
- [ ] Define incremental update strategy (best-effort; visible-window prioritized).
- [ ] Materialize spans only for visible rows; keep theme mapping paint-only.

### 6) Semantics (a11y) and selection state

- [ ] Define semantics role for the editor surface (likely `SemanticsRole::TextField` or a dedicated editor role).
- [ ] Ensure selection and composition ranges follow ADR 0071 rules (display text indices).
- [ ] Decide whether to expose visible-row-only semantics or a stub/viewport role for v1 (document the tradeoff).

### 7) Diagnostics and perf attribution

- [ ] Add bundle-friendly counters:
  - visible rows, overscan, cache hits/misses, text blob churn, glyph atlas pressure.
- [ ] Ensure windowed surface window telemetry is exported in diagnostics snapshots (align with ADR 0190).

---

## Risks / Open Questions

1) **Web IME variability**
   - Browsers differ in `beforeinput`/composition behavior; mobile is especially inconsistent.
   - Mitigation: keep bridge observability strong; default to textarea; proxy mode to avoid O(n) DOM updates.

2) **Policy drift**
   - Word boundaries and selection rules must be centralized; otherwise each editor-like component will diverge.

3) **Prepaint and retained host implementation gaps**
   - If ADR 0182/0192 remain unimplemented for too long, editor window changes may cause cache-root rerenders.
   - Mitigation: start with paint-driven windowed surfaces (stable tree) and adopt retained hosts only when needed.

---

## Commands (developer convenience)

- Format: `cargo fmt`
- Tests (preferred): `cargo nextest run`
- Targeted tests (examples):
  - `cargo nextest run -p fret-ui` (focus/scroll/semantics regressions)
  - `cargo nextest run -p fret-render` (text cache/atlas conformance)
