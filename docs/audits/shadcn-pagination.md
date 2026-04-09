# shadcn/ui v4 Audit - Pagination

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Pagination` against the upstream shadcn/ui v4 docs and
registry implementations in `repo-ref/ui`, with `repo-ref/primitives` / `repo-ref/base-ui` checked
as secondary headless references.

## Upstream references (source of truth)

Current source axes:

- Docs pages: `repo-ref/ui/apps/v4/content/docs/components/base/pagination.mdx`,
  `repo-ref/ui/apps/v4/content/docs/components/radix/pagination.mdx`
- Registry implementation (new-york visual baseline):
  `repo-ref/ui/apps/v4/registry/new-york-v4/ui/pagination.tsx`
- Base/radix registry copies (secondary structure check):
  `repo-ref/ui/apps/v4/registry/bases/base/ui/pagination.tsx`,
  `repo-ref/ui/apps/v4/registry/bases/radix/ui/pagination.tsx`
- Upstream demo: `repo-ref/ui/apps/v4/app/(internal)/sink/components/pagination-demo.tsx`
- No direct pagination primitive exists under `repo-ref/primitives` or `repo-ref/base-ui`

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
- Pass: UI Gallery `Usage` now demonstrates that explicit parts/children lane directly instead of
  teaching only the compact wrapper helpers, so the copyable teaching surface now matches the
  upstream docs shape more closely.
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
- Pass: `PaginationLink` defaults to icon-sized links, stamps active-page selection semantics when
  `active(true)` is set, and now resolves chrome from the shared shadcn button variant state tokens
  so outline focus borders, disabled opacity, and hover/press transitions stay aligned with the
  upstream `buttonVariants(...)` source of truth.
- Pass: `PaginationPrevious` / `PaginationNext` encode the responsive `sm` text visibility and swap
  icon direction correctly under RTL.
- Pass: `PaginationEllipsis` uses a centered 36px box with a 16px more icon and hides itself from the
  semantics tree while still labeling the content as `More pages`.

## Conclusion

- Result: This component does not currently point to a missing mechanism-layer gap, though a future
  `Navigation` landmark role in `fret-core` would let us tighten semantics further.
- Result: The remaining mechanism gap is still semantic naming (`Navigation` landmark / `aria-current`
  style dedicated state), not missing pagination layout primitives.
- Result: The concrete recipe drift was state-chrome ownership on `PaginationLink`: reusing the shared
  button variant state tokens is more correct than keeping a hand-rolled hover/focus/disabled chrome.
- Result: Composable children API is already supported via the explicit parts surface; the remaining
  gap was on the gallery teaching surface, not in the recipe or mechanism layer.
- Authoring-lane classification: treat `Pagination` as a dual-lane family.
  `Usage` should teach the explicit `Pagination` / `PaginationContent` / `PaginationItem` /
  `PaginationLink` parts surface as the upstream-shaped lane, while `Compact Builder` can keep the
  wrapper shorthand (`pagination(...)`, `pagination_content(...)`, `pagination_item(...)`,
  `pagination_link(...)`) visible as the Fret ergonomic follow-up.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib pagination_root_is_w_full_and_labeled pagination_content_and_item_emit_list_semantics pagination_link_active_stamps_selected pagination_ellipsis_is_hidden_in_semantics_tree pagination_active_link_focus_visible_border_uses_ring_token pagination_link_without_action_keeps_enabled_visual_chrome pagination_disabled_link_wraps_in_opacity --status-level fail`
- `cargo build -p fret-ui-gallery`
- `cargo test -p fret-ui-gallery --test pagination_docs_surface -- --nocapture`
- `cargo test -p fret-ui-shadcn --test web_vs_fret_layout pagination_demo`
- `./target/debug/fretboard-dev diag run tools/diag-scripts/ui-gallery/pagination/ui-gallery-pagination-docs-smoke.json --dir /tmp/fret-pagination-diag --session-auto --launch -- ./target/debug/fret-ui-gallery`
