# WGPU Paint Gradient Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow follow-on is closed. `paint_gradient_conformance.rs` now shares the existing
integration-test support module for final scene rendering, texture readback, and RGBA pixel
sampling.

No paint or gradient behavior changed. The migration only removed duplicated test harness code.

## Evidence

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/paint_gradient_conformance.rs`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test paint_gradient_conformance -j 1`
  - Result: 6 tests run, 6 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-On Policy

Do not reopen this lane broadly. `composite_group_conformance.rs` uses `Rgba8UnormSrgb`; migrate it
only after a separate follow-on decides whether shared support should grow a format-aware render
helper.
