# ADR 0187: UI Frame Pipeline Ordering (Propagate → Mount → Layout/Paint)

- Status: Proposed
- Date: 2026-01-16

## Context

Fret's declarative mount pass (`declarative::render_root`) can reuse cached subtrees (ViewCache) by skipping child execution when the subtree is clean. This makes per-frame correctness sensitive to *when* invalidation is applied.

In Fret today, invalidation is derived from two primary sources outside the UI tree itself:

- Model changes (`UiTree::propagate_model_changes`)
- Global changes (`UiTree::propagate_global_changes`)

Different runners/demos may currently call these in different places (or forget them), which can cause intermittent cache mis-hits (stale UI, missing updates, or inconsistent interaction behavior).

## Decision

We define a single per-frame ordering contract for each window:

1. Drain changed models / globals from the host/runtime
2. Apply invalidation to the UI tree:
   - `UiTree::propagate_model_changes`
   - `UiTree::propagate_global_changes`
3. Mount the declarative element tree (`declarative::render_root`)
4. Layout and paint (`UiTree::layout_all` / `UiTree::paint_all`)

This ordering is required when ViewCache subtree reuse is enabled.

To reduce call-site mistakes, `fret-ui` provides a small helper:

- `fret_ui::frame_pipeline::propagate_changes`

## Rationale

- ViewCache reuse decisions are made during mount. If invalidation is applied after mount, the reuse decision can be incorrect for that frame.
- Keeping the contract explicit makes it portable across platforms (native/web) and compatible with demo-driven development.

## Consequences

- Runners should converge on a single "golden path" for the frame pipeline.
- Diagnostics/inspection tooling can rely on a consistent sequence when explaining "why a subtree was reused".

## Non-goals

- This ADR does not define *how* model/global observation is collected.
- This ADR does not introduce a new caching unit beyond existing ViewCache boundaries.
