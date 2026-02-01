# shadcn/ui v4 Audit — Field

This audit compares Fret’s shadcn-aligned Field primitives against the upstream shadcn/ui v4
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/field.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-input.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-checkbox.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-group.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-fieldset.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-choice-card.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-switch.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-select.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-radio.tsx`
- Example: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-textarea.tsx`

## Fret implementation

- Components: `ecosystem/fret-ui-shadcn/src/field.rs`
- Theme tokens: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- `FieldGroup` matches `gap-7` (28px) via `component.field.group_gap`.
- `FieldGroup` supports per-instance gap overrides (e.g. `gap-3`) for checkbox/radio group recipes.
- `FieldSet` accounts for HTML `legend` formatting quirks observed in the web goldens (legend spacing
  is driven by `mb-3` + `FieldDescription` negative top margin rather than flex `gap`).
- `FieldSet` matches shadcn's `:has([data-slot=radio-group])` compact spacing via SemanticsRole-based
  detection (`gap-3` instead of `gap-6`).
- `FieldSet` matches shadcn's `:has([data-slot=checkbox-group])` compact spacing by treating
  `FieldGroupSlot::CheckboxGroup` as a distinct list semantics anchor (avoid mis-detecting single
  checkbox sync-fields).
- `Field` horizontal alignment matches shadcn's `has-[>[data-slot=field-content]]:items-start` rule
  (content-driven rows align to the top instead of centering).
- `FieldLabel` matches `leading-snug` via `component.field.label_line_height`.
- `FieldDescription` matches `leading-normal` via `component.field.description_line_height`.
- `FieldDescription` expands to full width by default to match upstream wrapping behavior (`w-full`).
- `FieldDescription` negative-margin detail for “description before final sibling” is supported
  (upstream uses `nth-last-2:-mt-1`).
- `FieldGroup` supports upstream `orientation="responsive"` behavior:
  - Approximates the `@md/field-group` container query via a viewport breakpoint (`>=768px`).
  - Applies `w-auto` to direct children when in row layout; for `<input>/<textarea>`, approximates
    the browser default `cols=20` intrinsic width (so the input does not expand to the widest sibling).

## Validation

- Web layout gate:
  `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_field_input_geometry`, `web_vs_fret_layout_field_checkbox_geometry`,
  `web_vs_fret_layout_field_group_geometry`, `web_vs_fret_layout_field_fieldset_geometry`,
  `web_vs_fret_layout_field_choice_card_geometry`,
  `web_vs_fret_layout_field_switch_geometry`, `web_vs_fret_layout_field_select_geometry`,
  `web_vs_fret_layout_field_radio_geometry`, `web_vs_fret_layout_field_textarea_geometry`,
  `web_vs_fret_layout_field_responsive_orientation_places_input_beside_content`).
