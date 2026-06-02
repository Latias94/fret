# Material3 Interaction Regression Family Split v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

`material3_interaction_regressions.rs` was a correctly named intermediate owner after the Radio
harness split, but it still mixed independent component families in one large binary. That made
review and nextest selection too coarse: navigation roving focus, overlay dismissal/focus, and
choice/action pressed-state regressions all had to move together.

## Decision

Split the clearly owned first packet into three purpose-owned test binaries:

- `material3_navigation_interactions.rs` for NavigationBar, NavigationRail, NavigationDrawer, and
  ModalNavigationDrawer interaction regressions.
- `material3_overlay_interactions.rs` for Snackbar, Menu, Dialog, Tooltip, RichTooltip, and
  DropdownMenu interaction regressions.
- `material3_choice_action_interactions.rs` for Switch, Tabs, IconButton, IconToggleButton, Chips,
  Checkbox, SegmentedButton, and ChipSet action-surface regressions.

Keep `material3_interaction_regressions.rs` as the residual owner for the tests that need a second
ownership decision: plain TextInput, TimePicker, Autocomplete, and ExposedDropdown.

## Boundaries

- This lane is a test-ownership refactor, not a component behavior change.
- Imports are tightened per binary so clippy can prove the split did not leave broad shared state.
- Harness modules remain local test support; this lane does not change public Material3 APIs.

## Non-Goals

- Do not move the plain TextInput regression into `fret-ui` mechanism coverage yet.
- Do not split TimePicker or field-family autocomplete/dropdown tests until their owner boundaries
  are audited.
- Do not rewrite interaction behavior while moving tests.

## Follow-On Shape

Future follow-ons should audit the residual file in this order:

1. TimePicker-owned interaction binary.
2. Field-family Autocomplete/ExposedDropdown binary.
3. Plain TextInput mechanism ownership audit.
