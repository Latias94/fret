# Fret Bundle Viewer (offline)

Offline web viewer for Fret `bundle.json` diagnostics exports.

## Why this exists

Fret targets editor-grade UI (multi-window, overlays/barriers, focus/capture routing). Debugging is often about:

- picking the right snapshot/frame,
- inspecting the semantics tree,
- checking overlay routing (barrier root / layer roots),
- correlating events + perf data,
- copying selectors / generating minimal script steps for repro.

This viewer is meant to make those workflows fast and shareable.

## Run (pnpm)

```powershell
cd tools/fret-bundle-viewer

# optional proxy
$env:HTTP_PROXY='http://127.0.0.1:10809'
$env:HTTPS_PROXY='http://127.0.0.1:10809'

pnpm install
pnpm dev
```

## Notes

- Designed to run fully offline after install; no telemetry is enabled by default.
- `bundle.json` format is treated as best-effort / forward-compatible (unknown fields are ignored).

