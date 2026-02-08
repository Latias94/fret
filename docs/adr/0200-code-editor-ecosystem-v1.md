# ADR 0200: Code Editor Ecosystem v1 (Buffer/View/Surface Contracts)

- Status: Proposed
- Date: 2026-01-27

## Context

Fret targets “editor-grade UI” and already locks several hard-to-change runtime contracts:

- Text system boundary (`TextBlobId` + `TextMetrics` + geometry queries) (ADR 0006 / ADR 0045 / ADR 0046).
- IME + committed text input split and candidate-window positioning feedback (`ImeEvent`, `Effect::ImeSetCursorArea`) (ADR 0012 / ADR 0071).
- Stable text editing command vocabulary (`text.*`) and UTF-8 byte index representation (ADR 0044).
- View caching + dirty view gating (`ViewCache`, `notify`) and multi-stream recording direction (`prepaint`) (ADR 1152 / ADR 0180 / ADR 0182).
- “Prepaint-windowed virtual surfaces” for scroll/viewport-driven content (ADR 0190), and the longer-term “retained windowed hosts” direction (ADR 0192).
- Attributed spans (shaping vs paint split) for rich text inputs that must wrap and remain cache-friendly under theme changes (ADR 0157 / ADR 0161).

The ecosystem currently contains `fret-code-view` (read-only syntax-highlighted blocks) and `fret-markdown`, which validate:

- syntax highlighting can be expressed as spans on top of the text system,
- large scrollable text surfaces benefit from windowing patterns (e.g. “windowed rows surface”).

What is missing is a **code editor ecosystem library** that can scale to large documents and editor workflows without forcing a rewrite of `fret-ui` internals later.

This ADR defines a v1 ecosystem architecture and the contract seams that keep:

- platform/IME integration in the runtime layer,
- editor policy (keymaps, LSP behaviors, UX defaults) in the ecosystem layer,
- and performance explainable via cache roots + windowed surfaces.

## Goals

1) Provide a reusable, editor-grade code editor surface for Fret apps (desktop first; wasm/mobile considered by contract).
2) Keep runtime contracts stable: reuse existing IME/text/geometry/command boundaries.
3) Make large documents feasible via virtualization/windowing and bounded caches.
4) Make theme-only changes paint-only (no reshaping explosions) by default.
5) Keep “policy-heavy” behaviors out of `crates/fret-ui` (ADR 0066).

## Non-goals (v1)

- Collaborative editing / CRDT / OT.
- Full VSCode-class feature parity.
- Guaranteed pixel-identical text layout across platforms beyond existing text system contracts.

## Decision

### 1) Introduce a 3-layer ecosystem split: buffer, view, UI surface

We introduce (names are normative unless later revised):

1. `ecosystem/fret-code-editor-buffer`
   - Text document model (large-document friendly).
   - Edit operations (insert/delete/replace) expressed in UTF-8 byte indices (ADR 0044).
   - Selection/cursor primitives (single cursor v1; multi-cursor as a follow-up).
   - Undo integration hooks (ADR 0136; app-owned policy).
   - Stable document identity for multi-document workflows.

Document identity contract (v1):

- The buffer MUST have a stable, opaque `DocId` used as the primary identity for caching and
  cross-layer coordination.
- The buffer MAY have an optional, URI-like `DocUri` intended for workspace shells and external
  integrations (e.g. LSP, “open recent”, file-backed documents).
  - `DocUri` is treated as an opaque string by the editor ecosystem crates.
  - Normalization and scheme decisions are owned by the workspace layer.
  - Changing a document’s `DocUri` is metadata-only and MUST NOT affect the buffer’s text
    revision.

2. `ecosystem/fret-code-editor-view`
   - A “display map” layer that maps buffer content into **display rows** and coordinate spaces:
     - buffer byte indices ↔ (display row, column) ↔ viewport pixels.
   - Responsibilities (v1 scope):
     - newline boundaries and row iteration
     - tab expansion policy (visual columns) (optional in v1)
     - soft wrap (can be deferred to v2; see rollout)
     - fold regions and placeholders (deferred unless required by MVP)
     - inlays / inline injected text (deferred)
     - block rows (diagnostics blocks) (deferred)
   - Incremental invalidation input: apply buffer edits and produce “invalidated row ranges” for caches.

3. `ecosystem/fret-code-editor` (UI surface crate)
   - Fret UI integration (rendering + input + IME + commands).
   - Implements the editor as a **windowed virtual surface** (ADR 0190):
     - stable element tree shape,
     - scroll-driven window updates handled in `prepaint`/paint without rerendering the whole cache root.
   - Paint model:
     - text layer (visible line window),
     - selection highlight layer,
     - caret + IME preedit underline layer,
     - decoration/inlay overlays (future).

This split mirrors proven decompositions (Zed/GPUI, Monaco) as outcomes, but does not copy implementations (see License Notes).

### 2) Performance baseline: windowed surface first, composable rows later

v1 performance baseline for the editor surface:

- Prefer a **windowed rows surface** style integration (`Scroll` + leaf `Canvas`) for the initial editor:
  - scroll-only deltas are transform-only where possible,
  - per-frame work is bounded by visible row count + overscan,
  - caches are keyed by stable row identity (e.g. `(doc_revision, row_index, wrap_width_bucket)`).
- Reserve the path to “composable rows” via ADR 0192 (retained windowed surface hosts) when needed for:
  - per-row semantics/focus,
  - embedded widgets inside the text flow,
  - complex gutter/chrome composition.

This is intentionally aligned with Fret’s existing perf strategy:

- dirty views + cache roots gate re-execution (ADR 0180 / ADR 1152),
- window membership changes are an “ephemeral prepaint update” (ADR 0190),
- interaction/semantics ranges must remain correct under cache reuse (ADR 0182).

### 3) Text rendering contract: line-local `TextBlobId`s, no monolithic document blob

The editor MUST NOT prepare a single `TextBlobId` for the entire document.

Instead:

- Each visible display row is prepared as its own `TextInput` (plain or attributed) with constraints.
- The editor caches prepared row layouts (`TextBlobId` + metrics) behind a bounded LRU keyed by:
  - document revision,
  - row identity (display row id),
  - font stack key,
  - wrap width bucket (if wrap is enabled),
  - paint/shaping fingerprints split per ADR 0157/0161 (theme-only changes should avoid reshaping).

This aligns with ADR 0046 (“large-document editors are a separate layer (virtualization)”) and the renderer’s cache boundary direction (ADR 0158).

### 4) Syntax highlighting: store semantic tokens, materialize spans only for visible rows

The editor’s syntax layer (initially within `fret-code-editor-view`, later extractable) uses a two-step model:

1) Maintain language parsing/highlighting state incrementally (tree-sitter or similar), producing:
   - per-row token ranges with **semantic highlight ids** (e.g. `"keyword"`, `"string"`) rather than concrete colors.
2) During rendering, convert tokens to `TextSpan` paint styles for the visible rows only:
   - mapping highlight ids → theme tokens happens late (paint-only), so theme changes avoid reshaping.

`ecosystem/fret-syntax` can remain the query/registry source, but the editor is expected to evolve from “whole document highlight” to incremental, visible-window-driven updates.

### 5) Input/IME integration: reuse runtime contracts, editor owns text layout + caret mapping

The editor surface integrates with the existing runtime IME model:

- consume `Event::Ime` and `Event::TextInput` (ADR 0012),
- render inline preedit (ADR 0012 / ADR 0071),
- emit `Effect::ImeSetCursorArea` based on caret rect in window logical coordinates (ADR 0012),
- maintain UTF-8 byte index invariants (ADR 0044 / ADR 0071).

Important scope boundary:

- `crates/fret-ui` owns platform plumbing and hard-to-change IME arbitration rules.
- `fret-code-editor` owns editor-specific caret mapping and “which row is the caret on” logic.

### 6) Commands + undo routing

The editor surface participates in the existing command ecosystem:

- Baseline editing uses `text.*` commands (ADR 0044) where semantics match.
- Editor-specific commands use a separate namespace (recommended: `editor.*`) to avoid expanding `text.*` beyond its intended baseline.

Undo/redo:

- `fret-code-editor-buffer` exposes hooks to record edit transactions (begin/update/commit/cancel).
- Apps own undo history policy and routing (ADR 0136); the default reusable history can be `ecosystem/fret-undo`.

## Rollout Plan (recommended)

### Phase 0 (MVP): “Editable text surface, fixed rows, no wrap”

- Rope/piece-table-backed buffer + byte-index selection.
- Fixed line height, no soft wrap, vertical scrolling + overscan.
- Single caret + mouse selection + clipboard copy/paste (basic).
- IME inline preedit + cursor area positioning (desktop).
- Optional: simple syntax highlighting on visible rows only (best-effort).

### Phase 1: “Incremental highlight + editor chrome”

- Incremental highlight state (per edit) with visible-window materialization.
- Gutter (line numbers), current line highlight, bracket match (optional).
- Better word boundary navigation (Unicode segmentation).

### Phase 2: “DisplayMap grows up (wrap/fold/inlay)”

- Soft wrap with stable coordinate mapping.
- Fold regions + placeholder rendering.
- Inlays (e.g. type hints) as injected display fragments.

### Phase 3: “Composable rows / embedded widgets (if required)”

- Adopt ADR 0192 retained windowed hosts for composable per-row subtrees (when needed).

## M0 Review Checklist (Non-Normative)

The workstream blocks on explicitly confirming these v1 decisions:

1) Layering: the normative split is buffer (`fret-code-editor-buffer`) → view (`fret-code-editor-view`)
   → UI surface (`fret-code-editor`), and editor policy remains ecosystem-owned (ADR 0066).
2) Document identity:
   - `DocId` is the primary, stable identity used for caching/cross-layer coordination.
   - `DocUri` is optional metadata for workspace shells; it is treated as opaque by the editor crates.
   - Changing `DocUri` MUST NOT affect the text revision.
3) Performance baseline:
   - windowed virtual surface first (ADR 0190),
   - no monolithic document `TextBlobId` (row-local shaping/caching only),
   - bounded caches keyed by stable row identity and revision.
4) Input/IME: reuse the runtime contracts (`Event::Ime` / `Event::TextInput`, ADR 0012/0071) and keep
   the web IME bridge runner-owned (ADR 0195).
5) Commands: baseline editing uses `text.*` (ADR 0044); editor-only behaviors live under `editor.*`.

## Evidence anchors (implementation)

- Ecosystem split (buffer/view/surface): `ecosystem/fret-code-editor-buffer/src/lib.rs`, `ecosystem/fret-code-editor-view/src/lib.rs`, `ecosystem/fret-code-editor/src/lib.rs`.
- Windowed surface + per-row text shaping/caching (no monolithic document blob): `ecosystem/fret-code-editor/src/editor/mod.rs` (`CodeEditor::into_element`), `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`paint_row`, `cached_row_text`, `cached_row_syntax_spans`, `materialize_row_rich_text`).
- Harness + regression tests: `apps/fret-ui-gallery/src/spec.rs` (`PAGE_CODE_EDITOR_MVP`, `PAGE_CODE_EDITOR_TORTURE`), `apps/fret-ui-gallery/src/ui.rs` (`preview_code_editor_mvp`, `preview_code_editor_torture`), `ecosystem/fret-code-editor/src/editor/tests/mod.rs`.
- Downstream validation (Markdown Editor v0, source mode): `apps/fret-ui-gallery/src/spec.rs` (`PAGE_MARKDOWN_EDITOR_SOURCE`), `apps/fret-ui-gallery/src/ui.rs` (`preview_markdown_editor_source`), `tools/diag-scripts/ui-gallery-markdown-editor-source-*.json`, `apps/fretboard/src/diag/stats.rs` (markdown editor gates).
- Soft-wrap regression gates (diag scripts + checks): `tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-editing-baseline.json`, `tools/diag-scripts/ui-gallery-code-editor-torture-soft-wrap-geom-fallback-baseline.json`, `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_geom_fallbacks_low`, `check_bundle_for_ui_gallery_code_editor_torture_marker_*`).
- Fold placeholder baseline (unwrapped only; wrap+fold semantics deferred): `ecosystem/fret-code-editor-view/src/folds.rs` (`apply_fold_spans`), `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range` fold materialization), `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_folds_placeholder_present`), `tools/diag-scripts/ui-gallery-code-editor-torture-folds-placeholder-baseline.json`.
- Inlay baseline (unwrapped only; wrap+inlay semantics deferred): `ecosystem/fret-code-editor-view/src/inlays.rs` (`apply_inlay_spans`), `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range` decoration materialization), `apps/fretboard/src/diag/stats.rs` (`check_bundle_for_ui_gallery_code_editor_torture_inlays_present`), `tools/diag-scripts/ui-gallery-code-editor-torture-inlays-baseline.json`.

## License Notes

Zed is licensed under copyleft terms (AGPL/GPL files are present in `repo-ref/zed`), and key editor crates declare GPL-compatible licenses in their manifests. This ADR treats Zed/GPUI only as a behavioral/architectural reference. Implementations in Fret must be original and must not copy Zed source code.

Monaco Editor is MIT-licensed; its “model/view split” and piece-tree concepts are used here as design references, not as code to be directly ported.

## Upstream Reference Anchors (Non-Normative)

These file paths exist solely to help readers map concepts to concrete upstream structures. They are not normative
and must not be treated as “copy this implementation”.

### Zed / GPUI (architecture reference only; do not copy code)

- Display-map layering overview: `repo-ref/zed/crates/editor/src/display_map.rs`
- Editor module decomposition (integration points): `repo-ref/zed/crates/editor/src/editor.rs`
- Rope and large-buffer concerns: `repo-ref/zed/crates/rope/src/rope.rs`
- Text buffer + history/transactions (conceptual reference): `repo-ref/zed/crates/text/src/text.rs`

### Monaco Editor (MIT; concept reference)

- Text model aggregation (events, decorations, undo integration): `repo-ref/monaco-editor/node_modules/monaco-editor-core/esm/vs/editor/common/model/textModel.js`
- Piece-tree text buffer: `repo-ref/monaco-editor/node_modules/monaco-editor-core/esm/vs/editor/common/model/pieceTreeTextBuffer/pieceTreeTextBuffer.js`
- Piece-tree base and line-start metadata: `repo-ref/monaco-editor/node_modules/monaco-editor-core/esm/vs/editor/common/model/pieceTreeTextBuffer/pieceTreeBase.js`

## References

- Runtime layering and “policy stays in ecosystem”: ADR 0066.
- Text boundary and geometry queries: ADR 0006 / ADR 0045 / ADR 0046.
- IME model: ADR 0012 / ADR 0071.
- Text command vocabulary: ADR 0044.
- Attributed spans: ADR 0157 / ADR 0161.
- Cache roots / dirty views / prepaint streams: ADR 1152 / ADR 0180 / ADR 0182.
- Windowed virtual surfaces: ADR 0190 / ADR 0192.
