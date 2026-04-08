# Grid Track And Slot Placement Parity v1

Date: 2026-04-07  
Status: Active execution lane

## Motivation

Recent Card parity work exposed a deeper contract gap:

- the docs-path `card-demo` bugs were fixed locally,
- but the underlying `CardHeader` / `CardAction` composition still did not match the upstream
  source of truth,
- and the root cause was not just recipe code: Fret's public grid contract only expressed evenly
  sized repeat tracks, not explicit non-uniform tracks such as `1fr auto`, and its in-flow
  grid-item `Fill` mapping could still blow out the primary `fr` track instead of filling the
  resolved grid area.

The follow-on sibling audit after that slice exposed the next contract tier:

- explicit tracks alone are not enough for source-aligned shadcn grid slots,
- upstream `Alert` / `AlertDialog` / exact `CardAction` parity also depend on grid container/item
  alignment semantics such as `justify-items-start`, `place-items-center`, `self-start`, and
  `justify-self-end`,
- and the next pressure point after alignment was axis-specific gaps (`gap-x-*` / `gap-y-*`)
  rather than wider track vocabulary.

This lane exists to close that contract gap at the correct layer:

- mechanism in `crates/fret-ui`,
- recipe parity in `ecosystem/fret-ui-shadcn`,
- gallery/docs geometry proof in `apps/fret-ui-gallery`.

## Scope

### In scope

- Extend the runtime grid contract so declarative grid containers can express explicit per-track
  sizing needed by upstream shadcn layouts.
- Extend the runtime grid contract with the missing alignment and gap surfaces that the sibling
  audit proved are not recipe-local (`justify-items`, grid item `align-self`, grid item
  `justify-self`, row/column gaps).
- Rebuild `CardHeader` / `CardAction` on that contract instead of using a `justify-between` flex
  approximation.
- Audit nearby shadcn components that rely on similar slot placement semantics and record whether
  they need the same contract.
- Leave focused regression gates and evidence.

### Out of scope

- A full CSS Grid surface (`subgrid`, named lines, areas, auto-fit/auto-fill taxonomy).
- Reopening broad `shadcn-source-alignment-v1` scope.
- Recipe-only cosmetic tweaks that do not depend on the grid contract.

## Assumptions First

- Confident: current `GridProps` only expresses evenly sized repeat tracks.
  - Evidence: `crates/fret-ui/src/element.rs` (`GridProps { cols, rows }`) and
    `crates/fret-ui/src/layout/engine/flow.rs` / `crates/fret-ui/src/declarative/host_widget/measure.rs`
    both map grid templates through `taffy::style_helpers::evenly_sized_tracks(...)`.
  - Consequence if wrong: this lane would be recipe-only and should not change the runtime
    contract.
- Confident: upstream `CardHeader` depends on explicit slot geometry, not a generic horizontal row.
  - Evidence: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx` uses
    `grid-rows-[auto_auto]`, `has-data-[slot=card-action]:grid-cols-[1fr_auto]`, and
    `CardAction` uses `col-start-2 row-start-1 row-span-2`.
  - Consequence if wrong: the current `justify-between` implementation would already be
    semantically complete.
- Likely: Alert and AlertDialog expose the same class of parity risk because upstream source uses
  explicit tracks or slot-start/span placement.
  - Evidence: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert.tsx`,
    `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`.
  - Consequence if wrong: this lane can stop at Card and leave the audit note as informational.
- Confident: explicit tracks plus grid-item `Fill -> stretch` still do not cover all source-aligned
  slot families; sibling components also depend on grid container/item alignment semantics.
  - Evidence: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert.tsx` uses
    `justify-items-start`, `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx` uses
    `place-items-center` / `place-items-start`, and `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx`
    uses `self-start justify-self-end`.
  - Consequence if wrong: the remaining sibling drift would truly be recipe-only and should not
    widen the runtime contract again.
- Confident: separate row/column gap control is part of the minimal runtime contract for the
  sibling surfaces in scope.
  - Evidence: upstream `Alert` uses `gap-y-0.5` plus conditional `gap-x-3`, while
    `AlertDialogHeader` uses `gap-1.5` plus conditional `gap-x-6`.
  - Consequence if wrong: wrappers or recipe-local margins would remain sufficient and this lane
    would not need to widen beyond alignment.

## Must-Be-True Outcomes

- Fret grid containers can express explicit track lists with stable typed semantics, not stringly
  class translation.
- Fret grid containers can express inline-axis item alignment (`justify-items`) without conflating
  it with content distribution (`justify-content`).
- Fret grid items can express slot-local `align-self` / `justify-self` without pretending flex
  cross-axis alignment already covers grid.
- Fret grid containers can express independent row/column gaps for source-aligned docs surfaces
  such as `Alert` and `AlertDialogHeader`.
- Existing equal-track `GridProps` call sites keep working without migration churn.
- `CardHeader` without action still uses the source-aligned two-row grid baseline.
- `CardHeader` with action uses the upstream slot lane (`1fr auto`, action in column 2 spanning
  both header rows) and the action slot keeps the upstream self-alignment semantics.
- UI Gallery geometry proves the visible docs-path outcome instead of trusting structure alone.
- Similar slot-placement components are explicitly classified as:
  - already aligned,
  - blocked on the same mechanism,
  - or needing a follow-on slice.

## Source Of Truth

- Upstream component source:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert.tsx`
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/alert-dialog.tsx`
- Upstream docs-path example:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/card-demo.tsx`
  - `repo-ref/ui/apps/v4/content/docs/components/base/card.mdx`
- Current Fret implementation:
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/layout/engine/flow.rs`
  - `crates/fret-ui/src/declarative/host_widget/measure.rs`
  - `ecosystem/fret-ui-shadcn/src/card.rs`
  - `apps/fret-ui-gallery/src/ui/snippets/card/demo.rs`

## Design Direction

### Runtime contract

- Keep the existing `cols` / `rows` equal-track shorthand for stable callers.
- Add explicit track-list support for cases that need non-uniform tracks.
- Keep the explicit track vocabulary small:
  - `Auto`
  - fixed px
  - `fr`
- Map that vocabulary directly to Taffy's grid track sizing functions.
- Add the minimal grid-only alignment surfaces that the sibling audit proved necessary:
  - container `justify-items`,
  - item `align-self`,
  - item `justify-self`,
  - row/column gap overrides on top of the shared `gap` shorthand.
- For in-flow grid items, translate `Length::Fill` as grid-area stretch semantics rather than a
  raw percent size against the whole grid container, so slot children do not push `fr auto`
  layouts off-canvas.

### Recipe translation

- Make `CardHeader` always a grid-root translation, not a flex split special case.
- When a `CardAction` slot is present:
  - use explicit columns `1fr auto`,
  - use explicit rows `auto auto`,
  - place the action in column 2, row 1, spanning 2 rows,
  - keep the upstream `self-start` / `justify-self-end` slot semantics.
- Keep `Card` root width caller-owned and keep the earlier docs/demo fixes in place.

### Audit policy

- Card is the first proof surface.
- Alert and AlertDialog are the first follow-up audit targets because upstream source already shows
  the same semantic family plus the next missing alignment tier.

## Risks

- Public contract drift if docs/ADR text still implies "grid exists" while the typed vocabulary is
  narrower than upstream usage.
- False closure if explicit track lists are added but grid-item `Fill` still expands against the
  whole grid container, pushing `CardAction`-style slots outside the intended lane.
- False closure if explicit tracks and fill semantics land, but the runtime still cannot express
  `justify-items` / `justify-self` / grid `align-self`, forcing sibling recipes back into flex
  approximations.
- False confidence if we only assert structure and do not keep a geometry gate on the gallery demo.
- Overcorrecting into a large CSS-grid surface instead of landing the minimal typed track +
  alignment contract.
- Missing the next real blocker by treating `gap-x-*` / `gap-y-*` pressure as “just styling” when
  sibling recipes demonstrably depend on independent row/column gaps for source-aligned geometry.
