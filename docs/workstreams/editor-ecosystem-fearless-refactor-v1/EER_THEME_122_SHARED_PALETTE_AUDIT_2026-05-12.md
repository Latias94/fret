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
- `editor.popup.*` owns popup surface background, border, radius, and shadow metrics.
- `editor.chrome.*` owns muted foreground, accent, and ring intent for shared editor chrome.
- `primitives::colors` centralizes panel/header/popup fallback order so composite surfaces do not
  each carry their own generic-token ladder.

However, removing all shared palette writes from the preset is not correct yet. Several real editor
surfaces still read generic palette keys directly:

- `controls/checkbox.rs` still uses generic `primary`, `primary-foreground`, and background/input
  fallbacks for checked and focused state.
- `controls/slider.rs` still uses generic `muted`, `primary`, and `background` for track, fill, and
  thumb colors.
- `controls/color_edit/popup/*` still uses generic `border`, `primary`, `foreground`, `popover`,
  and `popover-foreground` across picker, preview, swatch, and tooltip chrome.
- `primitives/chrome.rs` still keeps generic fallbacks such as `input`, `ring`,
  `foreground`, and `selection.background` after editor-owned text-field keys.
- `primitives/popup_surface.rs` still uses generic `muted` for the shadow color.

That means the shared writes in `editor_theme_patch_v1` and `imgui_like_dense_patch_v1` are still
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

1. Add editor-owned checkbox color tokens and update `controls/checkbox.rs` to prefer them before
   generic `primary` / `primary-foreground`.
2. Add editor-owned slider color tokens and update `controls/slider.rs` to prefer them before
   generic `muted` / `primary` / `background`.
3. Route color-edit popup picker, preview, swatch, and tooltip chrome through editor-owned popup or
   color token families before removing `popover`, `border`, `foreground`, or `primary` preset
   writes.
4. After those seams move, split the preset into an editor-owned patch plus an optional proof/demo
   host-palette patch, or shrink the shared palette writes directly if no proof surface still needs
   the app-wide colors.

## Gates

- `cargo nextest run -p fret-ui-editor`
- Focused screenshot proof for the seam being moved, for example:
  - `tools/diag-scripts/ui-editor/imui/imui-editor-proof-editor-components-screenshots-default.json`
  - `tools/diag-scripts/ui-editor/imui/imui-editor-proof-gradient-stop-color-popup-screenshots.json`
- `python tools/check_workstream_catalog.py`
