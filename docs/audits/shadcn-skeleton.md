# shadcn/ui v4 Audit — Skeleton

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Skeleton` against the upstream shadcn/ui v4 base docs,
base examples, and the current gallery/docs surface.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/skeleton.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/skeleton.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/skeleton-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/skeleton-avatar.tsx`, `repo-ref/ui/apps/v4/examples/base/skeleton-card.tsx`, `repo-ref/ui/apps/v4/examples/base/skeleton-text.tsx`, `repo-ref/ui/apps/v4/examples/base/skeleton-form.tsx`, `repo-ref/ui/apps/v4/examples/base/skeleton-table.tsx`, `repo-ref/ui/apps/v4/examples/base/skeleton-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/skeleton.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/skeleton.rs`

## Audit checklist

### Authoring surface

- Pass: `Skeleton::new()` covers the upstream leaf primitive path where callers set size and shape explicitly.
- Pass: `Skeleton::block()` remains a focused Fret convenience (`w-full h-4`) for common loading rows without changing the upstream default path.
- Pass: `Skeleton` is a visual leaf primitive, so Fret intentionally does not add a generic `compose()` builder here.

### Visual defaults and ownership

- Pass: Default chrome uses `accent` background with `rounded-md` corners.
- Pass: Pulse animation is enabled by default, matching the upstream `animate-pulse` outcome.
- Pass: Explicit width, height, aspect ratio, and fully rounded avatar shapes remain caller-owned rather than recipe defaults.

### Gallery / docs parity

- Pass: The gallery now mirrors the upstream base Skeleton docs path first: `Demo`, `Usage`, `Avatar`, `Card`, `Text`, `Form`, `Table`, `RTL`, and `API Reference`.
- Pass: `API Reference` remains a compact Fret follow-up summarizing ownership because upstream treats Skeleton as a very small leaf primitive.
- Pass: This work is docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --lib skeleton`
