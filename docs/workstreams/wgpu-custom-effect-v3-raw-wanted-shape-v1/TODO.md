# WGPU Custom Effect V3 Raw Wanted Shape v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Cross-Target Field Shape

- [x] WCRW-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/renderer/render_plan.rs,crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs,crates/fret-render-wgpu/src/renderer/render_plan_reporting_perf.rs]
  Goal: Make `CustomEffectV3Pass::raw_wanted` part of the cross-target render-plan data model.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: field definition, V3 pass construction, and reporting test literals no longer require
  `#[cfg(not(target_arch = "wasm32"))]`.
  Status: Done on 2026-05-18.

## M1 - Custom Effect V3 Behavior Gates

- [x] WCRW-020 [owner=codex] [deps=WCRW-010] [scope=crates/fret-render-wgpu/src/renderer/render_plan.rs,crates/fret-render-wgpu/src/renderer/render_plan_effects/custom.rs]
  Goal: Prove the unified flag shape does not weaken source target planning or summary counters.
  Validation: targeted `cargo nextest run -p fret-render-wgpu --locked ...` commands for Custom
  Effect V3 plan construction and reporting.
  Evidence: targeted Custom Effect V3 nextest gates passed.
  Status: Done on 2026-05-18.

## M2 - Closeout

- [x] WCRW-030 [owner=codex] [deps=WCRW-020] [scope=docs/workstreams/wgpu-custom-effect-v3-raw-wanted-shape-v1,docs/workstreams/README.md]
  Goal: Record the invariant, verification evidence, and close the narrow follow-on.
  Validation: `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/wgpu-custom-effect-v3-raw-wanted-shape-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: closeout audit records why lifecycle validation remains unconditional for V3 source
  views.
  Status: Done on 2026-05-18.
