# WGPU Composite Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is closed. The composite-group WGPU conformance test now uses the shared
integration-test support module for final render readback and pixel sampling, with a local wrapper
that keeps the `Rgba8UnormSrgb` output-format contract explicit.

## Next Action

No continuation action remains in this lane. Remaining WGPU tests with local readback helpers should
be migrated as separate one-family follow-ons only after their format, scale factor, and target setup
are checked.

## Validation

```bash
cargo nextest run -p fret-render-wgpu --locked --test composite_group_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```
