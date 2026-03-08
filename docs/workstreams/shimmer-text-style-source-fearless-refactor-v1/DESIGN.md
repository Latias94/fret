# Shimmer Text Style Source (Fearless Refactor v1)

Status: Draft (follow-on workstream note).

This document is **non-normative**. It captures the follow-on design work needed to close the
initial `PlanTitle` / `PlanDescription` streaming gap left open by ADR 0314, plus the remaining post-plan `Shimmer` audit.

Primary contract references:

- `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`
- `docs/adr/0315-shimmer-resolved-text-style-source-v1.md`

## Why this workstream exists

ADR 0314 made inherited text style real for passive text leaves.

That solved the main component-ecosystem pain:

- shadcn description family,
- AI supporting copy,
- mixed text/children slots,
- subtree-local typography defaults.

One important gap remains:

- after the `PlanTitle` / `PlanDescription` pilot closes, other visual text recipes that render
  their own text **and** paint an additional overlay still need classification between semantic
  subtree defaults and intentional explicit visual overrides.

`Shimmer` is the first pressure point.

`PlanTitle` and `PlanDescription` now use shared subtree typography scopes for both streaming and
non-streaming paths, but other `Shimmer` call sites still need the same decision.

This workstream now exists to keep the post-plan audit boring and prevent new manual typography
assembly from sneaking back in.

## Local evidence anchors

### Pilot closure and remaining gap

- `ecosystem/fret-ui-ai/src/elements/plan.rs`
  - `PlanTitle` streaming path
  - `PlanDescription` streaming path
  - both now consume subtree-resolved shimmer typography
- `ecosystem/fret-ui-ai/src/elements/shimmer.rs`
  - `Shimmer::text_style(...)` compatibility path remains
  - resolved passive-text bridge path now feeds both base `TextProps` and overlay canvas paint

### Existing migration wins we want to extend to streaming surfaces

- `ecosystem/fret-ui-shadcn/src/card.rs`
- `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`
- `ecosystem/fret-ui-shadcn/src/item.rs`
- `ecosystem/fret-ui-ai/src/elements/voice_selector.rs`
- `ecosystem/fret-ui-ai/src/elements/chain_of_thought.rs`

### Classified `Shimmer` call sites

- `ecosystem/fret-ui-ai/src/elements/reasoning.rs`
  - migrated to subtree-resolved semantic trigger typography
- `ecosystem/fret-ui-ai/src/elements/transcription.rs`
  - intentionally remains explicit because `text_style(...)` is a public authoring seam
- `ecosystem/fret-ui-ai/src/elements/terminal.rs`
  - migrated to subtree-resolved status-slot typography
- `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer_demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer_elements_demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/ai/shimmer_duration_demo.rs`
  - now audited as demo/documentation surfaces, with an inherited-style example added alongside explicit compatibility cases

## Problem statement

### What is missing

The missing piece is **not** another semantic preset helper.

The missing piece is a mechanism for custom visual-text recipes to obtain the same effective text
style / foreground that a passive text leaf would resolve under the current subtree.

The pilot used to be blocked because:

1. inherited text style is a runtime traversal concern for passive leaves,
2. `Shimmer` needs the resolved style early enough to size and paint the overlay text,
3. component authoring code did not have a public way to ask for “the current effective passive
   text style under this subtree”.

That mechanism gap is now closed for `Shimmer`, but the remaining call sites still need policy
classification about whether they should consume subtree defaults or stay on explicit visual text
ownership.

### Why `Shimmer` is special

`Shimmer` is not a plain passive text leaf.

It renders:

- a base text node,
- plus a custom canvas overlay that repaints the same glyph run with animated clipping.

That means it must keep these values in sync:

- resolved text style,
- resolved foreground / highlight colors,
- wrap / overflow constraints,
- measured baseline,
- overlay paint style.

If those diverge, the shimmer band stops matching the underlying glyphs.

### Why a component-only fix is insufficient

A component-only API like `Shimmer::text_style_refinement(...)` is not enough by itself.

It could describe a desired refinement, but it still would not solve the hard part:

- where does the component get the subtree's inherited text refinement / foreground,
- and at what stage does it resolve the final `TextStyle` used by both text layout and canvas paint?

That is a mechanism contract, not recipe policy.

## Scope / layering

### `crates/fret-ui`

Owns the mechanism for resolving effective passive text style / foreground for visual text recipes.

This is the hard contract surface.

### `ecosystem/fret-ui-kit`

May add ergonomics once the mechanism is stable, but it must not invent fake runtime state for this
problem.

### `ecosystem/fret-ui-ai`

Owns the `Shimmer` pilot migration and the recipe-level decision of which surfaces should consume the
new mechanism.

## Non-goals

This workstream does **not** aim to:

- introduce a general rich-text canvas authoring framework,
- migrate editors / inputs / document engines,
- remove explicit style overrides from `Shimmer` demos,
- or collapse all animated text recipes into one universal primitive.

## Invariants (must hold)

1. **Resolved style parity**
   - `Shimmer` must be able to use the same effective style / foreground a passive text leaf would
     receive under the same subtree.
2. **Base and overlay stay in lockstep**
   - text measurement, baseline, base text rendering, and overlay text rendering must all use the
     same resolved snapshot.
3. **Explicit override remains available**
   - callers must still be able to force a specific `TextStyle` when the recipe intentionally owns
     its typography.
4. **Policy stays out of `crates/fret-ui`**
   - the mechanism resolves style; AI / shadcn recipes decide whether a shimmer should look like a
     card title, description, or standalone demo headline.
5. **Migration should reduce call-site duplication**
   - once the mechanism exists, `PlanTitle` / `PlanDescription` streaming paths should stop manually
     rebuilding card typography.

## Options considered

### Option A: Keep `Shimmer::text_style(TextStyle)` as the long-term answer

Rejected as the end state.

Pros:

- zero mechanism work,
- current demos keep working,
- low immediate risk.

Cons:

- `PlanTitle` / `PlanDescription` stay split between streaming and non-streaming authoring paths,
- more AI surfaces will copy explicit card-description/card-title style assembly,
- the repo keeps teaching two answers for the same semantic copy.

### Option B: Add a component-local refinement API only

Rejected as insufficient.

Example shapes:

- `Shimmer::text_style_refinement(...)`
- `Shimmer::foreground(...)`
- `Shimmer::inherit_text_style(...)`

Why it is insufficient:

- it still does not provide late-bound access to the subtree's inherited refinement,
- it does not guarantee the overlay painter sees the same resolved style as passive text leaves,
- it risks re-implementing runtime resolution logic in `fret-ui-ai`.

### Option C: Add a mechanism-level resolved passive text source for visual recipes

Chosen direction.

Normative outcome:

- a custom visual-text recipe can ask for the effective passive text style / foreground that would
  apply under the current subtree,
- that resolved snapshot is stable enough to be shared across measure / base render / overlay paint,
- explicit overrides remain supported.

The exact API shape is intentionally left open in this workstream.

## Preferred direction

### Contract outcome

Fret should expose a mechanism-level way for custom visual-text recipes to resolve the same passive
text style contract already used by `TextProps` / `StyledTextProps` / `SelectableTextProps`.

This outcome may be implemented by one of several internal carriers, for example:

- a public resolver around the existing passive-text style resolution helpers,
- a runtime-owned resolved text snapshot available during layout / paint for special consumers,
- or a dedicated visual-text bridge that late-binds inherited style before custom canvas paint.

What is normative is the outcome, not the exact helper name.

### `Shimmer` migration target

Once the mechanism exists, `Shimmer` should support two style sources:

1. **Explicit**
   - current `.text_style(TextStyle)` compatibility path for demos / intentionally owned visual text.
2. **Subtree-resolved**
   - resolve effective style / foreground from inherited text-style + inherited foreground + theme
     defaults, then use that same snapshot for:
     - the base text node,
     - text measurement,
     - the overlay band paint.

### `PlanTitle` / `PlanDescription` migration target

After the `Shimmer` bridge lands:

- non-streaming path keeps using `CardTitle` / `CardDescription`,
- streaming path installs the same inherited scopes and lets `Shimmer` consume them,
- call sites stop assembling card title / card description `TextStyle` manually.

## Suggested implementation sequence

1. **Audit and contract lock**
   - classify all `Shimmer` call sites as:
     - must stay explicit,
     - should migrate to subtree-resolved,
     - or mixed / needs follow-up.
2. **Mechanism bridge in `crates/fret-ui`**
   - expose the minimum resolved passive text source needed by custom visual recipes.
3. **`Shimmer` compatibility bridge**
   - keep `.text_style(TextStyle)` working,
   - add subtree-resolved mode.
4. **Pilot migration**
   - migrate `PlanTitle` and `PlanDescription` streaming paths.
5. **Guardrails**
   - add tests proving resolved style parity and no overlay metric drift.

## Risks / review questions

- Can the resolved style be exposed without leaking runtime node internals into ecosystem code?
- Should the resolved source also include inherited foreground, or should foreground remain a
  separate input to visual-text recipes?
- Do we need a generic visual-text contract now, or is `Shimmer` enough as a pilot consumer?
- Can we keep the migration narrow enough that demos like `transcription` and `reasoning` retain
  explicit overrides without regressing?

## Exit criteria

This workstream is complete when:

- the repo has one boring answer for streaming `PlanTitle` / `PlanDescription`,
- `Shimmer` no longer forces semantic call sites to rebuild card typography manually,
- explicit style ownership remains available for demos / intentionally visual surfaces,
- and the behavior is covered by focused tests or diag gates.
