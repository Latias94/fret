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

## Build / Preview

```powershell
cd tools/fret-bundle-viewer

pnpm build
pnpm preview
```

The viewer is a static Vite app (no server required for production hosting).

## Inputs

- `bundle.json`: a diagnostics export produced by `fretboard diag pack`.
- `bundle.zip`: zipped bundle export (optionally includes `_root/*.json` artifacts).
- Screenshots: if the zip contains `*/_root/screenshots/*.png` (recommended) or `*/screenshots/*.png`, the `Overlay` tab can show them as a background under semantics bounds.
- If a screenshots `manifest.json` is present, the viewer can auto-select the matching screenshot for the currently selected snapshot.

## Exports

- Markdown summary (`*-summary.md`) — intentionally kept in English for sharing in issues/PRs.
- `triage.json` (`*.triage.json`) — machine-friendly summary for automation and quick triage.

## Notes

- Designed to run fully offline after install; no telemetry is enabled by default.
- `bundle.json` format is treated as best-effort / forward-compatible (unknown fields are ignored).

## Dev notes

- UI text is fully localizable (English/Chinese) via `tools/fret-bundle-viewer/lib/i18n.ts`.
- The Details panel includes an `Overlay` tab that visualizes semantics bounds and debug signals (hit chain, layer roots, barrier root, pointer).
