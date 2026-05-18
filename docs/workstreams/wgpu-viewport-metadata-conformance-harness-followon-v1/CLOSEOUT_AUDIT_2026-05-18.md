# WGPU Viewport Metadata Conformance Harness Follow-on v1 — Closeout Audit

Status: Closed
Last updated: 2026-05-18

## Summary

This lane removed duplicated final render/readback and pixel helpers from
`crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs`.

The test still owns source texture setup, `RenderTargetMetadata` registration/update, alpha-mode
assertions, and orientation assertions.

## Verification

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test viewport_surface_metadata_conformance -j 1`
  - Result: nextest run ID `ce30508a-9445-4bcb-a0d2-031cc466af0c`; 2 tests run, 2 passed, 0 skipped.
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-viewport-metadata-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Closeout Verdict

Closed. Future Vulkan, MSAA, or host-topology helper cleanup should start as separate narrow
follow-ons because their setup contracts differ.
