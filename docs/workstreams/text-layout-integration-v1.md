# Text Integration v1 — Layout, Selection, and Platform Interop

Status: Proposed (tracking UI gallery regressions; complements `docs/workstreams/text-system-v2-parley.md`).

This document focuses on **integration hazards** between:

- the text system (shaping, wrapping, metrics, baseline, glyph placement), and
- the layout system (flex/grid/constraints, intrinsic sizing, percent/fill semantics, clipping),
- the interaction layer (caret/selection, hit-testing, copy/paste),
- platform interop (IME, clipboard/primary selection, accessibility),

as seen in editor-grade UI surfaces (tables, form layouts, dockable panels, sidebars).

Normative contracts remain in ADRs; this is an implementation workstream.

## Problem Statement

UI Gallery currently exhibits several issues that are typical for a text+layout integration layer:

- text wraps unexpectedly and paints outside the background of its container
- sibling widgets overlap (parent height too small for painted multiline/line box)
- short labels can “break” in surprising ways under `Fill`/intrinsic measurement paths

These are not “component bugs”; they are usually caused by **measurement vs paint divergence** or
**ambiguous `Fill` semantics under flex/grid/intrinsic sizing**.

## Repro Cases (UI Gallery)

### A) Layout page: “Left (fill)” breaks into two lines and the second line paints outside its background

File: `apps/fret-ui-gallery/src/ui.rs` (`preview_layout`)

- Expected: three equal-width cards; label fits on one line within the colored box.
- Observed: `"Left (fill)"` becomes two lines (`"Left"` + `"(fill)"`) and the second line can paint
  outside the box background.

Hypothesis: `LayoutRefinement::w_full()` currently maps to `Length::Fill -> percent(100%)` in the
layout engine (`crates/fret-ui/src/declarative/taffy_layout.rs`), which relies on a **definite
containing block**. During intrinsic sizing / wrapper probing passes, percent resolution can
degenerate (effectively treating the available width as 0), which forces word wrapping.

### B) Forms page: checkbox/switch rows overlap with the “Tip:” text

File: `apps/fret-ui-gallery/src/ui.rs` (`preview_forms`)

- Expected: a vertical stack: input, textarea, toggles, then a tip line below.
- Observed: tip text overlaps the toggles (parent height underestimates painted content).

Hypothesis: text line boxes were previously allowed to be smaller than font extents, causing glyphs
to paint outside the measured bounds, and/or the measurement path used a different wrap width than
paint.

## Invariants We Want (Editor-Grade Baseline)

### I1 — Measurement and paint must agree on wrapping inputs

For any text element in the declarative UI runtime:

- the wrap width used to compute `TextMetrics.size.height` must match the wrap width used to prepare
  the `TextBlob` used for painting
- otherwise, parents will reserve insufficient height and siblings will overlap

### I2 — Line height must never underflow font extents

For any shaped line:

- `line_height_px >= ascent_px + |descent_px|`
- if `TextStyle.line_height` is provided, it must participate in shaping/layout results (not just
  paint)

### I3 — `Fill` must have unambiguous semantics in flex/grid

We need to clearly separate:

- “fill remaining space as a flex item” (Tailwind `flex-1`)
- “percent(100%) of containing block” (Tailwind `w-full`)

Treating `Fill` as `percent(1.0)` is convenient, but fragile in intrinsic sizing paths and wrapper
chains where the containing block may not be definite yet.

**Policy (v1):**

- `LayoutRefinement::w_full()` / `h_full()` means **percent sizing** (`100%` of the containing block).
  Use this when you actually want “match the parent box” and you know an ancestor provides a
  definite size.
- For “share remaining space in a flex row/column”, prefer `LayoutRefinement::flex_1()` (Tailwind
  `flex-1`). For text-heavy children, also apply `LayoutRefinement::min_w_0()` to allow shrinking
  instead of overflowing (web-like `min-width: 0`).

### I4 — Selection indices must be well-defined across layers (UTF-8 / UTF-16 / graphemes)

We need explicit rules for which unit each layer uses:

- renderer/layout: typically UTF-8 byte indices (cheap slicing) as long as we clamp to char boundaries
- IME/platform: often UTF-16 code units (macOS NSTextInputClient; Windows IME conventions)
- user expectations: grapheme clusters for caret movement (emoji ZWJ sequences, combining marks)

If we blur these, we get:

- caret moves “into” emoji sequences
- IME replacement ranges corrupt text
- selection ranges can be out of bounds or not round-trip across APIs

**Policy (v1):**

- **Internal text indices** (UI state, renderer geometry queries, selection/caret): UTF-8 **byte
  offsets**, clamped to valid UTF-8 char boundaries (and later: grapheme boundaries for movement).
- **Platform-facing indices** (IME, accessibility bridges, OS input handlers): UTF-16 **code unit**
  offsets and ranges.
- Provide lossless conversion utilities at the platform boundary (`utf8_bytes <-> utf16_units`),
  with explicit clamping rules for non-representable boundaries (surrogate pairs, ZWJ sequences,
  combining marks).

**Implementation sequencing (v1):**

- Windows first (desktop MVP); macOS interop follows once the contract surface is stable.

### I5 — Hit-testing + geometry queries must be stable under transforms

Caret rects, selection rects, and `character_index_for_point` must remain correct under:

- scrolling (content transforms)
- render transforms (AABB vs visual bounds)
- non-integer scale factors

## Recent Fixes Landed

These changes improve I1/I2 and should be treated as “minimum correctness constraints”:

- **Min-content text measurement** now treats `TextWrap::Word` as breakable under `MinContent` by
  using `max_width = Some(Px(0.0))` to avoid single-line intrinsic measurements drifting from later
  definite-width paint. Evidence: `crates/fret-ui/src/declarative/host_widget/measure.rs`.
- **Line height stabilization**: Parley shaping output clamps `line_height` to at least
  `ascent + |descent|` and respects `TextStyle.line_height`. Evidence:
  `crates/fret-render/src/text_v2/parley_shaper.rs` (tests included).

## Decision Snapshot (v1)

This section records “hard-to-change” integration policies for v1. ADRs should be added when we
stabilize the API surface that external code depends on.

- **Platform input indexing**: UTF-16 code units (Windows-first; macOS follows once stable).
- **Internal indexing**: UTF-8 byte offsets (clamped to char boundaries; movement can clamp further
  to grapheme boundaries).
- **UTF-8 ↔ UTF-16 conversions**:
  - conversions must be lossless for representable boundaries (round-trip stability)
  - for non-representable UTF-16 boundaries (middle of a surrogate pair), clamp:
    - range start: to the previous representable boundary
    - range end: to the next representable boundary
- **IME geometry** (`bounds_for_range`):
  - empty range: caret rect at the insertion point
  - non-empty range: bounding rect of the visual selection rects for that range
- **Selection geometry perf**: selection highlight rect generation should support viewport culling
  when the caller knows the visible region (e.g. scrollable text areas). Implementation hook:
  `TextService::selection_rects_clipped`.
- **Preedit representation**:
  - current widgets keep IME preedit text **out of the base buffer** and render it inline
  - any future platform-facing “marked range” queries must define whether ranges are expressed in:
    - base buffer coordinates (preedit excluded), or
    - a composed view (base + preedit spliced at caret)
  - Chosen (v1): platform-facing “marked range” coordinates use the **composed view** model. Widgets
    must provide deterministic mapping between base-buffer indices and composed-view indices.
- **Platform replace+mark** (`replace_and_mark_text_in_range`):
  - v1 supports a single caret-anchored marked range only (no partial marking)
  - constraints: `marked == [range.start, range.start + len(text)]` (UTF-16) and, while composing,
    `range` must equal the current `marked_text_range`; starting composition requires an empty
    replace range (`range.start == range.end`)
- **Caret movement mode (editable/selectable surfaces)**: grapheme cluster boundaries by default
  (editor-grade UX); stored indices remain UTF-8 byte offsets.
- **Wrap policy for long tokens**: introduce an “anywhere”/grapheme-break wrap mode (break between
  grapheme clusters) for CJK/paths/URLs/code identifiers. Keep `Word` as the default for general UI
  labels. Implementation: `TextWrap::Grapheme` (Parley wrapper).
- **Pixel snapping under non-integer scale factors**:
  - snap *vertical* line advances/baselines to device pixels to avoid cumulative drift
  - keep *horizontal* placement subpixel (text rendering quality), but ensure caret/selection x is
    derived from the same shaped clusters as paint.
- **Primary selection (Linux)**: supported behind a feature/setting; default off for v1.
- **Selection visibility when unfocused**: keep selection visible with reduced alpha (editor-grade
  expectation); exact policy remains component-layer but must not break range→rect queries.

## Open Failure Modes (Checklist)

This section captures pitfalls that we should explicitly test for.

### F1 — Long unbreakable tokens (CJK, file paths, URLs)

`TextWrap::Word` is not sufficient for:

- continuous CJK text
- `snake_case` / `camelCase` / long code identifiers
- long paths and URLs

We now have an additional wrap policy (`TextWrap::Grapheme`), but still need component-level
recipes and demos for code/path-heavy UI surfaces.

### F2 — “Multiline + Ellipsis” (line-clamp)

If we need “wrap but clamp to N lines with ellipsis”, it must be representable as:

- deterministic truncation rules
- stable caret/hit-test mapping at the truncation boundary

This is a separate feature from single-line ellipsis.

Decision (v1): `TextOverflow::Ellipsis` is single-line only; multiline “line clamp” requires an
explicit contract. See `docs/adr/1160-text-overflow-ellipsis-and-line-clamp-v1.md`.

### F3 — Flex min-size behavior (“min-width: auto” vs “min-width: 0”)

In web flexbox, many overflow bugs come from the default `min-width: auto`. In an editor UI,
common patterns (tabs, table cells, sidebars) often need “min-width: 0” to allow shrinking.

We should decide what Fret’s default should be for text-heavy flex items and provide a clear
mechanism/recipe surface (`min_w_0`, `flex_1`).

### F4 — Scale factors and pixel snapping

When `scale_factor` is not an integer, small rounding differences can accumulate:

- text metrics drift vs glyph raster placement
- baseline alignment mismatches across siblings

Zed/GPUI tends to round key typography scalars to pixels; we should adopt a similarly explicit
policy.

### F5 — Grapheme-aware caret and selection (emoji, combining marks)

Current “move by char boundary” behavior can split:

- emoji ZWJ sequences
- regional indicator flag pairs
- combining marks

We should provide a grapheme-aware cursor mode for editable/selectable surfaces (editor-grade UX).

### F6 — Platform IME selection/marked range indexing (UTF-16)

Zed/GPUI explicitly models `UTF16Selection` for platform-facing input methods.

If Fret’s platform boundary remains UTF-8 bytes, we need lossless conversion utilities and a clear
contract for:

- selected range
- marked (composing/preedit) range
- bounds_for_range
- character_index_for_point

### F7 — Primary selection (Linux) vs clipboard (cross-platform)

Editor-grade UX typically supports:

- `Ctrl+C`/copy: clipboard
- select-with-mouse: primary selection (Linux/Wayland/X11)
- middle click paste: primary selection

This requires a policy decision and a platform capability surface.

### F8 — Multi-line navigation semantics

Up/Down movement typically needs:

- “preferred x” retention (visual column)
- bidi-safe mapping for RTL runs
- consistent behavior under wrapping

This is separate from left/right/word/home/end.

### F9 — BiDi / RTL runs (caret movement, hit-testing, selection rects)

Even if most UI strings are LTR, editor-grade UIs routinely contain:

- Arabic/Hebrew snippets
- mixed-direction file paths
- embedded code + comments

Pitfalls:

- caret movement within RTL runs (visual vs logical order)
- hit-testing x → index must respect cluster directionality
- selection rects must remain stable and not “jump” across runs

We should add conformance inputs with mixed BiDi text and ensure geometry queries remain correct.

### F10 — Ligatures and shaping clusters (caret stops, hit-test granularity)

Some fonts produce ligatures (e.g. “fi”, “tt”) and complex clusters. If caret stops are generated
by naive per-character advances, caret placement will drift or become non-monotonic.

We should ensure our caret stop generation stays cluster-aware and monotonic under shaping.

### F11 — Combining marks and grapheme boundaries (selection painting and copy ranges)

Even if we adopt grapheme-aware caret movement (F5), we must also ensure:

- selection ranges are clamped to safe boundaries
- selection rect painting does not split combining mark clusters visually
- copy ranges preserve the intended grapheme units (no partial sequences)

### F12 — Decorations (underline/strikethrough) participate in layout/paint correctly

Editor UIs rely on decoration for:

- markdown emphasis
- diagnostics squiggles
- search highlights

Pitfalls:

- decoration thickness/offset under non-integer scaling
- decoration drawing outside measured bounds (overlap risk)
- ensuring decoration does not force reshaping (paint-only when possible)

### F13 — Trailing whitespace and newline edge cases

Common edge cases that break selection/caret geometry:

- empty lines
- consecutive newlines
- trailing newline at EOF
- trailing spaces at line end (should they be selectable? should they affect width?)

These should be covered by unit tests for caret/selection rect generation.

### F14 — Very large selections (performance and batching)

Selecting large buffers (thousands of lines) can create:

- huge numbers of selection rects (O(n) allocations)
- large scene command lists (GPU/CPU overhead)

We may need:

- rect coalescing (merge adjacent rects per line)
- “viewport intersection” culling for selection rect generation

### F15 — Accessibility text surfaces (range-to-rect, selection state)

To be editor-grade, we need a coherent story for a11y:

- expose selected ranges
- map ranges to screen bounds
- ensure assistive tech can query text content for a given range

This intersects the IME contract and should share the same indexing conversions.

## Zed/GPUI Reference Checklist

Zed’s GPUI layer is a useful “what exists in a mature editor-grade stack” reference, especially
for platform input/IME integration.

Key reference anchors (pinned in-repo):

- Platform input contract: `repo-ref/zed/crates/gpui/src/platform.rs`
  - `UTF16Selection`
  - `InputHandler::{selected_text_range, marked_text_range, text_for_range, replace_text_in_range, bounds_for_range, character_index_for_point}`
- Entity adapter surface: `repo-ref/zed/crates/gpui/src/input.rs`

What to learn from it (not to copy APIs verbatim):

- make the platform-facing indexing unit explicit (UTF-16)
- treat IME as a first-class contract (selected vs marked ranges)
- geometry queries are required for correct candidate window placement

## Design Options (for `Fill` semantics)

We should avoid mixing “percent” and “flex fill” under a single primitive.

### Option A (recommended): introduce a dedicated flex-item refinement (`flex_1`) in component layer

- Keep `Length::Fill` as percent sizing where it is already used intentionally (like `w-full`).
- Add a component-layer helper that sets `flex.grow=1`, `flex.shrink=1`, `flex.basis=0px` and (when
  necessary) `min_width=0`.
- Migrate gallery examples that intend “equal columns” from `w_full()` to `flex_1()`.

Pros:

- minimal blast radius (no runtime semantic change)
- matches Tailwind semantics (`w-full` vs `flex-1`)

Cons:

- requires auditing call sites (but that is desirable: it makes intent explicit)

### Option B: change `Length::Fill` mapping in the layout engine (context-dependent)

- Map `Fill` to `Dimension::auto` and set flex growth instead (or similar) in flex contexts.

Pros:

- fewer call-site changes

Cons:

- risky: `Fill` is used across multiple container types (grid, positioned layouts)
- hard to make “context-dependent” semantics predictable

## Immediate Next Steps

1. Confirm the two UI Gallery repro cases under:
   - native runner (`cargo run -p fret-ui-gallery`)
   - web runner (`cd apps/fret-ui-gallery-web; ...`)
2. Add a lightweight debug switch to dump:
   - computed layout boxes for the problematic subtree
   - measured text constraints vs painted text constraints (width, wrap, overflow)
3. Decide and implement Option A (`flex_1` + `min_w_0` recipes), then update the gallery pages to
   demonstrate correct intent.

4. Draft a “text interaction baseline” checklist (selection/caret/IME/clipboard) and prioritize
   3–5 items that unblock editor-grade usage first (see TODO tracker).

## Implementation Path (Recommended)

This is a suggested sequence that minimizes churn while improving correctness:

1. **Clarify layout intent** in component layer:
   - add `flex_1()` + `min_w_0()` helpers (ecosystem)
   - migrate gallery examples that intend flex fill away from `w_full()`
2. **Lock indexing contracts**:
   - keep internal indices as UTF-8 byte offsets (clamped to boundaries), but
   - expose a platform-facing UTF-16 selection interface (Zed/GPUI-aligned), with explicit
     conversion utilities
   - implement a data-only window snapshot seam (`WindowTextInputSnapshot`) published after paint
     to support Windows-first IME candidate positioning and future a11y bridges (selection/marked
     ranges in UTF-16 over the composed view). Evidence:
     `crates/fret-runtime/src/window_text_input_snapshot.rs`, `crates/fret-ui/src/tree/paint.rs`,
     `crates/fret-launch/src/runner/desktop/app_handler.rs`
3. **Harden caret/selection behavior**:
   - add grapheme-aware movement mode for emoji safety (at least left/right)
   - add multi-line Up/Down movement with preferred-x
4. **Add regression tests**:
   - integration: no-overlap layout for wrapped text stacks
   - unit: emoji sequences for caret movement + selection
   - conformance: IME replace ranges round-trip without corrupting UTF-8 text
