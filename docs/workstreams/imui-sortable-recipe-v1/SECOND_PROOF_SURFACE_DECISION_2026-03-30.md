# Second Proof Surface Decision — 2026-03-30

Status: accepted v1 decision

Related:

- `docs/workstreams/imui-sortable-recipe-v1/DESIGN.md`
- `docs/workstreams/imui-sortable-recipe-v1/TODO.md`
- `docs/workstreams/imui-sortable-recipe-v1/MILESTONES.md`
- `ecosystem/fret-ui-kit/src/recipes/imui_sortable.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `ecosystem/fret-imui/src/tests/interaction.rs`

## Question

Does `imui-sortable-recipe-v1` need a second first-party demo/proof surface before the current
vertical row recipe can be considered stable enough for v1?

## Short answer

No.

The current v1 contract already has two materially different evidence surfaces:

1. an app-facing tree/outliner proof in
   `apps/fret-examples/src/imui_editor_proof_demo.rs`
2. a flat-row real pointer interaction gate in
   `ecosystem/fret-imui/src/tests/interaction.rs`

That is enough for the current scope:

- single-list vertical reorder,
- response-driven row integration,
- app-owned final mutation,
- no multi-container transfer,
- no auto-scroll,
- no shell/workspace choreography.

## Why the current evidence is sufficient

The current recipe contract is intentionally narrow.

It only claims to package:

- row-level `drag_source(...)` + `drop_target::<T>(...)` wiring,
- vertical midpoint insertion-side derivation,
- and a minimal app-owned reorder helper for stable keys.

The two existing evidence surfaces already prove that this contract is not accidentally tied to one
specific authoring shape:

- the example proof uses `tree_node_with_options(...)` rows inside a richer editor-grade outliner,
- the interaction gate uses flat button rows with direct pointer-event verification.

So the recipe is already demonstrated across:

- hierarchical tree-node style rows,
- flat list/button style rows,
- app-facing proof/demo usage,
- and lower-level interaction-gate usage.

## Why we are not adding another demo right now

Adding another demo now would increase surface area without increasing decision quality very much.

The current missing evidence is not "another vertical single-list example."
The current missing evidence, if the contract is widened later, would be one of:

- multi-container transfer,
- auto-scroll during drag,
- richer collision policy,
- or shell/workspace-specific reorder behavior.

Those are different contracts, not just more copies of the same one-list recipe.

## Decision

For `imui-sortable-recipe-v1`:

- treat the current app proof plus interaction gate as sufficient second-surface evidence for the
  v1 vertical row recipe,
- do not add another first-party demo before closeout unless the recipe contract widens,
- and defer any further proof-surface expansion until there is new scope pressure beyond the
  current single-list contract.

## Immediate consequence

From this point forward:

1. keep the current v1 contract narrow,
2. keep insertion-side math recipe-local until a second shared extraction target appears,
3. do not widen the recipe toward multi-container or shell semantics without new evidence,
4. treat the next reopen trigger as a new contract question, not as a lack of proof duplication.
