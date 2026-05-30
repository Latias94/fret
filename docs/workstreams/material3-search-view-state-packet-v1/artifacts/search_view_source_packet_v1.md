# SearchView Source Packet v1

Date: 2026-05-28
Task: M3SV-010
Status: complete

## Truth

- Search text/query state and expanded presentation state are separate concerns.
- Search has at least two expanded presentations: docked and full-screen.
- Docked expanded search is a bounded surface attached to the input.
- Full-screen expanded search is modal/full-window and should focus the search input.
- Back/Escape-equivalent dismissal collapses the expanded search surface.

## Source Anchors

- Compose `SearchBar` collapsed surface: `SearchBar(state, inputField, ...)`.
- Compose `ExpandedDockedSearchBar`: bounded attached results surface.
- Compose `ExpandedFullScreenSearchBar`: dialog/full-screen expanded results surface.
- Compose `SearchBarState`: expanded/collapsed state and animation progress.
- Compose `BackHandler` / `PredictiveBackHandler`: collapse expanded search on back.

Local file:

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`

## Fret Mapping

- `material_recipe`: owns `SearchViewPresentation`, docked/full-screen composition, header/content
  layout, and stable part ids.
- `kit_policy`: existing modal/dismissible overlay policy owns Escape dismissal and focus restore.
- `diagnostics`: automation/golden/diag tests prove live selectors and expanded presentation.
- `mechanism`: a true platform back navigation event is out of scope unless Escape is insufficient.

## First Slice

Add `SearchViewPresentation::Docked | FullScreen`, keep Docked as default, and route FullScreen
through `OverlayRequest::modal` with a focus-trapped panel and overlay-local search header.

## Residual Risk

- Predictive back progress and shape interpolation are not covered.
- Mobile IME/window inset choreography is not covered.
- The first slice maps platform back to current Escape dismissal because Fret does not yet expose a
  design-system-agnostic platform back event.

## Result

Implemented in `SearchViewPresentation::FullScreen` with modal overlay policy, overlay-local header
focus, Escape collapse, automation-surface selectors, and a `full_screen_open` SearchView headless
golden case.
