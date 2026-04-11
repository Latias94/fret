# Adaptive Presentation Surface v1 — Milestones

Status: Closed milestone record
Last updated: 2026-04-11

## M0 — Baseline and lane creation

- Land an assumptions-first audit that explains why this is a new narrow follow-on instead of
  reopening the closed adaptive/device-shell/editor-rail lanes.
- Create the lane docs and register them in repo-wide entrypoints.

## M1 — Contract freeze

- Freeze the upper-interface owner split:
  - explicit app-shell composition,
  - family-specific wrapper boundary,
  - sidebar app-shell boundary,
  - editor-rail outer-shell downgrade boundary.
- Freeze the helper-extraction threshold.
- Completed narrow verdict on 2026-04-11:
  - `Dialog` / `Drawer` does not yet justify a new shared helper,
  - and future extraction must start as a narrower family-specific follow-on.

## M2 — Proof map

- Keep current source gates and the fixed-window panel-resize proof attached to this lane.
- Verify that the current proof set is sufficient to review future helper proposals.

## M3 — Decision

- Either close on an explicit no-new-helper verdict,
- or spin a narrower family-specific follow-on with concrete API + migration scope.

Closed outcome on 2026-04-11:

- explicit no-new-helper verdict landed,
- `Dialog` / `Drawer` stays an explicit proof pairing,
- and future extraction must start as a narrower family-specific follow-on.
