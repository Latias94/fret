# shadcn-web goldens

This folder stores JSON goldens extracted from the upstream shadcn/ui React app.

Upstream: https://github.com/shadcn-ui/ui

## Why JSON (not screenshots)

We want a stable “contract” that can be consumed by non-web runtimes (Fret). JSON lets us compare:

- layout geometry (DOM rects),
- computed styles (a curated subset),
- state attrs (`aria-*`, `data-state`, ...),

without depending on pixel-perfect rendering (fonts/AA/DPI differences).

## Quick sanity checks

For `button-default`, a *healthy* web golden typically has:

- `computedStyle.display: "inline-flex"`
- `computedStyle.paddingLeft: "16px"` (Tailwind `px-4`)
- `computedStyle.paddingTop: "8px"` (Tailwind `py-2`)
- `computedStyle.borderTopWidth: "0px"`
- `computedStyle.borderTopLeftRadius != "0px"` (Tailwind `rounded-md`)

If you see browser defaults like `display: inline-block` + `borderTopWidth: 2px`, regenerate using a
production server (`pnpm -C <shadcn-ui-root>/apps/v4 build` + `pnpm -C <shadcn-ui-root>/apps/v4 exec next start -p <port>`) instead of a dev server.

Current new-york-v4 snapshot count: `436` JSON files.
