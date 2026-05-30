# Material 3 SearchBar Headless Automation Packet v1

Date: 2026-05-28

## Truth

- SearchBar exposes stable dotted ids for the root, chrome, and icon slots.
- The recipe stays in Material3; no shared kit or mechanism gap is needed.
- Headless goldens cover the interactive state surface that matters for SearchBar.

## Artifacts

- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/search_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
- `ecosystem/fret-ui-material3/src/tokens/v30.rs`
- `ecosystem/fret-ui-material3/src/bin/material3_token_import.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_selector_audit_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_field_family_behavior_packet_v1.md`
- `docs/workstreams/material3-search-view-state-packet-v1/artifacts/search_view_source_packet_v1.md`

## Wiring

- `SearchBar` owns the pill field chrome and icon slots.
- `automation_surface` asserts the live selector surface.
- `radio_alignment` asserts the headless states across schemes and scales.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1`

## Residual Risk

- If a future SearchBar change introduces new visible chrome or interaction policy, it should be
  treated as a new SearchBar follow-on only if a real regression is proven.
- SearchView overlay and presentation drift belongs in the SearchView packet.
