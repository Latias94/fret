# WGPU Composite Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Composite Test Harness Migration

- [x] WCG-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/support/mod.rs,crates/fret-render-wgpu/tests/composite_group_conformance.rs]
  Goal: Add a format-aware shared WGPU final-render readback helper and route composite-group
  conformance through it without changing the test's `Rgba8UnormSrgb` output format.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test composite_group_conformance -j 1`.
  Evidence: the composite test no longer carries local `read_texture_rgba8` / `pixel_rgba` /
  `render_and_readback` copies, while its local wrapper still names the `Rgba8UnormSrgb` contract.
  Status: Done on 2026-05-18. Shared format-aware helper adopted for composite readback.

## M1 — Gates And Closeout

- [x] WCG-020 [owner=codex] [deps=WCG-010] [scope=docs/workstreams/wgpu-composite-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the composite test
  migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
