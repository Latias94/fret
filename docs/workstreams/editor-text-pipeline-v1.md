# Workstream: Editor Text Pipeline v1 (Rope + Rows + Parley Integration)

Status: Draft (design). This workstream focuses on editor-grade text surfaces, not general UI
labels.

Related workstreams:

- Text v2 (Parley) tracker: `docs/workstreams/text-system-v2-parley.md`
- Line breaking improvements: `docs/workstreams/text-line-breaking-v1.md`
- Code editor ecosystem baseline: `docs/workstreams/code-editor-ecosystem-v1.md`

## Problem Statement

Parley answers: “How should this shaped text be placed?”

An editor needs additional infrastructure:

- a high-performance mutable buffer (rope / sum tree),
- stable indexing semantics (UTF-8 bytes internally; UTF-16 at platform boundaries),
- virtualization (only shape/paint visible rows),
- syntax highlighting spans (tree-sitter), and
- wrap policy tuned for code and long tokens.

Fret already has a rope-based buffer:

- `ecosystem/fret-code-editor-buffer/src/lib.rs` (uses `ropey::Rope`)

The remaining gap is the integration contract between:

- `TextBuffer` / “display rows” (editor view model), and
- the renderer text system (`TextSystem::prepare_*`) that wants `&str` / `Arc<str>`.

Without a careful integration, the editor will regress into:

- large per-frame allocations (`to_string()` for big slices),
- O(n) work per edit on large documents,
- misaligned wrap logic between view-model and renderer (cursor drift / selection drift),
- unstable glyph caching due to per-row churn.

## Goals

1) Define a stable layering boundary:
   - mechanism in `crates/` (renderer text),
   - editor policies in `ecosystem/` (buffer/view/wrap policy).
2) Make editor shaping/painting:
   - incremental with respect to buffer edits,
   - bounded to the visible viewport (plus a small prefetch window),
   - stable under resize jitter.
3) Keep index semantics coherent:
   - UTF-8 bytes for internal state and renderer queries,
   - UTF-16 code units for platform IME / accessibility interop.
4) Provide explicit regression gates:
   - unit tests for mapping,
   - diag scripts for real interaction flows (optional).

## Non-goals (v1)

- A full “Zed-class” sum-tree (SumTree) replacement for ropey.
- Multi-document collaborative editing.
- Full semantic tokenization beyond tree-sitter highlighting spans.

## Current State (Evidence)

- Rope buffer exists:
  - `ecosystem/fret-code-editor-buffer/src/lib.rs`
- Code editor uses it:
  - `ecosystem/fret-code-editor/src/editor/mod.rs`
- Display rows + row-local text materialization exists:
  - `ecosystem/fret-code-editor-view/src/lib.rs` (`DisplayMap`, `materialize_display_row_text`)
- Editor paint path already caches visible row text as `Arc<str>` (LRU) keyed by:
  - buffer revision,
  - display row index,
  - wrap cols,
  - fold/inlay epochs:
  - `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range`)
- Renderer text system expects prepared blobs per string:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::prepare`, `prepare_attributed`)
- Current UI text wrap is renderer-owned and heuristic:
  - `crates/fret-render-wgpu/src/text/wrapper.rs`

## Proposed Architecture

### 1) Editor view model owns “display rows”

Editor surfaces should own:

- the mapping from buffer coordinates → display coordinates,
- wrapping decisions for code (policy),
- row fragmentation and caching keyed by `(buffer_revision, viewport, wrap_policy)`.

The renderer should remain responsible for:

- shaping + metrics for a provided string + spans + constraints,
- caret/selection/hit-test geometry for the prepared blob.

### Current call chain (evidence)

- `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`paint_row`)
  - `cached_row_text_with_range` → `DisplayMap::materialize_display_row_text`
  - `CanvasPainter::text_with_blob` / `CanvasPainter::rich_text_with_blob`
- Renderer entry points:
  - `crates/fret-render-wgpu/src/text/mod.rs` (`TextSystem::prepare`, `prepare_attributed`)

### 2) Row text cache: `Arc<str>` per visible row

Because Parley shaping consumes `&str` (and the renderer uses `Arc<str>` internally for cache keys),
we should standardize on:

- materializing **only visible rows** as `Arc<str>` (not the full document),
- caching those `Arc<str>` by:
  - `TextBuffer::revision()`,
  - display row index,
  - and (if needed) a small “window id” to handle partial updates.

This avoids repeated allocations and keeps text blob keys stable across frames.

### 3) Syntax highlighting spans are computed per row

Tree-sitter produces ranges over the underlying buffer.

The editor pipeline should:

- compute highlight spans for each display row (byte ranges relative to the row string),
- emit `AttributedText { text: Arc<str>, spans: Vec<TextSpan> }` to the renderer.

Important:

- keep paint-only changes (colors) out of shaping keys (per the v2 contract),
- keep shaping-affecting changes (font/axes/features) explicit and cache-keyed.

### 4) Wrap policy: distinct “code wrap” vs “UI wrap”

General UI text:

- `TextWrap::Word` with good Unicode break opportunities.

Code editor:

- a policy that prefers breaks at:
  - punctuation boundaries (`/`, `.`, `::`, `->`, `_`),
  - camelCase transitions,
  - but still supports emergency grapheme breaks for long tokens.

This policy should live in `ecosystem/` and be reflected in the display-row segmentation, not
implicitly assumed by the renderer wrapper.

Renderer still needs a high-quality wrap implementation (for generic UI), but editor-grade wrapping
should be driven from the editor view model to keep cursor movement and visual segmentation aligned.

## Regression Gates

Unit tests:

- mapping: buffer byte ↔ display point ↔ row-local byte
- wrapping stability under resize jitter
- highlight span stability across edits (no off-by-one at UTF-8 boundaries)
- platform text input interop (UTF-16 over composed view):
  - `TextInputRegion` should answer `PlatformTextInputQuery` deterministically from its
    `a11y_value`/ranges (surrogate pairs, clamping inside scalars).

Optional diag scripts:

- type, delete, IME compose, and ensure caret/selection geometry stays aligned.

## Milestones (High-Level)

- M0: Document the boundary + add row text cache plan (no behavior changes).
- M1: Harden row text caching and lock allocation regressions behind tests.
- M2: Integrate per-row attributed spans for syntax highlighting without reshaping on paint-only changes.
- M3: Wrap policy separation (code wrap driven by editor view model).
- M4: Platform text input interop surface (UTF-16 over composed view).

For detailed milestone checklists and task breakdown:

- `docs/workstreams/editor-text-pipeline-v1-milestones.md`
- `docs/workstreams/editor-text-pipeline-v1-todo.md`

## Next refactor direction (staging)

The current v1 seam for platform text input is `TextInputRegion`:

- The editor publishes a composed-window string via `TextInputRegionProps.a11y_value`.
- Selection/composition are expressed as UTF-8 byte offsets within that value (ADR 0071).
- The runtime/platform bridge expects UTF-16 code unit indices over the same composed view
  (`PlatformTextInputQuery` / `WindowTextInputSnapshot`).

Staging plan:

1) Lock down UTF-8↔UTF-16 conversion semantics for `TextInputRegion` (queries + snapshot), without
   attempting geometry (`BoundsForRange`) or editing (`replace_*`) in the mechanism layer.
2) Keep the editor in charge of mapping between buffer bytes and the composed display window
   (folds/inlays/preedit), then publish the composed view + ranges as data-only props.
3) Later, if needed, introduce a richer ecosystem-owned adapter that can answer bounds/replace
   queries using cached row geometry, while keeping `fret-ui` as a mechanism-only router.
