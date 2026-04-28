# ImUi Control Geometry Stability v1 - TODO

Status: active execution lane
Last updated: 2026-04-28

## Lane setup

- [x] Create the lane as a narrow follow-on after the text-control chrome stability closeout.
- [x] Record the owner split and local-only gate posture.
- [x] Wire the lane into `docs/workstreams/README.md`, `docs/roadmap.md`, and
      `docs/todo-tracker.md`.
- [x] Run the initial workstream catalog / JSON / diff hygiene gates.

## M0 - Baseline audit

- [x] Confirm the old control-chrome lane is closed and should not be reopened.
- [x] Confirm the text-control lane is closed and only supplies the inherited floor.
- [x] Confirm Linux/Wayland compositor acceptance belongs to the active docking parity lane, not
      this lane.
- [x] Inventory each base-control family and classify it:
      - already has geometry-stability coverage,
      - needs a focused test only,
      - needs a private chrome refactor,
      - or belongs to a different owner.

## M1 - State-invariant gate package

- [x] Add a small reusable test helper when it reduces repeated state/bounds assertions.
- [x] Add focused gates for button geometry across hover/focus/pressed.
- [x] Add focused gates for checkbox/radio/switch geometry across hover/focus/pressed/checked.
- [x] Add focused gates for slider geometry across hover/focus/pressed/value changes.
- [x] Add focused gates for combo trigger geometry across hover/focus/pressed/open.
- [x] Add focused gates for selectable geometry across hover/focus/pressed/selected.
- [x] Add focused gates for menu/submenu/tab trigger geometry only where current helpers expose a
      stable local proof path without reopening menu/tab policy depth.
- [x] Add a disabled-state geometry gate across text controls, base controls, menu/submenu
      triggers, and tab triggers.

## M2 - Refactor unstable controls

- [ ] Remove or rewrite any state-specific chrome path that changes outer dimensions.
- [ ] Keep shadcn recipe focus-ring policy out of IMUI compact controls unless explicitly reserved.
- [ ] Prefer shared IMUI chrome helpers only when they remove real duplication and preserve owner
      boundaries.
- [ ] Keep public `fret-imui` API unchanged unless the tests prove an authoring-surface gap.

## M3 - Closeout

- [ ] Update `EVIDENCE_AND_GATES.md` with the final gate set.
- [ ] Add `CLOSEOUT_AUDIT_2026-04-28.md` or later when the admitted control families are closed.
- [ ] Update repo-level workstream indexes with the closeout state.
