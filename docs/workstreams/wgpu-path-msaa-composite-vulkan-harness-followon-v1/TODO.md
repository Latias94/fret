# WGPU Path MSAA Composite Vulkan Harness Follow-on v1 — TODO

Status: Closed
Last updated: 2026-05-18

## M0 — Vulkan Composite Readback Migration

- [x] WPMCV-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/tests/path_msaa_composite_vulkan.rs]
  Goal: Remove local raw readback helper from the Vulkan path-MSAA composite smoke test and route it through `crates/fret-render-wgpu/tests/support/mod.rs`.
  Validation: `cargo nextest run -p fret-render-wgpu --locked --test path_msaa_composite_vulkan -j 1`.
  Evidence: the test no longer carries local `read_texture_rgba8` / `pixel_bgra` helper bodies, while the `pixel_bgra` alias, Vulkan guard, explicit BGRA target, and assertions remain equivalent.
  Status: Done on 2026-05-18. Shared helper adopted for raw readback and byte sampling.

## M1 — Gates And Closeout

- [x] WPMCV-020 [owner=codex] [deps=WPMCV-010] [scope=docs/workstreams/wgpu-path-msaa-composite-vulkan-harness-followon-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow follow-on after the test migrates.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`; `python tools/check_workstream_catalog.py`; `git diff --check`.
  Evidence: closeout audit that names the migrated file and gates.
  Status: Done on 2026-05-18. Lane closed with `CLOSEOUT_AUDIT_2026-05-18.md`.
