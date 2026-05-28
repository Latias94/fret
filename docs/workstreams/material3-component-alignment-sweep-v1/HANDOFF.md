# Material 3 Component Alignment Sweep v1 - Handoff

Status: Closed
Last updated: 2026-05-28

Status note (2026-05-28): the broad sweep remains closed, and the post-sweep follow-ons tracked in
the component matrix have now been closed by dedicated packets. Start from
`artifacts/material3_follow_on_closure_audit_v1.md` and
`artifacts/component_alignment_matrix_v1.json` for the current state. The older "Next Recommended
Action" list below is historical and should only be used when fresh evidence reopens one of those
scopes.

## Current State

The workstream is closed. The previous Material 3 parity harness lane is also closed and provides
seed evidence for Button, Select, and Switch. M3CAS-020 through M3CAS-120 are complete, and the
2026-05-28 follow-on closure audit records no remaining `packet_done_known_follow_ons` rows.

## Closeout Result

- All 39 matrix rows are classified.
- The matrix has no rows missing `selector_status`, `first_gate_kind`, or `layer_classification`.
- ModalNavigationDrawer now exposes dotted root/scrim/scrim.chrome/panel part IDs and has
  automation-surface proof.
- The 2026-05-28 follow-on closure audit records no remaining matrix residual queue.
- The broad sweep should not be reopened for historical follow-on scopes; start a new narrow lane
  only from fresh product evidence, source-backed drift, or release-readiness gates.

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
- M3CAS-080 added choice-control/chip selector coverage, confirmed shared state-layer/ripple and
  minimum interactive sizing stay in Material foundation, classified group roving as recipe-owned
  for now, and recorded slider internal canvas part selectors as a diagnostics follow-on.
  `automation_surface` now has 17 passing tests.
- M3CAS-090 added surface/data-display selector coverage, added `Badge::anchor_size` for
  deterministic TopRight placement, refreshed Badge/Divider/ProgressIndicator headless goldens, and
  classified Button/Card/CarouselItem/FAB/List/TopAppBar as low-risk recipe/foundation consumers.
  `automation_surface` now has 18 passing tests.
- M3CAS-100 consolidated dotted part-id generation into `foundation::test_id`, switched repeated
  `.chrome` helper construction across Select/Switch/Dialog/Menu/List/CarouselItem/NavigationDrawer
  and interactive-size helpers, and added NavigationDrawer selector proof. `automation_surface` now
  has 19 passing tests.
- M3CAS-110 split the low-coupling TopAppBar toolbar semantics smoke into
  `ecosystem/fret-ui-material3/tests/top_app_bar_alignment.rs`; old `radio_alignment.rs` still
  compiles.
- M3CAS-120 closed the sweep, corrected ModalNavigationDrawer's legacy hyphenated scrim selector
  into dotted part IDs, added ModalNavigationDrawer automation-surface proof, and split remaining
  work into narrow follow-ons.

## Next Recommended Action

Start a new narrow follow-on only when there is fresh evidence for one of these scopes:

1. NavigationDrawer and ModalNavigationDrawer visual/overlay packet plus stale navigation golden
   classification.
2. ProgressIndicator or Slider named canvas draw-region diagnostics.
3. SearchView full state/back/full-screen behavior.
4. Picker accessibility depth.
5. Rich tooltip interactivity.
6. BottomSheet chrome aliases.
7. Further `radio_alignment.rs` family splits.
