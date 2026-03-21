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
- Pass: `h1` / `h2` / `h3` / `h4` now also publish heading semantics (`SemanticsRole::Heading` with levels 1-4), matching the intent of the upstream heading tags without moving policy into `crates/fret-ui`.
- Pass: no extra generic block-children / `compose()` contract is added here; the remaining upstream gap is inline rich-text/link composition, which should land as a dedicated text surface rather than `children(Vec<AnyElement>)` on docs-only typography helpers.

### Ownership

- Pass: helper-owned defaults include the preset text styles for `h1`, `h2`, `h3`, `h4`, `p`, `blockquote`, `inline_code`, `lead`, `large`, `small`, and `muted`.
- Pass: caller-owned concerns include semantic heading hierarchy, document layout, table/list composition, and the surrounding width/wrapping context.
- Pass: this keeps typography aligned with Fret's mechanism-vs-policy split: the helpers are convenient recipes, not a hard runtime contract.

### Gallery / docs parity

- Pass: the gallery mirrors the upstream typography page structure (`Demo`, headings, paragraph, blockquote, table, list, inline code, lead, large, small, muted, and RTL) and now uses the same sample headings/body copy for the focused sections.
- Pass: the full demo/RTL story now tracks the upstream content order more closely, while keeping the single inline-link sentence flattened to plain text on the raw helper lane until inline link/rich-text composition is promoted as a separate contract.
- Pass: the gallery now also uses the upstream "Inline code" heading spelling, which keeps the display copy and the existing `docsec-inline-code-*` diagnostics anchors aligned.
- Pass: keeping the page available is still useful for copyable examples even though the status remains `Skip` in the registry baseline table.
- Pass: this is a deliberate `Skip` because the upstream page is documentation, not a true shipped component.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib typography::tests::typography_headings_attach_heading_semantics_levels -- --exact`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_typography web_vs_fret_typography_demo_targeted_gate_light_contract -- --exact`
- `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout layout_typography_fixtures::web_vs_fret_layout_typography_geometry_matches_web_fixtures -- --exact`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app typography_page_uses_typed_doc_sections_for_app_facing_snippets -- --exact`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-docs-smoke.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-inline-code-tab-scroll-range.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`
