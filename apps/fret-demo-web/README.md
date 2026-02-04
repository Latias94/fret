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

- `?demo=ui_gallery` (or `#ui_gallery`) — full UI Gallery app (pages use `?page=...`)
- `?demo=components_gallery` (or `#components_gallery`) — lightweight examples gallery
- `?demo=emoji_conformance_demo` (or `#emoji_conformance_demo`)
- `?demo=cjk_conformance_demo` (or `#cjk_conformance_demo`)
- `?demo=chart_demo` (or `#chart_demo`)
- `?demo=chart_multi_axis_demo` (or `#chart_multi_axis_demo`)
- `?demo=horizontal_bars_demo` (or `#horizontal_bars_demo`)
- `?demo=plot_demo` (or `#plot_demo`)
- `?demo=plot_image_demo` (or `#plot_image_demo`)
- `?demo=bars_demo` (or `#bars_demo`)
- `?demo=grouped_bars_demo` (or `#grouped_bars_demo`)
- `?demo=stacked_bars_demo` (or `#stacked_bars_demo`)
- `?demo=area_demo` (or `#area_demo`)
- `?demo=candlestick_demo` (or `#candlestick_demo`)
- `?demo=error_bars_demo` (or `#error_bars_demo`)
- `?demo=heatmap_demo` (or `#heatmap_demo`)
- `?demo=histogram_demo` (or `#histogram_demo`)
- `?demo=histogram2d_demo` (or `#histogram2d_demo`)
- `?demo=shaded_demo` (or `#shaded_demo`)
- `?demo=stairs_demo` (or `#stairs_demo`)
- `?demo=stems_demo` (or `#stems_demo`)
- `?demo=linked_cursor_demo` (or `#linked_cursor_demo`)
- `?demo=inf_lines_demo` (or `#inf_lines_demo`)
- `?demo=tags_demo` (or `#tags_demo`)
- `?demo=drag_demo` (or `#drag_demo`)

## Optional bundled fonts (WASM)

For Web/WASM, system fonts are not available. You can opt into extra bundled fonts:

- `cjk-lite-fonts` (default): includes a small subset of `Noto Sans CJK SC` (see `fret-fonts` docs).
- `emoji-fonts`: includes `Noto Color Emoji` (large).

Examples:

- Minimal build (no bundled fonts): `trunk serve --no-default-features`
- Add emoji fonts: `trunk serve --features emoji-fonts`

## Debug teardown

To stop the running demo instance without reloading the page, call this from the browser console:

- `window.fret_demo_destroy()`
