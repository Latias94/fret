# Material 3 Menu And Dropdown Diagnostics Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

Menu and DropdownMenu are closed for the current Material3 sweep evidence standard.

What changed:

- Added a Material3 Menu focus/dismiss diagnostics script, redirect, and suite manifest.
- Reused the existing menu item chrome-fill script as the visual chrome gate.
- Updated the component alignment matrix for `menu` and `dropdown_menu`.
- No Material3 Menu or DropdownMenu component code changed.

Resume guidance:

- Use the focus/dismiss diagnostics script before changing DropdownMenu overlay policy integration.
- Use the chrome-fill diagnostics script before changing Menu surface/item selector or chrome
  geometry.
- Keep shared dismiss/focus policy in `fret-ui-kit` unless another concrete cross-design-system
  gap requires a new kit abstraction.
