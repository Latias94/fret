# shadcn/ui v4 Audit - Separator


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Separator` against the upstream shadcn/ui v4 docs and
the base implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/base/separator.mdx`
- Component implementation: `repo-ref/ui/apps/v4/examples/base/ui/separator.tsx`
- Example compositions: `repo-ref/ui/apps/v4/examples/base/separator-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-vertical.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-menu.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-list.tsx`, `repo-ref/ui/apps/v4/examples/base/separator-rtl.tsx`

## Fret implementation

- Primitive: `ecosystem/fret-ui-kit/src/primitives/separator.rs`
- shadcn re-export: `ecosystem/fret-ui-shadcn/src/separator.rs`

## Audit checklist

### Authoring surface

- Pass: `Separator::new()` covers the common shadcn authoring path.
- Pass: `Separator::orientation(...)` covers the vertical separator variant used by the upstream demo.
- Note: `Separator` is a minimal leaf primitive, so Fret intentionally does not add a generic `compose()` builder here.

### Gallery / docs parity

- Pass: the gallery now mirrors the upstream docs path first: `Demo`, `Usage`, `Vertical`, `Menu`, `List`, `RTL`, and `API Reference`.
- Pass: the previous mixed demo has been split into dedicated `Vertical`, `Menu`, and `List` examples so the page is source-comparable against the upstream docs headings.
- Pass: no extra generic composition API is needed; the remaining work for this component is docs/page clarity.

### Layout & geometry (shadcn parity)

- Pass: Horizontal separators are `1px` tall and fill available width.
- Pass: Vertical separators are `1px` wide and fill available height.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_separator_demo_geometry`).
