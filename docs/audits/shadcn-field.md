# shadcn/ui v4 Audit — Field

This audit compares Fret’s shadcn-aligned Field primitives against the upstream shadcn/ui v4
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/field.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-input.tsx`

## Fret implementation

- Components: `ecosystem/fret-ui-shadcn/src/field.rs`
- Theme tokens: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- `FieldGroup` matches `gap-7` (28px) via `component.field.group_gap`.
- `FieldLabel` matches `leading-snug` via `component.field.label_line_height`.
- `FieldDescription` matches `leading-normal` via `component.field.description_line_height`.
- `FieldDescription` spacing detail for “description before control” is supported (upstream uses
  `nth-last-2:-mt-1`).

## Validation

- Web layout gate:
  `cargo nextest run -p fret-ui-shadcn -F fret-ui/layout-engine-v2 --test web_vs_fret_layout`
  (`web_vs_fret_layout_field_input_geometry`).

