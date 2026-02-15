# Workstream: Editor Text Pipeline v1 (Rope + Rows + Parley Integration)

Status: Active (design + partial implementation). This workstream focuses on editor-grade text
surfaces, not general UI labels.

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
- UI text wrap is renderer-owned and Parley-driven for `TextWrap::Word`:
  - `crates/fret-render-wgpu/src/text/wrapper.rs`
  - `crates/fret-render-wgpu/src/text/parley_shaper.rs`

## Capability Snapshot (2026-02-15)

This is a **non-normative** status dashboard for editor/self-drawn UI consumers. The authoritative
contracts are ADR-driven; use this section to keep “what works today” and “what is still missing”
easy to audit.

### Mechanism layer (renderer + UI bridge)

| Area | Capability | Status | Evidence / Notes |
| --- | --- | --- | --- |
| Shaping engine | Parley shaping + metrics | Supported | `crates/fret-render-wgpu/src/text/parley_shaper.rs` |
| OpenType features | `calt`/`liga`/`ssXX` etc via `TextShapingStyle.features` | Supported (best-effort) | Unknown tags are ignored by the resolved face; keep it deterministic via tests. |
| Variable axes | `wght`/`wdth` etc via `TextShapingStyle.axes` | Supported (best-effort) | Same “best-effort” contract as features. |
| Rich text paint | `fg`/`bg`/underline/strikethrough spans | Supported | `crates/fret-core/src/text/mod.rs` (`TextSpan`, `TextPaintStyle`) |
| Wrap (UI) | `TextWrap::Word` with paragraph line breaking | Supported + gated | `docs/workstreams/text-line-breaking-v1.md` + fixtures |
| Wrap (editor) | Editor-owned row segmentation (policy) | Supported (ecosystem) | Renderer wrap should be `TextWrap::None` per display row. |
| Overflow | `TextOverflow::Ellipsis` (single-line) | Supported | Deterministic mapping is gated in text tests. |
| Geometry queries | caret/hit-test/selection rects across wrap + RTL | Supported + gated | `crates/fret-render-wgpu/src/text/mod.rs` tests cover wrapped RTL/mixed scripts. |
| Font system | family overrides + fallback injection + invalidation key | Supported + gated | `TextFontFamilyConfig`, `TextFontStackKey` invalidation; see font workstreams. |
| Perf/diag guard | resize jitter catastrophic regression gate for word wrap | Supported | `tools/diag-scripts/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json` + `tools/perf/diag_text_wrap_resize_jitter_smoke_gate.py` |

### Ecosystem layer (editor pipeline)

| Area | Capability | Status | Evidence / Notes |
| --- | --- | --- | --- |
| Buffer | rope-backed editor buffer (`ropey::Rope`) | Supported | `ecosystem/fret-code-editor-buffer/src/lib.rs` |
| Display rows | `DisplayMap` + row-local text materialization | Supported (baseline) | `ecosystem/fret-code-editor-view/src/lib.rs` |
| Row text caching | visible-row `Arc<str>` cache keyed by revision + row + wrap | Supported (baseline) | `ecosystem/fret-code-editor/src/editor/paint/mod.rs` (`cached_row_text_with_range`) |
| Code wrap policy | presets + knobs, ecosystem-owned | Supported (baseline) | `ecosystem/fret-code-editor-view/src/code_wrap_policy.rs` |
| Platform text input | composed-view UTF-16 query + snapshot via `TextInputRegion` | Supported (baseline) | `TextInputRegionProps.a11y_value` + UTF-8 ranges → UTF-16 answers |
| Bounds/hit-test queries | `BoundsForRange` / `CharacterIndexForPoint` via hooks | Supported (opt-in) | Install `TextInputRegionActionHooks.on_platform_text_input_query` |
| Replace-by-range edits | `replace_*_utf16` via hooks | Supported (opt-in) | Install `TextInputRegionActionHooks` replace handlers; keep limitations explicit. |

### Key gaps (what to build next)

1) **Row-local attributed spans** for syntax highlighting (tree-sitter → per-row spans), with
   regression gates that paint-only changes do not trigger reshaping.
2) **Mapping contracts** for editor surfaces:
   buffer byte ↔ composed a11y window byte ↔ row-local byte ↔ geometry, under folds/inlays/preedit.
3) **Allocation/perf gates** for large documents:
   prevent per-frame `to_string()` churn, ensure shaping/raster work is bounded to visible rows, and
   keep resize jitter stable under width oscillations.
4) **IME-quality gates** for editor composition flows:
   cursor area (`ime_cursor_area`) stability, surrogate pairs, and deterministic UTF-8↔UTF-16 clamps.
5) **Calibrated perf baselines** (optional after the catastrophic smoke gate):
   record stable per-scenario budgets with attribution bundles.

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

#### Recommended “code wrap policy” surface (ecosystem-owned)

To keep the system engineering-friendly, the policy surface should:

- be a pure, deterministic function from `(row_text, wrap_width, policy)` → `row_breaks`,
- be auditable via fixtures (inputs + expected breakpoints),
- provide both:
  - presets for common editor behavior, and
  - a small set of knobs for apps that need tuning.

Suggested presets (names are illustrative; keep them stable once adopted):

- `Conservative`: prefer whitespace and obvious punctuation; avoid surprising mid-identifier breaks.
- `Balanced`: add identifier boundaries (snake/camel/digit transitions) and path/URL separators.
- `Aggressive`: prefer more break opportunities, while still preserving grapheme/cluster safety.

Suggested knobs (keep the list short; avoid “tweak fatigue”):

- Path/URL separators: allow breaks after `/`, `\\`, `?`, `&`, `#`, `=`.
- Punctuation runs: allow breaks after `.`, `,`, `:`, `;` (avoid starting a line with a forbidden
  closing punctuation where possible).
- Identifier boundaries:
  - snake `_`,
  - camelCase transitions (lower→upper, letter↔digit).
- Emergency behavior: when no preferred breakpoint fits, fall back to grapheme-safe breaks (matching
  the editor baseline for long tokens and CJK).

Proposed Rust surface (v1, ecosystem-owned):

```rust
pub enum CodeWrapPreset { Conservative, Balanced, Aggressive }

pub struct CodeWrapKnobs {
    pub break_after_path_separators: bool,
    pub break_after_url_separators: bool,
    pub break_after_punctuation: bool,
    pub break_at_identifier_boundaries: bool,
    pub break_around_operators: bool,
}

pub struct CodeWrapPolicy { pub preset: CodeWrapPreset, pub knobs: CodeWrapKnobs }

pub struct CodeWrapRowStart { pub byte: usize, pub col: usize }

// Deterministic: returns row starts for `text`.
// - `byte` is a UTF-8 byte index (char boundary) for slicing.
// - `col` is a Unicode-scalar column index for display-map bookkeeping.
// Outputs must be grapheme-safe (never split inside a grapheme cluster).
pub fn row_starts_for_code_wrap(text: &str, wrap_cols: usize, policy: CodeWrapPolicy) -> Vec<CodeWrapRowStart>;
```

Important: do not push code-specific wrap heuristics into the renderer wrapper. The renderer-owned
`TextWrap::Word` should remain a general UI facility; editor-grade wrap policy should be expressed
via display-row segmentation so caret/selection semantics cannot drift.

Implementation note:

- Once the editor has segmented display rows, each row should be shaped/painted with
  `TextWrap::None` (renderer wrap disabled). Otherwise the renderer may re-wrap and reintroduce
  cursor/selection drift.

## Regression Gates

Unit tests:

- mapping: buffer byte ↔ display point ↔ row-local byte
- wrapping stability under resize jitter
- highlight span stability across edits (no off-by-one at UTF-8 boundaries)
- font invalidation:
  - `TextFontStackKey` changes must not allow stale row-geometry to answer platform queries
    (bounds/hit-test) for a focused editor surface.
- platform text input interop (UTF-16 over composed view):
  - `TextInputRegion` should answer `PlatformTextInputQuery` deterministically from its
    `a11y_value`/ranges (surrogate pairs, clamping inside scalars).

Optional diag scripts:

- type, delete, IME compose, and ensure caret/selection geometry stays aligned.
- `tools/diag-scripts/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json`: gates
  catastrophic wrap regressions under window resize jitter (word wrap baseline).
- `tools/diag-scripts/ui-gallery-code-editor-ime-cursor-area.json`: focuses the code editor gate
  and asserts `WindowTextInputSnapshot.ime_cursor_area` is present and within window bounds.
- `tools/diag-scripts/ui-gallery-web-ime-harness-ime-cursor-area.json`: focuses the harness region
  and asserts `WindowTextInputSnapshot.ime_cursor_area` is present and within window bounds.

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

### Ecosystem adapter notes (Bounds/Hit-test)

`fret-ui` intentionally does not implement `BoundsForRange` / `CharacterIndexForPoint` for
`TextInputRegion` by default. Instead, editor-grade surfaces can opt into these queries by
installing an ecosystem-owned handler via:

- `TextInputRegionActionHooks.on_platform_text_input_query`

The code editor uses this hook to answer:

- `PlatformTextInputQuery::BoundsForRange` (best-effort caret rect at the range end), and
- `PlatformTextInputQuery::CharacterIndexForPoint` (hit-test using cached row geometry + fallbacks),

by mapping:

- UTF-16 query indices → UTF-8 byte offsets within `a11y_value`,
- `a11y_value` byte offsets → buffer byte offsets in the current display window,
- buffer byte offsets → caret rect / pointer hit-test results.

This keeps the mechanism layer routing-only while still allowing editor-grade IME/candidate window
positioning and pointer hit-testing to converge on the same geometry/cache contracts.

### Ecosystem adapter notes (Replace-by-range)

For platform text input clients that apply edits by requesting replacement (ADR 0261), editor-grade
surfaces can also opt into:

- `platform_text_input_replace_text_in_range_utf16`
- `platform_text_input_replace_and_mark_text_in_range_utf16`

via `TextInputRegionActionHooks` replace handlers.

The code editor implements a best-effort v1 surface:

- `replace_text_in_range_utf16` applies a buffer edit after mapping the UTF-16 composed-view range
   into the current a11y window and then into buffer byte indices.
- `replace_and_mark_text_in_range_utf16` is supported for caret-only composition (`range` empty),
   updating the editor preedit state without mutating the base buffer with the composing string.

In addition, when a composing operation specifies a non-empty range (selection replacement), the
editor applies a best-effort behavior:

- it represents the replacement purely in the composed view (semantics value + range mapping), and
- continues to treat the composing text itself as preedit-only (not inserted into the base buffer
  until commit).

Staging note:

- Selection-replacing preedit is represented in the platform-facing composed window via
  `CodeEditorState.preedit_replace_range`, and is also reflected in the display-row composition via
  `InlinePreedit { anchor, replace_range, text }` so shaping/paint can converge with platform
  queries during composition.
- Known gap (staging): replacement ranges that span newlines are currently clamped to the anchor
  logical line in the view display map. Keep it behind tests and revisit if multi-line composition
  becomes a required input mode.
