# WGPU Clip Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is closed. The two clip-related WGPU conformance tests now use the shared
integration-test support module for final render readback and pixel sampling.

## Next Action

No continuation action remains in this lane. Future renderer test-family migrations should start a
narrow follow-on instead of reopening this one.

## Validation

```bash
cargo nextest run -p fret-render-wgpu --locked --test clip_path_conformance --test affine_clip_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```
