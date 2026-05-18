# Retained Bridge Exit v1 Handoff

Updated: 2026-05-18

## Current State

`RBX-M1-010`, `RBX-M1-020`, `RBX-M1-021`, `RBX-M1-030`, and `RBX-M1-040` are complete. The docking
retained bridge audit is recorded in:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md`

The first implementation slice was:

- `RBX-M1-020` - Extract docking split geometry and handle painting from
  `fret_ui::retained_bridge`.

Readiness note:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`

## Key Finding

`fret-docking` cannot drop `fret-ui/unstable-retained-bridge` in one step. The retained bridge is
still the authoring/hosting substrate for:

- `DockSpace` as a retained `Widget`.
- public retained dock creation helpers.
- `imui.rs` retained subtree embedding.
- docking host lifecycle hooks for layout, prepaint, event, command, paint, and child-root placement.

The best first cut was helper extraction because it was private and behavior-preserving. The next
cut was controller extraction, not direct retained host deletion.

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

`RBX-M1-030` found that panel content is already declarative-capable through
`DockPanelRegistry`/`render_cached_panel_root(...)`. The missing piece is the host lifecycle around
`DockSpace`: externalized controller state, child-root placement, prepaint liveness, raw event
arbitration, command/focus routing, and custom chrome/child paint ordering.

`RBX-M1-040` added `DockSpaceController` as the docking-owned cross-frame host state object and kept
the retained `DockSpace` widget as the adapter. The extraction is intentionally behavior-preserving:
methods still live on `DockSpace` for now, with a transitional `Deref` / `DerefMut` shim delegating
state field access to the controller.

## Next Task

Pick the next M1 task from:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Recommended next implementation shape:

- `RBX-M1-050`: extract layout/paint snapshots so a future declarative adapter can consume the same
  host decisions without recomputing layout in paint.
- `RBX-M1-060`: decide whether existing declarative primitives are enough or whether `fret-ui` needs
  a narrow mechanism-only managed-surface primitive.
- Do not remove `DockSpace` retained hosting until those seams exist and the declarative host
  mechanism has a proof-of-life.

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

`RBX-M1-030` audit commands:

- `python3 tools/audit_crate.py --crate fret-docking` - passed.
- `rg -n "retained_bridge|UiTreeRetainedExt|create_node_retained|RetainedSubtree|impl<.*Widget|impl Widget|Widget<" ecosystem/fret-docking/src ecosystem/fret-docking/tests -g '*.rs'`
  - found retained host/lifecycle usage in docking source and tests.
- `cargo fmt --check` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

`RBX-M1-040` gates:

- `cargo check -p fret-docking` - passed.
- `cargo fmt --check` - passed.
- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

Previous `RBX-M1-020` gates:

- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `cargo check -p fret-demo --bin docking_arbitration_demo` - passed.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
