# Material 3 Navigation Drawer Overlay Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-27

## Smallest Current Repro

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1
```

Current baseline on 2026-05-27: fails on
`goldens/material3-headless/v1/material3-navigation.scale1_0.dark.tonal_spot.json`.

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-navigation-drawer-overlay-packet-v1/WORKSTREAM.json > $null
python tools/check_workstream_catalog.py
```

### Drawer And Modal Drawer

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_modal_navigation_drawer_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment modal_navigation_drawer_focus_is_contained_and_restored_across_schemes
```

### Golden Classification

```powershell
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1
```

This gate is red at lane open and must not be treated as closeout proof until M3ND-010 classifies
the mismatch and M3ND-030 either repairs or refreshes the evidence.

### Crate Inner Loop

```powershell
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Evidence Anchors

- `docs/workstreams/material3-component-alignment-sweep-v1/CLOSEOUT_AUDIT_2026-05-27.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_sweep_closeout_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/CLOSEOUT_AUDIT_2026-05-27.md`
- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/navigation_golden_baseline_v1.md`
- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/drawer_modal_packet_v1.md`
- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/navigation_geometry_resolution_v1.md`
- `docs/workstreams/material3-navigation-drawer-overlay-packet-v1/artifacts/navigation_drawer_diag_v1.md`
- `ecosystem/fret-ui-material3/src/navigation_drawer.rs`
- `ecosystem/fret-ui-material3/src/modal_navigation_drawer.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-navigation.*.json`

## Fresh Evidence Log

- 2026-05-27: Opened the navigation drawer overlay packet follow-on and reproduced the baseline red
  navigation golden gate.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1`
  - Result: failed on `material3-navigation.scale1_0.dark.tonal_spot.json`; the first visible
    mismatch has unchanged case coverage and broadly stable signatures, with rectangle geometry
    drift in `bar.selected`, `drawer.selected`, `modal_drawer.open`, and `rail.selected`.
  - Classification: Bar/Rail/underlay drift is stale fixture slot expectation; Drawer/ModalDrawer
    selected-pill shrink is a recipe/harness fill-boundary concern that blocks blanket golden
    refresh.
  - Evidence note: `artifacts/navigation_golden_baseline_v1.md`
- 2026-05-27: Completed M3ND-020 drawer/modal-drawer selector and focus packet.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_modal_navigation_drawer_exposes_stable_part_test_ids`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment modal_navigation_drawer_focus_is_contained_and_restored_across_schemes`
  - Result: all three gates passed. NavigationDrawer and ModalNavigationDrawer stable selector
    surfaces are live, and modal focus containment/restore remains covered by existing kit overlay
    policy.
  - Evidence note: `artifacts/drawer_modal_packet_v1.md`
- 2026-05-27: Completed M3ND-030 navigation geometry repair and golden refresh.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1` failed before repair, but after adding an explicit full-size layout to NavigationDrawer's internal roving flex the selected-pill width drift was repaired.
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1`
  - Result: refreshed navigation goldens after classifying remaining Bar/Rail/underlay drift as stale fixture slot expectation; the navigation golden suite passes without `FRET_UPDATE_GOLDENS`.
  - Evidence note: `artifacts/navigation_geometry_resolution_v1.md`
- 2026-05-27: Completed M3ND-040 drawer diagnostic repair and rerun.
  - Initial run: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json --dir target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - Result: failed at script step 13 because the script still navigated to `ui-gallery-nav-material3-gallery` while the drawer demo now lives on `ui-gallery-nav-material3-navigation-drawer`.
  - `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json > $null`
  - Rerun: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-drawer-item-chrome-fill.json --dir target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - Result: passed. AI packet: `target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun/sessions/1779898612003-126904/1779898964906/ai.packet`; pack: `target/fret-diag/material3-navigation-drawer-item-chrome-fill-m3nd040-rerun/sessions/1779898612003-126904/share/1779898964906.zip`.
  - Evidence note: `artifacts/navigation_drawer_diag_v1.md`
- 2026-05-27: Completed M3ND-050 closeout verification.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_navigation_suite_goldens_v1`
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
  - `python -m json.tool docs/workstreams/material3-navigation-drawer-overlay-packet-v1/WORKSTREAM.json > $null`
  - `python tools/check_workstream_catalog.py`
  - Result: automation surface has 20 passing tests; navigation golden suite passes; crate check
    and clippy pass; workstream JSON/catalog gates pass.
  - Evidence note: `CLOSEOUT_AUDIT_2026-05-27.md`
