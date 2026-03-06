# shadcn/ui v4 Audit - Card


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Card` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/card.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/card.rs`

## Audit checklist

### Authoring surface

- Pass: `Card::new([...])` plus `CardHeader` / `CardContent` / `CardFooter` / `CardTitle` / `CardDescription` / `CardAction` covers the common shadcn authoring path.
- Pass: Builder-first helpers such as `CardBuild` and `CardHeader::build(...)` cover more declarative composition without changing the core recipe surface.
- Pass: `CardSize::Sm` and per-slot layout/style refinements provide the expected recipe-level sizing hooks.
- Note: `Card` already exposes both slot components and builder helpers, so Fret intentionally does not add a separate generic `compose()` builder here.

### Layout & geometry (shadcn parity)

- Pass: Root chrome follows the upstream defaults: `rounded-xl`, `border`, `shadow-sm`, and vertical spacing between slots.
- Pass: `CardHeader` keeps title/description/action alignment compatible with the upstream two-row grid outcome.
- Pass: `CardContent` and `CardFooter` preserve the expected horizontal padding and allow richer compositions without collapsing intrinsic child sizes.

## Validation

- `cargo test -p fret-ui-shadcn --lib card`
- `cargo check -p fret-ui-gallery`