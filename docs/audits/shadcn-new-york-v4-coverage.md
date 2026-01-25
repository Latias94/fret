# shadcn/ui new-york-v4 Coverage (Fret)

This document tracks **coverage**, not deep conformance details:

- Which shadcn-web goldens exist for `new-york-v4`
- Which golden keys are referenced by `ecosystem/fret-ui-shadcn/tests`
- Where to focus next to increase “1:1” confidence efficiently

For deep, high-impact alignment notes and “gaps to check” per component, see:

- `docs/audits/shadcn-new-york-v4-alignment.md`

## Why coverage matters (and why viewport variants are not “nice-to-have”)

If the goal is 1:1 parity with upstream shadcn/ui + Radix primitives, coverage must include:

- **Breadth**: every component gets at least one golden + one gate (default outcome)
- **Depth for behavior-shaping viewports**: overlays and responsive components must be gated under
  constrained viewports early (clamp/shift/flip/max-height/scroll buttons/viewport sizing)

Practical rule:

- First pass: add a “default golden + minimal gate” for each component.
- In parallel: for overlays (menus, popovers, dialogs, navigation menu, etc.), add at least one
  constrained viewport variant as soon as the default is gated.

## Current snapshot (shadcn-web, v4/new-york-v4)

This is a **snapshot** from running `tools/golden_coverage.ps1` in this repo.

- Golden files: `487`
- Golden keys (normalized `.open` suffix): `448`
- Keys referenced by tests: `279` (`62.3%`)
- Keys not referenced by tests: `169`

Top missing prefixes (heuristic grouping by the substring before the first `.` or `-`):

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -GroupMissingByPrefix -TopGroups 20
```

At the time of writing, the largest missing groups were:

- `chart` (76 variants; high surface area)
- `typography` (13; baseline text metrics and prose defaults)
- `input` (10; control chrome + stacking patterns)
- `spinner` (9; control chrome + layout + visual alignment)
- `item` (8; list row chrome + slot alignment)

The largest referenced groups (already gated somewhere in `ecosystem/fret-ui-shadcn/tests`) were
(heuristic grouping by key prefix):

- `calendar` (34)
- `button` (25)
- `form` (19)
- `navigation` (17)
- `input` (17)
- `sidebar` (16)
- `toggle` (13)
- `dropdown` (10)
- `scroll` (10)
- `sheet` (10)
- `field` (9)
- `menubar` (9)
- `breadcrumb` (9)
- `carousel` (6)

Recompute locally:

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -ShowMissing -TopMissing 50
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -ShowUsed
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -GroupUsedByPrefix -TopGroups 20
```

To drill into a specific family:

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -FilterMissingPrefix calendar
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -FilterMissingPrefix carousel
```

## What to do next (recommended order)

1. **Finish the small missing families** (fast breadth gains):
   - Typography breadth (`typography-*`)
   - Spinner + item patterns (`spinner-*`, `item-*`)
2. **Fill breadth gaps**: add one golden + one gate per remaining component (default view).
3. **Add constrained viewport variants** early for overlay-like components:
   - menus/listboxes: max-height clamp + scroll buttons + row height (treat “menu height” as a styling outcome)
   - popovers/tooltips: flip/shift under low height
   - dialogs/sheets: insets under low height/width
4. **DPI / font-metrics gates**: keep this dimension small and targeted until breadth is higher
   (recommended targets: typography + menus/listboxes).
5. **Chart push**: treat `chart-*` as a dedicated sprint (surface area is large; likely needs additional audit
   notes + more selective gates).

## Common “coverage smells”

- Many goldens exist, but none are referenced by tests (coverage drift).
- A component is gated only at one viewport size, but it changes behavior at others (false confidence).
- A gate checks only placement while ignoring geometry (panel width/height), allowing “menu height”
  regressions to slip through.
