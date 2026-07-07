# shadcn/ui v4 Audit - Radio Group

## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- shadcn/ui: https://github.com/shadcn-ui/ui

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
This audit compares Fret's shadcn-aligned `RadioGroup` against the current upstream shadcn/ui v4
docs page, the `new-york-v4` registry source/examples, and the existing radio-group gates.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/radio-group.mdx`
- Component implementation: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/radio-group.tsx`
- Example compositions: `repo-ref/ui/apps/v4/registry/new-york-v4/examples/radio-group-demo.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/field-radio.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/form-rhf-radiogroup.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/form-tanstack-radiogroup.tsx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/dropdown-menu-radio-group.tsx`
- Upstream goldens: `goldens/shadcn-web/v4/new-york-v4/radio-group-demo.json`, `goldens/shadcn-web/v4/new-york-v4/radio-group-demo.focus.json`, `goldens/shadcn-web/v4/new-york-v4/field-radio.json`, `goldens/shadcn-web/v4/new-york-v4/form-rhf-radiogroup.json`, `goldens/shadcn-web/v4/new-york-v4/form-tanstack-radiogroup.json`, `goldens/shadcn-web/v4/new-york-v4/dropdown-menu-radio-group*.json`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/radio_group.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/radio_group.rs`

## Audit checklist

### Authoring surface

- Pass: `RadioGroup::uncontrolled(default)` and `RadioGroup::new(model)` remain the compact quick-start helpers for uncontrolled and controlled authoring.
- Pass: `RadioGroup::into_element_parts(...)` now covers the direct docs-parity row-composition lane for external `Field`, `Label`, `FieldLabel::for_control(...)`, and `FieldDescription` composition around the radio control.
- Pass: no extra generic root `compose()` / `children(...)` API is needed here because `into_element_parts(...)` keeps the row-composition seam typed while preserving roving-order ownership on the group.
- Pass: `RadioGroupItem::children(...)` and `variant(RadioGroupItemVariant::ChoiceCard)` remain valid Fret shorthands for full-row override and recipe-owned chrome, but they are no longer the only path for description / invalid / RTL row parity.
- Pass: `control_id(...)` remains the focused Fret bridge for label-forwarding and item-specific label association.

### Interaction & default-style ownership

- Pass: selection semantics, roving navigation, icon chrome, border, and focus ring remain recipe-owned.
- Pass: surrounding fieldset, card width, and row layout remain caller-owned composition.
- Pass: existing radio-group layout, control chrome, focus ring, primitive state, and dropdown-menu radio composition gates continue to cover representative interaction and geometry outcomes.

### Gallery / docs parity

- Pass: the gallery mirrors the current upstream Radio Group docs path first: `Demo` and `Usage`.
- Pass: `Description`, `Choice Card`, `Fieldset`, `Disabled`, `Required Disabled`, `Invalid`, `RTL`, `API Reference`, and `Label Association` remain explicit Fret follow-ups that document related Field/Form/RTL/association composition without pretending they are current upstream Radio Group headings.
- Pass: the composed rows use `into_element_parts(...)` for source-shaped composition instead of forcing richer rows through the item-owned child lane.
- Pass: the dedicated gallery docs-surface gate now locks the page order, snippet lane split, and diagnostic evidence anchors in `apps/fret-ui-gallery/tests/radio_group_docs_surface.rs`.
- Pass: this work remains docs/public-surface parity, not a mechanism-layer fix.

## Validation

- `cargo nextest run -p fret-ui-gallery --test radio_group_docs_surface`
- `cargo nextest run -p fret-ui-shadcn --lib --status-level fail radio_group`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_layout --status-level fail radio_group`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_control_chrome --status-level fail radio_group_demo`
- `cargo nextest run -p fret-ui-shadcn --features web-goldens --test web_vs_fret_overlay_placement --status-level fail dropdown_menu_radio_group`
- `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --status-level fail radio_group`
- `Get-ChildItem -Path tools/diag-scripts/ui-gallery/radio-group -Filter *.json | ForEach-Object { python -m json.tool $_.FullName | Out-Null }`
