# Mind model: Layout and sizing (Tailwind → Fret)

Goal: translate a Tailwind mental model into the Fret declarative API without guessing.

Fret is **not** a DOM renderer. If you port shadcn/Tailwind markup “by eye” but skip a few
constraints (especially `w-full`, `flex-1`, `items-stretch`, `min-w-0`), layouts drift fast:
cards/list items may shrink-to-content and make text wrap like a narrow column.

The fix is usually to make those constraints explicit in Fret (strategy layer / recipes), not to
change the layout engine.

## Mapping cheatsheet (common cases)

Sizing and fill:

- `w-full` → `LayoutRefinement::default().w_full()`
- `h-full` → `LayoutRefinement::default().h_full()`
- `size-full` (informal) → `LayoutRefinement::default().size_full()`
- `w-8` / `h-9` (fixed Tailwind sizes) → prefer tokens/metrics (`MetricRef` / theme keys) and apply
  with `.w_px(...)` / `.h_px(...)` (or `.w_space(...)` / `.h_space(...)` when you have a `Space` key)

Flex items:

- `flex-1` (Tailwind shorthand for `flex: 1 1 0%`) → `LayoutRefinement::default().flex_1()`
  - Tip: for text-heavy children, combine with `min_w_0()` to allow shrinking.
- `flex-none` → `LayoutRefinement::default().flex_none()`
- `grow` / `grow-0` → `LayoutRefinement::default().flex_grow(1.0)` / `.flex_grow(0.0)`
- `shrink-0` → `LayoutRefinement::default().flex_shrink_0()`

Min-size pitfalls (the “why is it wrapping like vertical text” class of bugs):

- `min-w-0` → `LayoutRefinement::default().min_w_0()`
  - Most important inside horizontal rows where a child contains text inputs or long labels.
- `min-h-0` → `LayoutRefinement::default().min_h_0()`
  - Common for scrollables inside a `vstack`/`column` where the middle pane must shrink.

Cross-axis alignment:

- `items-stretch` → `stack::HStackProps::default().items_stretch()` / `stack::VStackProps::default().items_stretch()`
  - Note: CSS flex defaults to `align-items: stretch`. Fret `hstack` defaults to `Items::Center`, so you often need to opt into stretch explicitly to match web.
- `items-center` → `items_center()`
- `justify-between` → `justify_between()`

Overflow:

- `overflow-hidden` → `LayoutRefinement::default().overflow_hidden()`
- `overflow-x-auto` (common in `Table`) → wrap the subtree in a `ScrollArea` with `axis(X)` and `type_(Auto)` (see `ecosystem/fret-ui-shadcn/src/table.rs` for a shadcn-aligned example)

Text (common shadcn recipes):

- `truncate` → `.truncate()` (on `UiBuilder<Text>` / `UiBuilder<StyledText>`)
- `whitespace-nowrap` → `.nowrap()`
- `break-words` → `.break_words()`

## A minimal “Tailwind flex row” port template

Upstream pattern (very common in shadcn):

- container: `flex items-center justify-between w-full gap-2`
- main content: `flex-1 min-w-0`
- title: `truncate`

Fret intent (pseudo, focus on constraints not exact widget names):

- row: `stack::hstack(..., HStackProps::default().gap(Space::N2).items_center().justify_between().layout(LayoutRefinement::default().w_full()), ...)`
- main content slot: `.layout(LayoutRefinement::default().flex_1().min_w_0()).overflow_hidden()`
- title text: `.truncate()`

If the row must behave like a block element (match web’s “fills available width”), always add
`w_full()` on the root chrome/container.

## Prefer tokens over hardcoded numbers

If it’s a shadcn recipe dimension (control heights, paddings, radii):

- store it in theme metrics (or reuse existing keys),
- then resolve via `MetricRef`/`Theme` instead of sprinkling `Px(…)` everywhere.

## Debugging layout regressions

Use `fretboard diag` to capture bundles and check geometry invariants before you chase pixels:

- add `test_id` on the relevant nodes,
- capture a bundle after the interaction,
- gate on bounds/semantics changes when possible.

## See also

- `fret-diag-workflow` (scripted repro + packaging)
