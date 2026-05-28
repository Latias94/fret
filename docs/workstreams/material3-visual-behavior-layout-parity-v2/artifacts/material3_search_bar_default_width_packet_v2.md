# Material3 SearchBar Default Width Packet v2

Date: 2026-05-28
Task: M3PV2-032

## Truth

- Ordinary SearchBar default width is intrinsic Material recipe layout because Compose's default
  `SearchBarDefaults.InputField` owns a `sizeIn` constraint.
- Ordinary SearchBar should use a 56px container height and clamp default width to the Compose
  360..720px range when the parent offers more space.
- SearchView-owned headers are not ordinary SearchBars for width ownership; full-screen and docked
  SearchView layouts control those header widths.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
  - `SearchBarDefaults.InputField` applies `Modifier.sizeIn(minWidth = SearchBarMinWidth,
    maxWidth = SearchBarMaxWidth, minHeight = InputFieldHeight)`.
  - `SearchBarMinWidth = 360.dp`.
  - `SearchBarMaxWidth = 720.dp`.
  - `InputFieldHeight = SearchBarTokens.ContainerHeight`, which resolves to 56dp in Material Web
    v30 tokens.

## Artifacts

- `ecosystem/fret-ui-material3/src/search_bar.rs`
- `ecosystem/fret-ui-material3/src/tokens/search_bar.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`

## Wiring

- SearchBar tokens now expose Fret-local Material metrics for the Compose width defaults:
  `md.sys.fret.material.search-bar.container.min-width` and
  `md.sys.fret.material.search-bar.container.max-width`.
- Ordinary `SearchBar` applies those min/max width constraints to the pressable root layout.
- `SearchBarHeaderTokens::SearchView` deliberately bypasses the ordinary SearchBar width clamp so
  SearchView overlay layout remains the width owner.

## Proof

Red before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids
```

The new gate failed because wide-parent SearchBar chrome expanded to `916px` instead of clamping to
`720px`.

Green after fix:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test search_view_behavior
cargo nextest run -p fret-ui-material3 --lib search_bar search_view
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

## Residual Risk

- SearchBar focus affordance and open/focus choreography still need a dedicated behavior packet.
- SearchBar motion remains open; this packet only proves settled default width.
- Compact-width overflow and adaptive placement should be covered by a future responsive SearchBar
  scenario if Fret adds phone-class examples.
