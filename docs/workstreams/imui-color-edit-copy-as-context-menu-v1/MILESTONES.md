# ImUi Color Edit Copy-As Context Menu v1 Milestones

Status: Closed
Last updated: 2026-05-05

## M1 - Policy Surface

- `ColorEditOptions::copy` defaults to enabled.
- Apps can opt out per control without global color-edit state.

## M2 - Context Menu And Clipboard

- The root swatch can open a copy-as context menu via right-click or keyboard context-menu shortcuts.
- Menu selection writes text through `Effect::ClipboardWriteText`.
- The menu closes after activation and keeps focus restoration local to the swatch.

## M3 - Evidence and Closeout

- Focused color-edit tests lock formatter output and default policy.
- Surface-policy and adapter smoke tests anchor the new public option.
- Workstream docs name the remaining eyedropper and picker-polish gaps.
