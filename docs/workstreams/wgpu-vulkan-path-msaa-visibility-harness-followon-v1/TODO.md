# WGPU Vulkan Path MSAA Visibility Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Vulkan Visibility Helper Migration

- [x] WVMV-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/vulkan_path_msaa_visibility_conformance.rs]
  Goal: Remove local readback/pixel helpers from the Vulkan path-MSAA visibility conformance test and route them through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test vulkan_path_msaa_visibility_conformance -j 1`.
  Evidence: the test no longer carries local `read_texture_rgba8` / `pixel_rgba` copies, while env guard, Vulkan capability guard, perf assertions, and visibility assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for readback and pixel sampling.

## M1 — Gates And Closeout

- [x] WVMV-020 [owner=codex] [deps=WVMV-010] [scope=docs/workstreams/wgpu-vulkan-path-msaa-visibility-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the test migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
