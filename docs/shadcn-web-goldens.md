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
- `portals[]` and `portalWrappers[]` snapshots for Radix portal content (wrapper geometry is used for placement checks).

## Prerequisites

- `pnpm`
- `repo-ref/ui` dependencies installed
- a shadcn v4 **production** server (`next start`) running locally (recommended)

## Run

1) Install deps:

`pnpm -C repo-ref/ui install`

2) Build the app (Terminal A).

Important: `pnpm -C repo-ref/ui/apps/v4 build` currently defaults to Turbopack and fails to resolve some transitive Radix deps under pnpm on Windows.
Force webpack instead, and provide the required `NEXT_PUBLIC_*` env vars at build time:

`pnpm -C repo-ref/ui --filter shadcn build`

`cd repo-ref/ui/apps/v4; $env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; $env:NEXT_PUBLIC_V0_URL='https://v0.dev'; pnpm exec next build --webpack`

3) Start a production server (Terminal A):

`$env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020`

4) Extract goldens (Terminal B):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts button-default tabs-demo --baseUrl=http://localhost:4020`

Extract both closed + open overlay states (writes `*.open.json` alongside the base file):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts popover-demo dropdown-menu-demo select-scrollable --modes=open --update --baseUrl=http://localhost:4020`

Extract open overlay states that require non-click input (the script infers the right open action per page):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts context-menu-demo tooltip-demo hover-card-demo command-dialog --modes=open --update --baseUrl=http://localhost:4020`

If a page opens via a keyboard chord, you can override the keys used for `openAction=keys`:

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts command-dialog --modes=open --update --baseUrl=http://localhost:4020 --openKeys=Control+KeyJ`

Extract nested open sequences (example: submenu open) by combining `--openVariants` and `--openSteps`.

For keyboard-driven submenus, prefer `keys=<selector>@<keys>` (no global `--openKeys` required):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts dropdown-menu-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants="submenu-kbd=[data-fret-golden-target] [data-slot='dropdown-menu-trigger']" --openSteps="keys=[data-slot='dropdown-menu-sub-trigger']@ArrowRight"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts context-menu-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants="submenu-kbd=[data-fret-golden-target] [data-slot='context-menu-trigger']" --openSteps="keys=[data-slot='context-menu-sub-trigger']@ArrowRight"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts menubar-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants="submenu-kbd=[data-fret-golden-target] [aria-haspopup='menu'][data-state='closed']" --openSteps="keys=[data-slot='menubar-sub-trigger']@ArrowRight"`

Extract multiple open variants for a single page (writes `*.{variant}.open.json` alongside the base files):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts sheet-side --modes=open --update --baseUrl=http://localhost:4020 --openVariants="right=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(2);bottom=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(3);left=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(4)"`

Extract *all* routable new-york-v4 pages (defaults match `/view/[style]/[name]`: block+component+example):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts --all --update --baseUrl=http://localhost:4020`

On the current setup, `--all` (default `--modes=closed`) generates `362` JSON files under
`goldens/shadcn-web/v4/new-york-v4/`.
This matches the current `repo-ref/ui` v4 registry index: `registry:block` (134) + `registry:example` (228).

If you also extract open overlay states (`--modes=open` or `--open`), you will get additional
`*.open.json` files alongside the base closed-mode goldens. In this repo, the current snapshot is:

- `362` closed-mode files (`*.json`, excluding `*.open.json`)
- `23` open-mode files (`*.open.json`)

Note: `*.open.json` also matches the glob `*.json`, so "total .json files" will include open-mode
snapshots unless you exclude `*.open.json`.

If the extracted `computedStyle` looks like browser defaults (e.g. `<button>` has `display:
inline-block`, `borderTopWidth: 2px`), your dev server is likely not producing Tailwind utilities.
In that case, prefer a production build:

`pnpm -C repo-ref/ui --filter shadcn build; cd repo-ref/ui/apps/v4; $env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; $env:NEXT_PUBLIC_V0_URL='https://v0.dev'; pnpm exec next build --webpack; pnpm exec next start -p 4020`

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
- `--repoRefUiDir=<path>` (optional; default: `<repo>/repo-ref/ui`)
- `--viewportW=1440` (default)
- `--viewportH=900` (default)
- `--deviceScaleFactor=2` (default; alias: `--dpr=2`)
- `--openSelector=<css>` (optional override for the "open overlay" trigger)
- `--openVariants="<variant>=<css>;..."` (optional; writes `name.<variant>.open.json` for each entry; overrides `--openSelector`)
- `--openAction=click|hover|contextmenu|keys` (optional override for the "open overlay" action; default is inferred per page)
- `--openKeys=<chord>` (optional; only used when `openAction=keys`; e.g. `Control+KeyJ` or `Meta+KeyJ`; env: `OPEN_KEYS`)
- `--openSteps="<action>=<value>;..."` (optional; extra steps after the initial open; actions: `click|hover|contextmenu|keys|wait`)
  - `keys=<selector>` uses `--openKeys` / `OPEN_KEYS`.
  - `keys=<selector>@<keys>` uses an inline key spec. `<keys>` supports a chord (`Shift+F10`) or a sequence (`ArrowDown ArrowRight` or `ArrowDown,ArrowRight`).
- `--baseUrl=http://localhost:4000`
- `--all` (env: `ALL_GOLDENS=1`)
- `--types=registry:block,registry:component,registry:example` (env: `TYPES=...`)
- `--outDir=<path>`
- `--update` (overwrite existing files; env: `UPDATE_GOLDENS=1`)
- `--timeoutMs=60000` (env: `TIMEOUT_MS=60000`)
