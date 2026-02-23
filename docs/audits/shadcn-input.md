# shadcn/ui v4 Audit — Input


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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

### File inputs (native)

- Note: Fret does not mirror DOM `type="file"`. The canonical recipe is `Input` + `Browse` button that opens a platform file dialog.
- Pass: UI gallery `Input` page wires `Browse` to `Effect::FileDialogOpen`, and diagnostics runs mock the picker to keep scripted gates deterministic.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_input_demo_geometry`).
- Invalid chrome gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_input_demo_aria_invalid_border_color_matches`).
- Focus ring chrome gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_input_demo_focus_ring_matches`, `web_vs_fret_input_demo_aria_invalid_focus_ring_matches`).
- Additional layout gate: `web_vs_fret_layout_input_with_label_geometry` (matches `gap-3`/`max-w-sm`
  form layout from `input-with-label`).
- UI gallery diag gate: `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-input-file-browse-mocked.json --launch -- cargo run -p fret-ui-gallery --release`.
