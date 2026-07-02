---
type: Work Progress
title: Phase 2 U11 Public mutation and toast wrappers
tags: fret,phase2,u11,appui,mutation,toast,public-facade
timestamp: 2026-07-02
related_plan: docs/plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md
git_branch: feat/ui-framework-phase2-refactor
---

# Phase 2 U11 Public Mutation And Toast Wrappers

## Summary

Phase 2 U11 adds the narrow public `AppUi` facade needed by the mutation cookbook and the future
mutation starter. Public app code can now submit/retry mutations, project terminal mutation results
into local state, and emit shadcn/Sonner success/error toasts without importing `fret_ui`,
`UiActionHostAdapter`, or naming `fret_runtime::ModelStore`.

The slice does not wrap the entire mutation/query runtime. Query invalidation remains on the
existing `cx.data().invalidate_query*` helpers, mutation lifecycle state remains on
`MutationHandle::read_layout(cx)`, and raw runtime escape hatches stay explicit.

## Changes

- `cx.actions().mutation_submit::<A, _, _>(...)` registers typed submit actions that build mutation
  input through `LocalStateTxn`.
- `cx.actions().mutation_retry_last::<A, _, _>(...)` registers explicit retry actions with a small
  local-state projection hook.
- `cx.data().update_locals_after_mutation_completion(...)` projects fresh terminal mutation state
  into local state without exposing `ModelStore`.
- `cx.effects().toast_success(...)` and `cx.effects().toast_error(...)` bridge to shadcn/Sonner
  without exposing `UiActionHostAdapter`.
- The mutation toast feedback cookbook now uses those public wrappers and no longer contains
  `UiActionHostAdapter`, direct `handle.submit(models, ...)`, direct `handle.retry_last(models, ...)`,
  or `fret_runtime::ModelStore`.
- `MutationHandle` and `MutationRuntimeHandle` now have manual `Clone` impls so handle cloning does
  not require mutation input/output payload types to implement `Clone`.

## Verification

Focused red/green evidence:

- Cookbook source-policy assertions were first inverted to require the new wrappers and forbid
  `UiActionHostAdapter` / `ModelStore`.
- The initial run failed because the wrappers did not exist and the old Sonner bridge required a
  mutable host adapter.
- After implementation, the focused facade and cookbook tests passed.

Verification passed before commit:

- `cargo check -p fret --features shadcn,state-mutation,state-query`
- `cargo nextest run -p fret view::tests::grouped_authoring_surfaces_replace_flat_app_ui_helpers --features shadcn,state-mutation,state-query --no-fail-fast`
- `cargo nextest run -p fret-cookbook --lib mutation_toast_feedback_example_keeps_submit_and_feedback_projection_split --features cookbook-mutation --no-fail-fast`
- `cargo check -p fret-cookbook --example mutation_toast_feedback_basics --features cookbook-mutation`
- `cargo nextest run -p fret-mutation mutation_handle_clone_does_not_require_input_or_output_clone --no-fail-fast`
- `cargo fmt --all --check`
- `python3 tools/check_surface_policy.py`
- `python3 tools/check_consumption_profiles.py`
- `python3 tools/check_execution_surface.py`

## Next Action

Continue to U12 by generating the mutation-workbench starter from these wrappers. Keep the generated
source on `fret::app::prelude::*`, explicit `fret::mutation` nouns, and shadcn recipe imports only.

# Citations

- [Phase 2 plan](../../../plans/2026-07-02-001-refactor-ui-framework-phase2-plan.md)
- [App action wrappers](../../../../ecosystem/fret/src/view/actions.rs)
- [App data wrappers](../../../../ecosystem/fret/src/view/data.rs)
- [App effect wrappers](../../../../ecosystem/fret/src/view/effects.rs)
- [Mutation cookbook](../../../../apps/fret-cookbook/examples/mutation_toast_feedback_basics.rs)
