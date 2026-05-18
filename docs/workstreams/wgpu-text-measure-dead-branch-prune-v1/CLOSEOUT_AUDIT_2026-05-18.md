# WGPU Text Measure Dead Branch Prune v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane removed stale unreachable measurement implementation copies from the WGPU text facade.

Deleted:

- the always-false plain-text `#[cfg(any())]` measurement block,
- the always-false attributed-text `#[cfg(any())]` measurement block.

Preserved:

- `TextSystem::measure` and `TextSystem::measure_attributed` still call
  `self.layout_cache.measure`.
- `fret_render_text::TextMeasureCaches` remains the single measurement cache and wrapping owner.
- Existing measure/prepare parity behavior remains unchanged.

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `rg -n "#\\[cfg\\(any\\(\\)\\)\\]|cfg\\(any\\(\\)\\)" crates/fret-render-wgpu/src/text/measure.rs`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `cargo nextest run -p fret-render-wgpu --locked text_measure_matches_prepare_across_fractional_scale_factors wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_attributed_matches_prepare_under_fractional_scale_factor`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-text-measure-dead-branch-prune-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. WGPU text measurement no longer carries unreachable duplicate implementation code.
