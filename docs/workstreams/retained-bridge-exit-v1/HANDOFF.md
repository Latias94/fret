# Retained Bridge Exit v1 Handoff

Updated: 2026-05-18

## Current State

`RBX-M1-010`, `RBX-M1-020`, and `RBX-M1-021` are complete. The docking retained bridge audit is
recorded in:

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

`RBX-M1-021` migrated `apps/fret-examples/src/docking_arbitration_demo.rs` diagnostics split
geometry off `fret_ui::retained_bridge::resizable_panel_group` and deleted the remaining retained
bridge resizable helper module/export after repo-wide no-user proof.

There are no remaining Rust source users of:

- `retained_bridge::resizable_panel_group`
- `retained_bridge::ResizablePanelGroupLayout`
- `resizable::compute_layout`

## Next Task

Pick the next M1 task from:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Recommended next implementation shape:

- Identify the minimal declarative primitives still missing for docking.
- Keep the task narrow: do not remove `DockSpace` retained hosting until the missing primitive list
  is explicit and backed by code evidence.

## Gates

Last run on 2026-05-18:

- `cargo fmt --check` - passed.
- `cargo check -p fret-demo --bin docking_arbitration_demo` - passed.
- `cargo clippy -p fret-demo --bin docking_arbitration_demo --no-deps -- -D warnings` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizablePanelGroupLayout|resizable::compute_layout" crates ecosystem apps -g '*.rs'`
  - no matches.

Previous `RBX-M1-020` gates:

- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `cargo check -p fret-demo --bin docking_arbitration_demo` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
