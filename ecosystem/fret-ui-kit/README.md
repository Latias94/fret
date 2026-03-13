# `fret-ui-kit`

Component authoring toolkit for Fret with declarative policies and reusable building blocks.

This is the ecosystem layer that sits above `crates/fret-ui` (mechanisms/contracts). It provides:

- token-driven layout/style surfaces (`UiBuilder`, `LayoutRefinement`, `ChromeRefinement`)
- reusable policy/toolbox primitives (overlays, tooltips, tables, scrolling)
- declarative authoring helpers (`use fret_ui_kit::prelude::*;`)

Use `fret-ui-kit`'s broad prelude when you depend on this crate directly as a substrate/policy
layer. If you are intentionally consuming the higher-level `fret` facade for reusable component
code, prefer `use fret::component::prelude::*;` so app-facing builder/runtime names stay off the
default teaching surface.

## Status

Experimental learning project (not production-ready).

## Features

Default features are minimal. Opt in as needed:

- `icons`: integrate with the shared icon registry (`fret-icons`)
- `dnd`: headless drag-and-drop toolbox integration (`fret-dnd`)
- `imui`: integration helpers for immediate-mode authoring frontends
- `recipes`: opinionated helpers closer to recipes than substrate

## References / Thanks

This project is inspired by many open-source projects. Thanks to the authors and contributors of:

- shadcn/ui: https://github.com/shadcn-ui/ui
- Radix UI Primitives: https://github.com/radix-ui/primitives
- MUI Base UI: https://github.com/mui/base-ui
- Floating UI: https://github.com/floating-ui/floating-ui
- Animata: https://github.com/codse/animata
- Motion (motion.dev): https://github.com/motiondivision/motion
- Flutter: https://github.com/flutter/flutter
- Dear ImGui: https://github.com/ocornut/imgui
- dear-imgui-rs: https://github.com/Latias94/dear-imgui-rs
