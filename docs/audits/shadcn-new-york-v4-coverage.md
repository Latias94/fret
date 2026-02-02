# shadcn/ui new-york-v4 Coverage (Fret)

This document tracks **coverage**, not deep conformance details:

- Which shadcn-web goldens exist for `new-york-v4`
- Which golden keys are **gated** by `ecosystem/fret-ui-shadcn/tests`
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

- `tools/golden_coverage.ps1` currently counts a key as “gated” if the key appears as a
  **string literal** somewhere under `ecosystem/fret-ui-shadcn/tests`, excluding `*_goldens_smoke.rs`.
- This is a reasonable proxy for “this golden is exercised by some dedicated test”, but it is not
  a perfect proxy for “high-signal behavior is gated”.
- `shadcn_web_goldens_smoke.rs` provides a separate, *dynamic* “smoke-parse” pass that ensures we
  can parse the JSON and that the exported rectangles are finite.
- `tools/golden_coverage.ps1` reports “smoke-parse coverage” only when it can infer that the smoke
  test targets the requested `-Style` (otherwise it prints `n/a` to avoid false confidence).
- To avoid local, uncommitted goldens skewing the counts, prefer `-TrackedOnly`.
- In addition to the “any gate” percentage, `tools/golden_coverage.ps1` reports a **targeted**
  percentage that excludes the broad “catch-all” layout file(s) (`web_vs_fret_layout.rs` and
  `snapshots.rs` by default). This helps answer “how much is covered by high-signal, purpose-built
  checks” rather than “is every page referenced somewhere”.

- Golden files (tracked): `574`
- Golden keys (tracked, normalized `.open` suffix): `530`
- Gated keys (string-literal heuristic): `530` (`100%`)
- Targeted gates (excluding `web_vs_fret_layout.rs`, `snapshots.rs`): `530` (`100%`)
- Smoke-parse coverage: `100%` (via `shadcn_web_goldens_smoke_parse_and_rects_valid`)

As of 2026-02-02 (tracked-only).

Note on “targeted” gates:

- Most targeted keys have purpose-built assertions, but a small number are still *contract-only* checks
  (e.g. “golden contains expected tags”) while the deeper paint/layout assertions remain in the
  broad `web_vs_fret_layout.rs` file (notably `typography-table`).

Top missing prefixes (heuristic grouping by the substring before the first `.` or `-`):

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -AsMarkdown -GroupMissingByPrefix -TopGroups 20
```

At the time of writing, there are no missing groups (all keys are gated by tests).

The largest gated groups (already gated somewhere in `ecosystem/fret-ui-shadcn/tests`) were
(heuristic grouping by key prefix):

- `chart` (92)
- `calendar` (44)
- `button` (29)
- `input` (27)
- `navigation` (20)
- `form` (19)
- `typography` (16)
- `sidebar` (16)
- `sonner` (15)
- `dropdown` (14)
- `menubar` (14)
- `context` (14)
- `toggle` (13)
- `field` (12)
- `date` (12)
- `select` (12)
- `item` (10)
- `sheet` (10)
- `spinner` (10)
- `scroll` (10)
- `breadcrumb` (9)
- `combobox` (9)
- `textarea` (8)
- `empty` (7)
- `carousel` (6)
- `otp` (5)
- `signup` (5)
- `login` (5)
- `kbd` (5)
- `badge` (4)
- `drawer` (4)
- `alert` (4)
- `native` (4)
- `checkbox` (4)
- `resizable` (4)
- `command` (4)
- `dialog` (3)
- `popover` (3)
- `tooltip` (3)
- `skeleton` (2)

Top untargeted groups (i.e. keys only referenced by broad gates like `web_vs_fret_layout.rs` / `snapshots.rs`):

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -GroupUntargetedByPrefix -TopGroups 20
```

At the time of writing, there are no untargeted groups (all keys are referenced by purpose-built
gate tests outside the broad catch-all files).

Recompute locally:

```powershell
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -ShowMissing -TopMissing 50
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -ShowTargetedMissing -TopMissing 50
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -ShowUsed
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -GroupUsedByPrefix -TopGroups 20
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -ShowGateBreakdown
pwsh -NoProfile -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -TrackedOnly -GroupUntargetedByPrefix -TopGroups 20
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

- Many goldens exist, but none are gated by tests (coverage drift).
- A component is gated only at one viewport size, but it changes behavior at others (false confidence).
- A gate checks only placement while ignoring geometry (panel width/height), allowing “menu height”
  regressions to slip through.
