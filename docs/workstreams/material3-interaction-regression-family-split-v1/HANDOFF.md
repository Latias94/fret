# Material3 Interaction Regression Family Split v1 Handoff

Status: Closed
Last updated: 2026-05-31

## What Changed

- Added `material3_navigation_interactions.rs` with 11 NavigationBar, NavigationRail,
  NavigationDrawer, and ModalNavigationDrawer tests.
- Added `material3_overlay_interactions.rs` with 12 Snackbar, Menu, Dialog, Tooltip, RichTooltip,
  and DropdownMenu tests.
- Added `material3_choice_action_interactions.rs` with 15 Switch, Tabs, IconButton,
  IconToggleButton, Chips, Checkbox, SegmentedButton, and ChipSet tests.
- Reduced `material3_interaction_regressions.rs` to 10 explicitly deferred TextInput, TimePicker,
  Autocomplete, and ExposedDropdown tests.

## What Remains

- TimePicker should get its own interaction binary once its residual tests are audited together.
- Autocomplete and ExposedDropdown should be split as a field-family packet.
- The plain TextInput event regression should be audited for possible `fret-ui` mechanism-layer
  ownership.

## Suggested Follow-Ons

- `material3-time-picker-interaction-family-split-v1`
- `material3-field-interaction-family-split-v1`
- `material3-text-input-mechanism-test-ownership-v1`
