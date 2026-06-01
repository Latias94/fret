# Material 3 SearchView Edge and Full-Screen Composition v1

Status: Complete
Owner: Codex goal `Material3 SearchView edge/collision and full-screen composition hardening`
Started: 2026-06-01
Completed: 2026-06-01

This follow-up closes the SearchView residual from the Material 3 composition-hardening lane. The
target is not a new SearchView policy surface; it is stronger evidence that the existing Material
recipe behaves correctly when used like an app surface near viewport edges and beside sibling
menus.

## Truth

- A docked `SearchView` near the bottom of a viewport must use the shared popper solver outcome:
  flip above the input when bottom space is insufficient, clamp height to available space, and stay
  inside the window collision boundary.
- A full-screen `SearchView` must use the modal overlay layer, keep editing focus in its overlay
  header, preserve query state when dismissed, and block sibling menu activation while the modal is
  open.
- After the full-screen `SearchView` is dismissed, the sibling `DropdownMenu` must be able to open,
  take focus, and expose menu semantics without reopening search.
- Gallery examples own width and layout constraints. `SearchView` remains the Material recipe for
  search presentation and overlay request wiring; it does not absorb page-specific sizing policy.

## Artifacts

- Component tests:
  `ecosystem/fret-ui-material3/tests/search_view_behavior.rs`
  (`search_view_docked_overlay_flips_and_clamps_near_viewport_bottom`)
- Composition tests:
  `ecosystem/fret-ui-material3/tests/material3_overlay_interactions.rs`
  (`search_view_full_screen_blocks_sibling_menu_until_dismissed`)
- Gallery repro:
  `apps/fret-ui-gallery/src/ui/snippets/material3/menu.rs`
  (`ui-gallery-material3-menu-search-bottom`,
  `ui-gallery-material3-menu-search-full-screen`)
- Diag script:
  `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-view-edge-fullscreen-composition.json`
- Gallery surface gate:
  `apps/fret-ui-gallery/tests/material3_search_view_surface.rs`

## Findings

- No core `fret-ui` or shared overlay solver bug was found in this pass. The focused bottom-edge
  test passed once added, confirming the existing `popper_content_layout_sized` path already flips
  and clamps correctly for SearchView.
- The incomplete part was coverage and real-page composition: existing gates proved docked
  SearchView + sibling Menu, but not bottom-edge placement or full-screen modal blocking.
- The full-screen composition test confirms the existing Material recipe already uses modal overlay
  ownership correctly: pointer activation aimed at the sibling menu does not toggle that menu while
  the full-screen SearchView is open.
- Diag authoring detail: `click_stable` is the wrong tool for proving a blocked underlay click,
  because it treats the modal barrier as a target mismatch. The pointer-blocking proof lives in the
  Rust event test; the real-page diag proves modal state, absence of the sibling menu while modal,
  and post-dismiss menu focus.

## Validation

- `cargo fmt -p fret-ui-material3 -p fret-ui-gallery`
- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-view-edge-fullscreen-composition.json`
- `cargo test -p fret-ui-material3 --features diagnostics --test search_view_behavior search_view_docked_overlay_flips_and_clamps_near_viewport_bottom -- --exact`
- `cargo test -p fret-ui-material3 --features diagnostics --test material3_overlay_interactions search_view_full_screen_blocks_sibling_menu_until_dismissed -- --exact`
- `cargo test -p fret-ui-gallery --test material3_search_view_surface`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --test search_view_behavior --test material3_overlay_interactions --no-deps -- -D warnings`
- `cargo check -p fret-ui-gallery --features gallery-material3`
- `python tools/check_workstream_catalog.py`
- `python tools/check_layering.py`
- `git diff --check`
- `.\target\debug\fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-view-edge-fullscreen-composition.json --dir target/fret-diag-material3-search-view-edge-fullscreen-composition --session-auto --timeout-ms 480000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  passed with `run_id=1780291459652`; session:
  `target/fret-diag-material3-search-view-edge-fullscreen-composition/sessions/1780291434681-234888`.
