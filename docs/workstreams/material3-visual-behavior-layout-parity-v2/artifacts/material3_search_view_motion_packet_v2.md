# Material3 SearchView Motion Packet v2

Date: 2026-05-28

## Truth

- SearchView motion is Material recipe policy, not a generic overlay primitive. Compose Material3
  models search transitions with `SearchBarState`: spatial progress drives geometry while a second
  content progress drives fade.
- Docked SearchView should fade and vertically expand/shrink its results panel instead of using a
  generic menu scale transform.
- Full-screen SearchView should animate from the collapsed input geometry toward the viewport,
  while initially expanded instances start settled at expanded geometry.

## Sources

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`
  - `SearchBarState` owns `animatable` and `contentAnimatable`.
  - `rememberSearchBarState` uses `SlowSpatial` expand and `DefaultSpatial` collapse defaults.
  - Docked legacy SearchBar uses `DockedEnterTransition = fadeIn + expandVertically` and
    `DockedExitTransition = fadeOut + shrinkVertically`.
  - `FullScreenSearchBarLayout` lerps collapsed input bounds toward viewport-sized expanded
    geometry.

## Artifacts

- `ecosystem/fret-ui-material3/src/foundation/search_motion.rs`
- `ecosystem/fret-ui-material3/src/foundation/mod.rs`
- `ecosystem/fret-ui-material3/src/search_view.rs`
- `ecosystem/fret-ui-material3/tests/search_view_behavior.rs`

## Wiring

- Added a Material-specific `foundation::search_motion` helper with:
  - `drive_search_motion` for search progress/content-alpha channels.
  - `SearchMotionKind::{Docked, FullScreen}` to select Compose-aligned spring families.
  - `search_full_screen_geometry_transform` for collapsed-input-to-viewport affine expansion.
- SearchView now uses search motion instead of `foundation::overlay_motion`:
  - docked overlays derive desired panel height from `motion.progress` and fade via
    `motion.content_alpha`;
  - full-screen overlays derive render transform from the underlay input bounds and fade via
    `motion.content_alpha`.
- Initial `open = true` SearchView starts settled at expanded progress, matching Compose
  `SearchBarState(initialValue = Expanded)`. Closed-to-open and open-to-closed changes animate.

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_docked_overlay_fades_and_expands_on_open_close_frames search_view_full_screen_overlay_expands_from_input_geometry
```

It failed because docked SearchView first-open layout height was already the full `240px`, and
full-screen SearchView had no collapsed-input expansion transform.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --test search_view_behavior search_view_docked_overlay_fades_and_expands_on_open_close_frames search_view_full_screen_overlay_expands_from_input_geometry
cargo nextest run -p fret-ui-material3 --test search_view_behavior
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_search_bar_exposes_stable_part_test_ids material3_search_view_exposes_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_search_bar_suite_goldens_v1 material3_headless_search_view_suite_goldens_v1
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json
python tools/check_workstream_catalog.py
git diff --check
```

## Matrix Impact

- `search_view.motion`: `covered_v2`.
- `search_bar.motion`: covered separately by `material3_search_bar_motion_packet_v2.md`; this
  packet only covers SearchView-owned transition choreography.

## Residual Risk

- Predictive-back SearchBar motion is not modeled yet.
- Full-screen shape morphing is approximated by the geometry transform; there is no dedicated
  corner-radius morph for the overlay surface yet.
- Ordinary standalone SearchBar indication motion is tracked separately; default container color
  motion is a no-op because upstream focused/unfocused default container colors match.
