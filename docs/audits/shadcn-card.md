# shadcn/ui v4 Audit - Card


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Card` against the upstream shadcn/ui v4 docs and base
example implementations in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/card.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/bases/base/ui/card.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/card-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/card-with-form.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/card.rs`

## Audit checklist

### Authoring surface

- Pass: `Card::new([...])` plus `CardHeader` / `CardContent` / `CardFooter` / `CardTitle` / `CardDescription` / `CardAction` covers the common shadcn authoring path.
- Pass: Free helpers (`card`, `card_header`, `card_title`, `card_description`, `card_action`, `card_content`, `card_footer`) now cover the copyable “children-style” path without forcing every example to allocate slot structs manually.
- Pass: `CardTitle::new_children(...)` plus `card_title_children(...)` now cover the missing title-side composable children lane for rich/selectable text and caller-owned composition roots.
- Pass: `CardDescription::new_children(...)` plus `card_description_children(...)` cover the matching rich/selectable description lane, so both text slots preserve recipe-owned typography while still accepting caller-owned child trees.
- Pass: No new generic `children(...)` API is warranted at the root/section layer. `Card::new([...])`, slot `::new([...])`, and the helper-family builders already provide the JSX-style composable children surface without widening the public API further.
- Pass: Builder-first helpers such as `CardBuild`, `CardHeader::build(...)`, and `CardFooter::build(...)` still cover advanced composition when a slot needs extra policy (for example footer direction/gap).
- Pass: `CardSize::Sm` and per-slot layout/style refinements provide the expected recipe-level sizing hooks.
- Note: Recommended authoring pattern is: free helpers for the common path, slot builders for advanced per-slot policy, root-level `refine_layout(...)` for call-site-owned width constraints, and title/description children helpers only when the compact text lane is genuinely too narrow.

### Layout & geometry (shadcn parity)

- Pass: Root chrome follows the upstream defaults: `rounded-xl`, `border`, `shadow-sm`, and vertical spacing between slots.
- Pass: Root width remains call-site owned; examples opt into widths such as `w-full max-w-sm` rather than the `Card` recipe forcing fill-width by default.
- Pass: Default recipe styles stay limited to intrinsic card chrome/slot spacing. Page- or container-negotiated constraints such as `w-full`, `min-w-0`, `max-w-*`, or `flex-1` stay at the call site unless the upstream recipe itself owns them.
- Pass: `CardHeader` keeps title/description/action alignment compatible with the upstream two-row grid outcome.
- Pass: `CardContent` and `CardFooter` preserve the expected horizontal padding and allow richer compositions without collapsing intrinsic child sizes.
- Pass: `CardFooter` row/column roots now request `w_full().min_w_0()` so footer-only or narrow-card text wraps against the card's inner width instead of collapsing into one word per line.
- Verdict: the remaining layout translation is recipe-level, not mechanism-level. `CardContent` uses `items_start()` because Fret's helper stacks are real flex containers, while upstream `CardContent` is a plain `div`; `CardFooter` requests `w_full().min_w_0()` to keep DOM-like wrap behavior inside Fret's GPU-first layout model.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Size`, `Image`, `RTL`, and `API Reference`.
- Pass: `Rich Title (Fret)` and `Rich Description (Fret)` now follow `API Reference` as the copyable app-facing teaching lanes for `card_title_children(...)` and `card_description_children(...)`, so rich text content does not require teaching slot builders directly.
- Pass: `Compositions`, `CardContent`, and `Meeting Notes` remain explicit Fret follow-ups after the upstream path, because they lock regression coverage for slot permutations and intrinsic-size behavior.
- Pass: `API Reference` now spells out the Fret public surface explicitly, including caller-owned width, `CardFooter` direction/gap ownership, and the `CardTitle` children lane.
- Pass: the `footer only` drift was a recipe-owned `CardFooter` width-budget issue, not a page-level `w-full` / `min-w-0` problem in the gallery composition.

## Validation

- `cargo nextest run -p fret-ui-shadcn card_footer_row_requests_fill_width_and_min_w_0 --status-level fail`
- `cargo nextest run -p fret-ui-shadcn card_title_children --status-level fail`
- `cargo nextest run -p fret-ui-shadcn card --status-level fail`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app card_page_uses_typed_doc_sections_for_app_facing_snippets`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app card_rich_title_snippet_prefers_copyable_card_title_children_helper`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/card/ui-gallery-card-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
