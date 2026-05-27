# Material 3 SearchView State Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Result

The lane is closed.

Fret `SearchView` now has an explicit full-screen presentation mode while keeping the existing
docked behavior as the default. The new mode uses existing modal overlay policy, focuses an
overlay-local search header input, exposes stable part ids, and collapses on Escape.

## Completed Work

- Added `SearchViewPresentation::{Docked, FullScreen}`.
- Added `SearchView::presentation(...)`, `SearchView::full_screen()`, and `SearchView::docked()`.
- Implemented full-screen SearchView with a modal overlay, focus trap, overlay-local header, and
  stable `*.overlay.header*` part ids.
- Prevented close autofocus from restoring focus to the collapsed underlay input, which would
  immediately reopen the full-screen SearchView through the existing focus-gained policy.
- Added `search_view_behavior` coverage for modal kind, stable ids, overlay-local focus, and Escape
  collapse.
- Extended `automation_surface` SearchView selectors.
- Added `full_screen_open` to the SearchView headless golden suite.
- Updated the broad Material3 component matrix row for `search_view`.

## Gate Evidence

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
- `python tools/check_workstream_catalog.py`: passed, 478 dedicated directories and 47 standalone
  markdown files.

## Boundary Notes

- No `crates/*` contract changed.
- No new generic platform-back event was added.
- Predictive back gesture progress, shape interpolation, mobile IME insets, and top app bar search
  behavior remain out of scope.

## Follow-On Policy

Open a new lane only for a concrete product or diagnostics need:

- generic platform back/navigation event contract,
- predictive back gesture progress,
- full top-app-bar search behavior,
- mobile IME/window inset choreography.
