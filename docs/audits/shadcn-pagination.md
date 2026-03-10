# shadcn/ui v4 Audit - Pagination

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Pagination` against the upstream shadcn/ui v4 docs and
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

There is no standalone `components/pagination.mdx` page in the current v4 repo snapshot. Use
these sources instead:

- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/pagination.tsx`
- Upstream demo: `repo-ref/ui/apps/v4/app/(internal)/sink/components/pagination-demo.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/pagination.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/pagination.rs`
- Copyable usage snippet: `apps/fret-ui-gallery/src/ui/snippets/pagination/usage.rs`
- Gallery examples: `apps/fret-ui-gallery/src/ui/snippets/pagination/simple.rs`, `apps/fret-ui-gallery/src/ui/snippets/pagination/icons_only.rs`

## Audit checklist

### Authoring surface

- Pass: Fret already exposes the upstream-shaped composable parts API: `Pagination`,
  `PaginationContent`, `PaginationItem`, `PaginationLink`, `PaginationPrevious`, `PaginationNext`,
  and `PaginationEllipsis`.
- Pass: The common usage path is already representable with explicit children composition:
  `Pagination::new([PaginationContent::new([...]).into_element(cx)])`.
- Pass: `PaginationLink::active(true)`, `PaginationLink::size(...)`, `PaginationPrevious::text(...)`,
  `PaginationNext::text(...)`, and `test_id(...)` cover the important shadcn recipe outcomes.
- Note: Because the explicit parts surface already matches the upstream authoring model, Fret does
  not need an additional generic `compose()` builder here.
- Note: Upstream uses anchor tags plus `href`; Fret intentionally keeps navigation/routing in the app
  layer, so pagination links expose command/action hooks while preserving link semantics.

### Layout & behavior parity

- Pass: `Pagination` defaults to `w-full` and centered content, matching the upstream root container.
  Because Fret does not currently expose a dedicated `Navigation` landmark role, the root now uses
  `Region + label("pagination")` as the closest portable approximation of upstream `<nav aria-label="pagination">`.
- Pass: `PaginationContent` renders a horizontal row with `gap-1`, matching the upstream list layout,
  and now stamps `List` semantics while `PaginationItem` stamps `ListItem` semantics to approximate
  the upstream `ul/li` structure.
- Pass: `PaginationLink` defaults to icon-sized links and stamps active-page selection semantics when
  `active(true)` is set.
- Pass: `PaginationPrevious` / `PaginationNext` encode the responsive `sm` text visibility and swap
  icon direction correctly under RTL.
- Pass: `PaginationEllipsis` uses a centered 36px box with a 16px more icon and hides itself from the
  semantics tree while still labeling the content as `More pages`.

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer gap, though a future
  `Navigation` landmark role in `fret-core` would let us tighten semantics further.
- Result: The main drift was semantics parity at the parts boundary (`nav`/`ul`/`li`-like outcomes), not
  default-style ownership. The root and inline layout defaults were already in the right place.
- Result: Composable children API is already supported via the explicit parts surface, so follow-up work
  should focus on richer examples or diag gates only if a concrete parity regression appears.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib pagination_root_is_w_full_and_labeled pagination_content_and_item_emit_list_semantics pagination_link_active_stamps_selected pagination_ellipsis_is_hidden_in_semantics_tree --status-level fail`
