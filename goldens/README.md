# Goldens

This directory stores machine-generated “golden” reference artifacts used for conformance and
regression testing.

## Layout

- `goldens/shadcn-web/`: goldens extracted from the upstream shadcn/ui React app (web runtime).
  - `goldens/shadcn-web/v4/new-york-v4/*.json`: per-component JSON describing:
    - a DOM subtree rooted at `[data-fret-golden-target]`,
    - per-node layout (`getBoundingClientRect`, relative to the root),
    - a whitelist of `getComputedStyle` fields,
    - selected `aria-*` / `data-*` attributes.
- `goldens/tailwind-spec/`: hand-authored conformance cases for Tailwind class parsing into typed tokens.
  - `goldens/tailwind-spec/v1/*.json`: per-case `classes[]` + expected normalized layout tokens.
- `goldens/material3-headless/`: renderer-agnostic “visual outcome” snapshots for Material 3 components.
  - `goldens/material3-headless/v1/*.json`: per-scene `SceneOp` signatures + quad paint/geometry.

## Generate web goldens (shadcn/ui v4)

See `docs/shadcn-web-goldens.md:1` for the full workflow.

Quickstart:

1) Install deps:

`pnpm -C repo-ref/ui install`

2) Build the local `shadcn` workspace package (required on fresh checkouts):

`pnpm -C repo-ref/ui --filter shadcn build`

3) Install a browser for puppeteer (only if you don't have a local Chrome/Edge):

`pnpm -C repo-ref/ui dlx puppeteer browsers install chrome`

4) Build + start a shadcn production server (Terminal A):

Important: `pnpm -C repo-ref/ui/apps/v4 build` currently defaults to Turbopack and may fail to
resolve some transitive Radix deps under pnpm on Windows. Prefer a webpack production build:

If your environment cannot reach Google Fonts during `next build`, you can still build shadcn v4
offline by using Next's `NEXT_FONT_GOOGLE_MOCKED_RESPONSES` hook (maps requested families to local
Windows font files):

`cd repo-ref/ui/apps/v4; $env:NEXT_FONT_GOOGLE_MOCKED_RESPONSES=(Resolve-Path ../../../../goldens/shadcn-web/scripts/next-font-google-mock.cjs).Path; $env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; $env:NEXT_PUBLIC_V0_URL='https://v0.dev'; pnpm exec next build --webpack`

`$env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020`

Alternatively, use the repo helper:

`python3 goldens/shadcn-web/scripts/serve-v4.py --port 4020`

5) Extract JSON goldens (Terminal B):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts button-default tabs-demo --update --baseUrl=http://localhost:4020`

Single-command alternative (starts/stops a production server in-process):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 button-default tabs-demo --update`

Extract *all* routable new-york-v4 pages (block+component+example):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts --all --update --baseUrl=http://localhost:4020`

Default output dir:

`goldens/shadcn-web/v4/new-york-v4/*.json`

### Proxy

If you need a proxy:

`$env:HTTP_PROXY='http://127.0.0.1:10809'; $env:HTTPS_PROXY='http://127.0.0.1:10809'; $env:ALL_PROXY='http://127.0.0.1:10809'`

## Use from Rust

The current Rust-side harness starts with “ingest + schema sanity + node selection”, as a minimal
end-to-end pipeline. Run:

`cargo nextest run -p fret-ui-shadcn --test web_goldens_smoke`

This test expects the JSON files to exist under `goldens/shadcn-web/...` and will fail fast with a
hint if they are missing.

### Button pipeline (web vs Fret)

For an end-to-end “consume web golden + extract Fret style” pipeline (no strict equality yet):

`cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`

To write a JSON report file under `ecosystem/fret-ui-shadcn/tests/web_reports/`:

`$env:WRITE_WEB_REPORT='1'; cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`

## Where assertions live

We keep web-conformance assertions in the component ecosystem crate (not in the runtime):

- `ecosystem/fret-ui-shadcn/tests/*` for shadcn-aligned “web vs Fret” comparisons.

Rationale:

- web goldens are an external conformance harness (paths, schema, normalization, tolerances),
  not component implementation details;
- centralizing the helpers avoids duplicating parsing/normalization across components.
