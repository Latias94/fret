# P0 Footgun Audit - 2026-04-12

Status: Historical reference (partially superseded by the later root-hosting and parity notes)

Status note (2026-04-21): this document remains useful as the original P0 footgun classification
pass, but the current shipped root-host guidance now lives in
`docs/workstreams/imui-editor-grade-product-closure-v1/P0_ROOT_HOSTING_RULE_2026-04-12.md`,
`docs/examples/README.md`, and `ecosystem/fret-imui/src/frontend.rs`. References below to
`imui_vstack(...)` or to `imui(...)` as bare sibling emission should be read as historical:
today `fret_imui::imui(...)` is the safe default stacked host, while `fret_imui::imui_raw(...)`
is the advanced explicit-layout seam.

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `P0_TEACHING_SURFACE_INVENTORY_2026-04-12.md`
- `docs/workstreams/imui-stack-fearless-refactor-v2/TEACHING_SURFACE_AUDIT_2026-03-31.md`

## Purpose

This note answers the remaining P0 question:

> after freezing the current golden pair, what still makes immediate authoring feel riskier or
> noisier than it should, and which of those problems are actually missing helpers versus
> documentation or proof-selection drift?

The answer is intentionally strict because the old `imui` lanes are already closed:

- do not turn every friction point into helper growth,
- do not widen the runtime to make the immediate path feel easier,
- and treat proof/golden-path ambiguity as a real product problem, not as "just docs."

## Audited evidence

- `ecosystem/fret-imui/src/frontend.rs`
- `docs/architecture.md`
- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `apps/fret-examples/src/imui_hello_demo.rs`
- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_floating_windows_demo.rs`
- `apps/fret-examples/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-examples/src/imui_node_graph_demo.rs`
- `apps/fret-examples/src/lib.rs`
- `docs/examples/README.md`

## A) Documentation / teaching footguns

### 1) Root embedding still has a real layout footgun

The `fret-imui` frontend already documents one of the biggest immediate authoring traps:

- `imui(...)` just emits siblings,
- `imui_vstack(...)` wraps them in a `Column`,
- and the wrapper exists specifically to avoid "all children overlap at `(0,0)`" when mounted
  under a non-layout parent.

Evidence:

- `ecosystem/fret-imui/src/frontend.rs:16`
- `ecosystem/fret-imui/src/frontend.rs:18`

The problem is not that the helper is missing. The problem is that the golden-path story still does
not foreground when to choose:

- nested `fret_imui::imui(cx, ...)`, versus
- root-hosted `fret_imui::imui_vstack(cx.elements(), ...)`.

Current evidence split:

- golden generic proof nests `fret_imui::imui(cx, ...)` under an explicit `Column`:
  `apps/fret-cookbook/examples/imui_action_basics.rs:130`
- multiple example surfaces still use `fret_imui::imui_vstack(cx.elements(), ...)` at the root:
  `apps/fret-examples/src/imui_hello_demo.rs:25`
  `apps/fret-examples/src/imui_response_signals_demo.rs:48`
  `apps/fret-examples/src/imui_floating_windows_demo.rs:38`
  `apps/fret-examples/src/imui_shadcn_adapter_demo.rs:43`

Classification:

- primary issue = teaching / default-path wording,
- not yet proof that a broad new helper family is needed.

### 2) Stable identity rules are real, but still under-taught on the golden path

The runtime and frontend make the identity model explicit:

- elements are move-only and identity-bearing,
- repeated structures should rebuild, not clone,
- dynamic collections should use explicit identity,
- and `for_each_unkeyed(...)` is only safe for order-stable static lists.

Evidence:

- `docs/architecture.md:146`
- `docs/architecture.md:154`
- `ecosystem/fret-imui/src/frontend.rs:92`
- `ecosystem/fret-imui/src/frontend.rs:94`

The problem is not missing API. The API already exists:

- `ui.id(...)`
- `ui.push_id(...)`
- `ui.for_each_keyed(...)`
- `ui.for_each_unkeyed(...)`

What is missing is a P0 explanation that connects those rules to the golden pair. Today:

- `imui_action_basics` is the right generic default proof, but it does not exercise keyed identity,
- the keyed identity story mostly shows up later in heavier proofs like `imui_editor_proof_demo`,
- and the first-contact docs do not yet teach "static list vs dynamic list" as part of the default
  immediate mental model.

Classification:

- documentation / teaching issue,
- not a helper gap.

### 3) The generic default path and the advanced/reference path still look too similar from a distance

`imui_hello_demo` is tiny and easy to copy, but it teaches:

- the advanced prelude,
- explicit `update_in(...)`,
- explicit `value_in(...)`,
- and root hosting through `imui_vstack(cx.elements(), ...)`.

Evidence:

- `apps/fret-examples/src/imui_hello_demo.rs:25`
- `apps/fret-examples/src/imui_hello_demo.rs:33`
- `apps/fret-examples/src/imui_hello_demo.rs:47`

That is not wrong code. It is just not the best first-contact teaching surface now that
`imui_action_basics` exists on the current app lane.

Classification:

- proof-selection and teaching problem,
- not a missing helper.

## B) Proof-selection footguns

### 1) Source-policy gates still group golden and reference proofs under one umbrella

The examples crate currently has a valuable source-policy test, but the scope is still too broad:

- `first_party_imui_examples_keep_current_facade_teaching_surface`

That gate checks that multiple examples keep using the current facade and do not reintroduce
deleted names, but it does **not** distinguish:

- default/golden proofs,
- reference proofs,
- compatibility-only proofs.

Evidence:

- `apps/fret-examples/src/lib.rs:1406`
- `apps/fret-examples/src/lib.rs:1418`
- `apps/fret-examples/src/lib.rs:1430`
- `apps/fret-examples/src/lib.rs:1443`
- `apps/fret-examples/src/lib.rs:1456`
- `apps/fret-examples/src/lib.rs:1469`

This means the gate protects boundary hygiene, but not product-message clarity.

Classification:

- proof-selection/gate-shape issue.

### 2) Public example docs classify one advanced immediate proof, but not the golden pair

The public examples README already does the right thing for `imui_floating_windows_demo`:

- it explicitly calls it an overlap/floating proof surface,
- and says it is not the default app lane.

Evidence:

- `docs/examples/README.md:189`

But the same doc set does not yet name the current P0 golden pair:

- `apps/fret-cookbook/examples/imui_action_basics.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`

So a user can still discover immediate-mode examples mostly through demo names or bin lists instead
of through an explicit "start here, then go here" path.

Classification:

- proof-selection / docs-entrypoint issue.

## C) Genuine missing-helper candidate

### 1) There is one credible helper-shape candidate: an app-lane root host helper

After auditing the current examples, only one helper-shape gap appears repeatedly enough to deserve
follow-on consideration:

- a root-host helper that makes the app-lane immediate entry shape obvious without forcing the
  author to reason about `cx.elements()` plus `imui` vs `imui_vstack` up front.

Why this one is credible:

- multiple example surfaces use `fret_imui::imui_vstack(cx.elements(), ...)` at the root,
- the generic golden proof uses nested `fret_imui::imui(cx, ...)` under an explicit layout host,
- and the existing frontend docs explicitly call out the overlap-at-`(0,0)` failure mode.

Evidence:

- `ecosystem/fret-imui/src/frontend.rs:16`
- `apps/fret-cookbook/examples/imui_action_basics.rs:130`
- `apps/fret-examples/src/imui_hello_demo.rs:25`

What this does **not** justify:

- a new generic helper pack,
- another editor-composite promotion pass,
- or widening `fret-ui-kit::imui` with more nouns just because root hosting is a bit confusing.

If a helper follow-on is ever opened here, it should stay narrow:

- one root-host shape,
- one clear default layout posture,
- and one proof that it reduces golden-path ambiguity across the chosen golden pair.

### 2) Most explicit state-plumbing in reference proofs is intentional, not helper debt

Examples like `imui_response_signals_demo` still perform explicit writes through
`ui.cx_mut().app.models_mut()` and explicit reads through `value_in(...)`.

Evidence:

- `apps/fret-examples/src/imui_response_signals_demo.rs`
- `apps/fret-examples/src/imui_hello_demo.rs:33`

That looks noisy, but it is not the same class of problem as the root-host ambiguity:

- these are response/behavior proofs,
- the explicit mutation is often the point,
- and inventing helper sugar here would risk reintroducing generic helper-growth pressure that the
  earlier closeout lanes deliberately avoided.

Classification:

- not a missing-helper gap,
- leave as explicit reference-path code unless repeated golden-path evidence says otherwise.

## Verdict

The remaining P0 footguns are dominated by:

1. documentation / teaching gaps,
2. proof-selection and gate-shape ambiguity,
3. with only one credible helper candidate:
   a narrow app-lane root-host helper.

This is the key result:

> Fret does not currently need another broad immediate helper-growth pass.
> It first needs a clearer golden path and sharper distinction between default proofs and reference
> proofs.

## Immediate execution consequence

From this point forward:

1. treat root-host ambiguity (`imui` vs `imui_vstack` + `cx.elements()`) as the only credible
   helper-shape follow-on candidate,
2. treat stable-identity teaching as a documentation problem unless new code evidence says the
   helper surface itself is insufficient,
3. promote source-policy gates and public docs that distinguish:
   - golden/default,
   - reference/product-validation,
   - compatibility-only,
4. do not reopen generic `imui` helper growth for response/state-plumbing noise alone.
