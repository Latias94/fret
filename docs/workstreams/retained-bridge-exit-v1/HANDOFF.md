# Retained Bridge Exit v1 Handoff

Updated: 2026-05-18

## Current State

`RBX-M1-010` is complete. The docking retained bridge audit is recorded in:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`

The first implementation slice is now:

- `RBX-M1-020` - Extract docking split geometry and handle painting from
  `fret_ui::retained_bridge`.

## Key Finding

`fret-docking` cannot drop `fret-ui/unstable-retained-bridge` in one step. The retained bridge is
still the host substrate for:

- `DockSpace` as a retained `Widget`.
- public retained dock creation helpers.
- `imui.rs` retained subtree embedding.
- docking split layout, hit-test, drag update, and handle paint helpers.

The best first cut is the helper extraction because it is private and behavior-preserving.

## Next Task

Run `RBX-M1-020` from:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Expected implementation shape:

- Add a private split helper module under `ecosystem/fret-docking/src/dock/`.
- Migrate imports in `layout.rs`, `hit_test.rs`, `paint.rs`, and `space.rs`.
- Delete unused `retained_bridge` resizable/handle exports only after repo-wide `rg` proves no
  remaining users.

## Gates

Last run on 2026-05-18:

- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
