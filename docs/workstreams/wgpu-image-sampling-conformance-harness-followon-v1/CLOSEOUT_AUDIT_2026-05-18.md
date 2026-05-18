# WGPU Image Sampling Conformance Harness Follow-on v1 — Closeout Audit

Date: 2026-05-18
Status: Closed

## Shipped Outcome

The image sampling WGPU conformance test now shares RGBA8 readback and pixel helpers while keeping
its explicit render-target setup local.

Migrated file:

- `crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs`

Preserved behavior:

- Nearest-vs-linear sampling difference coverage.
- Mixed primitive ordering coverage.
- Checkerboard image registration.
- Per-test explicit render target and clear behavior.

## Gates

- PASS: `cargo fmt --package fret-render-wgpu`
- PASS: `cargo nextest run -p fret-render-wgpu --locked --test image_sampling_hint_conformance -j 1`
- PASS: `cargo check -p fret-render-wgpu --locked --tests -j 1`
- PASS: `python tools/check_layering.py`
- PASS: `python tools/check_workstream_catalog.py`
- PASS: `python -m json.tool docs/workstreams/wgpu-image-sampling-conformance-harness-followon-v1/WORKSTREAM.json`
- PASS: `git diff --check`

## Residual Follow-ons

- Output transfer, viewport metadata, Vulkan, MSAA, and host-topology tests remain separate because
  they involve specialized formats, metadata, backend setup, or platform behavior.

## Verdict

Closed. This lane is a pure test-harness deduplication and does not change renderer semantics.
