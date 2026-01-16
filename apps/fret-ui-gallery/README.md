# fret-ui-gallery

Native + web gallery app for validating `fret-ui-shadcn` component behavior.

## Run (native)

- `cargo run -p fret-ui-gallery`

Optional:

- Start on a specific page: `FRET_UI_GALLERY_START_PAGE=data_table`
- Enable bisect flags: `FRET_UI_GALLERY_BISECT=<u32>`

## Run (web / wasm32)

Use the dedicated web harness:

- `cd apps/fret-ui-gallery-web`
- `trunk serve`

Then open the URL printed by Trunk.

