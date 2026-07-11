# EER-THEME-122 Shared Palette Audit

Status: Current audit; not closed
Date: 2026-05-12

## Scope

This note audits the remaining shared palette writes in
`ecosystem/fret-ui-editor/src/theme.rs` for `EER-THEME-122`.

The goal is not to remove every generic token fallback from editor widgets. Generic fallbacks remain
valid for app themes. The narrower goal is to decide whether the editor proof preset can stop
mutating shared app palette keys now that many visible editor seams have editor-owned tokens.

## Current Findings

`fret-ui-editor` is already in the right ownership shape for the main inspector/editor chrome:

- `editor.text_field.*` now owns text-field chrome and the preset no longer writes
  `component.text_field.*`.
- `editor.property.*` owns panel, group, header, and property-row frame colors and metrics.
- `editor.popup.*` owns popup surface background, border, radius, shadow metrics, and shadow color.
- `editor.chrome.*` owns muted foreground, accent, and ring intent for shared editor chrome.
- `primitives::colors` centralizes panel/header/popup fallback order so composite surfaces do not
  each carry their own generic-token ladder.

However, removing all shared palette writes from the preset is not correct yet. Several real editor
surfaces still read generic palette keys directly:

- `controls/checkbox.rs` now has editor-owned `editor.checkbox.*` colors for unchecked, checked,
  and ring state, with legacy `component.checkbox.*` / `component.input.*` / generic palette
  fallbacks still retained for compatibility.
- `controls/slider.rs` now has editor-owned `editor.slider.*` colors for track, fill, thumb, and
  thumb border, with legacy `component.slider.*` / generic palette fallbacks still retained for
  compatibility.
- `controls/color_edit/popup/*` now routes picker, preview, swatch, and tooltip chrome through
  editor-owned popup / chrome helpers instead of reading `border`, `primary`, `foreground`,
  `popover`, or `popover-foreground` directly.
- `primitives/chrome.rs` still keeps generic fallbacks such as `input`, `foreground`, and
  `selection.background` after editor-owned text-field keys; text-area focus and preedit ring now
  route through the editor focus-ring helper before host palette fallback.
- `primitives/popup_surface.rs` now uses `editor.popup.shadow.color` before the generic `muted`
  compatibility fallback.

That means the shared writes in `editor_theme_patch` and `imgui_like_dense_patch` are still
serving two different jobs:

1. editor-owned token seeding for reusable editor widgets, and
2. proof/demo app palette seeding for editor surfaces that have not yet moved to editor-owned color
   families.

Those two jobs should be split before calling the preset surface stable.

## Decision

Do not delete the residual shared palette writes in one batch.

Keep the current shared palette patch as proof/demo compatibility until the remaining direct
generic readers have editor-owned token lanes. The next cleanup should move one visible seam at a
time, with screenshot or unit evidence, then shrink the shared-write whitelist.

Also keep the existing invariant that editor presets must not write legacy `component.text_field.*`
keys. Those are compatibility read fallbacks only.

## Narrow Follow-Up Slices

1. Checkbox, slider, color-edit popup, popup-shadow, and text-area ring seams landed; next audit
   whether the remaining shared palette writes can shrink further once the primitive chrome helpers
   stop needing their current compatibility fallbacks.
2. After that seam moves, split the preset into an editor-owned patch plus an optional proof/demo
   host-palette patch, or shrink the shared palette writes directly if no proof surface still needs
   the app-wide colors.

## Gates

- `cargo nextest run -p fret-ui-editor`
- Focused screenshot proof for the seam being moved, for example:
  - `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`
  - `tools/diag-scripts/ui-editor/imui/imui-editor-proof-gradient-stop-color-popup-screenshots.json`
- `python tools/check_workstream_catalog.py`
