# P0 Root Hosting Rule - 2026-04-12

Status: accepted P0 teaching rule

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P0_FOOTGUN_AUDIT_2026-04-12.md`
- `P0_PROOF_BUDGET_RULE_2026-04-12.md`

## Question

What is the first-open teaching rule for `fret_imui::imui(...)` versus
`fret_imui::imui_vstack(...)`?

## Short answer

Use these rules:

1. If your IMUI content already lives under an explicit layout host, prefer
   `fret_imui::imui(cx, ...)`.
2. If you are mounting IMUI directly at the view root or under a non-layout parent, prefer
   `fret_imui::imui_vstack(cx.elements(), ...)`.

The reason is simple:

- `imui(...)` emits siblings directly,
- `imui_vstack(...)` adds the default stacked host,
- and the stacked host exists to avoid the common "all children overlap at `(0,0)`" footgun.

## Why this is a teaching rule, not a helper-growth trigger

The current audit already concluded that root-host ambiguity is real.
It did **not** conclude that the repo needs another broad helper pass.

This note therefore freezes the default explanation first:

- keep the behavior explicit,
- teach the two mounting shapes clearly,
- and do not treat root-host wording friction as automatic evidence for new public helper growth.

`imui_vstack(...)` is the explicit root-host bridge, not evidence that generic helper growth should
reopen.

## Current first-party evidence

The current repo already contains both shapes:

- nested under an explicit layout host:
  - `apps/fret-cookbook/examples/imui_action_basics.rs`
- root-hosted with the stacked bridge:
  - `apps/fret-examples/src/imui_hello_demo.rs`

This means the missing piece is not another helper.
The missing piece is that public docs should say which shape is for which situation.

## Public teaching consequence

The immediate-mode first-open path should now teach this in order:

1. start with the golden pair:
   - `apps/fret-cookbook/examples/imui_action_basics.rs`
   - `apps/fret-examples/src/imui_editor_proof_demo.rs`
2. read the mounting rule:
   - nested layout host -> `fret_imui::imui(cx, ...)`
   - root or non-layout parent -> `fret_imui::imui_vstack(cx.elements(), ...)`
3. treat `imui_hello_demo` as the small smoke/reference proof of the explicit root-hosted shape,
   not as the default first-contact path

## Decision

From this point forward:

1. public docs should explain the mounting rule explicitly,
2. root-host ambiguity should be treated as a teaching/documentation issue first,
3. and any later helper proposal must prove why this rule is not sufficient before reopening public
   helper growth.

## Immediate execution consequence

For P0, the next durable outcome is small and concrete:

- keep the generic/editor golden pair,
- keep `imui_hello_demo` as the explicit root-host smoke/reference surface,
- and lock the mounting rule in docs and source-policy gates.
