# Mind model: Layout and sizing (Tailwind → Fret)

Goal: translate a Tailwind mental model into the Fret declarative API without guessing.

## Mapping cheatsheet (common cases)

- `w-full` → `LayoutRefinement::default().w_full()`
- `h-full` → `LayoutRefinement::default().h_full()`
- `w-8` / `h-9` (fixed px in Tailwind) → use explicit px metrics/tokens in Fret (`MetricRef` / theme keys) and apply with `.w_px(...)` / `.h_px(...)`
- `min-h-0` (allow flex children to shrink) → ensure the child can shrink: set `min_width/min_height` to `0` where needed

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
