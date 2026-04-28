# ImUi Control Geometry Stability v1 - Milestones

Status: active execution lane
Last updated: 2026-04-28

## M0 - Baseline and Tracking

Exit criteria:

- existing lane ownership is resolved,
- Linux/Wayland acceptance is explicitly excluded,
- the lane is listed in repo-level indexes,
- and the initial hygiene gates pass.

Evidence:

- `docs/workstreams/imui-control-chrome-fearless-refactor-v1/WORKSTREAM.json`
- `docs/workstreams/imui-text-control-chrome-stability-v1/WORKSTREAM.json`
- `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`
- `docs/workstreams/imui-control-geometry-stability-v1/M0_BASELINE_AUDIT_2026-04-28.md`

## M1 - Geometry Gate Package

Current progress:

- `M1_BASE_CONTROL_GEOMETRY_GATE_2026-04-28.md` landed the first local gate for button,
  checkbox, radio, switch, slider, combo trigger, and selectable geometry across hover, focus,
  pressed, value/open, and selected states.
- `menu_and_tab_trigger_state_changes_keep_outer_bounds_stable` extends that gate package to
  menubar triggers, submenu triggers, and tab triggers across hover, focus, pressed, open, and
  selected states without reopening menu/tab policy depth.
- `control_disabled_state_changes_keep_outer_bounds_stable` closes the disabled-state gap for the
  inherited text-control surface, base controls, menu/submenu triggers, and tab triggers.

Exit criteria:

- the inherited text-control geometry floor still passes,
- every admitted base-control family has a focused bounds/state gate or an explicit owner deferral,
- and the gates run locally without Linux compositor dependencies.

Expected first gate families:

- button
- checkbox / radio / switch
- slider
- combo trigger
- selectable
- menu / submenu trigger
- tab trigger

## M2 - Fearless Refactor Slice

Exit criteria:

- unstable controls found by M1 are refactored,
- no demo-level size workaround is introduced,
- duplicated chrome/state code is deleted when the shared path is clearer,
- and the final code still respects the `fret-ui` mechanism / `fret-ui-kit::imui` policy split.

## M3 - Closeout

Exit criteria:

- `WORKSTREAM.json` moves to `closed`,
- `TODO.md` and `EVIDENCE_AND_GATES.md` reflect the final gate set,
- repo-level indexes point to the closeout,
- and any unresolved future work is split into a narrower follow-on instead of staying as vague
  backlog.
