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
- [x] Add a focused integration test in `fret-ui` that:
  - lays out a vstack containing a multi-line text node and a following sibling
  - asserts the sibling’s y-position is >= the first node’s painted height (no overlap)
  - Evidence: `crates/fret-ui/src/declarative/tests/layout.rs` (`text_measurement_and_paint_agree_on_wrap_width_in_a_column`)

### TLI1-M2 — `Fill` semantics clarity (w-full vs flex-1)

Decision needed:

- [x] Decide whether we adopt Option A (component-layer `flex_1` helpers) or change runtime semantics.
  - Chosen: Option A (`w_full`/`h_full` remain percent sizing; use `flex_1` + `min_w_0` for flex fill).

If Option A:

- [x] Add `LayoutRefinement` helpers in `ecosystem/fret-ui-kit`:
  - `flex_1()` (grow=1, shrink=1, basis=0px)
  - `min_w_0()` (min_width=0)
- [x] Audit UI Gallery pages and replace intent-mismatched `w_full()` usage:
  - layout preview “Left/Center/Right (fill)” now uses `flex_1().min_w_0()` for equal columns
  - `w_full()` remains reserved for percent sizing (100% of containing block)
- [x] Add a short note to `docs/workstreams/text-layout-integration-v1.md` documenting the chosen policy.

### TLI1-M3 — Wrap policies for editor text

Exit criteria:

- [x] Decide the long-token wrap policy direction.
  - Chosen: add an “anywhere/grapheme-break” wrap mode for editor surfaces (CJK/paths/URLs/code).
- [x] Add a new wrap policy for long tokens (e.g. `Anywhere` / grapheme-based) or document a
  component-layer strategy that handles:
  - CJK
  - file paths / URLs
  - code identifiers
  - Evidence: `crates/fret-core/src/text.rs` (`TextWrap::Grapheme`),
    `crates/fret-render/src/text_v2/wrapper.rs` (`wrap_grapheme*`).
- [x] Add 2–3 unit tests in `fret-render` wrapper:
  - long token without spaces
  - CJK string
  - mixed emoji + CJK
  - Evidence: `crates/fret-render/src/text_v2/wrapper.rs` (tests:
    `grapheme_wrap_breaks_long_token_without_spaces`,
    `grapheme_wrap_handles_cjk_string`,
    `grapheme_wrap_does_not_split_zwj_clusters`).

### TLI1-M3.5 — Grapheme-aware caret/selection

Exit criteria:

- [x] Decide the default caret movement mode for text surfaces:
  - UTF-8 char boundary (current)
  - grapheme cluster (recommended for editor-grade UI)
  - Chosen (v1): grapheme cluster boundaries by default for editable/selectable text surfaces; indices remain UTF-8 bytes.
- [x] Add tests for emoji sequences (ZWJ, flags, keycaps):
  - left/right movement does not split grapheme clusters
  - double click selection does not cut inside clusters
  - Evidence: `crates/fret-ui/src/text_edit.rs` (`grapheme_boundary_tests`).
- [x] Ensure selection ranges are always clamped to valid boundaries for the chosen mode.
  - Evidence: `crates/fret-ui/src/text_edit.rs` (`utf8::{select_word_range, select_line_range}`),
    `crates/fret-ui/src/text_surface.rs`, `crates/fret-ui/src/declarative/host_widget/event/selectable_text.rs`.

### TLI1-M3.6 — IME/platform selection contract (UTF-16)

Reference: Zed/GPUI `UTF16Selection` + input handler surface.

Exit criteria:

- [x] Decide the platform-facing indexing unit (UTF-16 recommended).
  - Chosen: platform-facing = UTF-16 code units; internal = UTF-8 byte offsets (clamped).
  - Sequencing: Windows first; macOS interop follows after the contract is stable.
- [x] Decide the “marked range” coordinate model given v1 widgets render preedit out-of-buffer:
  - base-buffer coordinates (preedit excluded), or
  - composed view coordinates (base + preedit spliced at caret)
  - Chosen (v1): composed view coordinates.
- [x] Provide conversion utilities between:
  - internal indices (UTF-8 byte offsets)
  - UTF-16 code unit indices (with deterministic clamp rules)
  - Evidence: `crates/fret-core/src/utf.rs` (tests), `crates/fret-ui/src/text_edit.rs` (`ime`).
- [x] Publish a window-scoped platform text-input snapshot after paint:
  - `focus_is_text_input`, `selection_utf16`, `marked_utf16`, `ime_cursor_area`
  - Evidence: `crates/fret-runtime/src/window_text_input_snapshot.rs`,
    `crates/fret-ui/src/tree/paint.rs`, `crates/fret-ui/src/tree/tests/window_text_input_snapshot.rs`,
    `crates/fret-launch/src/runner/desktop/app_handler.rs` (runner consumes `ime_cursor_area`).
- [x] Expose a minimal “platform input handler” surface for editable text:
  - selected_text_range
  - marked_text_range
  - text_for_range
  - replace_text_in_range / replace_and_mark_text_in_range
  - bounds_for_range
  - character_index_for_point
  - Evidence: `crates/fret-runtime/src/platform_text_input.rs`,
    `crates/fret-ui/src/tree/mod.rs`, `crates/fret-ui/src/text_input/widget.rs`,
    `crates/fret-ui/src/text_area/widget.rs`, `crates/fret-ui/src/tree/tests/platform_text_input.rs`
  - Note: `replace_and_mark_text_in_range` is implemented in a v1-compatible, caret-anchored mode:
    - `marked` must cover the entire inserted string (`marked == [range.start, range.start + len(text)]` in UTF-16)
    - starting composition requires an empty `range` (`range.start == range.end`)
    - while composing, `range` must match the current `marked_text_range`
    - arbitrary marked subranges are not supported until we unify in-buffer composition.

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

- [x] Decide whether to round key typography scalars (line height / baseline) similarly to Zed/GPUI.
  - Chosen: snap vertical line advances/baselines to device pixels under non-integer scale factors; keep horizontal subpixel.
- [ ] Add a regression test that renders multi-line text under a non-integer scale factor and
  checks for stable metrics/line offsets (no accumulating drift across lines).

## Backlog (issue-shaped TODOs)

- [ ] TLI1-001: Add a debug overlay that draws measured text bounds vs container bounds in UI Gallery.
- [ ] TLI1-002: Audit `TextOverflow` behavior for multiline (line-clamp needs a design).
- [ ] TLI1-003: Audit caching keys: width/wrap/overflow/scale/font-stack must be included in both measure and prepare paths.
- [x] TLI1-004: Decide whether `SelectableText` selection should remain visible when not focused (UX parity vs simplicity).
  - Chosen: keep visible with reduced alpha; must not break range→rect queries.
- [ ] TLI1-005: Add Linux primary selection policy (copy-on-select) behind a feature/setting.
  - Decision (v1): behind a feature/setting; default off.
- [ ] TLI1-006: Add multi-line Up/Down caret movement with “preferred x” behavior.
- [ ] TLI1-007: Add BiDi/RTL conformance strings to UI Gallery and ensure geometry queries match expectations.
- [ ] TLI1-008: Add decoration rendering tests (underline/strikethrough) under non-integer scale factors.
- [ ] TLI1-009: Decide trailing-whitespace selection policy and test it.
