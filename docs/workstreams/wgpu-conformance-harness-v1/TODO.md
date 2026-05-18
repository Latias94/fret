# WGPU Conformance Harness v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Path Test Harness Extraction

- [x] WCH-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/support/mod.rs,crates/fret-render-wgpu/tests/path_*_conformance.rs]
  Goal: Extract the common WGPU render/readback/pixel helpers into `tests/support/` and migrate the
  path base, path stroke style v2, path paint, and path material paint conformance tests.
  Validation: `cargo test -p fret-render-wgpu --locked --test path_base_conformance --test path_stroke_style_v2_conformance --test path_paint_conformance --test path_material_paint_conformance -j 1`.
  Evidence: path-related tests no longer carry local `read_texture_rgba8` / `render_and_readback`
  copies, while assertions remain behavior-equivalent.
  Status: Done on 2026-05-18. Shared helper extracted and first path batch migrated.

## M1 — Gates And Closeout

- [x] WCH-020 [owner=codex] [deps=WCH-010] [scope=docs/workstreams/wgpu-conformance-harness-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the lane if the first path-related batch is enough.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit or an explicit narrower follow-on if another renderer test family should
  migrate next.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
