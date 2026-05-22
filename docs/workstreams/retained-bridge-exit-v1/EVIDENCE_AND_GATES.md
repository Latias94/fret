# Retained Bridge Exit v1 Evidence and Gates

## 2026-05-18 - RBX-M1-010 Docking retained bridge audit

Claim verified:

- `fret-docking` retained bridge usage has been audited and classified.
- The first implementation slice has been selected as `RBX-M1-020`: extract docking split geometry
  and handle painting from `fret_ui::retained_bridge`.

Evidence:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_010_DOCKING_RETAINED_BRIDGE_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: existing docking behavior remains green after the audit/documentation update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: retained bridge allowlist and crate layering still pass.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed documentation has no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M1-010` is an audit/documentation task; the task-local docking gate and layering
    gate are sufficient.

## 2026-05-18 - RBX-M1-020 readiness checkpoint

Claim recorded:

- Retained bridge deletion remains the long-term target because declarative authoring is the
  primary Fret UI direction.
- `RBX-M1-020` should proceed as a docking-private extraction slice.
- Full deletion of `retained_bridge::resizable_panel_group` should wait if
  `apps/fret-examples/src/docking_arbitration_demo.rs` remains a consumer.

Evidence:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_020_READINESS_NOTE_2026-05-18.md`

Commands:

- `rg -n "resizable::|retained_bridge::resizable_panel_group|retained_bridge::ResizeHandle|ResizeHandle" apps/fret-examples/src/docking_arbitration_demo.rs ecosystem/fret-docking/src crates/fret-ui/src/retained_bridge.rs`
  - Result: found docking call sites and direct app/demo `resizable_panel_group` call sites.
  - Scope proven: deleting the bridge resizable helper in `RBX-M1-020` would widen the slice beyond
    `fret-docking`.

## 2026-05-18 - RBX-M1-020 docking-private split helper extraction

Claim verified:

- `fret-docking` no longer imports split geometry or handle painting through
  `fret_ui::retained_bridge`.
- No-user bridge exports/functions from this slice were deleted.
- `retained_bridge::resizable_panel_group::compute_layout` remains only because
  `apps/fret-examples/src/docking_arbitration_demo.rs` still consumes it; that follow-up is tracked
  as `RBX-M1-021`.

Evidence:

- `ecosystem/fret-docking/src/dock/split_geometry.rs`
- `ecosystem/fret-docking/src/dock/layout.rs`
- `ecosystem/fret-docking/src/dock/hit_test.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `crates/fret-ui/src/retained_bridge.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean.
- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: docking split layout, hit-test, drag, drop preview, viewport, and runtime tests
    remain green after the helper extraction.
- `cargo clippy -p fret-docking --all-targets --no-deps -- -D warnings`
  - Result: passed.
  - Scope proven: touched `fret-docking` targets are warning-clean under clippy.
- `cargo check -p fret-demo --bin docking_arbitration_demo`
  - Result: passed.
  - Scope proven: the remaining app/demo bridge `compute_layout` consumer still compiles.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist still pass.
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizeHandle|resizable::|ResizeHandle" ecosystem/fret-docking/src crates/fret-ui/src/retained_bridge.rs apps/fret-examples/src/docking_arbitration_demo.rs`
  - Result: only `apps/fret-examples/src/docking_arbitration_demo.rs` still directly uses
    `retained_bridge::resizable_panel_group`; no `fret-docking` call sites remain and
    `retained_bridge::ResizeHandle` is gone.
  - Scope proven: `RBX-M1-020` completed the docking-private extraction and identified the remaining
    app/demo follow-up.
- `rg -n "retained_bridge::ResizablePanelGroupStyle|retained_bridge::ResizablePanelGroupLayout|fret_ui::retained_bridge::\\{[^\\n]*ResizablePanelGroup|pub use crate::resize_handle::ResizeHandle" crates ecosystem apps -g '*.rs'`
  - Result: no direct repo consumers.
  - Scope proven: `retained_bridge::ResizablePanelGroupStyle` was safe to delete; the layout type is
    still retained only because `retained_bridge::resizable_panel_group::compute_layout` returns it.
- `rg -n "pub fn drag_update_fractions|pub fn drag_update_adjacent_fractions|pub use crate::resizable_panel_group::ResizablePanelGroupStyle" crates/fret-ui/src/retained_bridge.rs`
  - Result: no matches.
  - Scope proven: the no-user bridge drag helpers and style re-export were deleted.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

## 2026-05-18 - RBX-M1-021 demo diagnostics split helper migration

Claim verified:

- `apps/fret-examples/src/docking_arbitration_demo.rs` no longer depends on
  `fret_ui::retained_bridge::resizable_panel_group` for diagnostics split geometry.
- The remaining `retained_bridge::resizable_panel_group` helper module and
  `retained_bridge::ResizablePanelGroupLayout` re-export were deleted after repo-wide no-user
  proof.

Evidence:

- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `crates/fret-ui/src/retained_bridge.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the demo and bridge edits.
- `cargo check -p fret-demo --bin docking_arbitration_demo`
  - Result: passed.
  - Scope proven: the docking arbitration demo still compiles after migrating diagnostics geometry
    off the retained bridge helper.
- `cargo clippy -p fret-demo --bin docking_arbitration_demo --no-deps -- -D warnings`
  - Result: passed.
  - Scope proven: the touched demo target remains warning-clean under clippy.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering remains valid after shrinking the retained bridge surface.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.
- `rg -n "retained_bridge::resizable_panel_group|retained_bridge::ResizablePanelGroupLayout|resizable::compute_layout" crates ecosystem apps -g '*.rs'`
  - Result: no matches.
  - Scope proven: no Rust source still consumes the removed retained bridge split helper or
    retained bridge panel-group layout re-export.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M1-021` is a targeted demo diagnostics migration plus bridge surface deletion; the
    task-local demo compile/clippy gates and retained-bridge no-user proof cover the changed
    behavioral surface.

## 2026-05-18 - RBX-M1-030 docking declarative primitive gap audit

Claim verified:

- Docking panel content already has a declarative rendering path through `DockPanelRegistry` and
  `render_cached_panel_root(...)`.
- Docking cannot safely delete `DockSpace` retained hosting yet because the missing piece is a
  managed host lifecycle: controller state, child-root placement, prepaint liveness, raw event
  arbitration, command/focus routing, and controlled chrome/child paint ordering.
- The next implementation slice should extract a docking-owned `DockSpaceController` before choosing
  or adding a declarative managed-surface primitive.

Evidence:

- `docs/workstreams/retained-bridge-exit-v1/RBX_M1_030_DOCKING_DECLARATIVE_PRIMITIVE_GAP_AUDIT_2026-05-18.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `python3 tools/audit_crate.py --crate fret-docking`
  - Result: passed; `src/dock/space.rs` is the largest source file at 8057 lines; suggested task
    gates remain `cargo fmt`, `cargo nextest run -p fret-docking`, and
    `python3 tools/check_layering.py`.
  - Scope proven: `fret-docking` remains the right audit target for the retained host migration.
- `rg -n "retained_bridge|UiTreeRetainedExt|create_node_retained|RetainedSubtree|impl<.*Widget|impl Widget|Widget<" ecosystem/fret-docking/src ecosystem/fret-docking/tests -g '*.rs'`
  - Result: retained hosting remains in `dock/prelude_ui.rs`, `dock/mod.rs`, `dock/space.rs`,
    `imui.rs`, and retained-focused docking tests.
  - Scope proven: the remaining retained usage is host/lifecycle usage, not the already-removed
    split-helper bridge surface.

Validation:

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the documentation-only update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: retained bridge allowlist and crate layering remain valid after recording the
    docking migration plan.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed documentation has no whitespace errors.

Broader gates not run:

- `cargo nextest run -p fret-docking`
  - Reason: `RBX-M1-030` is an audit/documentation slice with no Rust behavior changes. The next
    implementation slice (`RBX-M1-040`) must run the targeted docking test gate.

## 2026-05-18 - RBX-M1-040 DockSpaceController state extraction

Claim verified:

- `DockSpace` no longer directly owns the bulky cross-frame docking host state fields.
- `DockSpaceController` now owns the retained host state needed by docking interactions, panel
  binding, viewport capture, tab chrome, menu state, prepared text/SVG caches, and diagnostics.
- The retained `DockSpace` widget remains the adapter for event/layout/prepaint/paint/command
  lifecycles; behavior is intentionally unchanged in this slice.

Evidence:

- `ecosystem/fret-docking/src/dock/space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the controller extraction type-checks before running the broader docking test
    gate.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the controller extraction.
- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: docking split layout, hit-test, drag, drop preview, viewport capture, panel
    binding, focus, floating-window, runtime, and diagnostics-sensitive tests remain green after
    moving state behind `DockSpaceController`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the task ledger update.
- `git diff --check`
  - Result: passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/editors/portal_command_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: tracked changes and the new untracked policy file have no whitespace errors.

Follow-up:

- `RBX-M1-050` should extract layout/paint snapshots so the retained adapter and future declarative
  host can consume the same per-frame docking decisions without recomputing layout in paint.

## 2026-05-18 - RBX-M1-050 docking layout/paint snapshot extraction

Claim verified:

- `DockSpace::layout` now builds a reusable private `DockSpaceLayoutSnapshot` after split-motion
  overrides are computed.
- `DockSpace::paint` consumes a same-frame valid snapshot when available and rebuilds one only as a
  fallback when paint does not have a matching layout snapshot.
- The snapshot centralizes root layout, floating layouts, merged layout, active panel bounds, paint
  panel bounds, viewport layouts, host bounds, frame identity, and split handle settings.
- The snapshot and builder are `pub(super)` internal surfaces, so future `dock` module adapters can
  consume the same frame decision object without exposing it as a public crate API.
- Retained `DockSpace` hosting remains in place; this slice only removes layout/paint decision
  duplication from the adapter path.

Evidence:

- `ecosystem/fret-docking/src/dock/space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the snapshot extraction type-checks before broader docking tests.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the snapshot extraction.
- `cargo nextest run -p fret-docking`
  - Result: passed, 111 tests.
  - Scope proven: docking layout, paint, split, drag/drop hints, viewport capture/layout, floating
    windows, panel binding, and runtime tests remain green after snapshot reuse.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the task ledger update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- `RBX-M1-060` should decide whether the existing declarative primitives can host the extracted
  controller + snapshot path, or whether a narrow mechanism-only managed-surface primitive is needed.

## 2026-05-19 - RBX-M1-060 declarative managed-surface host proof

Claim verified:

- Existing declarative primitives were not sufficient for docking because they did not expose a
  mechanism-only way for an ecosystem host to lay out child roots from runtime geometry, keep a
  host surface alive for prepaint, and choose child-root paint order/rects.
- `fret-ui` now provides a narrow `ManagedSurface` declarative primitive. It is mechanism-only:
  handlers are stored in element-local state, props remain plain data, and docking policy stays in
  `fret-docking`.
- A declarative managed surface can consume `DockSpaceLayoutSnapshot` for docking panel-root
  placement and paint ordering without `RetainedSubtreeProps`.
- `DockSpaceLayoutSnapshot::paint_panel_bounds` is graph-order stable; it no longer depends on
  `HashMap` iteration order.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/host_widget.rs`
- `crates/fret-ui/src/declarative/host_widget/layout.rs`
- `crates/fret-ui/src/declarative/host_widget/measure.rs`
- `crates/fret-ui/src/declarative/host_widget/paint.rs`
- `crates/fret-ui/src/declarative/mount.rs`
- `crates/fret-ui/src/declarative/tests/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-ui`
  - Result: passed.
  - Scope proven: the managed-surface primitive type-checks in the `fret-ui` mechanism layer.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the docking snapshot placement helper and proof compile without widening public
    retained APIs.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the managed-surface and docking proof edits.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 3 tests.
  - Scope proven: managed surfaces can place declarative child roots, paint child roots in
    host-selected order/rects, and run prepaint hooks when enabled.
- `cargo nextest run -p fret-docking`
  - Result: passed, 112 tests.
  - Scope proven: docking behavior remains green, including the new declarative managed-surface
    snapshot proof and the retained adapter's shared placement path.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained-bridge allowlist remain valid after adding the
    mechanism primitive and docking proof.

Blocked broader gate:

- `cargo nextest run -p fret-ui -p fret-docking`
  - Result: failed before completion on
    `fret-ui declarative::tests::anchored_layout_invalidation_harness::mechanism_harness_anchored_layout_invalidation_matches_oracles`.
  - Repeat: `cargo nextest run -p fret-ui mechanism_harness_anchored_layout_invalidation_matches_oracles`
    also fails with the same `first-panel` layout-bounds mismatch.
  - Assessment: this is an independent anchored-layout mechanism harness failure outside the
    `ManagedSurface` and docking snapshot proof surface. The `fret-ui managed_surface` targeted
    gate and the full `fret-docking` gate are green.

Follow-up:

- `RBX-M1-070` should replace public retained docking entry points with declarative entry points
  backed by the managed-surface mechanism.
- A separate follow-up should diagnose the anchored-layout harness red test before treating
  `cargo nextest run -p fret-ui -p fret-docking` as a green aggregate gate again.

## 2026-05-19 - RBX-M1-070 public declarative docking entry points

Claim verified:

- `fret-docking` now exposes a public declarative dock-space entry path backed by
  `ManagedSurface`.
- New declarative panel content returns `AnyElement` roots through
  `DockPanelElementRegistry` / `DockPanelElementRegistryService`; the public replacement path no
  longer asks callers to create retained `UiTree` nodes.
- The retained `create_dock_space_node*` and `mount_dock_space*` helpers are explicitly documented
  as legacy compatibility entry points.
- `imui` has a declarative wrapper (`dock_space_declarative*`) for callers that configure the dock
  graph before mounting declarative panel content.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/mod.rs`
- `ecosystem/fret-docking/src/lib.rs`
- `ecosystem/fret-docking/src/imui.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `ecosystem/fret-docking/tests/public_surface_policy.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the declarative docking entry points compile in the default `fret-docking`
    feature set.
- `cargo check -p fret-docking --features imui`
  - Result: passed.
  - Scope proven: the new `imui` declarative wrapper compiles together with the existing legacy
    retained imui bridge.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_hosts_registry_panel_roots`
  - Result: passed, 1 test.
  - Scope proven: public `dock_space_element_from_registry(...)` can render declarative panel roots,
    bind them through `DockPanelContentService`, and place/paint them from
    `DockSpaceLayoutSnapshot`.
- `cargo nextest run -p fret-docking public_docking_surface_prefers_declarative_entry_points retained_docking_entry_points_are_documented_as_legacy`
  - Result: passed, 2 tests.
  - Scope proven: public declarative symbols are exported and the declarative entry module does not
    reference `RetainedSubtreeProps`, `UiTreeRetainedExt`, or `create_node_retained`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 115 tests.
  - Scope proven: existing docking behavior remains green after adding the declarative public entry
    surface and legacy retained-entry documentation.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 3 tests.
  - Scope proven: the underlying `ManagedSurface` mechanism used by the public docking entry points
    remains green after the public surface wiring.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the M1-060/M1-070 code and tests.
- `cargo check -p fret-demo --bin todo_demo`
  - Result: passed.
  - Scope proven: the default native demo binary still compiles after the public docking surface
    additions and re-exports.
- `cargo check -p fret-cookbook --example docking_basics --features cookbook-docking`
  - Result: passed.
  - Scope proven: a public docking example that still uses the legacy retained adapter compiles
    while the new declarative entry points coexist.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after adding `RBX-M1-075`.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Known gap:

- App/demo call sites still use the legacy retained `DockSpace` adapter where they require full
  docking interactions. This is intentional for this slice: retained `DockSpace` still owns event,
  command, focus, internal-drag route, diagnostics, chrome painting, and viewport capture hooks.
  That remaining adapter migration is tracked as `RBX-M1-075` before `RBX-M1-080` can remove
  `fret-docking`'s `fret-ui/unstable-retained-bridge` dependency.

## 2026-05-19 - RBX-M1-075 first slice: managed-surface command/focus hooks

Claim verified:

- `ManagedSurface` now has mechanism-only hooks for event dispatch, command dispatch, and command
  availability. The hooks expose runtime context capabilities needed by interaction-heavy hosts
  without exposing the retained `Widget` API to ecosystem crates.
- `fret-docking` uses the new declarative command hook for
  `dock.focus_requested_panel`, so the public declarative dock-space entry point can focus a
  requested panel root without depending on retained `DockSpace::command`.
- `RBX-M1-075` is not complete yet. This slice only moved command/focus routing; raw docking
  pointer/internal-drag event arbitration, diagnostics/prepaint liveness, chrome painting, drop
  hints, viewport layout sync, and viewport capture still live on the retained `DockSpace` adapter.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/host_widget.rs`
- `crates/fret-ui/src/declarative/host_widget/event/mod.rs`
- `crates/fret-ui/src/declarative/tests/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/services.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-ui managed_surface_dispatches_event_command_and_availability_hooks`
  - Result: passed, 1 test.
  - Scope proven: the new mechanism hooks can receive a pointer event, request focus, report
    command availability, and handle a command through declarative managed-surface authoring.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 4 tests.
  - Scope proven: existing managed-surface layout/prepaint/paint proofs remain green with the new
    event/command hooks.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the declarative docking host command/focus hook compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_handles_focus_requested_panel_command`
  - Result: passed, 1 test.
  - Scope proven: `DockManager::request_activate_panel(..., focus: true)` can emit
    `dock.focus_requested_panel`; the public declarative dock-space host consumes it and focuses the
    requested panel root.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 2 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots
    and now also handles the focus-request command path.
- `cargo nextest run -p fret-docking`
  - Result: passed, 116 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    while the public declarative host starts taking over command/focus routing.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the managed-surface and docking command/focus
    edits.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after recording `RBX-M1-075` first-slice
    progress.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with a small event/prepaint slice. The next safest target is internal-drag
  route keep-alive plus one minimal event arbitration path before moving full docking drag/drop
  behavior.

## 2026-05-19 - RBX-M1-075 second slice: declarative internal-drag route keep-alive

Claim verified:

- The public declarative dock-space host now refreshes `DRAG_KIND_DOCK_PANEL` and
  `DRAG_KIND_DOCK_TABS` internal-drag routes during layout, prepaint, and paint.
- The public declarative dock-space host registers itself as `DockManager::dock_space_node(...)`
  for its window, preserving the retained adapter's dock-space node anchor contract.
- `ManagedSurface` layout/prepaint/paint contexts expose the current host `NodeId`, allowing
  mechanism users to install route anchors without retained `Widget` access.
- `RBX-M1-075` still remains open. This slice moved internal-drag route keep-alive; diagnostics,
  raw docking event arbitration, chrome/drop-hint painting, viewport layout sync, drag ghost, and
  viewport capture are still retained-adapter responsibilities.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_installs_internal_drag_route_anchor`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host installs the dock-panel route during
    layout, refreshes the dock-tabs route during paint/prepaint, and registers the window
    dock-space node without a retained dock-space node.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 3 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles the focus-request command path, and installs internal-drag route anchors.
- `cargo nextest run -p fret-docking`
  - Result: passed, 117 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    while the public declarative host takes over command/focus routing and internal-drag route
    keep-alive.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the route keep-alive edits.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after recording `RBX-M1-075`
    second-slice progress.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` by moving diagnostics publication or one minimal raw event arbitration path
  onto declarative hooks.

## 2026-05-19 - RBX-M1-075 third slice: declarative diagnostics and prepaint liveness

Claim verified:

- Common docking diagnostics publication is no longer embedded only in retained `DockSpace`
  layout/prepaint/paint bodies. Retained and declarative hosts now share the graph/drag diagnostics
  path in `dock/diagnostics.rs`.
- The public declarative dock-space host publishes `WindowInteractionDiagnosticsStore` snapshots
  from its prepaint hook, including active dock-drag diagnostics, dock graph stats, and dock graph
  signature.
- The public declarative dock-space host requests animation frames while an active dock drag affects
  its window, preserving the retained adapter's prepaint liveness intent on the declarative path.
- `RBX-M1-075` still remains open. Raw docking pointer/internal-drag event arbitration,
  chrome/drop-hint painting, viewport layouts, drag ghost, and viewport capture still live on the
  retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/diagnostics.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the diagnostics extraction and declarative prepaint hook compile in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_publishes_diagnostics_and_liveness`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host publishes active dock-drag diagnostics,
    dock graph stats, dock graph signature, and requests animation frames from prepaint while a dock
    drag is active.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 4 tests.
  - Scope proven: managed-surface layout/prepaint/paint/event/command hooks remain green after the
    docking diagnostics/liveness hook usage.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 4 tests.
  - Scope proven: the public declarative dock-space host places/paints registry panel roots, handles
    focus-request commands, installs internal-drag route anchors, and publishes diagnostics/liveness
    state.
- `cargo nextest run -p fret-docking`
  - Result: passed, 118 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    while the public declarative host takes over diagnostics publication and prepaint liveness.
- `cargo check -p fret-docking --features imui`
  - Result: passed.
  - Scope proven: the imui wrapper and retained/imui compatibility surface still compile after the
    shared diagnostics extraction and declarative host hook changes.
- `cargo check -p fret-demo --bin docking_demo --bin container_queries_docking_demo --bin imui_editor_proof_demo`
  - Result: passed.
  - Scope proven: the docking-heavy demo entry-point consumers still compile while app/demo call
    sites remain on retained dock-space helpers until the remaining interaction hooks migrate.
- `cargo check -p fret-cookbook --example docking_basics --features cookbook-docking`
  - Result: passed.
  - Scope proven: the cookbook docking example still compiles against the updated public docking
    surface.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the diagnostics extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after recording `RBX-M1-075`
    third-slice progress.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with a small paint/event slice. The next safest targets are declarative
  chrome/drop-hint/drag-ghost painting from shared snapshot inputs, or one minimal raw pointer event
  arbitration path. Do not switch app/demo call sites until those interaction paths are covered.

## 2026-05-19 - RBX-M1-075 fourth slice: managed-surface paint context capabilities

Claim verified:

- `ManagedSurfacePaintCx` now exposes mechanism-only access to `scale_factor()`, `services()`, and
  `child_bounds(...)` so interaction-heavy ecosystem hosts can prepare text/SVG paint inputs and
  respect actual child-root layout without using the retained `Widget` API.
- The public declarative dock-space host now paints panel roots using `child_bounds(node)` with the
  snapshot rect as a fallback, matching retained `DockSpace::paint` panel-root fallback semantics
  more closely.
- `RBX-M1-075` still remains open. This slice only added the paint context capabilities needed for
  later declarative chrome/drag-ghost/drop-hint painting.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/tests/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: managed-surface layout/prepaint/paint/event/command hooks remain green, including
    the new paint context proof for services, scale factor, and child bounds.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 4 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, and publishes
    diagnostics/liveness state while using actual child bounds for panel-root paint.
- `cargo nextest run -p fret-docking`
  - Result: passed, 118 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after the declarative panel-root paint fallback was aligned with the retained adapter.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the managed-surface paint context capability
    edits.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` by moving one more retained paint or viewport responsibility onto the
  public declarative dock-space host before attempting raw event arbitration.

## 2026-05-19 - RBX-M1-075 fifth slice: declarative viewport layout sync

Claim verified:

- The public declarative dock-space host now syncs `DockSpaceLayoutSnapshot::viewport_layouts` into
  `DockManager::sync_viewport_layouts_for_window(...)` from layout and prepaint.
- App/editor viewport consumers can read the same viewport mapping, draw rect, and stale-layout
  cleanup state from the declarative host path without waiting for retained `DockSpace::paint`.
- `RBX-M1-075` still remains open. Raw docking pointer/internal-drag event arbitration, chrome/drop
  hint painting, drag ghost painting, and viewport input capture still live on the retained
  `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_syncs_viewport_layouts`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host publishes viewport layout state for active
    viewport panels and clears stale viewport layouts for the same window.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the declarative viewport-layout sync compiles in the default `fret-docking`
    feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 5 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, and syncs viewport layouts.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the underlying managed-surface mechanism remains green after the docking
    viewport-layout sync usage.
- `cargo nextest run -p fret-docking`
  - Result: passed, 119 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    while the public declarative host takes over viewport-layout publication.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the declarative viewport-layout sync edits.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- The next implementation slice should target shared chrome/drop-hint/drag-ghost paint input
  extraction or a very small raw event arbitration path. Avoid moving full `DockSpace::event` in
  one step.

## 2026-05-19 - RBX-M1-075 sixth slice: declarative split-handle paint

Claim verified:

- Split-handle paint preparation is no longer embedded only in retained `DockSpace::paint`.
  `paint_split_handles(...)` now delegates through reusable `split_handle_paint_inputs(...)` and
  `paint_split_handle_inputs(...)` helpers.
- The public declarative dock-space host stores split-handle paint inputs in its per-frame output
  and paints split handles from the managed-surface paint hook without borrowing retained
  `DockSpace`.
- `RBX-M1-075` still remains open. Tab/floating chrome, drop hints/overlays, drag ghost painting,
  raw docking pointer/internal-drag event arbitration, and viewport input capture still live on the
  retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared split-handle paint inputs and the declarative paint hook compile in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 5 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, and now paints split handles.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for split-handle chrome.
- `cargo nextest run -p fret-docking`
  - Result: passed, 119 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after extracting split-handle paint inputs.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the split-handle paint extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- The next implementation slice should target either shared tab/floating chrome paint inputs or one
  tightly scoped raw pointer event arbitration path. Avoid moving full `DockSpace::event` in one
  step.

## 2026-05-19 - RBX-M1-075 seventh slice: declarative viewport-surface paint

Claim verified:

- Viewport-surface paint preparation is no longer embedded only in retained `paint_dock(...)`.
  `paint_dock(...)` now delegates through reusable viewport-surface paint input helpers.
- The public declarative dock-space host stores viewport-surface paint inputs in its per-frame
  output and paints `SceneOp::ViewportSurface` plus viewport overlay hooks from the managed-surface
  paint hook.
- Pure viewport panels can now render their viewport surface through the public declarative
  dock-space host without creating a retained `DockSpace` adapter.
- `RBX-M1-075` still remains open. Tab/floating chrome, drop hints/overlays, drag ghost painting,
  raw docking pointer/internal-drag event arbitration, and viewport input capture still live on the
  retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared viewport-surface paint inputs and the declarative paint hook compile in
    the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 6 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, and now paints
    viewport surfaces for pure viewport panels.
- `cargo nextest run -p fret-docking`
  - Result: passed, 120 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after extracting viewport-surface paint inputs.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for viewport surfaces.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the viewport-surface paint extraction.

Follow-up:

- Run the final repo boundary/doc gates after this documentation update:
  `python3 tools/check_layering.py`, `python3 tools/check_workstream_catalog.py`, and
  `git diff --check`.
- The next implementation slice should target either shared tab/floating chrome paint inputs,
  drop-hint/drag-ghost paint input extraction, or one tightly scoped raw pointer event arbitration
  path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 eighth slice: declarative floating chrome paint

Claim verified:

- Floating container chrome paint preparation is no longer embedded only in retained
  `DockSpace::paint`. The retained adapter now delegates the floating outer/title-bar/close-button
  chrome drawing through reusable `FloatingChromePaintInput` helpers.
- The public declarative dock-space host stores floating chrome paint inputs in its per-frame output
  and paints in-window floating outer/title-bar chrome from the managed-surface paint hook without
  borrowing retained `DockSpace`.
- `RBX-M1-075` still remains open. Tab chrome, floating hover/close interaction state, drop
  hints/overlays, drag ghost painting, raw docking pointer/internal-drag event arbitration, and
  viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/consts.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared floating chrome paint inputs and the declarative paint hook compile in
    the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_floating_chrome`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints floating outer/title-bar chrome
    from the declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 7 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, and now paints floating container chrome.
- `cargo nextest run -p fret-docking`
  - Result: passed, 121 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after extracting floating chrome paint inputs.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for floating chrome.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the floating chrome paint extraction.

Follow-up:

- Run the final repo boundary/doc gates after this documentation update:
  `python3 tools/check_layering.py`, `python3 tools/check_workstream_catalog.py`, and
  `git diff --check`.
- The next implementation slice should target drop-hint/drag-ghost paint input extraction, tab
  chrome paint inputs, floating hover/close event state, or one tightly scoped raw pointer event
  arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 ninth slice: declarative drag payload ghost paint

Claim verified:

- Active dock drag ghost snapshot selection is no longer embedded only in retained
  `DockSpace::paint`; it is now exposed as a reusable docking diagnostics helper.
- The public declarative dock-space host stores drag ghost snapshots in its per-frame output,
  prepares the dragged panel title via `ManagedSurfacePaintCx::services()`, and paints the payload
  ghost from the managed-surface paint hook without borrowing retained `DockSpace`.
- `RBX-M1-075` still remains open. Tab chrome, floating hover/close interaction state, drop
  hints/overlays, raw docking pointer/internal-drag event arbitration, and viewport input capture
  still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/diagnostics.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/types.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared drag ghost snapshot selection and the declarative paint hook compile in
    the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_drag_payload_ghost`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints the drag payload ghost from the
    declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 8 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, and now paints drag payload ghosts.
- `cargo nextest run -p fret-docking`
  - Result: passed, 122 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after wiring declarative drag payload ghost paint.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook services for drag ghost title preparation.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the drag payload ghost paint slice.

Follow-up:

- Run the final repo boundary/doc gates after this documentation update:
  `python3 tools/check_layering.py`, `python3 tools/check_workstream_catalog.py`, and
  `git diff --check`.
- The next implementation slice should target drop-hint/drop-overlay paint input extraction, shared
  tab chrome paint inputs, floating hover/close event state, or one tightly scoped raw pointer event
  arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 tenth slice: declarative basic drop overlay paint

Claim verified:

- Basic float/empty/center drop-overlay painting is now available through
  `paint_basic_drop_overlay(...)`, a reusable helper that does not require retained `DockSpace`
  state or full graph edge-preview decisions.
- The public declarative dock-space host stores `DockManager::hover` and
  `DockSpaceLayoutSnapshot::layout_all` in its per-frame output and paints center drop overlays from
  the managed-surface paint hook without borrowing retained `DockSpace`.
- `RBX-M1-075` still remains open. Tab chrome, floating hover/close interaction state, drop hint
  pads, complex edge/tab preview overlays, raw docking pointer/internal-drag event arbitration, and
  viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the basic drop-overlay helper and declarative paint hook compile in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_center_drop_overlay`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints center content/tab-bar drop
    overlays from the declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 9 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, and now paints basic
    center drop overlays.
- `cargo nextest run -p fret-docking`
  - Result: passed, 123 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after wiring declarative basic drop-overlay paint.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for basic drop overlays.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the basic drop-overlay paint slice.

Follow-up:

- Run the final repo boundary/doc gates after this documentation update:
  `python3 tools/check_layering.py`, `python3 tools/check_workstream_catalog.py`, and
  `git diff --check`.
- The next implementation slice should target complex edge/tab preview overlays, shared tab chrome
  paint inputs, floating hover/close event state, or one tightly scoped raw pointer event
  arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 eleventh slice: declarative drop hint pads

Claim verified:

- The public declarative dock-space host now derives `DockDropHints` from the resolved
  `DockManager::hover` target and carries that hint snapshot in its per-frame output.
- The managed-surface paint hook reuses `paint_drop_hints(...)` to paint the drop-hint plate and
  pads without borrowing retained `DockSpace`.
- `RBX-M1-075` still remains open. Tab chrome, floating hover/close interaction state, complex
  edge/tab preview overlays, raw docking pointer/internal-drag event arbitration, and viewport
  input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the declarative drop-hint frame snapshot and paint hook compile in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_drop_hint_pads`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints the drop-hint plate and pads from
    the declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 10 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, and now paints drop-hint pads.
- `cargo nextest run -p fret-docking`
  - Result: passed, 124 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after wiring declarative drop-hint pads.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for drop-hint pads.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the drop-hint pad paint slice.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps the managed-surface mechanism in `fret-ui` and docking policy in
    `fret-docking`.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: no whitespace errors were introduced.

Follow-up:

- Re-run `git diff --check` after this evidence update if this slice is committed separately.
- The next implementation slice should target tab title/close/overflow chrome details, tab-title
  preview text, floating hover/close event state, or one tightly scoped raw pointer event
  arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 thirteenth slice: declarative complex drop overlays

Claim verified:

- Non-text complex drop-overlay geometry is now available through reusable
  `ComplexDropOverlayPaintInput` / `paint_complex_drop_overlay_inputs(...)` helpers.
- Retained `paint_drop_overlay(...)` delegates tab insert markers and edge split-slot preview
  overlays through the shared helper, while still owning tab-title preview text.
- The public declarative dock-space host stores complex drop-overlay inputs in its per-frame output
  and paints edge split-slot previews plus tab insert markers from the managed-surface paint hook.
- `RBX-M1-075` still remains open. Tab title/close/overflow chrome details, tab-title preview text,
  floating hover/close interaction state, raw docking pointer/internal-drag event arbitration, and
  viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared complex drop-overlay inputs and the declarative paint hook compile in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_edge_drop_preview_slot public_declarative_dock_space_entry_point_paints_tab_insert_marker`
  - Result: passed, 2 tests.
  - Scope proven: the public declarative dock-space host paints edge split-slot previews and tab
    insert markers from the declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 13 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, and now paints complex
    edge/marker overlays.
- `cargo nextest run -p fret-docking`
  - Result: passed, 127 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after retained `paint_drop_overlay(...)` delegates non-text complex overlay geometry through the
    shared helper.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for complex drop overlays.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the complex drop-overlay slice.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps overlay policy in `fret-docking` and does not widen `fret-ui`
    mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed before this evidence update.
  - Scope proven: no whitespace errors were introduced in the implementation slice.

Follow-up:

- Re-run `git diff --check` after this evidence update if this slice is committed separately.
- The next implementation slice should target tab title/close/overflow chrome details, tab-title
  preview text, floating hover/close event state, or one tightly scoped raw pointer event
  arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twelfth slice: declarative structural tab chrome

Claim verified:

- Structural tab chrome painting is now available through reusable `TabChromePaintInput` /
  `paint_tab_chrome_inputs(...)` helpers.
- Retained `paint_dock(...)` delegates panel background, tab bar, active/hover tab plate, and active
  underline painting through the shared helper, while still owning tab title, close button,
  overflow, and viewport fill details.
- The public declarative dock-space host stores tab chrome inputs in its per-frame output and
  paints tab bar chrome before panel roots from the managed-surface paint hook.
- `RBX-M1-075` still remains open. Tab title/close/overflow chrome details, floating hover/close
  interaction state, complex edge/tab preview overlays, raw docking pointer/internal-drag event
  arbitration, and viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared structural tab chrome inputs and declarative paint hook compile in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_tab_chrome`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints tab bar chrome and the active tab
    underline from the declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 11 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, and now paints structural tab chrome.
- `cargo nextest run -p fret-docking`
  - Result: passed, 125 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after retained `paint_dock(...)` delegates structural tab chrome painting through the shared
    helper.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 5 tests.
  - Scope proven: the managed-surface mechanism remains green while the docking host consumes its
    paint hook for structural tab chrome.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the tab chrome paint slice.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps tab chrome policy in `fret-docking` and does not widen
    `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed before this evidence update.
  - Scope proven: no whitespace errors were introduced in the implementation slice.

Follow-up:

- Re-run `git diff --check` after this evidence update if this slice is committed separately.
- The next implementation slice should target tab title/close/overflow chrome details, complex
  edge/tab preview overlays, floating hover/close event state, or one tightly scoped raw pointer
  event arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 fourteenth slice: declarative tab-insert preview title

Claim verified:

- Tab-insert preview title painting is now available through reusable
  `paint_tab_insert_preview_title(...)`.
- Retained `paint_drop_overlay(...)` delegates the tab-insert preview title through the shared
  helper, and the public declarative dock-space host paints the same preview title from its
  managed-surface paint hook.
- `ManagedSurfacePaintCx` exposes `release_text_blob_on_next_paint(...)` so paint-time transient
  text blobs remain valid for the scene that references them and are released on the next
  managed-surface repaint or cleanup.
- `RBX-M1-075` still remains open. Tab title/close/overflow chrome details, floating hover/close
  interaction state, raw docking pointer/internal-drag event arbitration, and viewport input
  capture still live on the retained `DockSpace` adapter.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `crates/fret-ui/src/declarative/host_widget.rs`
- `crates/fret-ui/src/declarative/host_widget/paint.rs`
- `crates/fret-ui/src/declarative/tests/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-ui`
  - Result: passed.
  - Scope proven: managed-surface paint-time text release plumbing compiles in the default
    `fret-ui` feature set.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared preview-title painting and the declarative paint hook compile in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism still places/paints child roots, dispatches
    event/command hooks, exposes paint services/bounds, and now defers paint-time text release
    until the next repaint.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_tab_insert_preview_title`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints the tab-insert preview title plate
    and text from the declarative host path.
- `cargo nextest run -p fret-docking`
  - Result: passed, 128 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after retained and declarative hosts share tab-insert preview title painting.
- `cargo fmt`
  - Result: passed.
  - Scope proven: Rust sources were formatted after the implementation slice.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps the transient text mechanism in `fret-ui` and docking preview
    policy in `fret-docking`.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: no whitespace errors were introduced.

Follow-up:

- Continue `RBX-M1-075` with tab title/close/overflow chrome details, floating hover/close event
  state, viewport input capture, or one tightly scoped raw pointer event arbitration path. Avoid
  moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 fifteenth slice: declarative tab detail paint

Claim verified:

- Tab title, active-tab close affordance, overflow button, and overflow menu painting are now
  available through reusable `TabDetailPaintInput` / `paint_tab_detail_inputs(...)` helpers.
- Retained `paint_dock(...)` delegates tab detail painting through the shared helper while still
  owning retained tab resource caches and interaction state.
- The public declarative dock-space host prepares transient tab title/close/overflow text resources
  and paints tab details from its managed-surface paint hook without borrowing retained
  `DockSpace`.
- `RBX-M1-075` still remains open. Tab close/overflow interaction state, floating hover/close
  interaction state, raw docking pointer/internal-drag event arbitration, and viewport input
  capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared tab detail paint inputs/helpers and declarative paint hook integration
    compile in the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_paints_tab_details`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host paints tab title text, active-tab close
    text, and overflow button text from the declarative host path.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 15 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, and now paints tab details.
- `cargo nextest run -p fret-docking`
  - Result: passed, 129 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after retained `paint_dock(...)` delegates tab detail painting through the shared helper.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism and deferred paint-time text release contract
    remain green while the docking host uses transient tab detail text blobs.
- `cargo fmt`
  - Result: passed.
  - Scope proven: Rust sources were formatted after the implementation slice.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps tab paint policy in `fret-docking` and does not widen
    `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: no whitespace errors were introduced.

Follow-up:

- Continue `RBX-M1-075` with tab close/overflow interaction state, floating hover/close event
  state, viewport input capture, or one tightly scoped raw pointer event arbitration path. Avoid
  moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 sixteenth slice: declarative active-tab close interaction

Claim verified:

- The public declarative dock-space host now handles active-tab close affordance
  `PointerDown` / `PointerUp` without delegating to retained `DockSpace::event`.
- `ManagedSurfaceEventCx` remains mechanism-only: event hooks do not read prepaint output. The
  docking host rebuilds a temporary `DockSpaceLayoutSnapshot` from current bounds and `DockManager`
  state for tab-close hit-testing.
- Declarative docking interaction state tracks the pressed tab-close target, captures/releases the
  pointer through the managed-surface event context, and emits
  `Effect::Dock(DockOp::ClosePanel { ... })` when release lands on the same close affordance or
  remains within close-click slop.
- `RBX-M1-075` still remains open. Tab overflow and remaining tab interaction state, floating
  hover/close interaction state, raw docking pointer/internal-drag event arbitration, and viewport
  input capture still live on the retained `DockSpace` adapter.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative tab-close interaction state and managed-surface event context changes
    compile in the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_closes_tab_from_close_affordance`
  - Result: passed, 1 test.
  - Scope proven: clicking the active-tab close affordance through the public declarative dock-space
    entry point emits the expected `DockOp::ClosePanel`.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 16 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, and now handles active-tab close clicks.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green after keeping event context free of
    prepaint-output reads.
- `cargo nextest run -p fret-docking`
  - Result: passed, 130 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative active-tab close path.

Follow-up:

- Continue `RBX-M1-075` with tab overflow/remaining tab interaction state, floating hover/close
  event state, viewport input capture, or one tightly scoped raw pointer/internal-drag arbitration
  path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 seventeenth slice: declarative tab overflow-menu close interaction

Claim verified:

- The public declarative dock-space host now handles the tab overflow button and overflow-menu row
  close click path without delegating to retained `DockSpace::event`.
- Declarative overflow-menu state lives in `fret-docking` and reuses existing
  `TabOverflowMenuState` plus shared `TabDetailPaintInput` / `paint_tab_detail_inputs(...)`;
  `fret-ui` remains a mechanism layer and receives no docking policy/state.
- Clicking the declarative overflow button opens and paints the menu. Clicking a row close
  affordance emits `Effect::Dock(DockOp::ClosePanel { ... })` without also emitting
  `SetActiveTab`; clicking row content emits `Effect::Dock(DockOp::SetActiveTab { ... })` without
  closing a tab.
- `RBX-M1-075` still remains open. Remaining tab hover/scroll/drag interaction state, floating
  hover/close interaction state, raw docking pointer/internal-drag event arbitration, and viewport
  input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative overflow-menu state/event integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_handles_overflow_menu_close`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host opens/paints the overflow menu and emits
    `DockOp::ClosePanel` for row close without also activating the tab.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_activates_overflow_menu_row`
  - Result: passed, 1 test.
  - Scope proven: the public declarative dock-space host emits `DockOp::SetActiveTab` for row
    activation without also closing the tab.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 18 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, and now
    handles overflow-menu close and activation clicks.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while overflow-menu policy/state
    stays in `fret-docking`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 132 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative overflow-menu close and activation paths.

Follow-up:

- Continue `RBX-M1-075` with remaining tab hover/scroll/drag interaction state, floating
  hover/close event state, viewport input capture, or one tightly scoped raw pointer/internal-drag
  arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 eighteenth slice: declarative floating close interaction

Claim verified:

- The public declarative dock-space host now handles in-window floating close
  `PointerDown` / `PointerUp` without delegating to retained `DockSpace::event`.
- Declarative pressed-floating-close state lives in `fret-docking`; `fret-ui` remains a
  mechanism-only managed surface layer and receives no docking policy/state.
- The declarative host reuses `DockSpaceLayoutSnapshot` floating chrome geometry for close
  hit-testing and pressed close painting, emits `Effect::Dock(DockOp::RaiseFloating { ... })` on
  close press, and emits `Effect::Dock(DockOp::MergeFloatingInto { ... })` when release lands on
  the same close affordance.
- `RBX-M1-075` still remains open. Remaining tab hover/scroll/drag interaction state, floating
  hover/title-bar drag interaction state, raw docking pointer/internal-drag event arbitration, and
  viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_closes_floating_chrome`
  - Result: failed before implementation, then passed after implementation.
  - Scope proven: the new public declarative dock-space event path emits `RaiseFloating` on close
    press and `MergeFloatingInto` on close release.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative pressed-floating-close state/event integration compiles in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 19 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, and now handles floating close clicks.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while floating close policy/state
    stays in `fret-docking`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 133 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative floating close path.
- `cargo fmt`
  - Result: passed.
  - Scope proven: Rust sources were formatted after the implementation slice.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps floating close policy in `fret-docking` and does not widen
    `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with remaining tab hover/scroll/drag interaction state, floating
  hover/title-bar drag event state, viewport input capture, or one tightly scoped raw
  pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 nineteenth slice: declarative floating title-bar move interaction

Claim verified:

- The public declarative dock-space host now handles the narrow in-window floating title-bar
  `PointerDown` / `PointerMove` / `PointerUp` move-rect path without delegating to retained
  `DockSpace::event`.
- Declarative floating-drag state lives in `fret-docking`; `fret-ui` remains a mechanism-only
  managed surface layer and receives no docking policy/state.
- The declarative host reuses `DockSpaceLayoutSnapshot` floating chrome geometry for title-bar
  hit-testing, emits `Effect::Dock(DockOp::RaiseFloating { ... })` on title-bar press, captures
  the pointer, emits `Effect::Dock(DockOp::SetFloatingRect { ... })` while the title bar is
  dragged, and releases the pointer on `PointerUp` / cancel.
- This slice intentionally does not move dock-preview/merge-on-release arbitration for floating
  title-bar drags. `RBX-M1-075` still remains open. Remaining tab hover/scroll/drag interaction
  state, floating hover and title-bar dock-preview arbitration, raw docking pointer/internal-drag
  event arbitration, and viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_drags_floating_title_bar`
  - Result: failed before implementation, then passed after implementation.
  - Scope proven: the new public declarative dock-space event path emits `RaiseFloating` on
    title-bar press and `SetFloatingRect` while the title bar is dragged.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative floating-drag state/event integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 20 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles floating close clicks, and now handles floating
    title-bar move clicks.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while floating title-bar drag
    policy/state stays in `fret-docking`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 134 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative floating title-bar move path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps floating title-bar move policy in `fret-docking` and does not
    widen `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with remaining tab hover/scroll/drag interaction state, floating
  hover/title-bar dock-preview arbitration, viewport input capture, or one tightly scoped raw
  pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twentieth slice: declarative overflow-menu wheel scrolling

Claim verified:

- The public declarative dock-space host now handles overflow-menu wheel scrolling without
  delegating to retained `DockSpace::event`.
- The declarative host reuses the retained adapter's overflow-menu geometry and scroll formula:
  `next_scroll = (menu.scroll - (delta.x + delta.y)).clamp(0, max_scroll)`.
- Overflow-menu state stays in `fret-docking` via `DeclarativeDockInteractionService`;
  `fret-ui` remains a mechanism-only managed-surface layer and receives no docking policy/state.
- `RBX-M1-075` still remains open. Remaining tab hover/tab-strip wheel/drag interaction state,
  floating hover and title-bar dock-preview arbitration, raw docking pointer/internal-drag event
  arbitration, and viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_scrolls_overflow_menu_with_wheel`
  - Result: failed before implementation, then passed after implementation.
  - Scope proven: wheel scrolling through the public declarative dock-space host updates overflow
    menu scroll state and allows activating the row exposed by that scroll offset.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative overflow-menu wheel integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 21 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles floating close clicks, handles floating
    title-bar move clicks, and now handles overflow-menu wheel scrolling.

Follow-up:

- Continue `RBX-M1-075` with remaining tab hover/tab-strip scroll/drag interaction state, floating
  hover/title-bar dock-preview arbitration, viewport input capture, or one tightly scoped raw
  pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-first slice: declarative tab-strip wheel scrolling

Claim verified:

- The public declarative dock-space host now handles tab-strip wheel scrolling without delegating
  to retained `DockSpace::event`.
- Declarative tab-scroll state lives in `fret-docking`, keyed by window and tabs node; `fret-ui`
  remains a mechanism-only managed-surface layer and receives no docking policy/state.
- The declarative host feeds tab-scroll state into tab chrome/detail paint inputs, tab close
  hit-testing, overflow-menu opening, and tab-insert preview painting.
- The declarative host reuses the retained adapter's tab-strip scroll formula:
  `next_scroll = (scroll - (delta.x + delta.y)).clamp(0, max_scroll)`.
- `RBX-M1-075` still remains open. Remaining tab hover/drag interaction state, floating hover and
  title-bar dock-preview arbitration, raw docking pointer/internal-drag event arbitration, and
  viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_scrolls_tab_strip_with_wheel`
  - Result: failed before implementation by closing panel 0 with unscrolled hit-testing, then
    passed after implementation.
  - Scope proven: wheel scrolling through the public declarative dock-space host updates tab-strip
    scroll state and makes the expected tab close hit-testable.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative tab-scroll state/event integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 22 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles floating
    close clicks, handles floating title-bar move clicks, and now handles tab-strip wheel
    scrolling.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while tab-scroll policy/state stays
    in `fret-docking`.

Follow-up:

- Continue `RBX-M1-075` with remaining tab hover/drag interaction state, floating
  hover/title-bar dock-preview arbitration, viewport input capture, or one tightly scoped raw
  pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-second slice: declarative tab and overflow hover state

Claim verified:

- The public declarative dock-space host now handles tab hover, tab overflow button hover, and
  overflow menu row hover without delegating to retained `DockSpace::event`.
- Declarative tab hover state lives in `fret-docking` via `DeclarativeDockInteractionService`;
  `fret-ui` remains a mechanism-only managed-surface layer and receives no docking policy/state.
- The declarative paint hook now refreshes transient tab interaction paint state from the latest
  docking service state at paint time. This prevents hover/menu visuals from being stuck on the
  older layout/prepaint frame output while keeping stable geometry in `DockSpaceElementFrame`.
- `RBX-M1-075` still remains open. Remaining tab drag interaction state, floating hover and
  title-bar dock-preview arbitration, raw docking pointer/internal-drag event arbitration, and
  viewport input capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_hovers_tab_overflow_button`
  - Result: failed before implementation because the overflow button hover background was not
    painted from the public declarative host, then passed after implementation.
  - Scope proven: moving over an overflowing tab bar's overflow button paints the expected hover
    background through the public declarative dock-space host.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_hovers`
  - Result: passed, 3 tests.
  - Scope proven: the public declarative dock-space host paints ordinary tab hover, overflow
    button hover, and overflow menu row hover state.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative tab/overflow hover state integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 25 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles floating close clicks, handles floating title-bar move
    clicks, and now handles tab/overflow hover state.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while tab hover policy/state stays
    in `fret-docking`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 139 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative tab/overflow hover path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps tab hover policy in `fret-docking` and does not widen
    `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with remaining tab drag interaction state, floating hover/title-bar
  dock-preview arbitration, viewport input capture, or one tightly scoped raw
  pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-third slice: declarative tab drag activation

Claim verified:

- The public declarative dock-space host now handles the narrow panel-tab drag activation path
  without delegating to retained `DockSpace::event`.
- Declarative pending dock-drag state lives in `fret-docking` via
  `DeclarativeDockInteractionService`; `fret-ui` remains a mechanism-only managed-surface layer
  and receives no docking policy/state.
- The declarative host starts a `DRAG_KIND_DOCK_PANEL` runtime drag only after the configured
  `DockingInteractionSettings::tab_drag_threshold` is satisfied, releases pointer capture once the
  runtime drag starts, carries the tab-local grab offset into `DockPanelDragPayload`, and respects
  `DockingPolicy::allow_panel_drag`.
- `RBX-M1-075` still remains open. Tabs-group drag activation, floating hover and title-bar
  dock-preview arbitration, raw docking pointer/internal-drag event arbitration, and viewport input
  capture still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_starts_tab_drag_after_threshold`
  - Result: failed before implementation because no declarative runtime dock drag was created,
    then passed after implementation.
  - Scope proven: dragging a panel tab through the public declarative dock-space host starts a
    `DRAG_KIND_DOCK_PANEL` session with the expected panel payload, position, grab offset, and
    default dock-preview policy.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_ --retries 0`
  - Result: passed, 28 tests.
  - Scope proven: the public declarative dock-space host still passes the full filtered behavior
    suite, including tab drag activation, tab drag threshold gating, and
    `DockingPolicy::allow_panel_drag` gating.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative pending dock-drag state/event integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 28 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles tab/overflow hover state, handles floating close clicks,
    handles floating title-bar move clicks, and now handles panel-tab drag activation.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while tab drag policy/state stays in
    `fret-docking`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 142 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative panel-tab drag activation path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps tab drag policy in `fret-docking` and does not widen `fret-ui`
    mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with tabs-group drag activation, floating hover/title-bar dock-preview
  arbitration, viewport input capture, or one tightly scoped raw pointer/internal-drag arbitration
  path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-fourth slice: declarative tabs-group drag activation

Claim verified:

- The public declarative dock-space host now handles tabs-group drag activation from empty tab-bar
  space without delegating to retained `DockSpace::event`.
- Declarative pending tabs-drag state lives in `fret-docking` via
  `DeclarativeDockInteractionService`; `fret-ui` remains a mechanism-only managed-surface layer
  and receives no docking policy/state.
- The declarative host starts a `DRAG_KIND_DOCK_TABS` runtime drag only after the configured
  `DockingInteractionSettings::tab_drag_threshold` is satisfied, releases pointer capture once the
  runtime drag starts, carries the tab-bar-local grab offset into `DockTabsDragPayload`, and
  respects `DockingPolicy::allow_tabs_group_drag`.
- `RBX-M1-075` still remains open. Floating hover/title-bar dock-preview and merge-on-release
  arbitration, raw docking pointer/internal-drag event arbitration, and viewport input capture
  still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `ecosystem/fret-docking/src/dock/tests/mod.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_starts_tabs_group_drag_after_threshold public_declarative_dock_space_entry_point_respects_tabs_group_drag_policy`
  - Result: failed before implementation because no declarative runtime tabs drag was created,
    then passed after implementation, 2 tests.
  - Scope proven: dragging empty tab-bar space through the public declarative dock-space host
    starts a `DRAG_KIND_DOCK_TABS` session after threshold and respects
    `DockingPolicy::allow_tabs_group_drag`.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 30 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles tab/overflow hover state, handles floating close clicks,
    handles floating title-bar move clicks, handles panel-tab drag activation, and now handles
    tabs-group drag activation.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative pending tabs-drag state/event integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green while tabs-group drag policy/state
    stays in `fret-docking`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 144 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative tabs-group drag activation path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps tabs-group drag policy in `fret-docking` and does not widen
    `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with floating hover/title-bar dock-preview and merge-on-release
  arbitration, viewport input capture, or one tightly scoped raw pointer/internal-drag arbitration
  path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-fifth slice: declarative floating title-bar center merge

Claim verified:

- The public declarative dock-space host now handles the floating title-bar drag dock-preview and
  center merge-on-release path without delegating to retained `DockSpace::event`.
- Declarative floating-drag activation state lives in `fret-docking` via
  `DeclarativeDockInteractionService`; `fret-ui` remains a mechanism-only managed-surface layer
  and receives no docking policy/state.
- The declarative host latches dock-preview inversion policy when the configured
  `DockingInteractionSettings::tab_drag_threshold` is satisfied, resolves `DockManager::hover`
  over the root dock layout while the activated floating title-bar drag moves, respects
  `DockingPolicy::allow_dock_drop_target`, and emits `DockOp::MergeFloatingInto` on center drop
  release.
- `RBX-M1-075` still remains open. Raw docking pointer/internal-drag event arbitration, viewport
  input capture, and remaining floating hover visual state still live on the retained `DockSpace`
  adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_merges_floating_title_bar_drag_on_center_drop`
  - Result: failed before implementation because declarative floating title-bar drag left
    `DockManager::hover` as `None`, then passed after implementation, 1 test.
  - Scope proven: dragging a floating title bar through the public declarative dock-space host
    resolves a center dock hover and emits `DockOp::MergeFloatingInto` on release without creating
    retained `DockSpace`.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_merges_floating_title_bar_drag_on_center_drop public_declarative_dock_space_entry_point_drags_floating_title_bar`
  - Result: passed, 2 tests.
  - Scope proven: the declarative floating title-bar drag still emits `SetFloatingRect` while the
    new center merge path is active.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 31 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles tab/overflow hover state, handles floating close clicks,
    handles floating title-bar move clicks, handles panel-tab drag activation, handles tabs-group
    drag activation, and now handles floating title-bar center merge.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative floating-drag activation/hover/merge integration compiles in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking`
  - Result: passed, 145 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative floating title-bar center merge path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps floating title-bar drag policy in `fret-docking` and does not
    widen `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with viewport input capture, remaining floating hover visual state, or one
  tightly scoped raw pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event`
  in one step.

## 2026-05-19 - RBX-M1-075 twenty-sixth slice: declarative viewport input capture

Claim verified:

- The public declarative dock-space host now handles left-button viewport pointer capture without
  delegating to retained `DockSpace::event`.
- Declarative viewport capture state lives in `fret-docking` via
  `DeclarativeDockInteractionService`; `fret-ui` remains a mechanism-only managed-surface layer
  and receives no docking policy/state.
- The declarative host forwards `ViewportInputKind::PointerDown`, clamped captured `PointerMove`,
  `PointerUp`, and `PointerCancel` effects through the shared viewport helper path, and
  requests/releases pointer capture on the managed-surface host node.
- `RBX-M1-075` still remains open. Raw docking pointer/internal-drag event arbitration and
  remaining floating hover visual state still live on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_captures_viewport_pointer_input public_declarative_dock_space_entry_point_cancels_viewport_pointer_capture`
  - Result: failed before implementation because the declarative host emitted no viewport input
    effect and did not request pointer capture, then passed after implementation, 2 tests.
  - Scope proven: the public declarative dock-space host starts viewport capture on left pointer
    down, forwards clamped captured moves outside the draw rect to the original viewport, releases
    capture on pointer up, and emits pointer cancel input while releasing capture.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 33 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles tab/overflow hover state, handles floating close clicks,
    handles floating title-bar move clicks, handles panel-tab drag activation, handles tabs-group
    drag activation, handles floating title-bar center merge, and now handles viewport pointer
    capture.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative viewport capture state/event integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking`
  - Result: passed, 147 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative viewport capture path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps viewport capture policy/state in `fret-docking` and does not
    widen `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with remaining floating hover visual state or one tightly scoped raw
  pointer/internal-drag arbitration path. Avoid moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-seventh slice: declarative floating chrome hover

Claim verified:

- The public declarative dock-space host now handles floating close/title-bar hover visual state
  without delegating to retained `DockSpace::event`.
- Declarative floating hover state lives in `fret-docking` via
  `DeclarativeDockInteractionService`; `fret-ui` remains a mechanism-only managed-surface layer
  and receives no docking policy/state.
- The declarative host updates floating hover state from `PointerMove` hit-tests, applies that
  state at paint time so visuals use the latest event state rather than stale layout/prepaint frame
  output, and preserves retained cursor hints for floating close/title-bar hover.
- `RBX-M1-075` still remains open. Raw docking pointer/internal-drag event arbitration still lives
  on the retained `DockSpace` adapter.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_hovers_floating_chrome`
  - Result: failed before implementation because the declarative host painted floating chrome with
    hover fields fixed to `false`, then passed after implementation, 1 test.
  - Scope proven: moving the pointer over a floating title bar paints the retained-style
    translucent title-bar hover background, and moving over the floating close affordance paints
    the retained-style close hover affordance.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 34 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles tab/overflow hover state, handles floating close clicks,
    handles floating title-bar move clicks, handles panel-tab drag activation, handles tabs-group
    drag activation, handles floating title-bar center merge, handles viewport pointer capture,
    and now handles floating close/title-bar hover visuals.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative floating hover state/event/paint integration compiles in the default
    `fret-docking` feature set.
- `cargo nextest run -p fret-docking`
  - Result: passed, 148 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative floating hover path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps floating hover policy/state in `fret-docking` and does not
    widen `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with one tightly scoped raw pointer/internal-drag arbitration path. Avoid
  moving full `DockSpace::event` in one step.

## 2026-05-19 - RBX-M1-075 twenty-eighth slice: declarative internal-drag drop hover cleanup

Claim verified:

- The public declarative dock-space host now handles the stale-hover cleanup part of raw
  `InternalDrag` arbitration without delegating to retained `DockSpace::event`.
- Declarative docking state remains in `fret-docking`: the managed-surface event hook clears
  `DockManager::hover` for `InternalDragKind::{Drop, Leave, Cancel}` and requests redraw only when
  a hover was actually cleared.
- `fret-ui` remains a mechanism-only managed-surface layer; no docking policy/state was added to
  `fret-ui`.
- `RBX-M1-075` still remains open. Full `InternalDragKind::{Enter, Over, Drop}` target resolution,
  drop-intent application, diagnostics, and tear-off arbitration still need a separate declarative
  migration slice before `RBX-M1-080`.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_clears_hover_on_internal_drag_drop_without_drag_session`
  - Result: passed, 1 test.
  - Scope proven: an `InternalDragKind::Drop` delivered through the public declarative dock-space
    entry point clears stale `DockManager::hover` even when no runtime drag session is active.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 35 tests.
  - Scope proven: the public declarative dock-space host still places/paints registry panel roots,
    handles focus-request commands, installs internal-drag route anchors, publishes
    diagnostics/liveness state, syncs viewport layouts, paints split handles, paints viewport
    surfaces, paints floating container chrome, paints drag payload ghosts, paints basic center
    drop overlays, paints drop-hint pads, paints structural tab chrome, paints complex overlays,
    paints tab-insert preview titles, paints tab details, handles active-tab close clicks, handles
    overflow-menu close/activation clicks, handles overflow-menu wheel scrolling, handles
    tab-strip wheel scrolling, handles tab/overflow hover state, handles floating close clicks,
    handles floating title-bar move/merge paths, handles panel-tab drag activation, handles
    tabs-group drag activation, handles viewport pointer capture, handles floating close/title-bar
    hover visuals, and now clears stale hover on internal drag drop.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative internal-drag cleanup compiles in the default `fret-docking` feature
    set.
- `cargo nextest run -p fret-docking`
  - Result: passed, 149 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after adding the declarative internal-drag cleanup path.
- `cargo fmt --check`
  - Result: initially reported rustfmt changes for this slice; passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps raw docking arbitration policy/state in `fret-docking` and does
    not widen `fret-ui` mechanism contracts.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with full declarative `InternalDragKind::{Enter, Over, Drop}` hover/drop
  target resolution and drop-intent application. Keep this as one or more narrow slices; do not
  move the entire retained `DockSpace::event` body at once.

## 2026-05-19 - RBX-M1-075 twenty-ninth slice: shared drop target resolution and declarative internal-drag over

Claim verified:

- Dock drop-target resolution is no longer a retained-only local function inside
  `DockSpace::event`; the shared, docking-private resolution logic now lives in
  `dock/drop_resolve.rs`.
- The retained `DockSpace` adapter still uses the same resolution behavior through the shared
  helper, preserving existing drag/drop semantics while reducing retained-bridge coupling.
- The public declarative dock-space host now handles `InternalDragKind::{Enter, Over}` enough to
  resolve and publish `DockManager::hover` from the same shared drop-target resolver.
- `ManagedSurfaceEventCx` exposes the existing mechanism-only window-local pointer position helper
  so declarative hosts can use pre-transform event positions without depending on retained
  `EventCx`.
- `RBX-M1-075` still remains open. Drop-intent application, drop-time cancellation/end-drag,
  tear-off debounce mutation, and tab-bar drag auto-scroll still need separate declarative slices
  before `RBX-M1-080`.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `ecosystem/fret-docking/src/dock/drop_resolve.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_resolves_internal_drag_over_outer_hint_rect`
  - Result: passed, 1 test.
  - Scope proven: an `InternalDragKind::Over` delivered through the public declarative dock-space
    entry point resolves the root split's outer-left hint rect and updates `DockManager::hover`
    without creating retained `DockSpace`.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point`
  - Result: passed, 36 tests.
  - Scope proven: all public declarative dock-space entry-point behavior remains green with the new
    internal-drag `Over` hover resolution path.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared drop resolution and declarative internal-drag hover integration compile in
    the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking`
  - Result: passed, 150 tests.
  - Scope proven: existing retained docking drag/drop/viewport/floating behavior remains green
    after moving drop-target resolution into the shared docking-private helper.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the managed-surface mechanism remains green after exposing the
    window-local pointer position helper through `ManagedSurfaceEventCx`.
- `cargo fmt --check`
  - Result: initially reported rustfmt changes for this slice; passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the implementation and evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: this slice keeps docking policy/state in `fret-docking` and only adds a
    mechanism-only `ManagedSurfaceEventCx` context helper in `fret-ui`.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with declarative `InternalDragKind::Drop` drop-intent application using
  the shared resolver. Keep tear-off debounce mutation and tab-bar drag auto-scroll as explicit
  follow-on checks rather than silently treating them as covered by hover resolution.

## 2026-05-19 - RBX-M1-075 thirtieth slice: declarative internal-drag drop intent and end-drag

Claim verified:

- Dock drop-intent resolution/application is no longer retained-only local logic inside
  `DockSpace::event`; the shared, docking-private intent helpers now live in `dock/drop_resolve.rs`.
- The retained `DockSpace` adapter uses the shared drop-intent helpers, preserving the existing
  retained drop behavior while shrinking retained-only policy code.
- The public declarative dock-space host now handles `InternalDragKind::Drop` for active dock-panel
  and dock-tabs drags by resolving the target through the shared resolver, applying the shared
  `DockDropIntent` into `Effect::Dock(...)`, clearing hover, invalidating layout when an op is
  emitted, and ending the active dock drag session.
- A public declarative dock-space test proves an inner-left hint-rect drop emits
  `DockOp::MovePanel`, applies cleanly to split the tabs node, and cancels the active drag session
  without creating retained `DockSpace`.
- `RBX-M1-075` still remains open. Tear-off debounce mutation and tab-bar drag auto-scroll still
  need separate declarative slices before `RBX-M1-080`.

Evidence:

- `ecosystem/fret-docking/src/dock/drop_resolve.rs`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/space.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_drops_panel_on_inner_left_hint_rect`
  - Result: failed before implementation because the public declarative host emitted no `DockOp`;
    failed again after adding the end-drag assertion because the active drag session remained; passed
    after wiring shared drop-intent application and `cancel_drag(...)`.
  - Scope proven: this test drove the declarative `Drop` behavior end-to-end rather than only
    checking a private helper.
- `cargo nextest run -p fret-docking dock_drop_left_emits_move_panel_and_splits_tabs_node`
  - Result: passed.
  - Scope proven: the retained adapter's existing inner-left drop behavior remains green after
    extracting shared drop-intent helpers.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: shared drop-intent helpers and declarative `Drop` integration compile in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking`
  - Result: passed, 151 tests.
  - Scope proven: retained and declarative docking drag/drop/floating/viewport behavior remains
    green after moving `Drop` intent application onto the public declarative dock-space host.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after this slice.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: docking drop policy/state stays in `fret-docking`; no new retained bridge or
    reverse dependency leakage was introduced.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with tear-off debounce mutation and tab-bar drag auto-scroll on the public
  declarative dock-space host. Keep them explicit; do not claim `RBX-M1-080` until those retained
  event responsibilities have been either ported or intentionally deleted with evidence.

## 2026-05-19 - RBX-M1-075 thirty-first slice: declarative tab-bar drag auto-scroll

Claim verified:

- Tab-bar drag auto-scroll for active dock drags is no longer retained-only behavior in
  `DockSpace::event`; the public declarative dock-space host now applies the same event-side
  auto-scroll when `InternalDragKind::{Enter, Over}` resolves a center tab-bar drop target.
- Declarative docking now caches tab widths measured during managed-surface paint in
  `DeclarativeDockInteractionService`, then reuses those widths for event-side hit testing,
  drop-target resolution, and auto-scroll. This preserves retained behavior after paint while
  keeping the first-frame approximate-width fallback for hosts that have not painted yet.
- The retained `DockSpace` adapter's tab-bar drag auto-scroll behavior remains green.
- `RBX-M1-075` still remains open. Tear-off debounce mutation remains the last explicit retained
  event responsibility to port or intentionally delete before `RBX-M1-080`.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative measured tab-width cache and auto-scroll integration compile in the
    default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_auto_scrolls_tab_bar_during_dock_drag`
  - Result: failed before measured-width cache because scroll accumulated but the approximate
    short-title tab geometry kept the insert index at `7`; passed after paint-measured tab widths
    were cached and reused by event-side geometry.
  - Scope proven: repeated public declarative `InternalDragKind::Over` events near the tab-bar right
    edge advance the drop insert index without creating retained `DockSpace`.
- `cargo nextest run -p fret-docking dock_drag_auto_scrolls_tab_bar_near_edges`
  - Result: passed.
  - Scope proven: retained tab-bar drag auto-scroll behavior remains unchanged.
- `cargo nextest run -p fret-docking`
  - Result: passed, 152 tests.
  - Scope proven: retained and declarative docking drag/drop/floating/viewport behavior remains
    green after moving declarative tab-bar drag auto-scroll.
- `cargo fmt`
  - Result: passed.
  - Scope proven: Rust sources were formatted after the slice.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the docs/evidence update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: the measured tab-width cache stays in `fret-docking`; no `fret-ui` mechanism
    layer or retained-bridge dependency leakage was introduced.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Follow-up:

- Continue `RBX-M1-075` with tear-off debounce mutation on the public declarative dock-space host.
  After that, start `RBX-M1-080` by removing `fret-docking`'s `fret-ui/unstable-retained-bridge`
  dependency and retained docking public entry points that are no longer needed.

## 2026-05-19 - RBX-M1-075 thirty-second slice: declarative tear-off debounce mutation

Claim verified:

- Stable out-of-bounds tear-off debounce mutation is no longer retained-only behavior in
  `DockSpace::event`; the public declarative dock-space host now updates drag payload
  `tear_off_oob_start_frame`, suppresses duplicate tear-off requests, and emits
  `DockOp::RequestTearOffPanel` / `DockOp::RequestTearOffTabs` after the second stable OOB frame.
- Declarative docking now reads `PlatformCapabilities.ui.window_tear_off`, gates requests through
  `DockingPolicy::allow_tear_off(...)`, preserves the conservative default for multi-window
  sessions, and exposes `DockSpaceElementOptions::allow_multi_window_tear_off` for the per-host
  opt-in that the retained adapter previously provided through `DockSpace`.
- The retained adapter's tear-off comparison tests remain green, and the full `fret-docking` gate
  passes with 153 tests.
- `RBX-M1-075` is complete. The remaining M1 work is `RBX-M1-080`: remove `fret-docking`'s
  dependency on `fret-ui/unstable-retained-bridge` and delete/quarantine retained docking entry
  points.

Evidence:

- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: declarative tear-off debounce state, platform-capability gating, and option
    plumbing compile in the default `fret-docking` feature set.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_requests_tear_off_after_stable_oob_frame`
  - Result: passed after implementation.
  - Scope proven: the public declarative host emits a tear-off request only after a stable second
    OOB frame, without creating retained `DockSpace`.
- `cargo nextest run -p fret-docking`
  - Result: passed, 153 tests.
  - Scope proven: retained and declarative docking drag/drop/floating/viewport behavior remains
    green after moving tear-off debounce mutation onto the public declarative dock-space host.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the existing retained-bridge allowlist remain valid before
    `RBX-M1-080` removes `fret-docking` from that allowlist.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Follow-up:

- Start `RBX-M1-080` by migrating app/example call sites to declarative docking entry points, then
  remove retained docking public helpers and `fret-docking`'s
  `fret-ui/unstable-retained-bridge` dependency.

## 2026-05-19 - RBX-M1-080 retained capability parity audit and docking bridge removal

Claim under verification:

- `fret-docking` no longer depends on `fret-ui/unstable-retained-bridge`.
- The retained docking adapter and public retained docking entry points were deleted rather than
  quarantined because the public declarative dock-space host now covers docking layout, paint,
  event, command/focus, diagnostics, viewport, drag/drop, floating, tab, split, and tear-off
  behavior.
- Deleted retained tests were not kept as dead documentation; their covered capabilities are mapped
  to compiling public declarative or mechanism-level tests.

Retained test capability mapping:

- `bounds.rs`
  - Replaced by
    `public_declarative_dock_space_entry_point_records_panel_root_bounds_for_element_queries`,
    proving public declarative dock hosts place panel roots into the element bounds query path.
  - The underlying mechanism remains covered by `fret-ui` bounds/managed-surface tests.
- `tab_bar.rs`
  - Replaced by public declarative tab-drop tests:
    `public_declarative_dock_space_entry_point_tab_drop_uses_over_tab_halves_for_insert_index`,
    `public_declarative_dock_space_entry_point_tab_drop_reorders_tabs_when_move_op_is_applied`, and
    `public_declarative_dock_space_entry_point_tab_drop_reserved_overflow_header_inserts_at_end`.
  - Lower-level geometry remains covered by `tab_bar_geometry`, `tab_bar_kernel`, and
    `tab_bar_drop_target` tests.
- `viewport.rs`
  - Replaced by public declarative viewport tests for layout sync, viewport surface painting,
    left-button capture/move/up/cancel, other-pointer suppression, right-click forwarding, and
    right-drag context-menu suppression.
- `drag.rs`
  - Replaced by public declarative tests for tab drag activation, threshold gating, panel drag
    policy, tabs-group drag activation/policy, diagnostics/liveness, internal-drag route anchors,
    stale hover cleanup, over/drop resolution, auto-scroll, tear-off debounce, and floating
    title-bar merge.
  - Pointer occlusion and foreign-capture arbitration remain mechanism concerns in `fret-ui`; the
    docking host tests cover docking-owned policy and payload effects.
- `drop_hints.rs`
  - Replaced by public declarative drop-hint, center overlay, edge preview, tab insert marker,
    tab insert preview title, and inner/outer drop intent tests, plus lower-level drop resolver
    coverage.
- `floating.rs`
  - Replaced by public declarative floating chrome paint, floating hover, floating close,
    floating title-bar drag, floating title-bar center merge, in-window float/drop, and tear-off
    request tests.
- `split.rs`
  - Retained split tests were migrated instead of deleted: `dock::tests::split::*` now covers
    split-handle hover, drag, min-size policy, viewport min-size defaults, and n-ary adjacent
    resize through the public declarative dock host.

Evidence:

- `ecosystem/fret-docking/Cargo.toml`
- `ecosystem/fret-docking/src/dock/declarative.rs`
- `ecosystem/fret-docking/src/dock/drop_resolve.rs`
- `ecosystem/fret-docking/src/dock/host_frame.rs`
- `ecosystem/fret-docking/src/dock/paint.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `ecosystem/fret-docking/src/dock/tests/split.rs`
- `ecosystem/fret-docking/tests/public_surface_policy.rs`
- `tools/check_layering.py`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_tab_drop public_declarative_dock_space_entry_point_viewport_capture_ignores_other_pointer_move_and_up`
  - Result: passed earlier in this task, 4 tests.
  - Scope proven: tab drop parity and other-pointer viewport capture parity run through the public
    declarative dock-space host.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_records_panel_root_bounds_for_element_queries`
  - Result: passed earlier in this task, 1 test.
  - Scope proven: retained `bounds.rs` panel-root query behavior is covered on the public
    declarative entry point.
- `cargo check -p fret-docking`
  - Result: passed during cleanup.
  - Scope proven: after deleting retained adapter wrappers and dead retained-era fields,
    `fret-docking` compiles without retained bridge code.
- `cargo fmt --check -p fret-docking`
  - Result: passed.
  - Scope proven: touched `fret-docking` Rust sources are formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after rustfmt.
- `cargo check -p fret-docking`
  - Result: passed.
  - Scope proven: the default `fret-docking` feature set compiles after deleting retained docking
    code.
- `cargo check -p fret-docking --features imui`
  - Result: passed.
  - Scope proven: the imui compatibility wrapper now mounts declarative docking without retained
    bridge dependencies.
- `cargo clippy -p fret-docking --all-targets --features imui --no-deps -- -D warnings`
  - Result: passed after fixing clippy findings in the declarative host.
  - Scope proven: touched `fret-docking` targets are warning-clean under clippy.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 6 tests.
  - Scope proven: the `ManagedSurface` mechanism used by declarative docking still handles
    child-root layout, prepaint, paint ordering, paint-time text release, services/scale access,
    and event/command hooks.
- `cargo nextest run -p fret-docking`
  - Result: passed, 85 tests.
  - Scope proven: public declarative docking, split, viewport, tab, drop, floating, runtime, and
    public-surface policy tests all pass after deleting retained docking tests/code.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: `fret-docking` is no longer on the retained-bridge allowlist and no forbidden
    dependency direction was introduced.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.
- `rg -n "DockSpace::|create_node_retained|retained_bridge|UiTreeRetainedExt|RetainedSubtree|DockPanelRegistry|with_panel_content|unstable-retained-bridge" ecosystem/fret-docking/src ecosystem/fret-docking/tests ecosystem/fret-docking/Cargo.toml -g '*.rs' -g 'Cargo.toml'`
  - Result: only `ecosystem/fret-docking/tests/public_surface_policy.rs` negative assertion
    strings matched.
  - Scope proven: `fret-docking` source/Cargo files no longer contain retained bridge API or
    retained docking entry point dependencies.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: this is a focused retained-bridge exit slice with a very large workspace; task-local
    `fret-docking`, `fret-ui managed_surface`, layering, clippy, catalog, and residual-reference
    gates cover the changed contracts.

## 2026-05-19 - RBX-M1-080 follow-up public declarative parity backfill

Claim verified:

- Deleted retained docking tests now have additional public declarative entry-point coverage for
  cross-window element bounds scoping, cross-window overlay-anchor lookup, viewport-panel registry
  child event reachability, and missing non-viewport panel fallback UI.
- The viewport-panel registry test preserves the old retained `FocusOnDown` behavior by using a
  declarative `PointerRegion` root that actively requests focus on pointer down. This confirms the
  child root is event-reachable without conflating `SemanticsProps::focusable` with pointer-driven
  focus policy.

Evidence:

- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `ecosystem/fret-docking/src/dock/tests/mod.rs`

Commands:

- `cargo nextest run -p fret-docking public_declarative_registry_binds_viewport_panel_element_when_registry_returns_one`
  - Result: passed, 1 test.
  - Scope proven: registry-provided viewport panel elements remain bound, laid out, event-reachable,
    and able to request focus through the public declarative dock-space host.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_keeps_bounds_window_scoped_across_windows`
  - Result: passed, 1 test.
  - Scope proven: committed element bounds for panel roots remain window-scoped across multiple
    declarative dock-space hosts.
- `cargo nextest run -p fret-docking public_declarative_dock_space_entry_point_uses_window_local_anchor_for_overlay_placement`
  - Result: passed, 1 test.
  - Scope proven: overlay placement reads the window-local committed anchor bounds and does not
    leak anchors from another window.
- `cargo nextest run -p fret-docking public_declarative_registry_falls_back_to_placeholder_for_missing_non_viewport_panel_ui`
  - Result: passed, 1 test.
  - Scope proven: a missing registry element for a non-viewport panel still binds and paints the
    fallback placeholder UI through the public declarative registry path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean after the test backfill.
- `cargo nextest run -p fret-docking`
  - Result: passed, 89 tests.
  - Scope proven: public declarative docking, split, viewport, tab, drop, floating, runtime, and
    public-surface policy tests all pass with the retained bridge removed and the parity backfill in
    place.
- `cargo clippy -p fret-docking --all-targets --no-deps -- -D warnings`
  - Result: passed.
  - Scope proven: `fret-docking` all-targets remain warning-clean after adding the public
    declarative parity tests.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: this follow-up only backfills `fret-docking` tests. The package test gate, clippy,
    layering, catalog, format, and whitespace checks cover the changed files.

## 2026-05-19 - RBX-M1-080 follow-up anchored layout order hardening

Claim verified:

- Declarative layout submission for wrappers that mix layout-engine static children and manually
  positioned absolute children now preserves author child order when committing `layout_in(...)`
  side effects.
- A preceding absolute anchor element can be resolved by a later `Anchored` sibling in the same
  layout pass. This keeps public declarative docking overlay-anchor parity from depending on
  retained bridge behavior or previous-frame geometry.
- The fix deliberately does not add a broad runtime current-bounds fallback for future siblings;
  same-frame visibility remains ordered by author order to avoid implicit two-way layout feedback.

Evidence:

- `crates/fret-ui/src/declarative/host_widget/layout.rs`
- `crates/fret-ui/src/declarative/tests/anchored.rs`
- `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
- `ecosystem/fret-docking/tests/public_surface_policy.rs`

Commands:

- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: workspace Rust formatting is clean after the layout-order fix and anchored test.
- `cargo nextest run -p fret-ui anchored_can_resolve_preceding_absolute_anchor_element_in_same_frame mechanism_harness_anchored_layout_invalidation_matches_oracles`
  - Result: passed, 2 tests.
  - Scope proven: the targeted regression proves a preceding absolute anchor is committed before a
    later `Anchored` sibling reads it in the same frame, and the anchored invalidation fixture
    suite remains green.
- `cargo nextest run -p fret-docking`
  - Result: passed, 89 tests.
  - Scope proven: public declarative docking, split, viewport, tab, drop, floating, runtime, and
    public-surface policy tests remain green with the retained bridge removed and the anchored
    layout-order fix in place.
- `cargo clippy -p fret-docking --all-targets --no-deps -- -D warnings`
  - Result: passed.
  - Scope proven: `fret-docking` all-targets remain warning-clean after the declarative host
    mechanism fix.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained-bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Known independent red gate:

- `cargo nextest run -p fret-ui declarative::tests::layout::mechanism_harness::mechanism_harness_layout_primitives_match_oracles`
  - Result: failed on `chrome-container-stretch-keeps-outer-box`.
  - Scope assessed: this is a flex/chrome stretch fixture that does not use `Anchored` placement or
    the mixed absolute/static fallback path changed in this follow-up. It should be tracked as a
    separate layout primitive drift task rather than as docking retained-bridge regression evidence.

## 2026-05-19 - RBX-M2-010 fret-node retained feature entry narrowed

Claim verified:

- `fret-node` no longer exposes a generic `compat-retained-bridge` feature alias.
- The only `fret-node` feature that enables `fret-ui/unstable-retained-bridge` is the concrete
  `compat-retained-canvas` compatibility island.
- Default declarative `fret-node` UI and headless graph surfaces still compile without the retained
  bridge, while the explicit retained-canvas compatibility island still compiles.

Evidence:

- `ecosystem/fret-node/Cargo.toml`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
- `tools/check_layering.py`

Commands:

- `python3 tools/audit_crate.py --crate fret-node`
  - Result: passed.
  - Scope proven: current `fret-node` audit snapshot identifies `fret-ui` as optional UI
    integration and `compat-retained-canvas` as the retained-bridge-backed delete-planned escape
    hatch.
- `cargo check -p fret-node --no-default-features --features headless`
  - Result: passed.
  - Scope proven: headless graph/schema/rules surfaces remain independent of `fret-ui` and retained
    bridge.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default declarative UI integration compiles without `compat-retained-canvas` or
    `fret-ui/unstable-retained-bridge`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the explicit retained canvas compatibility island still compiles after deleting
    the generic retained bridge alias.
- `cargo nextest run -p fret-node retained_compatibility_surface_stays_declarative_only`
  - Result: passed, 1 test.
  - Scope proven: `fret-node` surface policy rejects a public `compat-retained-bridge` alias and
    keeps retained compatibility attached to `compat-retained-canvas`.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: `fret-node` remains the only M2 allowlisted node-graph retained-bridge user, and
    no forbidden dependency direction was introduced.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Follow-up:

- The retained node graph canvas/editor implementation is still present behind
  `compat-retained-canvas`. Next M2 work should migrate chrome/overlays/panels to declarative
  composition and shrink the retained leaf toward canvas-only rendering and interaction, then remove
  `compat-retained-canvas` from first-party apps.

## 2026-05-19 - RBX-M2-020 first-party gallery node graph retained-canvas exit

Claim verified:

- `apps/fret-ui-gallery` no longer enables `fret-node/compat-retained-canvas`.
- UI Gallery's node graph cull torture page and AI workflow node graph demo now use the
  declarative `NodeGraphSurfaceBinding` plus `node_graph_surface(...)` path.
- The workflow demo's zoom/fit/reset controls still have explicit stage bounds through
  `LayoutQueryRegion`, replacing the retained `BoundsRecorder` widget.
- `fret-node` policy tests now prevent first-party gallery node graph pages from reintroducing
  retained canvas authoring.

Evidence:

- `apps/fret-ui-gallery/Cargo.toml`
- `apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs`
- `apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: workspace Rust formatting is clean after the gallery/node policy edits.
- `cargo check -p fret-ui-gallery --features gallery-dev`
  - Result: passed.
  - Scope proven: UI Gallery compiles with the full dev feature set after removing its
    `fret-node/compat-retained-canvas` dependency.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: declarative `fret-node` UI integration still compiles without the retained bridge.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the explicit retained canvas compatibility island still compiles for legacy
    callers while first-party gallery no longer enables it.
- `cargo nextest run -p fret-node workflow_gallery_surface_stays_binding_first_for_viewport_controls first_party_gallery_node_graph_pages_stay_off_retained_canvas retained_compatibility_surface_stays_declarative_only`
  - Result: passed, 3 tests.
  - Scope proven: `fret-node` policy coverage locks the gallery workflow/cull pages to declarative
    node graph surfaces, keeps viewport controls binding-first, and preserves the explicit retained
    compatibility island policy.
- `cargo nextest run -p fret-node`
  - Result: passed, 269 tests.
  - Scope proven: full `fret-node` graph/runtime/controller/declarative-surface tests remain green
    after moving the first-party gallery consumer off retained canvas.
- `rg -n "RetainedSubtreeProps|retained_bridge|NodeGraphCanvas::new|NodeGraphEditor::new|create_node_retained|retained_subtree|compat-retained-canvas" apps/fret-ui-gallery/Cargo.toml apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs`
  - Result: no matches.
  - Scope proven: the targeted UI Gallery node graph files and manifest no longer name retained
    bridge/canvas APIs or enable the retained canvas feature.
- `cargo tree -p fret-ui-gallery --features gallery-dev -e features -i fret-node | rg -n "compat-retained-canvas|fret-node feature|fret-ui-gallery|fret-node v"`
  - Result: passed; the `fret-node` feature path only lists `default`, `fret-ui`, and `kit`.
  - Scope proven: `gallery-dev` no longer enables `fret-node/compat-retained-canvas`.
- `cargo tree -p fret-ui-gallery --features gallery-dev -e features -i fret-ui | tail -60`
  - Result: passed; remaining `fret-ui/unstable-retained-bridge` activation is through
    `fret-chart`.
  - Scope proven: UI Gallery still has retained bridge exposure from the M3 chart path, so this M2
    slice correctly claims only node-graph retained-canvas exit rather than whole-gallery
    retained-bridge exit.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-020` is a first-party gallery consumer migration plus source-policy coverage;
    the gallery compile gate, `fret-node` declarative feature check, policy tests, layering, and
    no-retained-search gate cover the changed surface.

Follow-up:

- First-party `apps/fret-examples` still contains legacy retained node graph demos behind
  `node-graph-demos-legacy`. The next M2 slice should either migrate or explicitly quarantine
  those examples before removing `compat-retained-canvas` from `fret-node` itself.

## 2026-05-19 - RBX-M1-085 first-party docking examples declarative entry-point closure

Claim verified:

- First-party docking demos and the cookbook docking example no longer use deleted public retained
  docking entry points.
- They now install declarative `DockPanelElementRegistry` services and mount dock spaces through
  `dock_space_element_from_registry(...)` or `dock_space_declarative_with(...)`.
- A `fret-docking` policy test now prevents first-party docking examples from reintroducing the
  retained public entry points.

Evidence:

- `apps/fret-examples/src/docking_demo.rs`
- `apps/fret-examples/src/container_queries_docking_demo.rs`
- `apps/fret-examples/src/docking_arbitration_demo.rs`
- `apps/fret-examples/src/imui_editor_proof_demo.rs`
- `apps/fret-cookbook/examples/docking_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `ecosystem/fret-docking/tests/public_surface_policy.rs`
- `docs/crate-usage-guide.md`
- `docs/docking-arbitration-checklist.md`
- `docs/docking-imgui-parity-matrix.md`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`

Commands:

- `cargo check -p fret-demo --bin docking_demo`
  - Result: passed.
  - Scope proven: main docking demo compiles with declarative dock-space host and registry.
- `cargo check -p fret-demo --bin container_queries_docking_demo`
  - Result: passed.
  - Scope proven: container-query docking demo compiles with declarative dock-space host and
    retained-free docking public surface.
- `cargo check -p fret-demo --bin docking_arbitration_demo`
  - Result: passed.
  - Scope proven: arbitration/diagnostics demo compiles after moving dock host and panel registry
    off the deleted retained entry points while preserving external diagnostic anchors.
- `cargo check -p fret-demo --bin imui_editor_proof_demo`
  - Result: passed.
  - Scope proven: imui editor proof embeds docking through `dock_space_declarative_with(...)`.
- `cargo check -p fret-cookbook --features cookbook-docking --example docking_basics`
  - Result: passed.
  - Scope proven: public cookbook docking example teaches the declarative registry/host path.
- `cargo nextest run -p fret-docking public_docking_surface_prefers_declarative_entry_points retained_docking_entry_points_are_not_public first_party_docking_examples_use_declarative_entry_points`
  - Result: passed, 3 tests.
  - Scope proven: public declarative symbols remain exported, retained public entry points remain
    absent, and first-party examples stay on the declarative entry-point path.
- `rg -n "DockPanelFactory|DockPanelRegistryBuilder|DockPanelRegistryService|create_dock_space_node|mount_dock_space|render_and_bind_dock_panels|dock_space_with|DockSpaceImUiOptions" apps crates ecosystem docs tools --glob '!target/**' --glob '!docs/workstreams/**' --glob '!docs/audits/**'`
  - Result: matches only `ecosystem/fret-docking/tests/public_surface_policy.rs` forbidden-string
    assertions.
  - Scope proven: live source/docs no longer teach or call the deleted retained docking public
    entry points.
- `cargo test -p fret-cookbook --lib advanced_examples_use_the_explicit_advanced_surface`
  - Result: passed, 1 test.
  - Scope proven: cookbook source-policy assertions now expect the declarative docking surface.

Broader gates not run:

- `cargo nextest run -p fret-cookbook`
  - Reason: the package-wide no-run phase currently fails in an unrelated example
    (`apps/fret-cookbook/examples/hello_counter.rs` imports `fret::icons::icon` without enabling
    the `icons` feature). The targeted cookbook docking compile gate and source-policy unit test
    cover this change.

## 2026-05-19 - RBX-M2-030 first-party legacy node graph demo entry-point removal

Claim verified:

- First-party legacy retained node graph demo entry points were removed.
- The supported first-party node graph demo path is now `node_graph_demo` behind
  `node-graph-demos`.
- `fret-node` policy tests prevent first-party app/demo sources from reintroducing
  `node-graph-demos-legacy`, legacy module/bin names, or `fret-node/compat-retained-canvas`.

Evidence:

- `apps/fret-demo/Cargo.toml`
- `apps/fret-examples/Cargo.toml`
- `apps/fret-examples/src/lib.rs`
- `apps/fretboard/src/dev/native.rs`
- `ecosystem/fret-node/src/lib.rs`
- `tools/examples_source_tree_policy/gate.py`
- `tools/gate_imui_facade_teaching_source.py`
- `docs/examples/README.md`
- `docs/node-graph-roadmap.md`
- `docs/node-graph-xyflow-parity.md`

Commands:

- `cargo check -p fret-demo --features node-graph-demos --bin node_graph_demo`
  - Result: passed.
  - Scope proven: the supported node graph demo binary still compiles.
- `cargo check -p fret-examples --features node-graph-demos`
  - Result: passed.
  - Scope proven: first-party example library still compiles with the declarative node graph demo
    feature.
- `cargo nextest run -p fret-node first_party_node_graph_demos_stay_declarative_only retained_compatibility_surface_stays_declarative_only first_party_gallery_node_graph_pages_stay_off_retained_canvas`
  - Result: passed, 3 tests.
  - Scope proven: node graph surface policy rejects legacy first-party demo entry points and keeps
    retained compatibility explicit.
- `PYTHONPATH=tools python3 tools/examples_source_tree_policy/gate.py`
  - Result: passed.
  - Scope proven: examples source tree policy no longer expects the deleted IMUI node graph legacy
    source.
- `python3 tools/gate_imui_facade_teaching_source.py`
  - Result: passed.
  - Scope proven: IMUI teaching source remains inside the current facade policy after removing the
    old node graph exception.
- `rg -n "node-graph-demos-legacy|fret-node/compat-retained-canvas|node_graph_legacy_demo|node_graph_domain_demo|imui_node_graph_demo|node_graph_tuning_overlay" apps crates ecosystem tools docs --glob '!docs/workstreams/**' --glob '!docs/audits/**' --glob '!target/**'`
  - Result: matches only `ecosystem/fret-node/src/lib.rs` negative policy assertions.
  - Scope proven: first-party app/demo/tool/doc sources no longer expose the legacy retained node
    graph demo entry points.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: this slice removes first-party retained node graph demo entry points and migrates
    first-party docking examples exposed by the deletion; the targeted app checks, policy tests,
    source-policy gates, layering gate, and no-retained-search gates cover the changed surfaces.

## 2026-05-19 - RBX-M2-040 node graph declarative retained-subtree shim removal

Claim verified:

- `fret-node` no longer exposes `node_graph_surface_compat_retained(...)` or
  `NodeGraphSurfaceCompatRetainedProps` from its declarative node graph public surface.
- The declarative node graph surface no longer depends on `RetainedSubtreeProps`.
- The lower-level `compat-retained-canvas` feature still compiles, so this slice removes the public
  retained-subtree compatibility entry point without deleting the remaining retained canvas/editor
  implementation island prematurely.
- The existing declarative node graph, controller, runtime, graph model, and policy tests remain
  green after the public shim removal.

Evidence:

- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- Deleted: `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`

Commands:

- `rg -n "node_graph_surface_compat_retained|NodeGraphSurfaceCompatRetainedProps|compat_retained|RetainedSubtreeProps" ecosystem/fret-node/src ecosystem/fret-node/Cargo.toml --glob '!target/**'`
  - Result: matches only negative policy assertions in `ecosystem/fret-node/src/lib.rs`.
  - Scope proven: the removed retained-subtree compatibility symbols no longer exist in live
    `fret-node` UI implementation/export code.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean after the shim removal.
- `cargo nextest run -p fret-node retained_compatibility_surface_stays_declarative_only`
  - Result: passed, 1 test.
  - Scope proven: the public surface policy rejects declarative retained-subtree compatibility
    entry points and keeps retained compatibility out of `fret-node::ui::declarative`.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the declarative `fret-node` UI integration still compiles without
    `compat-retained-canvas` or `fret-ui/unstable-retained-bridge`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the explicit retained canvas compatibility island still compiles after deleting
    the public declarative retained-subtree shim.
- `cargo nextest run -p fret-node`
  - Result: passed, 269 tests.
  - Scope proven: full `fret-node` graph/runtime/controller/declarative-surface coverage remains
    green after the public shim removal.

- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid after deleting the
    public `fret-node` retained-subtree shim.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the task ledger/evidence
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-040` removes one `fret-node` public compatibility shim and tightens
    `fret-node` policy coverage; the package-wide `fret-node` nextest gate, both relevant
    `fret-node` feature checks, retained-symbol search, layering, and catalog gates cover the
    changed surface.

## 2026-05-19 - RBX-M2-050 node graph retained widget public surface quarantine

Claim verified:

- `fret-node::ui` no longer exposes retained node graph widget/editor/overlay/panel/portal modules
  or retained widget root re-exports as public API.
- The explicit `compat-retained-canvas` compatibility island still compiles and its retained canvas,
  editor, overlay, minimap, a11y, middleware, paint-cache, and skin conformance matrix remains green.
- The default declarative `fret-node` UI surface still compiles and its package tests remain green
  without enabling `compat-retained-canvas`.
- First-party source no longer consumes the removed retained widget exports through
  `fret_node::ui`.

Evidence:

- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/middleware.rs`
- `ecosystem/fret-node/src/ui/editors/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- retained conformance test imports under `ecosystem/fret-node/src/ui/canvas/widget/tests/`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default declarative `fret-node` UI integration compiles without the retained
    canvas compatibility feature or `fret-ui/unstable-retained-bridge`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the explicit retained canvas compatibility island still compiles after removing
    public retained widget module/export access.
- `cargo nextest run -p fret-node`
  - Result: passed, 269 tests.
  - Scope proven: default graph/runtime/controller/declarative-surface coverage remains green after
    the public retained widget quarantine.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 906 tests.
  - Scope proven: retained canvas/editor/overlay/minimap/a11y/middleware/paint-cache/skin behavior
    remains covered by the compatibility island after the public exports were removed.
- `rg -n "fret_node::ui::(NodeGraphCanvas|NodeGraphCanvasWith|NodeGraphEditor|NodeGraphPanel|NodeGraphPortalHost|NodeGraphOverlayHost)|use fret_node::ui::\\{[^\\n]*(NodeGraphCanvas|NodeGraphCanvasWith|NodeGraphEditor|NodeGraphPanel|NodeGraphPortalHost|NodeGraphOverlayHost)" apps crates ecosystem tools docs --glob '!target/**' --glob '!docs/workstreams/**'`
  - Result: no matches.
  - Scope proven: first-party source outside the workstream notes does not consume the removed
    public retained widget exports.
- `rg -n "pub use (canvas|editor|editors|overlays|panel|portal)::|pub mod (canvas|a11y|diag_anchors|editor|editors|overlays|panel|portal);|NodeGraphSurfaceCompatRetainedProps|node_graph_surface_compat_retained|RetainedSubtreeProps" ecosystem/fret-node/src/ui ecosystem/fret-node/src/lib.rs ecosystem/fret-node/Cargo.toml --glob '!target/**'`
  - Result: matches only negative policy assertions in `ecosystem/fret-node/src/lib.rs` plus the
    still-public non-retained `NodeResizeHandle` export.
  - Scope proven: the removed retained widget module/export surface and the earlier declarative
    retained-subtree shim are absent from live `fret-node` UI exports.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist still pass after the quarantine.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the task ledger/evidence
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-050` changes the `fret-node` public surface and retained compatibility island
    imports only. The default `fret-node` package gate, the full `compat-retained-canvas` package
    gate, both relevant feature checks, public retained-symbol searches, layering, catalog, and
    whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-060 node graph overlay/panel policy default gate extraction

Claim verified:

- Node graph overlay/panel/screen-space pure policy and layout modules compile under the default
  declarative `fret-ui` feature without enabling `compat-retained-canvas` or
  `fret-ui/unstable-retained-bridge`.
- Retained overlay widget/paint modules remain gated behind `compat-retained-canvas`.
- Default `fret-node` test coverage now includes overlay/panel/minimap/toolbar/blackboard/rename
  and screen-space placement policy tests, so those behaviors are no longer protected only by the
  retained compatibility island.
- The full retained canvas compatibility behavior matrix still passes after the extraction.

Evidence:

- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/screen_space_placement.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
- `ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_policy.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default declarative `fret-node` UI compiles with overlay/panel policy modules
    available and without the retained bridge feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the explicit retained compatibility island still compiles after the policy module
    extraction.
- `cargo nextest run -p fret-node overlay_policy_modules_compile_without_retained_canvas_compat`
  - Result: passed, 1 test.
  - Scope proven: policy coverage locks `overlays` and `screen_space_placement` into the default
    declarative UI path while keeping retained overlay widget modules gated.
- `cargo nextest run -p fret-node`
  - Result: passed, 319 tests.
  - Scope proven: default `fret-node` coverage now includes 50 overlay/panel/screen-space policy
    tests that previously only ran with `compat-retained-canvas`.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 906 tests.
  - Scope proven: retained canvas/editor/overlay behavior coverage remains green after moving pure
    overlay policy modules into the default gate.
- `cargo fmt --check`
  - Result: passed after applying rustfmt.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist still pass after moving overlay
    policy modules out of the retained compatibility feature gate.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the task ledger/evidence
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-060` changes only `fret-node` UI module gating and overlay/panel policy
    coverage. The default `fret-node` package gate, the full `compat-retained-canvas` package gate,
    both feature checks, layering, catalog, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-070 node graph portal editor chrome default gate extraction

Claim verified:

- Portal editor chrome helpers compile under the default declarative `fret-ui` feature without
  enabling `compat-retained-canvas` or `fret-ui/unstable-retained-bridge`.
- Retained portal text/number editor command handlers remain gated behind `compat-retained-canvas`.
- Default `fret-node` test coverage now includes portal editor chrome tests.
- The full retained canvas compatibility behavior matrix still passes after the extraction.

Evidence:

- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/editors/mod.rs`
- `ecosystem/fret-node/src/ui/editors/chrome.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default declarative `fret-node` UI compiles with editor chrome available and
    without the retained bridge feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: retained portal text/number editor command handlers still compile inside the
    explicit compatibility island.
- `cargo nextest run -p fret-node editor_chrome_compiles_without_retained_canvas_compat ui::editors::chrome`
  - Result: passed, 4 tests.
  - Scope proven: editor chrome tests run in the default gate and policy coverage keeps retained
    portal editor modules gated.
- `cargo nextest run -p fret-node`
  - Result: passed, 324 tests.
  - Scope proven: default `fret-node` coverage now includes portal editor chrome tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 908 tests.
  - Scope proven: retained canvas/editor/overlay behavior coverage remains green after moving
    editor chrome into the default gate.

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist still pass after moving editor chrome
    out of the retained compatibility feature gate.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the task ledger/evidence
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-070` changes only `fret-node` UI module gating and editor chrome default
    coverage. The default `fret-node` package gate, the full `compat-retained-canvas` package gate,
    both feature checks, layering, catalog, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-080 node graph retained capability ledger and source usage gate

Claim verified:

- The remaining `fret-node` retained bridge usage is recorded as an explicit
  `compat-retained-canvas` migration ledger rather than a public authoring path.
- A new source-policy test fails if code-level retained bridge usage spreads outside the current
  retained migration ledger.
- Default declarative `fret-node` coverage and the full retained compatibility oracle both remain
  green after adding the gate and ledger.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 1 test.
  - Scope proven: the new source-policy gate recognizes only the explicit retained migration ledger.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default declarative `fret-node` UI compiles without the retained bridge feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained compatibility island still compiles.
- `cargo nextest run -p fret-node`
  - Result: passed, 325 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes the retained source
    usage ledger gate.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 909 tests.
  - Scope proven: the retained canvas/editor/overlay oracle remains green after adding the ledger
    gate and documentation.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after adding the new audit note.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.
- `rg -l "use fret_ui::retained_bridge|use fret_ui::\\{UiHost, retained_bridge|fret_ui::retained_bridge::|RetainedSubtreeProps|UiTreeRetainedExt" ecosystem/fret-node/src/ui -g '*.rs' | sort | wc -l`
  - Result: 175 files.
  - Scope proven: the retained oracle is still substantial and must be replaced family-by-family;
    it is now tracked by the ledger instead of treated as a hidden default-path dependency.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-080` is a `fret-node` source-policy/audit slice. The default `fret-node`
    package gate, the full `compat-retained-canvas` package gate, both feature checks, layering,
    catalog, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-085 portal command protocol default gate

Claim verified:

- Portal submit/cancel/step command IDs, parsing, step modes, and command outcomes now live in a
  default-gated protocol module instead of being owned by the retained portal host module.
- The retained `NodeGraphPortalHost` and retained portal text/number command handlers remain inside
  the explicit `compat-retained-canvas` island and consume the protocol through re-exports.
- Default declarative `fret-node` coverage includes protocol roundtrip and malformed-command tests,
  and the full retained compatibility oracle remains green after the extraction.

Evidence:

- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/portal_commands.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node portal_text_command_protocol`
  - Result: passed, 2 tests.
  - Scope proven: default-gated portal command builders/parsers roundtrip valid submit/cancel/step
    commands and reject malformed commands.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the portal command protocol compiles on the default declarative `fret-ui` path
    without enabling `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: retained portal host and editor handlers still compile while consuming the
    extracted protocol.
- `cargo nextest run -p fret-node`
  - Result: passed, 327 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes portal command
    protocol tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 911 tests.
  - Scope proven: retained canvas/editor/overlay behavior coverage remains green after the protocol
    extraction.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-085` changes only `fret-node` portal command protocol ownership. The default
    package gate, the full retained compatibility oracle, both feature checks, layering, catalog,
    formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-090 portal editor command policy default gate

Claim verified:

- Portal text/number submit/cancel/step decision policy now lives in a default-gated editor policy
  module instead of being owned by retained `CommandCx` handlers.
- Retained `portal_text.rs` and `portal_number.rs` remain behind `compat-retained-canvas`, but now
  act as session/model I/O adapters that consume default policy plans.
- Default declarative `fret-node` coverage includes text and number command policy tests, while the
  retained portal/compatibility oracle remains green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/editors/mod.rs`
- `ecosystem/fret-node/src/ui/editors/portal_command_policy.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node portal_command_policy editor_chrome_compiles_without_retained_canvas_compat`
  - Result: passed, 3 tests.
  - Scope proven: portal text/number command policy compiles and is tested on the default gate, and
    the editor policy surface remains outside `compat-retained-canvas`.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the extracted portal command policy compiles without retained bridge features.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: retained portal text/number handlers still compile after being converted into
    policy consumers.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal`
  - Result: passed, 28 tests.
  - Scope proven: retained portal lifecycle/keyboard/pointer/measured conformance and default
    portal command policy tests remain green together.
- `cargo nextest run -p fret-node`
  - Result: passed, 329 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes portal command policy
    tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 913 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after the portal
    command policy extraction.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed Rust and documentation files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-090` changes only `fret-node` editor command policy extraction and retained
    portal handler adapters. The default package gate, retained compatibility package gate,
    targeted portal compat gate, both feature checks, layering, catalog, formatting, and whitespace
    gates cover the changed surface.

## 2026-05-19 - RBX-M2-095 portal editor command session default gate

Claim verified:

- Portal text/number session command application now lives in a default-gated editor session module
  instead of being owned by retained `CommandCx` handlers.
- Retained `portal_text.rs` and `portal_number.rs` remain behind `compat-retained-canvas`, but their
  command handlers now provide retained model/session I/O adapters around the default command
  policy and session application.
- Default declarative `fret-node` coverage proves text/number cancel, submit, parse/error,
  normalization, and commit outcomes can be applied without retained `CommandCx`, while the retained
  portal/compatibility oracle remains green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/editors/mod.rs`
- `ecosystem/fret-node/src/ui/editors/portal_command_session.rs`
- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node without_retained_command_cx editor_chrome_compiles_without_retained_canvas_compat`
  - Result: passed, 3 tests.
  - Scope proven: portal text/number session command application compiles and is tested on the
    default gate, and the editor policy/session surface remains outside `compat-retained-canvas`.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the extracted portal command session adapter compiles without retained bridge
    features.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: retained portal text/number handlers still compile after being converted into
    session I/O adapters.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal`
  - Result: passed, 30 tests.
  - Scope proven: retained portal lifecycle/keyboard/pointer/measured conformance, default portal
    command policy tests, and default portal command session tests remain green together.
- `cargo nextest run -p fret-node`
  - Result: passed, 331 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes portal command
    session tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 915 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after the portal
    command session extraction.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/editors/portal_command_session.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked portal command session file has no whitespace errors before
    staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-095` changes only `fret-node` editor command session extraction and retained
    portal handler adapters. The default package gate, retained compatibility package gate,
    targeted portal compat gate, both feature checks, layering, catalog, formatting, and whitespace
    gates cover the changed surface.

## 2026-05-19 - RBX-M2-100 controls overlay declarative composition default gate

Claim verified:

- Controls overlay composition now has a default-gated declarative element tree that does not
  construct the retained `NodeGraphControlsOverlay` widget.
- The declarative controls tree preserves the retained overlay's static authoring capability:
  panel sizing, six-button roster/order, stable button labels, a11y labels, test IDs, command
  binding enabled/disabled state, and activation command dispatch hooks.
- The retained controls widget remains behind `compat-retained-canvas` as the oracle for the
  remaining pointer, keyboard, hover, focus, and retained paint conformance until those interaction
  families have default declarative coverage.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_declarative`
  - Result before implementation: failed, 2 tests.
  - Scope proven: the new tests were real red tests against the empty declarative controls stub.
- `cargo nextest run -p fret-node controls_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 5 tests.
  - Scope proven: declarative controls composition, default overlay module gating, and
    retained-dependency source policy are covered by targeted tests, including activation command
    dispatch and disabled-command suppression.
- `cargo nextest run -p fret-node`
  - Result: passed, 334 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes declarative controls
    composition tests.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: declarative controls composition compiles without enabling
    `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained controls widget and remaining retained island still compile beside
    the new declarative controls composition.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 918 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after adding the
    default declarative controls composition.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative controls composition file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-100` changes only `fret-node` controls overlay declarative composition and
    source-policy tests. The default package gate, retained compatibility package gate, both
    feature checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-105 blackboard overlay declarative composition default gate

Claim verified:

- Blackboard overlay composition now has a default-gated declarative element tree that does not
  construct the retained `NodeGraphBlackboardOverlay` widget.
- The declarative blackboard tree preserves the retained overlay's composition and activation
  handoff capability: panel sizing, sorted symbol rows, stable visible labels, root semantics,
  action a11y labels/test IDs, and a mechanism-only pointer activation hook that yields
  `BlackboardAction`.
- The retained blackboard widget remains behind `compat-retained-canvas` as the oracle for
  transaction submission, rename sessions, keyboard/focus navigation, pointer hover/press state,
  and retained paint conformance until those interaction families have default declarative coverage.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node blackboard_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 5 tests.
  - Scope proven: declarative blackboard composition, action-hook activation, default overlay module
    gating, and retained-dependency source policy are covered by targeted tests.
- `cargo nextest run -p fret-node`
  - Result: passed, 337 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes declarative
    blackboard composition/action-hook tests.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: declarative blackboard composition compiles without enabling
    `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained blackboard widget and remaining retained island still compile beside
    the new declarative blackboard composition.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 921 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after adding the
    default declarative blackboard composition.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative blackboard composition file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-105` changes only `fret-node` blackboard overlay declarative composition and
    source-policy tests. The default package gate, retained compatibility package gate, both
    feature checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-106 minimap overlay declarative composition default gate

Claim verified:

- Minimap overlay composition now has a default-gated declarative element tree that does not
  construct the retained `NodeGraphMiniMapOverlay` widget.
- The declarative minimap tree preserves the retained overlay's static composition and paint-plan
  capability: fixed panel sizing, root panel semantics, stable `node_graph.minimap` test ID,
  declarative canvas child, panel quad, projected node markers, and viewport marker.
- The retained minimap widget remains behind `compat-retained-canvas` as the oracle for keyboard
  pan/zoom/focus, pointer drag panning, focus/capture propagation, retained hit testing, and
  store/controller viewport updates until those interaction families have default declarative
  coverage.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node minimap_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 4 tests.
  - Scope proven: declarative minimap composition, minimap paint-plan ops, default overlay module
    gating, and retained-dependency source policy are covered by targeted tests.
- `cargo nextest run -p fret-node`
  - Result: passed, 339 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes declarative minimap
    composition and paint-plan tests.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: declarative minimap composition compiles without enabling
    `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained minimap widget and remaining retained island still compile beside
    the new declarative minimap composition.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 923 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after adding the
    default declarative minimap composition.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative minimap composition file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-106` changes only `fret-node` minimap overlay declarative composition and
    source-policy tests. The default package gate, retained compatibility package gate, both
    feature checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-107 toolbar overlay declarative composition default gate

Claim verified:

- Node and edge toolbar placement now has a default-gated declarative element tree that does not
  construct the retained `NodeGraphNodeToolbar` / `NodeGraphEdgeToolbar` widgets.
- The declarative toolbar trees preserve the retained overlay's placement and composition
  capability: retained-compatible anchor planning, `WhenSelected` / `Always` visibility,
  absolute toolbar layout, semantics/test IDs, and passthrough declarative children.
- The retained toolbar widgets remain behind `compat-retained-canvas` as the oracle for child
  measurement, retained child-root layout/paint, hit testing, and model/internals-driven target
  resolution until those families have default declarative coverage.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node toolbars_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 6 tests.
  - Scope proven: declarative toolbar composition, retained-compatible placement planning, default
    overlay module gating, and retained-dependency source policy are covered by targeted tests.
- `cargo nextest run -p fret-node`
  - Result: passed, 343 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes declarative toolbar
    placement/composition tests.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: declarative toolbar composition compiles without enabling
    `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained toolbar widgets and remaining retained island still compile beside
    the new declarative toolbar composition.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 927 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after adding the
    default declarative toolbar composition.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative toolbar composition file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-107` changes only `fret-node` toolbar overlay declarative composition and
    source-policy tests. The default package gate, retained compatibility package gate, both
    feature checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-108 rename overlay declarative composition default gate

Claim verified:

- Inline rename overlay composition now has a default-gated declarative element tree that does not
  construct the retained `NodeGraphOverlayHost` widget.
- The declarative rename tree preserves the retained overlay's static composition and command
  wiring capability: shared rename host layout planning, hidden/no-session behavior, group and
  symbol root/input semantics, stable test IDs, caller-owned text-model preservation, and
  submit/cancel command protocol roundtrips.
- The retained rename host remains behind `compat-retained-canvas` as the oracle for seed-text
  ownership, focus-loss close, focus request/restore, keyboard submit/cancel event routing,
  graph/edit queue transaction submission, blackboard rename handoff, and retained paint/hit
  testing until those families have default declarative coverage.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node rename_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 6 tests.
  - Scope proven: declarative rename composition, submit/cancel command protocol, default overlay
    module gating, and retained-dependency source policy are covered by targeted tests.
- `cargo nextest run -p fret-node rename_declarative minimap_declarative toolbars_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 12 tests.
  - Scope proven: the current default overlay composition set for minimap, toolbars, and rename is
    mutually compatible and remains outside the retained canvas compatibility gate.
- `cargo nextest run -p fret-node`
  - Result: passed, 347 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes declarative rename
    composition and command protocol tests.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: declarative rename composition compiles without enabling
    `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained rename host and remaining retained island still compile beside the
    new declarative rename composition.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 931 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after adding the
    default declarative rename composition.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative minimap composition file has no whitespace errors
    before staging.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative toolbar composition file has no whitespace errors
    before staging.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative rename composition file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-108` changes only `fret-node` overlay declarative composition and source-policy
    tests. The default package gate, retained compatibility package gate, both feature checks,
    layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-109 rename command/session application default gate

Claim verified:

- Rename submit/cancel command parsing, keyboard submit/cancel decision, and active-session
  application now live on the default overlay gate in `rename_command.rs` without constructing the
  retained `NodeGraphOverlayHost` widget.
- The default rename command/session policy can reject malformed commands, ignore stale-session
  submit/cancel requests, close matching active sessions, and return a `GraphTransaction` for
  active group/symbol rename commits.
- The retained rename host remains behind `compat-retained-canvas` as a model I/O and
  controller/edit-queue submission adapter. Its retained oracle coverage still proves Enter/Escape,
  focus-loss close, controller-backed commit, hit-test transparency, and focus restoration.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_command.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node rename_command rename_declarative rename_host_event overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 10 tests.
  - Scope proven: default rename command/session application, declarative rename command wiring,
    retained-host key decision coverage, default overlay module gating, and retained-dependency
    source policy are covered by targeted tests.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: rename command/session policy compiles without enabling `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained rename host and remaining retained island still compile after
    delegating close/commit decisions to default rename command/session policy.
- `cargo nextest run -p fret-node --features compat-retained-canvas rename_command rename_host_event overlay_group_rename_conformance`
  - Result: passed, 10 tests.
  - Scope proven: retained rename oracle behavior remains green for Escape close/focus restore,
    Enter commit, controller-backed commit, focus-loss close, hit testing, and stale-session
    replacement after the default policy extraction.
- `cargo nextest run -p fret-node`
  - Result: passed, 350 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes default rename
    command/session application tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 934 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after extracting
    rename command/session policy onto the default gate.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative minimap composition file has no whitespace errors
    before staging.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative toolbar composition file has no whitespace errors
    before staging.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_declarative.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked declarative rename composition file has no whitespace errors
    before staging.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_command.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked default rename command/session policy file has no whitespace
    errors before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-109` changes only `fret-node` overlay command/session policy plus retained
    rename I/O adaptation. The default package gate, retained compatibility package gate, both
    feature checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-110 rename lifecycle planning default gate

Claim verified:

- Rename seed-text ownership, first-open focus request, focus-loss close planning, and focus
  restoration planning now live in default-gated `rename_lifecycle.rs`.
- The retained `NodeGraphOverlayHost` consumes the default lifecycle plan and remains a retained
  model/tree I/O adapter instead of owning those lifecycle decisions.
- Existing retained rename and blackboard handoff oracle behavior remains green after the
  lifecycle extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node rename_lifecycle rename_host_event rename_command rename_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 15 tests.
  - Scope proven: default rename lifecycle, command/session policy, declarative rename wiring,
    retained-host key decision coverage, default overlay module gating, and retained-dependency
    source policy are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas rename_lifecycle rename_host_event overlay_group_rename_conformance overlay_symbol_rename_conformance overlay_blackboard_conformance`
  - Result: passed, 26 tests.
  - Scope proven: retained rename oracle behavior remains green for group/symbol seed/focus,
    focus-loss close, Enter/Escape close/commit, controller-backed commit, hit-test transparency,
    and blackboard rename handoff after the retained host starts consuming default lifecycle
    policy.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: rename lifecycle planning compiles without enabling `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained rename host and remaining retained island still compile after
    delegating lifecycle decisions to default rename lifecycle policy.
- `cargo nextest run -p fret-node`
  - Result: passed, 355 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes default rename
    lifecycle tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 939 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after extracting
    rename lifecycle planning onto the default gate.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked default rename lifecycle policy file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-110` changes only `fret-node` overlay lifecycle policy plus retained rename
    I/O adaptation. The default package gate, retained compatibility package gate, both feature
    checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-111 minimap interaction planning default gate

Claim verified:

- Minimap keyboard pan/zoom/focus decisions now live in default-gated
  `minimap_interaction_policy.rs`.
- Minimap pointer down/up drag-start, focus, capture, propagation, repaint, release, and finish
  planning now live in default-gated `minimap_interaction_policy.rs`.
- The retained minimap widget consumes the default interaction plans and remains a retained
  store/view-state I/O adapter plus retained event side-effect adapter.
- Existing retained minimap/control oracle behavior remains green after the interaction extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node minimap_interaction_policy minimap_drag_policy minimap_policy overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 11 tests.
  - Scope proven: default minimap keyboard interaction planning, pointer drag policy composition,
    lower-level minimap policy, default overlay module gating, and retained-dependency source
    policy are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas minimap_interaction_policy overlay_minimap_controls_conformance`
  - Result: passed, 19 tests.
  - Scope proven: retained minimap/control oracle behavior remains green for minimap layout, marker
    projection, keyboard panning/zooming/focus return, pointer drag panning, pointer capture, and
    controls overlay conformance after the retained widget starts consuming default interaction
    policy.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: minimap interaction planning compiles without enabling `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained minimap widget and remaining retained island still compile after
    delegating keyboard/pointer interaction decisions to default minimap interaction policy.
- `cargo nextest run -p fret-node`
  - Result: passed, 359 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes default minimap
    interaction tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 943 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after extracting
    minimap interaction planning onto the default gate.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked default minimap interaction policy file has no whitespace
    errors before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-111` changes only `fret-node` overlay interaction policy plus retained minimap
    I/O adaptation. The default package gate, retained compatibility package gate, both feature
    checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-19 - RBX-M2-112 toolbar layout/hit-test planning default gate

Claim verified:

- Toolbar visible-target filtering, node/edge child rect planning, empty-size hiding, and
  child-bound hit testing now live in default-gated `toolbar_layout_policy.rs`.
- Retained node/edge toolbar widgets consume the default layout/hit-test policy and remain retained
  target/model I/O, child measurement, `layout_in`, and child-root paint adapters.
- Declarative toolbar composition reuses the same default layout policy instead of carrying a
  separate rect/visibility implementation.
- Existing retained toolbar oracle behavior remains green after the layout/hit-test extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_layout.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node toolbar_layout_policy toolbars_declarative toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 15 tests.
  - Scope proven: default toolbar visible-target filtering, node/edge rect planning, empty-size
    hiding, child-bound hit testing, declarative toolbar composition, base toolbar policy, default
    overlay module gating, and retained-dependency source policy are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas toolbar_layout_policy toolbars_declarative overlay_toolbars_conformance`
  - Result: passed, 11 tests.
  - Scope proven: retained toolbar oracle behavior remains green for node/edge pointer fallthrough,
    focus release when hidden, and declarative/default toolbar planning after the retained widget
    starts consuming default layout/hit-test policy.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: toolbar layout/hit-test policy compiles without enabling
    `compat-retained-canvas`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: retained toolbar widgets and the remaining retained island still compile after
    delegating layout/hit-test decisions to default toolbar layout policy.
- `cargo nextest run -p fret-node`
  - Result: passed, 363 tests.
  - Scope proven: default `fret-node` coverage remains green and now includes default toolbar
    layout/hit-test tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 942 tests.
  - Scope proven: the full retained canvas/editor/overlay oracle remains green after extracting
    toolbar layout/hit-test planning onto the default gate.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting is clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog metadata still indexes cleanly after the new evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked Rust and documentation changes have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked default toolbar layout policy file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-112` changes only `fret-node` toolbar overlay layout/hit-test policy plus
    retained toolbar I/O adaptation. The default package gate, retained compatibility package gate,
    both feature checks, layering, catalog, formatting, and whitespace gates cover the changed
    surface.

## 2026-05-19 - RBX-M2-113 controls overlay interaction planning default gate

Claim verified:

- Controls overlay keyboard select/activate/focus-canvas planning and pointer
  hover/down/up focus/capture/repaint/activation planning now live in default-gated
  `controls_interaction_policy.rs`.
- Retained `NodeGraphControlsOverlay` consumes the default interaction plans and remains a retained
  side-effect adapter for focus, cursor, pointer capture, repaint completion, and command dispatch.
- Existing retained controls/minimap oracle behavior remains green after the interaction extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_interaction_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/controls.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_interaction_policy controls_declarative controls_layout controls_policy panel_navigation_policy panel_pointer_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 25 tests.
  - Scope proven: default controls keyboard/hover/pointer interaction planning, declarative
    controls composition, retained-independent layout/policy state helpers, default overlay module
    gating, and retained-dependency source policy are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
  - Result: passed, 22 tests.
  - Scope proven: retained controls/minimap oracle behavior remains green for controls pointer
    fallthrough/blocking, pointer focus, keyboard navigation/activation, command binding overrides,
    Escape focus return, active-descendant semantics, and tab traversal after the retained controls
    widget starts consuming default interaction policy.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default-gated overlay policy modules, including controls interaction
    planning, compile without the retained canvas compatibility feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after retained controls
    event handling was reduced to applying default interaction plans.
- `cargo nextest run -p fret-node`
  - Result: passed, 367 tests.
  - Scope proven: default `fret-node` behavior remains green, including default controls
    interaction policy, declarative controls composition, overlay policy/layout helpers, and
    retained-independent surface policy gates.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 946 tests.
  - Scope proven: the retained canvas compatibility oracle remains green after the controls
    overlay starts consuming default interaction plans, covering retained controls/minimap
    conformance, retained canvas/editor behavior, and default policy tests under the compatibility
    feature set.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the overlay policy extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_interaction_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked controls interaction policy file has no whitespace errors
    before staging.

Additional new-file whitespace gates:

- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs 2>&1); test -z "$out"`
  - Result: passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs 2>&1); test -z "$out"`
  - Result: passed.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-113` changes only `fret-node` controls overlay interaction policy plus
    retained controls side-effect adaptation. The default package gate, retained compatibility
    package gate, both feature checks, layering, catalog, formatting, and whitespace gates cover
    the changed surface.

## 2026-05-20 - RBX-M2-114 blackboard overlay interaction planning default gate

Claim verified:

- Blackboard overlay keyboard select/activate/focus-canvas planning and pointer
  hover/down/up focus/capture/repaint/activation planning now live in default-gated
  `blackboard_interaction_policy.rs`.
- Retained `NodeGraphBlackboardOverlay` consumes the default interaction plans and remains a
  retained side-effect adapter for focus, cursor, pointer capture, repaint, and
  transaction/rename dispatch.
- Existing retained blackboard oracle behavior remains green after the interaction extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_interaction_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node blackboard_interaction_policy blackboard_declarative blackboard_layout blackboard_policy panel_navigation_policy panel_pointer_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat`
  - Result: passed, 23 tests.
  - Scope proven: default blackboard keyboard/hover/pointer interaction planning, declarative
    blackboard composition, retained-independent layout/policy state helpers, and default overlay
    module gating are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_interaction_policy blackboard_declarative overlay_blackboard_conformance`
  - Result: passed, 17 tests.
  - Scope proven: retained blackboard oracle behavior remains green for hit-test transparency,
    Enter default activation, pointer activation, controller-over-edit-queue dispatch, symbol ref
    insertion, delete ordering, rename overlay opening, rename commit/cancel, and unchanged-rename
    close behavior after the retained blackboard widget starts consuming default interaction
    policy.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default-gated overlay policy modules, including blackboard interaction planning,
    compile without the retained canvas compatibility feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after retained blackboard
    event handling was reduced to applying default interaction plans.
- `cargo nextest run -p fret-node`
  - Result: passed, 371 tests.
  - Scope proven: default `fret-node` behavior remains green, including default blackboard
    interaction policy, declarative blackboard composition, overlay policy/layout helpers, and
    retained-independent surface policy gates.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 950 tests.
  - Scope proven: the retained canvas compatibility oracle remains green after the blackboard
    overlay starts consuming default interaction plans, covering retained blackboard conformance,
    retained canvas/editor behavior, and default policy tests under the compatibility feature set.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the overlay policy extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_interaction_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked blackboard interaction policy file has no whitespace errors
    before staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-114` changes only `fret-node` blackboard overlay interaction policy plus
    retained blackboard side-effect adaptation. The default package gate, retained compatibility
    package gate, both feature checks, layering, catalog, formatting, and whitespace gates cover
    the changed surface.

## 2026-05-20 - RBX-M2-115 blackboard paint planning default gate

Claim verified:

- Blackboard panel/button/label paint ordering, text constraints, active-action background
  selection, and missing-symbol label fallback now live in default-gated
  `blackboard_paint_plan.rs`.
- Retained `blackboard_paint.rs` consumes the default paint plan and remains a retained
  `PaintCx`/text-blob/scene-op adapter.
- Existing retained blackboard oracle behavior remains green after the retained paint decision
  extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_paint_plan.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_paint.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node blackboard_paint_plan blackboard_layout blackboard_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat`
  - Result: passed, 14 tests.
  - Scope proven: default blackboard paint planning, layout/policy dependencies, active-state
    background selection, missing-symbol fallback, text constraints, and default overlay module
    gating are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_paint_plan blackboard_declarative overlay_blackboard_conformance blackboard_paint`
  - Result: passed, 16 tests.
  - Scope proven: retained blackboard oracle behavior remains green after retained paint consumes
    the default plan, including hit-test transparency, activation/dispatch paths, rename
    commit/cancel behavior, and default/declarative blackboard coverage under the compatibility
    feature set.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default-gated overlay policy modules, including blackboard paint planning,
    compile without the retained canvas compatibility feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after retained blackboard
    paint was reduced to applying default paint plans.
- `cargo nextest run -p fret-node`
  - Result: passed, 374 tests.
  - Scope proven: default `fret-node` behavior remains green, including default blackboard paint
    planning, default overlay policies, and retained-independent surface policy gates.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 950 tests.
  - Scope proven: the retained canvas compatibility oracle remains green after blackboard retained
    paint starts consuming default paint plans, covering retained blackboard conformance, retained
    canvas/editor behavior, and default policy tests under the compatibility feature set.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the paint-plan extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_paint_plan.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked blackboard paint plan file has no whitespace errors before
    staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-115` changes only `fret-node` blackboard paint planning plus retained
    blackboard paint adaptation. The default package gate, retained compatibility package gate,
    both feature checks, layering, catalog, formatting, and whitespace gates cover the changed
    surface.

## 2026-05-20 - RBX-M2-116 controls paint planning default gate

Claim verified:

- Controls panel/button paint ordering, text constraints, connection-mode labels, pressed/hovered
  / keyboard-active background selection, and focus-gated keyboard highlight rules now live in
  default-gated `controls_paint_plan.rs`.
- Retained `NodeGraphControlsOverlay::paint` consumes the default paint plan and remains a retained
  `PaintCx`/text-blob/scene-op adapter.
- Existing retained controls/minimap oracle behavior remains green after the retained paint
  decision extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_paint_plan.rs`
- `ecosystem/fret-node/src/ui/overlays/controls.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_paint_plan`
  - Result: passed, 3 tests.
  - Scope proven: the new default controls paint plan compiles and its panel/button/label/state
    coverage runs before retained paint integration.
- `cargo nextest run -p fret-node controls_paint_plan controls_layout controls_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 16 tests.
  - Scope proven: default controls paint planning, layout/policy dependencies, active-state
    background selection, connection-mode labels, text constraints, source policy, and default
    overlay module gating are covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_paint_plan controls_declarative overlay_minimap_controls_conformance`
  - Result: passed, 21 tests.
  - Scope proven: retained controls/minimap oracle behavior remains green after retained controls
    paint consumes the default plan, including controls pointer fallthrough/blocking, pointer
    focus, keyboard navigation/activation, command binding overrides, Escape focus return,
    active-descendant semantics, tab traversal, and default/declarative controls coverage under
    the compatibility feature set.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default-gated overlay policy modules, including controls paint planning, compile
    without the retained canvas compatibility feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after retained controls
    paint was reduced to applying default paint plans.
- `cargo nextest run -p fret-node`
  - Result: passed, 377 tests.
  - Scope proven: default `fret-node` behavior remains green, including default controls paint
    planning, default overlay policies, and retained-independent surface policy gates.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 953 tests.
  - Scope proven: the retained canvas compatibility oracle remains green after controls retained
    paint starts consuming default paint plans, covering retained controls/minimap conformance,
    retained canvas/editor behavior, and default policy tests under the compatibility feature set.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the paint-plan extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_paint_plan.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked controls paint plan file has no whitespace errors before
    staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-116` changes only `fret-node` controls paint planning plus retained controls
    paint adaptation. The default package gate, retained compatibility package gate, both feature
    checks, layering, catalog, formatting, and whitespace gates cover the changed surface.

## 2026-05-20 - RBX-M2-117 controls host planning default gate

Claim verified:

- Controls panel hit-testing and pointer-down host side-effect planning now live in default-gated
  `controls_host_policy.rs`.
- Retained `NodeGraphControlsOverlay` consumes the default host policy for panel hit-testing and
  pointer-down focus, propagation, capture, and repaint decisions.
- Declarative controls composition uses a `PointerRegion` to handle blank panel pointer-downs
  without stealing descendant `Pressable` activation.
- Existing retained controls/minimap oracle behavior remains green after the controls host policy
  extraction.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_host_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/controls.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_declarative_panel_blank_pointer_down_focuses_overlay_without_command controls_host_policy controls_interaction_policy controls_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
  - Result: passed, 13 tests.
  - Scope proven: default controls host planning, retained-independent controls interaction
    planning, declarative controls composition, blank panel pointer-down focus/no-command
    behavior, pressable preservation policy, source policy, and default overlay module gating are
    covered by targeted tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
  - Result: passed, 26 tests.
  - Scope proven: retained controls/minimap oracle behavior remains green after retained controls
    hit-test and pointer-down handling consume default host policy, including controls panel input
    blocking/fallthrough, pointer focus, keyboard navigation/activation, command binding
    overrides, Escape focus return, active-descendant semantics, tab traversal, and default
    declarative controls coverage under the compatibility feature set.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default-gated overlay policy modules, including controls host planning, compile
    without the retained canvas compatibility feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after retained controls
    hit-test and pointer-down handling were reduced to applying default host plans.
- `cargo nextest run -p fret-node`
  - Result: passed, 381 tests.
  - Scope proven: default `fret-node` behavior remains green, including default controls host
    planning, declarative controls blank panel pointer-down behavior, default overlay policies, and
    retained-independent surface policy gates.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 957 tests.
  - Scope proven: the retained canvas compatibility oracle remains green after controls retained
    hit-test and pointer-down handling consume default host plans, covering retained
    controls/minimap conformance, retained canvas/editor behavior, and default policy tests under
    the compatibility feature set.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the host-policy extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the evidence update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_host_policy.rs 2>&1); test -z "$out"`
  - Result: passed.
  - Scope proven: the new untracked controls host policy file has no whitespace errors before
    staging.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-117` changes only `fret-node` controls host planning plus retained controls
    hit-test/pointer-down adaptation. The default package gate, retained compatibility package
    gate, both feature checks, layering, catalog, formatting, and whitespace gates cover the
    changed surface.

## 2026-05-20 - RBX-M2-118 controls declarative pointer-up completion parity

Claim verified:

- The default declarative controls button path completes the retained controls button pointer-up
  behavior family without constructing the retained controls widget.
- Declarative `Pressable` now has controls-specific coverage for pointer-down capture, no early
  command dispatch, pointer-up capture release, focus transfer to the activated button, command
  dispatch on in-bounds release, and capture completion without command dispatch when the release
  lands outside the button.
- This slice adds no new controls mechanism; it proves the existing declarative host mechanism is
  sufficient for this retained controls behavior family.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_declarative_activation_dispatches_commands_and_honors_disabled_bindings controls_host_policy controls_interaction_policy`
  - Result: passed, 9 tests.
  - Scope proven: default controls declarative button activation, pointer-up/capture completion,
    command dispatch timing, controls host planning, and retained-independent controls interaction
    planning are covered without enabling the retained canvas compatibility feature.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
  - Result: passed, 27 tests.
  - Scope proven: retained controls/minimap oracle behavior remains green while the default
    declarative controls button path proves pointer-up/capture/command completion parity under the
    compatibility feature set.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default declarative controls path, including the pointer-up completion test
    dependencies, compiles without the retained canvas compatibility feature.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after adding the
    declarative pointer-up/capture completion proof.
- `cargo nextest run -p fret-node`
  - Result: passed, 382 tests.
  - Scope proven: default `fret-node` behavior remains green, including the new declarative
    controls pointer-up/capture completion test and retained-independent source policy gates.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 958 tests.
  - Scope proven: the retained canvas compatibility oracle remains green while the declarative
    controls pointer-up/capture completion proof is active.
- `cargo fmt --check`
  - Result: passed after `cargo fmt`.
  - Scope proven: Rust formatting is clean after adding the controls declarative pointer-up test.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the RBX-M2-118 documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the RBX-M2-118
    documentation update.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the RBX-M2-118 documentation update.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-118` changes only `fret-node` declarative controls tests and workstream
    evidence. The default package gate, retained compatibility package gate, both feature checks,
    layering, formatting, and targeted controls gates cover the changed surface.

## 2026-05-20 - RBX-M2-119 toolbar declarative child measurement host parity

Claim verified:

- Declarative managed surfaces can now measure a child before choosing its final host placement.
- Node and edge toolbar declarative hosts can use Auto child measurement to compute the same child
  rects as the retained toolbar layout policy, then layout and paint that child through the
  declarative managed-surface path without constructing retained toolbar widgets.
- Existing retained toolbar oracle tests for pointer fallthrough/interception and focus release
  remain green. This slice deliberately does not delete retained toolbar widgets because those
  pointer/focus behavior families and model/internals-driven target resolution still need default
  declarative coverage before deletion.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `crates/fret-ui/src/declarative/tests/managed_surface.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 7 tests.
  - Scope proven: the mechanism-only managed-surface contract remains green and now directly
    covers measuring a child before final host placement.
- `cargo nextest run -p fret-node node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat`
  - Result: passed, 16 tests.
  - Scope proven: default toolbar policy/layout tests, source-policy gating, fixed-size
    declarative toolbar composition, and the new Auto child measurement/placement host tests run
    without enabling retained canvas compatibility.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget toolbars_declarative toolbar_layout_policy overlay_toolbars_conformance`
  - Result: passed, 13 tests.
  - Scope proven: retained toolbar oracle behavior remains green while the default declarative
    toolbar host proves Auto child measurement and child placement parity under the compatibility
    feature set.
- `cargo nextest run -p fret-node`
  - Result: passed, 384 tests.
  - Scope proven: default `fret-node` behavior remains green after adding managed-surface child
    measurement and declarative toolbar host tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 960 tests.
  - Scope proven: the retained canvas compatibility oracle remains green after adding the
    declarative toolbar host measurement path, including retained toolbar pointer/focus oracle
    tests and the default toolbar tests under the compatibility feature set.
- `cargo fmt`
  - Result: passed.
  - Scope proven: Rust formatting was applied after the managed-surface and toolbar host edits.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting remains clean after the RBX-M2-119 code and documentation
    updates.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default-gated toolbar declarative host code compiles without retained canvas
    compatibility.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after adding
    managed-surface child measurement and declarative toolbar hosts.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and the retained bridge allowlist remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the RBX-M2-119 documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-119` changes only the managed-surface measurement mechanism and `fret-node`
    toolbar declarative host tests. The `fret-ui` managed-surface gate, default toolbar gate, and
    retained toolbar oracle gate cover the changed behavior surface.

## 2026-05-20 - RBX-M2-120 toolbar model/internals target resolution parity

Claim verified:

- Node and edge toolbar model/internals-driven target resolution is no longer retained-widget-local
  logic.
- The default-gated toolbar policy now resolves selected fallback targets, requested selected
  targets, requested unselected targets, and missing internals geometry for both node and edge
  toolbars.
- Declarative toolbar target wrappers consume the same default resolver shape as the retained
  toolbar widgets.
- Existing retained toolbar oracle tests for pointer fallthrough/interception and focus release
  remain green after retained widgets are moved onto the shared resolver.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/toolbar_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node node_toolbar_declarative_target_resolution_uses_view_state_and_internals edge_toolbar_declarative_target_resolution_uses_view_state_and_internals`
  - Result: initially failed to compile because the declarative target resolver did not exist; then
    passed, 2 tests.
  - Scope proven: the new declarative target wrappers resolve node/edge targets from
    `NodeGraphViewState` and `NodeGraphInternalsStore` on the default path.
- `cargo nextest run -p fret-node toolbar_policy node_toolbar_declarative_target_resolution_uses_view_state_and_internals edge_toolbar_declarative_target_resolution_uses_view_state_and_internals`
  - Result: passed, 9 tests.
  - Scope proven: the default toolbar policy covers selected fallback, requested selected and
    unselected targets, and missing internals geometry for node and edge toolbars.
- `cargo nextest run -p fret-node toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat`
  - Result: passed, 23 tests.
  - Scope proven: default toolbar composition, layout/hit-test policy, target resolution policy,
    declarative managed-host pointer/focus behavior, and source-policy gating all run without
    retained canvas compatibility.
- `cargo nextest run -p fret-node --features compat-retained-canvas toolbars_declarative toolbar_layout_policy toolbar_policy overlay_toolbars_conformance`
  - Result: passed, 25 tests.
  - Scope proven: retained toolbar pointer/focus oracle behavior remains green after retained
    widgets consume the default model/internals target resolver.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after the resolver/test edits.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-120`.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-120` changes only `fret-node` toolbar target resolution and tests. The default
    toolbar gate and retained compatibility toolbar oracle gate cover the changed behavior surface.

## 2026-05-20 - RBX-M2-121 retained toolbar widget deletion

Claim verified:

- Retained node/edge toolbar widgets and retained toolbar layout adapter files were deleted after
  default declarative tests covered their behavior families.
- The retained toolbar conformance test module was deleted because its pointer/focus/target
  assertions are now covered by default `toolbars_declarative`, `toolbar_layout_policy`, and
  `toolbar_policy` tests.
- Test-only retained toolbar exports were removed from `ui/mod.rs` and `overlays/mod.rs`.
- `fret-node`'s retained bridge source-policy allowlist no longer includes toolbar retained files.
- `fret-node` default and `compat-retained-canvas` package gates remain green after deletion.

Evidence:

- Deleted `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
- Deleted `ecosystem/fret-node/src/ui/overlays/toolbars_layout.rs`
- Deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/toolbar_policy.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the compatibility feature build no longer needs retained toolbar modules or
    test-only toolbar exports.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default `fret-ui`-only `fret-node` still compiles after retained toolbar deletion.
- `cargo nextest run -p fret-node toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 24 tests.
  - Scope proven: default toolbar behavior coverage and source-policy gates remain green after
    deleting retained toolbar files.
- `cargo nextest run -p fret-node --features compat-retained-canvas toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 24 tests.
  - Scope proven: the compatibility feature set still runs the default toolbar behavior coverage
    and no longer requires retained toolbar oracle tests.
- `cargo nextest run -p fret-node`
  - Result: passed, 391 tests.
  - Scope proven: default `fret-node` package behavior remains green after retained toolbar deletion.
- `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Result: passed, 964 tests.
  - Scope proven: retained canvas compatibility package behavior remains green after retained
    toolbar deletion.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after shrinking
    the `fret-node` retained overlay island.
- `rg -n "\bNodeGraphNodeToolbar\b|\bNodeGraphEdgeToolbar\b|overlay_toolbars_conformance|toolbars_layout|mod toolbars;|src/ui/overlays/toolbars\.rs|src/ui/overlays/toolbars_layout\.rs" ecosystem/fret-node/src -g '*.rs'`
  - Result: no matches.
  - Scope proven: deleted retained toolbar types/modules/tests have no remaining Rust source
    consumers.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-121` deletes retained toolbar-only files inside `fret-node`. The default
    package gate, compatibility package gate, layering gate, and no-user search cover the changed
    behavior and source-policy surfaces.

## 2026-05-20 - RBX-M2-122 controls activation focus-restore parity

Claim verified:

- Declarative `Pressable` now has a narrow focus-capable activation hook that preserves the existing
  `pressable_on_activate` contract while allowing policy code to request focus to another
  declarative element after activation.
- Default declarative controls button pointer activation and keyboard activation can dispatch their
  bound command and restore focus to a node graph surface/canvas target without constructing the
  retained controls widget.
- Existing retained controls/minimap oracle behavior remains green under `compat-retained-canvas`,
  including controls button click focus return to canvas, keyboard activation focus return to
  canvas, Escape focus return, active-descendant semantics, command binding overrides, panel
  blocking, pointer fallthrough, and tab traversal.
- Retained controls were not deleted in this slice; the new default coverage removes the focus
  restore gap that would have made deletion unsafe.

Evidence:

- `crates/fret-ui/src/action.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
- `crates/fret-ui/src/declarative/tests/interactions/pressable.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-ui pressable_focus_activation_hook_can_restore_focus_after_pointer_activation pressable_focus_activation_hook_can_restore_focus_after_keyboard_activation pressable_on_activate_hook_runs_on_pointer_activation pressable_on_activate_hook_runs_on_keyboard_activation`
  - Result: passed, 4 tests.
  - Scope proven: the `Pressable` focus-capable activation hook restores focus after pointer and
    keyboard activation while the existing activation hook behavior remains green.
- `cargo nextest run -p fret-node controls_declarative_button_activation_restores_focus_to_surface_target controls_declarative_keyboard_activation_restores_focus_to_surface_target controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_declarative_activation_dispatches_commands_and_honors_disabled_bindings controls_host_policy controls_interaction_policy`
  - Result: passed, 11 tests.
  - Scope proven: default declarative controls pointer and keyboard activation can dispatch commands
    and restore focus to a surface target, alongside existing controls pointer-up/capture,
    activation binding, host, and interaction planning coverage.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative_button_activation_restores_focus_to_surface_target controls_declarative_keyboard_activation_restores_focus_to_surface_target controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
  - Result: passed, 29 tests.
  - Scope proven: retained controls/minimap oracle behavior remains green while the default
    declarative controls focus-restore proof runs under the compatibility feature set.
- `cargo fmt -p fret-ui -p fret-node`
  - Result: passed.
  - Scope proven: touched Rust files were formatted after the `Pressable` and controls edits.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-122`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after adding the
    `Pressable` focus-capable activation hook and controls focus-target wiring.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-122` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-122` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-122` changes only the `Pressable` activation hook mechanism, declarative
    controls focus-target wiring, and workstream evidence. The `fret-ui` mechanism gate, default
    controls gate, and retained compatibility controls/minimap oracle gate cover the changed
    behavior surface.

## 2026-05-20 - RBX-M2-123 controls root keyboard semantics and Escape parity

Claim verified:

- Declarative controls now expose the retained-compatible root semantics node on the default path:
  panel role, `Controls` label, `node_graph.controls` test ID, focusability, and active controls
  button value with fallback to the first button.
- Default declarative controls pointer-down and root keyboard navigation update active semantics
  value without constructing the retained controls widget.
- Default declarative controls root keyboard activation dispatches the selected command and restores
  focus to the node graph surface/canvas target.
- Default declarative controls Escape restores focus to the node graph surface/canvas target,
  dispatches no commands, and clears active semantics value back to `Toggle connection mode`.
- The retained controls/minimap oracle remains green under `compat-retained-canvas`, including the
  corresponding retained Escape, active semantics, keyboard activation, focus return, and command
  binding override tests.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_interaction_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/controls_host_policy.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_declarative_pointer_down_promotes_keyboard_active_semantics_value controls_declarative_root_keyboard_navigation_activation_dispatches_and_restores_focus controls_declarative_escape_restores_focus_without_dispatch_and_clears_active_semantics controls_declarative_button_activation_restores_focus_to_surface_target controls_declarative_keyboard_activation_restores_focus_to_surface_target controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_declarative_panel_blank_pointer_down_focuses_overlay_without_command`
  - Result: passed, 7 tests.
  - Scope proven: the new default controls root semantics, keyboard navigation/activation,
    Escape/focus behavior, and existing pointer/focus activation paths pass together.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
  - Result: passed, 17 tests.
  - Scope proven: default controls composition, root semantics, activation, host, and interaction
    policy coverage runs without retained canvas compatibility. The retained oracle module is not
    present in the default build.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
  - Result: passed, 32 tests.
  - Scope proven: the default declarative controls proof and retained controls/minimap oracle stay
    green together under the compatibility feature set.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after the controls root keyboard
    semantics edits.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-123`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after the
    declarative controls root semantics/keyboard edits.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-123` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-123` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-123` changes only declarative controls root semantics, root keyboard handling,
    tests, and workstream evidence. The default controls gate and retained compatibility
    controls/minimap oracle gate cover the changed behavior surface.

## 2026-05-20 - RBX-M2-124 controls declarative overlay integration parity

Claim verified:

- Default declarative controls now prove the retained controls overlay/surface integration behavior
  that must hold before deleting the retained controls widget.
- Pointer-down outside the controls panel falls through to the node graph surface.
- Blank pointer-down inside the controls panel blocks surface input and focuses the controls root
  for keyboard follow-up.
- Focus traversal can move from the node graph surface to the focusable controls root, and Escape
  returns focus to the surface without dispatching controls commands.
- The retained controls/minimap oracle remains green under `compat-retained-canvas`, including the
  old retained controls pointer fallthrough/blocking, focus traversal, Escape, keyboard activation,
  semantics, and command binding override tests.
- Retained `NodeGraphControlsOverlay` was not deleted in this slice; the next narrow deletion task
  must trim the combined retained controls/minimap oracle while preserving retained minimap
  coverage.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node controls_declarative_pointer_events_fall_through_outside_panel_to_surface controls_declarative_blocks_surface_input_within_panel_even_off_button controls_declarative_focus_traversal_reaches_controls_from_surface`
  - Result: passed, 3 tests.
  - Scope proven: the new default declarative controls integration tests pass together for surface
    fallthrough, panel blocking/focus, focus traversal into controls, and Escape focus return.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
  - Result: passed, 20 tests.
  - Scope proven: default controls composition, host policy, interaction policy, root semantics,
    activation, focus restore, overlay/surface integration, and focus traversal coverage run
    without retained canvas compatibility. The retained oracle module is absent in the default
    build.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
  - Result: passed, 35 tests.
  - Scope proven: the default declarative controls integration proof and retained controls/minimap
    oracle stay green together under the compatibility feature set.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after the controls integration
    tests.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-124`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after the
    controls declarative integration tests.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-124` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-124` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-124` changes only default declarative controls integration tests and
    workstream evidence. The default controls gate, retained compatibility controls/minimap oracle
    gate, formatting, layering, catalog, and whitespace gates cover the changed behavior surface.

## 2026-05-20 - RBX-M2-125 retained controls widget deletion

Claim verified:

- The retained `NodeGraphControlsOverlay` widget has been deleted after default declarative
  controls coverage proved the retained behavior families.
- Retained controls test-only exports and the `controls.rs` retained bridge source allowlist entry
  have been removed.
- The combined retained `overlay_minimap_controls_conformance` oracle has been trimmed to
  minimap-only coverage, so retained minimap pointer fallthrough, drag, keyboard pan/zoom,
  controller navigation binding, store/view sync, focus behavior, and semantics test ID coverage
  remain available under `compat-retained-canvas`.
- Source scanning finds no retained `NodeGraphControlsOverlay` users or retained `controls.rs`
  module/export entries in `ecosystem/fret-node/src`.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- deleted `ecosystem/fret-node/src/ui/overlays/controls.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained canvas compatibility island still compiles after deleting retained
    controls while retained minimap remains.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default declarative `fret-ui` package surface compiles without retained controls.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 22 tests.
  - Scope proven: default declarative controls behavior and source-policy gates remain green after
    deleting the retained controls widget.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 27 tests.
  - Scope proven: default controls behavior, retained bridge source-policy gates, and retained
    minimap oracle coverage remain green together under `compat-retained-canvas`.
- `rg -n "\\bNodeGraphControlsOverlay\\b|src/ui/overlays/controls\\.rs|include_str!\\(\\\"ui/overlays/controls\\.rs\\\"\\)|mod controls;|pub use controls::|controls_overlay_requires_explicit_editor_config_model|controls_overlay_" ecosystem/fret-node/src -g '*.rs'`
  - Result: no retained controls widget/module/export matches; only declarative
    `node_graph_controls_overlay_element(...)` names remain.
  - Scope proven: retained controls widget source and entry points are removed from `fret-node`
    source.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after deleting retained controls.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-125`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after shrinking
    `fret-node` retained bridge usage.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-125` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-125` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-125` deletes only the retained controls widget and controls-only retained
    oracle tests inside `fret-node`; the default controls gate, retained compatibility minimap
    oracle gate, compile gates, source scans, formatting, layering, catalog, and whitespace gates
    cover the changed behavior surface.

## 2026-05-20 - RBX-M2-126 declarative minimap managed-host side-effect parity

Claim verified:

- The default declarative minimap path now owns the retained minimap host side effects that were
  still missing after composition/policy extraction: minimap-only hit testing, pointer focus return,
  pointer capture/release, drag pan updates, keyboard pan/zoom, Escape focus return, redraw, and
  notify.
- The minimap tree is now a focusable `node_graph.minimap` semantics root backed by a
  `ManagedSurface` host and declarative canvas child; it does not construct the retained minimap
  widget.
- The retained minimap widget remains behind `compat-retained-canvas` as the oracle for the next
  deletion slice; no retained minimap source was deleted in this task.
- The old retained minimap oracle remains green beside the new default declarative proof.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `crates/fret-ui/src/widget.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_navigation_policy.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node minimap_declarative`
  - Result: passed, 5 tests.
  - Scope proven: default declarative minimap composition, paint plan, pointer fallthrough, drag
    view/store updates, pointer capture/release, keyboard pan/zoom, and Escape focus return pass
    without retained canvas compatibility.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 31 tests.
  - Scope proven: default declarative controls and minimap behavior plus retained-bridge source
    policy gates remain green together after adding the minimap managed host.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 36 tests.
  - Scope proven: the new default declarative minimap proof and the retained minimap oracle remain
    green together under `compat-retained-canvas`, including retained minimap pointer fallthrough,
    drag, keyboard pan/zoom, controller navigation binding, store/view sync, focus behavior, and
    semantics test ID coverage.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained minimap oracle and remaining retained island still compile after the
    default declarative minimap managed-host changes.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default declarative `fret-ui` surface compiles without enabling the retained
    canvas compatibility island.
- `cargo fmt -p fret-ui -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-ui` and `fret-node` Rust files were formatted after the minimap
    managed-host edits.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-126`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after the
    minimap managed-host changes.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-126` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-126` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-126` changes only `fret-ui` managed-surface/event-focus mechanisms and
    `fret-node` minimap declarative host/navigation behavior. The default minimap gate, retained
    compatibility minimap oracle gate, feature compile gates, formatting, layering, catalog, and
    whitespace gates cover the changed behavior surface.

## 2026-05-20 - RBX-M2-127 retained minimap widget deletion

Claim verified:

- The retained `NodeGraphMiniMapOverlay` widget has been deleted after `RBX-M2-126` proved the
  default declarative minimap host covers retained minimap hit-test, keyboard, pointer, focus,
  capture, redraw/notify, and store/controller viewport behavior.
- Retained minimap test-only exports and the retained minimap oracle module have been removed.
- `src/ui/overlays/minimap.rs` has been removed from the retained bridge source migration ledger.
- A deletion-preflight compat retained oracle run was performed in this worktree before deleting
  the retained minimap source.
- Source scanning finds no retained `NodeGraphMiniMapOverlay`, retained minimap module/export, or
  deleted minimap oracle module references in `ecosystem/fret-node/src`.

Evidence:

- deleted `ecosystem/fret-node/src/ui/overlays/minimap.rs`
- deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node --features compat-retained-canvas minimap_declarative minimap_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 16 tests.
  - Scope proven: deletion-preflight retained minimap oracle coverage was green in the current
    worktree before deleting the retained minimap widget and oracle module.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the remaining retained canvas compatibility island still compiles after deleting
    retained minimap.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default declarative `fret-ui` surface compiles without retained minimap.
- `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 31 tests.
  - Scope proven: default declarative controls/minimap behavior and source-policy gates remain
    green after deleting the retained minimap widget.
- `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 31 tests.
  - Scope proven: default declarative controls/minimap behavior and source-policy gates remain
    green under `compat-retained-canvas` after deleting the retained minimap widget.
- `rg -n "\\bNodeGraphMiniMapOverlay\\b|overlay_minimap_controls_conformance|src/ui/overlays/minimap\\.rs|include_str!\\(\\\"ui/overlays/minimap\\.rs\\\"\\)|mod minimap;|pub use minimap|MINIMAP_RS|minimap_navigation_surface_stays" ecosystem/fret-node/src -g '*.rs'`
  - Result: no retained minimap widget/module/export/oracle matches; only declarative
    `NodeGraphMiniMapOverlayElementProps` names remain.
  - Scope proven: retained minimap widget source and entry points are removed from `fret-node`
    source.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after deleting retained minimap.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-127`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after deleting
    retained minimap.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-127` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-127` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-127` deletes only the retained minimap widget and retained minimap oracle
    module inside `fret-node`; the deletion-preflight oracle, default declarative minimap gate,
    retained-compatibility compile/test gates, source scans, formatting, layering, catalog, and
    whitespace gates cover the changed behavior surface.

## 2026-05-20 - RBX-M2-128 declarative blackboard host side-effect parity

Claim verified:

- The default declarative blackboard path now owns the retained blackboard host side effects for
  focusable panel semantics, active action semantics value, blank-panel pointer blocking, outside
  pointer fallthrough, pointer capture/up completion through pressable, root keyboard
  navigation/activation, Escape focus return, and action hook dispatch.
- The new default tests exercise those behaviors without constructing `NodeGraphBlackboardOverlay`
  or enabling the retained canvas compatibility island.
- The retained blackboard widget remains behind `compat-retained-canvas` as the oracle for the next
  migration slice because graph/controller transaction submission and symbol-rename handoff are
  still retained-adapter behavior at the actual overlay integration boundary.
- The retained blackboard oracle remains green beside the new default declarative proof.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_interaction_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_paint_plan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 16 tests.
  - Scope proven: default declarative blackboard composition/host side effects, interaction policy,
    paint-plan policy, and retained-bridge source-policy gates pass without
    `compat-retained-canvas`.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_blackboard_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 25 tests.
  - Scope proven: the new default declarative blackboard proof and the retained blackboard oracle
    remain green together under `compat-retained-canvas`, including retained add/insert/delete,
    controller-first add, pointer click, outside-panel fallthrough, keyboard activation, and
    symbol-rename handoff coverage.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the remaining retained canvas compatibility island still compiles after adding
    default declarative blackboard host side effects.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default declarative `fret-ui` surface compiles without enabling retained
    canvas compatibility.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after the blackboard declarative
    host edits.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-128`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after the
    blackboard declarative host changes.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after the `RBX-M2-128` documentation
    update.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors after the `RBX-M2-128` updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-128` changes only `fret-node` blackboard declarative host behavior and
    workstream docs. The default blackboard gate, retained compatibility blackboard oracle gate,
    feature compile gates, formatting, layering, catalog, and whitespace gates cover the changed
    behavior surface.

## 2026-05-20 - RBX-M2-129 declarative blackboard action integration

Claim verified:

- The default declarative blackboard path can now execute the retained blackboard's remaining
  action integration responsibilities without `NodeGraphBlackboardOverlay`, `NodeGraphEditQueue`,
  or `fret-ui/unstable-retained-bridge`.
- Add Symbol, Insert Symbol Ref, and Delete Symbol commit through `NodeGraphSurfaceBinding` and
  the store/controller transaction path.
- Rename opens `NodeGraphOverlayState.symbol_rename` and does not queue a graph transaction.
- The retained blackboard oracle remains green under `compat-retained-canvas`, so this slice proves
  capability parity before any retained blackboard deletion.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/blackboard_policy.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/binding_store_sync.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node blackboard_declarative`
  - Result: passed, 10 tests.
  - Scope proven: default declarative blackboard composition, host side effects, and new binding
    action integration tests pass without retained canvas compatibility.
- `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan`
  - Result: passed, 17 tests.
  - Scope proven: declarative blackboard action integration remains green with the shared default
    interaction and paint-plan policy tests.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_blackboard_conformance`
  - Result: passed, 27 tests.
  - Scope proven: the new default declarative binding/overlay-state integration and the retained
    blackboard oracle remain green together, including retained add/insert/delete/controller-first
    add/rename handoff behavior.

Broader gates not run yet:

- `cargo fmt --check`, `python3 tools/check_layering.py`, `python3 tools/check_workstream_catalog.py`,
  and `git diff --check`
  - Reason: this was the action-integration proof slice. These gates will be run after the
    immediately-following retained blackboard deletion slice so the final source deletion and docs
    update are verified together.
- `cargo nextest run --workspace`
  - Reason: `RBX-M2-129` changes only `fret-node` blackboard declarative action integration. The
    default blackboard gate and retained blackboard oracle directly cover the changed behavior
    surface.

## 2026-05-20 - RBX-M2-130 retained blackboard deletion

Claim verified:

- The retained blackboard widget, retained blackboard paint adapter, and retained blackboard oracle
  test module were deleted only after a deletion-preflight compat retained oracle passed in the
  current worktree.
- Default declarative blackboard tests now carry the behavior contract for composition, host side
  effects, interaction policy, paint planning, binding transaction submission, and symbol-rename
  handoff.
- `fret-node` no longer allows `src/ui/overlays/blackboard.rs` or
  `src/ui/overlays/blackboard_paint.rs` in the retained bridge source migration ledger.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- Deleted: `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
- Deleted: `ecosystem/fret-node/src/ui/overlays/blackboard_paint.rs`
- Deleted: `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node --features compat-retained-canvas overlay_blackboard_conformance blackboard_declarative blackboard_interaction_policy blackboard_paint_plan`
  - Result: passed, 27 tests.
  - Scope proven: deletion-preflight retained blackboard oracle was green in the current worktree
    before deleting retained blackboard source and oracle tests.
- `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 20 tests.
  - Scope proven: default declarative blackboard behavior and source-policy gates pass after the
    retained blackboard deletion.
- `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
  - Result: passed, 19 tests.
  - Scope proven: default declarative blackboard behavior and retained bridge source-policy gates
    pass under `compat-retained-canvas` after deleting the retained blackboard oracle.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the remaining retained canvas compatibility island compiles after retained
    blackboard deletion.
- `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: default declarative `fret-ui` surface compiles without retained compatibility.
- `rg -n "\\bNodeGraphBlackboardOverlay\\b|overlay_blackboard_conformance|ui/overlays/blackboard\\.rs|blackboard_paint\\.rs|mod blackboard;|mod blackboard_paint;|pub use blackboard" ecosystem/fret-node/src -g '*.rs'`
  - Result: no retained blackboard widget/module/export/oracle matches; only declarative
    `NodeGraphBlackboardOverlayElementProps` names remain.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after deletion.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-130` deletes only the retained blackboard widget/paint/oracle inside
    `fret-node`; the deletion-preflight oracle, default declarative blackboard gates, compat
    compile/test gates, source scan, formatting, layering, catalog, and whitespace gates cover the
    changed behavior surface.

## 2026-05-20 - RBX-M2-131 declarative rename managed-host parity

Claim verified:

- The default declarative rename path now owns the retained rename host's seed/focus/focus-loss,
  submit/cancel, focus-restore, graph/store transaction, and hit-test masking responsibilities.
- The new tests exercise those behaviors without constructing the retained `NodeGraphOverlayHost`
  or enabling `compat-retained-canvas`.
- The retained group/symbol rename conformance oracle remains green in the same worktree, so
  retained rename host deletion is now eligible for a narrow deletion slice.

Evidence:

- `crates/fret-ui/src/managed_surface.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_command.rs`
- `ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs`
- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_group_rename_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_symbol_rename_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout`
  - Result: passed, 19 tests.
  - Scope proven: default declarative rename composition, managed-host seed/focus/hit-test,
    submit/cancel/focus restore, focus-loss close, lifecycle planning, command/session policy, and
    host layout planning pass without retained canvas compatibility.
- `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 9 tests.
  - Scope proven: `ManagedSurface` mechanism hooks for child measurement, focus request,
    host-selected hit-test rects, event/command hooks, prepaint, paint ordering, services access,
    and text release remain green.
- `cargo nextest run -p fret-node --features compat-retained-canvas overlay_group_rename_conformance overlay_symbol_rename_conformance rename_declarative rename_lifecycle rename_command`
  - Result: passed, 26 tests.
  - Scope proven: retained group/symbol rename oracle remains green beside the new default
    declarative managed-host tests under `compat-retained-canvas`.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after `RBX-M2-131`.
- `git diff --check -- crates/fret-ui/src/managed_surface.rs ecosystem/fret-node/src/ui/overlays/rename_declarative.rs ecosystem/fret-node/src/ui/overlays/mod.rs ecosystem/fret-node/src/ui/overlays/rename_host_layout.rs ecosystem/fret-node/src/ui/overlays/rename_policy.rs`
  - Result: passed.
  - Scope proven: tracked files touched by this slice have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-131` changes only the `ManagedSurface` mechanism surface and `fret-node`
    rename declarative host path; the focused `ManagedSurface`, default rename, compat retained
    rename oracle, formatting, and whitespace gates cover the changed behavior surface.

## 2026-05-20 - RBX-M2-132 retained rename host deletion

Claim verified:

- The retained `NodeGraphOverlayHost` rename adapter was deleted after the deletion-preflight
  retained group/symbol rename oracle and default declarative rename managed-host tests both passed
  in this worktree.
- The retained-only `rename_host_event.rs` adapter and retained group/symbol rename conformance
  modules were deleted.
- `group_rename.rs` now only carries overlay state, and `group_rename.rs` / `overlays/mod.rs` were
  removed from the retained bridge source usage ledger.
- Default declarative rename tests now own seed text, first-open focus, hit-test masking,
  submit/cancel, graph/store transaction submission, focus restore, and focus-loss close behavior.

Evidence:

- `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- deleted `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
- deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_group_rename_conformance.rs`
- deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_symbol_rename_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas overlay_group_rename_conformance overlay_symbol_rename_conformance rename_declarative rename_lifecycle rename_command`
  - Result: passed, 26 tests.
  - Scope proven: retained group/symbol rename oracle behavior was green immediately before
    deleting the retained host and oracle modules.
- deletion-preflight `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout`
  - Result: passed, 19 tests.
  - Scope proven: default declarative rename behavior was green before deletion.
- post-delete `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout`
  - Result: passed, 19 tests.
  - Scope proven: default declarative rename composition, managed-host side effects,
    command/session policy, lifecycle planning, and host layout planning still pass after deleting
    the retained host and oracle modules.
- post-delete `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the remaining retained canvas compatibility island still compiles after retained
    rename deletion.
- post-delete `cargo nextest run -p fret-node --features compat-retained-canvas rename_declarative rename_lifecycle rename_command retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge overlay_policy_modules_compile_without_retained_canvas_compat`
  - Result: passed, 19 tests.
  - Scope proven: rename/default overlay policy tests and retained bridge source-policy gates pass
    under `compat-retained-canvas` after removing the retained rename source allowlist entries.
- post-delete `cargo nextest run -p fret-ui managed_surface`
  - Result: passed, 9 tests.
  - Scope proven: `ManagedSurface` mechanism hooks used by the declarative rename host remain green.
- post-delete `cargo check -p fret-node --no-default-features --features fret-ui`
  - Result: passed.
  - Scope proven: the default declarative `fret-ui` node graph surface compiles without retained
    compatibility.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after deletion.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "NodeGraphOverlayHost|rename_host_event|overlay_group_rename_conformance|overlay_symbol_rename_conformance|layout_hidden_child_and_release_focus|src/ui/overlays/group_rename\\.rs" ecosystem/fret-node/src -g '*.rs'`
  - Result: no matches.
  - Scope proven: retained rename host, retained rename event adapter, retained rename oracle test
    modules, orphaned retained layout helper, and retained source ledger string no longer exist in
    `fret-node` source.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-132` deletes the retained rename host/oracles inside `fret-node`; the
    deletion-preflight retained oracle, default rename gates, compat compile/test gates,
    `ManagedSurface` mechanism gate, source scan, formatting, layering, catalog, and whitespace
    gates cover the changed behavior surface.

## 2026-05-20 - RBX-M2-133 retained diagnostics anchor deletion

Claim verified:

- The retained diagnostics-only `NodeGraphDiagAnchor` and `NodeGraphDiagConnectingFlag` widgets had
  no callers outside their own module and were deleted.
- The dead retained canvas diagnostics anchor port plumbing was deleted with them:
  `with_diagnostics_anchor_ports`, `diagnostics_anchor_ports`, and
  `retained_widget_layout_publish.rs`.
- `diag_anchors.rs` was removed from the retained bridge source usage ledger.
- `a11y.rs` remains intentionally because retained canvas active-descendant child semantics still
  need default declarative proof before deletion.

Evidence:

- deleted `ecosystem/fret-node/src/ui/diag_anchors.rs`
- deleted `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_publish.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/builders.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/construct.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_children.rs`
- `docs/ui-diagnostics-and-scripted-tests.md`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the remaining retained canvas compatibility island compiles after deleting the
    diagnostics anchor widgets and dead anchor-port layout plumbing.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: retained source usage ledger and crate-private compatibility island policy remain
    green after removing `diag_anchors.rs` from the allowlist.
- `rg -n "DiagnosticsAnchorPorts|diagnostics_anchor_ports|with_diagnostics_anchor_ports|retained_widget_layout_publish|publish_diagnostics_derived_outputs|NodeGraphDiagAnchor|NodeGraphDiagConnectingFlag|diag_anchors" ecosystem/fret-node/src docs/ui-diagnostics-and-scripted-tests.md docs/workstreams/retained-bridge-exit-v1 -g '*.rs' -g '*.md'`
  - Result: only historical workstream references remain.
  - Scope proven: runtime source and diagnostics docs no longer expose the deleted retained
    diagnostics anchor API.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-133` removes no-user retained diagnostics anchor widgets and dead retained
    canvas anchor-port plumbing; the compat compile gate, retained source-policy gate, and no-user
    scan cover the changed behavior surface.

## 2026-05-20 - RBX-M2-134 retained a11y anchor deletion

Claim verified:

- The default declarative node graph surface now owns active-descendant semantics through
  `NodeGraphSurfaceBinding::surface_props()` and `NodeGraphSurfaceProps::new(...)`, including
  focused port, focused edge, focused node, and the retained priority order of port before edge
  before node.
- The default declarative frame now publishes presenter-derived active-descendant labels and does
  not expose stale active descendants for selected nodes/edges that are missing from current
  geometry.
- The retained a11y child-anchor widgets were deleted after the default declarative tests and the
  retained compat oracle agreed in deletion preflight.
- `a11y.rs` was removed from the retained bridge source usage ledger.

Evidence:

- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/ui/binding.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
- deleted `ecosystem/fret-node/src/ui/a11y.rs`
- deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/a11y_active_descendant_conformance.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Deletion-preflight commands:

- `cargo nextest run -p fret-node node_graph_surface_active_descendant`
  - Result: passed, 2 tests.
  - Scope proven: the default declarative surface exposed active-descendant semantics for focused
    port and focused node before the retained oracle was deleted.
- `cargo nextest run -p fret-node --features compat-retained-canvas a11y_active_descendant_conformance node_graph_surface_active_descendant`
  - Result: passed, 4 tests.
  - Scope proven: retained child-anchor semantics and default declarative semantics agreed for the
    focused port/node cases before deleting the retained `a11y.rs` module and retained oracle test.

Post-delete commands:

- `cargo nextest run -p fret-node node_graph_surface_active_descendant`
  - Result: passed, 5 tests.
  - Scope proven: the default declarative binding surface exposes active-descendant semantics for
    selected/focused port, edge, and node, preserves the retained priority order of port before
    edge before node, wires both default props entry points to the binding internals store, and
    suppresses stale active descendants for missing graph items after deleting retained a11y
    anchors.
- `cargo nextest run -p fret-node node_graph_surface_active_descendant retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 6 tests.
  - Scope proven: default active-descendant behavior and the retained bridge source usage ledger
    remain green after removing `src/ui/a11y.rs`.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the remaining retained canvas compatibility island still compiles without
    `a11y.rs`.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound node_graph_surface_active_descendant`
  - Result: passed, 7 tests.
  - Scope proven: the compat feature still gates only the remaining retained island, while active
    descendant behavior is supplied by the default declarative binding surface.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted after adding declarative semantics
    tests and deleting retained a11y modules.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after shrinking
    `fret-node` retained source usage.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^\\s*(pub\\s+)?mod a11y;|NodeGraphA11yActiveDescendant|NodeGraphA11yFocused|a11y_active_descendant_conformance" ecosystem/fret-node/src -S`
  - Result: no matches.
  - Scope proven: runtime source no longer declares the deleted retained a11y module or references
    the deleted retained a11y widgets/conformance module.

Broader gates:

- `cargo nextest run --workspace`
  - Reason not planned for this slice: `RBX-M2-134` changes `fret-node` declarative semantics and
    deletes a retained `fret-node` oracle. The default semantic tests, compat compile/test gates,
    source-policy scan, formatting, layering, catalog, and whitespace gates cover the changed
    behavior surface.

## 2026-05-20 - RBX-M2-135 declarative portal lifecycle and measurement parity

Claim verified:

- The default declarative visible-subset portal path now uses the retained-compatible subtree
  lifecycle key: node id plus node kind plus node kind version.
- Default declarative tests prove portal subtree identity persists across frames for the same node
  kind/version and resets when either `kind_version` or `kind` changes.
- Default declarative measured-geometry flush coverage now proves portal node-size hints are
  growth-only and that removed graph nodes are pruned from `MeasuredGeometryStore`.
- Retained portal lifecycle, measured-geometry, and measured-internals oracle tests still pass
  under `compat-retained-canvas`; retained portal files are intentionally kept for arbitrary
  per-kind renderer subtree hosting and retained command-adapter follow-up work.

Evidence:

- `ecosystem/fret-node/src/ui/declarative/paint_only/portals.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_geometry_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_internals_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo nextest run -p fret-node declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes`
  - Result: passed, 2 tests.
  - Scope proven: default declarative portal lifecycle identity and measured-geometry flush
    behavior are covered without enabling `compat-retained-canvas`.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes`
  - Result: passed, 6 tests.
  - Scope proven: retained portal lifecycle, measurement publishing, and measured-internals oracle
    behavior still agree with the new default declarative coverage.

Broader gates:

- `cargo check -p fret-node`
  - Result: passed.
  - Scope proven: the default package compiles after the declarative portal lifecycle-key change.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed.
  - Scope proven: the retained compatibility island compiles after the default portal change.
- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 1 test.
  - Scope proven: retained bridge source usage did not spread while adding default portal parity
    coverage.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: the retained compatibility island remains crate-private and controller-bound
    under the compat feature.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 427 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

## 2026-05-20 - RBX-M2-140 declarative portal renderer hosting

Claim verified:

- The default declarative node graph surface can host arbitrary per-kind visible-subset portal
  subtrees without enabling `compat-retained-canvas`.
- `NodeGraphNodeTypes` now works as a default declarative portal renderer registry, including a
  fallback renderer for unregistered node kinds.
- Custom declarative portal renderer output preserves the retained-compatible `(node id, node kind,
  node kind_version)` lifecycle key and replaces the built-in lightweight label only when it returns
  a non-empty subtree.
- Custom portal subtree measurements publish through the same default portal measured-geometry
  pipeline into `MeasuredGeometryStore`.
- Retained portal lifecycle, measured-geometry, and measured-internals oracle tests still pass under
  `compat-retained-canvas`; retained portal files are intentionally kept only for command-adapter
  deletion-preflight work.

Evidence:

- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/portals.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/surface_shell.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/ui/registry.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_geometry_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_internals_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo nextest run -p fret-node declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
  - Result: passed, 3 tests.
  - Scope proven: default declarative portal renderer hosting covers custom per-kind subtree
    replacement, empty-output fallback to the built-in label, registry fallback rendering, lifecycle
    persistence/reset, and custom subtree measured-geometry publishing without
    `compat-retained-canvas`.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
  - Result: passed, 7 tests.
  - Scope proven: retained portal lifecycle, measurement publishing, measured-internals, and
    command-preference oracle coverage remains green while the default declarative renderer path
    carries arbitrary per-kind subtree hosting.

Broader gates:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default package compiles after adding the public declarative portal renderer
    surface.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after the default portal
    renderer addition.
- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger`
  - Result: passed, 1 test.
  - Scope proven: retained bridge source usage did not spread while adding the default declarative
    portal renderer surface.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: the retained compatibility island remains crate-private and controller-bound
    under the compat feature after the default portal renderer addition.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-140` changes the `fret-node` default declarative portal hosting surface and
    keeps retained files as oracle code; default/compat targeted tests, compile gates, source-policy
    gates, formatting, layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-145 declarative portal command hosting

Claim verified:

- The default declarative node graph surface can host portal command routing without constructing
  the retained `NodeGraphPortalHost` or importing retained `CommandCx`.
- `NodeGraphSurfaceProps::portal_command_handler` gives component/policy layers a default
  declarative seam for shared portal text command handling.
- `PortalCommandOutcome::Commit(...)` is submitted through `NodeGraphSurfaceBinding`, keeping the
  authoritative store and graph/view mirrors synchronized.
- Surface-root command availability is scoped to nodes present in the current binding, so unclaimed
  portal commands keep bubbling instead of being swallowed.
- Retained portal lifecycle, measured-geometry, and measured-internals oracle tests still pass under
  `compat-retained-canvas`; retained portal files now remain only for text/number command-adapter
  deletion-preflight work.

Evidence:

- `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_geometry_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_internals_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo nextest run -p fret-node declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
  - Result: passed, 4 tests.
  - Scope proven: default declarative portal command hosting submits a binding-backed graph
    transaction, keeps store and graph mirrors synchronized, leaves unhandled portal commands
    bubbling, and remains green with custom portal renderer/registry/measurement coverage.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
  - Result: passed, 8 tests.
  - Scope proven: retained portal lifecycle, controller-first command submission, measurement
    publishing, and measured-internals oracle coverage remains green while the default declarative
    command host carries portal command routing.
- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default package compiles after adding the public declarative portal command
    handler seam and protocol re-exports.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after the default portal
    command host addition.
- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: retained bridge source usage did not spread, and retained widget/portal modules
    remain crate-private while default declarative APIs expose only the command protocol and command
    host seam.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-145` changes only the `fret-node` declarative portal command surface and
    workstream docs. Default/compat targeted tests, compile gates, source-policy gates, formatting,
    layering, catalog, and whitespace checks cover the changed behavior and boundary surface.

## 2026-05-20 - RBX-M2-150 portal text/number command adapter deletion

Claim verified:

- First-party portal text and number editor command handlers now run on the default declarative
  `NodeGraphDeclarativePortalCommandHandler` seam without retained `CommandCx`.
- `portal_text.rs` and `portal_number.rs` no longer contain retained bridge command adapters and
  are no longer allowed in the retained bridge source usage ledger.
- Retained `NodeGraphPortalHost` lifecycle, controller-first command submission, measured-geometry,
  and measured-internals oracle coverage remains green after deleting the text/number retained
  adapters.

Evidence:

- `ecosystem/fret-node/src/ui/editors/portal_text.rs`
- `ecosystem/fret-node/src/ui/editors/portal_number.rs`
- `ecosystem/fret-node/src/ui/editors/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/mod.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
- `ecosystem/fret-node/src/ui/portal.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- Pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx`
  - Result: passed, 5 tests.
  - Scope proven: retained portal lifecycle, controller-first command submission, measured geometry,
    and measured internals were green in the current worktree before deleting the retained
    text/number command adapters.
- `cargo nextest run -p fret-node declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx retained_bridge_source_usage_stays_on_the_migration_ledger editor_chrome_compiles_without_retained_canvas_compat portal_command_session`
  - Result: passed, 6 tests.
  - Scope proven: default text and number portal editor handlers submit binding-backed graph
    transactions without retained `CommandCx`; editor modules compile on the default gate; text and
    number command session policy remains green; retained bridge source usage no longer allows
    `portal_text.rs` / `portal_number.rs`.
- `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 8 tests.
  - Scope proven: retained portal host lifecycle/measurement oracle coverage remains green after
    the retained text/number adapter deletion, and the retained compatibility island/source ledger
    still stays bounded.
- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default package compiles after moving editor handlers onto the declarative
    command seam.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained compatibility island still compiles after the retained text/number
    adapter deletion.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after removing
    `portal_text.rs` / `portal_number.rs` from the retained bridge source ledger.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-150` changes only `fret-node` portal editor command handling and retained
    source-policy documentation. Default/compat targeted tests, compile gates, source-policy gates,
    formatting, layering, catalog, and whitespace checks cover the changed behavior and boundary
    surface.

## 2026-05-20 - RBX-M2-160 retained portal host deletion

Claim verified:

- The retained `NodeGraphPortalHost` and retained portal command-handler adapter surface were
  deleted after the default declarative path proved parity for portal subtree lifecycle keys,
  measured-geometry cleanup/publishing, arbitrary per-kind renderer hosting, portal command
  routing, and first-party text/number editor command submission.
- `src/ui/portal.rs` and the retained portal lifecycle/measurement oracle modules are no longer
  part of the `fret-node` retained bridge source migration ledger.
- The remaining `compat-retained-canvas` island still compiles and the source-policy tests now
  reject reintroducing the retained portal module.

Evidence:

- `ecosystem/fret-node/src/ui/portal.rs` deleted
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs` deleted
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_geometry_conformance.rs` deleted
- `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_internals_conformance.rs`
  deleted
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- Pre-delete detached-HEAD oracle:
  `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx`
  - Result: passed, 12 tests.
  - Scope proven: the retained portal lifecycle, controller-first command submission,
    measured-geometry, and measured-internals oracle tests were green at the deletion base commit
    while the default declarative parity tests were also green.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after deleting the portal
    host module.
- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: default source-policy tests reject retained bridge usage outside the shrunken
    ledger and keep the compatibility island crate-private/controller-bound.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: the same source-policy and compatibility-island gates remain green under the
    retained compatibility feature after the retained portal host deletion.
- `cargo nextest run -p fret-node declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx`
  - Result: passed, 8 tests.
  - Scope proven: default declarative portal lifecycle, measured-geometry cleanup, renderer
    hosting, registry fallback, custom subtree measurement publishing, command routing, and
    text/number editor command submission remain green without constructing retained portal code.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after shrinking
    the `fret-node` retained source ledger.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "NodeGraphPortalHost|NodeGraphPortalCommandHandler|PortalNoopCommandHandler|PortalCommandHandlerChain|portal_lifecycle_conformance|portal_measured_geometry_conformance|portal_measured_internals_conformance|ui/portal\\.rs|mod portal;|pub\\(crate\\) use geometry::node_order" ecosystem/fret-node/src -g '*.rs'`
  - Result: only policy assertions and comments that describe the default replacement remain.
  - Scope proven: no live retained portal host module, retained portal command-handler adapter, or
    retained portal oracle module remains in `fret-node` source.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-160` changes only the `fret-node` retained portal host deletion slice and
    workstream docs. Pre-delete retained/default oracle tests plus post-delete default/compat
    targeted tests, compile gates, source-policy gates, formatting, layering, catalog, and
    whitespace checks cover the changed behavior and boundary surface.

## 2026-05-20 - RBX-M2-170 unused retained editor/panel wrapper deletion

Claim verified:

- Retained `NodeGraphEditor` and `NodeGraphPanel` had no live source consumers outside their own
  deleted files and policy assertions.
- The only retained panel placement math was already covered by the default
  `screen_space_placement::rect_in_bounds` contract.
- `src/ui/editor.rs` and `src/ui/panel.rs` are no longer retained bridge source ledger entries, and
  the remaining compatibility island still compiles.

Evidence:

- `ecosystem/fret-node/src/ui/editor.rs` deleted
- `ecosystem/fret-node/src/ui/panel.rs` deleted
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- Pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas positioned_rect_top_right_respects_margin rect_in_bounds_top_right_respects_margin retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 4 tests.
  - Scope proven: the retained panel wrapper's own placement test, the default placement equivalent,
    and retained source-policy gates were green before deleting the no-user wrappers.
- `rg -n "\b(NodeGraphEditor|NodeGraphPanel|NodeGraphPanelPosition|NodeGraphPanelSize)\b" ecosystem/fret-node/src apps crates ecosystem tools --glob '!target/**' --glob '!ecosystem/fret-node/src/lib.rs'`
  - Result: no matches.
  - Scope proven: no live source consumer still references the deleted retained editor/panel
    wrapper types.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after deleting the
    editor/panel wrapper modules.
- `cargo nextest run -p fret-node rect_in_bounds_top_right_respects_margin retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 3 tests.
  - Scope proven: the default placement contract and shrunken retained source-policy gates remain
    green after deleting the retained wrappers.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: source-policy and compatibility-island gates remain green under the retained
    compatibility feature after deleting editor/panel wrappers.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after shrinking
    the `fret-node` retained source ledger.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-170` deletes no-user retained wrappers in `fret-node` and updates workstream
    docs. No-user proof plus pre-delete placement/source-policy tests, post-delete default/compat
    targeted tests, compile gates, formatting, layering, catalog, and whitespace checks cover the
    changed behavior and boundary surface.

## 2026-05-20 - RBX-M2-180 no-user retained submit/tail/panel paint helper deletion

Claim verified:

- `retained_submit.rs`, `retained_event_tail.rs`, and `panel_button_paint.rs` no longer had live
  consumers after the previous overlay, portal, editor, and panel wrapper deletions.
- `panel_pointer_policy.rs` now contains only default hover/release policy and no retained
  `EventCx` adapter.
- `fret-node` overlay sources stay retained-bridge-free; the retained bridge source ledger now
  allows only the retained canvas widget root, middleware, and `canvas/widget/**`.

Evidence:

- `ecosystem/fret-node/src/ui/retained_submit.rs` deleted
- `ecosystem/fret-node/src/ui/retained_event_tail.rs` deleted
- `ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs` deleted
- `ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/ui/overlays/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- Pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target centered_text_origin_centers_within_button_rect leading_text_origin_keeps_padding_and_vertical_centering retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 6 tests.
  - Scope proven: the retained panel button paint helper tests, default panel pointer policy tests,
    and retained source-policy gates were green before deleting no-user retained helper modules.
- Pre-delete `rg -n "\b(retained_submit|submit_graph_transaction|submit_graph_and_view_transaction|retained_event_tail|request_paint_repaint|finish_paint_event|focus_canvas_and_finish_paint_event|focus_canvas_and_finish_layout_event|finish_portal_command|begin_panel_press|paint_panel_button|paint_panel_label|centered_text_origin|leading_text_origin)\b" ecosystem/fret-node/src apps crates ecosystem tools --glob '!target/**' --glob '!ecosystem/fret-node/src/lib.rs' --glob '!ecosystem/fret-node/src/ui/retained_submit.rs' --glob '!ecosystem/fret-node/src/ui/retained_event_tail.rs' --glob '!ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs' --glob '!ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs'`
  - Result: no live consumers outside module entries and deleted/self files.
  - Scope proven: deleting the retained helper modules does not remove a still-referenced runtime
    path.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after deleting the helper
    modules and retained panel press adapter.
- `cargo nextest run -p fret-node sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 5 tests.
  - Scope proven: default panel pointer policy and overlay/source policy gates remain green after
    overlay helper deletion.
- `cargo nextest run -p fret-node --features compat-retained-canvas sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 5 tests.
  - Scope proven: the same policy gates remain green under `compat-retained-canvas`.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `rg -n "retained_bridge|UiTreeRetainedExt|RetainedSubtreeProps|use fret_ui::retained_bridge|fret_ui::retained_bridge::" ecosystem/fret-node/src/ui/overlays -g '*.rs'`
  - Result: no matches.
  - Scope proven: overlay source files no longer contain retained bridge usage.

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after shrinking
    the `fret-node` retained source ledger.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-180` deletes no-user retained helper modules in `fret-node` and updates
    workstream docs. No-user proof plus pre-delete helper/policy tests, post-delete default/compat
    targeted tests, compile gates, formatting, layering, catalog, and whitespace checks cover the
    changed behavior and boundary surface.

## 2026-05-20 - RBX-M2-190 retained middleware event/command hook removal

Claim verified:

- `NodeGraphCanvasMiddleware` no longer exposes retained `EventCx` / `CommandCx` event and command
  hooks.
- Retained widget command/event dispatch no longer calls middleware hooks before normal canvas
  handling.
- The remaining middleware shape is a `before_commit` transaction guard; its commit-rejection
  behavior stays covered.
- `canvas/middleware.rs` no longer contains retained bridge usage and is no longer part of the
  retained bridge source allowlist.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/middleware.rs`
- `ecosystem/fret-node/src/ui/canvas/middleware/middleware_chain.rs`
- `ecosystem/fret-node/src/ui/canvas/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_command.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_shared.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/middleware_conformance.rs`
- `ecosystem/fret-node/src/ui/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- Pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas middleware_can_override_select_all_command middleware_can_reject_commits_before_apply retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 4 tests.
  - Scope proven: the old retained command override hook and the commit-rejection middleware path
    were green before deleting the event/command hook surface.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after removing middleware
    event/command hooks.
- `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 2 tests.
  - Scope proven: the default retained source policy passes after removing `canvas/middleware.rs`
    from the allowlist.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound middleware_can_reject_commits_before_apply`
  - Result: passed, 3 tests.
  - Scope proven: compat retained source policy passes, and `before_commit` still rejects a
    middleware-blocked transaction before apply.
- `rg -n "retained_bridge|CommandCx|EventCx|NodeGraphCanvasCommandOutcome|NodeGraphCanvasEventOutcome|handle_event\\(|handle_command\\(" ecosystem/fret-node/src/ui/canvas/middleware.rs ecosystem/fret-node/src/ui/canvas/middleware -g '*.rs'`
  - Result: no matches.
  - Scope proven: middleware source no longer depends on retained bridge event/command context
    types or event/command outcome hooks.

- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after shrinking
    the `fret-node` retained source ledger.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-190` narrows `fret-node` canvas middleware and updates workstream docs.
    Pre-delete middleware oracle coverage plus post-delete default/compat targeted tests, compile
    gates, formatting, layering, catalog, and whitespace checks cover the changed behavior and
    boundary surface.

## 2026-05-20 - RBX-M2-200 retained canvas widget tail Cx adapter isolation

Claim verified:

- Canvas widget tail actions for redraw, paint invalidation, and handled-event propagation stop now
  flow through retained-agnostic internal traits in `widget_tail.rs`.
- Retained `EventCx` / `CommandCx` / `LayoutCx` / `PaintCx` implementations for those tail actions
  are isolated in `retained_widget_tail.rs`.
- `paint_invalidation.rs`, `redraw_request.rs`, and `widget_tail.rs` no longer import or name
  retained bridge Cx types, and a default source-policy test locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_shared.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the remaining retained compatibility island compiles after moving tail action
    adapters behind traits.
- `cargo nextest run -p fret-node retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 3 tests.
  - Scope proven: default source-policy gates now reject retained bridge/Cx imports in the extracted
    tail policy helpers and keep the retained widget island crate-private/controller-bound.
- `cargo nextest run -p fret-node --features compat-retained-canvas widget_tail retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 5 tests.
  - Scope proven: compat retained source-policy gates remain green and the new retained-agnostic
    tail helper unit tests prove redraw/paint-invalidation/handled-event side-effect sequencing.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
  - Result: no matches.
  - Scope proven: the extracted tail policy helpers no longer depend on retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    adapter-boundary extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-200` is a narrow `fret-node` retained canvas widget adapter-boundary slice.
    Targeted default/compat source-policy tests, the new unit tests, the compat compile gate, and
    boundary checks cover the changed surface.

## 2026-05-20 - RBX-M2-210 wire-drag commit retained Cx adapter isolation

Claim verified:

- `wire_drag/commit_cx.rs` now defines only the retained-agnostic `WireCommitCx` seam and commit
  invalidation helper.
- Retained `EventCx` / `CommandCx` implementations for wire commit side effects live in
  `wire_drag/retained_commit_cx.rs`.
- The default source-policy gate now includes `wire_drag/commit_cx.rs`, preventing that pure seam
  from reintroducing retained bridge Cx names.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/mod.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving wire commit
    retained Cx implementations to the adapter module.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail commit_cx retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 6 tests.
  - Scope proven: compat source-policy gates remain green, the pure helper files stay off retained
    bridge Cx names, and wire commit invalidation sequencing remains redraw then paint invalidation.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
  - Result: no matches.
  - Scope proven: the extracted tail and wire commit policy helpers no longer depend on retained
    bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the wire commit retained Cx impls into the adapter module.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-210` is a narrow retained adapter relocation inside `fret-node`'s canvas widget
    island. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-220 pointer-up finish retained Cx adapter isolation

Claim verified:

- Pointer-up finish tail behavior now flows through retained-agnostic `PointerCaptureReleaseCx`
  plus `finish_pointer_capture_release(...)`.
- Retained `EventCx` still provides release-pointer-capture behavior, but that implementation is
  isolated in `retained_widget_tail.rs`.
- `pointer_up_finish.rs` and `pointer_up_session/cleanup.rs` no longer import or name retained
  bridge Cx types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-up finish
    release-capture behavior behind the widget tail seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 6 tests.
  - Scope proven: compat source-policy gates remain green, and widget tail unit tests now prove
    release pointer capture followed by redraw and paint invalidation.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs`
  - Result: no matches.
  - Scope proven: extracted tail, wire commit, and pointer-up finish policy helpers no longer
    depend on retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    pointer-up finish seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-220` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-230 sticky-wire finish retained Cx adapter isolation

Claim verified:

- Sticky-wire pointer-down finish behavior now flows through retained-agnostic
  `HandledPointerCaptureReleaseCx` plus `finish_handled_pointer_capture_release(...)`.
- `sticky_wire_connect/finish.rs` no longer imports or names retained bridge Cx types, and the
  default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving sticky-wire
    pointer-down finish side effects behind the widget tail seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 7 tests.
  - Scope proven: compat source-policy gates remain green, and widget tail unit tests now prove
    handled release-capture sequencing: release pointer capture, stop propagation, redraw, and
    paint invalidation.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs`
  - Result: no matches.
  - Scope proven: extracted tail, wire commit, pointer-up finish, and sticky-wire finish policy
    helpers no longer depend on retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    sticky-wire finish seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-230` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-240 edge-insert drag tail retained Cx adapter isolation

Claim verified:

- Edge-insert drag move finish paint invalidation now flows through retained-agnostic
  `WidgetPaintInvalidationCx` plus `invalidate_widget_paint(...)`.
- `edge_insert_drag/drag/tail.rs` no longer imports or names retained bridge Cx types, and the
  default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving edge-insert drag
    move tail invalidation behind the widget tail seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge finish_edge_insert_drag_move_invalidates_paint retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 4 tests.
  - Scope proven: compat source-policy gates remain green, and the new tail unit test proves
    edge-insert drag move finish requests redraw plus paint invalidation.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
  - Result: no matches.
  - Scope proven: extracted tail, wire commit, pointer-up finish, sticky-wire finish, and
    edge-insert drag tail policy helpers no longer depend on retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    edge-insert drag tail seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-240` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-250 cancel cleanup retained Cx adapter isolation

Claim verified:

- Cancel finish tail behavior now flows through retained-agnostic `HandledPointerCaptureReleaseCx`:
  release pointer capture, optionally stop propagation, request redraw, and invalidate paint.
- Retained `cx.app` timer I/O remains in `cancel.rs`, the retained event caller.
- `cancel_cleanup.rs` no longer imports or names retained bridge Cx types, and the default
  source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/cancel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving cancel cleanup
    finish tail behavior behind the widget tail seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas finish_cancel retained_canvas_tail_policy_helpers_stay_off_retained_bridge escape_cancel_releases_pointer_capture_during_panning escape_cancel_emits_connect_end_canceled escape_cancel_panning_emits_move_end_canceled node_drag_start_and_escape_cancel_emits_node_drag_end_canceled retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 9 tests.
  - Scope proven: compat source-policy gates remain green; new cancel cleanup unit tests prove
    consuming and non-consuming finish tail behavior; retained escape-cancel oracle tests still
    prove pointer capture release, connect-end cancellation, pan cancellation, and node-drag
    cancellation behavior.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs`
  - Result: no matches.
  - Scope proven: extracted tail, wire commit, pointer-up finish, sticky-wire finish, edge-insert
    drag tail, and cancel cleanup policy helpers no longer depend on retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    cancel cleanup seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-250` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-260 sticky-wire target picker retained Cx adapter isolation

Claim verified:

- Sticky-wire target picker host/window access plus handled-event finish behavior now flows through
  retained-agnostic `StickyWireTargetPickerCx`.
- Retained `EventCx` implements that seam in `sticky_wire_targets/retained_picker_cx.rs`.
- `sticky_wire_targets/picker.rs` no longer imports or names retained bridge Cx types, and the
  default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/retained_picker_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving sticky-wire target
    picker Cx access behind a retained-agnostic seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas finish_sticky_wire_target_picker_stops_and_invalidates_paint retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 4 tests.
  - Scope proven: compat source-policy gates remain green, and the new target picker unit test
    proves stop propagation plus redraw/paint invalidation sequencing.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
  - Result: no matches.
  - Scope proven: extracted tail, wire commit, pointer-up finish, sticky-wire finish, edge-insert
    drag tail, cancel cleanup, and sticky-wire target picker helpers no longer depend on retained
    bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    sticky-wire target picker seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-260` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-270 group preview tail retained Cx adapter isolation

Claim verified:

- Group drag/resize preview tail paint invalidation now flows through retained-agnostic
  `WidgetPaintInvalidationCx` plus `invalidate_widget_paint(...)`.
- Retained `cx.app` auto-pan view-state I/O remains in `group_drag.rs` / `group_resize.rs`, the
  retained event callers.
- `group_drag/tail.rs` and `group_resize/tail.rs` no longer import or name retained bridge Cx
  types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_drag/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_resize/tail.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving group preview
    tail invalidation behind the widget tail seam while retaining caller-owned auto-pan I/O.
- `cargo nextest run -p fret-node --features compat-retained-canvas update_drag_preview_state update_resize_preview_state group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 9 tests.
  - Scope proven: compat source-policy gates remain green; new group drag/resize tail unit tests
    prove preview state updates and no-op preview revision behavior; existing retained group resize
    oracle coverage still proves preview/commit and child-clamp behavior.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs ecosystem/fret-node/src/ui/canvas/widget/group_drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize/tail.rs`
  - Result: no matches.
  - Scope proven: extracted tail, wire commit, pointer-up finish, sticky-wire finish, edge-insert
    drag tail, cancel cleanup, sticky-wire target picker, and group preview tail helpers no longer
    depend on retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the group
    preview tail seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-270` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-280 group preview move retained Cx adapter isolation

Claim verified:

- Group drag/resize move handler host/bounds access now flows through retained-agnostic
  `GroupPreviewMoveCx`.
- Retained `EventCx` implements that seam in `group_preview_move_retained_cx.rs`.
- `group_drag.rs`, `group_resize.rs`, and `group_preview_move_cx.rs` no longer import or name
  retained bridge Cx types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving group preview
    move handler host/bounds access behind `GroupPreviewMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas group_preview_move_handlers_stay_off_retained_bridge update_drag_preview_state update_resize_preview_state group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 10 tests.
  - Scope proven: the new source-policy gate locks group drag/resize move handlers and the pure
    Cx seam off retained bridge Cx names; existing group preview tail tests and retained group
    resize oracle coverage remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
  - Result: no matches.
  - Scope proven: group preview move handlers and the pure Cx seam no longer depend on retained
    bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the group
    preview move Cx seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-280` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-290 pending group activation retained Cx adapter isolation

Claim verified:

- Pending group drag activation host access now flows through retained-agnostic
  `PendingGroupActivationCx`.
- Retained `EventCx` implements that seam in `pending_group_activation_retained_cx.rs`.
- Pending group resize activation no longer takes an unused retained Cx parameter.
- `pending_group_drag.rs`, `pending_group_resize.rs`, and `pending_group_activation_cx.rs` no
  longer import or name retained bridge Cx types, and the default source-policy gate locks that
  boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pending group
    activation host access behind `PendingGroupActivationCx` and removing the unused pending resize
    Cx parameter.
- `cargo nextest run -p fret-node --features compat-retained-canvas pending_group_activation_handlers_stay_off_retained_bridge group_preview_move_handlers_stay_off_retained_bridge pending_group_drag_release_clears_session_without_committing pending_group_resize_release_clears_session_without_committing group_header_click_selects_group_and_arms_pending_group_drag group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 9 tests.
  - Scope proven: the new source-policy gate locks pending group activation handlers and the pure
    Cx seam off retained bridge Cx names; existing pending group drag/resize release and group
    preview oracle coverage remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs`
  - Result: no matches.
  - Scope proven: pending group activation handlers and the pure Cx seam no longer depend on
    retained bridge Cx names.
- `cargo fmt -p fret-node`
  - Result: passed.
  - Scope proven: touched `fret-node` Rust files were formatted.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    pending group activation seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-290` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-300 pending release retained Cx adapter isolation

Claim verified:

- Pending group drag, pending group resize, and pending node resize release tail actions now flow
  through retained-agnostic `PointerCaptureReleaseCx`.
- Retained `EventCx` continues to implement that seam in `retained_widget_tail.rs`.
- `pointer_up_session/release.rs`, `pointer_up_pending/release.rs`,
  `pointer_up_pending/release/group.rs`, and `pointer_up_pending/release/node.rs` no longer import
  or name retained bridge Cx types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_tail.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pending release
    tail actions behind `PointerCaptureReleaseCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge pending_group_drag_release_clears_session_without_committing pending_group_resize_release_clears_session_without_committing pending_group_activation_handlers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 6 tests.
  - Scope proven: the source-policy gate locks the pending release helpers off retained bridge Cx
    names; existing pending group drag/resize release oracles remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/group.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/node.rs`
  - Result: no matches.
  - Scope proven: pending release helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    pending release seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-300` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-310 pending wire release retained Cx adapter isolation

Claim verified:

- Pending wire drag pointer-up release/promotion tail actions now flow through retained-agnostic
  `PointerCaptureReleaseCx`.
- Retained `EventCx` continues to implement that seam in `retained_widget_tail.rs`.
- `pointer_up_pending/wire_drag.rs` no longer imports or names retained bridge Cx types, and the
  default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_tail.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pending wire
    release tail actions behind `PointerCaptureReleaseCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge should_promote_pending_wire_drag_requires_click_connect_and_new_drag click_connect_target_port_click_commits_wire_and_clears_click_connect_state retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 5 tests.
  - Scope proven: the source-policy gate locks the pending wire release helper off retained bridge
    Cx names; existing pending wire promotion and click-connect commit coverage remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs`
  - Result: no matches.
  - Scope proven: pending wire release helper no longer depends on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    pending wire release seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-310` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-320 pending node drag click-select retained Cx adapter isolation

Claim verified:

- Pending node drag click-select release view-state I/O now flows through retained-agnostic
  `PendingNodeDragReleaseCx`.
- Retained `EventCx` implements that seam in `pending_node_drag_release_retained_cx.rs`.
- `pointer_up_pending/click_select.rs` and `pending_node_drag_release_cx.rs` no longer import or
  name retained bridge Cx types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/click_select.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pending node drag
    click-select release host access behind `PendingNodeDragReleaseCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas pending_node_drag_release_handlers_stay_off_retained_bridge apply_pending_node_selection_toggles_selection_and_keeps_node_last_in_draw_order shift_clicking_a_node_does_not_clear_selection node_click_does_not_select_node_when_node_selectable_is_false retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 6 tests.
  - Scope proven: the new source-policy gate locks pending node drag click-select release helpers
    and the pure Cx seam off retained bridge Cx names; existing click-select selection behavior
    remains green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/click_select.rs ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_cx.rs`
  - Result: no matches.
  - Scope proven: pending node drag click-select release helpers no longer depend on retained
    bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    pending node drag release seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-320` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-330 pointer-up commit retained Cx adapter isolation

Claim verified:

- Group drag, group resize, and node resize pointer-up commit host/window I/O now flows through
  retained-agnostic `PointerUpCommitCx`.
- Retained `EventCx` implements that seam in `pointer_up_commit_retained_cx.rs`.
- Pointer-up commit helpers no longer import or name retained bridge Cx types, and the default
  source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/group_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/node.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-up commit
    host/window access behind `PointerUpCommitCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_commit_handlers_stay_off_retained_bridge build_group_drag_ops_includes_group_and_moved_nodes_only build_node_resize_ops_collects_node_and_group_changes node_resize_expands_group_when_expand_parent_is_true group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 8 tests.
  - Scope proven: the new source-policy gate locks pointer-up commit helpers and the pure Cx seam
    off retained bridge Cx names; existing group drag op building, node resize commit, and group
    resize commit behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/group.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/node.rs`
  - Result: no matches.
  - Scope proven: pointer-up commit helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    pointer-up commit seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-330` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-340 node drag move tail retained Cx adapter isolation

Claim verified:

- Node drag move tail host I/O and paint invalidation now flow through retained-agnostic
  `NodeDragMoveTailCx`.
- Retained `EventCx` implements that seam in `node_drag_move_tail_retained_cx.rs`.
- `node_drag/tail.rs` and `node_drag_move_tail_cx.rs` no longer import or name retained bridge Cx
  types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving node drag move
    tail host access behind `NodeDragMoveTailCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_move_tail_stays_off_retained_bridge node_drag_move_emits_on_node_drag child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true node_drag_records_single_history_entry_for_multi_node_move retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 7 tests.
  - Scope proven: the new source-policy gate locks node drag move tail helpers and the pure Cx seam
    off retained bridge Cx names; existing node drag move callback, group-bound clamp/expand, and
    history behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_cx.rs`
  - Result: no matches.
  - Scope proven: node drag move tail helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the node
    drag move tail seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-340` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-350 marquee begin/finish retained Cx adapter isolation

Claim verified:

- Marquee begin capture/paint invalidation and marquee finish view-state I/O/release tail actions
  now flow through retained-agnostic `MarqueeCx`.
- Retained `EventCx` implements that seam in `marquee_retained_cx.rs`.
- `marquee_begin.rs`, `marquee_cx.rs`, and `marquee_finish.rs` no longer import or name retained
  bridge Cx types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving marquee
    begin/finish host/capture access behind `MarqueeCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas marquee_begin_finish_stays_off_retained_bridge background_click_starts_pending_marquee_and_clears_selection_on_up marquee_replace_mode_replaces_selection_even_with_ctrl_pressed marquee_selects_connected_edges_for_selected_nodes marquee_selects_connected_edges_for_selected_nodes_with_store retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 7 tests.
  - Scope proven: the new source-policy gate locks marquee begin/finish helpers and the pure Cx
    seam off retained bridge Cx names; existing pending marquee clear-selection and marquee
    selection behavior remain green, including store-backed selection.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs`
  - Result: no matches.
  - Scope proven: marquee begin/finish helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    marquee begin/finish seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-350` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-360 node drag preview compute retained Cx adapter isolation

Claim verified:

- Node drag preview host/graph-read I/O now flows through retained-agnostic
  `NodeDragPreviewCx`.
- Retained `EventCx` implements that seam in `node_drag_preview_retained_cx.rs`.
- `node_drag_preview.rs`, `node_drag_preview/compute.rs`, and `node_drag_preview_cx.rs` no longer
  import or name retained bridge Cx types, and the default source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview/compute.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving node drag
    preview host/graph-read access behind `NodeDragPreviewCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_preview_compute_stays_off_retained_bridge node_drag_move_emits_on_node_drag child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true node_drag_records_single_history_entry_for_multi_node_move retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 7 tests.
  - Scope proven: the new source-policy gate locks node drag preview wrapper/compute helpers and
    the pure Cx seam off retained bridge Cx names; existing node drag move callback, parent
    clamp/expansion, multi-node history, retained bridge ledger, and retained compat island gates
    remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview/compute.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_cx.rs`
  - Result: no matches.
  - Scope proven: node drag preview wrapper/compute helpers no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the node
    drag preview seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-360` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-370 node drag geometry retained Cx adapter isolation

Claim verified:

- Node drag snapline geometry reads and multi-drag extent geometry reads now flow through
  retained-agnostic `NodeDragGeometryCx`.
- Retained `EventCx` implements that seam in `node_drag_geometry_retained_cx.rs`.
- `node_drag_constraints.rs`, `node_drag_constraints_extent.rs`, `node_drag_geometry_cx.rs`, and
  `node_drag_snap.rs` no longer import or name retained bridge Cx types, and the default
  source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving node drag
    snapline and extent geometry reads behind `NodeDragGeometryCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_geometry_helpers_stay_off_retained_bridge node_drag_move_emits_on_node_drag node_drag_respects_per_node_extent_rect multi_node_drag_clamps_by_selection_bounds_in_node_extent_rect child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true snap_delta_for_rects_snaps_left_edge snap_delta_for_rects_snaps_center_y retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 10 tests.
  - Scope proven: the new source-policy gate locks node drag geometry helpers and the pure Cx seam
    off retained bridge Cx names; existing node drag move callback, per-node extent, multi-node
    extent, parent clamp/expansion, snapline alignment, retained bridge ledger, and retained compat
    island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_cx.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
  - Result: no matches.
  - Scope proven: node drag geometry helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the node
    drag geometry seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-370` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-380 keyboard pan activation retained Cx adapter isolation

Claim verified:

- Keyboard pan activation key-down/key-up paint invalidation and key-down stop-propagation now
  flow through retained-agnostic `widget_tail` seams.
- Retained `EventCx` already implements those seams through `retained_widget_tail.rs`.
- `keyboard_pan_activation.rs` no longer imports or names retained bridge Cx types, and the default
  source-policy gate locks that boundary.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving keyboard pan
    activation side effects behind retained-agnostic widget tail seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas keyboard_pan_activation_stays_off_retained_bridge space_to_pan_starts_left_mouse_panning_and_updates_viewport pan_activation_key_code_must_match_to_enable_space_to_pan pan_activation_key_code_none_disables_space_to_pan_activation space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
  - Result: passed, 7 tests.
  - Scope proven: the new source-policy gate locks keyboard pan activation off retained bridge Cx
    names; existing space-to-pan key down/up, configured pan activation key, disabled activation,
    scroll pan integration, retained bridge ledger, and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
  - Result: no matches.
  - Scope proven: keyboard pan activation no longer depends on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    keyboard pan activation seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-380` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-390 feedback/motion retained Cx adapter isolation

Claim verified:

- Clipboard feedback host/window access now flows through the retained-agnostic
  `ClipboardFeedbackCx` seam.
- Clipboard paste feedback and timer-motion paint invalidation now flow through retained-agnostic
  widget tail paint invalidation seams.
- Retained `EventCx` is isolated to `event_clipboard_feedback_retained_cx.rs` for clipboard
  feedback and already implements paint invalidation through `retained_widget_tail.rs`.
- Clipboard-unavailable feedback behavior remains intact for matching tokens and side-effect free
  for stale tokens.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/clipboard_conformance.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving feedback and
    motion invalidation helpers behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(feedback_motion_helpers_stay_off_retained_bridge) | test(clipboard_unavailable_with_matching_token_shows_toast_and_invalidates_paint) | test(clipboard_unavailable_with_stale_token_has_no_feedback_side_effects) | test(pan_inertia_emits_move_end_after_inertia_stops) | test(wheel_zoom_emits_move_start_and_debounced_move_end) | test(pinch_zoom_emits_move_start_and_debounced_move_end) | test(wheel_pan_emits_move_start_and_debounced_move_end) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 9 tests.
  - Scope proven: the new source-policy gate locks feedback/motion helpers off retained bridge Cx
    names; clipboard-unavailable feedback still clears matching pending paste, shows the info
    toast, schedules the toast timer, requests redraw, and invalidates paint; stale tokens do not
    produce feedback side effects; existing timer-motion callback tests, retained bridge ledger,
    and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
  - Result: no matches.
  - Scope proven: feedback/motion helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    feedback/motion seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-390` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-400 toast timer retained Cx adapter isolation

Claim verified:

- Expired-toast timer paint invalidation now flows through retained-agnostic
  `WidgetPaintInvalidationCx`.
- `event_timer_toast.rs` no longer imports or names retained bridge Cx types, and the default
  source-policy gate locks that boundary.
- Matching toast timer ticks still clear toast state, request redraw, and invalidate paint; stale
  toast timer ticks remain side-effect free.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/event_timer_toast.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/toast_timer_conformance.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving expired-toast
    timer invalidation behind the retained-agnostic widget tail seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(feedback_motion_helpers_stay_off_retained_bridge) | test(matching_toast_timer_clears_toast_and_invalidates_paint) | test(stale_toast_timer_keeps_toast_without_feedback_side_effects) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 5 tests.
  - Scope proven: the source-policy gate locks feedback/motion/toast timer helpers off retained
    bridge Cx names; matching toast timers clear toast state, request redraw, and invalidate paint;
    stale toast timers do not clear state, request redraw, or invalidate paint; retained bridge
    ledger and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_timer_toast.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
  - Result: no matches.
  - Scope proven: feedback/motion/toast timer helpers no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the toast
    timer seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-400` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-410 pending node resize unused retained Cx parameter removal

Claim verified:

- Pending node resize move handling no longer accepts or names retained bridge Cx types.
- The pointer-move dispatch path still invokes pending node resize threshold/activation handling.
- Pending node resize behavior remains intact: below-threshold movement stays pending, and
  above-threshold movement activates node resize.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/pending_resize_conformance.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after deleting the unused
    retained Cx parameter from pending node resize move handling.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pending_node_resize_move_stays_off_retained_bridge) | test(pending_node_resize_move_below_threshold_keeps_pending_resize) | test(pending_node_resize_move_past_threshold_activates_resize) | test(should_activate_pending_node_resize_respects_threshold) | test(activate_pending_node_resize_moves_pending_into_active) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 7 tests.
  - Scope proven: the source-policy gate locks pending node resize move off retained bridge Cx
    names; direct handler tests prove below-threshold and activation behavior; existing threshold,
    activation, retained bridge ledger, and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs`
  - Result: no matches.
  - Scope proven: pending node resize move no longer depends on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after deleting
    the unused retained Cx parameter.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-410` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-420 edge double-click finish retained Cx adapter isolation

Claim verified:

- Edge double-click finish no longer imports or names retained bridge Cx types.
- Stop-propagation plus paint invalidation now flows through the retained-agnostic
  `WidgetHandledCx` seam.
- Existing edge double-click reroute and insert-picker gesture behavior remains intact.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/finish.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving edge
    double-click finish side effects behind the retained-agnostic widget handled seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(double_click_edge_inserts_reroute_when_enabled) | test(alt_double_click_edge_opens_insert_node_picker) | test(alt_double_click_edge_prefers_picker_over_reroute_when_both_enabled) | test(edge_double_click_finish_stays_off_retained_bridge) | test(finish_double_click_stops_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 7 tests.
  - Scope proven: the source-policy gate locks the edge double-click finish helper off retained
    bridge Cx names; the local tail test proves finish still stops propagation, requests redraw,
    and invalidates paint; existing double-click edge reroute and insert-picker gesture tests,
    retained bridge ledger, and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/finish.rs`
  - Result: no matches.
  - Scope proven: edge double-click finish no longer depends on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the edge double-click finish helper behind the retained-agnostic seam.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-420` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-430 searcher dismiss retained Cx adapter isolation

Claim verified:

- Searcher dismiss release-capture, handled finish, and paint invalidation tails no longer import
  or name retained bridge Cx types.
- Searcher dismiss now uses retained-agnostic `PointerCaptureReleaseCx` /
  `HandledPointerCaptureReleaseCx` seams, while searcher finish/paint invalidation use
  `WidgetHandledCx` / `WidgetPaintInvalidationCx`.
- Searcher dismiss behavior remains intact for overlay and pending row-drag cleanup, pointer
  capture release, handled-event finish, and redraw/paint invalidation.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher dismiss
    tail side effects behind retained-agnostic widget-tail seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(clear_pending_searcher_row_drag_reports_and_clears_state) | test(clear_searcher_overlay_clears_searcher_and_pending_drag) | test(dismiss_searcher_overlay_clears_state_and_releases_capture_without_painting) | test(invalidate_searcher_paint_requests_redraw_and_paint_invalidation) | test(finish_searcher_event_stops_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks searcher dismiss helper files off retained bridge Cx names;
    state cleanup, capture release, finish, redraw, and paint invalidation behavior remain covered;
    retained bridge ledger and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher dismiss helper files no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher dismiss
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the searcher dismiss helper tails behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-430` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-440 searcher row-drag release retained Cx adapter isolation

Claim verified:

- Searcher row-drag release activation/dismiss coordination no longer imports or names retained
  bridge Cx types in `searcher_activation_state/release.rs`.
- Retained row activation now lives behind the retained-agnostic `SearcherReleaseCx` seam, with
  the retained `EventCx` implementation isolated to `searcher_activation_state/release_retained_cx.rs`.
- Searcher release behavior remains intact for no-pending-drag early return, row activation,
  outside dismiss, pointer capture release, handled finish, and redraw/paint invalidation.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher
    row-drag release coordination behind the retained-agnostic `SearcherReleaseCx` seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_release_without_pending_drag_is_side_effect_free) | test(searcher_release_on_row_activates_and_finishes) | test(searcher_release_outside_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks searcher dismiss/release helper files off retained bridge Cx
    names; searcher row-drag release preserves no-pending side-effect-free behavior, row
    activation, outside dismiss, release-capture, handled finish, redraw, and paint invalidation;
    retained bridge ledger and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher dismiss/release helper files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher release
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the searcher release helper behind the retained-agnostic seam.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-440` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-450 searcher row-drag arm retained Cx adapter isolation

Claim verified:

- Searcher row-drag arming no longer imports or names retained bridge Cx types in
  `searcher_activation_state/arm.rs`.
- Retained pointer id, tick id, and pointer capture access now live behind the retained-agnostic
  `SearcherArmCx` seam, with the retained `EventCx` implementation isolated to
  `searcher_activation_state/arm_retained_cx.rs`.
- Searcher arming behavior remains intact for unselectable-row early return, selectable-row active
  row sync, pending insert-node drag creation, pointer id/tick id recording, and pointer capture.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher
    row-drag arming pointer/timer/capture access behind the retained-agnostic `SearcherArmCx`
    seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(arm_searcher_row_drag_rejects_unselectable_row_without_side_effects) | test(arm_searcher_row_drag_records_pending_drag_and_captures_pointer) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 5 tests.
  - Scope proven: source-policy locks searcher arm/dismiss/release helper files off retained bridge
    Cx names; searcher arming preserves unselectable-row no-side-effect behavior and selectable-row
    pending-drag/capture behavior; retained bridge ledger and retained compat island gates remain
    green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher arm/dismiss/release helper files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher arm seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the searcher arm helper behind the retained-agnostic seam.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-450` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-460 searcher pointer-down retained Cx route isolation

Claim verified:

- Searcher pointer-down routing no longer imports or names retained bridge Cx types in
  `searcher_activation/pointer_down.rs`.
- Pointer-down routing now uses a retained-agnostic `SearcherPointerDownCx` capability composed
  from the searcher arm seam and widget-tail dismiss/finish seams.
- Searcher pointer-down behavior remains intact for no-searcher early return, row arm/finish,
  outside dismiss/finish, and secondary-button dismiss/finish.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher
    pointer-down routing behind the retained-agnostic `SearcherPointerDownCx` seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_down_without_searcher_is_side_effect_free) | test(searcher_left_pointer_down_on_row_arms_drag_and_finishes) | test(searcher_left_pointer_down_outside_dismisses_and_finishes) | test(searcher_secondary_pointer_down_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks searcher pointer-down/arm/dismiss/release helper files off
    retained bridge Cx names; pointer-down preserves no-searcher side-effect-free behavior,
    row arm/finish, outside dismiss/finish, and secondary-button dismiss/finish; retained bridge
    ledger and retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher pointer-down/arm/dismiss/release helper files no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher
    pointer-down seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the searcher pointer-down helper behind the retained-agnostic seam.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-460` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-470 searcher pointer-up retained Cx route isolation

Claim verified:

- Searcher pointer-up routing no longer imports or names retained bridge Cx types in
  `searcher_activation/pointer_up.rs`.
- Pointer-up routing now uses the retained-agnostic `SearcherReleaseCx` seam and keeps
  no-searcher pending-drag cleanup as pure interaction-state policy.
- Searcher pointer-up behavior remains intact for non-left button ignore, no-searcher pending-drag
  cleanup, row activation/finish, and outside dismiss/finish.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher
    pointer-up routing behind the retained-agnostic `SearcherReleaseCx` seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_up_ignores_non_left_button) | test(searcher_pointer_up_without_searcher_clears_pending_drag_only) | test(searcher_pointer_up_on_row_activates_and_finishes) | test(searcher_pointer_up_outside_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks searcher pointer-down/up/arm/dismiss/release helper files
    off retained bridge Cx names; pointer-up preserves non-left ignore, no-searcher pending-drag
    cleanup, row activation/finish, and outside dismiss/finish; retained bridge ledger and
    retained compat island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher pointer-down/up/arm/dismiss/release helper files no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher
    pointer-up seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the searcher pointer-up helper behind the retained-agnostic seam.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-470` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-480 outer searcher activation wrapper isolation

Claim verified:

- Outer searcher activation pointer-down/up wrappers no longer import or name retained bridge Cx
  types in `searcher_activation.rs`.
- The pointer-down wrapper now takes `SearcherPointerDownCx`, and the pointer-up wrapper now takes
  `SearcherReleaseCx` directly.
- Retained `EventCx` support remains available through the existing adapter implementations and
  the retained call site in `searcher.rs`; behavior remains covered by focused pointer-down/up
  tests plus source-policy gates.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the outer
    searcher activation wrapper behind retained-agnostic pointer-down/up seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_down_without_searcher_is_side_effect_free) | test(searcher_pointer_up_ignores_non_left_button) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 5 tests.
  - Scope proven: source-policy locks the outer searcher activation wrapper plus pointer-down/up
    helpers off retained bridge Cx names; pointer-down/up smoke behavior and retained ledger/island
    policy gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher activation wrapper and pointer-down/up/arm/dismiss/release helper files
    no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher
    activation wrapper seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the searcher activation wrapper behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-480` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-490 searcher pointer move/wheel retained Cx route isolation

Claim verified:

- Searcher pointer move and wheel routing no longer import or name retained bridge Cx types in
  `searcher_pointer.rs`, `searcher_pointer/move_event.rs`, or
  `searcher_pointer/wheel_event.rs`.
- Searcher pointer move and wheel routes now use the retained-agnostic
  `WidgetPaintInvalidationCx` seam.
- Searcher pointer move/wheel behavior remains intact for no-searcher no-op behavior, hover
  invalidation, repeated-hover no-op behavior, wheel scroll invalidation, plain wheel boundary
  consumption without paint, and Ctrl-wheel pass-through.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher pointer
    move/wheel routing behind the retained-agnostic paint invalidation seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_move_without_searcher_is_side_effect_free) | test(searcher_pointer_move_updates_hover_and_invalidates_paint) | test(searcher_pointer_move_same_hover_does_not_invalidate_paint_again) | test(searcher_wheel_without_searcher_is_side_effect_free) | test(searcher_wheel_scrolls_and_invalidates_paint) | test(searcher_wheel_at_scroll_boundary_consumes_plain_wheel_without_paint) | test(searcher_wheel_with_ctrl_does_not_consume_or_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 10 tests.
  - Scope proven: source-policy locks searcher activation/pointer/dismiss helper files off
    retained bridge Cx names; pointer move/wheel behavior and retained ledger/island gates remain
    green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher activation, pointer, dismiss, and release helper files no longer depend
    on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher pointer
    move/wheel seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    searcher pointer move/wheel routing behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-490` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-500 searcher key-down retained Cx route isolation

Claim verified:

- Searcher key-down routing no longer imports or names retained bridge Cx types in
  `searcher_input.rs`, `searcher_input/dispatch.rs`, or `searcher_input_query.rs`.
- Searcher key-down routing now uses the retained-agnostic `SearcherInputCx` seam.
- Retained row activation I/O remains available through the adapter-only
  `searcher_input/activation_retained_cx.rs`.
- Searcher key behavior remains intact for Enter activation/finish, ArrowDown navigation/finish,
  query update/finish, Ctrl text pass-through, and no-searcher no-op behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/activation_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher
    key-down routing behind `SearcherInputCx` and keeping row activation I/O in the retained
    adapter.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_enter_activates_active_row_and_finishes) | test(searcher_arrow_down_steps_active_row_and_finishes) | test(searcher_text_key_updates_query_and_finishes) | test(searcher_ctrl_text_key_is_not_handled) | test(searcher_key_without_searcher_is_side_effect_free) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks searcher activation/input/pointer/dismiss helper files off
    retained bridge Cx names; key behavior and retained ledger/island gates remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher activation, input, pointer, dismiss, and release helper files no longer
    depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher key-down
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    searcher key-down routing behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-500` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-20 - RBX-M2-510 top-level searcher retained Cx route isolation

Claim verified:

- The top-level searcher escape/key/pointer/wheel route wrapper no longer imports or names retained
  bridge Cx types in `searcher.rs`.
- `searcher.rs` now uses the retained-agnostic `SearcherCx` capability composed from
  `SearcherPointerDownCx`, `SearcherReleaseCx`, and `SearcherInputCx`.
- Retained pointer/timer/capture, row activation, and widget-tail I/O remain available only through
  the existing adapter implementations.
- Searcher top-level route behavior remains intact for Escape dismiss/finish, Enter row activation,
  and pointer-down row drag arming.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the top-level
    searcher route wrapper behind `SearcherCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_top_level_route_stays_off_retained_bridge) | test(searcher_top_level_escape_dismisses_and_finishes) | test(searcher_top_level_key_down_delegates_to_activation_seam) | test(searcher_top_level_pointer_down_arms_row_drag_without_retained_cx) | test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks the top-level searcher route and existing searcher helper
    files off retained bridge Cx names; top-level Escape, key activation, pointer-down arming, and
    retained ledger gates remain green.
- `rm -rf target/debug/incremental`
  - Result: completed after an initial targeted nextest run failed with
    `rustc-LLVM ERROR: IO failure on output stream: No space left on device`.
  - Scope proven: removed only rebuildable Cargo incremental artifacts to restore enough disk space
    for verification. Source and git-tracked files were not removed.
- `cargo fmt`
  - Result: passed.
  - Scope proven: Rust sources were formatted after the searcher route changes.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
  - Result: no matches.
  - Scope proven: searcher top-level route, activation, input, pointer, dismiss, and release helper
    files no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher top-level
    route seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the top-level searcher route behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-510` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-520 context menu UI retained Cx tail isolation

Claim verified:

- Context menu UI open/restore/dismiss/finish/invalidate helpers no longer import or name retained
  bridge Cx types in `context_menu/ui.rs` or `context_menu/ui/event.rs`.
- Context menu UI tail helpers now use retained-agnostic `WidgetHandledCx`,
  `WidgetPaintInvalidationCx`, and `ContextMenuFocusCx`.
- Retained context-menu focus-self I/O remains available through the adapter-only
  `context_menu/ui/event_retained_cx.rs`.
- Context menu UI tail behavior remains intact for open/focus/finish, restore/finish without
  focus, dismiss/finish, no-menu dismiss no-op behavior, and paint invalidation.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event_retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu UI
    tails behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_ui_tail_stays_off_retained_bridge) | test(open_context_menu_event_installs_menu_focuses_and_finishes) | test(restore_context_menu_event_restores_menu_and_finishes_without_focus) | test(dismiss_context_menu_event_clears_menu_and_finishes) | test(dismiss_context_menu_event_without_menu_is_side_effect_free) | test(invalidate_context_menu_paint_requests_redraw_and_paint_invalidation) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks context menu UI tail helpers off retained bridge Cx names;
    open, restore, dismiss, paint invalidation, and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
  - Result: no matches.
  - Scope proven: context menu UI tail helper files no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu UI tail
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu UI tails behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-520` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-530 context menu pointer-move retained Cx route isolation

Claim verified:

- Context menu pointer-move routing no longer imports or names retained bridge Cx types in
  `context_menu/key_navigation/pointer_move.rs`.
- Context menu pointer-move routing now uses the retained-agnostic `WidgetPaintInvalidationCx` seam.
- Context menu pointer-move behavior remains intact for no-menu no-op behavior, hover update paint
  invalidation, and repeated-hover no-op invalidation behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/pointer_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    pointer-move routing behind retained-agnostic paint invalidation.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_pointer_move_route_stays_off_retained_bridge) | test(pointer_move_without_context_menu_is_side_effect_free) | test(pointer_move_updates_hover_and_invalidates_paint) | test(pointer_move_same_hover_does_not_invalidate_paint_again) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks the context menu pointer-move helper off retained bridge Cx
    names; pointer-move hover/invalidation behavior and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/pointer_move.rs`
  - Result: no matches.
  - Scope proven: the context menu pointer-move helper no longer depends on retained bridge Cx
    names.

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    pointer-move seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu pointer-move routing behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-530` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-540 context menu key-down retained Cx route isolation

Claim verified:

- Context menu key-down routing no longer imports or names retained bridge Cx types in
  `context_menu/key_navigation.rs` or `context_menu/key_navigation/key_down.rs`.
- Context menu key-down routing now uses the retained-agnostic `ContextMenuKeyDownCx` seam.
- `RBX-M2-550` superseded the original key-down-specific retained adapter; retained context menu
  item execution I/O now lives in the shared adapter-only
  `context_menu/selection_activation/retained_cx.rs`.
- Context menu key-down behavior remains intact for no-menu no-op behavior, ArrowDown navigation
  and finish, Enter activation and close, Enter keep-open restore, typeahead, and Backspace
  typeahead pop behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    key-down routing behind retained-agnostic active-selection activation.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_key_down_route_stays_off_retained_bridge) | test(key_down_without_context_menu_is_side_effect_free) | test(key_down_arrow_down_advances_active_item_and_finishes) | test(key_down_enter_activates_active_item_and_closes_menu) | test(key_down_enter_keep_open_restores_menu_and_finishes) | test(key_down_typeahead_updates_active_item_and_finishes) | test(key_down_backspace_pops_typeahead_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the context menu key-down route off retained bridge Cx names;
    key-down navigation, activation, typeahead, and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
  - Result: no matches.
  - Scope proven: the context menu key-down route files no longer depend on retained bridge Cx
    names.

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    key-down seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu key-down routing behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-540` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-550 context menu selection activation retained Cx route isolation

Claim verified:

- Context menu shared selection activation no longer imports or names retained bridge Cx types in
  `context_menu/selection_activation.rs`.
- Context menu pointer-down routing no longer imports or names retained bridge Cx types in
  `context_menu/selection_activation/pointer_down.rs`.
- Context menu key-down routing now reuses the shared retained-agnostic
  `ContextMenuSelectionActivationCx` seam instead of a key-down-specific retained adapter.
- Retained context menu item execution I/O remains available through the adapter-only
  `context_menu/selection_activation/retained_cx.rs`.
- Context menu pointer-down behavior remains intact for no-menu no-op behavior, left enabled-item
  activation and close, left disabled-item restore, left outside-menu close, and right-button
  replacement-menu pass-through behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/pointer_down.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/tests.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving selection
    activation and pointer-down routing behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_selection_activation_route_stays_off_retained_bridge) | test(context_menu_key_down_route_stays_off_retained_bridge) | test(pointer_down_without_context_menu_is_side_effect_free) | test(pointer_down_left_inside_enabled_item_activates_and_closes_menu) | test(pointer_down_left_disabled_item_restores_menu_and_finishes) | test(pointer_down_left_outside_menu_closes_menu_and_finishes) | test(pointer_down_right_button_leaves_menu_taken_and_unfinished) | test(key_down_enter_activates_active_item_and_closes_menu) | test(key_down_enter_keep_open_restores_menu_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 10 tests.
  - Scope proven: source-policy locks context menu selection activation and key-down routes off
    retained bridge Cx names; pointer-down activation/restore/close/pass-through behavior, key-down
    activation behavior, and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
  - Result: no matches.
  - Scope proven: the context menu selection activation, pointer-down, and key-down route files no
    longer depend on retained bridge Cx names.

- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    selection activation seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu selection activation behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-550` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-560 context menu top-level retained Cx route isolation

Claim verified:

- Context menu top-level route wrappers no longer import or name retained bridge Cx types in
  `context_menu/mod.rs`.
- Context menu input and pointer wrapper modules now use retained-agnostic route-specific seams
  instead of direct retained `EventCx` signatures.
- `ContextMenuCx` composes the existing key-down and pointer-down seams for top-level routing while
  retained context menu item execution and focus-self I/O remain isolated in adapter-only modules.
- Context menu top-level behavior remains intact for Escape dismiss/finish, Enter active-item
  activation, pointer-down item activation, and pointer-move hover/invalidation.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/input.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/pointer.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    top-level routing behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_top_level_route_stays_off_retained_bridge) | test(context_menu_top_level_escape_dismisses_and_finishes) | test(context_menu_top_level_key_down_delegates_to_activation_seam) | test(context_menu_top_level_pointer_down_delegates_to_selection_activation) | test(context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint) | test(context_menu_selection_activation_route_stays_off_retained_bridge) | test(context_menu_key_down_route_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the top-level context menu route off retained bridge Cx
    names; Escape, Enter activation, pointer-down activation, pointer-move hover/invalidation, and
    retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/input.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/pointer.rs`
  - Result: no matches.
  - Scope proven: the context menu top-level route files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    top-level seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu top-level routing behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-560` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-570 context menu opening retained Cx route isolation

Claim verified:

- Context menu opening route helpers no longer import or name retained bridge Cx types in
  `context_menu/opening.rs` or its background/group/edge target helpers.
- `ContextMenuOpeningCx` carries the opening route's host, bounds, window availability, focus, and
  handled-finish capabilities while retained `EventCx` field access stays isolated in
  `context_menu/opening/retained_cx.rs`.
- Retained-path right-click opening behavior remains intact for background, group, and edge targets,
  including background item enablement, group selection, and edge selection/menu items.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/background.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/edge.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/group.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_menu_searcher_conformance.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    opening routes behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_opening_route_stays_off_retained_bridge) | test(right_click_background_opens_background_context_menu_with_paste_disabled_without_window) | test(right_click_group_opens_group_context_menu_and_selects_group) | test(right_click_edge_opens_edge_context_menu_and_selects_edge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 5 tests.
  - Scope proven: source-policy locks the context menu opening route off retained bridge Cx names;
    retained-path background, group, and edge right-click opening behavior and retained ledger
    behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/background.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/edge.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/group.rs`
  - Result: no matches.
  - Scope proven: the context menu opening route files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    opening seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu opening routes behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-570` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-580 context menu action activation retained Cx route isolation

Claim verified:

- Context menu item activation routing no longer imports or names retained bridge Cx types in
  `context_menu/activate.rs`, `context_menu/activate/command.rs`, or
  `context_menu/activate/target.rs`.
- `ContextMenuActionCx` composes command and target action seams for command dispatch, group
  selection sync, and target-specific background/edge/connection executor calls while retained
  `EventCx` access stays isolated in `context_menu/activate/retained_cx.rs`.
- Context menu item activation behavior remains intact for command selection-before-dispatch,
  non-command target delegation, ignored target actions, and retained-path key/pointer item
  activation.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/command.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/target.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    activation routes behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_activation_route_stays_off_retained_bridge) | test(command_items_select_group_before_dispatching_command) | test(non_command_items_delegate_to_target_action_executor) | test(ignored_target_actions_are_side_effect_free) | test(pointer_down_left_inside_enabled_item_activates_and_closes_menu) | test(key_down_enter_activates_active_item_and_closes_menu) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks the context menu activation route off retained bridge Cx
    names; command, target, ignored-action, pointer-down, key-down, and retained ledger behavior
    remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/command.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/target.rs`
  - Result: no matches.
  - Scope proven: the context menu activation route files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    activation seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu activation routes behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-580` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-590 context menu background execution retained Cx route isolation

Claim verified:

- Context menu background insert execution no longer imports or names retained bridge Cx types in
  `context_menu/background_execution.rs`, `context_menu/background_execution/activate.rs`, or
  `context_menu/background_execution/apply.rs`.
- `BackgroundInsertMenuCx` carries the background executor's host/window access while retained
  `EventCx` field access stays isolated in `context_menu/background_execution/retained_cx.rs`.
- Background insert action behavior remains intact for missing candidates, ignored non-candidate
  actions, rejected candidates, toast surfacing, recent-kind recording, and retained ledger
  behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/tests.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    background execution behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_background_execution_stays_off_retained_bridge) | test(background_insert_menu_plan_surfaces_create_node_errors) | test(background_insert_action_with_missing_candidate_is_handled_without_side_effects) | test(background_insert_action_ignores_non_candidate_actions) | test(background_insert_action_records_candidate_and_surfaces_rejection_toast) | test(context_menu_activation_route_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks background execution off retained bridge Cx names; plan
    rejection, candidate gating, ignored action, rejection toast, activation route, and retained
    ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/apply.rs`
  - Result: no matches.
  - Scope proven: the context menu background execution route files no longer depend on retained
    bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    background execution seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu background execution behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-590` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-600 context menu edge execution retained Cx route isolation

Claim verified:

- Context menu edge execution no longer imports or names retained bridge Cx types in
  `context_menu/edge_execution.rs` or its open-insert/reroute/delete/custom helper files.
- `EdgeContextActionCx` carries the edge executor's host/window access plus edge insert menu opening
  hook while retained `EventCx` field access stays isolated in
  `context_menu/edge_execution/retained_cx.rs`.
- Edge context action behavior remains intact for open insert menu delegation, delete-edge graph and
  selection updates, reroute insertion, custom presenter ops, ignored non-edge actions, and retained
  ledger behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/open_insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/reroute.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/delete.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/custom_action.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu edge
    execution behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_edge_execution_stays_off_retained_bridge) | test(open_insert_action_delegates_to_context_adapter) | test(delete_edge_action_removes_edge_and_selection) | test(insert_reroute_action_splits_edge_and_selects_inserted_node) | test(custom_edge_action_applies_presenter_ops) | test(ignored_edge_actions_are_side_effect_free) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks edge execution off retained bridge Cx names; open-insert,
    delete, reroute, custom action, ignored action, and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/open_insert.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/reroute.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/delete.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/custom_action.rs`
  - Result: no matches.
  - Scope proven: the context menu edge execution route files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu edge
    execution seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu edge execution behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-600` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-610 context menu connection insert execution retained Cx route isolation

Claim verified:

- Context menu connection insert execution no longer imports or names retained bridge Cx types in
  `context_menu/connection_execution_insert.rs` or its activate/apply/recovery helper files.
- `ConnectionInsertMenuCx` carries the connection insert executor's host/window access plus
  wire-drag resume/restore hooks while retained `EventCx` field access stays isolated in
  `context_menu/connection_execution_insert/retained_cx.rs`.
- Connection insert action behavior remains intact for missing candidates, ignored non-candidate
  actions, rejected candidates, toast surfacing, recent-kind recording, wire-drag restore/resume,
  and retained ledger behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    connection insert execution behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_connection_insert_execution_stays_off_retained_bridge) | test(connection_insert_action_with_missing_candidate_is_handled_without_side_effects) | test(connection_insert_action_ignores_non_candidate_actions) | test(connection_insert_action_records_candidate_and_restores_on_rejection) | test(connection_insert_apply_success_resumes_wire_drag) | test(connection_insert_apply_ignore_restores_wire_drag) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks connection insert execution off retained bridge Cx names;
    missing-candidate, ignored action, rejection toast/restore, success resume, ignore restore, and
    retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/apply.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs`
  - Result: no matches.
  - Scope proven: the context menu connection insert execution route files no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    connection insert seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu connection insert execution behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-610` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-620 context menu connection conversion execution retained Cx route isolation

Claim verified:

- Context menu connection conversion execution no longer imports or names retained bridge Cx types
  in `context_menu/connection_execution_conversion.rs` or its activate/apply helper files.
- `ConnectionConversionMenuCx` carries the connection conversion executor's host/window access plus
  wire-drag restore hook while retained `EventCx` field access stays isolated in
  `context_menu/connection_execution_conversion/retained_cx.rs`.
- Connection conversion action behavior remains intact for missing candidates, ignored
  non-candidate actions, rejected candidates, toast surfacing, recent-kind recording, successful
  conversion apply/selection, suspended wire-drag clearing, ignore restore, and retained ledger
  behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving context menu
    connection conversion execution behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_connection_conversion_execution_stays_off_retained_bridge) | test(connection_conversion_action_with_missing_candidate_is_handled_without_side_effects) | test(connection_conversion_action_ignores_non_candidate_actions) | test(connection_conversion_action_records_candidate_and_restores_on_rejection) | test(connection_conversion_apply_success_clears_suspended_drag_and_selects_node) | test(connection_conversion_apply_ignore_restores_wire_drag) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks connection conversion execution off retained bridge Cx names;
    missing-candidate, ignored action, rejection toast/restore, success apply/selection,
    suspended-drag clearing, ignore restore, and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs`
  - Result: no matches.
  - Scope proven: the context menu connection conversion execution route files no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the context menu
    connection conversion seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    context menu connection conversion execution behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-620` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-630 searcher row activation retained Cx route isolation

Claim verified:

- Searcher row activation no longer imports or names retained bridge Cx types in
  `searcher_row_activation.rs`.
- `SearcherRowActivationCx` carries the row activation executor's context-menu item activation
  side effect while retained `EventCx` field access stays isolated in
  `searcher_row_activation/retained_cx.rs`.
- Searcher row activation behavior remains intact for no-searcher no-op handling, unactivatable
  row restoration, candidate-row delegation to context-menu action activation, and retained ledger
  behavior.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/searcher_logic.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation/retained_cx.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving searcher row
    activation behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_row_activation_route_stays_off_retained_bridge) | test(searcher_row_activation_without_searcher_is_side_effect_free) | test(searcher_row_activation_restores_unactivatable_row) | test(searcher_row_activation_delegates_candidate_item_to_context_action) | test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks searcher row activation off retained bridge Cx names;
    no-searcher, unactivatable-row restore, candidate row delegation, searcher helper
    source-policy, and retained ledger behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_logic.rs`
  - Result: no matches.
  - Scope proven: the searcher row activation route files no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the searcher row
    activation seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    searcher row activation behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-630` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-640 right-click retained Cx route isolation

Claim verified:

- Right-click context-menu routing no longer imports or names retained bridge Cx types in
  `right_click.rs` or `right_click/pending.rs`.
- `RightClickCx` composes the existing retained-agnostic context-menu opening and pointer-capture
  release capabilities needed by right-click pointer-down/up routing.
- Pending right-click release planning remains intact for ignored inputs, missing pending state,
  drag release cleanup, and click release menu opening.
- Retained right-click context-menu behavior remains intact for direct right-click opening,
  deferred right-pan opening, drag-threshold rejection, and background/group/edge menu targets.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/right_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/right_click/pending.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving right-click
    routing behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(right_click_route_stays_off_retained_bridge) | test(pending_right_click_pointer_up_ignores_non_right_button) | test(pending_right_click_pointer_up_without_pending_state_is_side_effect_free) | test(pending_right_click_drag_release_clears_pending_and_releases_capture) | test(pending_right_click_click_release_requests_menu_open) | test(right_click_cancels_wire_drag_and_opens_context_menu) | test(right_pan_defers_context_menu_until_pointer_up) | test(right_pan_drag_does_not_open_context_menu) | test(right_click_background_opens_background_context_menu_with_paste_disabled_without_window) | test(right_click_group_opens_group_context_menu_and_selects_group) | test(right_click_edge_opens_edge_context_menu_and_selects_edge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 12 tests.
  - Scope proven: source-policy locks right-click routing off retained bridge Cx names; pending
    release planning and retained right-click background/group/edge context-menu behavior remain
    green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/right_click.rs ecosystem/fret-node/src/ui/canvas/widget/right_click/pending.rs`
  - Result: no matches.
  - Scope proven: the right-click route files no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the right-click seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    right-click routing behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-640` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-650 pointer-up guard dispatch retained Cx route isolation

Claim verified:

- Pointer-up guard arbitration no longer imports or names retained bridge Cx types in
  `event_pointer_up/dispatch.rs`.
- `PointerUpGuardCx` composes the existing retained-agnostic right-click and searcher seams for
  early pointer-up guard handling.
- The full retained fallback pointer-up path remains explicit in `event_pointer_up.rs` and is not
  hidden inside a retained-agnostic guard helper.
- Right-click pending click release, deferred right-pan menu opening, and searcher pointer-up
  guard behavior remain green.

Evidence:

- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
- `ecosystem/fret-node/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-up guard
    dispatch behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(right_click_route_stays_off_retained_bridge) | test(pending_right_click_click_release_requests_menu_open) | test(right_pan_defers_context_menu_until_pointer_up) | test(searcher_pointer_up_on_row_activates_and_finishes) | test(searcher_pointer_up_without_searcher_clears_pending_drag_only) | test(searcher_pointer_up_outside_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks pointer-up guard dispatch off retained bridge Cx names;
    right-click and searcher pointer-up guard behaviors remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
  - Result: no matches.
  - Scope proven: pointer-up guard dispatch no longer depends on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up guard
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    pointer-up guard dispatch behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-650` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-660 pointer-up release retained Cx route isolation

Claim verified:

- Pointer-up release routing and release state helpers no longer import or name retained bridge Cx
  types.
- Sticky-wire ignored release uses a paint-invalidation capability, and pan release uses the
  retained-agnostic `PointerUpReleaseCx` seam for host/window access plus pointer capture release.
- Retained `EventCx` adaptation is isolated in `pointer_up_release_retained_cx.rs`.
- Sticky-wire ignored release, pan inertia release, and right-pan context-menu behavior remain
  green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_state/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Red evidence:

- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_release_route_stays_off_retained_bridge`
  - Result: failed before implementation, 1 failed test.
  - Failure: `pointer-up release route must stay retained-Cx agnostic; found retained_bridge`.
  - Scope proven: the new source-policy test caught retained bridge naming in pointer-up release
    helpers before the seam extraction.

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-up
    release helpers behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_release_route_stays_off_retained_bridge) | test(sticky_wire_ignored_left_pointer_up_clears_ignore_and_invalidates_paint) | test(pan_inertia_emits_move_end_after_inertia_stops) | test(right_pan_defers_context_menu_until_pointer_up) | test(right_pan_drag_does_not_open_context_menu) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks pointer-up release helpers off retained bridge Cx names;
    sticky-wire ignored release, pan inertia release, right-pan context-menu deferral, and right-pan
    drag behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_cx.rs`
  - Result: no matches.
  - Scope proven: pointer-up release route/state helpers and pure seam no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up release
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    pointer-up release helpers behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-660` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-670 pointer-up left double-click retained Cx route isolation

Claim verified:

- The plain double-click edge-insert pointer-up subroute no longer imports or names retained bridge
  Cx types.
- The subroute reuses `PointerUpReleaseCx` for host/window access, pointer capture release, and
  paint invalidation.
- The real pointer-up path still opens the edge insert picker, clears edge-drag hover state, and
  invalidates paint.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/double_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_insert_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Red evidence:

- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_left_double_click_route_stays_off_retained_bridge`
  - Result: failed before implementation, 1 failed test.
  - Failure: `pointer-up left double-click route must stay retained-Cx agnostic; found retained_bridge`.
  - Scope proven: the new source-policy test caught retained bridge naming in the left
    double-click pointer-up helper before the seam extraction.

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the left
    double-click pointer-up subroute behind `PointerUpReleaseCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_left_double_click_route_stays_off_retained_bridge) | test(plain_double_click_edge_insert_left_up_opens_picker_and_invalidates_paint) | test(should_open_edge_insert_picker_requires_plain_double_click) | test(edge_insert_left_up_does_not_open_picker_when_searcher_is_open) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 5 tests.
  - Scope proven: source-policy locks the left double-click pointer-up helper off retained bridge
    Cx names; the real pointer-up path still opens the edge insert picker and invalidates paint.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/double_click.rs`
  - Result: no matches.
  - Scope proven: the left double-click pointer-up helper no longer depends on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the left double-click
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the left double-click pointer-up helper behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-670` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-680 pointer-up commit dispatch retained Cx route isolation

Claim verified:

- Pointer-up commit dispatch helpers no longer import or name retained bridge Cx types in
  `pointer_up_commit.rs`, `pointer_up_node_drag.rs`, or
  `pointer_up_left_route/dispatch/commit.rs`.
- The commit release chain reuses `PointerUpCommitCx` for host/window access plus pointer-up tail
  side effects.
- Retained `EventCx` adaptation remains isolated in `pointer_up_commit_retained_cx.rs`.
- Node-drag release and group resize commit behavior remain green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Red evidence:

- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_commit_handlers_stay_off_retained_bridge`
  - Result: failed before implementation, 1 failed test.
  - Failure: source-policy found `retained_bridge` in the newly added pointer-up commit source set.
  - Scope proven: the source-policy gate caught direct retained Cx naming before the seam
    extraction.

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the commit
    dispatch helpers behind `PointerUpCommitCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_commit_handlers_stay_off_retained_bridge) | test(node_drag_pointer_up_emits_node_drag_end_committed) | test(node_drag_end_batches_group_rect_ops_in_sorted_group_id_order) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(group_resize_clamps_to_children) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks pointer-up commit helpers off retained bridge Cx names; node
    drag commit release and group resize commit behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs`
  - Result: no matches.
  - Scope proven: the migrated pointer-up commit helpers no longer depend on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up commit
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    pointer-up commit helpers behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-680` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-690 pointer-up pending dispatch retained Cx route isolation

Claim verified:

- Pointer-up pending dispatch no longer imports or names retained bridge Cx types in
  `pointer_up_left_route/dispatch/pending.rs`.
- The pending release chain reuses `PendingNodeDragReleaseCx` for pending node selection host access
  plus pointer-capture release and paint invalidation.
- Pending group drag/resize release, pending node click-select release, and pending wire-drag
  promotion remain green through the real `pointer_up::handle_pointer_up` path.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Red evidence:

- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_pending_dispatch_stays_off_retained_bridge`
  - Result: failed before implementation, 1 failed test.
  - Failure: `pointer-up pending dispatch must stay retained-Cx agnostic; found retained_bridge`.
  - Scope proven: the new source-policy test caught direct retained Cx naming in pending dispatch
    before the seam extraction.

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pending dispatch
    behind `PendingNodeDragReleaseCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_pending_dispatch_stays_off_retained_bridge) | test(pending_group_drag_release_clears_session_without_committing) | test(pending_group_resize_release_clears_session_without_committing) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(pending_wire_drag_release_promotes_to_active_wire_drag_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks pointer-up pending dispatch off retained bridge Cx names;
    pending group drag/resize release, pending node click-select release, and pending wire-drag
    promotion still work through the real pointer-up route.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs`
  - Result: no matches.
  - Scope proven: the migrated pointer-up pending dispatch helper no longer depends on retained
    bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up pending
    seam changes and behavior tests.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    pointer-up pending dispatch behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-690` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-700 pointer-up active dispatch retained Cx route isolation

Claim verified:

- Pointer-up active dispatch no longer imports or names retained bridge Cx types in
  `pointer_up_left_route/dispatch/active.rs`.
- Direct active pointer-up leaf helpers for edge insert and edge drag no longer import or name
  retained bridge Cx types.
- Active dispatch composes existing `WireCommitCx` with `PointerUpReleaseCx`; retained `EventCx`
  adaptation remains in the existing retained adapter files.
- Wire left-up, edge-insert left-up, and edge-drag left-up behavior remain green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/active.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/active.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_drag_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Red evidence:

- `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_active_dispatch_stays_off_retained_bridge`
  - Result: failed before implementation, 1 failed test.
  - Failure: `pointer-up active dispatch must stay retained-Cx agnostic; found retained_bridge`.
  - Scope proven: the new source-policy test caught direct retained Cx naming in active dispatch
    and active leaf helpers before the seam extraction.

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving active dispatch
    and active leaf helpers behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_active_dispatch_stays_off_retained_bridge) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_insert_left_up_does_not_open_picker_when_searcher_is_open) | test(plain_double_click_edge_insert_left_up_opens_picker_and_invalidates_paint) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks active pointer-up dispatch and direct active leaf helpers off
    retained bridge Cx names; wire left-up, edge-insert left-up, and edge-drag left-up behavior
    remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/active.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/active.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/pointer_up.rs`
  - Result: no matches.
  - Scope proven: the migrated active pointer-up dispatch helpers no longer depend on retained
    bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up active
    seam changes and behavior test.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    pointer-up active dispatch behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-700` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-710 pointer-up route wrapper retained Cx isolation

Claim verified:

- The top-level pointer-up fallback route wrappers no longer import or name retained bridge Cx
  types in `pointer_up.rs`, `pointer_up/left.rs`, `pointer_up_left_route.rs`, or
  `pointer_up_left_route/dispatch.rs`.
- The `marquee.rs` forwarding path now stays retained-Cx agnostic through marquee begin, move,
  selection, pending-to-select, pending-to-pan, and left-up finish helpers.
- Pan begin behavior used by marquee pending-to-pan promotion now goes through the narrow
  retained-agnostic `PanZoomBeginCx` seam. The retained pan-move route remains a later slice.
- Real pointer-up, marquee selection, and panning behavior remain green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up/left.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-up route
    wrappers, marquee move helpers, and pan-begin helpers behind retained-agnostic seams.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(marquee_begin_finish_stays_off_retained_bridge) | test(marquee_move_handlers_stay_off_retained_bridge) | test(pan_zoom_begin_helpers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(marquee_replace_mode_replaces_selection_even_with_ctrl_pressed) | test(middle_mouse_panning_tracks_screen_delta_under_render_transform) | test(panning_emits_move_start_and_move_end) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 12 tests.
  - Scope proven: source-policy locks pointer-up route wrappers, marquee move helpers, and
    pan-begin helpers off retained bridge Cx names; marquee begin/up, marquee selection,
    pending-node click-select release, edge reconnect release, edge-drag left-up cleanup, and
    panning behavior remain green through the real retained compatibility route.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up/left.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/marquee.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated route wrappers and helper seams no longer depend on retained bridge
    Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up route,
    marquee, and pan-begin seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    wrapper/helpers behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after the user's pull.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-710` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-720 pointer-up event entry retained Cx isolation

Claim verified:

- `NodeGraphCanvasWith::handle_pointer_up(...)` no longer imports or names retained bridge Cx
  types in `event_pointer_up.rs`.
- The event entry composes the already-migrated guard and pointer-up route capabilities through
  `PointerUpRouteCx`.
- The upper pointer-event parser remains retained-bound for a later route isolation slice.
- Real pointer-up guard dispatch, marquee release, pending node click-select release, edge
  reconnect release, and edge-drag left-up cleanup remain green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Red evidence:

- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_event_entry_stays_off_retained_bridge) | test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: failed before the final rename, 1 failed test.
  - Failure: `pointer-up event entry must stay retained-Cx agnostic; found EventCx`.
  - Scope proven: the source-policy test caught retained Cx naming in the event-entry seam name
    before the final `PointerUpRouteCx` cleanup.

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the pointer-up
    event entry behind the composed route seam, with no new `fret-node` private-bound warning after
    widening `PointerUpGuardCx` to widget-internal visibility.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_event_entry_stays_off_retained_bridge) | test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the pointer-up event entry, guard dispatch, and route
    wrappers off retained bridge Cx names; real pointer-up guard, marquee, pending-node, edge
    reconnect, and edge-drag left-up behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
  - Result: no matches.
  - Scope proven: the migrated pointer-up event entry and guard dispatch no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up event
    entry seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the pointer-up event entry behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after the user's pull.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-720` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-730 pointer-up button router retained Cx isolation

Claim verified:

- `event_router_pointer_button/up.rs` no longer imports or names retained bridge Cx types.
- Pointer-up button routing now reuses the composed `PointerUpRouteCx` seam from
  `event_pointer_up.rs`.
- The upper button router and its pointer-down/pointer-move branches remain retained-bound for
  later route isolation slices.
- Real pointer-up guard dispatch, marquee release, pending node click-select release, edge
  reconnect release, and edge-drag left-up cleanup remain green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/up.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the pointer-up
    button router behind the composed route seam.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_button_router_stays_off_retained_bridge) | test(pointer_up_event_entry_stays_off_retained_bridge) | test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 9 tests.
  - Scope proven: source-policy locks the pointer-up button router, pointer-up event entry, guard
    dispatch, and route wrappers off retained bridge Cx names; real pointer-up guard, marquee,
    pending-node, edge reconnect, and edge-drag left-up behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/up.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
  - Result: no matches.
  - Scope proven: the migrated pointer-up button router and related pointer-up entry helpers no
    longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pointer-up button
    router seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after moving
    the pointer-up button router behind retained-agnostic seams.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-730` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-21 - RBX-M2-740 pan-zoom move retained Cx isolation

Claim verified:

- `pan_zoom_move.rs` no longer imports or names retained bridge Cx types.
- `pan_zoom.rs` now routes panning move through the retained-agnostic `PanZoomCx` seam.
- `PanZoomBeginCx` remains the begin-only extension over `PanZoomCx` for pointer capture.
- Middle-mouse panning, space-to-pan panning, panning move start/end callbacks, and pan inertia end
  behavior remain green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving panning move
    behind `PanZoomCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pan_zoom_move_helpers_stay_off_retained_bridge) | test(pan_zoom_begin_helpers_stay_off_retained_bridge) | test(middle_mouse_panning_tracks_screen_delta_under_render_transform) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(panning_emits_move_start_and_move_end) | test(pan_inertia_emits_move_end_after_inertia_stops) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks pan-zoom begin/move helpers off retained bridge Cx names;
    real middle-mouse panning, space-to-pan panning, panning callback, and pan inertia end behavior
    remain green through the retained compatibility island.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs`
  - Result: no matches.
  - Scope proven: the pan-zoom wrapper, begin helper, shared capability seam, and move helper no
    longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after formatting the pan-zoom move seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the pan-zoom move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after the user's pull and this
    slice's updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-740` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-750 pointer-move primary surface retained Cx isolation

Claim verified:

- `pointer_move_dispatch/primary/surface.rs` no longer imports or names retained bridge Cx types.
- The primary surface pointer-move route now uses the retained-agnostic `MarqueeCx` capability,
  which already covers pan begin/move and marquee begin/move side effects.
- The surrounding primary pointer-move router plus group/node/connection branches remain
  retained-bound for later route isolation slices.
- Panning move and marquee move behavior remain green through the retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/surface.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the primary
    surface pointer-move route behind `MarqueeCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_surface_route_stays_off_retained_bridge) | test(pan_zoom_move_helpers_stay_off_retained_bridge) | test(marquee_move_handlers_stay_off_retained_bridge) | test(middle_mouse_panning_tracks_screen_delta_under_render_transform) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(marquee_replace_mode_replaces_selection_even_with_ctrl_pressed) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the primary surface pointer-move route, pan move helpers, and
    marquee move helpers off retained bridge Cx names; real panning and marquee behavior remain
    green through the retained compatibility island.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/surface.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs ecosystem/fret-node/src/ui/canvas/widget/marquee.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs`
  - Result: no matches.
  - Scope proven: the migrated surface pointer-move route and called pan/marquee helpers no longer
    depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the pointer-move surface seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the primary surface pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-750` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-760 pointer-move primary group retained Cx isolation

Claim verified:

- `pointer_move_dispatch/primary/group.rs` no longer imports or names retained bridge Cx types.
- The primary group pointer-move route now composes the retained-agnostic
  `PendingGroupActivationCx` and `GroupPreviewMoveCx` capabilities.
- The surrounding primary pointer-move router plus node/connection branches remain retained-bound
  for later route isolation slices.
- Pending group drag activation and group resize preview/commit behavior remain green through the
  retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the primary group
    pointer-move route behind retained-agnostic capabilities.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_group_route_stays_off_retained_bridge) | test(group_preview_move_handlers_stay_off_retained_bridge) | test(pending_group_activation_handlers_stay_off_retained_bridge) | test(group_header_click_selects_group_and_arms_pending_group_drag) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(group_resize_clamps_to_children) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 7 tests.
  - Scope proven: source-policy locks the primary group pointer-move route, pending group activation
    helpers, and group preview move helpers off retained bridge Cx names; real pending group drag
    activation and group resize preview/commit behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated group pointer-move route and called group helpers no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the pointer-move group route seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the primary group pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-760` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-770 pointer-move primary node retained Cx isolation

Claim verified:

- `pointer_move_dispatch/primary/node.rs` no longer imports or names retained bridge Cx types.
- Pending node drag activation now uses the retained-agnostic `PendingNodeDragActivationCx`
  capability for host access plus pointer-capture release.
- `pending_drag_session::abort_pending_node_drag(...)` now only needs the existing
  `PointerCaptureReleaseCx` tail seam.
- Pending node resize move remains retained-free without an event Cx parameter.
- The surrounding primary pointer-move router plus connection branch remain retained-bound for
  later route isolation slices.
- Node drag activation/cancel, node drag threshold, and pending node resize activation behavior
  remain green through the retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_drag_session/node.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_activation_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_activation_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the primary node
    pointer-move route behind `PendingNodeDragActivationCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_node_route_stays_off_retained_bridge) | test(pending_node_drag_activation_handlers_stay_off_retained_bridge) | test(pending_node_resize_move_stays_off_retained_bridge) | test(node_drag_does_not_start_when_nodes_draggable_is_false) | test(node_drag_start_and_escape_cancel_emits_node_drag_end_canceled) | test(node_drag_threshold_is_zoom_invariant_in_screen_space) | test(pending_node_resize_move_past_threshold_activates_resize) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the primary node pointer-move route, pending node drag
    activation helper, and pending node resize move helper off retained bridge Cx names; real node
    drag activation/cancel, threshold, and pending resize behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs ecosystem/fret-node/src/ui/canvas/widget/pending_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_activation_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_drag_session/node.rs`
  - Result: no matches.
  - Scope proven: the migrated node pointer-move route and called pending-node helpers no longer
    depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the pointer-move node route seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the primary node pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after the user's pull and this
    slice's updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-770` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-780 pointer-move primary connection retained Cx isolation

Claim verified:

- `pointer_move_dispatch/primary/connection.rs` no longer imports or names retained bridge Cx
  types.
- Pending and active wire-drag move handling now use the retained-agnostic `WireDragMoveCx`
  capability for host access, bounds, and paint invalidation.
- Retained `EventCx` adaptation is isolated in `wire_drag_move_retained_cx.rs`.
- Pending and active edge-insert move helpers now use the existing `WidgetPaintInvalidationCx`
  seam instead of retained `EventCx`.
- The primary pointer-move wrapper remains retained-bound for a later route isolation slice, but
  the surface, group, node, and connection branches now all have retained-agnostic seams.
- Wire drag hover/threshold and pending edge-insert threshold behavior remain green through the
  retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pending_wire_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_move_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/auto_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/prelude.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending/activate.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/prelude.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/connection.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the primary
    connection pointer-move route behind `WireDragMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_connection_route_stays_off_retained_bridge) | test(wire_drag_move_handlers_stay_off_retained_bridge) | test(edge_insert_move_handlers_stay_off_retained_bridge) | test(wire_drag_hover_marks_valid_target_port_as_valid) | test(wire_drag_hover_tracks_invalid_port_in_strict_mode) | test(connection_drag_threshold_is_zoom_invariant_in_screen_space) | test(edge_insert_drag_threshold_is_zoom_invariant_in_screen_space) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the primary connection pointer-move route, wire-drag move
    helpers, and edge-insert move helpers off retained bridge Cx names; real wire-drag hover,
    connection drag threshold, and edge-insert drag threshold behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/connection.rs ecosystem/fret-node/src/ui/canvas/widget/pending_wire_drag.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag_move_cx.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/mod.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/auto_pan.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending/activate.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
  - Result: no matches.
  - Scope proven: the migrated connection pointer-move route and called wire/edge-insert move
    helpers no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the pointer-move connection route
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the primary connection pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-780` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-790 pointer-move primary route wrapper retained Cx isolation

Claim verified:

- `pointer_move_dispatch/primary.rs` no longer imports or names retained bridge Cx types.
- The primary pointer-move wrapper now uses `PrimaryPointerMoveCx`, composed from the already
  retained-agnostic surface, group, node, and connection branch capabilities.
- `PrimaryPointerMoveCx` adds no new side-effect methods; retained `EventCx` satisfies it only
  through existing branch adapters.
- Representative primary pointer-move behavior remains green for marquee, group drag activation,
  node drag threshold, and connection drag threshold.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/primary_pointer_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the primary
    pointer-move wrapper behind `PrimaryPointerMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_primary_surface_route_stays_off_retained_bridge) | test(pointer_move_primary_group_route_stays_off_retained_bridge) | test(pointer_move_primary_node_route_stays_off_retained_bridge) | test(pointer_move_primary_connection_route_stays_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(group_header_click_selects_group_and_arms_pending_group_drag) | test(node_drag_threshold_is_zoom_invariant_in_screen_space) | test(connection_drag_threshold_is_zoom_invariant_in_screen_space) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 10 tests.
  - Scope proven: source-policy locks the primary pointer-move wrapper plus all four primary
    branches off retained bridge Cx names; representative marquee, group, node, and connection
    primary pointer-move behavior remains green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary.rs ecosystem/fret-node/src/ui/canvas/widget/primary_pointer_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated primary pointer-move wrapper and composed capability no longer
    depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the primary pointer-move wrapper
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the primary pointer-move wrapper.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-790` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-800 pointer-move secondary node retained Cx isolation

Claim verified:

- `pointer_move_dispatch/secondary/node.rs` no longer imports or names retained bridge Cx types.
- Node drag move handling now uses `NodeDragMoveCx`, composed from the existing node drag geometry,
  preview, and move-tail capabilities plus bounds access.
- Node resize move handling now uses `NodeResizeMoveCx` for host access and paint invalidation.
- Retained `EventCx` adaptation is isolated in `node_drag_move_retained_cx.rs` and
  `node_resize_move_retained_cx.rs`.
- Secondary connection/insert pointer-move routing remains retained-bound for later isolation
  slices.
- Node drag move and node resize behavior remain green through the retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_resize/move_update.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_resize_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/node_resize_move_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/node.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the secondary node
    pointer-move route behind `NodeDragMoveCx + NodeResizeMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_node_route_stays_off_retained_bridge) | test(node_drag_move_handlers_stay_off_retained_bridge) | test(node_resize_move_handlers_stay_off_retained_bridge) | test(node_drag_move_emits_on_node_drag) | test(node_drag_respects_per_node_extent_rect) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(node_resize_expands_group_when_expand_parent_is_true) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the secondary node pointer-move route, node drag move
    handlers, and node resize move handlers off retained bridge Cx names; real node drag move and
    node resize behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/node.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_cx.rs ecosystem/fret-node/src/ui/canvas/widget/node_resize/move_update.rs ecosystem/fret-node/src/ui/canvas/widget/node_resize_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated secondary node route and called node drag/resize move helpers no
    longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the secondary node pointer-move
    route seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the secondary node pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after the user's pull and this
    slice's updates.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-800` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-810 pointer-move secondary connection retained Cx isolation

Claim verified:

- `pointer_move_dispatch/secondary/connection.rs` no longer imports or names retained bridge Cx
  types.
- Wire-drag move handling continues to use the retained-agnostic `WireDragMoveCx`.
- Active edge-insert move handling continues to use the retained-agnostic
  `WidgetPaintInvalidationCx` seam.
- Edge reconnect drag movement now uses `EdgeDragMoveCx` for host access plus paint invalidation.
- Retained `EventCx` adaptation is isolated in `edge_drag_move_retained_cx.rs`.
- Secondary insert pointer-move routing remains retained-bound for a later isolation slice.
- Wire/edge-insert move source policy and edge reconnect threshold/cancel/radius behavior remain
  green through the retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/move_start.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/prelude.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/edge_drag_move_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the secondary
    connection pointer-move route behind `WireDragMoveCx + EdgeDragMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_connection_route_stays_off_retained_bridge) | test(edge_drag_move_handlers_stay_off_retained_bridge) | test(wire_drag_move_handlers_stay_off_retained_bridge) | test(edge_insert_move_handlers_stay_off_retained_bridge) | test(edge_reconnect_requires_drag_threshold_before_starting_wire_drag) | test(edge_reconnect_drag_cancels_when_endpoint_not_reconnectable) | test(edge_reconnect_radius_is_zoom_invariant_in_screen_space) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the secondary connection pointer-move route, edge-drag move
    handlers, wire-drag move handlers, and edge-insert move handlers off retained bridge Cx names;
    edge reconnect threshold, cancel, and radius behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/mod.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/move_start.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated secondary connection route and called edge-drag move helpers no
    longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the secondary connection
    pointer-move route seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the secondary connection pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-810` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-820 pointer-move secondary insert retained Cx isolation

Claim verified:

- `pointer_move_dispatch/secondary/insert.rs` no longer imports or names retained bridge Cx types.
- Pending insert-node drag movement now uses `InsertNodeDragMoveCx` for
  pointer/window/bounds/tick/host access plus pointer-capture release and paint invalidation.
- Retained `EventCx` adaptation is isolated in `insert_node_drag_move_retained_cx.rs`.
- `insert_node_drag/mod.rs` no longer owns the retained internal drag event body; that retained
  entry is isolated in `insert_node_drag/internal_event.rs`.
- Internal drag enter/over/drop I/O remains retained-bound for a later slice.
- Insert-node drag threshold, drag start, and searcher cleanup behavior remain green through the
  retained compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/prelude.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/session.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_event.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_drop.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag_move_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/insert.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the secondary
    insert pointer-move route behind `InsertNodeDragMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_insert_route_stays_off_retained_bridge) | test(insert_node_drag_move_handlers_stay_off_retained_bridge) | test(insert_node_drag_does_not_start_until_threshold) | test(insert_node_drag_starts_after_threshold) | test(insert_node_drag_start_clears_searcher_overlay_state) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 6 tests.
  - Scope proven: source-policy locks the secondary insert pointer-move route and pending
    insert-node drag move helpers off retained bridge Cx names; insert-node drag threshold, drag
    start, and searcher cleanup behavior remain green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/insert.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/mod.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/session.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated secondary insert route and pending insert-node drag move helpers no
    longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the secondary insert pointer-move
    route seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the secondary insert pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-820` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-830 pointer-move secondary route wrapper retained Cx isolation

Claim verified:

- `pointer_move_dispatch/secondary.rs` no longer imports or names retained bridge Cx types.
- The secondary pointer-move wrapper now uses `SecondaryPointerMoveCx`, composed from the already
  retained-agnostic node, connection, and insert branch capabilities.
- `SecondaryPointerMoveCx` adds no new side-effect methods; retained `EventCx` satisfies it only
  through existing branch adapters.
- Representative secondary pointer-move behavior remains green for node drag move, edge reconnect,
  and insert-node drag start.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/secondary_pointer_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the secondary
    pointer-move wrapper behind `SecondaryPointerMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_secondary_node_route_stays_off_retained_bridge) | test(pointer_move_secondary_connection_route_stays_off_retained_bridge) | test(pointer_move_secondary_insert_route_stays_off_retained_bridge) | test(node_drag_move_emits_on_node_drag) | test(edge_reconnect_requires_drag_threshold_before_starting_wire_drag) | test(insert_node_drag_starts_after_threshold) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the secondary pointer-move wrapper plus all three secondary
    branches off retained bridge Cx names; representative node, connection, and insert secondary
    pointer-move behavior remains green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary.rs ecosystem/fret-node/src/ui/canvas/widget/secondary_pointer_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated secondary pointer-move wrapper and composed capability no longer
    depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the secondary pointer-move wrapper
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the secondary pointer-move wrapper.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-830` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-840 pointer-move overlay route retained Cx isolation

Claim verified:

- `pointer_move_dispatch/overlay.rs` no longer imports or names retained bridge Cx types.
- Searcher and context-menu pointer-move facades now require only
  `WidgetPaintInvalidationCx`, matching the already-retained-agnostic hover-update leaf helpers.
- Overlay pointer-move dispatch no longer pulls searcher key/down/up route traits or context-menu
  key/down activation traits into its Cx bound.
- Representative searcher and context-menu hover move behavior remains green through the retained
  compatibility island.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/overlay.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the overlay
    pointer-move route behind `WidgetPaintInvalidationCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_overlay_route_stays_off_retained_bridge) | test(searcher_top_level_route_stays_off_retained_bridge) | test(context_menu_top_level_route_stays_off_retained_bridge) | test(context_menu_pointer_move_route_stays_off_retained_bridge) | test(searcher_pointer_move_updates_hover_and_invalidates_paint) | test(context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint) | test(pointer_move_updates_hover_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 8 tests.
  - Scope proven: source-policy locks the overlay pointer-move route and existing
    searcher/context-menu top-level routes off retained bridge Cx names; representative
    searcher/context-menu hover updates still invalidate paint.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/overlay.rs ecosystem/fret-node/src/ui/canvas/widget/searcher.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
  - Result: no matches.
  - Scope proven: the migrated overlay route and narrowed searcher/context-menu pointer-move
    facades no longer depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the overlay pointer-move route
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the overlay pointer-move route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-840` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-850 pointer-move hover fallback retained Cx isolation

Claim verified:

- `hover.rs` no longer imports or names retained bridge Cx types.
- Fallback edge/anchor hover pointer-move handling now uses `HoverMoveCx` for host access plus
  paint invalidation.
- Retained `EventCx` adaptation is isolated in `hover_move_retained_cx.rs`.
- Real retained compatibility behavior still updates `hover_edge`, requests paint invalidation, and
  does not repeat invalidation when the hover state is unchanged.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/hover.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/hover_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/hover_move_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_hover_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving fallback hover
    pointer-move handling behind `HoverMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_hover_fallback_stays_off_retained_bridge) | test(hover_fallback_updates_hover_edge_and_invalidates_paint_once) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 3 tests.
  - Scope proven: source-policy locks fallback hover off retained bridge Cx names; real retained
    compatibility behavior still updates edge hover and invalidates paint only on state change.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/hover.rs ecosystem/fret-node/src/ui/canvas/widget/hover_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated fallback hover path and pure capability seam no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the hover fallback seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the hover fallback.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-850` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-21 - RBX-M2-860 top-level pointer-move route wrapper retained Cx isolation

Claim verified:

- `pointer_move_dispatch.rs` no longer imports or names retained bridge Cx types.
- The top-level pointer-move route wrapper now uses `PointerMoveCx`, composed from the already
  retained-agnostic primary, secondary, and hover fallback capabilities.
- `PointerMoveCx` adds no new side-effect methods; retained `EventCx` satisfies it only through
  existing branch adapters.
- Representative primary, secondary, overlay, and hover pointer-move behavior remains green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the top-level
    pointer-move route wrapper behind `PointerMoveCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_route_wrapper_stays_off_retained_bridge) | test(pointer_move_primary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_secondary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_overlay_route_stays_off_retained_bridge) | test(pointer_move_hover_fallback_stays_off_retained_bridge) | test(node_drag_move_emits_on_node_drag) | test(edge_reconnect_requires_drag_threshold_before_starting_wire_drag) | test(searcher_pointer_move_updates_hover_and_invalidates_paint) | test(context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint) | test(hover_fallback_updates_hover_edge_and_invalidates_paint_once) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 11 tests.
  - Scope proven: source-policy locks the top-level pointer-move wrapper and branch wrappers off
    retained bridge Cx names; representative primary, secondary, overlay, and hover behavior stays
    green.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated top-level pointer-move wrapper and composed capability no longer
    depend on retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the top-level pointer-move wrapper
    seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the top-level pointer-move route wrapper.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-860` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M2-910 pointer-move release route retained Cx isolation

Claim verified:

- `event_pointer_move.rs`, `event_pointer_move/release.rs`, `event_pointer_move/tail.rs`,
  `pointer_move_release.rs`, `pointer_move_release_pan.rs`,
  `pointer_move_release_pan/missing_release.rs`, and
  `pointer_move_release_pan/pending_right_click.rs` no longer import or name retained bridge Cx
  types.
- Pointer-move release routing now uses a composed `PointerMoveReleaseCx` over existing
  `PointerUpCx` and `PanZoomBeginCx` capabilities without adding a new retained adapter.
- Real retained compatibility behavior still infers missed pan pointer-up from mouse button state
  and still starts/finishes right-button pan behavior correctly.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/missing_release.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default declarative `fret-node` surface still compiles without enabling the
    retained canvas compatibility island.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the pointer-move
    release route behind `PointerMoveReleaseCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_release_route_stays_off_retained_bridge) | test(pointer_move_missing_left_release_stays_off_retained_bridge) | test(missing_pan_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(right_pan_drag_does_not_open_context_menu) | test(right_pan_defers_context_menu_until_pointer_up) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 9 tests.
  - Scope proven: source-policy locks the release route off retained bridge Cx names; missed pan
    release inference, missed left pointer-up inference, and right-button pan behavior remain green
    through the retained compatibility path.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/release.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/tail.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/missing_release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs`
  - Result: no matches.
  - Scope proven: the migrated pointer-move release route no longer depends on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the release-route seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the pointer-move release route.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-910` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M2-900 pointer-move missing-left-release retained Cx isolation

Claim verified:

- `pointer_move_release_left.rs` no longer imports or names retained bridge Cx types.
- Missing-left-release inference now reuses the retained-agnostic `PointerUpCx` /
  `PointerUpReleaseCx` seams and does not need a new retained adapter.
- Real retained compatibility behavior still infers a missed left pointer-up from empty mouse
  button state for node drag, reconnect wire drag, and new wire drag paths.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_left.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default declarative `fret-node` surface still compiles without enabling the
    retained canvas compatibility island.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving missing-left-
    release inference behind `PointerUpCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_missing_left_release_stays_off_retained_bridge) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state_for_wire_reconnect_drag) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state_for_new_wire_drag) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 5 tests.
  - Scope proven: source-policy locks the missing-left-release helper off retained bridge Cx names;
    retained compatibility behavior still handles missed pointer-up inference for node and wire
    drags.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_left.rs`
  - Result: no matches.
  - Scope proven: the migrated missing-left-release helper no longer depends on retained bridge Cx
    names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the missing-left-release seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    missing-left-release inference.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-900` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M2-890 pointer-move tail wrapper retained Cx isolation

Claim verified:

- `event_pointer_move_tail.rs` and `pointer_move_tail_cx.rs` no longer import or name retained
  bridge Cx types.
- Pointer-move tail dispatch now uses `PointerMoveTailCx`, composed from the already-isolated
  cursor, pointer-move route, and auto-pan timer capabilities.
- Representative retained compatibility behavior for cursor updates and auto-pan timer scheduling
  remains green.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_tail_cx.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default declarative `fret-node` surface still compiles without enabling the
    retained canvas compatibility island.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving the pointer-move
    tail wrapper behind `PointerMoveTailCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_tail_wrapper_stays_off_retained_bridge) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(pointer_move_cursor_update_sets_close_button_cursor) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 4 tests.
  - Scope proven: source-policy locks the tail wrapper off retained bridge Cx names; representative
    cursor and auto-pan timer behavior remain green through the retained compatibility path.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_tail_cx.rs`
  - Result: no matches.
  - Scope proven: the migrated tail wrapper and composed capability seam no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the pointer-move tail wrapper seam
    changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the pointer-move tail wrapper.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-890` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M2-880 pointer-move auto-pan timer retained Cx isolation

Claim verified:

- `auto_pan_timer_cx.rs` and `event_pointer_move_tail/timer.rs` no longer import or name retained
  bridge Cx types.
- Pointer-move auto-pan timer sync now uses `AutoPanTimerCx` for host, window, and bounds access.
- Retained `EventCx` adaptation is isolated in `auto_pan_timer_retained_cx.rs`.
- Real retained compatibility behavior still starts a repeating auto-pan timer when a node drag
  reaches the viewport edge during a pointer move.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/auto_pan_timer_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/auto_pan_timer_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/timer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_timer_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default declarative `fret-node` surface still compiles without enabling the
    retained canvas compatibility island.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-move
    auto-pan timer sync behind `AutoPanTimerCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_auto_pan_timer_stays_off_retained_bridge) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 3 tests.
  - Scope proven: source-policy locks auto-pan timer sync off retained bridge Cx names; real
    retained compatibility behavior still starts the timer during a node drag near the viewport
    edge.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/auto_pan_timer_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/timer.rs`
  - Result: no matches.
  - Scope proven: the migrated timer sync path and pure capability seam no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the auto-pan timer seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after
    isolating the pointer-move auto-pan timer path.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-880` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M2-870 pointer-move cursor update retained Cx isolation

Claim verified:

- `cursor.rs` and `event_pointer_move_tail/cursor.rs` no longer import or name retained bridge Cx
  types.
- Pointer-move cursor updates now use `CanvasCursorCx` for host access plus cursor icon side
  effects.
- Retained `EventCx` adaptation is isolated in `cursor_retained_cx.rs`.
- Real retained compatibility behavior still requests the pointer cursor when the pointer moves over
  the node graph close button.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cursor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cursor_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cursor_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/cursor.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_cursor_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default declarative `fret-node` surface still compiles without enabling the
    retained canvas compatibility island.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after moving pointer-move
    cursor updates behind `CanvasCursorCx`.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_cursor_update_stays_off_retained_bridge) | test(pointer_move_cursor_update_sets_close_button_cursor) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 3 tests.
  - Scope proven: source-policy locks cursor update helpers off retained bridge Cx names; real
    retained compatibility behavior still requests the close-button pointer cursor.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/cursor.rs ecosystem/fret-node/src/ui/canvas/widget/cursor_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/cursor.rs`
  - Result: no matches.
  - Scope proven: the migrated cursor update path and pure capability seam no longer depend on
    retained bridge Cx names.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the cursor update seam changes.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after isolating
    the pointer-move cursor update path.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-870` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M2-920 cancel/escape lifecycle retained Cx isolation

Claim verified:

- `cancel.rs` and `cancel_viewport_state.rs` no longer import or name retained bridge Cx types.
- Cancel/escape lifecycle cleanup now uses `CancelGestureCx` plus a host-based viewport-state seam.
- Retained `EventCx` adaptation is isolated in `cancel_retained_cx.rs`.
- Existing cancel/escape behavior still releases pointer capture, cancels viewport motion, and
  emits the expected canceled end callbacks.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/cancel_viewport_state.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/callbacks_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/escape_cancel_releases_pointer_capture_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: pending.
  - Scope planned: confirm the default declarative `fret-node` surface still compiles after the
    cancel/escape lifecycle refactor.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: pending.
  - Scope planned: confirm the retained canvas compatibility island still compiles with the new
    cancel retained adapter.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(cancel_lifecycle_helpers_stay_off_retained_bridge) | test(escape_cancel_emits_connect_end_canceled) | test(escape_cancel_panning_emits_move_end_canceled) | test(node_drag_start_and_escape_cancel_emits_node_drag_end_canceled) | test(escape_cancel_releases_pointer_capture_during_panning) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: pending.
  - Scope planned: prove the cancel/escape lifecycle still behaves correctly on the retained
    compatibility path.
- `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/cancel.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_viewport_state.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cx.rs`
  - Result: pending.
  - Scope planned: prove the pure cancel/viewport-state helpers no longer depend on retained bridge
    Cx names.
- `cargo fmt --check`
  - Result: pending.
- `python3 tools/check_layering.py`
  - Result: pending.
- `python3 tools/check_workstream_catalog.py`
  - Result: pending.
- `git diff --check`
  - Result: pending.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: pending.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-920` is a narrow adapter-boundary slice in `fret-node`'s retained canvas
    widget. The compat compile gate, targeted compat nextest gate, source-policy scan, formatting,
    layering, catalog, whitespace, and merge-marker checks should cover the changed surface.

## 2026-05-22 - RBX-M2-930 pointer-wheel and timer-motion retained Cx isolation

Claim verified:

- Pointer-wheel and timer-motion route helpers now use retained-agnostic seams instead of naming
  retained bridge Cx types directly.
- `EventCx` adaptation for the wheel platform seam is isolated in `pointer_wheel_retained_cx.rs`.
- Existing wheel zoom/pan, viewport animation, auto-pan, and timer-motion behavior still passes on
  the retained compatibility path.

Evidence:

- `ecosystem/fret-node/src/lib.rs`
- `ecosystem/fret-node/src/ui/canvas/widget.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/event_timer_route.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_motion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_retained_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/apply.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/pinch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/wheel.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_auto_pan.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_auto_pan/dispatch.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_pan_inertia.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport/animation.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport/debounce.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/viewport_motion_cx.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/callbacks_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_timer_conformance.rs`
- `ecosystem/fret-node/src/ui/canvas/widget/tests/viewport_animation_conformance.rs`
- `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-node`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the default declarative `fret-node` surface still compiles after the wheel/timer
    motion seam extraction.
- `cargo check -p fret-node --features compat-retained-canvas`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the retained canvas compatibility island compiles after the wheel/timer motion
    seam extraction.
- `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_wheel_route_stays_off_retained_bridge) | test(timer_motion_route_stays_off_retained_bridge) | test(wheel_zoom_emits_move_start_and_debounced_move_end) | test(pinch_zoom_emits_move_start_and_debounced_move_end) | test(wheel_pan_emits_move_start_and_debounced_move_end) | test(wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom) | test(frame_view_animates_over_timer_ticks_and_reaches_target) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(pinch_gesture_zooms_in_about_pointer) | test(wheel_zoom_zooms_about_pointer) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
  - Result: passed, 11 tests.
  - Scope proven: source-policy coverage keeps wheel/timer helper files off retained bridge Cx
    names, and the wheel/timer behavior matrix remains green on the retained compatibility path.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: workspace Rust formatting remains clean after the wheel/timer motion seam
    extraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge feature allowlist remain valid after the
    wheel/timer motion seam extraction.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M2-930` is another narrow adapter-boundary slice in `fret-node`'s retained
    canvas widget. The compat compile gate, targeted compat nextest gate, source-policy scan,
    formatting, layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M3-010 Plot3D declarative viewport panel

Claim verified:

- `fret-plot3d` no longer depends on `fret-ui/unstable-retained-bridge`.
- The retained `Plot3dCanvas` widget surface was deleted and replaced by the public declarative
  `plot3d_panel(...)` / `Plot3dPanelProps` surface.
- First-party Plot3D demos now mount the Plot3D UI through `declarative::RenderRootContext` instead
  of `UiTreeRetainedExt::create_node_retained(...)`.
- Follow-up behavior coverage now proves the declarative viewport helper and Plot3D panel paint a
  non-zero `SceneOp::ViewportSurface` and keep Plot3D panel chrome, preventing a retained-to-
  declarative migration from silently compiling while rendering a zero-sized viewport.

Evidence:

- `ecosystem/fret-plot3d/src/declarative.rs`
- `ecosystem/fret-plot3d/src/lib.rs`
- `ecosystem/fret-plot3d/Cargo.toml`
- `ecosystem/fret-ui-kit/src/declarative/viewport_surface.rs`
- `apps/fret-examples/src/plot3d_demo.rs`
- `apps/fret-examples/src/gizmo3d_demo.rs`
- `tools/check_layering.py`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- `docs/audits/implot3d-alignment.md`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-plot3d`
  - Result: passed with the pre-existing `fret-ui` warning for
    `current_effective_opacity` dead code.
  - Scope proven: the new declarative Plot3D package surface compiles without enabling the retained
    bridge.
- `cargo check -p fret-demo --bin plot3d_demo`
  - Result: passed.
  - Scope proven: the standalone Plot3D demo compiles after migrating from retained
    `Plot3dCanvas` creation to declarative root mounting.
- `cargo check -p fret-demo --bin gizmo3d_demo`
  - Result: passed.
  - Scope proven: the gizmo/Plot3D demo compiles after migrating from retained `Plot3dCanvas`
    creation to declarative root mounting.

- `cargo nextest run -p fret-plot3d`
  - Result: passed, 2 tests.
  - Scope proven: source-policy coverage locks `fret-plot3d` to its declarative public surface,
    prevents reintroducing `Plot3dCanvas` or the retained bridge feature, and behavior coverage
    proves `plot3d_panel_with_model(...)` emits a non-zero `SceneOp::ViewportSurface` while
    preserving panel background chrome.
- `cargo nextest run -p fret-ui-kit viewport_surface_panel_fills_pointer_region_and_paints_surface`
  - Result: passed, 1 test.
  - Scope proven: the shared declarative `viewport_surface_panel(...)` helper sizes its inner
    `ViewportSurface` to the pointer region bounds and emits a non-zero viewport surface scene op.
- `cargo check -p fret-demo --bin plot3d_demo`
  - Result: passed.
  - Scope proven: the standalone Plot3D demo still compiles after adding behavior coverage for the
    declarative viewport-surface path.
- `cargo check -p fret-demo --bin gizmo3d_demo`
  - Result: passed.
  - Scope proven: the gizmo/Plot3D demo still compiles after adding behavior coverage for the
    declarative viewport-surface path.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: workspace Rust formatting is clean after the Plot3D declarative migration.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering remains valid and `fret-plot3d` has been removed from the
    retained bridge allowlist.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after documentation updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "fret_plot3d::retained|Plot3dCanvas::|use fret_plot3d::\\{[^\\n]*Plot3dCanvas|use fret_plot3d::retained" ecosystem/fret-plot3d apps/fret-examples/src/plot3d_demo.rs apps/fret-examples/src/gizmo3d_demo.rs -g '*.rs' -g 'Cargo.toml'`
  - Result: no matches.
  - Scope proven: no Plot3D source/demo call site still imports or constructs the deleted retained
    `Plot3dCanvas` surface.
- `cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name=="fret-plot3d") | .dependencies[]? | select(.name=="fret-ui") | (.features|join(","))'`
  - Result: empty feature list.
  - Scope proven: `fret-plot3d` depends on `fret-ui` without `unstable-retained-bridge`.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-010` is a targeted Plot3D bridge-exit slice. The package compile/test gate,
    first-party demo compile gates, retained-bridge no-user proof, formatting, layering, catalog,
    whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M3-020 Chart declarative canvas capability baseline

Claim verified:

- `fret-chart` now has a behavior baseline proving `chart_canvas_panel(...)` can render a seeded,
  controlled `Model<ChartEngine>` through a real declarative UI frame without constructing
  `retained::ChartCanvas`.
- The test proves more than compilation: the declarative chart subtree has non-zero layout, the
  engine records viewport bounds, delinea produces rect marks for the seeded bar chart, and the
  declarative canvas emits non-background, non-zero chart mark quads.
- This is deliberately a pre-delete baseline. `retained::ChartCanvas`, `ChartCanvasOutput`,
  multi-grid helpers, and retained demo/gallery consumers remain because deleting them before
  consumer migration would remove chart capabilities.

Evidence:

- `ecosystem/fret-chart/src/declarative/panel.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart chart_canvas_panel_paints_seeded_chart_marks_on_declarative_path`
  - Result: passed, 1 test.
  - Scope proven: the new declarative chart capability test passes in the default `fret-chart`
    feature set.
- `rg -n "fret_chart::retained|ChartCanvas::|retained::ChartCanvas|chart_canvas_panel\\(" apps ecosystem crates -g '*.rs'`
  - Result: found retained `ChartCanvas` consumers in first-party chart demos, cookbook/gallery
    chart paths, `retained::multi_grid`, and `linking` output types; found existing declarative
    `chart_canvas_panel(...)` usage in `apps/fret-examples/src/echarts_demo.rs`.
  - Scope proven: retained chart removal is not safe in this slice; the correct next work is
    consumer migration plus output/multi-grid replacement, using the new declarative baseline as a
    regression gate.
- `cargo nextest run -p fret-chart`
  - Result: passed, 40 tests.
  - Scope proven: the declarative capability baseline and existing retained chart oracle tests
    remain green together.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after adding the declarative chart baseline test.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the allowlist for the retained public/demo/gallery migration follow-up.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task and evidence updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-020` adds a targeted `fret-chart` declarative capability baseline without
    changing shared runtime contracts. The package test gate, formatting, layering, catalog,
    whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M3-030 Chart declarative line/scatter capability baseline

Claim verified:

- `chart_canvas_panel(...)` now has default-feature behavior coverage for line and scatter chart
  families, not only bar charts.
- The test proves the controlled `ChartEngine` produces `Polyline` marks for the line series and
  `Points` marks for the scatter series, then proves the declarative canvas paints those as a
  `SceneOp::Path` plus non-background, non-zero point quads.
- This is still a pre-consumer-migration baseline. Retained axes, visual-map, data-zoom, output, and
  multi-grid surfaces remain known migration work before retained chart deletion.

Evidence:

- `ecosystem/fret-chart/src/declarative/panel.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart chart_canvas_panel_paints_line_and_scatter_marks_on_declarative_path`
  - Result: passed, 1 test.
  - Scope proven: the new declarative line/scatter capability test passes in the default
    `fret-chart` feature set.
- `cargo nextest run -p fret-chart`
  - Result: passed, 41 tests.
  - Scope proven: the bar and line/scatter declarative baselines remain green alongside the existing
    retained chart oracle tests.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after adding the line/scatter declarative baseline.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task and evidence updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-030` is a targeted `fret-chart` behavior-baseline slice. The package test gate,
    formatting, layering, catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M3-040 Chart declarative output model publication

Claim verified:

- `ChartCanvasOutput` is now a top-level `fret-chart` output contract with a retained compatibility
  re-export, rather than a type owned only by `retained`.
- `chart_canvas_panel(...)` can publish a caller-provided `Model<ChartCanvasOutput>` without
  constructing `retained::ChartCanvas`.
- The new declarative output test proves output revision advancement, `LinkAxisKey` domain-window
  publication, brush publication, link-event preservation, and link-events revision advancement.
- Retained chart output behavior still passes because retained `ChartCanvas::publish_output(...)`
  now uses the same shared snapshot/update helper.

Evidence:

- `ecosystem/fret-chart/src/output.rs`
- `ecosystem/fret-chart/src/declarative/panel.rs`
- `ecosystem/fret-chart/src/retained/output.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-chart`
  - Result: passed.
  - Scope proven: `fret-chart` type-checks after moving `ChartCanvasOutput` to the shared output
    module and wiring declarative publication.
- `cargo nextest run -p fret-chart chart_canvas_panel_publishes_output_model_on_declarative_path`
  - Result: passed, 1 test.
  - Scope proven: the new declarative output-model publication test passes in the default
    `fret-chart` feature set.
- `cargo nextest run -p fret-chart`
  - Result: passed, 42 tests.
  - Scope proven: the new declarative output baseline, previous declarative paint baselines, and
    retained output/linking/tooltip oracle tests remain green together.

- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the shared output helper and declarative output
    test.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist while remaining chart retained
    consumers are migrated.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-040` is a targeted `fret-chart` output publication slice. The package check,
    targeted declarative output test, full `fret-chart` package test gate, formatting, layering,
    catalog, whitespace, and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M3-045 Gallery chart snippets use declarative panel

Claim verified:

- UI Gallery's copyable shadcn-style chart `usage.rs` and `demo.rs` snippets no longer construct
  retained `ChartCanvas` widgets through `RetainedSubtreeProps` / `cx.retained_subtree(...)`.
- Those snippets now seed controlled `Model<ChartEngine>` instances and render chart bodies through
  `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)`.
- The "First Chart" snippet still shares a `ChartCanvasOutput` model with `ChartContainer`, so
  tooltip/legend output wiring remains on the shared declarative output contract introduced in
  `RBX-M3-040`.
- The chart page and grid/axis follow-up text no longer describes ordinary chart body authoring as
  retained. The accessibility snippet remains an explicit known gap because declarative keyboard
  point navigation has not been migrated yet.

Evidence:

- `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
- `apps/fret-ui-gallery/src/ui/snippets/chart/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/chart/grid_axis.rs`
- `apps/fret-ui-gallery/src/ui/pages/chart.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-ui-gallery --features gallery-chart chart_snippets_prefer_declarative_canvas_panel`
  - Result: passed, 1 test.
  - Scope proven: the new source-policy gate requires `usage.rs` and `demo.rs` to teach
    `ChartEngine` + `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)`, and rejects retained
    chart authoring markers.
- `cargo check -p fret-ui-gallery --features gallery-chart`
  - Result: passed.
  - Scope proven: the Gallery chart snippets compile with the optional `gallery-chart` feature
    after migrating from cached retained subtrees to declarative chart panels.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the Gallery chart snippet migration.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid after the
    Gallery chart consumer migration.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.
- `rg -n "RetainedSubtreeProps|UiTreeRetainedExt|cx\\.retained_subtree|ChartCanvas::new\\(|use fret_chart::ChartCanvas;|use fret_chart::\\{ChartCanvas," apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs apps/fret-ui-gallery/src/ui/snippets/chart/demo.rs`
  - Result: no matches.
  - Scope proven: the migrated Gallery chart usage/demo snippets do not contain the retained chart
    authoring markers guarded by the policy test.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-045` is a targeted UI Gallery chart docs/consumer migration slice. The planned
    package feature check, targeted source-policy test, formatting, layering, catalog, whitespace,
    and merge-marker checks cover the changed surface.

## 2026-05-22 - RBX-M3-050 Chart declarative accessibility navigation

Claim verified:

- `chart_canvas_panel(...)` now supports a declarative chart accessibility layer without
  constructing retained `ChartCanvas`.
- The retained chart accessibility index has been extracted into a crate-private shared helper so
  retained and declarative paths use the same mark/data-index mapping.
- Declarative chart panels can expose a focusable viewport semantics node with collection position,
  tooltip value, and arrow-key point navigation.
- UI Gallery's first-chart example now opts into declarative chart accessibility, and its
  accessibility snippet no longer teaches retained `ChartCanvas` helper authoring.

Evidence:

- `ecosystem/fret-chart/src/a11y.rs`
- `ecosystem/fret-chart/src/declarative/panel.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
- `apps/fret-ui-gallery/src/ui/snippets/chart/accessibility.rs`
- `apps/fret-ui-gallery/src/ui/pages/chart.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-chart`
  - Result: passed.
  - Scope proven: `fret-chart` type-checks after adding declarative accessibility props and moving
    `ChartA11yIndex` into the shared crate-private module.
- `cargo nextest run -p fret-chart chart_canvas_panel_keyboard_navigation_publishes_tooltip_lines_on_declarative_path`
  - Result: passed, 1 test.
  - Scope proven: the new declarative accessibility oracle advances semantics `pos_in_set` from 1
    to 2 on `ArrowRight` and publishes non-empty tooltip lines to `ChartCanvasOutput`.
- `cargo nextest run -p fret-chart`
  - Result: passed, 43 tests.
  - Scope proven: declarative chart accessibility, existing declarative output/paint baselines, and
    retained chart accessibility/output/tooltip oracle tests remain green together.
- `cargo nextest run -p fret-ui-gallery --features gallery-chart chart_snippets_prefer_declarative_canvas_panel`
  - Result: passed, 1 test.
  - Scope proven: Gallery chart snippets, including accessibility docs, teach declarative
    `ChartCanvasPanelProps` and reject retained chart authoring markers.
- `cargo nextest run -p fret-ui-gallery --features gallery-chart chart_first_chart_keyboard_navigation_shows_auto_wired_tooltip_under_default_cache_policy`
  - Result: passed, 1 test.
  - Scope proven: the real Gallery first-chart path exposes focusable chart semantics, handles
    `ArrowRight`, advances the accessibility index, publishes an accessibility value, and shows the
    auto-wired tooltip.
- `cargo nextest run -p fret-ui-gallery --features gallery-chart chart_first_chart_keyboard_navigation_shows_auto_wired_tooltip_under_default_cache_policy chart_snippets_prefer_declarative_canvas_panel`
  - Result: passed, 2 tests.
  - Scope proven: the final post-extraction Gallery behavior and source-policy gates pass together.
- `cargo check -p fret-ui-gallery --features gallery-chart`
  - Result: passed.
  - Scope proven: Gallery chart docs compile with declarative accessibility props after the
    retained helper migration.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after chart accessibility and Gallery docs updates.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.
- `rg -n "RetainedSubtreeProps|UiTreeRetainedExt|cx\\.retained_subtree|ChartCanvas::new\\(|use fret_chart::ChartCanvas;|use fret_chart::\\{ChartCanvas,|fret_chart::ChartCanvas" apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs apps/fret-ui-gallery/src/ui/snippets/chart/demo.rs apps/fret-ui-gallery/src/ui/snippets/chart/accessibility.rs apps/fret-ui-gallery/src/ui/pages/chart.rs`
  - Result: only `ChartCanvasOutput` / `ChartCanvasPanelProps` matches; no retained `ChartCanvas`
    widget-authoring markers.
  - Scope proven: Gallery chart docs no longer teach retained chart authoring for usage/demo or
    accessibility helper code.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-050` is a targeted `fret-chart` accessibility parity and UI Gallery chart-docs
    migration slice. The package gate, Gallery feature check, focused Gallery integration/source
    gates, formatting, layering, catalog, whitespace, and merge-marker checks cover the changed
    surface.

## 2026-05-22 - RBX-M3-060 Gallery chart torture uses declarative panel

Claim verified:

- UI Gallery's chart torture page no longer constructs retained `ChartCanvas` widgets through
  `RetainedSubtreeProps` / `cx.retained_subtree(...)`.
- The torture page now keeps the stress `ChartEngine` in a `Model<ChartEngine>` and renders through
  `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)`.
- The diagnostics handle still exposes the shared chart engine and `ChartCanvasOutput` model for
  data-zoom, axis-output, domain-window, and tooltip snapshot collection.
- The explicit Y link-map fixture remains wired on the declarative chart panel path.

Evidence:

- `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`
- `apps/fret-ui-gallery/src/harness.rs`
- `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
- `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-ui-gallery --features gallery-dev`
  - Result: passed.
  - Scope proven: the Gallery dev surface compiles after chart torture moved from an
    `Rc<RefCell<ChartEngine>>` retained-widget handle to a shared `Model<ChartEngine>` declarative
    panel handle.
- `cargo nextest run -p fret-ui-gallery --features gallery-dev chart_torture_preview_uses_declarative_chart_panel`
  - Result: failed once because the new test used a helper that is not imported in
    `ui_authoring_surface_internal_previews.rs`; fixed to use the file's existing
    `manifest_path(...)` + `read_path(...)` pattern.
- `cargo nextest run -p fret-ui-gallery --features gallery-dev chart_torture_preview_uses_declarative_chart_panel`
  - Result: passed, 1 test.
  - Scope proven: the internal chart torture preview source-policy test requires
    `ChartEngine::new`, `ChartCanvasPanelProps::new`, `chart_canvas_panel_in(...)`, and the
    model-backed diagnostics handle, while rejecting retained chart torture authoring markers.
- `cargo nextest run -p fret-chart`
  - Result: passed, 43 tests, with the pre-existing `fret-ui`
    `current_effective_opacity` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility baselines and retained chart
    output/linking/tooltip/accessibility oracle tests remain green after migrating the Gallery
    torture consumer.
- `cargo nextest run -p fret-ui-gallery --features gallery-chart -E 'test(chart_snippets_prefer_declarative_canvas_panel) | test(chart_first_chart_keyboard_navigation_shows_auto_wired_tooltip_under_default_cache_policy)'`
  - Result: passed, 2 tests.
  - Scope proven: Gallery chart source-policy and first-chart accessibility/tooltip integration
    remain green after the chart torture migration.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the chart torture migration and validation
    helper fix.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    still intentionally remains on the allowlist while deeper retained chart capabilities are
    migrated.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" .`
  - Result: no matches.
  - Scope proven: the worktree has no textual merge-conflict markers after this slice.
- `rg -n "ChartCanvas::new_shared|RetainedSubtreeProps|UiTreeRetainedExt|cx\\.retained_subtree|retained_bridge|shared_engine" apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs apps/fret-ui-gallery/src/harness.rs apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
  - Result: no matches.
  - Scope proven: the migrated chart torture source and diagnostics handle no longer depend on
    retained chart widget authoring or the old `shared_engine` retained handle.

Notes:

- `apps/fret-ui-gallery/src/ui/snippets/ai/prompt_input_cursor_demo.rs` was minimally adjusted
  during this slice because the broad `gallery-dev` compile gate exposed an unrelated stale
  authoring pattern: it passed `ColorRef` to `AnyElement::inherit_foreground(...)` and applied
  `.layout(...)` directly to raw `AnyElement` text. The fix resolves token color to
  `theme.color_token("muted-foreground")` and wraps the affected text in layout-capable flex
  containers so the `gallery-dev` gate can prove the chart torture path.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-060` is a targeted UI Gallery chart torture consumer migration. The Gallery dev
    check, internal source-policy test, full `fret-chart` package gate, focused Gallery chart
    integration/source gates, formatting, layering, catalog, whitespace, and merge-marker checks
    cover the changed surface.

## 2026-05-22 - RBX-M3-070 Cookbook chart interactions use declarative panel

Claim verified:

- The cookbook chart interactions example no longer constructs retained `ChartCanvas` widgets
  through `RetainedSubtreeProps` / `cx.retained_subtree(...)`.
- The example now keeps its seeded `ChartEngine` in a `Model<ChartEngine>` and renders through
  `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)`.
- App-owned zoom/reset/selection, the stable chart test id, default chart input map, accessibility
  layer, and live hover selection still share the same chart engine that the panel renders.

Evidence:

- `apps/fret-cookbook/examples/chart_interactions_basics.rs`
- `apps/fret-cookbook/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-cookbook --example chart_interactions_basics --features cookbook-chart`
  - Result: passed.
  - Scope proven: the cookbook chart interactions example compiles after moving from the retained
    widget bridge to declarative `ChartCanvasPanelProps` with a shared engine/output model.
- `cargo nextest run -p fret-cookbook --features cookbook-chart chart_interactions_example_prefers_declarative_chart_panel`
  - Result: passed, 1 test.
  - Scope proven: the cookbook source-policy gate requires `ChartCanvasPanelProps`,
    `chart_canvas_panel_in(...)`, model-backed engine/output state, and rejects retained chart
    authoring markers.
- `cargo nextest run -p fret-chart`
  - Result: passed, 43 tests, with the pre-existing `fret-ui`
    `current_effective_opacity` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility baselines and retained chart
    output/linking/tooltip/accessibility oracle tests remain green after migrating the cookbook
    consumer.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the cookbook chart migration.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist while deeper retained chart surfaces are
    migrated.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "ChartCanvas::new_shared|RetainedSubtreeProps|UiTreeRetainedExt|cx\\.retained_subtree|retained_bridge|use fret_chart::ChartCanvas;|fret_chart::\\{ChartCanvas," apps/fret-cookbook/examples/chart_interactions_basics.rs`
  - Result: no matches.
  - Scope proven: the cookbook chart interactions example no longer contains retained chart widget
    authoring markers.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-070` is a targeted cookbook chart consumer migration. The cookbook example
    check, cookbook source-policy test, full `fret-chart` package gate, formatting, layering,
    catalog, whitespace, and retained-marker scan cover the changed surface.

## 2026-05-22 - RBX-M3-080 Basic chart demos use declarative panel

Claim verified:

- `chart_demo`, `category_line_demo`, and `horizontal_bars_demo` no longer construct retained
  `ChartCanvas` widgets through `ChartCanvas::new(...)` / `ChartCanvas::create_node(...)`.
- The demos now seed `ChartEngine` directly, store it as a `Model<ChartEngine>`, and render through
  `fret_ui::declarative::render_root(...)` plus `ChartCanvasPanelProps` +
  `chart_canvas_panel(...)`.
- The existing chart specs, datasets, and initial category-line data window are preserved on the
  same engine that the declarative chart panel observes and paints.

Evidence:

- `apps/fret-examples/src/chart_demo.rs`
- `apps/fret-examples/src/category_line_demo.rs`
- `apps/fret-examples/src/horizontal_bars_demo.rs`
- `apps/fret-examples/tests/basic_chart_demos_surface.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-demo --bin chart_demo --bin category_line_demo --bin horizontal_bars_demo`
  - Result: passed.
  - Scope proven: the three native demo bins compile after moving from retained `ChartCanvas`
    node creation to model-backed declarative chart panel roots.
- `cargo nextest run -p fret-examples basic_chart_demos_use_declarative_canvas_panel`
  - Result: passed, 1 test.
  - Scope proven: the new first-party examples source-policy gate requires
    `ChartCanvasPanelProps`, `chart_canvas_panel(...)`, `Model<ChartEngine>`, and
    `fret_ui::declarative::render_root(...)` while rejecting retained chart widget authoring
    markers.
- `cargo nextest run -p fret-chart`
  - Result: passed, 43 tests, with the pre-existing `fret-ui`
    `current_effective_opacity` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility baselines and retained chart
    output/linking/tooltip/accessibility oracle tests remain green after migrating these demo
    consumers.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the demo migration and new source-policy test.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist while remaining retained chart surfaces
    are migrated.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" apps/fret-examples/src/chart_demo.rs apps/fret-examples/src/category_line_demo.rs apps/fret-examples/src/horizontal_bars_demo.rs apps/fret-examples/tests/basic_chart_demos_surface.rs docs/workstreams/retained-bridge-exit-v1`
  - Result: no matches.
  - Scope proven: the changed demo, test, and workstream files have no textual merge-conflict
    markers after the pull/rebase state check.
- `rg -n "use fret_chart::retained::ChartCanvas|ChartCanvas::new\\(|ChartCanvas::new_shared|ChartCanvas::create_node|create_node_retained" apps/fret-examples/src/chart_demo.rs apps/fret-examples/src/category_line_demo.rs apps/fret-examples/src/horizontal_bars_demo.rs`
  - Result: no matches.
  - Scope proven: the migrated demo sources no longer contain retained chart widget authoring
    markers.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-080` is a targeted first-party chart demo consumer migration. The three-demo
    compile gate, source-policy test, full `fret-chart` package gate, formatting, layering,
    catalog, whitespace, merge-marker, and retained-marker scans cover the changed surface.

## 2026-05-22 - RBX-M3-090 Chart stress demo uses declarative panel

Claim verified:

- `chart_stress_demo` no longer constructs retained `ChartCanvas` widgets or retains a
  `ChartStressCanvas` wrapper.
- The stress demo now seeds `(ChartEngine, ChartSpec)`, stores the engine as a
  `Model<ChartEngine>`, and renders through `fret_ui::declarative::render_root(...)` plus
  `ChartCanvasPanelProps` + `chart_canvas_panel(...)`.
- LOD/progressive data seeding, continuous redraw, and periodic delinea stage/emitted stats
  reporting are preserved on the same engine model that the declarative chart panel renders.

Evidence:

- `apps/fret-examples/src/chart_stress_demo.rs`
- `apps/fret-examples/tests/basic_chart_demos_surface.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-examples chart_stress_demo_uses_declarative_canvas_panel`
  - Result: failed first as expected when the new source-policy test proved the stress demo still
    used retained `ChartCanvas`; passed after the migration.
  - Scope proven: the test acted as the red/green gate for replacing retained stress chart
    authoring with the declarative chart panel.
- `cargo check -p fret-demo --bin chart_stress_demo`
  - Result: passed.
  - Scope proven: the native stress demo compiles after removing the retained wrapper and moving
    chart state into a `Model<ChartEngine>`.
- `cargo nextest run -p fret-examples -E 'test(basic_chart_demos_use_declarative_canvas_panel) | test(chart_stress_demo_uses_declarative_canvas_panel)'`
  - Result: passed, 2 tests.
  - Scope proven: both basic chart demo and stress demo source-policy gates pass together after
    the stress migration.
- `cargo nextest run -p fret-chart`
  - Result: passed, 43 tests, with the pre-existing `fret-ui`
    `current_effective_opacity` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility baselines and retained chart
    output/linking/tooltip/accessibility oracle tests remain green after migrating the stress demo
    consumer.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the stress demo migration and source-policy test
    extension.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist while remaining retained chart surfaces
    are migrated.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "use fret_chart::retained::ChartCanvas|ChartStressCanvas|impl<.*Widget.*ChartStressCanvas|fret_ui::retained_bridge|ChartCanvas::new\\(|ChartCanvas::create_node|create_node_retained|avg_canvas_paint" apps/fret-examples/src/chart_stress_demo.rs`
  - Result: no matches.
  - Scope proven: the migrated stress demo source no longer contains retained chart widget
    authoring markers or the deleted retained-wrapper paint metric.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-090` is a targeted first-party chart stress demo consumer migration. The
    stress demo compile gate, source-policy red/green gate, full `fret-chart` package gate,
    formatting, layering, catalog, whitespace, and retained-marker scans cover the changed surface.

## 2026-05-22 - RBX-M3-100 Chart multi-axis demo uses declarative panel

Claim verified:

- `chart_multi_axis_demo` no longer constructs retained `ChartCanvas` widgets through
  `ChartCanvas::new_shared(...)` / `ChartCanvas::create_node(...)`.
- The demo no longer composes the two chart panes through retained `FixedSplit` node creation.
- The multi-axis demo now stores each chart engine as a `Model<ChartEngine>` and renders both
  charts through `ChartCanvasPanelProps` + `chart_canvas_panel(...)` inside a declarative vertical
  flex root.
- Declarative chart panels now consume linked brush, linked axis pointer, and linked domain-window
  shared models before stepping/publishing output, preserving the linked chart behavior needed by
  the multi-axis demo and diagnostics auto-zoom.

Evidence:

- `ecosystem/fret-chart/src/declarative/panel.rs`
- `apps/fret-examples/src/chart_multi_axis_demo.rs`
- `apps/fret-examples/tests/basic_chart_demos_surface.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `git status --short --branch && git ls-files -u`
  - Result: no unmerged index entries; branch `main...origin/main [ahead 92]`; only the in-flight
    chart panel file was modified at session start.
  - Scope proven: the user pull left no Git-level conflict entries for this slice.
- `rg -n "<<<<<<<|=======|>>>>>>>" . -g '!target' -g '!repo-ref'`
  - Result: no source merge-conflict markers; matches were only strings in code/docs that mention
    separator patterns.
  - Scope proven: no unresolved textual conflict marker blocked this slice.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: the new declarative linked-input panel API compiles.
- `cargo nextest run -p fret-chart explicit_y_domain_window_propagates_to_second_declarative_chart_output_model`
  - Result: passed, 1 test.
  - Scope proven: a linked explicit Y domain window can propagate through `LinkedChartGroup` into a
    second declarative chart panel output model without constructing retained `ChartCanvas`.
- `cargo check -p fret-demo --bin chart_multi_axis_demo`
  - Result: passed.
  - Scope proven: the native multi-axis demo compiles after replacing retained chart/split nodes
    with a model-backed declarative root.
- `cargo nextest run -p fret-examples chart_multi_axis_demo_uses_declarative_canvas_panel_with_linked_inputs`
  - Result: failed after `cargo fmt` because an intentionally strict source-policy marker still
    expected a pre-rustfmt function signature; passed after adjusting the marker to the formatted
    declarative build shape.
  - Scope proven: the source-policy gate caught marker drift and was re-run green.
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface`
  - Result: passed, 3 tests.
  - Scope proven: the first-party chart demo source-policy target now covers basic demos, stress
    demo, and multi-axis demo retained-authoring bans together.
- `cargo nextest run -p fret-chart`
  - Result: passed, 44 tests, with the pre-existing `fret-ui`
    `current_effective_opacity` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking baselines and retained
    chart output/linking/tooltip/accessibility oracle tests remain green after migrating the
    multi-axis demo and adding declarative linked-input consumption.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the linked-input API, demo migration, and
    source-policy update.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist while remaining retained chart surfaces
    are migrated.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.
- `rg -n "ChartCanvas|FixedSplit|Rc<|RefCell|create_node_retained|new_shared|create_node" apps/fret-examples/src/chart_multi_axis_demo.rs`
  - Result: only `ChartCanvasOutput` type-name matches remain; no retained chart widget authoring,
    retained split node composition, `Rc<RefCell<ChartEngine>>`, or retained create-node markers.
  - Scope proven: the migrated multi-axis demo source no longer teaches retained chart/split
    authoring.
- `cargo nextest run -p fret-examples basic_chart_demos_surface`
  - Result: failed with `error: no tests to run` because this filter does not match individual
    test names.
  - Follow-up: `cargo nextest run -p fret-examples --test basic_chart_demos_surface` passed and is
    the recorded source-policy target for this slice.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-100` is a targeted first-party chart multi-axis demo migration plus
    declarative chart linked-input parity slice. The demo compile gate, declarative linked-domain
    parity test, full `fret-chart` package gate, source-policy test target, formatting, layering,
    catalog, whitespace, conflict-marker, and retained-marker scans cover the changed surface.

## 2026-05-22 - RBX-M3-110 ECharts multi-grid demo uses declarative panels

Claim verified:

- `echarts_multi_grid_demo` no longer uses retained `UniformGrid` /
  `create_multi_grid_chart_canvas_nodes(...)` helpers.
- The demo now stores one shared `Model<ChartEngine>` and renders one declarative chart panel per
  grid plus one overlay-only panel through `fret_ui::declarative::render_root(...)`.
- Declarative chart panels now cover the multi-grid capabilities needed by this demo: per-grid plot
  viewport publication, per-grid series painting, and overlay-only legend/tooltip hit testing that
  falls through outside the legend panel.
- The `ManagedSurface` hit-test mask now clips both the host and full-size descendants, preventing a
  full-size overlay child from stealing input outside host-selected hit regions.

Evidence:

- `ecosystem/fret-chart/src/declarative/panel.rs`
- `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
- `crates/fret-ui/src/declarative/host_widget.rs`
- `crates/fret-ui/src/declarative/tests/managed_surface.rs`
- `apps/fret-examples/src/echarts_multi_grid_demo.rs`
- `apps/fret-examples/tests/basic_chart_demos_surface.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `git ls-files -u`
  - Result: no output.
  - Scope proven: the user pull left no Git-level unresolved conflict entries.
- `rg -n "^(<<<<<<<|=======|>>>>>>>)" apps/fret-examples/src/echarts_multi_grid_demo.rs apps/fret-examples/tests/basic_chart_demos_surface.rs crates/fret-ui/src/declarative/host_widget.rs crates/fret-ui/src/declarative/tests/managed_surface.rs ecosystem/fret-chart/src/declarative/legend_overlay.rs ecosystem/fret-chart/src/declarative/panel.rs docs/workstreams/retained-bridge-exit-v1`
  - Result: no matches.
  - Scope proven: changed code and workstream docs have no textual merge-conflict markers.
- `rg -n "UniformGrid|create_multi_grid_chart_canvas_nodes|ChartCanvas::new_grid_view|ChartCanvas::new_overlay|ChartCanvas::create_node|create_node_retained|Rc<RefCell<ChartEngine>>|std::rc::Rc<std::cell::RefCell<ChartEngine>>" apps/fret-examples/src/echarts_multi_grid_demo.rs`
  - Result: no matches.
  - Scope proven: the migrated multi-grid demo source no longer contains retained multi-grid chart
    helper authoring markers.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: declarative chart panel mode and overlay-mask API changes compile.
- `cargo nextest run -p fret-chart chart_canvas_panel_grid_view_publishes_grid_viewport_without_global_viewport`
  - Result: passed, 1 test.
  - Scope proven: a declarative grid panel writes its panel bounds to
    `plot_viewports_by_grid[grid]` and does not overwrite the shared global viewport.
- `cargo nextest run -p fret-chart chart_canvas_panel_grid_view_paints_only_series_for_that_grid`
  - Result: passed, 1 test.
  - Scope proven: multiple declarative grid panels sharing one engine paint only the marks for
    their assigned grid.
- `cargo nextest run -p fret-chart chart_canvas_panel_overlay_hit_test_falls_through_outside_legend_panel`
  - Result: passed, 1 test.
  - Scope proven: overlay-only chart panels remain interactive over the legend while falling
    through to the underlay outside the legend panel.
- `cargo nextest run -p fret-ui managed_surface_hit_test_mask_clips_full_size_children`
  - Result: passed, 1 test, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: managed-surface hit-test masks clip full-size child subtrees outside the
    host-selected rects.
- `cargo nextest run -p fret-ui declarative::tests::managed_surface`
  - Result: passed, 10 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: the managed-surface mask change did not regress child layout, child paint,
    prepaint, text lifetime, focus request, event, command, availability, or measurement behavior.
- `cargo check -p fret-demo --bin echarts_multi_grid_demo`
  - Result: passed.
  - Scope proven: the native multi-grid demo compiles after replacing retained chart node creation
    with declarative grid panels and an overlay-only panel.
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay`
  - Result: passed, 1 test.
  - Scope proven: the new source-policy gate requires declarative multi-grid panel and overlay
    authoring markers while rejecting retained multi-grid chart helper markers.
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface`
  - Result: passed, 4 tests.
  - Scope proven: first-party chart demo source-policy tests for basic, stress, multi-axis, and
    ECharts multi-grid demos pass together.
- `cargo nextest run -p fret-chart`
  - Result: passed, 47 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines and
    retained chart output/linking/tooltip/accessibility oracle tests remain green after the
    multi-grid migration.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the multi-grid demo migration and tests.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist until remaining retained chart surfaces
    have declarative parity or are deleted.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-110` is a targeted first-party ECharts multi-grid demo migration plus a
    declarative chart-panel multi-grid/overlay parity slice. The demo compile gate, targeted
    multi-grid behavior tests, full `fret-chart` package gate, managed-surface regression gate,
    source-policy target, formatting, layering, catalog, whitespace, conflict-marker, and
    retained-marker scans cover the changed surface.

## 2026-05-22 - RBX-M3-120 Retained chart multi-grid helper deletion

Claim verified:

- The retained `UniformGrid` / `create_multi_grid_chart_canvas_nodes(...)` helper island is deleted.
- `retained::ChartCanvas` no longer exposes no-user multi-surface constructors
  `new_grid_view(...)` / `new_overlay(...)`.
- Ordinary retained `ChartCanvas` remains green as the remaining chart oracle while multi-grid
  first-party authoring is carried by declarative `ChartCanvasPanelMode`.
- The first-party ECharts multi-grid demo still compiles and its source-policy gate still requires
  declarative grid panels plus an overlay-only panel.

Evidence:

- `ecosystem/fret-chart/src/lib.rs`
- `ecosystem/fret-chart/src/retained/mod.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- deleted `ecosystem/fret-chart/src/retained/multi_grid.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart retained_multi_grid_helpers_are_removed_from_public_surface`
  - Result: failed first after adding the source-policy test because retained `mod multi_grid` /
    `pub use multi_grid::*` still existed; passed after deleting the helper module and no-user
    retained multi-surface constructors.
  - Scope proven: the policy test acted as the red/green gate for deleting the retained multi-grid
    public surface.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: `fret-chart` compiles after removing retained multi-grid helper exports and
    retained `ChartCanvas` shared-engine/mode branches.
- `cargo nextest run -p fret-chart`
  - Result: passed, 48 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines and
    ordinary retained chart output/linking/tooltip/legend/visual-map/slider/keyboard oracle tests
    remain green after the retained multi-grid helper deletion.
- `cargo check -p fret-demo --bin echarts_multi_grid_demo`
  - Result: passed.
  - Scope proven: the native ECharts multi-grid demo still compiles through the declarative
    multi-grid panel path after deleting retained multi-grid helpers.
- `cargo nextest run -p fret-examples --test basic_chart_demos_surface echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay`
  - Result: passed, 1 test.
  - Scope proven: the first-party demo source-policy gate still requires declarative multi-grid
    panel and overlay markers while rejecting retained multi-grid markers.
- `rg -n "multi_grid|UniformGrid|create_multi_grid_chart_canvas_nodes|new_grid_view|new_overlay|ChartCanvasMode|grid_override|paint_overlay_only|new_shared|SharedChartEngine|ChartCanvasEngine" ecosystem/fret-chart/src apps ecosystem crates -g '*.rs' -g 'Cargo.toml'`
  - Result: no retained chart helper implementation matches; remaining matches are declarative
    multi-grid tests, ECharts/delinea headless multi-grid tests, and source-policy marker strings.
  - Scope proven: the retained multi-grid helper implementation and no-user retained constructors
    are gone from Rust source.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after deleting the helper and simplifying retained
    `ChartCanvas`.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist for remaining retained chart surfaces.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: tracked changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-120` is a targeted retained chart multi-grid helper deletion. The red/green
    public-surface policy test, full `fret-chart` package gate, ECharts multi-grid demo compile
    gate, demo source-policy gate, formatting, layering, catalog, whitespace, and retained-marker
    scan cover the changed surface.

## 2026-05-22 - RBX-M3-130 Chart style and tooltip contracts moved out of retained namespace

Claim verified:

- Shared chart style and tooltip contracts now live in top-level `fret-chart` modules instead of
  retained-owned modules.
- Declarative chart panel, overlay, and output code no longer import these shared contracts through
  `crate::retained::*`.
- Retained `ChartCanvas` remains as the ordinary chart behavior oracle while consuming the same
  top-level style/tooltip contracts.
- A `fret-chart` public-surface policy test prevents declarative shared contracts from depending
  on retained namespace markers again.

Evidence:

- `ecosystem/fret-chart/src/lib.rs`
- `ecosystem/fret-chart/src/style.rs`
- `ecosystem/fret-chart/src/tooltip.rs`
- `ecosystem/fret-chart/src/retained/mod.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/declarative/panel.rs`
- `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
- `ecosystem/fret-chart/src/declarative/tooltip_overlay.rs`
- `ecosystem/fret-chart/src/output.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `git ls-files -u`
  - Result: no output.
  - Scope proven: the user pull left no Git-level unresolved conflict entries before this slice.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after moving the modules and imports.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: the top-level style/tooltip modules, declarative imports, retained imports, and
    crate-root exports compile.
- `cargo nextest run -p fret-chart`
  - Result: passed, 49 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    top-level tooltip/style tests, and ordinary retained chart output/linking/tooltip/legend/
    visual-map/slider/keyboard oracle tests remain green after the namespace move.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid; `fret-chart`
    intentionally remains on the retained bridge allowlist for remaining retained chart surfaces.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-130` is a targeted namespace and contract-ownership move within `fret-chart`.
    The full `fret-chart` package gate, public-surface policy test, formatting, layering, catalog,
    and whitespace checks cover the changed surface.

## 2026-05-22 - RBX-M3-135 Chart linking output contract moved off retained namespace

Claim verified:

- `LinkedChartMember` and `LinkedChartGroup` now consume the top-level `ChartCanvasOutput`
  contract instead of naming it through `crate::retained::*`.
- A red/green `fret-chart` public-surface policy test prevents chart linking from depending on
  retained output namespace markers again.
- Linked chart behavior remains covered by the full `fret-chart` package gate after the namespace
  move.

Evidence:

- `ecosystem/fret-chart/src/linking.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart chart_linking_does_not_depend_on_retained_output_namespace`
  - Result: failed first after adding the policy test because `linking.rs` still used
    `crate::retained::ChartCanvasOutput`; passed after moving the import to top-level
    `ChartCanvasOutput`.
  - Scope proven: the policy test acted as the red/green gate for removing the retained output
    namespace dependency from chart linking.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the linking import move and policy test.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: chart linking compiles when naming the shared output contract from the top-level
    crate surface.
- `cargo nextest run -p fret-chart`
  - Result: passed, 50 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    top-level tooltip/style tests, public-surface policy tests, and ordinary retained chart oracle
    tests remain green after the linking output namespace move.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-135` is a targeted `fret-chart` linking namespace cleanup. The red/green
    policy test, full `fret-chart` package gate, formatting, layering, catalog, and whitespace
    checks cover the changed surface.

## 2026-05-22 - RBX-M3-140 Retained chart output re-export deletion

Claim verified:

- The no-user `retained::ChartCanvasOutput` / `retained::ChartCanvasOutputSnapshot` compatibility
  re-export has been deleted.
- Retained `ChartCanvas` now consumes top-level `ChartCanvasOutput` directly while remaining as the
  ordinary retained chart behavior oracle.
- A `fret-chart` public-surface policy test prevents the retained output re-export from returning.

Evidence:

- `ecosystem/fret-chart/src/retained/mod.rs`
- deleted `ecosystem/fret-chart/src/retained/output.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `rg -n "retained::ChartCanvasOutput|crate::retained::ChartCanvasOutput|pub use crate::output::\\{ChartCanvasOutput" ecosystem/fret-chart apps ecosystem crates -g '*.rs'`
  - Result: no external consumer matches; remaining pre-change matches were the retained
    compatibility re-export, retained canvas's self-import, and public-surface policy marker
    strings.
  - Scope proven: deleting the retained output re-export does not remove a known first-party
    consumer path.
- `cargo nextest run -p fret-chart retained_output_reexport_is_removed_from_public_surface`
  - Result: passed, 1 test, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: retained `mod output` / `pub use output::*` stayed deleted and retained
    `ChartCanvas` no longer imports `crate::retained::ChartCanvasOutput`.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after deleting the re-export module.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: `fret-chart` compiles after deleting the retained output re-export and moving
    retained `ChartCanvas` to the top-level output contract import.
- `cargo nextest run -p fret-chart`
  - Result: passed, 51 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    top-level tooltip/style tests, public-surface policy tests, and ordinary retained chart oracle
    tests remain green after the retained output re-export deletion.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-140` is a targeted deletion of a no-user `fret-chart` compatibility re-export.
    The consumer scan, public-surface policy test, full `fret-chart` package gate, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-22 - RBX-M3-145 Retained chart widget crate-root glob re-export removal

Claim verified:

- Retained chart widgets are no longer glob re-exported from the `fret-chart` crate root.
- Retained chart widgets remain available only through explicit `fret_chart::retained` imports
  while retained `ChartCanvas` continues to serve as the behavior oracle.
- A `fret-chart` public-surface policy test prevents `pub use retained::*` from returning.

Evidence:

- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `rg -n "use fret_chart::\\{[^\\n]*(ChartCanvas|ChartCanvasOutput|ChartCanvasPanel)|use fret_chart::ChartCanvas|fret_chart::ChartCanvas|ChartCanvas::" apps ecosystem crates -g '*.rs'`
  - Result: no first-party source consumers import `ChartCanvas` from the crate root; remaining
    ordinary consumers use declarative panels or top-level shared output contracts, while retained
    `ChartCanvas` references are internal oracle tests or policy marker strings.
  - Scope proven: removing the crate-root retained glob re-export does not break known first-party
    chart consumers.
- `cargo nextest run -p fret-chart retained_widgets_are_not_glob_reexported_from_crate_root`
  - Result: failed first because the test's literal marker self-matched; passed after building the
    marker dynamically and checking the actual crate-root source.
  - Scope proven: the policy test now guards against reintroducing `pub use retained::*`.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after the crate-root export change.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: `fret-chart` compiles without the crate-root retained glob re-export.
- `cargo nextest run -p fret-chart`
  - Result: passed, 52 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    top-level tooltip/style tests, public-surface policy tests, and ordinary retained chart oracle
    tests remain green after the public-surface contraction.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-145` is a targeted `fret-chart` public-surface contraction. The first-party
    consumer scan, public-surface policy test, full `fret-chart` package gate, formatting,
    layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-22 - RBX-M3-150 Chart legend scroll policy moved to shared logic

Claim verified:

- Chart legend scroll max/clamp/wheel policy now lives in shared `legend_logic` instead of being
  duplicated in retained and declarative chart paths.
- Retained `ChartCanvas` and the declarative legend overlay both consume the shared scroll policy.
- Retained legend scroll oracle coverage remains green, and direct shared-policy tests cover the
  moved behavior.

Evidence:

- `ecosystem/fret-chart/src/legend_logic.rs`
- `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart legend_scroll_policy`
  - Result: passed, 3 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: shared legend scroll policy clamps to content height, applies the retained 0.75
    wheel speed, resets stale scroll when content fits, and both retained/declarative paths route
    through the shared policy.
- `cargo nextest run -p fret-chart legend_scroll_clamps_to_content_height`
  - Result: passed, 1 test, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: the retained legend scroll oracle remains green after delegating to shared
    `legend_logic`.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: retained and declarative chart paths compile after the shared policy extraction,
    with no new `fret-chart` dead-code warnings.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the shared policy extraction.
- `cargo nextest run -p fret-chart`
  - Result: passed, 55 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    shared legend/tooltip/style tests, public-surface policy tests, and ordinary retained chart
    oracle tests remain green after moving legend scroll policy.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-150` is a targeted `fret-chart` shared-policy extraction. The direct shared
    policy tests, retained oracle test, source-policy test, full `fret-chart` package gate,
    formatting, layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-22 - RBX-M3-160 Chart slider math policy moved to shared logic

Claim verified:

- Pure chart slider math for data-zoom and visual-map interactions now lives in shared
  `slider_logic` instead of retained `ChartCanvas`.
- Retained `ChartCanvas` consumes shared slider norm/value/window policy while still owning retained
  event routing and engine action orchestration as the current oracle.
- Shared slider tests and a public-surface policy test prevent the pure slider math helpers from
  returning to retained canvas.

Evidence:

- `ecosystem/fret-chart/src/slider_logic.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart slider_`
  - Result: failed first because the new source-policy test expected fully qualified
    `crate::slider_logic::*` markers while retained canvas imported the helpers and used short
    names; passed after checking the shared import plus short-name usage.
  - Scope proven: shared slider math tests, the retained slider window oracle, and the source-policy
    guard pass together.
- `cargo nextest run -p fret-chart slider_math_policy_lives_in_shared_logic`
  - Result: passed, 1 test, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: retained `ChartCanvas` no longer defines the pure slider math functions and
    routes through shared `slider_logic`.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: retained data-zoom and visual-map slider paths compile after the shared-policy
    extraction, with no new `fret-chart` warnings.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after adding `slider_logic` and simplifying retained
    canvas.
- `cargo nextest run -p fret-chart`
  - Result: passed, 58 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    shared legend/slider/tooltip/style tests, public-surface policy tests, and ordinary retained
    chart oracle tests remain green after moving slider math policy.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 428 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-160` is a targeted `fret-chart` shared-policy extraction. The direct shared
    slider tests, retained slider oracle test, source-policy test, full `fret-chart` package gate,
    formatting, layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-22 - RBX-M3-170 Chart visual-map policy moved to shared logic

Claim verified:

- Pure chart visual-map track layout, track hit selection, domain-window conversion, and value-to-y
  mapping now live in shared `visual_map_logic` instead of retained `ChartCanvas`.
- Retained `ChartCanvas` consumes shared visual-map geometry/mapping policy while still owning
  retained paint, event routing, and engine action orchestration as the current oracle.
- Shared visual-map tests and a public-surface policy test prevent the pure visual-map helpers from
  returning to retained canvas.

Evidence:

- `ecosystem/fret-chart/src/visual_map_logic.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart visual_map`
  - Result: failed first because retained tests still called the removed
    `ChartCanvas::visual_map_y_at_value`; passed after moving that assertion to the shared helper.
    A second red/green step fixed the shared gap/padding test expectation from the item origin to
    the padded track origin.
  - Scope proven: shared visual-map endpoint mapping, padded/gapped track layout, hit selection,
    retained visual-map y-mapping oracle, retained visual-map style-padding oracle, and the
    source-policy guard pass together.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: retained visual-map paint/event paths compile after the shared-policy extraction,
    with no new `fret-chart` warnings.
- `cargo fmt --check`
  - Result: passed after resolving and formatting the unrelated Gallery merge-conflict file in a
    separate merge-format commit.
  - Scope proven: Rust formatting is clean after adding `visual_map_logic` and simplifying retained
    canvas.
- `cargo nextest run -p fret-chart`
  - Result: passed, 62 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    shared legend/slider/visual-map/tooltip/style tests, public-surface policy tests, and ordinary
    retained chart oracle tests remain green after moving visual-map policy.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 429 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-170` is a targeted `fret-chart` shared-policy extraction. The direct shared
    visual-map tests, retained visual-map oracle tests, source-policy test, full `fret-chart`
    package gate, formatting, layering, catalog, and whitespace checks cover the changed surface.

## 2026-05-22 - RBX-M3-180 Chart visual-map interaction policy moved to shared logic

Claim verified:

- Visual-map piecewise mask/reset/shift-range decision policy now lives in shared
  `visual_map_logic` instead of retained `ChartCanvas`.
- Continuous visual-map handle-vs-pan-vs-jump drag-start decision policy now lives in shared
  `visual_map_logic` instead of retained `ChartCanvas`.
- Retained `ChartCanvas` consumes the shared decisions while still owning retained event routing,
  pointer capture, invalidation/redraw, and engine action orchestration as the current oracle.

Evidence:

- `ecosystem/fret-chart/src/visual_map_logic.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart visual_map`
  - Result: passed, 8 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: shared visual-map piecewise toggle/range/reset policy, continuous handle/pan/jump
    drag-start policy, retained visual-map style/y-mapping oracles, and the source-policy guard
    pass together.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: retained visual-map event/action paths compile after the shared interaction
    policy extraction, with no new `fret-chart` warnings.
- `cargo nextest run -p fret-chart`
  - Result: passed, 64 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    shared legend/slider/visual-map/tooltip/style tests, public-surface policy tests, and ordinary
    retained chart oracle tests remain green after moving visual-map interaction policy.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after extending `visual_map_logic` and simplifying
    retained canvas event logic.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 429 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-180` is a targeted `fret-chart` visual-map interaction policy extraction. The
    direct shared-policy tests, retained visual-map oracle tests, source-policy test, full
    `fret-chart` package gate, formatting, layering, catalog, and whitespace checks cover the
    changed surface.

## 2026-05-22 - RBX-M3-190 Chart data-zoom slider interaction policy moved to shared logic

Claim verified:

- Data-zoom slider handle-vs-pan-vs-jump drag-start policy now lives in shared `slider_logic`
  instead of retained `ChartCanvas`.
- Data-zoom slider drag-update projection and span-anchor policy now live in shared `slider_logic`.
- Retained `ChartCanvas` consumes the shared decisions while still owning retained event routing,
  pointer capture, invalidation/redraw, and engine action orchestration as the current oracle.

Evidence:

- `ecosystem/fret-chart/src/slider_logic.rs`
- `ecosystem/fret-chart/src/retained/canvas.rs`
- `ecosystem/fret-chart/src/lib.rs`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo nextest run -p fret-chart slider_`
  - Result: passed, 7 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: shared slider x/y drag-start, handle/pan/jump, permission locks, drag-update
    projection, window-anchor policy, retained slider window oracle, and source-policy guard pass
    together.
- `cargo check -p fret-chart`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: retained data-zoom slider event/action paths compile after the shared interaction
    policy extraction, with no new `fret-chart` warnings.
- `cargo nextest run -p fret-chart`
  - Result: passed, 67 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: declarative chart paint/output/accessibility/linking/multi-grid baselines,
    shared legend/slider/visual-map/tooltip/style tests, public-surface policy tests, and ordinary
    retained chart oracle tests remain green after moving data-zoom slider interaction policy.
- `cargo fmt --check`
  - Result: passed.
  - Scope proven: Rust formatting is clean after extending `slider_logic` and simplifying retained
    canvas event logic.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering and retained bridge allowlist policy remain valid.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 429 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-190` is a targeted `fret-chart` data-zoom slider interaction policy extraction.
    The direct shared-policy tests, retained slider oracle test, source-policy test, full
    `fret-chart` package gate, formatting, layering, catalog, and whitespace checks cover the
    changed surface.

## 2026-05-22 - RBX-M3-200 Default declarative `fret-plot` line plot baseline and retained bridge isolation

Claim verified:

- Default `fret-plot` no longer enables `fret-ui/unstable-retained-bridge`.
- Shared plot data/state/style contracts live on the default `fret_plot::{models,state,style}`
  surface instead of requiring `fret_plot::retained`.
- A default declarative `line_plot_panel(...)` renders seeded line-series data through
  `fret_ui::ElementContext::canvas(...)` without constructing retained `PlotCanvas`.
- Retained plot canvases remain available only behind the explicit `compat-retained-canvas` feature
  as the migration oracle for remaining retained plot demos and interactions.

Evidence:

- `ecosystem/fret-plot/Cargo.toml`
- `ecosystem/fret-plot/src/declarative.rs`
- `ecosystem/fret-plot/src/lib.rs`
- `ecosystem/fret-plot/src/models.rs`
- `ecosystem/fret-plot/src/state.rs`
- `ecosystem/fret-plot/src/style.rs`
- `ecosystem/fret-plot/src/retained/mod.rs`
- `apps/fret-examples/Cargo.toml`
- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1-todo.md`
- `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`

Commands:

- `cargo check -p fret-plot`
  - Result: passed, with the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.
  - Scope proven: default `fret-plot` compiles without enabling retained bridge features and with
    no new `fret-plot` warnings.
- `cargo nextest run -p fret-plot line_plot_panel_paints_seeded_line_on_declarative_path`
  - Result: passed, 1 test, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: default declarative line plot panel can render/layout/paint seeded line data and
    emits a canvas path without retained `PlotCanvas`.
- `cargo nextest run -p fret-plot`
  - Result: passed, 23 tests, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: default `fret-plot` model, linking, input-map, cartesian, decimation, axis, and
    public-surface policy tests remain green after moving shared contracts out of retained.
- `cargo check -p fret-plot --features compat-retained-canvas`
  - Result: passed, with the pre-existing `fret-ui` dead-code warning.
  - Scope proven: retained plot canvases still compile as an explicit compatibility oracle.
- `cargo check -p fret-examples`
  - Result: passed.
  - Scope proven: first-party examples that still consume `fret_plot::retained::*` continue to
    compile through the explicit `fret-plot/compat-retained-canvas` opt-in instead of relying on
    default retained bridge access.
- `cargo metadata --no-deps --format-version 1 | python3 -c '...'`
  - Result: printed an empty feature list for `fret-plot`'s `fret-ui` dependency.
  - Scope proven: default `fret-plot` depends on `fret-ui` without `unstable-retained-bridge`.
- `rg -n "unstable-retained-bridge|compat-retained-canvas|pub mod retained|LinePlotPanelProps|line_plot_panel" ecosystem/fret-plot/Cargo.toml ecosystem/fret-plot/src -g '*.rs' -g 'Cargo.toml'`
  - Result: retained bridge usage appears only in the `compat-retained-canvas` feature definition,
    retained module/compat gates, and source-policy tests; declarative line-plot API markers are
    present.
  - Scope proven: retained bridge access is explicitly gated and the default declarative entry
    point exists.
- `cargo fmt --check`
  - Result: passed after running `cargo fmt`.
  - Scope proven: Rust formatting is clean after the module moves and declarative panel addition.
- `python3 tools/check_layering.py`
  - Result: passed.
  - Scope proven: crate layering remains valid after narrowing `fret-plot` retained bridge access.
- `python3 tools/check_workstream_catalog.py`
  - Result: passed; validated 429 dedicated directories and 47 standalone markdown files.
  - Scope proven: workstream catalog indexes remain valid after task/evidence/handoff updates.
- `git diff --check`
  - Result: passed.
  - Scope proven: changed files have no whitespace errors.

Broader gates not run:

- `cargo nextest run --workspace`
  - Reason: `RBX-M3-200` is a targeted `fret-plot` default-surface and declarative-baseline slice.
    The default/compat `fret-plot` checks, full `fret-plot` package tests, metadata dependency
    proof, formatting, layering, catalog, and whitespace checks cover the changed surface.
