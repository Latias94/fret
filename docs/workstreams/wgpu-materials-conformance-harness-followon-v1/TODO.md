# WGPU Materials Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Material Test Harness Migration

- [x] WMH-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/materials_conformance.rs,crates/fret-render-wgpu/tests/materials_sampled_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the material-related conformance tests
  and route those tests through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test materials_conformance --test materials_sampled_conformance -j 1`.
  Evidence: the two tests no longer carry local `read_texture_rgba8` / `render_and_readback` copies,
  while existing material assertions and setup behavior remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for final readback in both material tests.

## M1 — Gates And Closeout

- [x] WMH-020 [owner=codex] [deps=WMH-010] [scope=docs/workstreams/wgpu-materials-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the material batch
  migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated files and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
