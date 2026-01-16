# fret-ui-gallery-web

Dedicated WebAssembly harness for `fret-ui-gallery`.

## Run locally (with Trunk)

1. Install prerequisites:
   - `rustup target add wasm32-unknown-unknown`
   - `cargo install trunk`

2. Start the dev server:
   - `cd apps/fret-ui-gallery-web`
   - `trunk serve`

Then open the URL printed by Trunk (usually `http://127.0.0.1:8080`).

## Select a page

`fret-ui-gallery` reads the initial page from the URL on web targets:

- `?page=data_table`
- `#page=data_table`
- `?start_page=data_table`
- `#start_page=data_table`

## Debug teardown

To stop the running instance without reloading the page, call this from the browser console:

- `window.fret_ui_gallery_destroy()`

