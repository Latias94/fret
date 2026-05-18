# Renderer Render Plan Postprocess Tests Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`renderer/render_plan/tests.rs` still owns all render-plan tests after the production
postprocess helpers moved into `render_plan/postprocess.rs`. Postprocess behavior tests now have a
clear owner, but their test code is still mixed with debug validation, lifecycle analysis, and
compiler guardrail tests.

## Target State

Postprocess-specific tests live in a sibling test owner module:

- `crates/fret-render-wgpu/src/renderer/render_plan/tests/postprocess.rs`

The root `tests.rs` keeps shared helpers and the remaining non-postprocess tests.

## Scope

- `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/tests/postprocess.rs`
- `docs/workstreams/README.md`

## Non-Goals

- No production behavior changes.
- No assertion changes.
- No broad test taxonomy migration.
- No debug validation or lifecycle test moves in this slice.

## Architecture Direction

Keep tests aligned with owner modules. Once `postprocess.rs` owns lowering behavior, postprocess
tests should live behind a matching test owner so future postprocess work has local evidence.
