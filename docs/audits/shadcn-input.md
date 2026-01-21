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

### States (`aria-invalid`)

- Pass: `aria-invalid=true` border color matches shadcn-web (`input-demo.invalid`).
- Note: shadcn's `aria-invalid:ring-*` is a ring color override; the ring only becomes visible when `focus-visible:ring-[3px]` is active.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_input_demo_geometry`).
- Invalid chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_input_demo_aria_invalid_border_color_matches`).
- Focus ring chrome gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_input_demo_focus_ring_matches`, `web_vs_fret_input_demo_aria_invalid_focus_ring_matches`).
- Additional layout gate: `web_vs_fret_layout_input_with_label_geometry` (matches `gap-3`/`max-w-sm`
  form layout from `input-with-label`).
