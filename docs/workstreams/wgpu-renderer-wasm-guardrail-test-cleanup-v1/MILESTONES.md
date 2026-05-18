# WGPU Renderer Wasm Guardrail Test Cleanup v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Wasm-only Guardrail Lives In Wasm-only Scope

Exit criteria:

- The derivatives smoke shader constant is inside the wasm-only guardrail module.
- Native builds no longer need a `dead_code` allowance for that constant.

Status: Complete on 2026-05-18.

## M1 - Wgpu 29 Test API Compiles

Exit criteria:

- Native `cargo check -p fret-render-wgpu --locked --tests -j 1` passes.
- Wasm feature `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1` passes.
- Guardrail shader tests still pass.

Status: Complete on 2026-05-18.

## M2 - Workstream Closed

Exit criteria:

- Catalog and JSON checks pass.
- Closeout note records residual test-support allowance left out of scope.

Status: Complete on 2026-05-18.
