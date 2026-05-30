# Material 3 Navigation Drawer Overlay Packet v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-27

## Verdict

This lane is closed.

The drawer/modal-drawer follow-on split from the Material 3 component alignment sweep is complete:
the stale navigation golden drift was classified, the real Drawer/ModalDrawer selected-pill
fill-boundary issue was fixed, the navigation goldens were refreshed after classification, and the
gallery diagnostic route was repaired and rerun.

## Shipped State

- `NavigationDrawer` internal roving flex now explicitly fills the drawer container, restoring the
  selected pill to full row width.
- `ModalNavigationDrawer` keeps dotted root/scrim/scrim.chrome/panel selectors from the sweep
  closeout.
- `material3_headless_navigation_suite_goldens_v1` passes without `FRET_UPDATE_GOLDENS`.
- The drawer item chrome-fill diagnostic now navigates to the dedicated Material 3 Navigation
  Drawer page and passes.

## Layer Result

- `material_recipe`: owns the NavigationDrawer internal fill constraint and drawer/modal drawer
  part IDs.
- `test_harness`: owned stale outer slot expectations and stale diag page navigation.
- `kit_policy`: still owns overlay dismissal, focus trap, and focus restore.
- `material_foundation`: no new shared navigation foundation helper was required.
- `mechanism`: no `crates/*` gap was found.

## Fresh Gates

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json > $null
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json --dir target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
python -m json.tool docs/workstreams/material3-navigation-drawer-overlay-packet-v1/WORKSTREAM.json > $null
python tools/check_workstream_catalog.py
git diff --check -- docs/workstreams/README.md docs/workstreams/material3-navigation-drawer-overlay-packet-v1 tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json ecosystem/fret-ui-material3/src/navigation_drawer.rs goldens/material3-headless/v1
```

All closeout gates passed. The broader diff check may still report the known CRLF warning for
`ecosystem/fret-ui-material3/tests/radio_alignment.rs` when that broader file set is included.

## Follow-Ons

Start a new narrow lane only if future evidence proves:

- modal drawer motion interruption/timing drift not covered by the existing overlay transition and
  focus gates,
- another navigation component needs shared Material navigation geometry foundation,
- or gallery navigation semantics change again and invalidates the repaired diagnostic route.

## Residual Risks

- This lane does not claim full upstream 1:1 Material navigation behavior.
- The diagnostic covers standard drawer item chrome fill; modal drawer motion remains covered by
  headless scene output and focus/overlay behavior gates rather than a dedicated fixed-timestep
  script.
