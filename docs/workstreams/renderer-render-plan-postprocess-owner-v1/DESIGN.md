# Renderer Render Plan Postprocess Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`renderer/render_plan.rs` still owns debug postprocess construction helpers alongside the
render-plan data model. Pixelate and blur postprocess lowering is a pass-construction policy over an
existing plan, not part of the core pass data model.

## Target State

The main render-plan module owns the data model and orchestration methods. Debug postprocess lowering
helpers live in a sibling owner module:

- `crates/fret-render-wgpu/src/renderer/render_plan/postprocess.rs`

## Scope

- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/postprocess.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs`

## Non-Goals

- No postprocess behavior changes.
- No render-plan pass model changes.
- No compiler behavior changes.
- No validation or lifecycle-analysis changes.
- No public API changes.

## Architecture Direction

Keep `render_plan.rs` focused on the pass model and final orchestration. Debug postprocess lowering
should be owned by a module named after that behavior so future pixelate/blur changes have locality.
