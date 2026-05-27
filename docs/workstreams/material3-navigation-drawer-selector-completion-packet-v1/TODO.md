# TODO - Material 3 NavigationDrawer Selector Completion Packet v1

## Task Ledger

### M3NDS-010 - Open selector-completion packet

Status: Done
Owner: Codex
Scope:

- Record source truth, scope, non-goals, and layer ownership.

Validation:

- Workstream docs exist and name the executable selector slice.

### M3NDS-020 - Complete NavigationDrawer part selectors

Status: Done
Owner: Codex
Depends on: M3NDS-010
Scope:

- Add drawer root `.chrome`.
- Add item `.icon`, `.label`, and optional `.badge`.
- Preserve root item and `.chrome` ids.
- Prove standard and modal drawer content selectors in automation.

Validation:

- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_navigation_drawer_exposes_stable_part_test_ids material3_modal_navigation_drawer_exposes_stable_part_test_ids`

### M3NDS-030 - Update matrix and evidence

Status: Done
Owner: Codex
Depends on: M3NDS-020
Scope:

- Update `component_alignment_matrix_v1.json`.
- Record the packet artifact and residual follow-ons.

Validation:

- `python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json > $null`
- `python tools/check_workstream_catalog.py`

### M3NDS-040 - Verify and close

Status: Done
Owner: Codex
Depends on: M3NDS-030
Scope:

- Run format, focused automation, crate check, clippy, JSON, catalog, and whitespace gates.
- Close the lane only if all gates pass.

Validation:

- `cargo fmt --package fret-ui-material3 -- --check`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- `git diff --check`
