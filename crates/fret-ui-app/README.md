# `fret-ui-app`

UI-oriented app integration layer for wiring `fret-app` and `fret-ui` surfaces.

This crate exists to keep app/demo/editor code ergonomic (type aliases + re-exports) while
allowing `fret-ui` to remain host-generic and independent from `fret-app`.

In particular it provides `UiTree = fret_ui::UiTree<fret_app::App>`.

## Status

Experimental learning project (not production-ready).

