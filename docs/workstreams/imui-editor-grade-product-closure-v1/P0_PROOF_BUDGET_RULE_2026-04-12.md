# P0 Proof Budget Rule - 2026-04-12

Status: accepted P0 decision

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`
- `P0_DEMOTE_DELETE_PLAN_2026-04-12.md`

## Question

What proof budget must exist before future `fret-ui-kit::imui` public helper widening is allowed
to reopen?

## Short answer

Any future `fret-ui-kit::imui` public helper widening must name at least two real first-party proof
surfaces that both need the helper and cannot reasonably stay explicit.

For P0, the current minimum proof budget is the frozen immediate-mode golden pair:

1. Generic/default proof:
   - `apps/fret-cookbook/examples/imui_action_basics.rs`
2. Editor-grade proof:
   - `apps/fret-examples/src/imui_editor_proof_demo.rs`

Reference, advanced, or compatibility-only surfaces do not count by themselves.

## Why this rule exists

The earlier `imui` lanes already closed the broad helper-growth question.

The current P0 audit does not show a large missing-helper backlog. It shows:

- default-path teaching ambiguity,
- proof-surface selection drift,
- and one narrow credible helper candidate:
  an app-lane root-host helper.

Without an explicit proof budget rule, one noisy reference demo could easily be mistaken for
"evidence that `fret-ui-kit::imui` needs more public nouns."

That is exactly the failure mode this note closes.

## What counts as a qualifying proof surface

A qualifying proof surface should be all of:

1. first-party and actively maintained,
2. app-facing or author-facing enough to represent the taught public path,
3. materially different in role or ownership so the same missing helper is not just duplicated in
   one narrow shape,
4. and strong enough to show that keeping the code explicit on both surfaces would be unreasonable.

Supporting implementation evidence still matters, but it is not a substitute for the proof budget.
For example:

- `ecosystem/fret-ui-kit/src/imui.rs`,
- `ecosystem/fret-ui-editor/src/imui.rs`,
- and lower-level tests

can support a widening proposal, but they do not replace the need for two real first-party proof
surfaces.

## What does not count on its own

These surfaces remain valuable, but they cannot justify widening by themselves:

- reference/smoke:
  - `apps/fret-examples/src/imui_hello_demo.rs`
- reference/product-validation:
  - `apps/fret-examples/src/imui_response_signals_demo.rs`
  - `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- advanced/reference:
  - `apps/fret-examples/src/imui_floating_windows_demo.rs`
- compatibility-only:
  - `apps/fret-examples/src/imui_node_graph_demo.rs`

These can strengthen a case after the minimum proof budget already exists, but they are not enough
to reopen helper growth alone.

## Current implication for P0

The current minimum proof budget already exists, but it does **not** automatically authorize
widening.

It means a future proposal is only eligible for review when it can name:

1. the missing helper or noun precisely,
2. why both golden-pair proof surfaces need it,
3. why explicit code on both surfaces is no longer the better choice,
4. and which bounded gate package will prove the change.

If a proposal only improves one surface, or only makes a reference surface nicer, keep the code
explicit and treat the pressure as teaching/reference maintenance instead of helper growth.

## Decision

From this point forward:

1. no single immediate-mode proof surface can reopen `fret-ui-kit::imui` helper growth,
2. the current minimum proof budget is:
   `apps/fret-cookbook/examples/imui_action_basics.rs` +
   `apps/fret-examples/src/imui_editor_proof_demo.rs`,
3. reference, advanced, and compatibility-only surfaces may support a case but cannot justify it on
   their own,
4. and any widening proposal must arrive with a named gate/evidence package before review.

## Immediate execution consequence

For this lane, the practical effect is simple:

- keep P0 focused on teaching-surface closure,
- keep the current golden pair as the minimum shared evidence budget,
- and treat the narrow app-lane root-host helper as the only credible follow-on candidate until
  stronger multi-surface pressure appears.
