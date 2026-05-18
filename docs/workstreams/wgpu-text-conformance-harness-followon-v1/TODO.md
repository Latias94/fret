# WGPU Text Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Text Harness Migration

- [x] WTX-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/text_paint_conformance.rs,crates/fret-render-wgpu/tests/text_outline_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the named text conformance tests and
  route them through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test text_paint_conformance --test text_outline_conformance -j 1`.
  Evidence: the tests no longer carry local `read_texture_rgba8` / `pixel_rgba` /
  `render_and_readback` copies, while deterministic font setup and existing text assertions remain
  equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for the named tests.

## M1 — Gates And Closeout

- [x] WTX-020 [owner=codex] [deps=WTX-010] [scope=docs/workstreams/wgpu-text-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the named tests migrate.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated files and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
