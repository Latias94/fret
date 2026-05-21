# Text Clean Geometry Stability v1 - Closeout Audit

Date: 2026-05-21
Status: Closed

## Shipped

- Split the text clean-geometry proof out of `scroll-optimization-v1`.
- Recorded the current text eligibility matrix for `Text`, `StyledText`, and `SelectableText`.
- Kept the runtime boundary conservative: only cached `TextWrap::None`, `TextOverflow::Clip`,
  `TextAlign::Start` text with stable height, cached measured size, and unchanged fingerprint may
  propagate clean geometry.
- Added additive diagnostics detail for text clean-geometry rejection sub-causes without changing
  layout behavior.
- Proved the detail field reaches both focused Rust tests and a real UI Gallery diagnostics bundle.

## Verification

Fresh current-worktree gates run on 2026-05-21:

- `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow clean_geometry_small_resize_skips_nowrap_text_width_delta_when_height_stable clean_geometry_small_resize_rejects_nowrap_text_height_delta --no-fail-fast`
  - Result: `3/3` passed.
- `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics clean_geometry_rejection_detail_is_additive --no-fail-fast`
  - Result: `1/1` passed.
- `cargo fmt -p fret-ui -p fret-bootstrap --check`
  - Result: passed.
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/text-wrap/ui-gallery-text-measure-overlay-window-resize-drag-jitter-steady.json --dir target/fret-diag/text-clean-geometry-detail-20260521-r1 --session-auto --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --features gallery-dev`
  - Result: passed; captured the bundle listed below.
- `python -m json.tool docs/workstreams/text-clean-geometry-stability-v1/WORKSTREAM.json`
  - Result: passed.
- `python -m json.tool docs/workstreams/scroll-optimization-v1/WORKSTREAM.json`
  - Result: passed.
- `python -m json.tool docs/workstreams/imui-editor-grade-product-closure-v1/WORKSTREAM.json`
  - Result: passed.
- `python tools/check_workstream_catalog.py`
  - Result: passed; validated 429 dedicated directories and 47 standalone markdown files.
- `python tools/gate_imui_workstream_source.py`
  - Result: passed.
- `git diff --check`
  - Result: passed.

## Evidence Anchors

- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `crates/fret-ui/src/tree/debug/layout.rs`
- `crates/fret-ui/src/declarative/tests/layout/layout_engine.rs`
- `ecosystem/fret-bootstrap/src/ui_diagnostics/layout_paint_hotspot_diagnostics.rs`
- `target/fret-diag/text-clean-geometry-detail-20260521-r1/sessions/1779305958600-170448/1779306373288/bundle.schema2.json`

## Residual Risk

No wrapped-text clean-geometry behavior changed here. Future wrapped-text optimization remains
deliberately deferred until fresh evidence shows it is a material perf owner and a dedicated
line-break/computed-box signature proves unchanged layout, measure, glyph, and paint outcomes.
