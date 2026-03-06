# shadcn/ui v4 Audit - Skeleton


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Skeleton` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/skeleton.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/skeleton.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/skeleton.rs`

## Audit checklist

### Authoring surface

- Pass: `Skeleton::new()` covers the common shadcn authoring path for custom-sized placeholders.
- Pass: `Skeleton::block()` provides a Fret convenience baseline (`w-full h-4`) for common loading rows.
- Note: `Skeleton` is a visual leaf primitive, so Fret intentionally does not add a generic `compose()` builder here.

### Visual defaults (shadcn parity)

- Pass: Default chrome uses `accent` background with `rounded-md` corners.
- Pass: Pulse animation is enabled by default, matching the upstream `animate-pulse` outcome.
- Note: Upstream shadcn leaves size entirely to author CSS classes; Fret mirrors that with `Skeleton::new()` and documents `refine_layout(...)` / `Skeleton::block()` for explicit size control.

## Validation

- `cargo test -p fret-ui-shadcn --lib skeleton`