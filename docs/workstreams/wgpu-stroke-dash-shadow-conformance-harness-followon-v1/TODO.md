# WGPU Stroke Dash Shadow Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Stroke, Dash, And Shadow Test Harness Migration

- [x] WSDS-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/dashed_border_conformance.rs,crates/fret-render-wgpu/tests/dash_semantics_rrect_vs_path_conformance.rs,crates/fret-render-wgpu/tests/stroke_paint_conformance.rs,crates/fret-render-wgpu/tests/shadow_rrect_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the named stroke, dash, and shadow
  conformance tests and route them through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test dashed_border_conformance --test dash_semantics_rrect_vs_path_conformance --test stroke_paint_conformance --test shadow_rrect_conformance -j 1`.
  Evidence: the tests no longer carry local `read_texture_rgba8` / `pixel_rgba` /
  `render_and_readback` copies, while existing scale-factor loops and assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for the named tests.

## M1 — Gates And Closeout

- [x] WSDS-020 [owner=codex] [deps=WSDS-010] [scope=docs/workstreams/wgpu-stroke-dash-shadow-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the named tests migrate.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated files and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
