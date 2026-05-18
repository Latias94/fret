# Renderer Dump Wasm Stub Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`renderer/mod.rs` carried inline wasm stub implementations for renderer debug dump modules:

- `RenderPlanJsonDumpScratch`
- `Renderer::maybe_dump_render_text_json`

It also declared wasm-only empty modules for native-only dump implementation details. This made the
renderer root module own implementation details instead of only module assembly.

## Target State

Platform-specific dump stubs live in their own modules:

- `render_plan_dump_assemble_wasm.rs`
- `render_text_dump_wasm.rs`

`renderer/mod.rs` only selects the correct module file for the target. Native-only helper modules
are declared only on native targets.

## Scope

- `crates/fret-render-wgpu/src/renderer/mod.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan_dump_assemble_wasm.rs`
- `crates/fret-render-wgpu/src/renderer/render_text_dump_wasm.rs`

## Non-Goals

- No debug dump schema changes.
- No native dump behavior changes.
- No wasm dump enablement.
- No public renderer API changes.

## Architecture Direction

Keep module roots declarative. Target-specific no-op implementations should live next to the module
they satisfy, not inside the root module declaration list.
