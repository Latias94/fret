# shadcn/ui v4 Audit — Skeleton

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Skeleton` against the upstream shadcn/ui v4 base docs,
base examples, the current gallery/docs surface, and the absence of any dedicated headless
`Skeleton` primitive in the Base UI / Radix primitives reference axes.

## Upstream references (source of truth)

- Docs pages: `repo-ref/ui/apps/v4/content/docs/components/base/skeleton.mdx`,
  `repo-ref/ui/apps/v4/content/docs/components/radix/skeleton.mdx`
- Component implementations: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/skeleton.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/base/ui/skeleton.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/radix/ui/skeleton.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/skeleton-demo.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/skeleton-card.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/base/examples/skeleton-example.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/radix/examples/skeleton-example.tsx`
- Headless references: no dedicated `Skeleton` primitive exists under `repo-ref/primitives` or
  `repo-ref/base-ui`; those axes therefore confirm that this family is recipe/docs work, not a
  missing mechanism contract.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/skeleton.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/skeleton.rs`

## Audit checklist

### Authoring surface

- Pass: `Skeleton::new()` covers the upstream leaf primitive path where callers set size and shape explicitly.
- Pass: `Skeleton::block()` remains a focused Fret convenience (`w-full h-4`) for common loading rows without changing the upstream default path.
- Pass: `Skeleton` is a visual leaf primitive, so Fret intentionally does not add a generic `compose()` builder here.
- Pass: No composable children API is needed here; the upstream shadcn/base/radix surfaces all expose `Skeleton` as a leaf `div`/placeholder boundary rather than a compound parts family.

### Visual defaults and ownership

- Pass: Default chrome uses `accent` background with `rounded-md` corners.
- Pass: Pulse animation is enabled by default, matching the upstream `animate-pulse` outcome.
- Pass: Explicit width, height, aspect ratio, and fully rounded avatar shapes remain caller-owned rather than recipe defaults.

### Mechanism boundary

- Pass: `repo-ref/primitives` and `repo-ref/base-ui` do not define a dedicated `Skeleton`
  primitive, so there is no missing mechanism/headless contract to port into `fret-ui` or
  `fret-ui-kit`.
- Pass: Existing `web_vs_fret_layout::skeleton_*` and reduced-motion tests already cover the
  runtime/layout side; the remaining work here is public-surface/docs alignment.

### Gallery / docs parity

- Pass: The gallery now mirrors the upstream base Skeleton docs path after collapsing the top
  preview into `Demo` and skipping `Installation`: `Demo`, `Usage`, `Examples` (`Avatar`, `Card`,
  `Text`, `Form`, `Table`), `RTL`, then Fret-only `API Reference` and `Notes`.
- Pass: `API Reference` remains a compact Fret follow-up summarizing ownership because upstream
  treats Skeleton as a very small leaf primitive.
- Pass: `Notes` now record the source axes and explicitly document why no extra composable children
  API or mechanism-layer work is needed.

## Validation

- `CARGO_TARGET_DIR=target-codex-skeleton cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-skeleton cargo test -p fret-ui-shadcn --lib skeleton`
- `CARGO_TARGET_DIR=target-codex-skeleton cargo test -p fret-ui-shadcn --test web_vs_fret_layout skeleton`
- `CARGO_TARGET_DIR=target-codex-skeleton cargo test -p fret-ui-gallery --test skeleton_docs_surface`
- `target/debug/fretboard-dev diag run tools/diag-scripts/ui-gallery/skeleton/ui-gallery-skeleton-docs-smoke.json --warmup-frames 5 --exit-after-run --launch -- env CARGO_TARGET_DIR=target-codex-skeleton cargo run -p fret-ui-gallery --release`
