# Renderer Render Plan Debug Validation Tests Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`renderer/render_plan/tests.rs` still owns debug validation tests even though the production
validators live in `render_plan/debug.rs`. That keeps validation-specific fixtures mixed with
lifecycle, compiler, and effect guardrail tests.

## Target State

Debug validation tests live in a sibling test owner module:

- `crates/fret-render-wgpu/src/renderer/render_plan/tests/debug_validation.rs`

The root `tests.rs` keeps shared helpers and the remaining non-debug-validation tests.

## Scope

- `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/tests/debug_validation.rs`
- `docs/workstreams/README.md`

## Non-Goals

- No production behavior changes.
- No assertion changes.
- No lifecycle or compiler test moves in this slice.
- No debug validator rule changes.

## Architecture Direction

Keep tests aligned with owner modules. Once `debug.rs` owns validation behavior, validation tests
should live behind a matching test owner so future validation work has local evidence.
