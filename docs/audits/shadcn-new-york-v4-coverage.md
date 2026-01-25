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

- Golden files: `482`
- Golden keys (normalized `.open` suffix): `448`
- Keys referenced by tests: `248` (`55.4%`)
- Keys not referenced by tests: `200`

Top missing prefixes (heuristic grouping by the substring before the first `.` or `-`):

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -GroupMissingByPrefix -TopGroups 20
```

At the time of writing, the largest missing groups were:

- `chart` (76 variants; high surface area)
- `form` (19; field composition + validation chrome)
- `calendar` (6; primitives-heavy; tends to expose text metrics + grid layout edge cases)
- `typography` (13; baseline text metrics and prose defaults)
- `input` (10; control chrome + stacking patterns)

The largest referenced groups (already gated somewhere in `ecosystem/fret-ui-shadcn/tests`) were:

- `button` (25)
- `input` (17)
- `navigation` (17)
- `calendar` (23)
- `sidebar` (16)
- `toggle` (13)
- `dropdown` (10)
- `scroll` (10)
- `sheet` (10)
- `field` (9)
- `breadcrumb` (9)
- `menubar` (9)

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

1. **Gate missing primitives-heavy widgets** first (high churn risk):
   - Remaining calendar variants (10 missing): prioritize open-state popover/drawer composition
     (`calendar-23..30`, `calendar-32`) and `calendar-hijri`.
2. **Gate missing form composition + validation chrome** (high leverage):
   - `form-*` and any remaining `field-*` / `input-*` invalid variants
3. **Gate missing medium-surface components** next:
   - Carousel (`carousel-*`)
   - Typography breadth (`typography-*`)
4. **Fill breadth gaps**: add one golden + one gate per remaining component (default view).
5. **Add constrained viewport variants** for overlay-like components (if not already gated):
   - menus: height/width clamp + scroll buttons
   - popovers/tooltips: flip/shift under low height
   - dialogs/sheets: insets under low height/width
6. Keep `docs/audits/shadcn-new-york-v4-alignment.md` updated as behavior becomes audited (add
   “Conformance gates” anchors and “Known gaps” notes).

## Common “coverage smells”

- Many goldens exist, but none are referenced by tests (coverage drift).
- A component is gated only at one viewport size, but it changes behavior at others (false confidence).
- A gate checks only placement while ignoring geometry (panel width/height), allowing “menu height”
  regressions to slip through.
