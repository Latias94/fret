# shadcn/ui v4 Audit — Textarea

This audit compares Fret’s shadcn-aligned `Textarea` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/textarea.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/textarea.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- Pass: Default minimum height matches `min-h-16` (64px).

### Semantics

- Pass: Exposes `SemanticsRole::TextField` and supports `a11y_label`.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn -F fret-ui/layout-engine-v2 --test web_vs_fret_layout`
  (`web_vs_fret_layout_textarea_demo_geometry`).

