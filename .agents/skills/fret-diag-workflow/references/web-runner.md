# Web/WASM runner workflow (devtools-ws)

For web apps, `fretboard diag run` (filesystem-trigger transport) is usually not applicable. Prefer the devtools WS loopback:

1. Start the WS hub (prints a token):
   - `cargo run -p fret-devtools-ws`
2. Serve the WASM app:
   - `cd apps/fret-ui-gallery-web && trunk serve --port 8080`
3. Open the app with query params:
   - `http://127.0.0.1:8080/?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=<token>`
4. Run a script and materialize an export under `.fret/diag/exports/<timestamp>/bundle.json`:
   - `cargo run -p fret-diag-export -- --script tools/diag-scripts/<script>.json --token <token>`

Notes:

- `fret-diag-export` is the headless-friendly path to “pull” bundles from a web session.
- If a script never calls `capture_bundle`, you may get no exports.
- Capabilities are session-advertised; missing features should fail fast via required capabilities rather than time out.
