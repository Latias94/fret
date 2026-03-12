# `fret-ui-shadcn`

Shadcn/ui-inspired component set and recipes for Fret.

This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
transfer knowledge and recipes directly.

## Status

Experimental learning project (not production-ready).

## When to use

- You want a productive, cohesive component surface for apps (forms, tables, overlays, layouts).
- You want shadcn-style mental models, but in a GPU-first Rust UI runtime (not HTML/CSS).

## Features

- `app-integration`: explicit app helpers under `fret_ui_shadcn::app::{install, install_with, ...}` and advanced hooks under `fret_ui_shadcn::advanced::{...}` (optional)
- `state-selector` / `state-query`: opt into derived/async state helpers
- `state`: enables both selector + query integration

## App integration

Keep the component taxonomy and app wiring separate:

- recipes/components should prefer the curated facade import
  `use fret_ui_shadcn::{facade as shadcn, prelude::*};`
- app-owned setup stays under `fret_ui_shadcn::app::*`
- environment / `UiServices`-boundary hooks stay under `fret_ui_shadcn::advanced::*`
- raw root access stays an explicit escape hatch for low-level/internal use

Example:

```rust
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fret_ui_shadcn::app::install(app);

let _button = shadcn::Button::new("Save");
```

## Upstream references (non-normative)

This crate intentionally mirrors upstream taxonomies and behavior outcomes where practical.
Primary references:

- shadcn/ui (v4 docs + recipes): https://github.com/shadcn-ui/ui
- Radix Primitives (overlay + interaction semantics): https://github.com/radix-ui/primitives
- cmdk (command palette behavior): https://github.com/pacocoursey/cmdk
- Base UI (headless composition patterns): https://github.com/mui/base-ui
- Floating UI (placement vocabulary + collision/shift/flip outcomes): https://github.com/floating-ui/floating-ui
- WAI-ARIA Authoring Practices (APG): https://github.com/w3c/aria-practices

See also:

- [`docs/reference-stack-ui-behavior.md`](../../docs/reference-stack-ui-behavior.md) (how each reference is used)
