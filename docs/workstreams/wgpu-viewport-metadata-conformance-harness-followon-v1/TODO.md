# WGPU Viewport Metadata Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Viewport Metadata Harness Migration

- [x] WVM-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/viewport_surface_metadata_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the viewport metadata conformance test and route them through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test viewport_surface_metadata_conformance -j 1`.
  Evidence: the test no longer carries local `read_texture_rgba8` / `pixel_rgba` / `render_and_readback` copies, while source texture writers, metadata registration, and assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for final render/readback and pixel sampling.

## M1 — Gates And Closeout

- [x] WVM-020 [owner=codex] [deps=WVM-010] [scope=docs/workstreams/wgpu-viewport-metadata-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the test migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
