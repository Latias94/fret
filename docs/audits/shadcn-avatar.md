# shadcn/ui v4 Audit — Avatar


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `Avatar` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/avatar.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/avatar.tsx`
- Underlying primitive: Radix `@radix-ui/react-avatar`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/avatar.rs`
- Radix-aligned fallback delay helper: `ecosystem/fret-ui-kit/src/primitives/avatar.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- Pass: Root defaults to `size-8` (32px) and `shrink-0`.
- Pass: Root uses overflow clipping and a fully-rounded shape by default.
- Pass: `rounded-lg` can be expressed via `ChromeRefinement::rounded(Radius::Lg)` for parity demos.
- Pass: Group overlap can be expressed via `LayoutRefinement::ml_neg(Space::N2)` to match `-space-x-2`.
- Note: The web golden gates the avatar-group **visual** bounds by unioning the avatar item rects,
  rather than relying on a wrapper node's bounds.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_avatar_demo_geometry`).
- Additional avatar gates in the same suite:
  - `web_vs_fret_layout_empty_avatar_geometry`
  - `web_vs_fret_layout_empty_avatar_group_geometry`
  - `web_vs_fret_layout_item_avatar_geometry`
