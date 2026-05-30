# Material 3 Navigation Drawer Overlay Packet v1 - TODO

Status: Closed
Last updated: 2026-05-27

Task IDs use `M3ND-*`.

## M0 - Baseline Classification

- [x] M3ND-010 [owner=codex] [deps=none] [scope=ecosystem/fret-ui-material3/tests/radio_alignment.rs,goldens/material3-headless/v1,docs/workstreams/material3-navigation-drawer-overlay-packet-v1]
  Goal: Classify the current `material3_headless_navigation_suite_goldens_v1` failure before any golden refresh or recipe edit.
  Validation: red baseline command recorded; structural diff note separates signature drift, geometry drift, stale expectation, and potential real recipe behavior.
  Review: DONE_WITH_CONCERNS. Bar/Rail/underlay drift is stale fixture slot expectation, but Drawer/ModalDrawer selected-pill shrink looks like recipe or harness fill-boundary drift and must be repaired or disproven before refreshing navigation goldens.
  Evidence: `artifacts/navigation_golden_baseline_v1.md`.
  Handoff: Continue with M3ND-020 packet proof, then M3ND-030 geometry repair/refresh; do not blanket-refresh navigation goldens from the current output.

## M1 - Drawer And Modal Drawer Packet

- [x] M3ND-020 [owner=codex] [deps=M3ND-010] [scope=ecosystem/fret-ui-material3/src/navigation_drawer.rs,ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs,ecosystem/fret-ui-material3/tests]
  Goal: Write the drawer/modal-drawer packet and verify stable selectors, selected item chrome, semantics, focus containment, focus restore, scrim, and panel surfaces.
  Validation: `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_modal_navigation_drawer_exposes_stable_part_test_ids`; `cargo nextest run -p fret-ui-material3 --test radio_alignment modal_navigation_drawer_focus_is_contained_and_restored_across_schemes`.
  Review: DONE_WITH_KNOWN_FOLLOW_ONS. Selector and overlay/focus boundaries are proven; selected-pill visual geometry remains M3ND-030 scope.
  Evidence: `artifacts/drawer_modal_packet_v1.md`.
  Handoff: Keep overlay/focus policy in `fret-ui-kit`; continue with M3ND-030 before refreshing navigation goldens.

## M2 - Geometry Repair Or Golden Refresh

- [x] M3ND-030 [owner=codex] [deps=M3ND-010,M3ND-020] [scope=ecosystem/fret-ui-material3/src/navigation_drawer.rs,ecosystem/fret-ui-material3/src/navigation_bar.rs,ecosystem/fret-ui-material3/src/navigation_rail.rs,goldens/material3-headless/v1]
  Goal: Repair real recipe/foundation geometry drift or refresh stale navigation goldens after classification.
  Validation: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1`.
  Review: DONE. NavigationDrawer internal roving flex now fills the drawer, repairing selected-pill shrink; remaining Bar/Rail/underlay drift was stale slot expectation and navigation goldens were refreshed after classification.
  Evidence: `artifacts/navigation_geometry_resolution_v1.md`.
  Handoff: Continue with M3ND-040 only if modal drawer motion/scrim/panel diagnostics need coverage beyond the passing headless and focus gates.

## M3 - Overlay Motion And Diagnostics

- [x] M3ND-040 [owner=codex] [deps=M3ND-020,M3ND-030] [scope=tools/diag-scripts/ui-gallery/material3,apps/fret-ui-gallery,ecosystem/fret-ui-material3]
  Goal: Add or confirm focused diagnostics for modal drawer scrim/panel visibility, dismiss behavior, and motion if headless gates are insufficient.
  Validation: fixed-timestep `fretboard diag run` or explicit note explaining why existing headless gates are sufficient.
  Review: DONE_WITH_SCRIPT_REPAIR. Existing drawer item chrome diag was stale and now routes to the dedicated Material 3 Navigation Drawer page; the rerun passed. No new modal motion script is needed until timing/interruption drift is proven.
  Evidence: `artifacts/navigation_drawer_diag_v1.md`.
  Handoff: Proceed to M3ND-050 closeout with the repaired diag script, headless navigation golden gate, and focused selector/focus gates.

## M4 - Verification And Closeout

- [x] M3ND-050 [owner=codex] [deps=M3ND-030,M3ND-040] [scope=docs/workstreams/material3-navigation-drawer-overlay-packet-v1]
  Goal: Close the navigation drawer overlay packet with fresh gates and split any remaining navigation work.
  Validation: JSON/catalog gates, targeted Rust gates, navigation golden gate, and diff check.
  Review: DONE. Fresh Rust, diag, JSON/catalog, and diff gates pass; remaining work is narrow future evidence only.
  Evidence: `CLOSEOUT_AUDIT_2026-05-27.md`.
  Handoff: Return to the broader Material goal; do not reopen this drawer packet unless fresh motion or shared-foundation evidence appears.
