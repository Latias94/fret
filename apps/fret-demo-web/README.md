# fret-demo-web

WebAssembly demo for the `fret` UI runtime.

## Run locally (recommended: via fretboard)

From the workspace root:

- `cargo run -p fretboard -- dev web --demo ui_gallery`
- `cargo run -p fretboard -- dev web --open --demo custom_effect_v2_web_demo`

This uses Trunk under the hood and prints an `Open: ...` URL once the dev server is listening (so it does not get
buried in build warnings).

## Run locally (with Trunk)

1. Install prerequisites:
   - `rustup target add wasm32-unknown-unknown`
   - `cargo install trunk`

2. Start the dev server:
   - `cd apps/fret-demo-web`
   - `trunk serve`

Then open the URL printed by Trunk (usually `http://127.0.0.1:8080`).

## Select a demo

By default, the web runner starts the components gallery. To launch a specific demo, use the
canonical query route:

- `?demo=ui_gallery` — full UI Gallery app (pages use `?page=...`)
- `?demo=components_gallery` — lightweight examples gallery
- `?demo=simple_todo` — starter-style todo example
- `?demo=emoji_conformance_demo`
- `?demo=cjk_conformance_demo`
- `?demo=chart_demo`
- `?demo=chart_multi_axis_demo`
- `?demo=horizontal_bars_demo`
- `?demo=plot_demo`
- `?demo=plot_image_demo`
- `?demo=bars_demo`
- `?demo=grouped_bars_demo`
- `?demo=stacked_bars_demo`
- `?demo=area_demo`
- `?demo=candlestick_demo`
- `?demo=error_bars_demo`
- `?demo=heatmap_demo`
- `?demo=histogram_demo`
- `?demo=histogram2d_demo`
- `?demo=shaded_demo`
- `?demo=stairs_demo`
- `?demo=stems_demo`
- `?demo=linked_cursor_demo`
- `?demo=inf_lines_demo`
- `?demo=tags_demo`
- `?demo=drag_demo`
- `?demo=external_texture_imports_web_demo` — external image source → render target (web copy path)
- `?demo=custom_effect_v2_web_demo` — CustomV2 end-to-end WebGPU smoke (register + user image input)
- `?demo=custom_effect_v2_lut_web_demo` — CustomV2 LUT authoring template (input image used as LUT)
- `?demo=custom_effect_v2_identity_web_demo` — CustomV2 minimal starter template (mix + input debug)
- `?demo=custom_effect_v2_glass_chrome_web_demo` — CustomV2 glass/chrome recipe variant
- `?demo=custom_effect_v3_web_demo` — CustomV3 renderer-provided sources (src_raw + bounded pyramid)

Compatibility note:

- legacy token hashes such as `#ui_gallery` and `#simple-todo` still resolve for older shared links,
  but new docs/examples should always use `?demo=...`
- legacy query aliases such as `simple-todo` / `simple_todo_demo` are still decoded, but
  `simple_todo` is the canonical ID

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
