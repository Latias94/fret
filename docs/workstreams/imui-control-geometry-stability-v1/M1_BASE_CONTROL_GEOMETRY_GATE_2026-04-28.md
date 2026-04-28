# M1 Base Control Geometry Gate - 2026-04-28

## Goal

Lock the first local geometry-stability floor for base IMUI controls after the text-control focus
chrome fix.

## Landed Gates

`ecosystem/fret-imui/src/tests/composition.rs` now includes
`base_control_state_changes_keep_outer_bounds_stable`.

The gate renders a compact vertical IMUI surface and records baseline outer bounds for:

- button
- checkbox
- radio
- switch
- slider
- combo trigger
- selectable

It then verifies that those outer bounds stay unchanged across:

- pointer hover
- focus
- pressed state
- checkbox / switch value changes
- slider value changes
- combo open state
- radio / selectable selected state

The same file also includes `menu_and_tab_trigger_state_changes_keep_outer_bounds_stable`.

That gate records baseline outer bounds for:

- menubar trigger
- submenu trigger inside an open parent menu
- tab trigger

It then verifies that those outer bounds stay unchanged across:

- pointer hover
- focus
- pressed state
- top-level menu open state
- submenu open state
- tab selected state

`control_disabled_state_changes_keep_outer_bounds_stable` closes the disabled-state half of the
same invariant. It records enabled-state baseline bounds, toggles the controls to disabled, and
verifies stable outer bounds for:

- input text / textarea
- button
- checkbox / radio / switch
- slider
- combo trigger
- selectable
- menubar trigger
- submenu trigger inside an open parent menu
- tab trigger

## Result

The admitted base-control set already preserves stable outer geometry. No product code change was
needed for this slice.

This is still useful because the previous text-control bug showed that the repo lacked a single
cross-control regression floor. Future chrome refactors now have a focused local gate before they
reach demos or screenshot diagnostics.

## Gates

- `cargo test -p fret-imui base_control_state_changes_keep_outer_bounds_stable -- --nocapture`
- `cargo nextest run -p fret-imui base_control_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui menu_and_tab_trigger_state_changes_keep_outer_bounds_stable --no-fail-fast`
- `cargo nextest run -p fret-imui control_disabled_state_changes_keep_outer_bounds_stable --no-fail-fast`

## Deferred

Linux/Wayland real-host acceptance remains deferred to
`docs/workstreams/docking-multiwindow-imgui-parity/`.
