---
title: Layout parity footguns (stretch / hit box)
status: living
scope: shadcn alignment, layout policy defaults
---

# Layout parity footguns (stretch / hit box)

This checklist is for shadcn parity issues that are *not* about colors or tokens, but about default
layout behavior accidentally changing interactive geometry (e.g. a button’s pressable region grows
to fill a whole row).

## Why this happens in Fret

Fret recipes often wrap shadcn-like “plain div boundaries” with layout helpers like `vstack/hstack`
for ergonomics. In the DOM, a `div` does **not** imply `display:flex`, and it does **not** impose
cross-axis stretch. In Fret, a `vstack` is a *real flex container* and its default cross-axis
alignment may be `Stretch`, which changes children’s final bounds (and therefore hit-testing).

The key rule: if upstream is a plain `div`, your default should be “do not introduce flex policy
unless you have an explicit reason”.

## Fast triage (5 minutes)

1) Find the upstream class contract

- Source (shadcn/ui v4 registry): `https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui`
- Doc usage (shadcn components): `https://ui.shadcn.com/docs/components`

Ask: does this slot use `flex`, `grid`, `gap-*`, or is it just padding/typography?

2) Compare what our recipe *adds*

- Does our subtree introduce `vstack/hstack` where upstream is a `div`?
- Does it rely on a default that differs from the DOM?
  - `items-stretch` vs `items-start`
  - `w_full()` on children vs on wrappers
  - implicit `flex_shrink` / `flex_grow` choices

3) Decide the owning layer

- If the issue is “default alignment / fill / shrink / wrap”, it is almost always
  `ecosystem/fret-ui-shadcn` (recipe policy), not `crates/fret-ui`.
- If the issue is “hit-testing / routing / transforms / clipping”, it might be mechanism
  (`crates/fret-ui`) — confirm via the mechanism parity checklist.

## Gate patterns that work well

### Unit test (preferred for layout defaults)

Assert the actual layout node props emitted by the recipe. Example intent:

- “`CardContent` should not cross-axis stretch its children by default.”

Implementation strategy:

- Build the element in `with_element_cx(...)`.
- Inspect the immediate stack child (`ElementKind::Column` / `Row`) and assert its `align` is
  `Start` (not `Stretch`).

### Diag script (preferred for interactive geometry)

Gate a simple predicate against a stable `test_id`:

- `bounds_max_size` to prevent accidental full-row stretch
- `bounds_approx_equal` when you have both the hit target and the visual chrome target

Keep the script minimal:

- navigate to the smallest UI gallery page that renders the component,
- `scroll_into_view`,
- one geometry predicate,
- one bundle + optional screenshot.

Remember to:

- add a suite stub under `tools/diag-scripts/suites/<suite>/`,
- regenerate `tools/diag-scripts/index.json` via `python tools/check_diag_scripts_registry.py --write`.

## Case study: CardContent “button becomes full-row hit box”

Upstream shadcn/ui v4:

- `CardContent` is a plain `div` with `px-6` (no flex).

Failure mode in Fret:

- `CardContent` wraps children in a `vstack` whose default `items` is `Stretch`.
- A button inside becomes full width, growing the pressable region unexpectedly.

Fix pattern:

- Keep the container boundary (`px-*`, `w_full` on the section wrapper) but set the inner stack to
  `items_start()` by default, or avoid introducing a stack if it is not needed.

Regression protection:

- Unit test that checks `ColumnProps.align == CrossAlign::Start`.
- UI gallery demo that places an inline-sized button directly under `CardContent`.
- Diag script that gates `bounds_max_size` on that button.
