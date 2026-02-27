# shadcn/ui v4 Audit — Badge


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `Badge` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/badge.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/badge.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/badge.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- Pass: Padding matches `px-2 py-0.5` and the chrome is `rounded-full`.
- Pass: Default height matches `line-height (16px) + 2*py (4px) + 2*border (2px) = 22px`.
- Pass: Label uses `font-medium` (not semibold).
- Pass: Defaults to `shrink-0` and `overflow-hidden` so the badge behaves like shadcn's `inline-flex` chip in constrained rows.
- Note: Width is label/font dependent, so we gate height only.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_badge_demo_heights`).
- Unit tests: `cargo nextest run -p fret-ui-shadcn badge_defaults_to_font_medium_and_shrink_0`.
