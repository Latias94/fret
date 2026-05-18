# WGPU Stroke Dash Shadow Conformance Harness Follow-on v1 — Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This narrow follow-on is closed. The named stroke, dash, and shadow WGPU conformance tests now use
the shared integration-test support module for final render readback and pixel sampling.

## Next Action

No continuation action remains in this lane. Remaining WGPU tests with local readback helpers should
be migrated as separate one-family follow-ons after their target format, scale factor, render
target, and setup are checked.

## Validation

```bash
cargo nextest run -p fret-render-wgpu --locked --test dashed_border_conformance --test dash_semantics_rrect_vs_path_conformance --test stroke_paint_conformance --test shadow_rrect_conformance -j 1
cargo check -p fret-render-wgpu --locked --tests -j 1
python tools/check_layering.py
python tools/check_workstream_catalog.py
git diff --check
```
