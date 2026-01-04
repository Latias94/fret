# fret-demo-web

WebAssembly demo for the `fret` UI runtime.

## Run locally (with Trunk)

1. Install prerequisites:
   - `rustup target add wasm32-unknown-unknown`
   - `cargo install trunk`

2. Start the dev server:
   - `cd apps/fret-demo-web`
   - `trunk serve`

Then open the URL printed by Trunk (usually `http://127.0.0.1:8080`).

## Debug teardown

To stop the running demo instance without reloading the page, call this from the browser console:

- `window.fret_demo_destroy()`

