# RBX-M1-010 Docking Retained Bridge Audit

Date: 2026-05-18
Status: Done
Workstream: `retained-bridge-exit-v1`
Task: `RBX-M1-010`

## Purpose

Audit every `fret-docking` use of `fret_ui::retained_bridge`, classify the usage, and choose the
first narrow removal or migration slice. This is an evidence pass only; implementation belongs in a
follow-on task.

## Snapshot

`fret-docking` still enables `fret-ui/unstable-retained-bridge`:

- `ecosystem/fret-docking/Cargo.toml`

This is allowed only by the explicit guard in `tools/check_layering.py`:

- `_check_unstable_retained_bridge_allowlist(...)`
- `unstable_retained_bridge_allowlist = {"fret-chart", "fret-docking", "fret-node", "fret-plot", "fret-plot3d"}`

The dependency is not accidental. Docking still uses the bridge as its primary retained authoring
substrate.

## Classification

### Keep Temporarily With Gate

1. `DockSpace` retained widget
   - Evidence:
     - `ecosystem/fret-docking/src/dock/space.rs:2565` implements `Widget<H> for DockSpace`.
     - `ecosystem/fret-docking/src/dock/prelude_ui.rs:4` re-exports retained widget context types.
   - Why keep for now:
     - This is the current editor-grade docking host. It owns layout, tab interaction, drag routing,
       focus commands, panel child layout, painting, diagnostics, and viewport forwarding.
     - Removing it before a declarative host exists would collapse the backbone of docking demos and
       diagnostics.
   - Gate:
     - Keep the allowlist in `tools/check_layering.py` narrow.
     - Do not add new bridge exports for docking unless the export has a delete plan.

2. Docking tests that create retained nodes directly
   - Evidence:
     - `ecosystem/fret-docking/src/dock/tests/**` uses `create_node_retained(DockSpace::new(...))`
       extensively.
   - Why keep for now:
     - The tests currently verify the retained host's real behavior and are useful while migration is
       staged.
   - Exit:
     - Convert tests to the declarative docking host once that host exists.

### Migrate

1. Public retained creation helpers
   - Evidence:
     - `ecosystem/fret-docking/src/dock/mod.rs:202` `create_dock_space_node(...)`
     - `ecosystem/fret-docking/src/dock/mod.rs:214` `create_dock_space_node_with_test_id(...)`
     - `ecosystem/fret-docking/src/dock/mod.rs:233` `mount_dock_space(...)`
     - `ecosystem/fret-docking/src/dock/mod.rs:246` `mount_dock_space_with_test_id(...)`
     - `ecosystem/fret-docking/src/lib.rs:16` re-exports the helpers.
   - Why migrate:
     - These helpers make retained authoring look like a supported public integration path.
     - First-party callers can be moved when a declarative docking host API exists.
   - Exit:
     - Introduce a declarative docking host helper, migrate first-party apps/examples, then delete
       these helpers instead of preserving compatibility aliases.

2. imui retained subtree embedding
   - Evidence:
     - `ecosystem/fret-docking/src/imui.rs:72` creates `RetainedSubtreeProps`.
     - `ecosystem/fret-docking/src/imui.rs:74` creates a `RetainedSubtreeFactory`.
     - `ecosystem/fret-docking/src/imui.rs:109` creates the retained `DockSpace`.
     - `ecosystem/fret-docking/src/imui.rs:112` creates retained `DockHostRoot`.
   - Why migrate:
     - This is the bridge from the declarative/immediate authoring surface back into retained widget
       authoring.
     - `DockHostRoot` exists mainly to perform per-frame configure, panel binding, child layout, and
       paint forwarding.
   - Exit:
     - Replace with a declarative docking host element or recipe once the host can perform the same
       configure/bind/layout responsibilities without `RetainedSubtreeProps`.

3. Diagnostics-only tab drag anchor
   - Evidence:
     - `ecosystem/fret-docking/src/imui.rs:110` optionally creates `DockTabDragAnchor`.
     - `ecosystem/fret-docking/src/imui.rs:129` implements `Widget<H> for DockTabDragAnchor`.
   - Why migrate/delete:
     - It is a semantics-only retained widget for scripted diagnostics.
   - Exit:
     - Prefer a declarative semantics/test-id anchor or remove it if docking diagnostics can target a
       stable real tab node.

### Migrate By Extraction

1. Split layout, hit-test, drag, and handle painting helpers
   - Evidence:
     - `ecosystem/fret-docking/src/dock/layout.rs:6` imports
       `retained_bridge::resizable_panel_group`.
     - `ecosystem/fret-docking/src/dock/hit_test.rs:11` imports
       `retained_bridge::resizable_panel_group`.
     - `ecosystem/fret-docking/src/dock/paint.rs:16` imports `retained_bridge::ResizeHandle`.
     - `ecosystem/fret-docking/src/dock/paint.rs:17` imports
       `retained_bridge::resizable_panel_group`.
     - `ecosystem/fret-docking/src/dock/space.rs:36` imports
       `retained_bridge::resizable_panel_group`.
     - `ecosystem/fret-docking/src/dock/space.rs:5392` uses
       `drag_update_adjacent_fractions(...)`.
   - Why migrate by extraction first:
     - This code is private docking split geometry and scene emission, not retained widget lifecycle.
     - It can move out of `retained_bridge` without replacing `DockSpace`.
     - It reduces bridge surface area before the harder declarative host migration.
   - Exit:
     - Add a docking-private split helper module and migrate the call sites.
     - If no other retained bridge client uses `retained_bridge::resizable_panel_group` or
       `retained_bridge::ResizeHandle`, delete those bridge exports.

### Delete After Replacement

No retained bridge use is safe to delete immediately in `RBX-M1-010`, because this task is an audit
slice and `DockSpace` is still the active docking host. The delete candidates are:

1. Public retained creation helpers
   - Evidence:
     - `ecosystem/fret-docking/src/dock/mod.rs:202`
     - `ecosystem/fret-docking/src/dock/mod.rs:214`
     - `ecosystem/fret-docking/src/dock/mod.rs:233`
     - `ecosystem/fret-docking/src/dock/mod.rs:246`
   - Delete condition:
     - A declarative docking host exists and first-party callers have migrated.

2. Diagnostics-only tab drag anchor
   - Evidence:
     - `ecosystem/fret-docking/src/imui.rs:110`
     - `ecosystem/fret-docking/src/imui.rs:129`
   - Delete condition:
     - Diagnostics can target a stable real tab node or a declarative semantics anchor.

3. Bridge resizable exports
   - Evidence:
     - `crates/fret-ui/src/retained_bridge.rs:11`
     - `crates/fret-ui/src/retained_bridge.rs:97`
   - Delete condition:
     - `RBX-M1-020` migrates docking off these helpers and repository-wide `rg` proves there are no
       remaining users.

## First Slice Chosen

`RBX-M1-020`: Extract docking split geometry and handle painting from
`fret_ui::retained_bridge`.

Proposed scope:

- Add a private docking module, likely `ecosystem/fret-docking/src/dock/split_geometry.rs`.
- Move or wrap the needed behavior currently reached through
  `retained_bridge::resizable_panel_group`:
  - `compute_layout(...)`
  - `drag_update_adjacent_fractions(...)`
- Move docking handle paint/hit geometry away from `retained_bridge::ResizeHandle`.
- Update these docking call sites:
  - `ecosystem/fret-docking/src/dock/layout.rs`
  - `ecosystem/fret-docking/src/dock/hit_test.rs`
  - `ecosystem/fret-docking/src/dock/paint.rs`
  - `ecosystem/fret-docking/src/dock/space.rs`
- Then remove unused bridge exports if repository-wide `rg` proves they have no remaining users.

Why this is the right first cut:

- It is behavior-preserving and private to docking.
- It does not require a new declarative dock host.
- It narrows the unstable bridge before touching public app/example integration.
- It gives a concrete gate for the retained bridge shrink direction.

## Not Chosen First

1. Delete public dock creation helpers
   - Reason:
     - First-party apps/examples still call them directly.
     - Deleting them first would force a larger integration migration without the declarative host
       replacement being ready.

2. Replace `RetainedSubtreeProps` in `imui.rs`
   - Reason:
     - This needs a real declarative docking host or equivalent mechanism for per-frame configure,
       panel binding, layout forwarding, and paint forwarding.
     - It is the right follow-up after the bridge helper surface is smaller.

3. Rewrite `DockSpace` as declarative immediately
   - Reason:
     - `DockSpace` is currently the main docking behavior owner and is large enough to need staged
       extraction, diagnostics, and test gates.

## Evidence Commands

Commands used for the audit:

- `rg -n "retained_bridge|UiTreeRetainedExt|RetainedSubtreeProps|RetainedSubtreeFactory|Widget<|ResizeHandle|CommandCx|EventCx|LayoutCx|PaintCx|PrepaintCx|SemanticsCx" ecosystem/fret-docking/src ecosystem/fret-docking/Cargo.toml`
- `rg -n "resizable::|ResizeHandle|UiTreeRetainedExt|RetainedSubtreeProps|Widget<|create_node_retained" ecosystem/fret-docking/src/dock/space.rs ecosystem/fret-docking/src/dock/layout.rs ecosystem/fret-docking/src/dock/hit_test.rs ecosystem/fret-docking/src/dock/paint.rs ecosystem/fret-docking/src/imui.rs ecosystem/fret-docking/src/dock/mod.rs`
- `rg -n "RetainedSubtreeProps|dock_space_with|create_dock_space_node_with_test_id|create_dock_space_node\\(" apps ecosystem crates -g '*.rs'`
- `python3 tools/audit_crate.py --crate fret-docking`

Validation run on 2026-05-18:

- `cargo nextest run -p fret-docking` - passed, 111 tests.
- `python3 tools/check_layering.py` - passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.
