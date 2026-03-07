# ADR 0315 — Shimmer Resolved Text Style Source v1

Status: Accepted.

## Context

ADR 0314 introduced inherited text-style cascade for passive text leaves.

That contract covers:

- `TextProps`
- `StyledTextProps`
- `SelectableTextProps`

and resolves passive text as:

1. explicit leaf style,
2. inherited subtree refinement,
3. theme default.

This is enough for ordinary text surfaces, but not for visual text recipes that:

- render a base text node,
- then paint additional text-aligned effects in a custom canvas layer,
- and therefore need the same resolved style snapshot for measure, baseline, and paint.

`ecosystem/fret-ui-ai/src/elements/shimmer.rs` is the first pressure point.

Today `Shimmer` solves this by owning an explicit `TextStyle` input (`.text_style(...)`) and using
that style for both:

- the base `TextProps` node,
- and the overlay canvas paint.

This keeps the effect visually aligned, but it also forces semantic call sites such as
`PlanTitle` / `PlanDescription` streaming paths to rebuild card title / card description typography
locally instead of consuming subtree-local inherited text style.

The missing capability is not another preset helper. The missing capability is a mechanism-level way
for custom visual-text recipes to consume the same effective passive text style / foreground a
passive text leaf would resolve under the same subtree.

## Decision

### 1) Introduce a mechanism-level resolved passive text source for visual-text recipes

Fret will define a mechanism-level contract that allows a custom visual-text recipe to consume the
same effective passive text style / foreground that would apply to a passive text leaf under the
current subtree.

This contract is for components like `Shimmer` that must keep base text and custom paint in sync.

### 2) The resolved source follows the passive text precedence contract

The resolved style / foreground used by a visual-text recipe must follow the same precedence as
passive text:

1. explicit recipe override,
2. inherited subtree text-style / foreground,
3. theme default.

This preserves compatibility with explicit demo / override use cases while letting semantic call
sites rely on subtree-installed defaults.

### 3) Resolution must be late-bound relative to subtree inheritance

Authoring-time component code does not currently have enough information to resolve inherited text
style for custom visual painters.

Therefore the contract must resolve late enough that subtree inheritance is already known.

The exact internal carrier is not fixed by this ADR. Acceptable implementations include a public
resolver bridge, a runtime-owned resolved snapshot, or another equivalent mechanism.

What is normative is the outcome:

- visual-text recipes can use the same effective text style / foreground as passive text leaves,
- without re-implementing the runtime cascade in ecosystem code.

### 4) `Shimmer` is the pilot consumer

`Shimmer` will be the first consumer of this contract.

During migration it must keep the explicit `.text_style(TextStyle)` path working, but it should also
support a subtree-resolved mode once the mechanism exists.

### 5) Scope is intentionally narrow

This ADR does **not** introduce a general custom text-paint framework.

It only covers the minimum mechanism needed for passive-text-like visual recipes whose own paint
must match passive text resolution.

Out of scope for v1:

- editors / text inputs,
- general rich-text canvas composition,
- arbitrary per-span effect pipelines,
- or recipe policy such as “card description” / “muted supporting copy”.

Those policies remain in ecosystem crates.

## Consequences

### Positive

- Streaming semantic surfaces such as `PlanTitle` / `PlanDescription` can converge on the same
  typography contract as their non-streaming counterparts.
- `Shimmer` keeps visual correctness without forcing semantic call sites to rebuild text styles.
- The repo gets a principled answer for future custom visual-text recipes.

### Costs / risks

- The mechanism boundary is deeper than a helper-only change because inherited text style is not an
  authoring-time context today.
- We must avoid leaking runtime node internals into general ecosystem authoring APIs.
- The compatibility path for explicit `.text_style(...)` must remain intact during migration.

## Implementation plan (tracked)

Primary workstream:

- `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/TODO.md`

Recommended sequence:

1. Lock the call-site inventory and contract gap.
2. Add the minimum resolved passive text source in `crates/fret-ui`.
3. Teach `Shimmer` to consume the new source while retaining explicit overrides.
4. Migrate `PlanTitle` / `PlanDescription` streaming paths.
5. Add focused tests / diag gates and remove redundant manual card typography assembly.

## Initial evidence anchors

### Pilot implementation evidence

- `crates/fret-ui/src/canvas.rs`
- `crates/fret-ui/src/declarative/tests/canvas.rs`
- `ecosystem/fret-ui-ai/src/elements/shimmer.rs`
- `ecosystem/fret-ui-ai/src/elements/plan.rs`

### Remaining audit items

- `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer*.rs`

### Existing passive text contract

- `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`
- `crates/fret-ui/src/text/props.rs`
- `crates/fret-ui/src/declarative/host_widget/{layout,measure,paint}.rs`

### Existing ecosystem migration evidence

- `ecosystem/fret-ui-shadcn/src/card.rs`
- `ecosystem/fret-ui-ai/src/elements/voice_selector.rs`
- `ecosystem/fret-ui-ai/src/elements/chain_of_thought.rs`
