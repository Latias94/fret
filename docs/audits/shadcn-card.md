# shadcn/ui v4 Audit - Card


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Card` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page (radix): `repo-ref/ui/apps/v4/content/docs/components/radix/card.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/card.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/radix/card-demo.tsx`, `repo-ref/ui/apps/v4/examples/radix/card-small.tsx`, `repo-ref/ui/apps/v4/examples/radix/card-image.tsx`, `repo-ref/ui/apps/v4/examples/radix/card-rtl.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/card.rs`

## Audit checklist

### Authoring surface

- Pass: `Card::new([...])` plus `CardHeader` / `CardContent` / `CardFooter` / `CardTitle` / `CardDescription` / `CardAction` covers the common shadcn authoring path.
- Pass: Free helpers (`card`, `card_header`, `card_title`, `card_description`, `card_action`, `card_content`, `card_footer`) now cover the copyable “children-style” path without forcing every example to allocate slot structs manually.
- Pass: Builder-first helpers such as `CardBuild`, `CardHeader::build(...)`, and `CardFooter::build(...)` still cover advanced composition when a slot needs extra policy (for example footer direction/gap).
- Pass: `CardSize::Sm` and per-slot layout/style refinements provide the expected recipe-level sizing hooks.
- Note: Recommended authoring pattern is: free helpers for the common path, slot builders for advanced per-slot policy, and root-level `refine_layout(...)` for call-site-owned width constraints.

### Layout & geometry (shadcn parity)

- Pass: Root chrome follows the upstream defaults: `rounded-xl`, `border`, `shadow-sm`, and vertical spacing between slots.
- Pass: Root width remains call-site owned; examples opt into widths such as `w-full max-w-sm` rather than the `Card` recipe forcing fill-width by default.
- Pass: Default recipe styles stay limited to intrinsic card chrome/slot spacing. Page- or container-negotiated constraints such as `w-full`, `min-w-0`, `max-w-*`, or `flex-1` stay at the call site unless the upstream recipe itself owns them.
- Pass: `CardHeader` keeps title/description/action alignment compatible with the upstream two-row grid outcome.
- Pass: `CardContent` and `CardFooter` preserve the expected horizontal padding and allow richer compositions without collapsing intrinsic child sizes.

## Validation

- `cargo test -p fret-ui-shadcn --lib card`
- `cargo check -p fret-ui-gallery`
