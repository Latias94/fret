# Closeout Audit — 2026-03-20

This audit records the final closeout read for the view-locals authoring v1 lane.

Goal:

- verify whether the repo still has an active default app-lane backlog around grouped
  view-owned `LocalState<T>` organization,
- separate the landed organization rule from rejected helper/API growth,
- and decide whether this lane should remain active or become historical maintenance evidence.

## Audited evidence

Core workstream docs:

- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/view-locals-authoring-fearless-refactor-v1/TODO.md`

Default-path docs/templates:

- `docs/README.md`
- `docs/roadmap.md`
- `docs/authoring-golden-path-v2.md`
- `docs/examples/todo-app-golden-path.md`
- `ecosystem/fret/README.md`
- `apps/fretboard/src/scaffold/templates.rs`

Implementation / proof anchors:

- `apps/fret-examples/src/todo_demo.rs`
- `apps/fret-examples/src/simple_todo_demo.rs`
- `apps/fret-cookbook/examples/simple_todo.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`
- `apps/fret-cookbook/examples/form_basics.rs`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-cookbook/src/lib.rs`

Validation run used for closeout:

- `cargo nextest run -p fret-examples --lib todo_demo_prefers_default_app_surface simple_todo_demo_prefers_default_app_surface canonical_default_app_examples_stay_local_state_first`
- `cargo nextest run -p fret-cookbook --lib onboarding_examples_use_the_new_app_surface migrated_basics_examples_use_the_new_app_surface`
- `cargo nextest run -p fretboard todo_template_uses_default_authoring_dialect simple_todo_template_has_low_adapter_noise_and_no_query_selector template_readmes_capture_authoring_guidance`
- `git diff --check`
- `python3 .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols`

## Findings

### 1. Default app-facing proof surfaces now teach one coherent grouped-locals organization rule

The canonical first-party proof set no longer teaches the old forwarding-helper shape:

- `todo_demo` and `simple_todo_demo` now use `TodoLocals::new(cx)` plus
  `locals.bind_actions(cx)`,
- cookbook `simple_todo` and `simple_todo_v2_target` teach the same shape,
- and the generated `simple-todo` / `todo` templates now emit the same organization rule instead
  of `bind_todo_actions(...)` free functions that forward several local handles explicitly.

Just as important, the bundled shape stays transparent:

- reads are still ordinary `LocalState<T>` reads such as `locals.todos.layout_value(cx)`,
- writes still happen through the existing `cx.actions()` APIs,
- and row payload mutation stays on the already-closed row-write posture.

Conclusion:

- the core default-path migration goal of this lane is landed.

### 2. The lane is no longer Todo-only

This lane was only worth keeping if the organization rule survived at least one non-Todo
surface.

That proof now exists in `form_basics`:

- the example now groups `name`, `email`, and `error` into `FormBasicsLocals`,
- action wiring and command availability stay inside that bundle,
- and the result is still clearly app-lane code rather than a hidden runtime abstraction.

This matters because it shows the rule is not “special casing Todo demos.” It generalizes to the
broader default app lane whenever one view owns several related local slots.

Conclusion:

- the lane clears its own non-Todo promotion bar.

### 3. The correct solution was organizational, not API growth

The most important negative finding is what did **not** land:

- no new `cx.actions()` helper family,
- no new `LocalState<T>` helper family,
- no prelude widening,
- no storage-contract redesign,
- and no router/query reopen.

Instead, the repo now productizes a small first-party organization rule:

- one or two trivial locals may stay inline,
- once a view owns several related locals, prefer a small `*Locals` bundle,
- and use optional `bind_actions(&self, cx)` only when that grouping already exists.

That is exactly the narrow contract this lane needed to close on.

Conclusion:

- this folder closes on a no-new-API verdict.

### 4. Docs, scaffold guidance, and source-policy gates now agree on the same rule

The closeout is not just a code patch:

- `docs/authoring-golden-path-v2.md` now teaches the `1-2 inline / 3+ bundle` rule,
- `docs/examples/todo-app-golden-path.md` and `ecosystem/fret/README.md` teach the same grouped
  locals posture,
- scaffold README text now names `*Locals` and optional `bind_actions(&self, cx)`,
- and the source-policy tests in `apps/fret-examples`, `apps/fret-cookbook`, and `fretboard`
  guard the migrated shape directly.

An incidental stale assertion in `apps/fret-cookbook/src/lib.rs` around `query_basics` imports was
also corrected during validation. That was source-gate drift, not a reopened query design question.

Conclusion:

- the lane no longer owns an active examples/docs/templates convergence backlog.

### 5. What remains is maintenance only

After closeout, the remaining useful work is narrow:

1. keep docs/templates/source-policy tests synchronized with the shipped grouped-locals rule,
2. keep the “one or two trivial locals may stay inline” nuance intact,
3. reopen only if fresh cross-surface evidence shows a new repeated organization problem that this
   rule does not handle.

What does **not** belong here anymore:

- another helper-growth pass,
- reopening storage-model design from authoring discomfort alone,
- or forcing every view onto a bundle even when inline locals stay clearer.

Conclusion:

- this workstream no longer owns an active implementation queue.

## Decision from this audit

Treat `view-locals-authoring-fearless-refactor-v1` as:

- closed for the current grouped view-owned locals organization question,
- maintenance/historical evidence by default,
- and reopenable only through fresh cross-surface evidence that the shipped `1-2 inline / 3+ bundle`
  rule is no longer sufficient.

## Immediate execution consequence

From this point forward:

1. keep one or two trivial `LocalState<T>` slots inline when that is clearer,
2. prefer a small `*Locals` bundle once a view owns several related local slots or keeps threading
   them through helpers,
3. treat `bind_actions(&self, cx)` as an optional organization tool, not as a new framework
   abstraction,
4. keep reads/writes explicit through `LocalState<T>` and the existing `cx.actions()` APIs,
5. do not reopen selector/query/router/storage-model questions from this lane unless fresh
   evidence explicitly exceeds the current closeout.
