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

## Upstream references (non-normative)

Primary references:

- Material Design 3: https://m3.material.io/
- Material Web: https://github.com/material-components/material-web
- Jetpack Compose Material 3: https://developer.android.com/jetpack/compose/designsystems/material3
- MUI Material UI: https://github.com/mui/material-ui

See also:

- [`docs/reference-stack-ui-behavior.md`](../../docs/reference-stack-ui-behavior.md)
- [`docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`](../../docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md)
