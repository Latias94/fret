# radix-web goldens

This folder stores JSON goldens extracted from the upstream shadcn/ui v4 app (`repo-ref/ui`) for
**Radix primitive behavior** (interaction + accessibility semantics).

Unlike `shadcn-web` (layout/style snapshots), these goldens are **timelines**:

- drive pointer/keyboard actions (open/close, roving focus, selection),
- record DOM semantics (`role`, `aria-*`, `data-state`, ...),
- record the Accessibility Tree snapshot after each step.
- optionally record viewport-relative DOM rects (`getBoundingClientRect`) for included nodes (for
  layout-driven contracts like overlay placement).

Generator script:

- `goldens/radix-web/scripts/extract-behavior.mts`

Run it via the shadcn app toolchain (so it can reuse `puppeteer` from `repo-ref/ui`):

`pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/radix-web/scripts/extract-behavior.mts --all --update --baseUrl=http://localhost:4020`

By default the extractor runs in `light` mode (and writes `*.light.json`). If you ever need `dark`,
pass `--theme=dark`.
