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
