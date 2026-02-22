# Renderer RenderPlan Semantics Audit v1 — Milestones

## M0 — Guardrails in place

Completion criteria:
- `RenderPlan::debug_validate()` exists (debug-only).
- It is called from `render_scene_execute` before pass recording.
- It detects at least:
  - read-after-release
  - `LoadOp::Load` on uninitialized targets

## M1 — Invariants documented

Completion criteria:
- `renderer-render-plan-semantics-audit-v1.md` lists the semantics we rely on.
- Any “unknown/ambiguous” semantics are explicitly called out as TODO.

## M2 — Regression tests for brittle semantics

Completion criteria:
- At least 3 targeted tests cover the most failure-prone semantics (lifetime/load/scissor mapping).
- Tests are stable and do not require large golden assets.

## M3 — Plan diagnostics for refactors

Completion criteria:
- A compact per-pass plan dump exists for trace/debug builds.
- It is easy to compare plan shapes across refactors and across degradations.

