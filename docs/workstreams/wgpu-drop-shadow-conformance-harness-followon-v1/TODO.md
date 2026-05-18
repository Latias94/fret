# WGPU Drop Shadow Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Drop Shadow Harness Migration

- [x] WDS-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/effect_drop_shadow_v1_conformance.rs]
  Goal: Remove local final-render readback/pixel helpers from the drop-shadow conformance test and
  route it through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test effect_drop_shadow_v1_conformance -j 1`.
  Evidence: the test no longer carries local `read_texture_rgba8` / `pixel_rgba` /
  `render_and_readback` copies, while the DropShadowV1 scene and assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for the test.

## M1 — Gates And Closeout

- [x] WDS-020 [owner=codex] [deps=WDS-010] [scope=docs/workstreams/wgpu-drop-shadow-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the test migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
