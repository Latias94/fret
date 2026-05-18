# WGPU Image Sampling Conformance Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Image Sampling Helper Migration

- [x] WIS-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/image_sampling_hint_conformance.rs]
  Goal: Remove local readback/pixel helpers from the image sampling conformance test and route them
  through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test image_sampling_hint_conformance -j 1`.
  Evidence: the test no longer carries local `read_texture_rgba8` / `pixel_rgba` copies, while
  explicit render-target setup and image sampling assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for readback and pixel sampling.

## M1 — Gates And Closeout

- [x] WIS-020 [owner=codex] [deps=WIS-010] [scope=docs/workstreams/wgpu-image-sampling-conformance-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the test migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`;
  `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
