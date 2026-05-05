# ImUi Color Edit Tooltip Preview v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Policy Surface

- `ColorEditOptions::tooltip` defaults to enabled.
- Apps can opt out per control without global color-edit state.

## M2 - Tooltip Rendering

- Hovering the root swatch drives a tooltip overlay.
- Tooltip preview reuses the shared alpha-preview stack.
- Tooltip text includes hex, RGB, and HSV lines with alpha visibility following `show_alpha`.

## M3 - Evidence and Closeout

- Focused color-edit tests lock formatter output and default policy.
- Surface-policy and adapter smoke tests anchor the new public option.
- Workstream docs name the remaining context/eyedropper gaps.
