# P0 Stable Identity Rule - 2026-04-12

Status: accepted P0 teaching rule

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`
- `P0_ROOT_HOSTING_RULE_2026-04-12.md`
- `P0_PROOF_BUDGET_RULE_2026-04-12.md`

## Question

What is the first-open teaching rule for stable identity on the immediate-mode lane?

## Short answer

Use these rules:

1. For static lists whose order never changes, `ui.for_each_unkeyed(...)` is acceptable.
2. For dynamic collections that can insert, remove, reorder, or preserve per-row state, prefer
   `ui.for_each_keyed(...)` or wrap each row in `ui.id(key, ...)`.
3. Rebuild UI rows each frame; do not treat elements as cloneable reusable values.

The point is not "more ceremony." The point is preserving cross-frame state on the intended row.

## Why this is a teaching rule, not a missing-helper signal

The runtime and the `fret-imui` frontend already expose the right identity model:

- element state survives by stable identity,
- move-only element values are intentional,
- and IMUI already provides `ui.id(...)`, `ui.push_id(...)`, `ui.for_each_keyed(...)`, and
  `ui.for_each_unkeyed(...)`.

So the current gap is not API absence.
The current gap is that the first-open path did not explicitly say:

- static list -> unkeyed is fine,
- dynamic list -> key it.

This note closes that wording gap without reopening helper growth.

## Current first-party evidence

The current golden pair already covers the right boundary:

- `apps/fret-cookbook/examples/imui_action_basics.rs`
  - good generic/default immediate proof,
  - but it does not need dynamic row identity yet
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
  - heavier editor-grade proof,
  - already uses explicit `ui.id(...)` where stable panel identity matters

That means the teaching rule should not wait for a new example.
It should explain how to read the existing pair.

## Public teaching consequence

The immediate-mode first-open path should now teach this in order:

1. start with the golden pair:
   - `apps/fret-cookbook/examples/imui_action_basics.rs`
   - `apps/fret-examples/src/imui_editor_proof_demo.rs`
2. apply the identity rule:
   - static/order-stable list -> `ui.for_each_unkeyed(...)`
   - dynamic/reorderable/per-row-state list -> `ui.for_each_keyed(...)` or `ui.id(key, ...)`
3. treat explicit keyed identity as part of the default mental model for dynamic IMUI, not as an
   advanced escape hatch

## Decision

From this point forward:

1. public docs should explain the static-vs-dynamic identity split explicitly,
2. dynamic IMUI collections should be taught with explicit identity as the default posture,
3. and future helper proposals must not use identity wording drift as a substitute for real
   cross-surface evidence.

## Immediate execution consequence

For P0, the durable outcome is small and concrete:

- keep `imui_action_basics` as the generic/default proof,
- keep `imui_editor_proof_demo` as the heavier proof where explicit stable identity already shows
  up,
- and lock the identity rule in docs and source-policy gates rather than inventing new helper
  surface.
