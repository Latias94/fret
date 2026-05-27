# Material 3 Foundation Consolidation v1

Status: closed with known follow-ons
Date: 2026-05-27
Task: M3CAS-100

## Scope

This pass reviewed packet evidence from navigation, fields, overlays, choice controls, and
surface/data-display components. The only repeated logic with enough consumer proof for a low-risk
shared refactor was stable dotted part-id generation.

## Consolidated

- Added `foundation::test_id::chrome_part_test_id`.
- Added `foundation::test_id::optional_part_test_id`.
- Added `foundation::test_id::optional_chrome_part_test_id`.
- Replaced hand-written `.chrome` string formatting in:
  - `foundation::interactive_size`
  - `CarouselItem`
  - `DialogAction`
  - `List`
  - `MenuItem`
  - `NavigationDrawer`
  - `Select`
  - `Switch`

## Not Consolidated

- State-layer/ripple remains in `foundation::indication`; no new wrapper was added because the
  packet evidence still shows meaningful component-local geometry and token inputs.
- Minimum interactive sizing remains in `foundation::interactive_size`; no new policy surface was
  needed.
- List and chip roving remain recipe-owned until another design system proves shared policy reuse.
- ProgressIndicator canvas internals remain scene/golden-gated until named draw-region diagnostics
  exist.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
- `cargo nextest run -p fret-ui-material3 --test select_behavior`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment navigation_drawer_roving_skips_disabled_and_updates_model`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`

## Diagnostic Note

`material3_headless_navigation_suite_goldens_v1` currently reports stale geometry drift unrelated to
the part-id helper refactor. The narrower NavigationDrawer behavior gate plus the automation-surface
selector gate were used for this task instead of refreshing a broad navigation golden during a helper
cleanup.

## Residual Risk

- Additional selector helpers should be added only when two or more consumers prove the same dotted
  part-id shape.
- NavigationDrawer still needs a proper component packet for modal/drawer visuals, overlay
  behavior, and broad navigation goldens.
