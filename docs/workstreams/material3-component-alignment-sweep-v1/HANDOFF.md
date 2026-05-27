# Material 3 Component Alignment Sweep v1 - Handoff

Status: Active
Last updated: 2026-05-27

## Current State

The workstream is active. The previous Material 3 parity harness lane is closed and provides seed
evidence for Button, Select, and Switch. M3CAS-020 through M3CAS-070 are complete.

## Active Task

- Task ID: M3CAS-080
- Goal: align choice controls and chips around state layer/ripple, selected indicators, group
  semantics, gesture handling, and minimum touch target policy.
- Current progress: M3CAS-070 closed with known follow-ons. Menu, DropdownMenu, Dialog,
  BottomSheet, Tooltip, and Snackbar now have packet-level selector/behavior classification.
  `automation_surface` has 15 passing tests, and snackbar/menu-dialog/bottom-sheet headless
  goldens were refreshed after stale drift was classified.
- Reason: overlay/focus policy stayed in existing kit primitives; the next likely duplication risk
  is state-layer/ripple and choice-control group semantics.

## Key Decisions

- This is a component sweep plus foundation-escalation lane, not a component-by-component port.
- Every component needs a matrix row; only high-risk components need full parity packets.
- Shared drift across at least two components should be considered for Material foundation or
  `fret-ui-kit` policy.
- Material policy stays out of `crates/*` unless a packet proves a mechanism/contract gap.
- shadcn is a Fret-side harness exemplar only, not Material visual truth.

## Seed Evidence

- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`
- `tools/parity-discovery/suites/material3_parity_discovery_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_controls_golden_drift_v1.md`

## Completed Since Open

- M3CAS-020 classified the known controls golden drift as stale golden plus an underspecified
  controls-suite alignment assumption.
- The controls-suite vertical flex now explicitly uses `CrossAlign::Start`.
- All twelve `goldens/material3-headless/v1/material3-controls.*.json` files were refreshed after
  that harness clarification.
- The targeted controls golden gate passes without `FRET_UPDATE_GOLDENS`.
- M3CAS-030 added dotted Material recipe part ids for Tabs, NavigationBar, and NavigationRail and
  proved them through `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface`.
- M3CAS-040 added `foundation::active_indicator`, moved shared indicator paint/motion out of
  Tabs/NavigationBar/NavigationRail, kept component-specific geometry in recipes, added the
  navigation indicator packet/report, and passed three fixed-timestep diagnostics.
- M3CAS-050 selector progress added dotted part ids for TextField, Autocomplete popup/options,
  SearchBar, and SearchView overlay.
- M3CAS-050 behavior packet split filled field active-indicator paint into `foundation::field`,
  reused it from TextField and Select, repaired stale Autocomplete/ExposedDropdown diag selectors,
  and recorded SearchView full state as a follow-on. `automation_surface` now has 10 passing tests.
- M3CAS-060 replaced picker-local hyphen/global ids with base-derived dotted parts for DatePicker
  and TimePicker, updated the TimePicker chrome-fill diagnostic, refreshed date/time headless
  goldens after classifying stale modal underlay/action stretch drift, and recorded accessibility
  depth as a follow-on. `automation_surface` now has 12 passing tests.
- M3CAS-070 added stable overlay/feedback selectors for Menu, DropdownMenu, Dialog, BottomSheet,
  Tooltip, and Snackbar, added dialog panel semantics, passed focused dismiss/focus/tooltip/snackbar
  behavior gates, refreshed snackbar/menu-dialog/bottom-sheet headless goldens, and recorded rich
  tooltip interactivity plus bottom-sheet chrome aliases as follow-ons. `automation_surface` now has
  15 passing tests.

## Next Recommended Action

Run M3CAS-080 with `run-workstream-task`:

1. Audit Checkbox, Radio, Slider, SegmentedButton, Chip, ChipSet, FilterChip, InputChip,
   SuggestionChip, and IconButton around state layer/ripple, selected indicators, gestures, and
   group semantics.
2. Classify findings by `material_foundation`, `material_recipe`, `kit_policy`, `diagnostics`, and
   `test_harness`.
3. Consolidate shared Material foundation only when at least two component rows prove the same
   duplication.
