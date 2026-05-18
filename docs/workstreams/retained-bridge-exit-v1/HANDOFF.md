# Retained Bridge Exit v1 Handoff

Updated: 2026-05-18

## Current State

`RBX-M1-010` and `RBX-M1-020` are complete. The docking retained bridge audit is recorded in:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`

The first implementation slice was:

- `RBX-M1-020` - Extract docking split geometry and handle painting from
  `fret_ui::retained_bridge`.

Readiness note:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`

## Key Finding

`fret-docking` cannot drop `fret-ui/unstable-retained-bridge` in one step. The retained bridge is
still the host substrate for:

- `DockSpace` as a retained `Widget`.
- public retained dock creation helpers.
- `imui.rs` retained subtree embedding.
- docking split layout, hit-test, drag update, and handle paint helpers.

The best first cut is the helper extraction because it is private and behavior-preserving.

## Completed Implementation

`RBX-M1-020` added a docking-private split geometry helper and migrated `fret-docking` source/tests
off the retained bridge split helpers and `retained_bridge::ResizeHandle`.

Remaining direct split-helper bridge consumer:

- `apps/fret-examples/src/docking_arbitration_demo.rs`

## Next Task

Run `RBX-M1-021` from:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Expected implementation shape:

- Migrate `apps/fret-examples/src/docking_arbitration_demo.rs` off
  `retained_bridge::resizable_panel_group`.
- Delete the remaining bridge resizable helper module only after repo-wide `rg` proves no remaining
  users.

## Gates

Last run on 2026-05-18:

- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `cargo check -p fret-demo --bin docking_arbitration_demo` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
