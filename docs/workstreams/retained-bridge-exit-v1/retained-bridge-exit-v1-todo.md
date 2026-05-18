# Retained Bridge Exit Plan v1 — TODO Tracker

Status: Active (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1.md`

## Milestones

### M0 — Governance gates (blast radius control)

- [x] CI: reject `crates/* -> ecosystem/*` reverse dependencies (`tools/check_layering.py`).
- [x] CI: restrict `fret-ui/unstable-retained-bridge` to an explicit allowlist (`tools/check_layering.py`).
- [x] Document the current allowlist and rationale per crate (docking/node/chart/plot).
  - Source of truth: `tools/check_layering.py` (`unstable_retained_bridge_allowlist`).
  - Current allowlist (workspace crate names):
    - `fret-docking`
      - Why: hosts retained subtrees for docking UI and reuses retained helpers (e.g. resizable panel group sizing / capture / hit-test policy) while the declarative surface is still closing.
      - Evidence: `ecosystem/fret-docking/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained hosting in `ecosystem/fret-docking/src/imui.rs`.
      - Exit target: M1 (primary target).
    - `fret-node`
      - Why: node graph canvas + portal editors are still authored as retained widgets; it also exercises overlays/commands in the retained path.
      - Evidence: `ecosystem/fret-node/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained widget surface in `ecosystem/fret-node/src/ui/canvas/widget.rs`.
      - Exit target: M2.
    - `fret-chart`
      - Why: retained canvas widget used for interactive charts; still depends on retained layout/paint/event wiring.
      - Evidence: `ecosystem/fret-chart/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained canvas in `ecosystem/fret-chart/src/retained/canvas.rs`.
      - Exit target: M3.
    - `fret-plot`
      - Why: retained plotting surfaces still use `RetainedSubtreeProps` and retained canvas widgets for performance/interaction while declarative authoring migrates.
      - Evidence: `ecosystem/fret-plot/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained subtree hosting in `ecosystem/fret-plot/src/imui.rs` and retained canvas in `ecosystem/fret-plot/src/retained/canvas/mod.rs`.
      - Exit target: M3.
    - `fret-plot3d`
      - Why: retained 3D plot surface uses retained viewport-surface helpers and widget lifecycle plumbing.
      - Evidence: `ecosystem/fret-plot3d/Cargo.toml` enables `fret-ui/unstable-retained-bridge`; retained widget in `ecosystem/fret-plot3d/src/retained.rs`.
      - Exit target: M3.

### M1 — Docking declarative closure (primary target)

- [x] RBX-M1-010 Audit docking retained bridge usage and choose the first removal/migration slice.
  - Scope:
    - `ecosystem/fret-docking/Cargo.toml`
    - `ecosystem/fret-docking/src/`
    - `crates/fret-ui/src/retained_bridge.rs` only for evidence; do not widen bridge exports.
  - Goal: classify every docking use of `fret_ui::retained_bridge` / `UiTreeRetainedExt` /
    `RetainedSubtreeProps` as `delete`, `migrate`, or `keep temporarily with gate`, then pick one
    smallest behavior-preserving slice that removes or narrows retained usage.
  - Validation:
    - `cargo nextest run -p fret-docking`
    - `python3 tools/check_layering.py`
  - Evidence:
    - `docs/workstreams/bottom-up-fearless-refactor-v1/ARCHITECTURE_ISSUES_LEDGER_2026-05-18.md#fir-001---retained-bridge-blast-radius-is-still-the-clearest-compatibility-debt`
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`
  - Result:
    - `DockSpace` retained widget usage: keep temporarily with gate.
    - public retained creation helpers: migrate/delete after declarative host exists.
    - `imui.rs` retained subtree embedding: migrate after host replacement exists.
    - splitter layout/hit-test/paint helpers: migrate by extracting first.
  - Selected next slice:
    - `RBX-M1-020`
  - Handoff: do this before broad node/chart/plot migration work so the editor-grade backbone
    proves the exit strategy first.
- [x] RBX-M1-020 Extract docking split geometry and handle painting from `fret_ui::retained_bridge`.
  - Scope:
    - `ecosystem/fret-docking/src/dock/layout.rs`
    - `ecosystem/fret-docking/src/dock/hit_test.rs`
    - `ecosystem/fret-docking/src/dock/paint.rs`
    - `ecosystem/fret-docking/src/dock/space.rs`
    - new private docking helper module if needed, e.g. `ecosystem/fret-docking/src/dock/split_geometry.rs`
    - `crates/fret-ui/src/retained_bridge.rs` only to delete unused bridge exports after repo-wide proof.
  - Goal:
    - Replace docking imports of `retained_bridge::resizable_panel_group` and
      `retained_bridge::ResizeHandle` with docking-private helpers.
  - Validation:
    - `cargo nextest run -p fret-docking`
    - `python3 tools/check_layering.py`
    - `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizeHandle" ecosystem/fret-docking crates apps`
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md#first-slice-chosen`
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-18---rbx-m1-020-docking-private-split-helper-extraction`
  - Result:
    - Added docking-private split geometry in `ecosystem/fret-docking/src/dock/split_geometry.rs`.
    - Migrated `fret-docking` source/tests off `retained_bridge::resizable_panel_group` and
      `retained_bridge::ResizeHandle`.
    - Removed no-user bridge exports/functions: `retained_bridge::ResizeHandle`,
      `retained_bridge::ResizablePanelGroupStyle`,
      `retained_bridge::resizable_panel_group::drag_update_fractions`, and
      `retained_bridge::resizable_panel_group::drag_update_adjacent_fractions`.
    - Kept `retained_bridge::resizable_panel_group::compute_layout` because
      `apps/fret-examples/src/docking_arbitration_demo.rs` still consumes it; that is tracked by
      `RBX-M1-021`.
  - Decision checkpoint:
    - Proceed now with the docking-private extraction slice.
    - Do not force deletion of `retained_bridge::resizable_panel_group` in this task if
      `apps/fret-examples/src/docking_arbitration_demo.rs` still consumes it.
    - If the app diagnostics harness remains the only consumer, split that migration to
      `RBX-M1-021`.
- [x] RBX-M1-021 Migrate `docking_arbitration_demo` diagnostics geometry off retained bridge split helpers.
  - Scope:
    - `apps/fret-examples/src/docking_arbitration_demo.rs`
    - `crates/fret-ui/src/retained_bridge.rs` only after repo-wide proof of no remaining users.
  - Goal:
    - Remove the final direct app/demo dependency on `retained_bridge::resizable_panel_group`, then
      delete the bridge helper module if possible.
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`
  - Result:
    - Migrated `docking_arbitration_demo` diagnostics split geometry to local panel-rect
      computation, preserving the existing split sizing semantics needed by diagnostic anchors.
    - Deleted the remaining `fret_ui::retained_bridge::resizable_panel_group` module and
      `retained_bridge::ResizablePanelGroupLayout` re-export after repo-wide no-user proof.
    - Verified with formatting, targeted demo check, targeted demo clippy, layering, workstream
      catalog, whitespace, and retained-bridge split-helper no-match gates.
- [x] RBX-M1-030 Identify the minimal declarative primitives missing for docking.
  - Scope:
    - `ecosystem/fret-docking/src/dock/space.rs`
    - `ecosystem/fret-docking/src/dock/mod.rs`
    - `ecosystem/fret-docking/src/dock/panel_registry.rs`
    - `ecosystem/fret-docking/src/imui.rs`
    - `crates/fret-ui/src/element.rs`
    - `crates/fret-ui/src/widget.rs`
  - Goal:
    - Audit whether docking is blocked by missing panel-content declarative authoring or by a
      missing host/lifecycle primitive, then record the smallest implementation slices.
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md`
  - Result:
    - Panel content is already declarative-capable through `DockPanelRegistry` and
      `render_cached_panel_root(...)`.
    - The blocker is the retained `DockSpace` host surface: controller state, child-root placement,
      prepaint liveness, raw event arbitration, command/focus routing, and custom chrome/child paint
      ordering.
    - Next implementation slice should extract `DockSpaceController` before adding or choosing a
      declarative managed-surface primitive.
- [x] RBX-M1-040 Extract `DockSpaceController` while keeping the retained adapter.
  - Scope:
    - `ecosystem/fret-docking/src/dock/space.rs`
    - new private docking controller/state module if needed, e.g.
      `ecosystem/fret-docking/src/dock/space_controller.rs`
  - Goal:
    - Move cross-frame docking host state and practical transition helpers out of the retained
      `Widget` struct so the retained adapter and future declarative adapter can share the same
      policy engine.
  - Validation:
    - `cargo fmt --check`
    - `cargo nextest run -p fret-docking`
    - `python3 tools/check_layering.py`
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md#rbx-m1-040-extract-dockspacecontroller`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-18---rbx-m1-040-dockspacecontroller-state-extraction`
  - Result:
    - Added `DockSpaceController` as the docking-owned cross-frame host state object.
    - Kept the retained `DockSpace` widget as the adapter and delegated field access through a
      transitional `Deref` / `DerefMut` shim.
    - Preserved current behavior with the full `fret-docking` nextest gate.
- [x] RBX-M1-050 Extract docking layout/paint snapshots.
  - Scope:
    - `ecosystem/fret-docking/src/dock/space.rs`
    - `ecosystem/fret-docking/src/dock/layout.rs`
    - `ecosystem/fret-docking/src/dock/paint.rs`
    - new private snapshot/frame module if needed.
  - Goal:
    - Make layout produce a reusable host frame/snapshot consumed by paint, including active panel
      bounds, floating layouts, viewport layouts, and drop-hint paint inputs.
  - Validation:
    - `cargo nextest run -p fret-docking`
    - targeted unit tests for split, floating, viewport, and drop-hint snapshot cases
    - `python3 tools/check_layering.py`
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md#rbx-m1-050-extract-layoutpaint-snapshots`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-18---rbx-m1-050-docking-layoutpaint-snapshot-extraction`
  - Result:
    - Added a private `DockSpaceLayoutSnapshot` carrying root layout, floating layouts, merged
      layout map, active panel bounds, paint panel bounds, viewport layouts, bounds/frame identity,
      and split handle settings.
    - Exposed the snapshot and builder at `pub(super)` scope so a future declarative dock host
      adapter can reuse the same docking-frame decision object inside the `dock` module.
    - `DockSpace::layout` now builds and stores the snapshot after computing split-motion overrides.
    - `DockSpace::paint` reuses a same-frame valid snapshot and falls back to rebuilding one when
      paint runs without a matching layout snapshot.
    - Preserved retained adapter behavior with the full `fret-docking` nextest gate.
- [ ] RBX-M1-060 Decide and prove the declarative docking host mechanism.
  - Scope:
    - `crates/fret-ui/src/element.rs` / declarative host internals only if existing primitives are
      insufficient.
    - `ecosystem/fret-docking/src/` proof-of-life declarative host.
  - Goal:
    - Try existing primitives first; add a narrow mechanism-only managed-surface primitive only if
      docking still cannot express child-root placement, lifecycle liveness, raw event actions, and
      controlled child painting.
  - Validation:
    - `cargo nextest run -p fret-ui -p fret-docking`
    - `python3 tools/check_layering.py`
    - a small docking layout/diagnostics proof for declarative panel-root placement
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md#rbx-m1-060-decide-the-declarative-host-mechanism`
- [ ] Replace retained subtree hosting in docking with declarative composition where feasible.
- [ ] Add/upgrade `fretboard-dev diag` scripts to lock in docking drag + tear-off correctness.
- [ ] Remove `unstable-retained-bridge` from `ecosystem/fret-docking` dependencies.

### M2 — Node graph migration

- [ ] Split node graph into:
  - declarative composition for chrome/overlays/panels,
  - `Canvas`/`ViewportSurface`-style leaf for heavy rendering where needed.
- [ ] Remove `unstable-retained-bridge` from `ecosystem/fret-node` dependencies.

### M3 — Charts/plots migration

- [ ] Convert chart/plot surfaces to `Canvas`-first declarative authoring.
- [ ] Remove `unstable-retained-bridge` from `ecosystem/fret-chart`, `ecosystem/fret-plot`, `ecosystem/fret-plot3d`.

### M4 — Bridge shrink and delete (or quarantine)

- [ ] Audit `crates/fret-ui/src/retained_bridge.rs` exports; delete anything not required by remaining clients.
- [ ] If allowlist becomes empty: remove `fret-ui/unstable-retained-bridge` feature and all bridge code.
- [ ] Otherwise: quarantine the remaining retained path behind a narrower, clearly named compatibility facade with
  explicit “do not grow” policy and separate tracking.
