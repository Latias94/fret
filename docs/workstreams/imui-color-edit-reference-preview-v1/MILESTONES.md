# ImUi Color Edit Reference Preview v1 Milestones

Status: Closed.

## M0 - Upstream Behavior

- Dear ImGui `ColorEdit` stores the current color before opening the picker.
- `ColorPicker4` shows `##current` and, when a reference is present, `##original`.
- Activating `##original` copies three components for no-alpha targets and four components for
  alpha-visible targets.

## M1 - Fret Policy Surface

- `ColorEditPopupSidePreview` exposes `Hidden`, `Current`, and `CurrentAndOriginal`.
- `ColorEditPopupOptions::default()` selects `CurrentAndOriginal`.
- The popup can remain useful when only the side preview row is visible.

## M2 - Product Behavior

- Popup-open captures a stable reference color.
- The preview row renders current and original cells.
- Original activation restores through the same RGB/RGBA alpha rules as Dear ImGui's component
  count behavior.

## M3 - Closeout

- Focused tests cover default policy, no-alpha preview opacity, and restore behavior.
- Workstream, roadmap, tracker, gap audit, and umbrella evidence are updated.
- Gates pass.
