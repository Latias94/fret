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
- optional scroll metrics for scrollable viewports (`scrollWidth/clientWidth`, `scrollHeight/clientHeight`, `scrollLeft/scrollTop`, ...).
- `portals[]` and `portalWrappers[]` snapshots for Radix portal content (wrapper geometry is used for placement checks).

## Stability notes

- Recharts-backed `chart-*` pages render key SVG nodes asynchronously (ResizeObserver + RAF + JS-driven animation).
  The extractor waits for series nodes to reach stable geometry (and may fall back to SVG bbox transforms for
  Recharts layers) to avoid capturing partial frames (e.g. `chart-bar-default` missing `path.recharts-rectangle`,
  or radar/radial charts captured at their animation origin).

## Prerequisites

- `pnpm`
- `repo-ref/ui` dependencies installed
- shadcn v4 **production assets** built (`next build --webpack`)
- either:
  - a local shadcn v4 **production** server (`next start`) running, or
  - use `--startServer` to start/stop a production server in-process

## Run

Note:

- Positional arguments are **route names** (e.g. `chart-line-interactive`), not output keys.
- Do not pass suffixes like `.hover-mid` / `.open` as part of the name. Use `--variants=...` and/or
  `--modes=...` instead.

1) Install deps:

`pnpm -C repo-ref/ui install`

2) Build the app (Terminal A).

Important: `pnpm -C repo-ref/ui/apps/v4 build` currently defaults to Turbopack and fails to resolve some transitive Radix deps under pnpm on Windows.
Force webpack instead, and provide the required `NEXT_PUBLIC_*` env vars at build time:

`pnpm -C repo-ref/ui --filter shadcn build`

If your environment cannot reach Google Fonts during `next build`, you can still build shadcn v4
offline by using Next's `NEXT_FONT_GOOGLE_MOCKED_RESPONSES` hook (maps requested families to local
Windows font files):

`cd repo-ref/ui/apps/v4; $env:NEXT_FONT_GOOGLE_MOCKED_RESPONSES=(Resolve-Path ../../../../goldens/shadcn-web/scripts/next-font-google-mock.cjs).Path; $env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; $env:NEXT_PUBLIC_V0_URL='https://v0.dev'; pnpm exec next build --webpack`

3) Start a production server (Terminal A):

`$env:NEXT_PUBLIC_APP_URL='http://localhost:4020'; pnpm -C repo-ref/ui/apps/v4 exec next start -p 4020`

4) Extract goldens (Terminal B):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts button-default tabs-demo --baseUrl=http://localhost:4020`

Alternative (in-process server, single command):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 button-default tabs-demo`

Note: `--startServer` runs the Next app with `process.cwd()` set to `--nextDir` for the lifetime of
the in-process server. This avoids mis-resolving app-local config files (e.g. `apps/v4/source.config.ts`)
when invoking the extractor from the outer repo root.

Extract both closed + open overlay states (writes `*.open.json` alongside the base file):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts popover-demo dropdown-menu-demo dropdown-menu-dialog item-dropdown select-scrollable --modes=open --update --baseUrl=http://localhost:4020`

Freeze time (optional):

Some pages depend on `new Date()` (notably DatePicker presets). To keep goldens deterministic across days,
pass `--freezeDate=YYYY-MM-DD`.

Example (pick “Tomorrow” with a fixed baseline date):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 --freezeDate=2026-01-15 --style=new-york-v4 --modes=open --variants=preset-tomorrow --openAction=click --openSelector=\"[data-fret-golden-target] button[aria-controls]\" --openSteps=\"click=[data-slot=select-trigger];waitFor=[data-slot=select-content]\" --steps=\"click=[data-radix-select-viewport] [data-slot=select-item]:nth-of-type(2);wait=50\" date-picker-with-presets --update`

Hover-only scripted steps:

Some variants (e.g. “highlight-first”) only require a hover/focus change and do not open a new portal surface.
Use `hoverNoWait=...` to avoid deadlocking on the default “wait for new portal” heuristic:

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 --style=new-york-v4 --modes=open --variants=highlight-first-vp375x240 --viewportW=375 --viewportH=240 --openSteps=\"hoverNoWait=[data-slot='dropdown-menu-item']\" dropdown-menu-demo --update`

Extract open overlay states that require non-click input (the script infers the right open action per page):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts context-menu-demo tooltip-demo hover-card-demo command-dialog --modes=open --update --baseUrl=http://localhost:4020`

DropdownMenu open-state examples (checkboxes / radio group):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts dropdown-menu-checkboxes dropdown-menu-radio-group --modes=open --update --baseUrl=http://localhost:4020`

DropdownMenu open-state examples (button-group-demo / mode-toggle):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts button-group-demo --modes=open --update --baseUrl=http://localhost:4020 --openAction=click --openSelector=\"[data-fret-golden-target] button[aria-label='More Options']\"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts button-group-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants=\"submenu-kbd=[data-fret-golden-target] button[aria-label='More Options']\" --openSteps=\"keys=[data-slot='dropdown-menu-sub-trigger']@ArrowRight\"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts mode-toggle --modes=open --update --baseUrl=http://localhost:4020 --openAction=click --openSelector=\"[data-fret-golden-target] button\"`

Combobox open-state examples:

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 combobox-demo --modes=open --update --openSelector=\"[data-fret-golden-target] button[role='combobox']\"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 combobox-popover --modes=open --update --openSelector=\"[data-fret-golden-target] [data-state='closed'][aria-haspopup]\"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 combobox-dropdown-menu --modes=open --update --openSelector=\"[data-fret-golden-target] [aria-haspopup='menu'][data-state='closed']\"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 combobox-responsive --modes=open --update --openSelector=\"[data-fret-golden-target] [data-state='closed'][aria-haspopup]\"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 combobox-demo --modes=open --update --viewportW=375 --viewportH=320 --openVariants=\"vp375x320=[data-fret-golden-target] button[role='combobox']\"`

Select open-state example (`select-demo`):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 select-demo --modes=open --update --openSelector=\"[data-fret-golden-target] button[role='combobox']\"`

Breadcrumb responsive variants (`breadcrumb-responsive`):

Desktop open (dropdown menu trigger uses `aria-label="Toggle menu"`):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 breadcrumb-responsive --modes=open --update --openSelector=\"[data-fret-golden-target] button[aria-label='Toggle menu']\"`

Mobile closed + open (drawer trigger uses `aria-label="Toggle Menu"`):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 breadcrumb-responsive --update --viewportW=375 --viewportH=812 --variants=\"vp375x812\"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 breadcrumb-responsive --modes=open --update --viewportW=375 --viewportH=812 --openVariants=\"vp375x812=[data-fret-golden-target] button[aria-label='Toggle Menu']\"`

DataTable empty-state variant (drives a React-controlled filter input):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 data-table-demo --variants=\"empty\" --steps=\"type=[data-fret-golden-target] input[placeholder='Filter emails...']@zzzzzz\" --update`

If a page opens via a keyboard chord, you can override the keys used for `openAction=keys`:

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts command-dialog --modes=open --update --baseUrl=http://localhost:4020 --openKeys=Control+KeyJ`

Extract nested open sequences (example: submenu open) by combining `--openVariants` and `--openSteps`.

For keyboard-driven submenus, prefer `keys=<selector>@<keys>` (no global `--openKeys` required):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts dropdown-menu-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants="submenu-kbd=[data-fret-golden-target] [data-slot='dropdown-menu-trigger']" --openSteps="keys=[data-slot='dropdown-menu-sub-trigger']@ArrowRight"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts context-menu-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants="submenu-kbd=[data-fret-golden-target] [data-slot='context-menu-trigger']" --openSteps="keys=[data-slot='context-menu-sub-trigger']@ArrowRight"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts menubar-demo --modes=open --update --baseUrl=http://localhost:4020 --openVariants="submenu-kbd=[data-fret-golden-target] [aria-haspopup='menu'][data-state='closed']" --openSteps="keys=[data-slot='menubar-sub-trigger']@ArrowRight"`

Extract Menubar top-level open variants (useful for checkbox/radio item gates):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts menubar-demo --modes=open --update --baseUrl=http://localhost:4020 --openAction=click --openVariants="view=[data-fret-golden-target] button[data-slot='menubar-trigger']:nth-of-type(3);profiles=[data-fret-golden-target] button[data-slot='menubar-trigger']:nth-of-type(4)"`

Extract a constrained-viewport open variant (useful for max-height/clamp/scroll behavior):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts dropdown-menu-demo --modes=open --update --baseUrl=http://localhost:4020 --viewportH=320 --openVariants="vp1440x320=[data-fret-golden-target] [aria-haspopup='menu'][data-state='closed']"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 dropdown-menu-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [data-state='closed'][aria-haspopup]"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 context-menu-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [data-slot='context-menu-trigger']"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 menubar-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [data-slot='menubar-trigger']"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 select-scrollable --modes=open --update --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [role='combobox'][aria-expanded='false']"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 combobox-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [role='combobox'][aria-expanded='false']"`

Extract ScrollArea hover/scrolled variants (useful for scrollbar + thumb geometry):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts scroll-area-demo --modes=closed --update --baseUrl=http://localhost:4020 --variants=hover --steps="wait=200;hover=[data-slot=scroll-area];waitFor=[data-slot=scroll-area-scrollbar]"`

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts scroll-area-demo --modes=closed --update --baseUrl=http://localhost:4020 --variants=scrolled --steps="wait=200;hover=[data-slot=scroll-area];waitFor=[data-slot=scroll-area-scrollbar];scroll=[data-radix-scroll-area-viewport]@0,80;wait=50"`

Extract ScrollArea hover-out variants (useful for `scrollHideDelay` timing):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts scroll-area-demo scroll-area-horizontal-demo --modes=closed --update --baseUrl=http://localhost:4020 --variants=hover-out-650ms --steps="wait=200;hover=body;wait=50;hover=[data-slot=scroll-area];waitFor=[data-slot=scroll-area-scrollbar];move=1,1;wait=650"`

Extract a constrained-viewport submenu variant (useful for "submenu flips/clamps and scrolls" behavior):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts dropdown-menu-demo --modes=open --update --baseUrl=http://localhost:4020 --viewportH=320 --openVariants="submenu-kbd-vp1440x320=[data-fret-golden-target] [data-slot='dropdown-menu-trigger']" --openSteps="keys=[data-slot='dropdown-menu-sub-trigger']@ArrowRight"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 dropdown-menu-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="submenu-kbd-vp1440x240=[data-fret-golden-target] [data-slot='dropdown-menu-trigger']" --openSteps="keys=[data-slot='dropdown-menu-sub-trigger']@ArrowRight"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 context-menu-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="submenu-kbd-vp1440x240=[data-fret-golden-target] [data-slot='context-menu-trigger']" --openSteps="keys=[data-slot='context-menu-sub-trigger']@ArrowRight"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 menubar-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="submenu-kbd-vp1440x240=[data-fret-golden-target] [aria-haspopup='menu'][data-state='closed']" --openSteps="keys=[data-slot='menubar-sub-trigger']@ArrowRight"`

Extract multiple open variants for a single page (writes `*.{variant}.open.json` alongside the base files):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts sheet-side --modes=open --update --baseUrl=http://localhost:4020 --openVariants="right=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(2);bottom=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(3);left=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(4)"`

Extract *all* routable new-york-v4 pages (defaults match `/view/[style]/[name]`: block+component+example):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts --all --update --baseUrl=http://localhost:4020`

On the current setup, `--all` (default `--modes=closed`) generates `370` JSON files under
`goldens/shadcn-web/v4/new-york-v4/`.
This number can drift as `repo-ref/ui` adds/removes routable pages.

If you also extract open overlay states (`--modes=open` or `--open`), you will get additional
`*.open.json` files alongside the base closed-mode goldens. The exact counts drift as upstream
adds/removes routable pages.

To compute the current snapshot counts (PowerShell):

`$open=(Get-ChildItem -Path goldens/shadcn-web/v4/new-york-v4 -Filter *.open.json -File).Count; $closed=(Get-ChildItem -Path goldens/shadcn-web/v4/new-york-v4 -Filter *.json -File | Where-Object { $_.Name -notmatch '\\.open\\.json$' }).Count; \"$closed closed, $open open\"`

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

If you use multiple git worktrees and the repo config points Cargo at a shared `target-dir`, you may
hit stale cross-worktree artifacts. In that case, run `cargo clean -p fret-ui-kit -p fret-ui-shadcn`
before re-running `cargo nextest`.

## NavigationMenu open variants

We keep multiple open variants for `navigation-menu-demo` because each trigger exercises different
panel content and can surface layout/placement regressions.

Desktop (viewport disabled):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 navigation-menu-demo --modes=open --update --openAction=click --openVariants="components=[data-fret-golden-target] li:nth-of-type(2) [data-slot='navigation-menu-trigger'];list=[data-fret-golden-target] li:nth-of-type(4) [data-slot='navigation-menu-trigger'];simple=[data-fret-golden-target] li:nth-of-type(5) [data-slot='navigation-menu-trigger'];with-icon=[data-fret-golden-target] li:nth-of-type(6) [data-slot='navigation-menu-trigger']"`

Mobile (viewport enabled, 375x812):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 navigation-menu-demo --modes=open --update --viewportW=375 --viewportH=812 --openAction=click --openVariants="home-mobile=[data-fret-golden-target] li:nth-of-type(1) [data-slot='navigation-menu-trigger'];components-mobile=[data-fret-golden-target] li:nth-of-type(2) [data-slot='navigation-menu-trigger']"`

Constrained viewport (useful for clamping/placement regressions):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 navigation-menu-demo --modes=open --update --viewportW=1440 --viewportH=320 --openAction=click --openVariants="components-vp1440x320=[data-fret-golden-target] li:nth-of-type(2) [data-slot='navigation-menu-trigger'];list-vp1440x320=[data-fret-golden-target] li:nth-of-type(4) [data-slot='navigation-menu-trigger'];simple-vp1440x320=[data-fret-golden-target] li:nth-of-type(5) [data-slot='navigation-menu-trigger'];with-icon-vp1440x320=[data-fret-golden-target] li:nth-of-type(6) [data-slot='navigation-menu-trigger']"`

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 navigation-menu-demo --modes=open --update --viewportW=375 --viewportH=320 --openAction=click --openVariants="home-mobile-vp375x320=[data-fret-golden-target] li:nth-of-type(1) [data-slot='navigation-menu-trigger'];components-mobile-vp375x320=[data-fret-golden-target] li:nth-of-type(2) [data-slot='navigation-menu-trigger']"`

Constrained mobile hover-switch (click Home, then hover Components):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 navigation-menu-demo --modes=open --update --viewportW=375 --viewportH=320 --openAction=click --openVariants="home-mobile-vp375x320-then-hover-components=[data-fret-golden-target] li:nth-of-type(1) [data-slot='navigation-menu-trigger']" --steps="hover=[data-fret-golden-target] li:nth-of-type(2) [data-slot='navigation-menu-trigger'];wait=300"`

Indicator (opt-in child, `shadow-md` diamond):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 navigation-menu-demo-indicator --modes=open --update`

Other constrained overlay variants (useful for tight-height flip/shift behavior):

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 popover-demo hover-card-demo tooltip-demo alert-dialog-demo dialog-demo sheet-demo --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [data-slot$='trigger']"`

Note: `command-dialog` uses a different trigger selector:

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 command-dialog --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="vp1440x240=[data-fret-golden-target] [data-state='closed'][aria-haspopup]"`

Extract constrained-viewport `sheet-side` variants:

`node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 sheet-side --modes=open --update --viewportW=1440 --viewportH=240 --openVariants="top-vp1440x240=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(1);right-vp1440x240=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(2);bottom-vp1440x240=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(3);left-vp1440x240=[data-fret-golden-target] button[data-slot='sheet-trigger']:nth-of-type(4)"`


Output directory (default):

`goldens/shadcn-web/v4/new-york-v4/*.json`

## Consume from Rust

Minimal web-golden ingest smoke:

`cargo nextest run -p fret-ui-shadcn --test web_goldens_smoke`

Button “web vs Fret” pipeline (writes no files by default):

`cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`

This test extracts the first `<button>` from the web golden and compares a small set of “style
contract” fields against the Fret paint scene:

- `backgroundColor`, `color`, `borderTopWidth`, `borderTopColor`, `borderTopLeftRadius`
- It parses `rgb/rgba` and `lab()` values. Note that CSS `lab()` uses a D50 whitepoint, so the
  comparison converts Lab(D50) -> linear sRGB via Bradford D50->D65 adaptation before asserting.
- Border color is only asserted when the border width is > 0px (web goldens keep a `borderTopColor`
  even when `borderTopWidth` is `0px`).

To emit a JSON comparison report:

`$env:WRITE_WEB_REPORT='1'; cargo nextest run -p fret-ui-shadcn --test web_vs_fret_button`

Control chrome conformance (borders/radii + a few key control sizes):

`cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`

Overlay placement + menu sizing conformance:

`cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_placement`

This suite asserts portal wrapper placement (side/align/gap) and also includes explicit checks for
menu sizing signals such as per-item heights, content insets, and Select metrics (listbox
width/height, option row height, scroll buttons, and viewport insets), including constrained-viewport
variants that force max-height/scroll behavior.

NavigationMenu coverage includes both placement checks and content sizing gates (content
width/height) so variant panels (e.g. `simple`, `with-icon`) can't silently drift from the upstream
shadcn layout contract.

It also includes geometry gates for cmdk-derived listboxes such as `combobox-demo` (option row
height and option insets within the listbox), and menu item metrics for mixed cmdk+menu recipes
such as `combobox-dropdown-menu` (menu item height + dropdown content insets).

Overlay chrome conformance (border, radii, and selected colors):

`cargo nextest run -p fret-ui-shadcn --test web_vs_fret_overlay_chrome`

This suite compares web computed styles against the Fret paint scene for overlay surfaces. It
currently gates border widths + corner radii broadly, and additionally asserts selected surface
colors (background + border) derived from web `computedStyle` for a growing set of overlay
surfaces (`dialog-content`, `sheet-content`, `popover-content`, `dropdown-menu-content`,
`dropdown-menu-sub-content`, `context-menu-content`, `context-menu-sub-content`, `menubar-content`,
`menubar-sub-content`, `navigation-menu-content`, `select-content`,
`hover-card-content`, `tooltip-content`, `drawer-content`) across light/dark themes when
available.

For nested menus, the chrome suite also covers the constrained-viewport keyboard variants (for
example: `*.submenu-kbd-vp1440x240.open.json`) so tight-height placement/clamping can't regress
surface styling.

## Layout conformance (geometry-first)

For layout-engine refactors, prefer geometry-first assertions (rects + spacing invariants) over
pixel diffs. See: `docs/audits/shadcn-web-layout-conformance.md`.

This suite also includes a few paint-backed checks where upstream styling is effectively part of the
geometry contract (for example: ScrollArea thumb background/alpha in hover-visible states).

Current layout gates include:

- `accordion-demo`: trigger + open-content wrapper geometry and item heights (light+dark).
- `table-demo`: header/body/footer row heights + caption gap (Table recipe conformance).
- `data-table-demo`: row height + checkbox/action button sizing (Table + Checkbox + Button icon sizing).
- `data-table-demo.empty`: empty-state `td` spans full table width (colSpan) + `h-24` height.
- `typography-table`: 2-column typography table geometry (row heights + cell rects) + `even:bg-muted` background (paint-backed) using the prose typography baseline (light+dark).
- `progress-demo`: track + indicator geometry and background colors, including percent-translate parity for the indicator (light+dark).
- `navigation-menu-demo`: trigger geometry + content sizing (width/height) across variants.
- `scroll-area-demo` / `scroll-area-horizontal-demo`: thumb bounds + (paint-backed) thumb background/alpha in hover-visible states.

## Options

- `--style=new-york-v4`
- `--themes=light,dark` (default)
- `--modes=closed,open` (default: `closed`)
- `--open` (shorthand for `--modes=closed,open`)
- `--viewportW=1440` (default)
- `--viewportH=900` (default)
- `--deviceScaleFactor=2` (default)
- `--variants="<variant>;..."` (optional; writes `name.<variant>.json` for each entry, regardless of mode)
- `--openSelector=<css>` (optional override for the "open overlay" trigger)
- `--openVariants="<variant>=<css>;..."` (optional; writes `name.<variant>.open.json` for each entry; overrides `--openSelector`)
- `--openAction=click|hover|contextmenu|keys` (optional override for the "open overlay" action; default is inferred per page)
- `--openKeys=<chord>` (optional; only used when `openAction=keys`; e.g. `Control+KeyJ` or `Meta+KeyJ`; env: `OPEN_KEYS`)
- `--steps="<action>=<value>;..."` (optional; scripted interactions before extraction; actions: `click|hover|contextmenu|keys|type|wait|waitFor|move|scroll|scrollTo`)
- `--openSteps="<action>=<value>;..."` (optional; extra steps after the initial open; actions: `click|hover|contextmenu|keys|type|wait|waitFor|move|scroll|scrollTo`)
  - `keys=<selector>` uses `--openKeys` / `OPEN_KEYS`.
  - `keys=<selector>@<keys>` uses an inline key spec. `<keys>` supports a chord (`Shift+F10`) or a sequence (`ArrowDown ArrowRight` or `ArrowDown,ArrowRight`).
  - `type=<selector>@<text>` sets a controlled `<input>/<textarea>` value and dispatches `input`/`change` (React-friendly).
  - `waitFor=<selector>` waits for a selector to appear (useful for hover-gated ScrollArea scrollbars).
  - `move=<x>,<y>` moves the mouse to an absolute viewport position (useful to force pointerenter/leave).
  - `scroll=<selector>@<dx>,<dy>` scrolls an element via `scrollBy(dx, dy)`.
  - `scrollTo=<selector>@<left>,<top>` sets `scrollLeft/scrollTop` via `scrollTo(left, top)` (useful when a component re-syncs scroll on open).
- `--baseUrl=http://localhost:4000`
- `--startServer` (env: `START_SERVER=1`)
- `--nextDir=<path>` (env: `NEXT_DIR=...`; default: `repo-ref/ui/apps/v4`)
- `--all` (env: `ALL_GOLDENS=1`)
- `--types=registry:block,registry:component,registry:example` (env: `TYPES=...`)
- `--outDir=<path>`
- `--update` (overwrite existing files; env: `UPDATE_GOLDENS=1`)
- `--timeoutMs=60000` (env: `TIMEOUT_MS=60000`)

## Coverage quick check

To see which shadcn web golden keys are referenced by Rust tests (rough heuristic, but useful for
planning):

`powershell -ExecutionPolicy Bypass -File tools/golden_coverage.ps1 -Kind shadcn-web -Style v4/new-york-v4 -ShowMissing`
