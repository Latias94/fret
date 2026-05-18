# WGPU Text Measure Dead Branch Prune v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Delete Unreachable Measurement Copies

- [x] WTMD-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/text/measure.rs]
  Goal: Remove the always-false `#[cfg(any())]` inline measurement implementations from the WGPU
  text facade.
  Validation: `rg -n "#\\[cfg\\(any\\(\\)\\)\\]|cfg\\(any\\(\\)\\)" crates/fret-render-wgpu/src/text/measure.rs`; `cargo check -p fret-render-wgpu --locked --tests -j 1`.
  Evidence: `measure.rs` contains only facade calls into `TextMeasureCaches`.
  Status: Done on 2026-05-18.

## M1 - Measurement Parity Gates

- [x] WTMD-020 [owner=codex] [deps=WTMD-010] [scope=crates/fret-render-wgpu/src/text,crates/fret-render-wgpu/tests/text_measure_matches_prepare.rs]
  Goal: Prove renderer-facing plain and attributed measurement still matches prepared layout metrics.
  Validation: `cargo nextest run -p fret-render-wgpu --locked text_measure_matches_prepare_across_fractional_scale_factors wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_matches_prepare_under_fractional_scale_factor grapheme_wrapped_measure_attributed_matches_prepare_under_fractional_scale_factor`.
  Evidence: targeted text measurement nextest gate passed.
  Status: Done on 2026-05-18.

## M2 - Closeout

- [x] WTMD-030 [owner=codex] [deps=WTMD-020] [scope=docs/workstreams/wgpu-text-measure-dead-branch-prune-v1,docs/workstreams/README.md]
  Goal: Record the ownership invariant and close the narrow follow-on.
  Validation: `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/wgpu-text-measure-dead-branch-prune-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: closeout audit records `fret-render-text::TextMeasureCaches` as the active measurement owner.
  Status: Done on 2026-05-18.
