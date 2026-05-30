# 2026-05-28 - M3DIALOGDIAG-010 through M3DIALOGDIAG-030

Opened and closed the Dialog diagnostics follow-on.

Actions:

- Added `ui-gallery-material3-dialog-focus-trap-restore.json`.
- Added a top-level redirect and suite manifest.
- Fixed an initial script mistake where clicking `Discard` closed the Dialog before the focus
  assertion; the final script uses modal/focus barrier assertions plus Escape focus restore.
- Ran the final diagnostics script.
- Queried the bundle for Material3 Dialog selectors.
- Ran focused automation-surface and `radio_alignment` Dialog tests.
- Updated the component matrix.

Evidence:

- dialog diag run: `1779939874070`
- `automation_surface` Dialog/bottom-sheet selector test passed
- `radio_alignment` Dialog focus containment/restore test passed
- `radio_alignment` Dialog scrim-dismiss underlay-blocking test passed
- `radio_alignment` Dialog style override test passed
