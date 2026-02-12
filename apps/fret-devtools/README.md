# fret-devtools

Minimal native DevTools GUI skeleton for Fret diagnostics.

This app hosts a loopback-only WebSocket server and speaks the same transport envelope as
`ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.

## Run

```bash
cargo run -p fret-devtools
```

The app prints a `ws://127.0.0.1:<port>/?fret_devtools_token=...` URL on startup.

When multiple apps connect, use the **Session** selector in the toolbar to target a specific app.

## Script Studio

- **Refresh Scripts** scans:
  - workspace scripts: `tools/diag-scripts/*.json`
  - user scripts: `.fret/diag/scripts/*.json`
- **Fork** copies the loaded `tools/diag-scripts/*.json` file into `.fret/diag/scripts/` (to avoid editing workspace scripts by default).
- **Save** writes the current editor text back to the loaded `.fret/diag/scripts/*.json` file.
- **Apply Pick** replaces a JSON pointer (e.g. `/steps/0/target`) with the best selector from the latest `pick.result`.
- **Run & Pack** runs the current script and packs the latest dumped bundle into `.fret/diag/packs/*.zip`.
- **Pack last bundle** packs the latest dumped bundle into `.fret/diag/packs/*.zip` (useful after a manual dump/run).

## Offline bundle viewer

Run the viewer:

```powershell
cd tools/fret-bundle-viewer
pnpm install
pnpm dev
```

Then use **Open viewer** in DevTools and drop/open the generated `.zip` in the viewer UI.

## Web runner note

When the target app runs on the web runner, `bundle.dumped` includes an in-memory `bundle` payload. DevTools
materializes it into `.fret/diag/exports/` before packing, so `Run & Pack` still works without filesystem access
inside the browser.

## Connect a target app (native)

Set:

- `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/`
- `FRET_DEVTOOLS_TOKEN=<token>`

and run the target app with `fret-bootstrap` `diagnostics-ws` enabled.

## Optional: filesystem transport (native-only)

If you want to drive the existing file-trigger workflow (no WS bridge), run DevTools with:

- `FRET_DEVTOOLS_TRANSPORT=fs`
- `FRET_DIAG_DIR=target/fret-diag` (optional; defaults to `target/fret-diag`)

In this mode DevTools polls `FRET_DIAG_DIR` for `latest.txt`, `pick.result.json`, `script.result.json`,
and `screenshots.result.json` updates (and writes `*.touch` triggers for inspect/pick/script/bundle dump).

## Connect a target app (web runner)

Add query parameters:

- `fret_devtools_ws=ws://127.0.0.1:7331/`
- `fret_devtools_token=<token>`

Example:

`?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=...`
