# Code Editor Ecosystem v1 - Refactor Plan & TODO Tracker

Status: Active (workstream document; normative contracts live in ADRs)
Last updated: 2026-02-09

Recent changes (2026-02-09):

- Diagnostics: add toggle-stability gates for Markdown editor fold/inlay fixtures to ensure UI decoration toggles cannot mutate the buffer revision/length or move the caret (ADR 0200).
- Diagnostics: add a folds clamp-selection regression gate for the Markdown source editor fixture (caret inside a folded span clamps to the fold start once the placeholder is visible; buffer revision/length remain unchanged) (ADR 0200).
- UI Gallery: expose the fold fixture span in `app_snapshot` and add a deterministic “Caret: in fold” fixture control for diag scripts.
- Diagnostics (web): allow `fretboard diag run` to drive UI diagnostics over DevTools WebSocket for wasm runners, and add a minimal IME bridge attach gate + baseline script (ADR 0195).
- Diagnostics: gate “no stale lines” scroll stability by asserting windowed row window changes repaint (scene fingerprint updates when `visible_start` changes).
- Code editor: allow folds/inlays to remain visible under inline preedit when soft wrap is off (unwrapped baseline), and lock it with dedicated diag baselines + gates (ADR 0203 staging).
- Code editor: add an opt-in to allow folds/inlays to remain visible under inline preedit when soft wrap is on (wrapped baseline), and lock it with dedicated diag baselines + gates (ADR 0203 staging).

Recent changes (2026-02-08):

- Code editor interaction: add `CodeEditorInteractionOptions` (editor/read-only/disabled) and gate input/edit/undo/redo so downstream consumers can control interaction policy without pushing it into `crates/fret-ui` (ADR 0066).
- UI Gallery: add “Markdown Editor (Source)” downstream milestone page (source editor + `fret-markdown` preview) with best-effort Markdown syntax highlighting (`syntax-markdown` feature).
- Diagnostics: add scripted repros + fretboard gates for Markdown source editor read-only behavior and soft-wrap toggle stability.

Recent changes (2026-02-07):

- Diagnostics: add a strict (0-allowed) soft-wrap geometry fallback gate for the code editor torture harness, covering pointer hit-testing, caret rects, and vertical caret moves.
- Editor paint: when caret stops are unavailable, prefer renderer-provided `TextService::caret_x` over the monospace `cell_w` heuristic (keeps caret X pixel-aligned with shaped glyph runs).
- Editor perf/correctness: shift the per-row geometry cache across single-line edits in soft-wrap mode (reduces “first paint” geometry churn and avoids unnecessary fallback spikes).
- Display map: start scaffolding fold placeholder materialization in the unwrapped baseline and add a UI Gallery + fretboard diag gate to keep buffer↔display mapping regression-testable.
- Display map: add an unwrapped inlay fixture + gate so “injected display fragments” can evolve without silently breaking caret/selection/hit-test mapping.

Recent changes (2026-02-04):

- Editor paint: anchor caret Y to the actual row text blob baseline (fixes caret drifting above glyphs in mixed-font / rich-span lines).
- Editor selection: clamp windowed row hit-testing during drags, and add timer-driven edge autoscroll while drag-selecting (continues even when the pointer is stationary at the viewport edge).
- Editor caret: when the text backend doesn't provide `caret_rect`, fall back to the blob box for caret top/height so the caret doesn't render from the row top.
- Diagnostics: add a fretboard gate for windowed-rows surfaces to ensure scroll-driven view-cache reuse cannot freeze the visible window.
- Diagnostics: add a dedicated UI Gallery repro script for code editor scroll stability (`ui-gallery-code-editor-torture-scroll-stability.json`).

Recent changes (2026-02-02):

- Branch sync: merge local `main` into `code-editor-ecosystem-v1` to stay aligned with the latest runner/text/diagnostics baselines.
- Web: fix the UI Gallery code editor MVP being “visible but not editable” by binding focus/key/command/pointer hooks to the `TextInputRegion` element id scope (not the outer keyed scope); this unblocks Web/WASM input routing.
- Web IME: emit `Effect::ImeAllow { enabled: true }` during pointer-down focus for editor-grade `TextInputRegion` surfaces so the hidden textarea bridge can be focused within the same user-activation gesture (browser restrictions).
- UiTree: compute `focus_is_text_input` from the declarative element kind (`TextInput` / `TextArea` / `TextInputRegion`) when available, not only from cached host-widget flags (fixes global “IME stays disabled” regressions on web).
- Web runner: flush effect/event turns synchronously on pointer-button events (`WindowEvent::PointerButton`, press/release) so `Effect::ImeAllow` can focus the hidden textarea within the browser user-activation window (prevents “click input but IME never enables”).
- Buffer: move `TextBuffer` to rope-backed storage (`ropey`) while preserving the UTF-8 byte-index contract; adapt view/editor consumers to slice-based APIs.
- Editor geometry: start migrating caret/selection/pointer hit-testing from the monospace "cell width" heuristic to renderer-provided caret stops (per-row cached); add `CanvasPainter::{text_with_blob, rich_text_with_blob}` to support geometry queries.
- Editor geometry: harden caret-stop hit-testing for non-monotonic X runs (e.g. mixed-direction/bidi rows) while keeping the caret-stop path as the v1 event-stage baseline.
- Editor code health: split `fret-code-editor` into `editor/*` submodules (`input`, `paint`, `geom`, `a11y`, `tests`) to reduce merge conflicts and unblock follow-up refactors.
- Editor navigation: preserve a pixel `preferred_x` for caret up/down navigation (via caret stops), not the last display column.
- Editor harness: show caret geometry hints (`preferred_x`, cached caret stops) in the code editor torture overlay for faster mixed-script debugging.

Recent changes (2026-02-03):

- Web IME: anchor the hidden textarea to the caret rect **center** (instead of origin) and adapt textarea line metrics to caret height to reduce candidate/composition UI drift.
- Web IME: add an opt-in browser console logger for focus/cursor-area updates (`?ime_debug=1` or `window.__FRET_IME_DEBUG=true`) to debug cases where the in-app panel isn't visible.
- Scroll/view-cache: add a mechanism switch (`ScrollProps.windowed_paint`) so windowed paint surfaces can force view-cache rerender on scroll-offset changes (prevents “stale lines” when a cached subtree would otherwise replay).
- Windowed surfaces: fix `windowed_rows_surface` to pass row `Rect`s anchored at the canvas bounds origin (not `0,0`), fixing “left side clipped / prefixes missing” in `code_editor_torture` and the windowed-surface torture pages.
- Editor UX: scroll the caret into view after edits/navigation and refresh `last_bounds` from paint so IME cursor-area anchoring can follow keyboard-only flows.
- UI Gallery: make the word-boundary harness render both the fixture and debug output using `CodeEditor` surfaces (avoid backend-dependent multiline text rendering).
- Editor infra: add `CodeEditor::key(u64)` so multiple editors can coexist under the same element-id scope without keyed-id collisions (fixes semantics snapshot cycles in multi-editor UIs).
- UI Gallery: add a “Dump layout…” button that writes a Taffy subtree dump to `.fret/taffy-dumps` for debugging nested scroll/clip/layout issues.

Next up (priority order):

1. Keep the “no stale lines” torture harness gateable:
   - run the dedicated diag script with view-cache enabled and keep the windowed-rows scroll gate green.
2. Editor surface MVP correctness (native first):
   - ensure typing, selection, undo/redo, and caret navigation remain correct under scroll + soft-wrap.
3. Fonts on web:
   - confirm `cjk-lite` default bundle is sufficient for common CJK IME flows; decide whether to enable `emoji-fonts` by default or gate behind a flag (payload tradeoff).
4. Web IME (wasm): **deferred**
   - Known issue: IME enable/focus can still be flaky on some browsers/dev setups (activation-window timing).
   - Keep `?demo=ui_gallery&page=web_ime_harness` as the repro surface and revisit after core editor correctness/stability is locked.

Verification quickstart:

- Web (UI Gallery): `cargo run -p fretboard -- dev web --demo ui_gallery`
  - `http://127.0.0.1:8080/?demo=ui_gallery&page=code_editor_mvp`
  - `http://127.0.0.1:8080/?demo=ui_gallery&page=code_editor_torture`
  - `http://127.0.0.1:8080/?demo=ui_gallery&page=web_ime_harness` (deferred; use for repro only)
- Native (optional): `cargo run -p fretboard -- dev native --bin components_gallery`
- Debug logs: set `FRET_WINDOWED_ROWS_POINTER_DEBUG=1` to print pointer/row mapping for windowed row surfaces.

Recent changes (2026-02-01):

- Branch sync: merge local `main` into `code-editor-ecosystem-v1` to stay aligned with the text/diagnostics baselines.
- Web: avoid wasm panics from unsupported `std::time` usage; prefer `fret_core::time` / wasm-capable time sources.
- Buffer: update `TextBuffer` line index incrementally on edits (avoid full rescans); add newline-boundary tests.
- Web IME: add a debug snapshot surface + UI Gallery harness to observe textarea bridge state/counters.
- Web IME: add a cursor-area debug overlay and include mount/DPR fields in the debug snapshot for faster triage.
- Web IME: add textarea DOM metrics (client/scroll size + selectionStart/End) to debug snapshot to debug candidate UI jitter.
- Web IME: widen the hidden textarea and align cursor-area to integer pixels to reduce candidate UI jitter.
- View: add a minimal display map that supports column-based soft wrap (byte ↔ wrapped row/col mapping + tests).
- View: expose `DisplayMap::display_row_byte_range` to slice buffer text into wrapped display rows (tests included).
- Editor: render wrapped display rows in the windowed surface (selection/caret/preedit/syntax spans operate in display-row space).
- Editor: fix caret up/down movement clamping to display rows when soft wrap is enabled (regression test included).
- View/Editor: reduce per-keystroke overhead by avoiding full display-map rebuilds when wrap is disabled and line count is unchanged.
- Editor: implement Home/End navigation over visual rows under soft wrap (Ctrl+Home/End clamps to document bounds).
- Editor: add word navigation/deletion shortcuts (`Ctrl/Alt+Arrow`, `Ctrl/Alt+Backspace/Delete`) using the shared text-boundary mode.
- Editor: add PageUp/PageDown navigation based on the scroll viewport height (moves caret + scrolls).
- Editor: bubble Ctrl/Meta+PageUp/PageDown to workspace keymaps (do not cancel preedit for these chords).
- Editor: add bundle-friendly cache counters (row text + syntax) and expose them via `CodeEditorHandle::cache_stats()`.
- UI Gallery: show code-editor cache counters (total + per-frame deltas) in the torture overlay.
- UI Gallery: add a word-boundary harness (fixture + char/word stepping + apply caret/selection into the editor).
- UI Kit/Diagnostics: record and export windowed-rows-surface visible-window telemetry in UI diagnostics snapshots.
- UI Kit/Diagnostics: include a best-effort app/harness snapshot blob in UI diagnostics bundles (ui-gallery exports page + code-editor toggles; includes web IME bridge snapshot when available).
- View: expand word-boundary conformance tests (UnicodeWord vs Identifier) for selection and movement; fix `move_word_left` at token boundaries.
- Web IME: improve hidden textarea styling to reduce IME activation flakiness.
- Web IME: prevent preedit wrapping in the hidden textarea to reduce candidate UI vertical jitter.
- Web IME: track hidden textarea bridges per `AppWindowId` (no longer a global singleton).
- Web IME: mount the hidden textarea into a per-canvas overlay layer (wrapper + overlay; no longer appended directly to `document.body`).
- Web IME: treat the mount element as either wrapper or overlay; do not clobber overlay positioning when applying “mount-owned” styles.
- A11y: promote `TextInputRegion` to `SemanticsRole::TextField` and allow publishing value/selection/composition ranges (ADR 0071).
- A11y: wire `SetTextSelection` into the code editor via `TextInputRegion` (best-effort, windowed value).
- Web: enable a default CJK demo font bundle (`cjk-lite` subset) to reduce “tofu” in IME/editor harnesses (still limited; use “Load fonts…” for full coverage).
- UI Gallery: add a code-editor “Load fonts…” action (file dialog → `Effect::TextAddFonts`) and a soft-wrap toggle for wrap-boundary regression checks.
- Desktop: update Windows taskbar visibility wiring for winit 0.31 platform attributes.

This document is an implementation-focused tracker for building an editor-grade **code editor ecosystem** for Fret.
It is intentionally non-authoritative; the normative contracts are:

- `docs/adr/0200-code-editor-ecosystem-v1.md`
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

- Scroll stability: fix the “no stale lines” torture failure (row cache keys, invalidation boundaries, replay correctness).
- Editing baseline: ensure typing/selection/caret/undo remain correct under scroll + soft-wrap.
- Selection + composition range invariants: expand ADR 0071 coverage across TextInput/TextArea/CodeEditor (including a11y selection actions).
- Code editor: keep pointer hit-testing stable on mixed-direction (bidi) rows (caret-stop hit-testing is the event-stage baseline; decide later whether to expose `TextService::hit_test_point` to pointer handlers).
- Web: document and enforce the default font story for editor-grade surfaces (monospace + CJK + emoji).
  - Baseline: `TextFontFamilyConfig` + `common_fallback` seeded via `FontFamilyDefaultsPolicy::FillIfEmptyWithCuratedCandidates`.
  - Web default: ship a small `cjk-lite` subset + optional emoji bundle; accept that glyph coverage is limited and "tofu" is expected outside the subset.
  - Escape hatch: UI Gallery “Load fonts…” (`Effect::TextAddFonts`) for full CJK coverage when validating IME/editor behavior.
  - Diagnostics: surface `TextFontStackKey` + `TextFontFamilyConfig` + `FontCatalog` in the UI Gallery web IME harness panel to make tofu causes debuggable.
  - Runner: load bundled default fonts during web renderer adoption (not via a delayed `TextAddFonts` effect) to reduce “first frame” tofu.
  - Evidence: `crates/fret-runtime/src/font_bootstrap.rs` (curated defaults), `crates/fret-render-wgpu/src/text.rs` (`cjk_fallback_*` tests).

P1 (robustness and testability):

- Expand word-boundary + click-selection tests across widgets and scroll/transform cases.
- Add regression checks around wrap boundaries (caret, selection, preedit, syntax spans) using the existing UI Gallery soft-wrap toggle.

P2 (features):

- Improve the incremental syntax strategy (edits → visible-window invalidation) and document the tradeoffs.
- Wrap: add a pixel-accurate wrapping mode (measure-driven) while preserving stable buffer ↔ display ↔ pixel mapping.
- Grow the display-map surface (wrap → fold → inlay) without breaking caret/selection invariants.
- Diagnostics: add renderer-level churn counters (text blob churn, glyph atlas pressure) to make perf regressions bundle-debuggable.

Deferred (Web/WASM IME):

- Web IME: stabilize enable/focus across browsers (user-activation timing) and reduce candidate UI drift.
- Web IME: harden the mount strategy for future multi-canvas/docking (per-canvas wrapper/overlay exists; next is a true per-window overlay registry).

---

## Downstream Milestone: Markdown Editor v0 (source mode)

This workstream needs a concrete, app-shaped milestone to validate the editor ecosystem surfaces
without prematurely baking “editor policy” into `crates/fret-ui` (ADR 0066). The v0 Markdown editor
is that milestone: it exercises the code editor contracts in a way that is representative of
editor-grade workflows, while keeping the UX surface intentionally small.

### Scope (v0)

- **Source-mode editing** only (no WYSIWYG): Markdown is edited as plain text using `fret-code-editor`.
- **Syntax highlighting**: Markdown + fenced code blocks (best-effort; incremental visible-window strategy).
- **Soft wrap**: stable caret/selection/hit-test mapping under wrap.
- **IME correctness**: native + web bridge seams remain stable (ADR 0195), including cursor-area feedback.
- **Selection/navigation**: word boundaries, double/triple click, and baseline commands (ADR 0194).
- **Interaction control**: surfaces can be configured as:
  - editable,
  - read-only (select/copy/nav, but no mutations),
  - disabled (no focus/IME routing, no selection updates).
- **Optional preview** (nice-to-have): a second panel that renders the current buffer via `fret-markdown`,
  using `fret-code-view` for fenced blocks. The preview is explicitly *not* required for the editor
  contract validation.

### Non-goals (v0)

- WYSIWYG / ProseMirror-class behavior (inline widgets, block reflow semantics, “source maps” to rendered nodes).
- Multi-cursor, multi-selection, or complex edit transforms.
- LSP integration, diagnostics, formatting, or code actions.
- Full Markdown spec parity (tables/footnotes/task lists can be validated later).

### Why this milestone matters

If the code editor surfaces can reliably power a minimal Markdown source editor, then the ecosystem
contracts are likely “good enough” for broader editor-grade use cases (logs, diffs, config editors,
note-taking) without forcing a `fret-ui` rewrite.

### Milestone breakdown (v0)

The downstream milestone is intentionally split into smaller, gateable increments so we can ship
contract confidence without waiting for a full “editor product”.

#### M10.1 — Source editor shell + interaction control

Exit criteria:

- A UI Gallery page (`markdown_editor_source`) exists and is stable enough for diagnostics.
- The editor can be configured as:
  - editable,
  - read-only (select/copy/nav, but no buffer mutations),
  - disabled (no focus/IME routing, no selection updates).

Evidence:

- `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeEditorInteractionOptions`, `CodeEditorState::set_interaction`)
- `ecosystem/fret-code-editor/src/editor/input/mod.rs` (edit/undo/redo gating)
- `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MARKDOWN_EDITOR_SOURCE`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_markdown_editor_source`)
- `apps/fret-ui-gallery/src/ui.rs` (`ui-gallery-markdown-editor-mode-disabled`)
- `tools/diag-scripts/ui-gallery-markdown-editor-source-disabled-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-disabled-inject-preedit-baseline.json`
- `crates/fret-diag/src/stats.rs` (`check_bundle_for_ui_gallery_markdown_editor_source_disabled_blocks_edits`)

#### M10.2 — Soft-wrap + selection/navigation consistency

Exit criteria:

- Soft-wrap can be toggled without destabilizing caret/selection/buffer revision.
- Word boundaries and double-click selection match ADR 0194 baselines in source mode.
- Selection remains stable across wrap boundaries while editing (not only on toggles).

Diagnostics gates (baseline set; add more as needed):

- `tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-toggle-stability-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-inlays-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-word-boundary-double-click-inlays-soft-wrap-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-line-boundary-triple-click-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-soft-wrap-editing-selection-wrap-baseline.json`
- Fold/inlay decoration sanity (ADR 0200; present under wrap; suppressed under inline preedit):
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-placeholder-baseline.json` (captures A/B/C for toggle-stability)
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-clamp-selection-baseline.json`
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-baseline.json`
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-folds-soft-wrap-inline-preedit-baseline.json`
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-baseline.json` (captures A/B/C for toggle-stability)
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-caret-navigation-baseline.json`
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-baseline.json`
  - `tools/diag-scripts/ui-gallery-markdown-editor-source-inlays-soft-wrap-inline-preedit-baseline.json`

#### M10.3 — IME bridge seam validation (native + web)

Exit criteria:

- Native IME:
  - inline preedit renders without breaking buffer↔display mapping,
  - cursor-area feedback is best-effort correct for editor-grade surfaces.
- Web/WASM IME:
  - the runner-owned bridge remains attachable to the focused editor region (ADR 0195),
  - we have at least one non-flaky diagnostics baseline that detects “IME not attached” regressions.

Notes:

- The web IME baseline gate must be stable; if it cannot be made stable, it stays as a manual harness.

Diagnostics gates (baseline set; add more as needed):

- `tools/diag-scripts/ui-gallery-markdown-editor-source-a11y-composition-baseline.json`
- `tools/diag-scripts/ui-gallery-markdown-editor-source-a11y-composition-soft-wrap-baseline.json`
- Web/WASM attach smoke (ADR 0195):
  - `tools/diag-scripts/ui-gallery-web-markdown-editor-source-ime-bridge-attach-baseline.json` (run via DevTools WS transport)
  - Gate: `--check-ui-gallery-web-ime-bridge-enabled`

#### M10.4 — Diag gates as the definition-of-done

Exit criteria:

- The Markdown source editor v0 contract is continuously regression-tested via `fretboard diag` gates.
- Each “hard-to-change” behavior claimed by this milestone has:
  - an ADR reference (normative),
  - a diagnostic script (repro),
  - and a gate check (assertion) in the suite runner.

## Architectural Principles (performance-first, Fret-aligned)

1) **No CSS runtime dependency**
   - Fret remains a non-DOM UI runtime: layout and styling are `LayoutStyle` + theme tokens (ADR 0066).
   - On wasm, a **hidden DOM textarea** may require setting inline styles to function reliably for IME.
     This is a runner implementation detail, not a framework contract (ADR 0195).

2) **Windowed surface first**
   - The editor UI should be a “windowed virtual surface” (ADR 0190): bounded work per frame based on
     viewport + overscan, not document length.

3) **Line-local text blobs**
   - No monolithic `TextBlobId` for an entire document. Prepare and cache text per visible row (ADR 0200).

4) **Theme-only updates should be paint-only**
   - Syntax highlighting should be expressed as semantic tokens and materialized as paint-only spans for visible rows.

5) **Keep runtime mechanism-only**
   - Word boundary rules, code-specific behaviors, and editor defaults belong in ecosystem policy layers (ADR 0066).

---

## Target Crate Layout (v1 direction)

See ADR 0200 for the normative split. This workstream assumes:

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

- ADR 0200/0194/0195 reviewed and accepted or revised with explicit decisions.
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

Current policy note (v1 + current v2 direction):

- While inline IME preedit is active, fold placeholders and inlays are suppressed (do not compose).
- Revisit composition only after preedit is modeled as a first-class injected display fragment and the DisplayMap can
  compose multiple fragment sources under a single, deterministic buffer↔display↔a11y mapping surface.

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

- [x] Review ADR 0200 and confirm crate split and v1 baseline (windowed surface first).
  - See: ADR 0200 “M0 Review Checklist (Non-Normative)”.
- [x] Review ADR 0194 and confirm the preferred seam:
  - window-scoped `InputContext.text_boundary_mode` + override stack.
  - See: ADR 0194 “M0 Review Checklist (Non-Normative)”.
- [x] Review ADR 0195 and confirm web strategy:
  - hidden textarea bridge,
  - `beforeinput` + `composition*` translation rules,
  - proxy mode (no full document mirroring).

### 1) Web runner IME bridge (ADR 0195)

- [x] Define DOM element strategy: per-window textarea creation, attach layer, z-order and isolation (mounted into a per-canvas overlay layer).
- [x] Define focus lifecycle and mapping to `Effect::ImeAllow`.
- [x] Define caret anchoring mapping to `Effect::ImeSetCursorArea` (best-effort, mobile-leaning).
- [x] Define event translation and suppression rules:
  - `beforeinput` → `TextInput` (filtered),
  - `composition*` → `ImeEvent`,
  - command-dispatch suppression to avoid double insert.
- [x] Implement UTF-16 ↔ UTF-8 conversion utility with deterministic clamping.
- [x] Add debug-only counters/logging for bridge behavior (snapshot published as a global for harness views, including a small recent-event ring buffer).
- [x] Add a web harness page (or demo mode) dedicated to IME conformance.
- [!] Deferred: IME enable/focus can still be flaky in some browsers/dev setups (activation-window timing). Keep `?demo=ui_gallery&page=web_ime_harness` as the repro surface and revisit later.

Evidence anchors:

- `crates/fret-platform-web/src/wasm.rs` (`WebImeBridge`, `WebPlatformServices::handle_effects`)
- `crates/fret-platform-web/src/ime_dom_state.rs` (command-path suppression + event ordering state machine)
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
- [x] Define test cases for Unicode and identifier modes (seed tests added; expand coverage).

Evidence anchors:

- `crates/fret-runtime/src/input.rs` (`InputContext.text_boundary_mode`, `TextBoundaryMode`)
- `crates/fret-runtime/src/window_text_boundary_mode.rs` (`WindowTextBoundaryModeService`)
- `crates/fret-ui/src/element.rs` (`TextInputRegionProps.text_boundary_mode_override`)
- `crates/fret-ui/src/declarative/mount.rs` (mounts focused override into the runtime tree)
- `crates/fret-ui/src/tree/dispatch.rs` / `crates/fret-ui/src/tree/paint.rs` (publishes focused override in `InputContext`)
- `crates/fret-text-nav/src/lib.rs` (shared Unicode/identifier boundary algorithms + tests)
- `crates/fret-ui/src/text_edit.rs` (delegates word/line navigation to `fret-text-nav`)
- `crates/fret-ui/src/text_input/widget.rs` / `crates/fret-ui/src/text_area/widget.rs` / `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs` (integration)
- `crates/fret-ui/src/declarative/host_widget.rs` / `crates/fret-ui/src/text_input/bound.rs` / `crates/fret-ui/src/text_area/bound.rs` (platform text input delegation for declarative widgets)
- `crates/fret-ui/src/declarative/tests/interactions.rs` (scroll/transform double-click selection coverage for TextInput/TextArea)
- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditorHandle::set_text_boundary_mode`)
- `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`, `preview_code_editor_torture` boundary mode toggle)

### 3) Windowed editor surface (ADR 0190/0200)

- [x] Choose v1 surface implementation strategy:
  - paint-driven windowed surface (stable tree, `Scroll` + `Canvas`), or
  - VirtualList-based rows (only if composability is required early).
- [x] Implement a minimal editor surface vertical slice (fixed-height rows, no wrap):
- [x] Implement a minimal editor surface vertical slice (fixed-height rows, optional column-based soft wrap):
  - per-row text paint via windowed surface,
  - caret + selection (mouse + keyboard),
  - clipboard copy/paste (best-effort),
  - IME preedit (inline underline) + cursor-area reporting (best-effort).
- [x] Define row cache keys and budgets (text blobs + shaping caches + token spans).
  - Key: `(buffer_revision, display_wrap_cols, display_row_index)`; caches reset on revision or wrap-mode changes.
  - Budget: derived from `viewport_rows + 2*overscan + 128`, clamped to `[256, 8192]`, and applied consistently to:
    - per-row editor-local caches (row text, row geometry, syntax spans),
    - and the `CanvasCachePolicy.text` shaping cache for prepared row blobs.
- [x] Define selection/caret painting layers (paint-only where possible).
  - Layering: text → selection highlights → IME preedit underline/range highlight → caret (plus optional debug overlays).
  - Keep theme-only changes paint-only by expressing selection/preedit as paint decorations on top of prepared row text.
- [x] Implement inline IME preedit rendering (underline + optional range highlight).
- [x] Ensure `ImeSetCursorArea` caret rect accounts for preedit cursor (best-effort).
- [x] Cancel inline preedit deterministically on selection/navigation actions (v1 policy).
- [x] Add a UI Gallery page for the editor MVP (manual interaction harness).
- [x] Add a “scroll stability / no stale paint” torture harness entry (reuse ui-gallery patterns).

Evidence anchors:

- `ecosystem/fret-code-editor/src/lib.rs` (`CodeEditor` row painting + input + per-canvas text cache policy)
- `ecosystem/fret-code-editor/src/editor/mod.rs` (cache budgets derived from viewport + overscan; exported cache stats)
- `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (row text/syntax/geom caches + eviction; paint layer ordering)
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

### 4) Document model (buffer) and undo hooks (ADR 0200 / ADR 0136)

- [x] Select v1 text buffer structure:
  - rope (`ropey`) while preserving the UTF-8 byte-index contract.
- [x] Define edit operation vocabulary (insert/delete/replace) in UTF-8 byte indices.
- [x] Define transaction boundaries (begin/update/commit/cancel) compatible with `fret-undo`.
- [x] Define document identity (URI-like) and multi-document story for workspace shells.

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

### 5) Syntax and highlighting (ADR 0200)

- [x] Define semantic token schema (independent of theme colors).
- [x] Define incremental update strategy (best-effort; visible-window prioritized).
- [x] Materialize spans only for visible rows; keep theme mapping paint-only.
- [x] Expose a UI Gallery toggle for manual validation.

### 6) Semantics (a11y) and selection state

- [x] Define semantics role for the editor surface (v1: `TextInputRegion` emits `SemanticsRole::TextField`, plus a sibling `Viewport` node for the windowed surface).
- [x] Ensure selection and composition ranges follow ADR 0071 rules (`value` is windowed display text; `text_selection`/`text_composition` are UTF-8 byte offsets into that `value`; `SetTextSelection` is mapped best-effort into buffer indices).
- [x] Decide whether to expose visible-row-only semantics or a stub/viewport role for v1.
  - v1 decision: **stub/viewport semantics**.
  - We expose:
    - one `TextField` node (the `TextInputRegion`) whose `value` is a **windowed** buffer slice around the caret,
      plus selection/composition ranges within that value (ADR 0071).
    - one `Viewport` node for the scrollable windowed surface (no per-row semantics nodes).
  - Tradeoff: this is not full-document accessible text. It is, however, stable and performant for very large
    documents and keeps the semantics tree bounded and deterministic while the editor virtualization story evolves.

### 7) Diagnostics and perf attribution

- [x] Add bundle-friendly counters:
  - Done: visible window + overscan (windowed surfaces), editor-local cache hits/misses (row text + syntax), and renderer-level churn counters.
  - Known gaps: tighten “text blob churn + glyph atlas pressure” attribution (likely from renderer/canvas caches).
- [x] Ensure windowed surface window telemetry is exported in diagnostics snapshots (align with ADR 0190).

### 8) Display map expansion (wrap/fold/inlay) (optional v1 → v2)

- [~] Soft wrap with stable coordinate mapping (buffer ↔ display ↔ pixels).
  - Implemented: column-based wrapping + stable byte ↔ display row/col mapping.
  - Known gaps: not pixel-accurate wrapping. Fallbacks still exist when caret stops/metrics are unavailable (e.g. before the first paint), but the torture harness now includes a strict “0 geometry fallbacks after warmup” diag gate (evaluated after the last stats reset) to keep migration regressions observable and actionable.
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
