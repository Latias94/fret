# shadcn/ui new-york-v4 Coverage (Fret)

This document tracks **coverage**, not deep conformance details:

- Which shadcn-web goldens exist for `new-york-v4`
- Which golden keys are referenced by `ecosystem/fret-ui-shadcn/tests`
- Where to focus next to increase “1:1” confidence efficiently

For deep, high-impact alignment notes and “gaps to check” per component, see:

- `docs/audits/shadcn-new-york-v4-alignment.md`
- Chart-specific audit: `docs/audits/shadcn-chart.md`

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

Notes:

- `tools/golden_coverage.ps1` currently counts a key as “referenced” if the key appears as a
  **string literal** somewhere under `ecosystem/fret-ui-shadcn/tests`.
- This is a good proxy for “this golden is exercised by some test”, but it is not a perfect proxy
  for “high-signal behavior is gated”.
- To avoid local, uncommitted goldens skewing the counts, prefer `-TrackedOnly`.

- Golden files (tracked): `512`
- Golden keys (tracked, normalized `.open` suffix): `473`
- Keys referenced by tests (string-literal heuristic): `473` (`100%`)
- Smoke-parse coverage: `100%` (via `shadcn_web_goldens_smoke_parse_and_rects_valid`)

Top missing prefixes (heuristic grouping by the substring before the first `.` or `-`):

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -GroupMissingByPrefix -TopGroups 20
```

At the time of writing, there are no missing groups (all keys are referenced by tests).

The largest referenced groups (already gated somewhere in `ecosystem/fret-ui-shadcn/tests`) were
(heuristic grouping by key prefix):

- `chart` (84)
- `calendar` (34)
- `input` (27)
- `button` (26)
- `form` (19)
- `navigation` (17)
- `sidebar` (16)
- `typography` (14)
- `toggle` (13)
- `field` (12)
- `dropdown` (11)
- `spinner` (10)
- `sheet` (10)
- `scroll` (10)
- `menubar` (10)
- `item` (10)
- `breadcrumb` (9)
- `textarea` (8)
- `select` (9)
- `combobox` (8)
- `context` (8)

Recompute locally:

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -ShowMissing -TopMissing 50
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -ShowUsed
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -GroupUsedByPrefix -TopGroups 20
```

To drill into a specific family:

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -FilterMissingPrefix calendar
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -FilterMissingPrefix carousel
```

## What to do next (recommended order)

1. **Depth push**: convert “smoke” gates into high-signal conformance checks (especially where layout depends on viewport).
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
