# ADR 0045: Text Geometry Queries (Hit Testing and Caret Metrics)

Status: Accepted

## Context

Editor-grade UI needs precise and consistent “text geometry queries” for:

- mouse caret placement and drag selection,
- selection painting,
- IME candidate window placement (cursor area),
- long-term code editor widgets (multi-line layout, wrapping, scrolling).

If the framework does not define this boundary early, each text widget tends to invent its own:

- “measure prefixes” loops,
- ad-hoc hit testing,
- inconsistent caret/selection semantics,
- IME cursor positioning bugs.

References:

- Text boundary (`TextBlobId` + `TextMetrics`): `docs/adr/0006-text-system.md`
- Text pipeline strategy (GPUI-inspired): `docs/adr/0029-text-pipeline-and-atlas-strategy.md`
- Text editing state + command vocabulary: `docs/adr/0044-text-editing-state-and-commands.md`
- Multiline geometry semantics (caret affinity, local coordinates): `docs/adr/0046-multiline-text-layout-and-geometry-queries.md`
- Zed/GPUI text system (design reference):
  - `repo-ref/zed/crates/gpui/src/text_system.rs`

## Decision

### 1) `TextBlobId` must support geometry queries (not only paint)

`TextBlobId` remains the stable handle for text prepared by the text system.

In addition to paint (`SceneOp::Text`), prepared text must expose geometry queries via `TextService`:

- caret position queries (byte index -> x offset),
- hit-testing (x offset -> nearest caret byte index),
- selection geometry (byte range -> rectangles).

Rationale:

- UI event handlers do not have access to shaping internals.
- Geometry should be derived from the same shaped layout used for rendering.

### 2) Coordinate spaces are explicit

All geometry query results use **logical pixels** (not physical pixels).

For v1 (single-line):

- caret X and selection rects are relative to the text origin:
  - x=0 at the start of the line
  - y=0 at the top of the text box
- widgets remain responsible for placing text within their own bounds (padding, baseline offsets, clipping).

### 3) Index representation is byte offsets at UTF-8 char boundaries

This ADR adopts ADR 0044’s index representation:

- `index: usize` is a byte offset into the UTF-8 string,
- indices are clamped to valid UTF-8 char boundaries,
- hit-testing returns such a byte index.

### 4) Performance constraints

Geometry queries must not require rasterizing glyph images.

Implementations are expected to compute caret/hit-test data from the shaping/layout results (e.g. `cosmic-text`
layout glyph clusters), and cache them on the blob where appropriate.

## API Shape (Core Contract)

`fret-core::TextService` provides default geometry query hooks.

Single-line primitives (usable by any widget; also used as building blocks for multiline):

- `caret_x(blob, index) -> Px`
- `hit_test_x(blob, x) -> usize`
- `selection_rects(blob, range, out)`
- `caret_stops(blob, out)` (escape hatch for UI event handlers)

Multiline-safe primitives (the long-term contract; see ADR 0046):

- `caret_rect(blob, index, affinity) -> Rect`
- `hit_test_point(blob, point) -> HitTestResult { index, affinity }`

Notes:

- For single-line blobs, implementations may ignore `affinity` and treat `point.y` as 0.
- For multiline blobs, `affinity` disambiguates caret placement at line breaks.

## Consequences

- Text widgets can implement mouse caret placement and selection without prefix-measure loops.
- IME cursor-area placement can track the actual caret position.
- Future code editor widgets can build on the same geometry contract.

## Implementation Notes (Current Workspace)

- Renderer provides geometry queries by caching caret stops on prepared blobs:
  - `crates/fret-render-wgpu/src/text.rs`
  - `crates/fret-render-wgpu/src/renderer/mod.rs`
- TextInput uses caret-stop tables for event hit-testing and uses `caret_x` for painting/cursor area:
  - `crates/fret-ui/src/text_input/mod.rs`
- Multiline text input experiments (wrapping/scrolling + geometry queries):
  - `crates/fret-ui/src/text_area/mod.rs`
