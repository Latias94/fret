# WGPU Stroke Dash Shadow Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow follow-on is closed. The named stroke, dash, and shadow WGPU conformance tests now share
the integration-test support module for final scene rendering, texture readback, and RGBA pixel
sampling.

No renderer behavior changed. The migration only removed duplicated test harness code and preserved
the existing `Rgba8Unorm` transparent-clear behavior and scale-factor coverage.

## Evidence

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/dashed_border_conformance.rs`
- `crates/fret-render-wgpu/tests/dash_semantics_rrect_vs_path_conformance.rs`
- `crates/fret-render-wgpu/tests/stroke_paint_conformance.rs`
- `crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test dashed_border_conformance --test dash_semantics_rrect_vs_path_conformance --test stroke_paint_conformance --test shadow_rrect_conformance -j 1`
  - Result: 9 tests run, 9 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-On Policy

Do not reopen this lane broadly. Future readback-helper migrations should stay one test family at a
time, especially where target format, scale factor, render-target setup, Vulkan-specific behavior,
or metadata assertions differ from the default `Rgba8Unorm` shared helper.
