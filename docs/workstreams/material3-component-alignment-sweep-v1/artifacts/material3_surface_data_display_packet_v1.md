# Material 3 Surface/Data Display Packet v1

Status: closed with known follow-ons
Date: 2026-05-27
Task: M3CAS-090

## Scope

Components audited:

- Badge
- Button
- Card
- CarouselItem
- Divider
- FAB
- List
- ProgressIndicator
- TopAppBar

## Reference Stack

- Material UX intent: low-interaction surfaces should have stable visual chrome, predictable
  semantics, and no hidden policy in the mechanism layer.
- Compose Material3 local references:
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Badge.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Card.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/Divider.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/FloatingActionButton.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ListItem.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/ProgressIndicator.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/AppBar.kt`
  - `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/carousel/Carousel.kt`
- MUI Material local mirror was not present in this worktree, so this packet used Compose and the
  existing Fret Material token/golden surfaces as the local source anchors.

## Truth

- Every low-interaction component exposes a stable root `test_id`; components with recipe-owned
  chrome expose a dotted `.chrome` child where the chrome is inspectable.
- Exact canvas operations are not exposed as fake selectors. Linear ProgressIndicator now has
  truthful rectangular recipe anchors for `track` and `active-track`; circular arcs and
  indeterminate segmented regions remain scene/golden gated until a generic named draw-region
  mechanism is justified.
- Shared state-layer, ripple, and minimum target behavior remain in Material foundation; visual
  surface composition stays in component recipes.
- TopAppBar scroll behavior remains recipe-owned. The later scroll diagnostics packet proved the
  promoted gallery scroll scenarios without broader policy pressure.
- Badge TopRight placement requires an explicit anchor-size contract for deterministic geometry.

## Artifacts

- `ecosystem/fret-ui-material3/src/badge.rs`
- `ecosystem/fret-ui-material3/src/foundation/test_id.rs`
- `ecosystem/fret-ui-material3/src/progress_indicator.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `ecosystem/fret-ui-material3/tests/radio_alignment.rs`
- `goldens/material3-headless/v1/material3-badge.*.json`
- `goldens/material3-headless/v1/material3-divider.*.json`
- `goldens/material3-headless/v1/material3-progress-indicator.*.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

- `Badge::anchor_size` makes deterministic anchor placement explicit. `navigation_anchor_size`
  remains as the existing convenience wrapper.
- `material3_surface_data_display_expose_stable_part_test_ids` renders Badge, Button, Card,
  CarouselItem, Divider, FAB, List, LinearProgressIndicator, CircularProgressIndicator, and
  TopAppBar together and proves their stable automation targets are live.
- LinearProgressIndicator exposes hidden diagnostic anchors for `linear_progress_indicator.track`
  and `linear_progress_indicator.active-track` through absolute layout regions.
- Existing headless suites continue to own the visual/motion evidence for Badge, Divider, FAB, List,
  ProgressIndicator, TopAppBar, CarouselItem, and the broad controls/card/button surface.
- The promoted TopAppBar scroll gallery script owns pinned, enter-always, enter-always-settle,
  exit-until-collapsed, and exit-until-collapsed-settle diagnostics.

## Layer Classification

- `material_recipe`: Badge anchoring and chrome; Button/Card/CarouselItem/FAB surface composition;
  Divider orientation/thickness; List item semantics and selected state; ProgressIndicator canvas
  paint and rectangular diagnostic anchors; TopAppBar variants/actions/scroll state.
- `material_foundation`: existing state-layer/ripple/minimum target helpers used by interactive
  Button/Card/CarouselItem/FAB/List surfaces.
- `diagnostics/test_harness`: stable selector proofs, rectangular progress anchors, and refreshed
  Badge/Divider/ProgressIndicator headless goldens.
- `kit_policy`: no new shared policy was proven. List roving/selection and TopAppBar scroll behavior
  should move only if another design system proves reuse pressure.
- `mechanism`: no `crates/*` change was needed.

## Proof

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_badge_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_divider_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_progress_indicator_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_fab_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_list_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_top_app_bar_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_carousel_item_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
- `cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment top_app_bar_exposes_toolbar_semantics_role`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-top-app-bar-scroll-screenshots.json --dir target/fret-diag/material3-top-app-bar-scroll-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`

## Residual Risk

- ProgressIndicator circular arcs and indeterminate segments remain canvas paint operations, not
  queryable automation parts.
- TopAppBar scroll diagnostics are closed for current gallery scenarios by
  `docs/workstreams/material3-top-app-bar-scroll-diagnostics-packet-v1/`.
- Badge text TopRight placement uses the explicit anchor box and token large-size as the stable
  layout contract; text-measured edge alignment can be refined only if a real product surface needs
  it.
