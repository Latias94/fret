# Renderer Render Plan Debug Validation Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`renderer/render_plan.rs` still owns debug-only validation helpers (`debug_validate` and
`debug_validate_first_output_write_is_clear`) alongside the core render-plan model and compiler
surface. That keeps diagnostic-only checking logic inline with the main render-plan module.

## Target State

The main render-plan module owns the data model and core methods. Debug-only validation helpers live
in a sibling debug-assertions owner module:

- `crates/fret-render-wgpu/src/renderer/render_plan/debug.rs`

## Scope

- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/debug.rs`

## Non-Goals

- No render-plan compiler behavior changes.
- No pass/type reshaping.
- No validation rule changes.
- No public API changes.
- No wasm-only behavior changes.

## Architecture Direction

Keep the render-plan data model deep and the debug validation surface separate. Debug assertions
should not live inline with the core model once they can be owned by a narrower debug module.
