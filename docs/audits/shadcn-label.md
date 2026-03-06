# shadcn/ui v4 Audit — Label


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned `Label` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/label.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/label.tsx`
- Underlying primitive: Radix `@radix-ui/react-label`

## Fret implementation

- Primitive implementation: `ecosystem/fret-ui-kit/src/primitives/label.rs`
- Re-export surface: `ecosystem/fret-ui-shadcn/src/label.rs`

## Audit checklist

### Authoring surface

- Pass: `Label::new(text)` covers the common shadcn path.
- Pass: `Label::for_control(ControlId)` models the upstream `htmlFor` association and forwards
  click-to-focus / control activation through Fret's control registry.
- Note: `Label` is already a minimal leaf primitive, so Fret intentionally does not add a generic
  `compose()` builder here.

### Layout & geometry (shadcn parity)

- Pass: Uses `text-sm` (14px) and `leading-none` (14px line-height) via theme metrics:
  `component.label.text_px` / `component.label.line_height`.
- Pass: `label-demo` layout matches the web golden: checkbox (16px) + `space-x-2` (8px) alignment.

## Validation

- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_label_demo_geometry`).
