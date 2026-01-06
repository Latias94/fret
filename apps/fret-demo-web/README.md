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

## Select a demo

By default, the web runner starts the components gallery. To launch a specific demo, use a URL
hash or query string:

- `?demo=plot_demo` (or `#plot_demo`)
- `?demo=bars_demo` (or `#bars_demo`)
- `?demo=grouped_bars_demo` (or `#grouped_bars_demo`)
- `?demo=stacked_bars_demo` (or `#stacked_bars_demo`)
- `?demo=area_demo` (or `#area_demo`)
- `?demo=candlestick_demo` (or `#candlestick_demo`)
- `?demo=error_bars_demo` (or `#error_bars_demo`)
- `?demo=heatmap_demo` (or `#heatmap_demo`)
- `?demo=histogram_demo` (or `#histogram_demo`)
- `?demo=shaded_demo` (or `#shaded_demo`)
- `?demo=stairs_demo` (or `#stairs_demo`)
- `?demo=stems_demo` (or `#stems_demo`)
- `?demo=linked_cursor_demo` (or `#linked_cursor_demo`)
- `?demo=inf_lines_demo` (or `#inf_lines_demo`)

## Debug teardown

To stop the running demo instance without reloading the page, call this from the browser console:

- `window.fret_demo_destroy()`

