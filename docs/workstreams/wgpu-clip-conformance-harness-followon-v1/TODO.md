# WGPU Clip Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Clip Test Harness Migration

- [x] WCF-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/clip_path_conformance.rs,crates/fret-render-wgpu/tests/affine_clip_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the clip-related conformance tests and
  route those tests through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test clip_path_conformance --test affine_clip_conformance -j 1`.
  Evidence: the two tests no longer carry local `read_texture_rgba8` / `render_and_readback` copies,
  while existing assertions and setup behavior remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for final readback in both clip tests.

## M1 — Gates And Closeout

- [x] WCF-020 [owner=codex] [deps=WCF-010] [scope=docs/workstreams/wgpu-clip-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the clip batch migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated files and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
