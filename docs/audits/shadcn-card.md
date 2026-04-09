# shadcn/ui v4 Audit - Card

Status note (2026-04-01): the dedicated card shadow footprint gate now exists in
`ecosystem/fret-ui-shadcn/tests/web_vs_fret_control_chrome.rs`, and `shadow-sm` was realigned to
the current `new-york-v4` web footprint for this recipe. The broader repo-wide shadow surface
closure is still tracked in:

- `docs/workstreams/shadow-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/shadow-surface-fearless-refactor-v1/TODO.md`
- `docs/workstreams/shadow-surface-fearless-refactor-v1/MILESTONES.md`

References below to `shadow-sm` are therefore no longer purely provisional for `Card`. The broader
v1 shadow contract/theme closure is now captured in ADR 0060, ADR 0286, and
`docs/adr/IMPLEMENTATION_ALIGNMENT.md`. Remaining follow-up in the linked workstream is limited to
cleanup of stale docs and re-auditing remaining manual shadow sites, not the earlier contract/theme
decision itself.

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
- Pass: `Card` shadow footprint now has a dedicated light/dark web-vs-Fret gate, so `shadow-sm`
  on this recipe is backed by direct footprint evidence rather than inferred visual judgment.
- Pass: Root width remains call-site owned; examples opt into widths such as `w-full max-w-sm` rather than the `Card` recipe forcing fill-width by default.
- Pass: Default recipe styles stay limited to intrinsic card chrome/slot spacing. Page- or container-negotiated constraints such as `w-full`, `min-w-0`, `max-w-*`, or `flex-1` stay at the call site unless the upstream recipe itself owns them.
- Pass: `CardHeader` now uses the same source-aligned grid family as upstream instead of a flex
  approximation:
  - explicit `auto auto` header rows,
  - explicit `1fr auto` columns when `CardAction` is present,
  - `CardAction` placed in column 2, row 1, spanning both header rows,
  - `CardAction` now also carries the upstream `self-start` / `justify-self-end` slot alignment.
- Pass: the runtime now translates in-flow grid-item `Fill` to grid-area stretch semantics in this
  slot family, so the first column no longer expands to the full card width and push the action
  lane outside the card when title/description helpers request fill width.
- Pass: `CardContent` and `CardFooter` preserve the expected horizontal padding and allow richer compositions without collapsing intrinsic child sizes.
- Pass: `CardFooter` row/column roots now request `w_full().min_w_0()` so footer-only or narrow-card text wraps against the card's inner width instead of collapsing into one word per line.
- Verdict: Card parity required both mechanism and recipe work. The visible docs-path demo bugs were
  local snippet issues, but the durable fix required:
  - a runtime grid-track contract strong enough to express the upstream header slot geometry, and
  - a runtime grid-item fill mapping that preserves `fr auto` slot lanes instead of expanding the
    first track against the whole card.
  `CardContent` and `CardFooter` remain recipe-level translations; `CardHeader` is no longer a
  `justify-between` approximation.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Size`, `Image`, `RTL`, and `API Reference`.
- Pass: `Rich Title (Fret)` and `Rich Description (Fret)` now follow `API Reference` as the copyable app-facing teaching lanes for `card_title_children(...)` and `card_description_children(...)`, so rich text content does not require teaching slot builders directly.
- Pass: `Compositions`, `CardContent`, and `Meeting Notes` remain explicit Fret follow-ups after the upstream path, because they lock regression coverage for slot permutations and intrinsic-size behavior.
- Pass: `API Reference` now spells out the Fret public surface explicitly, including caller-owned width, `CardFooter` direction/gap ownership, and the `CardTitle` children lane.
- Pass: the `footer only` drift was a recipe-owned `CardFooter` width-budget issue, not a page-level `w-full` / `min-w-0` problem in the gallery composition.

## Validation

- `cargo nextest run -p fret-ui-shadcn card_footer_row_requests_fill_width_and_min_w_0 --status-level fail`
- `CARGO_TARGET_DIR=/tmp/fret-codex-card-target cargo nextest run -p fret-ui -E 'test(grid_places_children_in_columns) or test(grid_explicit_tracks_place_spanning_child_in_source_aligned_lanes)'`
- `CARGO_TARGET_DIR=/tmp/fret-codex-card-target cargo nextest run -p fret-ui-shadcn --lib -E 'test(card_header_without_action_uses_source_aligned_grid_layout) or test(card_header_with_action_uses_explicit_grid_slot_placement)'`
- `cargo nextest run -p fret-ui-shadcn card_title_children --status-level fail`
- `cargo nextest run -p fret-ui-shadcn card --status-level fail`
- `cargo nextest run -p fret-ui-shadcn web_vs_fret_card_demo_shadow_matches_web_light --status-level fail`
- `cargo nextest run -p fret-ui-shadcn web_vs_fret_card_demo_shadow_matches_web_dark --status-level fail`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app card_page_uses_typed_doc_sections_for_app_facing_snippets`
- `cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app card_rich_title_snippet_prefers_copyable_card_title_children_helper`
- `CARGO_TARGET_DIR=/tmp/fret-codex-card-target cargo nextest run -p fret-ui-gallery --lib -E 'test(gallery_card_demo_header_action_stays_in_the_upstream_top_right_lane) or test(gallery_card_demo_keeps_docs_form_controls_visible_and_aligned)'`
- `CARGO_TARGET_DIR=/tmp/fret-codex-card-target cargo nextest run -p fret-ui-gallery --test card_docs_surface -E 'test(card_page_documents_source_axes_and_children_api_decision) or test(card_docs_path_snippets_stay_copyable_and_docs_aligned) or test(card_docs_diag_script_covers_docs_path_and_fret_followups)'`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/card/ui-gallery-card-docs-smoke.json --session-auto --launch -- cargo run -p fret-ui-gallery --release`
- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
