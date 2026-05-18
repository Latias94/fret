# Renderer Render Plan Analysis Owner v1 - Design

Status: Closed
Last updated: 2026-05-18

## Problem

`renderer/render_plan.rs` still owns plan-analysis helpers alongside the render-plan data model and
postprocess construction helpers. The peak intermediate memory estimator and early-release insertion
logic are lifecycle analysis concerns, not model definitions.

## Target State

The main render-plan module owns the data model and orchestration methods. Plan-analysis helpers live
in a sibling owner module:

- `crates/fret-render-wgpu/src/renderer/render_plan/analysis.rs`

## Scope

- `crates/fret-render-wgpu/src/renderer/render_plan.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/analysis.rs`
- `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs`

## Non-Goals

- No render-plan pass model changes.
- No compiler behavior changes.
- No memory-estimation rule changes.
- No early-release insertion behavior changes.
- No public API changes.

## Architecture Direction

Keep the core render-plan model compact. Analysis over an already-built pass list should sit behind a
narrow sibling module so future lifecycle and memory diagnostics do not grow inside the model file.
