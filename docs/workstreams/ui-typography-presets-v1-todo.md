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
- [x] `fret-ui-shadcn`: migrate key builder-chain control text callsites (menus, buttons, kbd/select)
      to stable fixed line boxes (`fixed_line_box_px(..)` + `line_box_in_bounds()`).
- [ ] `fret-ui-shadcn`: continue the builder-chain audit across the remaining surfaces (command,
      breadcrumbs, avatar, dialogs, calendars) and remove redundant `FixedFromStyle` callsites.
- [x] `fret-ui-kit`: migrate ad-hoc control text in primitives/overlays (e.g. labels, text fields,
      toasts) to the intent-first stability defaults.
- [x] `fret-ui-material3`: ensure generated typography styles include stable line box policy for control surfaces (plus regression gates).
- [x] `fret-code-view`: audit where monospace presets should be used.
  - Evidence: `ecosystem/fret-code-view/src/code_block.rs`, `ecosystem/fret-code-view/src/copy_button.rs`
- [x] `fret-ui-editor`: audit where monospace presets should be used.
  - Evidence: `ecosystem/fret-ui-editor/src/primitives/chrome.rs`, `ecosystem/fret-ui-editor/src/controls/text_field.rs`
- [x] `fret-markdown`: decide per-surface default (control vs content) and document it.
  - Evidence: `ecosystem/fret-markdown/src/lib.rs`, `ecosystem/fret-markdown/src/pulldown_render.rs`

## Regression gates

- [x] Add a “first-line jump” targeted gate for control text with **real shaping** (Parley + bundled
      fonts), not a fake `TextService`, covering:
  - emoji + mixed scripts
  - same widget height across frames
  - stable metrics snapshot (line height / baseline)
  - Evidence: `ecosystem/fret-ui-kit/tests/typography_real_shaping.rs`
- [x] Add at least one Material 3 regression gate that exercises real shaping with bundled fonts.
  - Evidence: `ecosystem/fret-ui-material3/src/lib.rs`
- [x] Add at least one UI Gallery screenshot gate that includes the above control.
  - Evidence: `tools/diag-scripts/ui-gallery-text-mixed-script-fallback-screenshot.json`

## Cleanup

- [ ] Remove redundant local `font_line_height(theme)` helpers once presets are adopted.
- [ ] Consolidate duplicate token key lookups (prefer `fret-ui-kit` preset API).
