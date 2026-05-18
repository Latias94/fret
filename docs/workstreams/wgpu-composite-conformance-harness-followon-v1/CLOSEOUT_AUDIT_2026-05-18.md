# WGPU Composite Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow follow-on is closed. `composite_group_conformance.rs` now shares the WGPU integration
test support module for final scene rendering, texture readback, and RGBA pixel sampling.

No composite behavior changed. The migration only removed duplicated test harness code and made the
test's `Rgba8UnormSrgb` output format explicit through a format-aware shared helper.

## Evidence

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/composite_group_conformance.rs`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test composite_group_conformance -j 1`
  - Result: 4 tests run, 4 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-On Policy

Do not reopen this lane broadly. Future readback-helper migrations should stay one test family at a
time, especially where target format, scale factor, or custom render-target setup differs from the
default `Rgba8Unorm` shared helper.
