# Workstream TODO: UI Typography Presets v1

This is a checklist-style tracker. It is **non-normative**.

## Preset surface (`fret-ui-kit`)

- [x] Define a stable preset vocabulary for:
  - [x] control vs content intent
  - [x] ui vs monospace family
  - [x] xs/sm/base/prose sizes (token-backed)
- [x] Document when to use `BoundsAsLineBox` placement for fixed-height controls.
- [x] Provide helpers for widgets that take `TextStyle` directly (e.g. text inputs).
- [x] Add an intent-first entry point (e.g. `TypographyPreset` / `TextIntent`) so components can
      declare “control vs content” without manually composing `TextStyle`.

## Migrations

- [x] `fret-ui-shadcn`: migrate core control text (inputs, menus, tabs, sidebars, tables) to stable
      fixed line boxes via `fret-ui-kit` helpers.
- [x] `fret-ui-shadcn`: finish remaining ad-hoc control text literals (e.g. `button_group.rs`) and
      remove redundant local helpers where feasible.
- [ ] `fret-ui-shadcn`: audit remaining builder-chain control text callsites that set
      `text_size_px` + `line_height_px` + `FixedFromStyle`, and migrate to intent/preset helpers
      (or add `BoundsAsLineBox` placement for fixed-height controls).
- [x] `fret-ui-kit`: migrate ad-hoc control text in primitives/overlays (e.g. labels, text fields,
      toasts) to the intent-first stability defaults.
- [x] `fret-ui-material3`: ensure generated typography styles include stable line box policy for control surfaces (plus regression gates).
- [ ] `fret-ui-editor` / `fret-code-view`: audit where monospace presets should be used.
- [ ] `fret-markdown`: decide per-surface default (control vs content) and document it.

## Regression gates

- [x] Add a “first-line jump” targeted gate for control text with **real shaping** (Parley + bundled
      fonts), not a fake `TextService`, covering:
  - emoji + mixed scripts
  - same widget height across frames
  - stable metrics snapshot (line height / baseline)
  - Evidence: `ecosystem/fret-ui-kit/tests/typography_real_shaping.rs`
- [x] Add at least one Material 3 regression gate that exercises real shaping with bundled fonts.
  - Evidence: `ecosystem/fret-ui-material3/src/lib.rs`
- [ ] Add at least one UI Gallery screenshot gate that includes the above control.

## Cleanup

- [ ] Remove redundant local `font_line_height(theme)` helpers once presets are adopted.
- [ ] Consolidate duplicate token key lookups (prefer `fret-ui-kit` preset API).
