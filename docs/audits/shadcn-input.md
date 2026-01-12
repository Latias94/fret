# shadcn/ui v4 Audit — Input

This audit compares Fret’s shadcn-aligned `Input` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/input.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/input.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/input.rs`
- Shared chrome resolver: `ecosystem/fret-ui-kit/src/recipes/input.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- Pass: Default height matches `h-9` from the web golden (`input-demo`).
- Note: Width is typically `w-full`, so we gate width only in scenarios with deterministic parent bounds.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn -F fret-ui/layout-engine-v2 --test web_vs_fret_layout`
  (`web_vs_fret_layout_input_demo_geometry`).

