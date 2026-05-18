# WGPU Paint Gradient Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Paint Gradient Test Harness Migration

- [x] WPG-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/paint_gradient_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the paint-gradient conformance test
  and route it through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test paint_gradient_conformance -j 1`.
  Evidence: the test no longer carries local `read_texture_rgba8` / `render_and_readback` copies,
  while existing gradient assertions and setup behavior remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for final readback.

## M1 — Gates And Closeout

- [x] WPG-020 [owner=codex] [deps=WPG-010] [scope=docs/workstreams/wgpu-paint-gradient-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the paint-gradient test
  migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
