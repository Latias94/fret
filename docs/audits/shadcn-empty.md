# shadcn/ui v4 Audit - Empty

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Empty` against the upstream shadcn/ui v4 docs and the
in-repo web goldens that currently gate visual geometry.

## Upstream references (source of truth)

- Docs page order: `repo-ref/ui/apps/v4/content/docs/components/base/empty.mdx`
- Component implementations reviewed: `repo-ref/ui/apps/v4/examples/base/ui/empty.tsx`, `repo-ref/ui/apps/v4/examples/radix/ui/empty.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/empty-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/empty-outline.tsx`, `repo-ref/ui/apps/v4/examples/base/empty-background.tsx`, `repo-ref/ui/apps/v4/examples/base/empty-avatar.tsx`, `repo-ref/ui/apps/v4/examples/base/empty-avatar-group.tsx`, `repo-ref/ui/apps/v4/examples/base/empty-input-group.tsx`, `repo-ref/ui/apps/v4/examples/base/empty-rtl.tsx`
- Visual geometry gates: `goldens/shadcn-web/v4/new-york-v4/empty-demo.json`, `goldens/shadcn-web/v4/new-york-v4/empty-background.json`, `goldens/shadcn-web/v4/new-york-v4/empty-outline.json`

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

- Pass: the gallery now mirrors the upstream Empty docs path first: `Demo`, `Usage`, the example set through `RTL`, and `API Reference`.
- Pass: the gallery `Demo` and `RTL` snippets now follow the upstream `new-york-v4` teaching shape more closely: folder-code icon media, a centered two-button action row, and a semantic link CTA.
- Pass: the old gallery `Notes` section is replaced by an explicit `API Reference` section that records ownership and source-of-truth decisions.
- Pass: no mechanism-layer gap was identified in this pass; the work here is docs parity plus documenting the current recipe/golden ownership choice.

## Validation

- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --test empty_responsive_padding empty_padding_switches_at_md_using_container_queries -- --exact`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --test web_vs_fret_layout empty::web_vs_fret_layout_empty_geometry_matches_web_fixtures -- --exact`
- `CARGO_TARGET_DIR=target-codex-avatar cargo test -p fret-ui-shadcn --test web_vs_fret_layout empty::web_vs_fret_layout_empty_icon_geometry_matches_web -- --exact`
- `CARGO_TARGET_DIR=target-codex-avatar cargo nextest run -p fret-ui-gallery gallery_empty_demo_keeps_upstream_action_row_and_link_separation -- --exact`
- `CARGO_TARGET_DIR=target-codex-avatar cargo check -p fret-ui-gallery --message-format short`
