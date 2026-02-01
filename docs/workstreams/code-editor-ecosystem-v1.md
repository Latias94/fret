# Code Editor Ecosystem v1 — Refactor Plan & TODO Tracker

Status: Active (workstream document; normative contracts live in ADRs)
Last updated: 2026-02-01

Recent changes (2026-02-01):

- Web: avoid wasm panics from unsupported `std::time` usage; prefer `fret_core::time` / wasm-capable time sources.
- Buffer: update `TextBuffer` line index incrementally on edits (avoid full rescans); add newline-boundary tests.
- Web IME: add a debug snapshot surface + UI Gallery harness to observe textarea bridge state/counters.
- Web IME: add a cursor-area debug overlay and include mount/DPR fields in the debug snapshot for faster triage.
- Web IME: add textarea DOM metrics (client/scroll size + selectionStart/End) to debug snapshot to debug candidate UI jitter.
- Web IME: widen the hidden textarea and align cursor-area to integer pixels to reduce candidate UI jitter.
- View: add a minimal display map that supports column-based soft wrap (byte ↔ wrapped row/col mapping + tests).
- Web IME: improve hidden textarea styling to reduce IME activation flakiness.
- Web IME: prevent preedit wrapping in the hidden textarea to reduce candidate UI vertical jitter.
- Web IME: track hidden textarea bridges per `AppWindowId` (no longer a global singleton).
- Web IME: mount the hidden textarea into a per-canvas overlay layer (wrapper + overlay; no longer appended directly to `document.body`).
- Web IME: treat the mount element as either wrapper or overlay; do not clobber overlay positioning when applying “mount-owned” styles.
- A11y: promote `TextInputRegion` to `SemanticsRole::TextField` and allow publishing value/selection/composition ranges (ADR 0071).
- A11y: wire `SetTextSelection` into the code editor via `TextInputRegion` (best-effort, windowed value).
- Web: enable a default CJK demo font bundle to avoid “tofu” squares in IME/editor harnesses.
- Desktop: update Windows taskbar visibility wiring for winit 0.31 platform attributes.

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

## Next Up (recommended priority)

P0 (correctness and contracts):

- Web IME: stabilize caret anchoring and reduce candidate UI jitter across browsers (textarea style + cursor-area mapping).
- Web IME: harden the mount strategy for future multi-canvas/docking (per-canvas wrapper/overlay exists; next is a true per-window overlay registry).
- Selection + composition range invariants: expand ADR 0071 coverage across TextInput/TextArea/CodeEditor (including a11y selection actions).
- Web: ensure editor surfaces have a robust default CJK fallback (avoid tofu when using monospace stacks).

P1 (robustness and testability):

- Expand word-boundary + click-selection tests across widgets and scroll/transform cases.
- Add diagnostics counters + snapshots for windowed surfaces and caches (align with ADR 0190).

P2 (features):

- Improve the incremental syntax strategy (edits → visible-window invalidation) and document the tradeoffs.
- Start a display-map growth spike (wrap → fold → inlay) while keeping buffer↔display↔pixel mapping stable.

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
- triple-click selects a logical line, including the trailing newline when present

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

### M4 — Buffer model + undo hooks

Exit criteria:

- A v1 text buffer structure is selected (rope / piece table / hybrid) with stated tradeoffs.
- Edit ops remain UTF-8 byte indexed and transaction hooks remain compatible with ADR 0136.
- Document identity (URI-like) is locked for multi-document workflows (workspace shells).

### M5 — Incremental highlighting (visible-window materialization)

Exit criteria:

- Syntax state updates incrementally on edits (best-effort).
- Only visible rows are materialized into spans for paint.
- Theme changes do not trigger reshaping; only paint changes.

### M6 — Semantics (a11y) + selection/composition invariants

Exit criteria:

- The editor surface exports a stable semantics role and a documented projection strategy for windowed content.
- Selection and IME composition ranges follow ADR 0071 rules (display text indices) consistently.

### M7 — Diagnostics and perf attribution

Exit criteria:

- Bundle-friendly counters exist for windowed surfaces and editor caches (hits/misses/churn/pressure).
- Diagnostics snapshots expose window telemetry for windowed surfaces (ADR 0190 alignment).

### M8 — Display map growth (wrap/fold/inlay) (optional for v1)

Exit criteria:

- Soft wrap is supported with stable coordinate mapping (buffer ↔ display ↔ pixels).
- Fold regions and placeholders (if adopted) do not break caret/selection semantics.
- Inlays (if adopted) are represented without mutating the underlying buffer.

### M9 — Composable rows / retained host (only if needed)

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
- [x] Review ADR 0195 and confirm web strategy:
  - hidden textarea bridge,
  - `beforeinput` + `composition*` translation rules,
  - proxy mode (no full document mirroring).

### 1) Web runner IME bridge (ADR 0195)

- [~] Define DOM element strategy: textarea creation, attach layer, z-order and isolation (global element today; per-window attachment TBD).
- [x] Define focus lifecycle and mapping to `Effect::ImeAllow`.
- [x] Define caret anchoring mapping to `Effect::ImeSetCursorArea` (best-effort, mobile-leaning).
- [x] Define event translation and suppression rules:
  - `beforeinput` → `TextInput` (filtered),
  - `composition*` → `ImeEvent`,
  - command-dispatch suppression to avoid double insert.
- [x] Implement UTF-16 ↔ UTF-8 conversion utility with deterministic clamping.
- [x] Add debug-only counters/logging for bridge behavior (snapshot published as a global for harness views).
- [x] Add a web harness page (or demo mode) dedicated to IME conformance.

Evidence anchors:

- `crates/fret-platform-web/src/wasm.rs` (`WebImeBridge`, `WebPlatformServices::handle_effects`)
- `crates/fret-core/src/input.rs` (`WebImeBridgeDebugSnapshot`)
- `crates/fret-core/src/utf.rs` (`utf16_range_to_utf8_byte_range`)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_WEB_IME_HARNESS`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_web_ime_harness`)

### 2) Word boundaries seam (ADR 0194)

- [x] Add/standardize `TextBoundaryMode` definition (in `fret-runtime` input context).
- [x] Add window-scoped snapshot for the mode (`InputContext`).
- [x] Provide an override stack service (push/pop token) for overlays and focused surfaces.
- [x] Allow focused text input regions to override `TextBoundaryMode` (mechanism-only).
- [x] Allow code-editor-grade surfaces to select the mode explicitly (policy input), and expose a UI Gallery toggle.
- [x] Ensure `TextInput`, `TextArea`, `SelectableText` consult the mode for:
  - word move/select commands,
  - double-click selection,
  - triple-click line selection.
- [~] Define test cases for Unicode and identifier modes (seed tests added; expand coverage).

Evidence anchors:

- `crates/fret-runtime/src/input.rs` (`InputContext.text_boundary_mode`, `TextBoundaryMode`)
- `crates/fret-runtime/src/window_text_boundary_mode.rs` (`WindowTextBoundaryModeService`)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps.text_boundary_mode_override`)
- `crates/fret-ui/src/declarative/mount.rs` (mounts focused override into the runtime tree)
- `crates/fret-ui/src/tree/dispatch.rs` / `crates/fret-ui/src/tree/paint.rs` (publishes focused override in `InputContext`)
- `crates/fret-ui/src/text_edit.rs` (Unicode/identifier segmentation + tests)
- `crates/fret-ui/src/text_input/widget.rs` / `crates/fret-ui/src/text_area/widget.rs` / `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs` (integration)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::set_text_boundary_mode`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`, `preview_code_editor_torture` boundary mode toggle)

### 3) Windowed editor surface (ADR 0190/0193)

- [x] Choose v1 surface implementation strategy:
  - paint-driven windowed surface (stable tree, `Scroll` + `Canvas`), or
  - VirtualList-based rows (only if composability is required early).
- [x] Implement a minimal editor surface vertical slice (fixed-height rows, no wrap):
  - per-row text paint via windowed surface,
  - caret + selection (mouse + keyboard),
  - clipboard copy/paste (best-effort),
  - IME preedit (inline underline) + cursor-area reporting (best-effort).
- [~] Define row cache keys and budgets (text blobs + shaping caches + token spans).
- [~] Define selection/caret painting layers (paint-only where possible).
- [x] Implement inline IME preedit rendering (underline + optional range highlight).
- [x] Ensure `ImeSetCursorArea` caret rect accounts for preedit cursor (best-effort).
- [x] Cancel inline preedit deterministically on selection/navigation actions (v1 policy).
- [x] Add a UI Gallery page for the editor MVP (manual interaction harness).
- [x] Add a “scroll stability / no stale paint” torture harness entry (reuse ui-gallery patterns).

Evidence anchors:

- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditor` row painting + input + per-canvas text cache policy)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps`, `ElementKind::TextInputRegion`)
- `crates/fret-ui/src/declarative/host_widget/event/text_input_region.rs` (IME/TextInput forwarding)
- `ecosystem/fret-ui-kit/src/declarative/windowed_rows_surface.rs` (pointer up/cancel support)
- `ecosystem/fret-code-editor-view/src/lib.rs` (baseline buffer↔display mapping)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_MVP`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_TORTURE`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_torture`)
- `ecosystem/fret-code-editor/src/lib.rs` (`caret_rect_for_selection` preedit cursor offset)
- `ecosystem/fret-code-editor/src/lib.rs` (`paint_row`, `materialize_preedit_rich_text` underline)

### 4) Document model (buffer) and undo hooks (ADR 0193 / ADR 0136)

- [~] Select v1 text buffer structure:
  - rope, piece table, or hybrid (document decision).
- [x] Define edit operation vocabulary (insert/delete/replace) in UTF-8 byte indices.
- [x] Define transaction boundaries (begin/update/commit/cancel) compatible with `fret-undo`.
- [~] Define document identity (URI-like) and multi-document story for workspace shells.

Evidence anchors:

- `ecosystem/fret-code-editor-buffer/src/lib.rs` (`TextBuffer`, `Edit`, `TextBufferTransaction`, `TextBufferTx`, `apply_in_transaction`, `rollback_transaction`)
- `ecosystem/fret-code-editor-buffer/src/lib.rs` (`DocId`, `DocUri`, `TextBuffer::uri`, `TextBuffer::set_uri`)
- `ecosystem/fret-code-editor/src/lib.rs` (`UndoGroupKind`, `UndoGroup`, `apply_and_record_edit`, `UndoHistory::record_or_coalesce`)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::replace_buffer`, `CodeEditorHandle::set_text`)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::set_language`, `cached_row_syntax_spans`, `materialize_row_rich_text`)
- `ecosystem/fret-code-editor/Cargo.toml` (`syntax` / `syntax-rust` / `syntax-all`)
- `apps/fret-ui-gallery/Cargo.toml` (native enables `fret-code-editor` `syntax-rust`)
- `apps/fret-ui-gallery/src/driver.rs` (`code_editor_syntax_rust` model)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`, `preview_code_editor_torture` syntax toggle)

### 5) Syntax and highlighting (ADR 0193)

- [x] Define semantic token schema (independent of theme colors).
- [~] Define incremental update strategy (best-effort; visible-window prioritized).
- [x] Materialize spans only for visible rows; keep theme mapping paint-only.
- [x] Expose a UI Gallery toggle for manual validation.

### 6) Semantics (a11y) and selection state

- [~] Define semantics role for the editor surface (currently: `TextInputRegion` emits `SemanticsRole::TextField`).
- [~] Ensure selection and composition ranges follow ADR 0071 rules (partial: `TextInputRegion` can publish UTF-8 ranges within an app-provided value).
- [x] Decide whether to expose visible-row-only semantics or a stub/viewport role for v1.
  - v1 decision: **stub/viewport semantics**.
  - We expose:
    - one `TextField` node (the `TextInputRegion`) whose `value` is a **windowed** buffer slice around the caret,
      plus selection/composition ranges within that value (ADR 0071).
    - one `Viewport` node for the scrollable windowed surface (no per-row semantics nodes).
  - Tradeoff: this is not full-document accessible text. It is, however, stable and performant for very large
    documents and keeps the semantics tree bounded and deterministic while the editor virtualization story evolves.

### 7) Diagnostics and perf attribution

- [ ] Add bundle-friendly counters:
  - visible rows, overscan, cache hits/misses, text blob churn, glyph atlas pressure.
- [ ] Ensure windowed surface window telemetry is exported in diagnostics snapshots (align with ADR 0190).

### 8) Display map expansion (wrap/fold/inlay) (optional v1 → v2)

- [ ] Soft wrap with stable coordinate mapping (buffer ↔ display ↔ pixels).
- [ ] Fold regions + placeholders without breaking caret/selection.
- [ ] Inlays (injected display fragments) without mutating the underlying buffer.

### 9) Retained host / composable rows (only if required)

- [ ] Decide whether we need composable per-row subtrees (embedded widgets, rich gutters).
- [ ] If yes, adopt the retained host direction (ADR 0192) so window boundary crossings do not force parent rerenders.

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
