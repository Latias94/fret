# WGPU Renderer Wasm Guardrail Test Cleanup v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`crates/fret-render-wgpu/src/renderer/tests.rs` carried a wasm-only WebGPU/Tint guardrail shader
constant at the outer test-module scope, which forced a `dead_code` allowance on native builds.

While fixing that, the wasm-only guardrail test also exposed two API drift issues against wgpu 29:

- `InstanceDescriptor::default()` no longer existed on that type,
- `ErrorScopeGuard` needed explicit `pop()` handling instead of `Device::pop_error_scope().await`.

This lane keeps the wasm guardrail logic in the wasm-only module and updates the test to the current
wgpu 29 API.

## Assumptions First

- Confident: the derivatives smoke shader is wasm-only. Evidence:
  it is only referenced inside the `#[cfg(all(target_arch = "wasm32", feature = "wasm-webgpu-tests"))]`
  guardrail module. If wrong, native builds would still need the constant at outer scope.
- Confident: `InstanceDescriptor::new_without_display_handle()` is the correct replacement for the
  removed `default()` call in this test path. Evidence:
  `wgpu-29.0.0` exposes that constructor.
- Confident: `ErrorScopeGuard::pop()` is the correct replacement for awaiting `Device::pop_error_scope`.
  Evidence: `wgpu-29.0.0` exposes `ErrorScopeGuard::pop(self)`.
- Confident: the `raw_wanted` field in `CustomEffectV3Pass` test literals must stay behind the same
  `#[cfg(not(target_arch = "wasm32"))]` shape as the struct field. If wrong, wasm test compilation
  would continue to fail on field mismatch.

## Target State

- The wasm-only derivatives smoke shader constant lives inside the wasm-only guardrail module.
- Native builds no longer need a `dead_code` allowance for that constant.
- The wasm-only guardrail test compiles on wgpu 29.
- The perf reporting test literals match the current `CustomEffectV3Pass` field layout across cfgs.

## Out Of Scope

- Rewriting the whole renderer test suite.
- Removing the remaining test-support `dead_code` allowance in `tests/support/mod.rs`.
- Changing shader semantics or guardrail coverage.

## Closure Policy

Close this lane once native and wasm feature checks pass and the guardrail test remains in the
wasm-only cfg module.

## Closure

Closed on 2026-05-18 after moving the wasm-only guardrail constant and updating the wgpu 29 test API.
