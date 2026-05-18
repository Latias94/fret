# WGPU Mask Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Verdict

This narrow follow-on is closed. The mask-related WGPU conformance tests now share the existing
integration-test support module for final scene rendering, texture readback, and RGBA pixel
sampling.

No mask behavior changed. The migration only removed duplicated test harness code.

## Evidence

- `crates/fret-render-wgpu/tests/support/mod.rs`
- `crates/fret-render-wgpu/tests/mask_gradient_conformance.rs`
- `crates/fret-render-wgpu/tests/mask_image_conformance.rs`

## Gates Run

2026-05-18:

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test mask_gradient_conformance --test mask_image_conformance -j 1`
  - Result: 7 tests run, 7 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `git diff --check`

## Follow-On Policy

Do not reopen this lane broadly. If another renderer test family still carries duplicated WGPU
readback or render harness code, start a new follow-on that names that family explicitly.
