# Material3 SearchView Full-Screen Header Layout Packet v2

Date: 2026-05-28
Task: M3PV2-031

## Truth

- Full-screen SearchView header layout is intrinsic Material recipe layout, not caller-owned page
  layout.
- Compose full-screen search places the input field after `SearchBarVerticalPadding = 8.dp` and
  places the search content after the input field plus another 8px bottom padding.
- Material Web v30 exposes the same settled outcome as
  `md.comp.search-view.full-screen.header.container.height = 72`.
- Stable automation ids should target the overlay panel, header slot, divider, and body separately
  so future a11y/motion packets can reuse the same geometry anchors.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
  - `FullScreenSearchBarLayout` computes `topPadding` from top insets plus
    `SearchBarVerticalPadding`.
  - It measures the input field at `InputFieldHeight` and places content at
    `animatedTopPadding + inputFieldPlaceable.height + bottomPadding`.
  - `SearchBarVerticalPadding = 8.dp`; `InputFieldHeight` resolves to the 56dp search-bar
    container height.
- `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`
  - `md.comp.search-view.full-screen.header.container.height = 72`.

## Artifacts

- `ecosystem/fret-ui-material3/src/search_view.rs`
- `ecosystem/fret-ui-material3/src/tokens/search_view.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/search_view_behavior.rs`
- `goldens/material3-headless/v1/material3-search-view.*.json`

## Wiring

- Full-screen SearchView now wraps its overlay-local header SearchBar in a token-driven
  `overlay.header-slot`.
- The header slot is 72px tall and gives the 56px SearchBar header 8px top and bottom padding.
- Full-screen and docked SearchView overlays expose stable `overlay.divider` and `overlay.body`
  part ids.
- The full-screen divider and body are placed after the 72px header slot.

## Proof

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_view_exposes_stable_part_test_ids
```

The new gate failed because the overlay did not expose stable divider/body part ids and full-screen
SearchView had no 72px header slot.

Green after fix:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test search_view_behavior
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_view_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --lib search_view
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- SearchView accessibility relations remain open: this packet does not add explicit search-result
  ownership or active-descendant semantics.
- SearchView motion remains open: predictive back and fixed-timestep open/close transitions still
  need a dedicated packet.
- SearchBar default width/focus affordance still deserves a source-backed packet; this packet only
  changed SearchView overlay layout.
