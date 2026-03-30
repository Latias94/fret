# shadcn/ui v4 Audit — Field


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret’s shadcn-aligned Field primitives against the upstream shadcn/ui v4 base
docs and example implementations in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs pages: `repo-ref/ui/apps/v4/content/docs/components/base/field.mdx`, `repo-ref/ui/apps/v4/content/docs/components/radix/field.mdx`
- shadcn component implementations: `repo-ref/ui/apps/v4/registry/bases/base/ui/field.tsx`, `repo-ref/ui/apps/v4/registry/bases/radix/ui/field.tsx`
- shadcn examples: `repo-ref/ui/apps/v4/examples/base/field-demo.tsx`, `repo-ref/ui/apps/v4/examples/base/field-input.tsx`, `repo-ref/ui/apps/v4/examples/base/field-textarea.tsx`, `repo-ref/ui/apps/v4/examples/base/field-select.tsx`, `repo-ref/ui/apps/v4/examples/base/field-slider.tsx`, `repo-ref/ui/apps/v4/examples/base/field-fieldset.tsx`, `repo-ref/ui/apps/v4/examples/base/field-checkbox.tsx`, `repo-ref/ui/apps/v4/examples/base/field-radio.tsx`, `repo-ref/ui/apps/v4/examples/base/field-switch.tsx`, `repo-ref/ui/apps/v4/examples/base/field-choice-card.tsx`, `repo-ref/ui/apps/v4/examples/base/field-group.tsx`, `repo-ref/ui/apps/v4/examples/base/field-rtl.tsx`, `repo-ref/ui/apps/v4/examples/base/field-responsive.tsx`
- Default visual-chrome examples: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-{demo,input,textarea,select,slider,fieldset,checkbox,radio,switch,choice-card,group,rtl,responsive}.tsx`
- Headless mechanism reference: `repo-ref/base-ui/packages/react/src/field/index.parts.ts`, `repo-ref/base-ui/packages/react/src/field/root/FieldRoot.tsx`
- Radix primitives note: `repo-ref/primitives` does not ship a standalone generic `Field` primitive, so the mechanism cross-check for Field-specific association/validation ownership is primarily Base UI rather than Radix.

## Fret implementation

- Components: `ecosystem/fret-ui-shadcn/src/field.rs`
- Theme tokens: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`

## Audit checklist

### Authoring surface

- Pass: `Field`, `FieldSet`, `FieldGroup`, `FieldLegend`, `FieldContent`, `FieldLabel`, `FieldTitle`, `FieldDescription`, `FieldSeparator`, and `FieldError` cover the documented base Field family.
- Pass: `FieldLabel::for_control(...)` and `FieldLabel::wrap(...)` cover the upstream `htmlFor` and choice-card label-wrapping paths without requiring a generic `compose()` API.
- Pass: `Field::new([...])` stays the eager wrapper lane, while `Field::build(...)` is the focused field-local association lane for controls that benefit from shared label/description context.
- Pass: the current public surface already matches the main upstream authoring outcomes; no `fret-ui` mechanism-layer expansion or generic root `children(...)` / `compose()` / `asChild` widening is indicated here.

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
- Plain `FieldLabel` and `FieldTitle` approximate upstream `w-fit` defaults by keeping width `Auto` and opting out of cross-axis stretch unless callers explicitly request width.
- `FieldDescription` matches `leading-normal` via `component.field.description_line_height`.
- `FieldDescription` expands to full width by default to match upstream wrapping behavior (`w-full`).
- `FieldDescription` negative-margin detail for “description before final sibling” is supported
  (upstream uses `nth-last-2:-mt-1`).
- `FieldGroup` supports upstream `orientation="responsive"` behavior:
  - Approximates the `@md/field-group` container query via a viewport breakpoint (`>=768px`).
  - Applies `w-auto` to direct children when in row layout; for `<input>/<textarea>`, approximates
    the browser default `cols=20` intrinsic width (so the input does not expand to the widest sibling).

### Gallery / docs parity

- Pass: the gallery now keeps the full upstream base/radix Field docs path intact through `API Reference`: `Usage`, `Anatomy`, `Form`, the example set through `Field Group`, `RTL`, `Responsive Layout`, `Validation and Errors`, `Accessibility`, and `API Reference`.
- Pass: `Usage` and `Anatomy` are now real snippet-backed examples instead of page-local hand-written Rust strings, so the default docs lane is copyable and compiled.
- Pass: `Composable Children` remains available as a focused Fret follow-up after `API Reference`, which keeps the upstream page order reviewable while still documenting the `FieldLabel::wrap(...)` lane explicitly.
- Pass: the previous docs drift was mostly page/public-surface parity and source-of-truth staleness (`new-york-v4` references), not a recipe default-style bug.
- Pass: ownership notes now stay explicit: `FieldDescription` remains recipe-owned full-width wrapping, while plain `FieldLabel` / `FieldTitle` remain intrinsic-width by default unless the enclosing `Field` orientation or call site expands them.

## Validation

- Gallery compile: `CARGO_TARGET_DIR=target-codex-field cargo check -p fret-ui-gallery --message-format short`
- Docs-surface gate: `apps/fret-ui-gallery/tests/field_docs_surface.rs`
- Source-policy gate: `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs` (`field_snippets_prefer_ui_cx_on_the_default_app_surface`, `field_page_usage_prefers_field_wrapper_family`)
- Gallery docs smoke: `tools/diag-scripts/ui-gallery/field/ui-gallery-field-docs-smoke.json`
- Web layout gate:
  `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_field_input_geometry`, `web_vs_fret_layout_field_checkbox_geometry`,
  `web_vs_fret_layout_field_group_geometry`, `web_vs_fret_layout_field_fieldset_geometry`,
  `web_vs_fret_layout_field_choice_card_geometry`,
  `web_vs_fret_layout_field_switch_geometry`, `web_vs_fret_layout_field_select_geometry`,
  `web_vs_fret_layout_field_radio_geometry`, `web_vs_fret_layout_field_textarea_geometry`,
  `web_vs_fret_layout_field_responsive_orientation_places_input_beside_content`).
