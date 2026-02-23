# Length percentage semantics v1 — Milestones

## Overview

Milestones are ordered by “blast radius vs ROI”:

- close the mechanism semantics first (so recipes don’t need workarounds)
- expand the authoring language incrementally (min/max → spacing → positioning)
- migrate ecosystem components only when the mechanism closure is stable

Each milestone must end with at least one regression gate.

## M0 — Baseline percent sizing semantics (done)

**Goal**

Support percent sizing for `size` and `flex-basis` without collapse under intrinsic measurement.

**Exit criteria**

- `Length::Fraction` exists and is used by authoring shorthands.
- `Fill`/`Fraction` resolve to px only under definite available space; otherwise behave like `auto`.
- Wrapper chains promote percent descendants so percent can resolve under a definite containing block.
- At least two unit tests are green under `cargo nextest`.

## M1 — Percent sizing for min/max constraints

**Goal**

Allow constraints like “max-width: 50%” without requiring per-component viewport math.

**Exit criteria**

- `min_*` / `max_*` accept percent/fraction in authoring.
- Declarative bridge resolves them with the v1 rule.
- A unit test proves constraints do not collapse under intrinsic measurement.

## M2 — Percent sizing for spacing (padding + gap)

**Goal**

Enable web-like “padding: 5%” and percent gaps for layout-driven recipes.

**Exit criteria**

- `padding` and `gap` have a definite-only percent-capable representation.
- Authoring shorthands exist (`padding_percent`, `gap_percent`, or equivalent).
- At least one unit test proves correct behavior under both definite and intrinsic measurement.

## M3 — Percent sizing for positioning (inset + margin)

**Goal**

Enable percent-based positioning for overlays and editor UI chrome, without ad-hoc math.

**Exit criteria**

- `inset` and `margin` can express percent/fraction (preserving `auto` where meaningful).
- A unit test proves basic percent inset outcomes.

## M4 — Ecosystem migration + cleanup

**Goal**

Remove recipe-layer workarounds and make shadcn components match upstream docs by default.

**Exit criteria**

- At least one shadcn component removes an ad-hoc px clamp in favor of native percent/fraction semantics.
- A diag script gate exists for the migrated component page in UI gallery.

