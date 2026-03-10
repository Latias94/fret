# fret-ui-gallery

Native + web gallery app for validating `fret-ui-shadcn` component behavior.

Surface taxonomy:

- **Advanced / reference**: use this app for component discovery, conformance checks, and product
  polish after the default onboarding ladder.
- **Not the default learning surface**: start with `hello`, `simple-todo`, and `todo` first if you
  are learning the authoring model.

Recommended order:

1. `hello`
2. `simple-todo`
3. `todo`
4. then use `fret-ui-gallery` for recipe discovery, parity checks, and polish validation

## Features

By default the gallery only exposes a small set of Core pages + the Shadcn component set.

- `gallery-dev`: shows internal harness/debug pages (perf torture pages, AI/Magic spikes, etc).
- `gallery-material3`: enables Material 3 pages (pulls in optional `fret-ui-material3`).
- `gallery-full`: enables everything (`gallery-dev` + `gallery-material3`).

## Authoring Notes

- Prefer ecosystem helpers for app/UI text: `fret_ui_kit::ui::{label, text, text_block}`.
- Reserve `cx.text(...)` / `TextProps::new(...)` for mechanism-level harnesses and debugging surfaces.

## Run (native)

- `cargo run -p fret-ui-gallery`

Optional:

- Enable internal pages: `cargo run -p fret-ui-gallery --features gallery-dev`
- Enable Material 3 pages: `cargo run -p fret-ui-gallery --features gallery-material3`
- Enable all pages: `cargo run -p fret-ui-gallery --features gallery-full`
- Start on a specific page: `FRET_UI_GALLERY_START_PAGE=data_table`
- Enable bisect flags: `FRET_UI_GALLERY_BISECT=<u32>`

## Diagnostics

Lite mode smoke gate (first frame + basic navigation):

- `cargo run -p fretboard -- diag suite ui-gallery-lite-smoke --launch -- cargo run -p fret-ui-gallery`

## Run (web / wasm32)

Use the dedicated web harness:

- `cd apps/fret-ui-gallery-web`
- `trunk serve`

Then open the URL printed by Trunk.
