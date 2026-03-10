# shadcn/ui v4 Audit — Typography

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit records why Fret keeps typography as a docs/helper surface rather than treating it as a
`registry:ui` component contract.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/typography.mdx`

## Fret implementation

- Helper module: `ecosystem/fret-ui-shadcn/src/typography.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/typography.rs`

## Audit checklist

### Surface classification

- Pass: upstream typography is a docs-only page demonstrating utility-class patterns rather than a shipped component implementation.
- Pass: Fret therefore treats typography as a helper/docs surface, not as a registry component that must satisfy strict prop-for-prop parity.
- Pass: no extra generic `children` / `compose()` contract is needed here because the helpers are just focused convenience functions for common long-form text patterns.

### Ownership

- Pass: helper-owned defaults include the preset text styles for `h1`, `h2`, `h3`, `h4`, `p`, `blockquote`, `inline_code`, `lead`, `large`, `small`, and `muted`.
- Pass: caller-owned concerns include semantic heading hierarchy, document layout, table/list composition, and the surrounding width/wrapping context.
- Pass: this keeps typography aligned with Fret's mechanism-vs-policy split: the helpers are convenient recipes, not a hard runtime contract.

### Gallery / docs parity

- Pass: the gallery already mirrors the upstream typography page structure (`Demo`, headings, paragraph, blockquote, table, list, inline code, lead, large, small, muted, and RTL).
- Pass: keeping the page available is still useful for copyable examples even though the status remains `Skip` in the registry baseline table.
- Pass: this is a deliberate `Skip` because the upstream page is documentation, not a true shipped component.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
