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
- If the helper is a wrapper (`h_row`, `h_flex`, `v_stack`, `v_flex`) with an outer container plus
  an inner flex root, confirm width/min/max constraints reach the actual flex node rather than only
  the wrapper. A `w_full().min_w_0()` patch on the outer shell is not enough if the inner flex root
  still resolves intrinsic width.
- Does it rely on a default that differs from the DOM?
  - `items-stretch` vs `items-start`
  - `w_full()` on children vs on wrappers
  - implicit `flex_shrink` / `flex_grow` choices
- Did we only compare the component source and forget the exact docs-path example file?
  - example-local `size`, `variant`, wrapper layout, and slot-local utility classes are parity
    truth too
  - example-local placeholder copy, `text-sm` vs default body text, and explanatory copy injected
    into the demo body are parity truth too; they can change height and visual weight without
    changing the slot structure

3) Decide the owning layer

- If the issue is “default alignment / fill / shrink / wrap”, it is almost always
  `ecosystem/fret-ui-shadcn` (recipe policy), not `crates/fret-ui`.
- If the upstream slot uses explicit non-uniform grid tracks (`grid-cols-[1fr_auto]`,
  `grid-cols-[0_1fr]`, fixed+`fr`, explicit `grid-rows-[...]`) and Fret only exposes evenly sized
  grid tracks, stop and audit the runtime contract first. That is a mechanism gap, not a recipe
  polish issue.
- If explicit tracks are already present, do not close the audit yet. Check the next grid tier:
  - container `justify-items` / `place-items-*`,
  - item `align-self` / `justify-self`,
  - and whether upstream uses separate `gap-x-*` / `gap-y-*` that Fret currently compresses into
    one gap value.
- If the issue is “hit-testing / routing / transforms / clipping”, it might be mechanism
  (`crates/fret-ui`) — confirm via the mechanism parity checklist.
- If the issue is specifically item self-alignment (`self-start`, `self-stretch`, `justify-self-*`)
  and the recipe currently mutates `props.layout.*` after style resolution, prefer landing or
  using a shared declarative surface first (`LayoutRefinement::self_*`,
  `LayoutRefinement::justify_self_*`, and the matching `UiBuilder` helpers). Do not keep cloning
  raw prop patches across recipes once the common surface exists.

## Default-style ownership (recipe vs call site)

Before changing a default, inspect *where upstream applies the class*: 

- If the class lives on the upstream component source, it is a candidate recipe default in Fret.
  - Typical examples: border, radius, shadow, slot padding, title typography, header/action alignment.
- If the class lives on the upstream example call site, keep it caller-owned in Fret.
  - Typical examples: `w-full`, `max-w-sm`, `min-w-0`, `flex-1`, centering, page/grid-specific wrappers.

Heuristic:

- recipe default = intrinsic to the component across most uses
- caller-owned = negotiated with the surrounding page/container

When in doubt, prefer caller-owned for width/flex/grid negotiation. It is much easier to opt in at
the call site than to unwind an overly opinionated default later.

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

### Source-policy test (preferred for docs-path example drift)

Pair geometry gates with a small source assertion when the upstream demo depends on exact
example-local props or copy:

- link button stays default-sized instead of `size="sm"`
- password input omits a placeholder if upstream omits it
- body copy stays on the default text lane instead of drifting to `text-sm`
- demo-only copyability notes stay in page notes/section descriptions rather than the example body


## Case study: Card root width “should Card default to w-full?”

Upstream shadcn/ui v4:

- `Card` source owns border/radius/shadow/vertical padding.
- Example files opt into widths like `w-full max-w-sm` at the call site.

Failure mode in Fret:

- The recipe bakes `w_full()` into the `Card` root because a gallery/doc page looked too narrow.
- This fixes one page, but makes the recipe more opinionated than upstream and hides the real page
  layout problem.

Fix pattern:

- Keep root width caller-owned.
- Keep section internals fill-width where that is part of the recipe outcome.
- Put `w_full`, `min_w_0`, and `max_w(...)` on the page/grid/flex wrapper that actually negotiates
  width.

Regression protection:

- Unit test for the recipe default (`Card` root width remains `Auto`).
- UI gallery invariant test that relevant card examples keep the intended shared width.

## Case study: CardHeader action lane “looks close, but Sign Up is in the wrong place”

Upstream shadcn/ui v4:

- `CardHeader` switches to a grid when `CardAction` is present.
- `CardAction` occupies the top-right lane with `col-start-2 row-start-1 row-span-2 self-start
  justify-self-end`.
- The demo file also matters: `Sign Up` is `variant=\"link\"` at the default size, not `size=\"sm\"`.

Failure modes in Fret:

- The recipe collapses the header to a generic `justify-between` row without proving the action
  still occupies the same visual lane.
- The runtime gains explicit tracks, but first-column children still author `w-full` / `Length::Fill`
  as a raw percent width against the whole grid container, so the `1fr` track expands to the full
  card width and pushes the action slot outside the card.
- The snippet copies the slot structure but changes example-local props (`size=\"sm\"`), so the
  visible result drifts even when the component source is otherwise fine.

Fix pattern:

- Audit both `ui/card.tsx` and the exact example file (for example `examples/card-demo.tsx`).
- Before rewriting the recipe, confirm that Fret can actually express the upstream track list.
  If the current grid contract only exposes evenly sized tracks, land the runtime vocabulary first
  and then translate the recipe onto it.
- After explicit tracks land, check whether the remaining drift is really a child-sizing problem:
  upstream grid slots often rely on default grid stretch rather than explicit `w-full` on the
  first-column children. If Fret still needs `w-full`-shaped authoring for that slot family, the
  runtime may need a grid-item `Fill -> stretch` translation so `fr auto` lanes stay stable.
- After fill semantics land, check the remaining slot alignment semantics too. Upstream grid slots
  may still rely on `justify-items-start`, `place-items-center`, `self-start`, or
  `justify-self-end`; if Fret cannot express those, the recipe is still blocked on the runtime
  contract even though track placement already works.
- Keep an eye on axis-specific gaps. `gap-x-*` / `gap-y-*` pressure is often the next parity clue
  after tracks and self-alignment are correct, especially in Alert / AlertDialog-like headers.
- Treat header/action placement as recipe-owned when the positioning comes from the component
  source.
- Add a geometry gate that checks the action stays in the top-right lane relative to title and
  description, rather than relying on source similarity alone.

Regression protection:

- UI gallery geometry test for `title / description / action` relative bounds.
- Source-policy test that the copied demo keeps the upstream example-local button props.
