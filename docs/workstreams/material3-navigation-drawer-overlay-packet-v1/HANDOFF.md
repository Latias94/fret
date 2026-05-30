# Material 3 Navigation Drawer Overlay Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-27

## Current State

This narrow follow-on from the closed Material 3 component alignment sweep is closed.

The first baseline gate has been reproduced:

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1
```

It fails on `material3-navigation.scale1_0.dark.tonal_spot.json` with geometry drift across
navigation bar, drawer, modal drawer, and rail cases.

## Completed Task

- Task ID: M3ND-010
- Result: DONE_WITH_CONCERNS.
- Evidence: `artifacts/navigation_golden_baseline_v1.md`
- Classification: Bar/Rail/underlay drift is stale fixture slot expectation; Drawer/ModalDrawer
  selected-pill shrink is likely recipe or harness fill-boundary drift.

## Completed Task

- Task ID: M3ND-020
- Result: DONE_WITH_KNOWN_FOLLOW_ONS.
- Evidence: `artifacts/drawer_modal_packet_v1.md`
- Gates: NavigationDrawer automation surface, ModalNavigationDrawer automation surface, and modal
  drawer focus containment/restore all pass.

## Completed Task

- Task ID: M3ND-030
- Result: DONE.
- Evidence: `artifacts/navigation_geometry_resolution_v1.md`
- Gates: navigation headless golden suite was refreshed after classification and now passes without
  `FRET_UPDATE_GOLDENS`.

## Completed Task

- Task ID: M3ND-040
- Result: DONE_WITH_SCRIPT_REPAIR.
- Evidence: `artifacts/navigation_drawer_diag_v1.md`
- Gates: repaired drawer item chrome-fill diagnostic passes against the dedicated Material 3
  Navigation Drawer page.

## Closeout Result

- Task ID: M3ND-050
- Result: DONE.
- Evidence: `CLOSEOUT_AUDIT_2026-05-27.md`
- Gates: automation surface, navigation golden suite, crate check, clippy, repaired drawer diag,
  JSON/catalog, and diff checks passed.

## Boundaries

- Keep overlay/focus policy in `fret-ui-kit`.
- Keep Material visual geometry in recipe or Material foundation only when repeated evidence proves
  shared ownership.
- Navigation goldens have been refreshed after classification. Drawer item chrome diag is repaired.
  Do not add new motion diagnostics unless they prove an invariant not covered by the current
  headless/focus gates.
