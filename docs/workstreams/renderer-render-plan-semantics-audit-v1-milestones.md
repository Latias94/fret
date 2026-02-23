# Renderer RenderPlan Semantics Audit v1 — Milestones

## M0 — Guardrails in place

Completion criteria:
- `RenderPlan::debug_validate()` exists (debug-only).
- It is called from `render_scene_execute` before pass recording.
- It detects at least:
  - read-after-release
  - `LoadOp::Load` on uninitialized targets
  - scissor bounds / intersection mistakes

## M1 — Invariants documented

Completion criteria:
- `renderer-render-plan-semantics-audit-v1.md` lists the semantics we rely on.
- Any “unknown/ambiguous” semantics are explicitly called out as TODO.

## M2 — Regression tests for brittle semantics

Completion criteria:
- At least 3 targeted tests cover the most failure-prone semantics (lifetime/load/scissor mapping).
- Tests are stable and do not require large golden assets.

Progress record (Pass-by-pass semantics checklist + scale/scissor mapping notes):

- Date: 2026-02-23
- Status: Landed (docs)
- Evidence anchors:
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1.md` (Pass semantics summary, Scale/scissor mapping notes)
  - `docs/workstreams/renderer-render-plan-semantics-audit-v1-todo.md` (audit item checked)

Progress record (Scissored in-place effect preservation tests):

- Date: 2026-02-23
- Status: Landed (unit tests)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs` (`scissored_*` tests)
  - `crates/fret-render-wgpu/src/renderer/render_plan_effects.rs` (in-place patterns + `LoadOp::Load`)

Progress record (PathMsaaBatch init pass shape test):

- Date: 2026-02-23
- Status: Landed (unit test)
- Evidence anchors:
  - `crates/fret-render-wgpu/src/renderer/render_plan/tests.rs` (`compile_for_scene_path_msaa_batch_initializes_output_via_empty_clear_pass`)
  - `crates/fret-render-wgpu/src/renderer/render_plan_compiler.rs` (`flush_scene_range` before `PathMsaaBatch`)

## M3 — Plan diagnostics for refactors

Completion criteria:
- A compact per-pass plan dump exists for trace/debug builds.
- It is easy to compare plan shapes across refactors and across degradations.
