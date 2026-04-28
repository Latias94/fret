# Closeout Audit - 2026-04-28

## Verdict

Closed.

This lane now owns a local, non-Linux geometry-stability floor for the admitted compact IMUI control
families. The implementation slice found no state-driven outer-bounds drift, so no product code
refactor was required.

## Covered Families

Automated gates now cover stable outer bounds for:

- input text / textarea inherited chrome floor
- button
- checkbox / radio / switch
- slider
- combo trigger
- selectable
- menubar trigger
- submenu trigger inside an open parent menu
- tab trigger

The covered states are:

- hover
- focus
- pressed
- value / selected changes
- top-level menu open
- submenu open
- combo open
- disabled

## Gates

- `cargo nextest run -p fret-ui-kit --features imui compact_imui_chrome_without_focus_ring --no-fail-fast`
- `cargo nextest run -p fret-imui input_text_focus_keeps_control_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui base_control_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui menu_and_tab_trigger_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui control_disabled_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo check -p fret-ui-kit --features imui --jobs 2`
- `cargo fmt --package fret-imui --check`
- `python tools/check_workstream_catalog.py`
- `python -m json.tool docs/workstreams/imui-control-geometry-stability-v1/WORKSTREAM.json`
- `git diff --check`

## M2 Refactor Verdict

No unstable admitted control family was found by M1. The correct M2 action is therefore to leave
product code unchanged and keep the regression floor.

Do not widen this lane into:

- Linux/Wayland compositor acceptance
- docking or OS-window hand-feel
- editor-grade tab overflow/reorder/close policy
- shadcn recipe focus-ring parity
- public `fret-imui` identity ergonomics

Those require separate owner lanes if fresh evidence appears.

## Follow-On Policy

Keep this lane closed. Future geometry regressions may add tests here only when they are narrow
maintenance for the same invariant. New authoring APIs, identity syntax, floating-window contract
cleanup, or compositor behavior must start narrower follow-ons instead of reopening this execution
lane.
