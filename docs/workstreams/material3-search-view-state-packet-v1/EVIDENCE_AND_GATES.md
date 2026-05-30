# Material 3 SearchView State Packet v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-28

## Current Evidence

- `SearchView` is currently a docked MVP using a `SearchBar` underlay plus dismissible popover:
  - `ecosystem/fret-ui-material3/src/search_view.rs`
- The closed field-family packet explicitly left full Compose search transitions/back handling as a
  follow-on:
  - `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- Compose Material3 provides both docked and full-screen expanded search surfaces and collapses them
  via back handling:
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`

## Gate Set

```powershell
python -m json.tool docs/workstreams/material3-search-view-state-packet-v1/WORKSTREAM.json | Out-Null
python tools/check_workstream_catalog.py
cargo fmt --package fret-ui-material3 -- --check
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
```

Use `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
before closeout if recipe code changes.

## Evidence Log

- 2026-05-28: Opened the SearchView state packet as a narrow follow-on from the closed Material3
  component alignment sweep.
- 2026-05-28: M3SV-010 source packet completed from local Compose `SearchBar.kt`. Classification:
  full-screen presentation is Material recipe work; modal dismissal/focus stays in existing
  `fret-ui-kit` overlay policy; predictive/platform back remains a possible mechanism follow-on.
- 2026-05-28: M3SV-020/M3SV-030 red/green implementation.
  - Red compile proof:
    `cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_full_screen_uses_modal_overlay_and_closes_on_escape`
    failed on missing `SearchViewPresentation` and `presentation(...)`.
  - Behavioral red proof after the first implementation: the same test failed because closing the
    full-screen modal restored focus to the collapsed underlay input and immediately reopened the
    SearchView.
  - Final result: full-screen SearchView uses modal overlay, exposes `m3-search-view.overlay` and
    `m3-search-view.overlay.header`, focuses the overlay-local header input, and closes on Escape.
- 2026-05-28: M3SV-040 golden guard added.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`:
    failed before refresh because `full_screen_open` was a new SearchView golden case.
  - `$env:FRET_UPDATE_GOLDENS='1'; cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1; Remove-Item Env:FRET_UPDATE_GOLDENS`:
    passed and refreshed `goldens/material3-headless/v1/material3-search-view.*.json`.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`:
    passed without `FRET_UPDATE_GOLDENS`.
- 2026-05-28: M3SV-050 focused verification.
  - `cargo fmt --package fret-ui-material3 -- --check`: passed.
  - `cargo nextest run -p fret-ui-material3 --test search_view_behavior`: passed.
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids`:
    passed.
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1`:
    passed.
  - `cargo check -p fret-ui-material3 --features diagnostics --tests`: passed.
  - `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`:
    passed.
  - `python -m json.tool docs/workstreams/material3-search-view-state-packet-v1/WORKSTREAM.json | Out-Null`:
    passed.
  - `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null`:
    passed.
  - `python tools/check_workstream_catalog.py`: passed, 478 dedicated directories and 47
    standalone markdown files.
