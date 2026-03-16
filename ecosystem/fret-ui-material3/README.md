# `fret-ui-material3`

Material Design 3 (and Expressive) component surface for Fret.

This crate is a **design-system surface** intended to mirror Material 3 visual and interaction
outcomes while keeping `crates/fret-ui` focused on mechanisms rather than Material-specific policy.

## Status

Experimental learning project (not production-ready).

## When to use

- You want a Material 3 / Material Expressive component surface on top of Fret.
- You want theme-token-driven components rather than ad-hoc styling.
- You want app-facing widgets to follow the same action-first authoring story as the rest of the
  action-first/view-runtime workstream.

## Features

- `state-selector`: opt into derived-state helper integration
- `state-query`: opt into async/query helper integration
- `state`: enables both selector + query integration

## Authoring note

- Prefer action-first public spellings on normal app-facing surfaces.
- Keep command-shaped or lower-level spellings only where the surface is intentionally exposing a
  deeper compatibility/interop boundary.
- For example, snackbar-style actions should prefer the explicit action-first naming path in
  default-facing examples/docs.
- Default-facing clickable families such as `Button`, `Fab`, `IconButton`, `Checkbox`, `Switch`,
  `Radio`, `AssistChip`, `SuggestionChip`, `FilterChip`, and `InputChip` now expose
  `action(...)` directly; prefer that over wiring `.on_activate(cx.actions().dispatch::<A>())`
  when you only need a stable unit action on the app-facing lane.

## Icons

- Material 3 widgets consume semantic `IconId` / `ui.*` ids; they do not choose a vendor pack as
  part of the component contract.
- This crate does not install a default icon provider for you.
- App/bootstrap code should install a pack explicitly (`fret_icons_lucide::app::install`,
  `fret_icons_radix::app::install`, or your own bundle surface).
- If a reusable ecosystem bundle depends on Material 3 plus a specific icon pack, keep that
  composition on one installer/bundle surface so the app composes one named dependency bundle.

Example:

```rust
use fret_icons::ids;

fret_icons_lucide::app::install(app);

let _button = fret_ui_material3::Button::new("Search")
    .leading_icon(ids::ui::SEARCH);
```

## Upstream references (non-normative)

Primary references:

- Material Design 3: https://m3.material.io/
- Material Web: https://github.com/material-components/material-web
- Jetpack Compose Material 3: https://developer.android.com/jetpack/compose/designsystems/material3
- MUI Material UI: https://github.com/mui/material-ui

See also:

- [`docs/reference-stack-ui-behavior.md`](../../docs/reference-stack-ui-behavior.md)
- [`docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`](../../docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md)
