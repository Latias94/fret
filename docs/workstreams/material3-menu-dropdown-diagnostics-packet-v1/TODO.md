# Material 3 Menu And Dropdown Diagnostics Packet v1 - TODO

Status: Closed
Last updated: 2026-05-28

- [x] M3MENUDIAG-010 [owner=codex] [scope=tools/diag-scripts/ui-gallery/material3]
  Goal: Add a dedicated Material3 Menu focus/dismiss diagnostics script.
  Review: DONE. The script opens the Material3 Menu page, opens the default DropdownMenu, asserts
  Menu role, `.chrome`, item focus under the menu root, Escape dismissal, and trigger focus restore.

- [x] M3MENUDIAG-020 [owner=codex] [deps=M3MENUDIAG-010]
  Goal: Run focus/dismiss diagnostics, chrome diagnostics, and focused Rust gates.
  Review: DONE. Focus/dismiss diagnostics run `1779940975756` passed; chrome-fill diagnostics run
  `1779941390986` passed; automation-surface, DropdownMenu dismiss/restore, Menu pressed-scene,
  and Menu style override gates passed.

- [x] M3MENUDIAG-030 [owner=codex] [deps=M3MENUDIAG-020] [scope=docs/workstreams/material3-component-alignment-sweep-v1]
  Goal: Close Menu and DropdownMenu in the component matrix.
  Review: DONE. Both rows now record diagnostics alignment and keep the existing recipe/kit
  boundary.
