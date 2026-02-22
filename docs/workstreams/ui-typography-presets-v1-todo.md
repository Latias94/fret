# Workstream TODO: UI Typography Presets v1

This is a checklist-style tracker. It is **non-normative**.

## Preset surface (`fret-ui-kit`)

- [ ] Define a stable preset vocabulary for:
  - [ ] control vs content intent
  - [ ] ui vs monospace family
  - [ ] xs/sm/base/prose sizes (token-backed)
- [ ] Document when to use `BoundsAsLineBox` placement for fixed-height controls.
- [ ] Provide helpers for widgets that take `TextStyle` directly (e.g. text inputs).

## Migrations

- [ ] `fret-ui-shadcn`: replace ad-hoc `TextStyle { line_height: Some(...) }` for control text with presets/helpers.
- [ ] `fret-ui-material3`: ensure generated typography styles include stable line box policy for control surfaces.
- [ ] `fret-ui-editor` / `fret-code-view`: audit where monospace presets should be used.
- [ ] `fret-markdown`: decide per-surface default (control vs content) and document it.

## Regression gates

- [ ] Add a “first-line jump” targeted gate for control text with:
  - emoji + mixed scripts
  - same widget height across frames
  - stable metrics snapshot (line height / baseline)
- [ ] Add at least one UI Gallery screenshot gate that includes the above control.

## Cleanup

- [ ] Remove redundant local `font_line_height(theme)` helpers once presets are adopted.
- [ ] Consolidate duplicate token key lookups (prefer `fret-ui-kit` preset API).

