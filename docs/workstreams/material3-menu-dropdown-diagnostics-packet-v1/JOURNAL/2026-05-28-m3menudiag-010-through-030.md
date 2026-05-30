# 2026-05-28 - M3MENUDIAG-010 through M3MENUDIAG-030

Opened and closed the Menu/DropdownMenu diagnostics follow-on.

Actions:

- Added `ui-gallery-material3-menu-focus-dismiss.json`.
- Added a top-level redirect and suite manifest.
- Ran the focus/dismiss diagnostics script.
- Re-ran the existing menu item chrome-fill diagnostics script.
- Queried the bundles for Material3 Menu roots and item selectors.
- Ran focused automation-surface and `radio_alignment` Menu/DropdownMenu tests.
- Updated the component matrix.

Evidence:

- focus/dismiss diag run: `1779940975756`
- chrome-fill diag run: `1779941390986`
- `automation_surface` Menu/Dropdown selector test passed
- `radio_alignment` DropdownMenu dismiss/focus-restore test passed
- `radio_alignment` Menu pressed-scene test passed
- `radio_alignment` Menu style override test passed
