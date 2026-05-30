# Navigation Drawer Diagnostic v1

Status: done
Date: 2026-05-27

## Problem

The existing `ui-gallery-material3-navigation-drawer-item-chrome-fill.json` diagnostic was stale. It
searched for `material3 gallery`, clicked `ui-gallery-nav-material3-gallery`, and then waited for
NavigationDrawer item IDs that now live on the dedicated Material 3 Navigation Drawer page.

The first M3ND-040 run failed at the wait for `ui-gallery-material3-drawer-search`.

## Fix

The script now searches for `material3 navigation drawer`, clicks
`ui-gallery-nav-material3-navigation-drawer`, and waits for
`ui-gallery-page-material3-navigation-drawer` before asserting drawer item chrome bounds.

## Proof

```powershell
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json > $null
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json --dir target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

The rerun passed:

- AI packet:
  `target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun/sessions/1779898612003-126904/1779898964906/ai.packet`
- Pack:
  `target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun/sessions/1779898612003-126904/share/1779898964906.zip`

## Diagnostic Decision

No new modal drawer motion script was added in this slice. The existing gates cover the needed
packet surface for this lane:

- headless navigation goldens cover standard and modal drawer visual scene output,
- `modal_navigation_drawer_focus_is_contained_and_restored_across_schemes` covers focus
  containment/restore,
- the repaired drawer item chrome diagnostic covers gallery-level item/chrome fill behavior with
  stable selectors.

Add a dedicated modal drawer motion script only if future evidence shows timing/interruption drift
that is not covered by the existing overlay transition/focus gates.
