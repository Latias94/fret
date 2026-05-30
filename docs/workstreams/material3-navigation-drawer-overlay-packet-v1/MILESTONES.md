# Material 3 Navigation Drawer Overlay Packet v1 - Milestones

Status: Closed
Last updated: 2026-05-27

## M0 - Baseline Classification

Exit criteria:

- The current navigation golden failure is reproduced.
- Drift is separated into stale geometry, real behavior drift, or harness instability.
- No golden refresh happens without classification.

Evidence:

- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/navigation_golden_baseline_v1.md`

## M1 - Drawer And Modal Drawer Packet

Exit criteria:

- NavigationDrawer and ModalNavigationDrawer owner boundaries are explicit.
- Stable root/item/scrim/panel part IDs are proven live.
- Focus containment and focus restore are gated.

Evidence:

- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/drawer_modal_packet_v1.md`

## M2 - Geometry Repair Or Golden Refresh

Exit criteria:

- Real recipe/foundation issues are fixed with targeted gates, or stale goldens are refreshed after
  proof.
- The navigation golden suite passes or remaining mismatch is split.

Evidence:

- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/navigation_geometry_resolution_v1.md`

## M3 - Overlay Motion And Diagnostics

Exit criteria:

- Modal drawer scrim/panel/motion behavior has either a focused diag gate or a documented reason
  headless gates are sufficient.

Evidence:

- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/navigation_drawer_diag_v1.md`

## M4 - Verification And Closeout

Exit criteria:

- Workstream JSON and catalog gates pass.
- Targeted Rust gates pass.
- Remaining work is split into a narrow follow-on.

Evidence:

- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/CLOSEOUT_AUDIT_2026-05-27.md`
