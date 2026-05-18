# WGPU Text Measure Dead Branch Prune v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-18

## Baseline Observation

`TextSystem::measure` and `TextSystem::measure_attributed` returned immediately through
`self.layout_cache.measure`, then retained old inline implementations behind `#[cfg(any())]`.

The active implementation owner is `fret_render_text::TextMeasureCaches`, stored in
`TextLayoutCacheState::measure`.

## Gate Set

```bash
cargo fmt --package fret-render-wgpu
rg -n "#\\[cfg\\(any\\(\\)\\)\\]|cfg\\(any\\(\\)\\)" crates/fret-render-wgpu/src/text/measure.rs
cargo check -p fret-render-wgpu --locked --tests -j 1
cargo nextest run -p fret-render-wgpu --locked text_measure_matches_prepare_across_fractional_scale_factors wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_attributed_matches_prepare_under_fractional_scale_factor
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/wgpu-text-measure-dead-branch-prune-v1/WORKSTREAM.json
git diff --check
```

## Results

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `rg -n "#\\[cfg\\(any\\(\\)\\)\\]|cfg\\(any\\(\\)\\)" crates/fret-render-wgpu/src/text/measure.rs`
  - Result: no matches.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked text_measure_matches_prepare_across_fractional_scale_factors wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_attributed_matches_prepare_under_fractional_scale_factor`
  - Result: nextest run ID `7a03baa6-c36e-4ad2-af6a-7add713fb9f6`; 4 tests run, 4 passed, 282 skipped.
- PASS: `python tools/check_workstream_catalog.py`
  - Result: 411 dedicated directories and 47 standalone markdown files validated.
- PASS: `python -m json.tool docs/workstreams/wgpu-text-measure-dead-branch-prune-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Evidence Anchors

- `crates/fret-render-wgpu/src/text/measure.rs`
- `crates/fret-render-wgpu/src/text/layout_cache_state.rs`
- `crates/fret-render-text/src/measure.rs`
- `crates/fret-render-wgpu/tests/text_measure_matches_prepare.rs`
- `docs/workstreams/wgpu-text-measure-dead-branch-prune-v1/CLOSEOUT_AUDIT_2026-05-18.md`
