# shadcn/ui v4 Audit - Empty

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Empty` against the upstream shadcn/ui v4 docs and the
in-repo web goldens that currently gate visual geometry.

## Upstream references (source of truth)

- Docs page order: `repo-ref/ui/apps/v4/content/docs/components/empty.mdx`
- Default visual recipe source reviewed: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/empty.tsx`
- Example compositions:
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-demo.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-outline.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-background.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-avatar.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-avatar-group.tsx`,
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-input-group.tsx`, and
  `repo-ref/ui/apps/v4/registry/new-york-v4/examples/empty-icon.tsx`.
- Visual geometry gates: `goldens/shadcn-web/v4/new-york-v4/empty-demo.json`,
  `goldens/shadcn-web/v4/new-york-v4/empty-background.json`,
  `goldens/shadcn-web/v4/new-york-v4/empty-outline.json`,
  `goldens/shadcn-web/v4/new-york-v4/empty-icon.json`,
  `goldens/shadcn-web/v4/new-york-v4/empty-avatar.json`,
  `goldens/shadcn-web/v4/new-york-v4/empty-avatar-group.json`, and
  `goldens/shadcn-web/v4/new-york-v4/empty-input-group.json`.

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/empty.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/empty.rs`

## Audit checklist

### Authoring surface

- Pass: `Empty::new([...])` plus `EmptyHeader`, `EmptyMedia`, `EmptyTitle`, `EmptyDescription`, and `EmptyContent` matches the upstream slot model directly, and the gallery `Usage` snippet now teaches that eager compound-children lane first.
- Pass: `EmptyMedia::variant(...)` covers the documented `default` and `icon` variants without widening the public surface.
- Pass: no extra generic `asChild` / `compose()` helper is needed here; the current children-based slot API already matches the upstream composition story, and CTA link semantics can stay button-owned through `ButtonRender::Link`.

### Layout & default-style ownership

- Pass: the current recipe stays aligned to the in-repo `new-york-v4` web geometry gates that already cover `empty-demo`, `empty-background`, and `empty-outline`.
- Pass: this means the recipe currently keeps the existing chrome/spacing baseline (`p-6 md:p-12`, `gap-6`, rounded dashed card chrome) rather than re-translating the base source classes one-to-one in this pass.
- Note: a direct port of the base source defaults (`gap-4`, fixed `p-6`, smaller title/media sizing, and no default border width) diverged from the current gated web geometry. That source-of-truth tension should be resolved deliberately in a follow-up instead of slipping in as an incidental refactor.
- Pass: preview min-height, background paint, inline content layout, embedded `InputGroup` width, and page/grid placement remain caller-owned refinements.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream Empty docs path first: `Demo`, `Usage`, examples
  through `InputGroup`, and `API Reference`; the RTL example is kept as an explicit Fret follow-up
  because the current upstream Empty docs do not include an RTL example.
- Pass: the `Usage` snippet now leads with direct `Empty::new([...])` compound-children composition, which matches the upstream JSX slot model more closely than the lazy wrapper helpers.
- Pass: the gallery `Demo` snippet follows the upstream `new-york-v4` teaching shape closely:
  folder-code icon media, a centered two-button action row, and a semantic link CTA. The RTL
  follow-up keeps the same structure under `LayoutDirection::Rtl`.
- Pass: the old gallery `Notes` section is replaced by an explicit `API Reference` section that records ownership and source-of-truth decisions.
- Pass: no mechanism-layer gap was identified in this pass; the remaining nuance is source-of-truth split between base docs page structure and `new-york-v4` geometry gates, so the work here is teaching-surface parity plus documenting the current recipe/golden ownership choice.

## Validation

- `python -m json.tool tools/diag-scripts/suites/ui-gallery-empty-demo-action-state/suite.json | Out-Null`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/empty -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail empty`
- `cargo nextest run -p fret-ui-shadcn --test empty_responsive_padding --status-level fail`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail empty`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_empty --status-level fail`
- `cargo nextest run -p fret-ui-gallery --test empty_docs_surface --status-level fail`
- `cargo nextest run -p fret-ui-gallery --lib --status-level fail gallery_empty_demo_keeps_upstream_action_row_and_link_separation`
