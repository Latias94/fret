# Mind model: Overlays and focus

Goal: overlays (menus/select/dialogs/tooltips) feel “Radix-correct” and don’t regress.

## What to verify (always)

- **Dismiss**: escape, outside press, trigger re-click, nested overlays (submenus) behave predictably.
- **Focus**: initial focus, focus trap (when modal), and focus restore to trigger on close.
- **Placement**: constrained viewport clamping + scrollable content max-height (tiny viewport variants).
- **Semantics**: listbox/menu roles, active descendant (when focus stays in input), disabled items.

## Prevent regressions

- Prefer a scripted repro in `tools/diag-scripts/` that opens the overlay and asserts a stable condition.
- Add a small unit test for the most fragile invariant (e.g., “content stays within viewport”, “trigger regains focus”).

## Source of truth

When behavior is unclear:

- Start with public docs:
  - shadcn/ui components: https://ui.shadcn.com/docs/components
  - Radix Primitives: https://www.radix-ui.com/primitives/docs/components
- For exact implementation details, inspect source code:
  - shadcn/ui v4 source (New York v4 registry): https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui
  - Radix Primitives source: https://github.com/radix-ui/primitives/tree/main/packages/react
  - Optional local pinned snapshots (if your checkout includes `repo-ref/`; not necessarily present on GitHub): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/<component>.tsx`, `repo-ref/primitives/packages/react/<primitive>/src/*`

## See also

- `references/recipes/INDEX.md` (recipe inventory + backlog)
- `fret-shadcn-source-alignment` (upstream parity workflow)
- `fret-diag-workflow` (scripted repro + packaging)
