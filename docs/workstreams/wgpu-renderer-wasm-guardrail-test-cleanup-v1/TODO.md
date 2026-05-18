# WGPU Renderer Wasm Guardrail Test Cleanup v1 - TODO

# Status: Closed
Last updated: 2026-05-18

## M0 - Guardrail Constant Relocation

- [x] WGWG-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/renderer/tests.rs]
  Goal: Move the derivatives smoke shader constant into the wasm-only guardrail module.
  Validation: native `cargo check -p fret-render-wgpu --locked --tests -j 1`.
  Evidence: native `dead_code` scan no longer reports the guardrail constant at outer scope.
  Status: Done on 2026-05-18.

## M1 - Wgpu 29 Test API Drift Fixes

- [x] WGWG-020 [owner=codex] [deps=WGWG-010] [scope=crates/fret-render-wgpu/src/renderer/tests.rs,crates/fret-render-wgpu/src/renderer/render_plan_reporting_perf.rs]
  Goal: Update the wasm-only guardrail test and perf reporting literals to the current wgpu 29 / pass struct APIs.
  Validation: `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`; `cargo nextest run -p fret-render-wgpu --locked shaders_parse_as_wgsl shaders_validate_for_webgpu`.
  Evidence: both native and wasm feature checks pass.
  Status: Done on 2026-05-18.

## M2 - Closeout

- [x] WGWG-030 [owner=codex] [deps=WGWG-020] [scope=docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1,docs/workstreams/README.md]
  Goal: Record verification evidence and close the narrow test-cleanup lane.
  Validation: `python tools/check_workstream_catalog.py`; `python -m json.tool docs/workstreams/wgpu-renderer-wasm-guardrail-test-cleanup-v1/WORKSTREAM.json`; `git diff --check`.
  Evidence: closeout audit names the moved guardrail constant and the API drift fixes.
  Status: Done on 2026-05-18.
