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

### Current conclusion

- Pass: the primary drift was in `ecosystem/fret-ui-shadcn/src/typography.rs`, not in `crates/fret-ui` mechanisms. The helper defaults had stayed too close to the general UI text baseline instead of the shadcn typography docs recipe.
- Pass: after updating the helper metrics/chrome/RTL logical edges, the remaining differences are docs-surface or call-site choices rather than a mechanism failure.
- Pass: the current gap is still richer inline composition (for links / mixed inline content), not a missing generic `children(Vec<AnyElement>)` API on the typography helpers.

### Surface classification

- Pass: upstream typography is a docs-only page demonstrating utility-class patterns rather than a shipped component implementation.
- Pass: Fret therefore treats typography as a helper/docs surface, not as a registry component that must satisfy strict prop-for-prop parity.
- Pass: `h1` / `h2` / `h3` / `h4` now also publish heading semantics (`SemanticsRole::Heading` with levels 1-4), matching the intent of the upstream heading tags without moving policy into `crates/fret-ui`.
- Pass: no extra generic block-children / `compose()` contract is added here; the remaining upstream gap is inline rich-text/link composition, which should land as a dedicated text surface rather than `children(Vec<AnyElement>)` on docs-only typography helpers.

### Ownership

- Pass: helper-owned defaults now include the shadcn docs-aligned metrics/chrome for `h1`, `h2`, `h3`, `h4`, `p`, `blockquote`, `inline_code`, `lead`, `large`, `small`, and `muted`.
- Pass: the corrected helper defaults now cover the real recipe drift: heading sizes/line-heights/tracking, `h2` bottom rule + padding, `inline_code` padding/radius/monospace weight, `blockquote` logical inline-start border/padding, and list inline-start spacing under both LTR and RTL.
- Pass: caller-owned concerns include semantic heading hierarchy, document layout, table/list composition, and the surrounding width/wrapping context.
- Pass: `h1` center alignment remains a caller-owned/docs-page decision because the upstream centering is applied on the example call site, not in the typography helper recipe itself.
- Pass: this keeps typography aligned with Fret's mechanism-vs-policy split: the helpers are convenient recipes, not a hard runtime contract.

### Gallery / docs parity

- Pass: the gallery mirrors the upstream typography page structure (`Demo`, headings, paragraph, blockquote, table, list, inline code, lead, large, small, muted, and RTL) and now uses the same sample headings/body copy for the focused sections.
- Pass: the full demo/RTL story now tracks the upstream content order and vertical rhythm more closely, replacing the previous one-gap-fits-all stacking with section-specific spacing closer to the docs page.
- Pass: the single inline-link sentence still stays flattened to plain text on the raw helper lane until inline link/rich-text composition is promoted as a separate contract.
- Pass: the gallery now also uses the upstream "Inline code" heading spelling, which keeps the display copy and the existing `docsec-inline-code-*` diagnostics anchors aligned.
- Pass: keeping the page available is still useful for copyable examples even though typography remains a docs/helper surface rather than a strict registry component baseline.
- Pass: the remaining page-level differences are intentional and narrow:
  - `Notes` is a Fret-specific follow-up section appended after the upstream docs flow.
  - Single-sample `h1` centering is still owned by the page/snippet, not baked into the helper defaults.
  - Inline link / mixed rich-text composition is deferred until a dedicated text composition surface exists.

## Validation

- `cargo nextest run -p fret-ui-shadcn --lib typography::tests::`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app typography_`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-docs-smoke.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/typography/ui-gallery-typography-inline-code-tab-scroll-range.json --pack --ai-packet --launch -- cargo run -p fret-ui-gallery --release`
