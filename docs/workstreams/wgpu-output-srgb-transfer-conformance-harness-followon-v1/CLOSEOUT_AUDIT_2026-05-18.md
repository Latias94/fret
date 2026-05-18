# WGPU Output sRGB Transfer Conformance Harness Follow-on v1 — Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane removed duplicated RGBA8 readback and pixel helpers from
`crates/fret-render-wgpu/tests/output_srgb_transfer_conformance.rs`.

The test still owns its explicit `Rgba8Unorm` output texture setup and verifies the explicit final
sRGB transfer for non-sRGB 8-bit output formats.

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test output_srgb_transfer_conformance -j 1`
  - Result: nextest run ID `6088c6c3-073e-44b5-b5be-b178dd208421`; 1 test run, 1 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-output-srgb-transfer-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. Future viewport metadata, Vulkan, MSAA, or host-topology helper cleanup should start as
separate narrow follow-ons because their setup contracts differ.
