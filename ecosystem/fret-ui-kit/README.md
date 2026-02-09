# `fret-ui-kit`

Component authoring toolkit for Fret with declarative policies and reusable building blocks.

This is the ecosystem layer that sits above `crates/fret-ui` (mechanisms/contracts). It provides:

- token-driven layout/style surfaces (`UiBuilder`, `LayoutRefinement`, `ChromeRefinement`)
- reusable policy/toolbox primitives (overlays, tooltips, tables, scrolling)
- declarative authoring helpers (`use fret_ui_kit::prelude::*;`)

## Status

Experimental learning project (not production-ready).

## Features

Default features are minimal. Opt in as needed:

- `icons`: integrate with the shared icon registry (`fret-icons`)
- `dnd`: headless drag-and-drop toolbox integration (`fret-dnd`)
- `imui`: integration helpers for immediate-mode authoring frontends
- `recipes`: opinionated helpers closer to recipes than substrate

