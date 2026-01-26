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
- Keys referenced by tests: `372` (`83.0%`)
- Keys not referenced by tests: `76`

Top missing prefixes (heuristic grouping by the substring before the first `.` or `-`):

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -GroupMissingByPrefix -TopGroups 20
```

At the time of writing, the largest missing groups were:

- `chart` (76 variants; high surface area)

The largest referenced groups (already gated somewhere in `ecosystem/fret-ui-shadcn/tests`) were
(heuristic grouping by key prefix):

- `calendar` (34)
- `input` (27)
- `button` (26)
- `form` (19)
- `navigation` (17)
- `sidebar` (16)
- `typography` (14)
- `toggle` (13)
- `field` (12)
- `sheet` (10)
- `scroll` (10)
- `item` (10)
- `spinner` (10)
- `dropdown` (10)
- `breadcrumb` (9)
- `menubar` (9)
- `textarea` (8)
- `select` (7)
- `combobox` (7)
- `context` (7)

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

1. **Chart push**: treat `chart-*` as a dedicated sprint (surface area is large; likely needs additional audit notes).
2. **Add constrained viewport variants** early for overlay-like components:
   - menus/listboxes: max-height clamp + scroll buttons + row height (treat “menu height” as a styling outcome)
   - popovers/tooltips: flip/shift under low height
   - dialogs/sheets: insets under low height/width
4. **DPI / font-metrics gates**: keep this dimension small and targeted until chart geometry is stable
   (recommended targets: typography + menus/listboxes).

## Common “coverage smells”

- Many goldens exist, but none are referenced by tests (coverage drift).
- A component is gated only at one viewport size, but it changes behavior at others (false confidence).
- A gate checks only placement while ignoring geometry (panel width/height), allowing “menu height”
  regressions to slip through.
