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
  - `repo-ref/ui/apps/v4/examples/base/table-footer.tsx`
  - `repo-ref/ui/apps/v4/examples/base/table-actions.tsx`
  - `repo-ref/ui/apps/v4/examples/base/table-rtl.tsx`

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
- Pass: `TableCell` still stays on a single-child-root surface, and `text_align_end()` now aligns
  both plain text and a composed child root (for example an actions dropdown trigger) without
  forcing callers back through an app-side wrapper helper.
- Note: this audit still did not find enough first-party pressure to widen `TableCell` into a
  sibling-child collector; `table_cell(...)` remains the right lane for a composed child subtree.
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
- Pass: checkbox-column padding parity now stays where upstream owns it: `TableHead` /
  `TableCell` narrow recipe defaults drop right padding when a checkbox descendant is present.
- Pass: mixed-height body-row checkbox cells now also center vertically at the recipe layer via
  auto vertical margins on the cell root, matching the `align-middle` outcome without widening any
  `fret-ui` mechanism contract.
- Pass: `TableCell::text_align_end()` now also positions non-text child roots at the inline end,
  closing the recipe/API seam that previously forced the gallery actions example to wrap the
  dropdown trigger in a local alignment helper.
- Note: the upstream direct-child `translate-y-[2px]` rule remains a DOM/table-baseline
  compensation; Fret's flex-backed table did not need a literal transform port once checkbox cells
  centered correctly inside taller rows.

### Gallery / docs parity

- Pass: the gallery now mirrors the shadcn docs flow after `Installation`: `Demo`, `Usage`,
  `Footer`, `Actions`, `Data Table`, `RTL`, and `API Reference`.
- Pass: the page copy now records the exact docs/source axes (`table.mdx`, `ui/table.tsx`, and the
  demo/footer/actions/rtl example files) plus the conclusion that this was not a missing
  `fret-ui` mechanism or extra Radix/Base UI primitive port.
- Pass: `Children (Fret)` is now an explicit follow-up section that documents the new composable
  `table_head_children(...)` / `table_caption_children(...)` lane without displacing the upstream
  docs path.
- Pass: the `Actions` snippet now matches the docs story more closely: upstream product/price copy,
  ghost icon trigger, end-aligned dropdown content, and no gallery-only `align_end(...)` wrapper.
- Pass: the `RTL` snippet now follows the docs story instead of a shortened English stub: Arabic
  copy, seven invoice rows, translated caption/footer labels, and the same footer presence as the
  upstream example.
- Pass: `Data Table` now appears as a dedicated handoff section instead of being buried inside
  prose notes, which matches the way upstream docs position the TanStack-backed guide.
- Pass: the remaining diagnosis is now explicit on the page: this was public-surface/docs drift in
  `fret-ui-shadcn` + UI Gallery, not a `fret-ui` mechanism defect.
- Pass: the checkbox/table gallery snippet now uses a header cell surface for the select-all
  column instead of bypassing `TableHead` through `table_cell(...)`.
- Pass: the former native diagnostics timeout was split into two concrete issues and resolved:
  `diag windows` was misreporting `window_bounds` for older/logical-key sidecars, and the table docs
  smoke script was letting `scroll_into_view` succeed on partial intersection before a stricter
  `bounds_within_window` check. The diagnostics command now reads both bounds-key shapes, and the
  `Actions` / `RTL` scroll steps now require the headings to land fully within the visible region.

## Validation

- `cargo nextest run -p fret-ui-shadcn table_head_children table_caption_children --status-level fail`
- `cargo test -p fret-ui-shadcn --lib 'table::tests::table_head_checkbox' -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib 'table::tests::table_cell_checkbox' -- --nocapture`
- `cargo test -p fret-ui-shadcn --lib 'table::tests::table_cell_text_align_end_aligns_non_text_child_roots_without_fill_width_wrapper' -- --exact`
- `cargo nextest run -p fret-ui-shadcn table_root_defaults_to_w_full_but_allows_overrides table_build_ui_builder_path_applies_layout_patches --status-level fail`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app table_page_uses_typed_doc_sections_for_app_facing_snippets`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app selected_table_snippets_prefer_table_wrapper_family`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app selected_table_snippet_helpers_prefer_into_ui_element_over_anyelement`
- `cargo test -p fret-ui-gallery --test table_docs_surface`
- `cargo test -p fret-ui-shadcn --test web_vs_fret_layout 'table::web_vs_fret_layout_data_table_demo_checkbox_column_padding_and_action_button_size' -- --exact`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/table/ui-gallery-table-docs-smoke.json --dir /tmp/fret-diag-table-docs-2 --session-auto --timeout-ms 900000 --poll-ms 200 --launch -- env CARGO_TARGET_DIR=/tmp/fret-table-diag-target-2 CARGO_NET_OFFLINE=true cargo run -p fret-ui-gallery --release`
