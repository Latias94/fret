# Text Integration v1 (TODO)

Status: Proposed

Narrative plan: `docs/workstreams/text-layout-integration-v1.md`

Legend:

- [ ] pending
- [x] done
- [~] in progress
- [!] blocked / needs decision

## Priority Order (recommended)

P0 (editor-grade correctness):

- TLI1-M2: `Fill` semantics clarity (`w_full` vs `flex_1`) + gallery fixes
- TLI1-M3.6: IME/platform selection contract (UTF-16) to avoid corrupted composition edits
- TLI1-M3.5: Grapheme-aware caret movement for emoji safety (at least for left/right)

P1 (UX parity and completeness):

- TLI1-M3: Wrap policies for long tokens (CJK/paths/code)
- TLI1-M4: Pixel snapping policy under non-integer scale factors
- TLI1-006: Multi-line Up/Down caret movement with preferred-x

P2 (polish):

- Primary selection (Linux) policy + settings
- Selection visibility when unfocused

## Milestones

### TLI1-M0 — Repro + diagnostics baseline

Exit criteria:

- [ ] UI Gallery repro steps are documented (native + web).
- [ ] A debug capture can report, for a selected subtree:
  - measured `TextConstraints` (max_width, wrap, overflow, scale_factor)
  - painted `TextConstraints` (same fields)
  - computed layout bounds for the relevant nodes

### TLI1-M1 — Measurement/paint consistency hardening

Exit criteria:

- [x] `MinContent + TextWrap::Word` measurement does not assume infinite-width single-line text.
  - Evidence: `crates/fret-ui/src/declarative/host_widget/measure.rs` (helper
    `text_max_width_for_constraints`).
- [x] Shaping output clamps `line_height` to at least font extents and respects `TextStyle.line_height`.
  - Evidence: `crates/fret-render/src/text_v2/parley_shaper.rs` (tests:
    `clamps_line_height_to_font_extents`, `respects_explicit_line_height_override`).
- [ ] Add a focused integration test in `fret-ui` that:
  - lays out a vstack containing a multi-line text node and a following sibling
  - asserts the sibling’s y-position is >= the first node’s painted height (no overlap)

### TLI1-M2 — `Fill` semantics clarity (w-full vs flex-1)

Decision needed:

- [ ] Decide whether we adopt Option A (component-layer `flex_1` helpers) or change runtime semantics.

If Option A:

- [ ] Add `LayoutRefinement` helpers in `ecosystem/fret-ui-kit`:
  - `flex_1()` (grow=1, shrink=1, basis=0px)
  - `min_w_0()` (min_width=0)
- [ ] Audit UI Gallery pages and replace intent-mismatched `w_full()` usage:
  - layout preview “Left/Center/Right (fill)” should use `flex_1()` instead of `w_full()` for equal columns
- [ ] Add a short note to `docs/workstreams/text-layout-integration-v1.md` documenting the chosen policy.

### TLI1-M3 — Wrap policies for editor text

Exit criteria:

- [ ] Add a new wrap policy for long tokens (e.g. `Anywhere` / grapheme-based) or document a
  component-layer strategy that handles:
  - CJK
  - file paths / URLs
  - code identifiers
- [ ] Add 2–3 unit tests in `fret-render` wrapper:
  - long token without spaces
  - CJK string
  - mixed emoji + CJK

### TLI1-M3.5 — Grapheme-aware caret/selection

Exit criteria:

- [ ] Decide the default caret movement mode for text surfaces:
  - UTF-8 char boundary (current)
  - grapheme cluster (recommended for editor-grade UI)
- [ ] Add tests for emoji sequences (ZWJ, flags, keycaps):
  - left/right movement does not split grapheme clusters
  - double click selection does not cut inside clusters
- [ ] Ensure selection ranges are always clamped to valid boundaries for the chosen mode.

### TLI1-M3.6 — IME/platform selection contract (UTF-16)

Reference: Zed/GPUI `UTF16Selection` + input handler surface.

Exit criteria:

- [ ] Decide the platform-facing indexing unit (UTF-16 recommended).
- [ ] Provide conversion utilities between:
  - internal indices (UTF-8 bytes or grapheme indices)
  - UTF-16 code unit indices
- [ ] Expose a minimal “platform input handler” surface for editable text:
  - selected_text_range
  - marked_text_range
  - text_for_range
  - replace_text_in_range / replace_and_mark_text_in_range
  - bounds_for_range
  - character_index_for_point

### TLI1-M3.7 — BiDi / RTL correctness baseline

Exit criteria:

- [ ] Add conformance inputs with mixed-direction text (LTR+RTL, numbers, punctuation).
- [ ] Ensure `hit_test_point` and caret rects are stable and cluster-aware in RTL runs.
- [ ] Add unit tests for selection rect generation across direction changes.

### TLI1-M3.8 — Large selection performance and rect coalescing

Exit criteria:

- [ ] Add rect coalescing for selection highlights (merge adjacent rects per line).
- [ ] Add culling: only generate selection rects intersecting the current viewport when possible.
- [ ] Add a micro-benchmark-like test/demo in UI Gallery (or a diagnostic counter) to track rect count.

### TLI1-M4 — Pixel snapping policy

Exit criteria:

- [ ] Decide whether to round key typography scalars (line height / baseline) similarly to Zed/GPUI.
- [ ] Add a regression test that renders multi-line text under a non-integer scale factor and
  checks for stable metrics/line offsets (no accumulating drift across lines).

## Backlog (issue-shaped TODOs)

- [ ] TLI1-001: Add a debug overlay that draws measured text bounds vs container bounds in UI Gallery.
- [ ] TLI1-002: Audit `TextOverflow` behavior for multiline (line-clamp needs a design).
- [ ] TLI1-003: Audit caching keys: width/wrap/overflow/scale/font-stack must be included in both measure and prepare paths.
- [ ] TLI1-004: Decide whether `SelectableText` selection should remain visible when not focused (UX parity vs simplicity).
- [ ] TLI1-005: Add Linux primary selection policy (copy-on-select) behind a feature/setting.
- [ ] TLI1-006: Add multi-line Up/Down caret movement with “preferred x” behavior.
- [ ] TLI1-007: Add BiDi/RTL conformance strings to UI Gallery and ensure geometry queries match expectations.
- [ ] TLI1-008: Add decoration rendering tests (underline/strikethrough) under non-integer scale factors.
- [ ] TLI1-009: Decide trailing-whitespace selection policy and test it.
