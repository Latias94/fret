# Shadcn Web Goldens (JSON)

Goal: generate stable, reviewable JSON “goldens” from the upstream shadcn/ui (React) rendering,
capturing both layout geometry (DOM rects) and computed styles, so Fret can validate its typed
Tailwind + Radix-aligned primitives without relying on eyeballing.

## What gets exported

For each component page, the exporter writes a JSON file with:

- a DOM subtree rooted at `[data-fret-golden-target]` (added in the view wrapper),
- per-node `getBoundingClientRect()` relative to the root,
- a whitelist of `window.getComputedStyle(...)` fields (layout + a few visuals),
- selected accessibility-related attrs (`role`, `aria-*`, `data-state`, ...).

## Prerequisites

- `pnpm`
- `repo-ref/ui/apps/v4` dependencies installed
- a shadcn v4 **production** server (`next start`) running locally

## Run

1) Install deps:

`pnpm -C repo-ref/ui/apps/v4 install`

2) Build the app (Terminal A):

`pnpm -C repo-ref/ui/apps/v4 build`

3) Start a production server (Terminal A):

`$env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020`

4) Extract goldens (Terminal B):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts button-default tabs-demo --baseUrl=http://localhost:4020`

Extract *all* routable new-york-v4 pages (defaults match `/view/[style]/[name]`: block+component+example):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts --all --update --baseUrl=http://localhost:4020`

On the current setup, `--all` generates `362` JSON files under `goldens/shadcn-web/v4/new-york-v4/`.

If the extracted `computedStyle` looks like browser defaults (e.g. `<button>` has `display:
inline-block`, `borderTopWidth: 2px`), your dev server is likely not producing Tailwind utilities.
In that case, prefer a production build:

`pnpm -C repo-ref/ui/apps/v4 build; pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020`

Note: the extractor intentionally refuses to run against a dev server (it detects `hmr-client`),
because turbopack dev output does not expose stable asset URLs (and computed styles become unreliable).

If you want to keep a dev server running on `:4000`, start production on a different port and pass
`--baseUrl=...` to the extractor script.


Output directory (default):

`goldens/shadcn-web/v4/new-york-v4/*.json`

## Consume from Rust

Minimal web-golden ingest smoke:

`cargo nextest run -p fret-ui-shadcn --test web_goldens_smoke`

Button “web vs Fret” pipeline (writes no files by default):

`cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`

To emit a JSON comparison report:

`$env:WRITE_WEB_REPORT='1'; cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`

## Layout conformance (geometry-first)

For layout-engine refactors, prefer geometry-first assertions (rects + spacing invariants) over
pixel diffs. See: `docs/audits/shadcn-web-layout-conformance.md`.

## Options

- `--style=new-york-v4`
- `--themes=light,dark` (default)
- `--modes=closed,open` (default: `closed`)
- `--open` (shorthand for `--modes=closed,open`)
- `--openSelector=<css>` (optional override for the "open overlay" trigger)
- `--baseUrl=http://localhost:4000`
- `--all` (env: `ALL_GOLDENS=1`)
- `--types=registry:block,registry:component,registry:example` (env: `TYPES=...`)
- `--outDir=<path>`
- `--update` (overwrite existing files; env: `UPDATE_GOLDENS=1`)
- `--timeoutMs=60000` (env: `TIMEOUT_MS=60000`)
