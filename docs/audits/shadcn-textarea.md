# shadcn/ui v4 Audit - Textarea


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `Textarea` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/textarea.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/textarea.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/textarea.rs`

## Audit checklist

### Authoring surface

- Pass: `Textarea::new(model)` covers the common shadcn authoring path.
- Pass: `Textarea::control_id(ControlId)` supports label association via `Label::for_control(...)` when forms need click-to-focus semantics.
- Note: `Textarea` is a leaf text control, so Fret intentionally does not add a generic `compose()` builder here.

### Layout & geometry (shadcn parity)

- Pass: Default minimum height matches `min-h-16` (64px).

### Semantics

- Pass: Exposes `SemanticsRole::TextField` and supports `a11y_label`.

### States (`aria-invalid`)

- Pass: `aria-invalid=true` border color matches shadcn-web (`textarea-demo.invalid`).
- Pass: Focus ring thickness (`ring-[3px]`) and invalid ring color overrides match shadcn-web focus variants
  (`textarea-demo.focus`, `textarea-demo.invalid-focus`).

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_textarea_demo_geometry`).
- Chrome + focus ring gates: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_control_chrome`
  (`web_vs_fret_textarea_demo_control_chrome_matches`, `web_vs_fret_textarea_demo_aria_invalid_border_color_matches`,
  `web_vs_fret_textarea_demo_focus_ring_matches`, `web_vs_fret_textarea_demo_aria_invalid_focus_ring_matches`).
