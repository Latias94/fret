# fret-devtools

Minimal native DevTools GUI skeleton for Fret diagnostics.

This app hosts a loopback-only WebSocket server and speaks the same transport envelope as
`ecosystem/fret-bootstrap/src/ui_diagnostics.rs`.

## Run

```bash
cargo run -p fret-devtools
```

The app prints a `ws://127.0.0.1:<port>/?fret_devtools_token=...` URL on startup.

## Connect a target app (native)

Set:

- `FRET_DEVTOOLS_WS=ws://127.0.0.1:7331/`
- `FRET_DEVTOOLS_TOKEN=<token>`

and run the target app with `fret-bootstrap` `diagnostics-ws` enabled.

## Connect a target app (web runner)

Add query parameters:

- `fret_devtools_ws=ws://127.0.0.1:7331/`
- `fret_devtools_token=<token>`

Example:

`?fret_devtools_ws=ws://127.0.0.1:7331/&fret_devtools_token=...`

