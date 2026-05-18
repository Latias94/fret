# Renderer Dump Wasm Stub Owner v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Move Wasm Stub Owners Out Of Root Module

- [x] RDWS-010 [owner=codex] [deps=none] [scope=crates/fret-render-wgpu/src/renderer]
  Goal: Move inline wasm debug dump stubs out of `renderer/mod.rs` and delete unused wasm empty
  module declarations for native-only dump internals.
  Validation: `cargo check -p fret-render-wgpu --locked --target wasm32-unknown-unknown --features wasm-webgpu-tests --tests -j 1`.
  Evidence: wasm compile gate passed after moving the stubs to module-owned files.
  Status: Done on 2026-05-18.

## M1 - Native And Boundary Guardrails

- [x] RDWS-020 [owner=codex] [deps=RDWS-010] [scope=crates/fret-render-wgpu,tools/check_layering.py]
  Goal: Prove native test builds and crate layering remain unchanged.
  Validation: `cargo check -p fret-render-wgpu --locked --tests -j 1`; `python tools/check_layering.py`.
  Evidence: native compile and layering gates passed.
  Status: Done on 2026-05-18.
