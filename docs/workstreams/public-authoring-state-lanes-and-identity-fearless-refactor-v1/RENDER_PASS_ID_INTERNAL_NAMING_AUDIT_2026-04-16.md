# render_pass_id Internal Naming Audit — 2026-04-16

Status: Frozen

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/tests/identity.rs`

## Scope

Close the last open M1 diagnostics-posture question:

- should the internal `render_pass_id` field keep its current name,
- or should this lane spend another internal-only rename step to avoid GPU-loaded wording?

This note does not reopen runtime substrate convergence, public diagnostics API design, or any
user-facing render terminology.

## Assumptions-first checkpoint

1. The real contract question is privacy and semantics, not the spelling of one private field.
   Confidence: Confident.
   Evidence: `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`,
   `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`.
2. `render_pass_id` is already fully internal to kernel-owned repeated-call diagnostics.
   Confidence: Confident.
   Evidence: `crates/fret-ui/src/elements/cx.rs`,
   `crates/fret-ui/src/declarative/tests/identity.rs`.
3. A standalone rename only makes sense if the current spelling still leaks into user-facing
   authoring or materially obscures the implementation.
   Confidence: Likely.
   Evidence: `crates/fret-ui/src/elements/cx.rs`,
   `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`.

## Findings

### 1) The field no longer defines the public concept

The public-facing diagnostic concept is already `render evaluation`, not `render pass`.

That is visible in the only public-ish internal hook name that remains:

- `ElementContext::note_repeated_call_in_render_evaluation_at(...)`

and in the surrounding debug-only storage type:

- `RenderEvaluationCallsiteDiagnostics`
- `last_render_pass_id`
- `calls_in_render_pass`

In other words, the meaningful semantics are already carried by the function/type names that sit
around the private counter, not by teaching `render_pass_id` itself as a user-facing term.

### 2) The current spelling is private enough that renaming it would be churn without contract gain

Current evidence shows:

- no public getter exists,
- no doc teaches `render_pass_id` as an authoring primitive,
- source-policy tests explicitly forbid exposing it as a public method.

That means a rename from `render_pass_id` to something like `render_evaluation_id` would only
shuffle internal debug bookkeeping names without changing the actual contract:

- evaluation-boundary diagnostics stay internal,
- keyed identity remains the real user-facing rule,
- and facade code still reuses kernel-owned bookkeeping instead of forking it.

### 3) The correct verdict is “keep as-is unless a deeper diagnostics rewrite replaces it”

This lane should not create a cosmetic internal rename work item just because the old field name
still contains renderer-adjacent vocabulary.

The correct closure is narrower:

- keep the field private,
- keep the public contract phrased in terms of render evaluation boundaries,
- and only revisit the name if a later internal diagnostics refactor changes the underlying
  bookkeeping model anyway.

That avoids reopening M1 with zero user-facing benefit and keeps the evidence honest about what the
framework actually ships today.

## Evidence

- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/tests/identity.rs`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MIGRATION_MATRIX.md`

## Gate commands

- `cargo check -p fret-ui --tests`
- `cargo nextest run -p fret-ui element_context_identity_docs_classify_component_internal_lane`

## Outcome

The M1 diagnostics-posture verdict is now frozen:

1. `render_pass_id` stays a private internal bookkeeping name.
2. The public contract is render-evaluation diagnostics, not a public render-pass token.
3. No standalone rename follow-on is needed; revisit only if a deeper diagnostics rewrite replaces
   the mechanism itself.
