# Material 3 Component Alignment Sweep v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-27

Status note (2026-05-28): this closeout records the broad sweep state as of 2026-05-27. The
current post-follow-on closure state lives in
`docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_follow_on_closure_audit_v1.md`.
As of 2026-05-28, `component_alignment_matrix_v1.json` has 39 classified rows, no
`packet_done_known_follow_ons` rows, and no rows missing `packet_artifacts`,
`layer_classification`, or `first_gate_kind`. The matrix summary and follow-on list below should be
read as historical 2026-05-27 closeout context.

## Verdict

This lane is closed with narrow follow-ons.

The sweep target is satisfied for this lane: all 39 `fret-ui-material3` components in
`component_alignment_matrix_v1.json` have a current classification, high-risk families have packet
evidence or an explicitly split follow-on, and the shared refactors that did land are backed by
consumer-level gates.

Two rows remain intentionally queued as follow-ons, not hidden broad work:

- `navigation_drawer`
- `modal_navigation_drawer`

Both rows now have selector seed proof. Their remaining work is visual/overlay packet evidence and
stale navigation-golden classification, not a missing mechanism contract.

## Shipped State

- M3CAS-020 stabilized the known controls golden drift before broad component evidence was reused.
- M3CAS-030 established the first stable selector audit.
- M3CAS-040 packeted Tabs/NavigationBar/NavigationRail active indicators and moved shared
  active-indicator paint/motion into Material foundation.
- M3CAS-050 packeted the field family and moved filled-field active-indicator paint into shared
  Material foundation used by TextField and Select.
- M3CAS-060 packeted DatePicker/TimePicker selectors and stale picker goldens.
- M3CAS-070 packeted overlay and feedback selectors/behavior without moving overlay policy out of
  `fret-ui-kit`.
- M3CAS-080 packeted choice controls and chips, keeping state-layer/ripple/minimum-size behavior in
  existing Material foundation.
- M3CAS-090 packeted low-interaction surface/data-display components, added
  `Badge::anchor_size`, and refreshed Badge/Divider/ProgressIndicator goldens after drift
  classification.
- M3CAS-100 consolidated dotted part-id helper construction into `foundation::test_id`.
- M3CAS-110 split TopAppBar semantics into `top_app_bar_alignment.rs`.
- M3CAS-120 corrected ModalNavigationDrawer's legacy hyphenated scrim selector into the dotted
  root/scrim/scrim.chrome/panel part-id contract and added automation-surface proof.

## Matrix Summary

- `packet_done_foundation_refactored`: 6 rows
- `packet_done_known_follow_ons`: 24 rows
- `packet_done_low_risk`: 7 rows
- `follow_on_queued_selector_seeded`: 1 row
- `follow_on_queued_overlay_packet`: 1 row

There are no unclassified rows. Every row has `selector_status`, `first_gate_kind`, and
`layer_classification`.

## Implementation Boundaries

- No Material policy was moved into `crates/*`.
- Overlay dismissal, focus containment, focus restore, tooltip delay, and toast live-region policy
  remain in `fret-ui-kit`.
- Material-wide recipe helpers live in `ecosystem/fret-ui-material3/src/foundation`.
- Component-local geometry, slot composition, and stable part IDs remain in the recipe files.
- Navigation drawer/modal drawer broad visual evidence is split because
  `material3_headless_navigation_suite_goldens_v1` still contains stale geometry drift unrelated to
  the M3CAS-100 helper consolidation.

## Fresh Gates

Closeout used these fresh gates after the ModalNavigationDrawer selector correction:

```powershell
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_modal_navigation_drawer_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment modal_navigation_drawer_focus_is_contained_and_restored_across_schemes
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/WORKSTREAM.json > $null
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null
python tools/check_workstream_catalog.py
git diff --check -- docs/workstreams/material3-component-alignment-sweep-v1 ecosystem/fret-ui-material3/src ecosystem/fret-ui-material3/tests goldens/material3-headless/v1
```

All closeout gates passed. `git diff --check` emitted only the known CRLF warning for
`ecosystem/fret-ui-material3/tests/radio_alignment.rs`.

## Follow-Ons

Start new narrow workstreams instead of reopening this broad sweep for:

- NavigationDrawer and ModalNavigationDrawer visual/overlay packet plus stale
  `material3_headless_navigation_suite_goldens_v1` classification.
- Named canvas draw-region diagnostics before exposing ProgressIndicator internal track/segment/arc
  parts.
- Named canvas draw-region diagnostics before exposing Slider internal track/handle/state-layer
  parts.
- SearchView full Compose-style SearchBarState, full-screen transition, and back-handling parity.
- DatePicker/TimePicker deeper accessibility such as localized labels, disabled date/time items,
  and live-region announcements. Several DatePicker and TimePicker slices were closed later by
  dedicated packets.
- Rich tooltip interactivity, because current tooltip overlays are click-through.
- BottomSheet layout-safe chrome aliases if consumers need stable subpart automation.
- Further `radio_alignment.rs` family splits, one stable golden family at a time.

## Residual Risks

- Full workspace test/clippy were not run because this lane touched a focused Material crate slice.
- The lane proves classification and targeted parity packets, not complete upstream 1:1 behavioral
  parity for every possible Material state.
- Navigation drawer/modal drawer visual evidence is intentionally split because the current broad
  navigation golden suite is stale.
