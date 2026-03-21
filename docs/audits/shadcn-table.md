# shadcn/ui v4 Audit - Table

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Table` against the upstream shadcn/ui v4 docs and the
current registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/table.mdx`
- Component implementation:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/table.tsx`
  - `repo-ref/ui/apps/v4/examples/base/ui/table.tsx`
- Example compositions:
  - `repo-ref/ui/apps/v4/registry/new-york-v4/examples/table-demo.tsx`
  - `repo-ref/ui/apps/v4/registry/bases/base/examples/table-example.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/table.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/table.rs`
- Copyable snippets: `apps/fret-ui-gallery/src/ui/snippets/table/*.rs`

## Audit checklist

### Surface classification

- Pass: this is still a recipe/public-surface task, not a `fret-ui` mechanism issue. Root overflow,
  row hover/selected chrome, footer/background ownership, and the `DataTable` handoff already live
  in the correct layer.
- Pass: `Table`, `TableHeader`, `TableBody`, `TableFooter`, `TableRow`, `TableHead`, `TableCell`,
  and `TableCaption` still expose the expected upstream-shaped parts surface.
- Pass: the root `Table` recipe still owns the responsive `w-full overflow-x-auto` wrapper
  outcome, matching upstream source ownership.

### Public surface parity

- Pass: `TableHead::new_children(...)` plus `table_head_children(...)` now cover the missing
  composable `th` lane that upstream exposes directly through `React.ComponentProps<"th">`.
- Pass: `TableCaption::new_children(...)` plus `table_caption_children(...)` now cover the missing
  composable `caption` lane while preserving recipe-owned caption typography and muted foreground.
- Pass: `table_head(...)` and `table_caption(...)` remain the compact text-first constructors for
  the common docs examples, so the new children lane does not replace the default authoring path.
- Note: `TableCell` remains on a single-child-root surface for now because callers can already pass
  an arbitrary composed child subtree through `table_cell(...)`, and this audit did not find enough
  first-party pressure to widen it into a sibling-child collector yet.
- Note: no extra generic `compose()` root builder is warranted here; the free wrapper family plus
  explicit part structs already cover the common shadcn lane.

### Layout & behavior parity

- Pass: root width/overflow ownership still matches upstream wrapper ownership.
- Pass: header/body/footer/caption remain separate recipe boundaries, preserving the same authoring
  split as shadcn.
- Pass: table rows still keep hover/selected background behavior in the recipe layer without
  leaking data-table policy into the base component.
- Pass: the existing row-height/caption-gap and data-table geometry gates still indicate that the
  core table layout is not blocked on a runtime/mechanism change.
- Note: checkbox-column padding/offset parity is still best treated as a narrow recipe follow-up if
  future evidence shows visible drift; it is not a reason to widen the mechanism layer.

### Gallery / docs parity

- Pass: the gallery now mirrors the shadcn docs flow after `Installation`: `Demo`, `Usage`,
  `Footer`, `Actions`, `Data Table`, `RTL`, and `API Reference`.
- Pass: `Children (Fret)` is now an explicit follow-up section that documents the new composable
  `table_head_children(...)` / `table_caption_children(...)` lane without displacing the upstream
  docs path.
- Pass: `Data Table` now appears as a dedicated handoff section instead of being buried inside
  prose notes, which matches the way upstream docs position the TanStack-backed guide.
- Pass: the remaining diagnosis is now explicit on the page: this was public-surface/docs drift in
  `fret-ui-shadcn` + UI Gallery, not a `fret-ui` mechanism defect.

## Validation

- `cargo nextest run -p fret-ui-shadcn table_head_children table_caption_children --status-level fail`
- `cargo nextest run -p fret-ui-shadcn table_root_defaults_to_w_full_but_allows_overrides table_build_ui_builder_path_applies_layout_patches --status-level fail`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app table_page_uses_typed_doc_sections_for_app_facing_snippets`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app selected_table_snippets_prefer_table_wrapper_family`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
