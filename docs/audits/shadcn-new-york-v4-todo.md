# shadcn/ui new-york-v4 TODO (Fret)

This is a short, high-signal backlog to keep the “goldens-driven 1:1” effort grounded.
Prefer adding deterministic gates over adding more smoke coverage.

## P0 (Overlays)

- Menus: destructive *idle* vs *focused* state matrix (ContextMenu done; replicate where applicable).
- Menubar destructive variant: **not exercised** by `menubar-demo` in upstream `new-york-v4` registry.
  Options:
  - Add a dedicated upstream example (requires upstream changes; avoid), or
  - Introduce a “golden-only” harness page in our extractor (keeps repo-ref pristine), or
  - Add a separate style harness that includes `base-nova` examples (new goldens + theme alignment).

## P1 (Typography)

- Gate typography outcomes under multiple widths (line wrapping, margins, list markers).

## P1 (Calendar)

- Gate month grid geometry + selection/hover states across viewports.

## P2 (Contracts / Variants)

- Prototype a “variants contract JSON” generator for one recipe-heavy component (e.g. Button).
  Evaluate maintenance cost before scaling to more components.

