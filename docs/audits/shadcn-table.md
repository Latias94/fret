# shadcn/ui v4 Audit - Table

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Table` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/table.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/table.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/table.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/table.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/table/usage.rs`

## Audit checklist

### Authoring surface

- Pass: Fret already exposes the upstream-shaped parts API: `Table`, `TableHeader`, `TableBody`,
  `TableFooter`, `TableRow`, `TableHead`, `TableCell`, and `TableCaption`.
- Pass: Standard shadcn authoring is already representable with explicit children composition through
  `Table::new([...])` plus the row/cell parts.
- Pass: `TableHead::text_align_end()`, `TableCell::text_align_end()`, `TableCell::col_span(...)`,
  and layout refinement hooks cover the important shadcn recipe outcomes.
- Note: Because the explicit parts surface already matches the upstream authoring model, Fret does
  not need an additional generic `compose()` builder here.

### Layout & behavior parity

- Pass: The root `Table` recipe owns the responsive horizontal overflow container, matching the
  upstream `relative w-full overflow-x-auto` wrapper outcome.
- Pass: Header/body/footer/caption remain separate parts, preserving the same authoring boundaries as shadcn.
- Pass: Table rows expose hover/selection-ready row chrome without forcing higher-level data-table policy
  into the base component.
- Pass: Caption placement and first-column fixed width examples are representable in gallery snippets.

## Conclusion

- Result: This component does not currently indicate a missing mechanism-layer issue.
- Result: The main missing piece was gallery/docs alignment with the upstream `Usage` section.
- Result: Complex data-grid behavior belongs in `DataTable` recipes, not the base `Table` component.

## Validation

- `cargo check -p fret-ui-gallery --message-format short`
