# Retained Bridge Exit Plan v1 — TODO Tracker

Status: Active (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1.md`

## Milestones

### M0 — Governance gates (blast radius control)

- [x] CI: reject `crates/* -> ecosystem/*` reverse dependencies (`tools/check_layering.py`).
- [x] CI: restrict `fret-ui/unstable-retained-bridge` to an explicit allowlist (`tools/check_layering.py`).
- [x] Document the current allowlist and rationale per crate.
  - Source of truth: `tools/check_layering.py` (`unstable_retained_bridge_allowlist`).
  - Current allowlist (workspace crate names):
    - `fret-node`
      - Why: node graph canvas + portal editors are still authored as retained widgets inside the
        explicit `compat-retained-canvas` compatibility island; it also exercises
        overlays/commands in the retained path.
      - Evidence: `ecosystem/fret-node/Cargo.toml` enables `fret-ui/unstable-retained-bridge` only
        through `compat-retained-canvas`; retained widget surface in
        `ecosystem/fret-node/src/ui/canvas/widget.rs`.
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
  - Removed from allowlist:
    - `fret-docking`
      - Result: removed in `RBX-M1-080`; docking now uses public declarative dock-space entry
        points and no longer depends on `fret-ui/unstable-retained-bridge`.

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
- [x] RBX-M1-060 Decide and prove the declarative docking host mechanism.
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
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-19---rbx-m1-060-declarative-managed-surface-host-proof`
  - Result:
    - Existing declarative primitives were not sufficient because docking needs host-selected
      child-root layout, prepaint liveness, and controlled child paint order without exposing the
      retained `Widget` API.
    - Added a narrow mechanism-only `ManagedSurface` primitive in `fret-ui`; it owns no docking
      policy and only exposes layout/prepaint/paint hooks to place and paint child roots.
    - Added `fret-ui` proof tests for child-root placement, child-root paint order/rects, and
      prepaint output flow.
    - Extracted `panel_root_placements_for_snapshot(...)` so retained `DockSpace::layout` and a
      future declarative dock host can share the same panel placement decision.
    - Made `DockSpaceLayoutSnapshot::paint_panel_bounds` graph-order stable instead of relying on
      `HashMap` iteration order.
    - Added a docking proof that a declarative managed surface consumes `DockSpaceLayoutSnapshot`
      for panel-root layout and paint without `RetainedSubtreeProps`.
- [x] RBX-M1-070 Replace public retained docking entry points.
  - Scope:
    - `ecosystem/fret-docking/src/dock/mod.rs`
    - `ecosystem/fret-docking/src/imui.rs`
    - call sites in apps/examples that create dock spaces through retained helpers.
  - Goal:
    - Add or switch to public declarative docking entry points backed by the managed-surface
      mechanism while preserving docking policy in `fret-docking`.
    - Keep retained `DockSpace` only as a private/temporary compatibility adapter until the
      declarative entry points cover the public integration surface.
  - Validation:
    - `cargo nextest run -p fret-docking`
    - targeted app/example checks for docking entry point consumers
    - `python3 tools/check_layering.py`
  - Evidence:
    - `ecosystem/fret-docking/src/dock/declarative.rs`
    - `ecosystem/fret-docking/src/dock/mod.rs`
    - `ecosystem/fret-docking/src/imui.rs`
    - `ecosystem/fret-docking/tests/public_surface_policy.rs`
    - `ecosystem/fret-docking/src/dock/tests/dock_space.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-19---rbx-m1-070-public-declarative-docking-entry-points`
  - Result:
    - Added public declarative docking entry points backed by `ManagedSurface`:
      `dock_space_element(...)`, `dock_space_element_from_registry(...)`, and
      `dock_panel_element(...)`.
    - Added `DockPanelElementRegistry` / `DockPanelElementRegistryService` so declarative panel
      content returns `AnyElement` roots instead of retained `UiTree` node ids.
    - Added an imui declarative wrapper (`dock_space_declarative*`) that mounts the new public
      declarative host path.
    - Documented the retained `create_dock_space_node*` / `mount_dock_space*` helpers as legacy and
      locked that policy with a public surface test.
    - Kept retained `DockSpace` as the temporary full-interaction adapter because drag/event/command
      policy is still implemented through retained `Widget` hooks. Migrating app/demo call sites to
      the declarative host before that migration would silently drop editor-grade docking
      interactions.
- [x] RBX-M1-075 Move docking interaction host hooks off the retained `Widget` adapter.
  - Scope:
    - `crates/fret-ui/src/managed_surface.rs`
    - `crates/fret-ui/src/declarative/host_widget.rs`
    - `crates/fret-ui/src/declarative/host_widget/event/mod.rs`
    - `crates/fret-ui/src/elements/cx.rs`
    - `ecosystem/fret-docking/src/dock/space.rs`
    - `ecosystem/fret-docking/src/dock/declarative.rs`
    - app/example docking entry-point call sites.
  - Goal:
    - Extract event, command, focus, internal-drag route, diagnostics, chrome painting, and viewport
      capture hooks so the declarative host can replace the retained `DockSpace` adapter for real
      app/demo entry points.
    - After this task, app/example call sites should no longer need `create_dock_space_node*`.
  - Validation:
    - `cargo nextest run -p fret-docking`
    - `cargo nextest run -p fret-ui managed_surface`
    - targeted app/example checks for `docking_demo`, `container_queries_docking_demo`,
      `docking_basics`, and `imui_editor_proof_demo`
    - `python3 tools/check_layering.py`
  - Progress:
    - 2026-05-19 first slice completed:
      - Extended `ManagedSurface` with mechanism-only event, command, and command availability
        hooks. These hooks expose lifecycle context capabilities needed by docking without
        exposing the retained `Widget` API.
      - Added a `fret-ui` proof that a declarative managed surface can receive pointer events,
        request focus, handle commands, and answer command availability.
      - Moved `dock.focus_requested_panel` handling onto the public declarative dock-space host so
        `dock_space_element_from_registry(...)` can focus the requested panel root without the
        retained `DockSpace::command` adapter.
      - Locked the new docking path with a public declarative dock-space focus-request test.
    - 2026-05-19 second slice completed:
      - Exposed current host `node()` through `ManagedSurface` layout/prepaint/paint contexts.
      - Made the public declarative dock-space host refresh dock panel/tabs internal-drag routes
        during layout, prepaint, and paint, matching the retained adapter keep-alive contract.
      - Made the public declarative dock-space host register as `DockManager::dock_space_node(...)`
        for its window, matching the retained adapter window-anchor contract.
      - Locked the declarative route-anchor path with a public declarative dock-space test.
    - 2026-05-19 third slice completed:
      - Moved common docking diagnostics publication into `dock/diagnostics.rs` so retained and
        declarative hosts share one graph/drag diagnostics path instead of carrying duplicate
        `DockSpace`-local helpers.
      - Made the public declarative dock-space host publish `WindowInteractionDiagnosticsStore`
        snapshots from its prepaint hook, including active dock-drag diagnostics, dock graph stats,
        and dock graph signature.
      - Made the public declarative dock-space host request animation frames while an active dock
        drag affects its window, preserving the retained adapter's prepaint liveness intent for the
        declarative path.
      - Locked the declarative diagnostics/liveness path with a public declarative dock-space test.
    - 2026-05-19 fourth slice completed:
      - Extended `ManagedSurfacePaintCx` with mechanism-only access to `scale_factor()`,
        `services()`, and `child_bounds(...)`, keeping those capabilities in `fret-ui` without
        exposing the retained `Widget` API.
      - Updated the public declarative dock-space host to paint panel roots using actual child
        bounds with the snapshot rect as a fallback, matching retained `DockSpace::paint`
        child-root fallback semantics more closely.
      - Locked the new managed-surface paint capabilities with a focused `fret-ui` test.
    - 2026-05-19 fifth slice completed:
      - Added declarative viewport layout sync from the public dock-space host's shared
        `DockSpaceLayoutSnapshot` path into `DockManager::sync_viewport_layouts_for_window(...)`.
      - Runs the sync from layout and prepaint so app/editor viewport consumers can read stable
        viewport mapping/draw rect state without retained `DockSpace::paint`.
      - Locked stale-layout cleanup and viewport mapping publication with a public declarative
        dock-space test.
    - 2026-05-19 sixth slice completed:
      - Extracted split-handle paint inputs from retained `paint_split_handles(...)` into reusable
        `split_handle_paint_inputs(...)` / `paint_split_handle_inputs(...)` helpers.
      - Made the public declarative dock-space host carry split-handle paint inputs in its
        per-frame output and paint split handles without borrowing retained `DockSpace`.
      - Locked the declarative split-handle paint path through the public declarative dock-space
        panel-root test.
    - 2026-05-19 seventh slice completed:
      - Extracted viewport-surface paint inputs from retained `paint_dock(...)` into reusable
        viewport paint-input helpers.
      - Made the public declarative dock-space host carry viewport-surface paint inputs in its
        per-frame output and paint `SceneOp::ViewportSurface` plus viewport overlay hooks from its
        managed-surface paint hook.
      - Locked the pure viewport-panel declarative path with a public declarative dock-space test.
    - 2026-05-19 eighth slice completed:
      - Extracted floating container chrome paint inputs from retained `DockSpace::paint` into
        reusable `FloatingChromePaintInput` / `paint_floating_chrome_input(s)` helpers.
      - Made the public declarative dock-space host carry floating chrome paint inputs in its
        per-frame output and paint in-window floating outer/title-bar chrome from its
        managed-surface paint hook without borrowing retained `DockSpace`.
      - Locked the public declarative floating chrome path with a public declarative dock-space
        test.
    - 2026-05-19 ninth slice completed:
      - Extracted active dock drag ghost snapshot selection into a reusable diagnostics helper
        shared by retained and declarative hosts.
      - Made the public declarative dock-space host carry drag ghost snapshots in its per-frame
        output, prepare the dragged panel title through `ManagedSurfacePaintCx::services()`, and
        paint the payload ghost from its managed-surface paint hook.
      - Locked the public declarative drag payload ghost path with a public declarative dock-space
        test.
    - 2026-05-19 tenth slice completed:
      - Extracted basic float/empty/center drop-overlay painting into a reusable
        `paint_basic_drop_overlay(...)` helper that does not require retained `DockSpace` state.
      - Made the public declarative dock-space host carry `DockManager::hover` plus
        `DockSpaceLayoutSnapshot::layout_all` in its per-frame output and paint center drop
        overlays from its managed-surface paint hook.
      - Locked the public declarative center drop-overlay path with a public declarative
        dock-space test.
    - 2026-05-19 eleventh slice completed:
      - Made the public declarative dock-space host derive `DockDropHints` from the resolved
        `DockManager::hover` target and carry the hints in its per-frame output.
      - Reused the shared `paint_drop_hints(...)` helper from the managed-surface paint hook so
        declarative docking paints the ImGui-style drop-hint plate and pads without borrowing
        retained `DockSpace`.
      - Locked the public declarative drop-hint pad path with a public declarative dock-space test.
    - 2026-05-19 twelfth slice completed:
      - Extracted the structural tab chrome quads (panel background, tab bar, active/hover tab
        plate, active underline) into reusable `TabChromePaintInput` /
        `paint_tab_chrome_inputs(...)` helpers.
      - Retained `paint_dock(...)` now delegates those structural tab chrome quads through the
        shared helper, while still owning tab title, close button, overflow, and viewport fill
        details.
      - Made the public declarative dock-space host carry tab chrome inputs in its per-frame output
        and paint tab bar chrome from its managed-surface paint hook before painting panel roots.
      - Locked the public declarative tab chrome path with a public declarative dock-space test.
    - 2026-05-19 thirteenth slice completed:
      - Extracted non-text complex drop-overlay geometry into reusable
        `ComplexDropOverlayPaintInput` / `paint_complex_drop_overlay_inputs(...)` helpers.
      - Retained `paint_drop_overlay(...)` now delegates tab insert markers and edge split-slot
        preview overlays through the shared helper while still owning tab-title preview text.
      - Made the public declarative dock-space host carry complex drop-overlay inputs in its
        per-frame output and paint edge split-slot previews plus tab insert markers from its
        managed-surface paint hook.
      - Locked the public declarative edge preview and tab insert marker paths with public
        declarative dock-space tests.
    - 2026-05-19 fourteenth slice completed:
      - Extracted tab-insert preview title painting into reusable
        `paint_tab_insert_preview_title(...)` so retained and declarative dock-space hosts share
        the same preview title behavior.
      - Extended `ManagedSurfacePaintCx` with `release_text_blob_on_next_paint(...)` so
        paint-time transient text blobs remain valid for the scene that references them and are
        released on the next managed-surface repaint or cleanup.
      - Made the public declarative dock-space host paint tab-insert preview titles from its
        managed-surface paint hook without borrowing retained `DockSpace`.
      - Locked the public declarative tab-insert preview title path with a public declarative
        dock-space test.
    - 2026-05-19 fifteenth slice completed:
      - Extracted tab title, active-tab close affordance, overflow button, and overflow menu
        painting into reusable `TabDetailPaintInput` / `paint_tab_detail_inputs(...)` helpers.
      - Retained `paint_dock(...)` now delegates tab detail painting through the shared helper
        while still owning the retained tab resource cache and interaction state.
      - Made the public declarative dock-space host prepare transient tab title/close/overflow text
        resources and paint tab details from its managed-surface paint hook.
      - Locked the public declarative tab detail path with a public declarative dock-space test.
    - 2026-05-19 sixteenth slice completed:
      - Moved the active-tab close affordance `PointerDown` / `PointerUp` path onto the public
        declarative dock-space host.
      - Kept `ManagedSurfaceEventCx` mechanism-only by avoiding prepaint-output reads from event
        hooks; the declarative docking host rebuilds a temporary `DockSpaceLayoutSnapshot` from the
        current bounds and `DockManager` for close hit-testing.
      - Added declarative docking interaction state for pressed tab-close tracking and pointer
        capture, then emits `DockOp::ClosePanel` through `Effect::Dock` on click/slop release.
      - Locked the public declarative active-tab close path with a public declarative dock-space
        event test.
    - 2026-05-19 seventeenth slice completed:
      - Moved the tab overflow button/menu close click path onto the public declarative dock-space
        host.
      - Added declarative overflow-menu state in `fret-docking`, reused existing
        `TabOverflowMenuState` and `paint_tab_detail_inputs(...)`, and kept the state out of
        `fret-ui`.
      - The declarative host now opens the overflow menu, paints it through the existing tab detail
        helper, emits `DockOp::ClosePanel` for overflow-menu row close without also activating the
        tab, and emits `DockOp::SetActiveTab` for overflow-menu row activation without closing a
        tab.
      - Locked the public declarative overflow-menu close and activation paths with public
        declarative dock-space event/paint tests.
    - 2026-05-19 eighteenth slice completed:
      - Moved the in-window floating close `PointerDown` / `PointerUp` path onto the public
        declarative dock-space host.
      - Added declarative pressed-floating-close state in `fret-docking`, keeping floating close
        policy out of `fret-ui`.
      - The declarative host now raises a floating container on close press, captures/releases the
        pointer, and emits `DockOp::MergeFloatingInto { ... }` on release over the same close
        affordance.
      - Reused the existing `DockSpaceLayoutSnapshot` floating chrome geometry for close
        hit-testing and pressed close painting.
      - Locked the public declarative floating close path with a declarative dock-space event test.
    - 2026-05-19 nineteenth slice completed:
      - Moved the in-window floating title-bar drag `PointerDown` / `PointerMove` / `PointerUp`
        state onto the public declarative dock-space host for the narrow move-rect path.
      - Added declarative floating-drag state in `fret-docking`, keeping drag policy out of
        `fret-ui`.
      - The declarative host now raises a floating container on title-bar press, captures/releases
        the pointer, and emits `DockOp::SetFloatingRect { ... }` while the title bar is dragged.
      - Reused the existing `DockSpaceLayoutSnapshot` floating chrome geometry for title-bar
        hit-testing and the same clamp-to-window-bounds behavior as the retained adapter.
      - Locked the public declarative floating title-bar drag path with a declarative dock-space
        event test.
    - 2026-05-19 twentieth slice completed:
      - Moved overflow-menu wheel scrolling onto the public declarative dock-space host.
      - Reused the retained adapter's menu geometry and scroll formula:
        `next_scroll = (menu.scroll - (delta.x + delta.y)).clamp(0, max_scroll)`.
      - Kept overflow-menu state in `fret-docking`; `fret-ui` remains a mechanism-only
        `ManagedSurface` host with no docking policy/state.
      - Locked the public declarative overflow-menu wheel path with a test that scrolls the menu
        and then activates a row exposed by the new scroll offset.
    - 2026-05-19 twenty-first slice completed:
      - Moved tab-strip wheel scrolling onto the public declarative dock-space host.
      - Added declarative tab-scroll state in `fret-docking`, keyed by window and tabs node, and
        wired it into tab chrome/detail paint inputs, tab close hit-testing, overflow-menu opening,
        and tab-insert preview painting.
      - Reused the retained adapter's tab-strip scroll formula:
        `next_scroll = (scroll - (delta.x + delta.y)).clamp(0, max_scroll)`.
      - Locked the public declarative tab-strip wheel path with a test that scrolls the tab strip
        and then closes the tab made hit-testable by the new scroll offset.
    - 2026-05-19 twenty-second slice completed:
      - Moved tab hover, tab overflow button hover, and overflow menu row hover state onto the
        public declarative dock-space host.
      - Added declarative tab-hover state in `fret-docking` and kept hover policy out of
        `fret-ui`.
      - Refreshed transient tab interaction paint state from the latest docking service state at
        paint time so hover/menu visuals do not lag behind an older layout/prepaint frame output.
      - Locked the public declarative hover paths with tests for ordinary tab hover, overflow
        button hover, and overflow menu row hover.
    - 2026-05-19 twenty-third slice completed:
      - Moved the narrow panel-tab drag activation path onto the public declarative dock-space
        host.
      - Added declarative pending dock-drag state in `fret-docking`, reusing the retained runtime
        drag startup helper for `DRAG_KIND_DOCK_PANEL`.
      - Preserved retained semantics for tab-local grab offset, configured tab drag threshold,
        dock-preview inversion policy, pointer capture release on runtime drag start, and
        `DockingPolicy::allow_panel_drag`.
      - Locked the public declarative tab drag path with tests for activation, threshold gating,
        and panel drag policy gating.
    - 2026-05-19 twenty-fourth slice completed:
      - Moved tabs-group drag activation from empty tab-bar space onto the public declarative
        dock-space host.
      - Added declarative pending tabs-drag state in `fret-docking`, reusing the retained runtime
        drag startup helper for `DRAG_KIND_DOCK_TABS`.
      - Preserved retained semantics for tab-bar-local grab offset, configured tab drag threshold,
        dock-preview inversion policy, pointer capture release on runtime drag start, and
        `DockingPolicy::allow_tabs_group_drag`.
      - Locked the public declarative tabs-group drag path with tests for activation and tabs-group
        drag policy gating.
    - 2026-05-19 twenty-fifth slice completed:
      - Moved the floating title-bar drag dock-preview and center merge-on-release path onto the
        public declarative dock-space host.
      - Added declarative floating-drag activation state in `fret-docking`, latching
        dock-preview inversion policy at threshold activation while keeping policy/state out of
        `fret-ui`.
      - The declarative host now resolves `DockManager::hover` while an activated floating
        title-bar drag moves over the root dock layout, and emits `DockOp::MergeFloatingInto` on
        center drop release.
      - Locked the public declarative floating title-bar merge path with a test that proves hover
        resolution and release-time merge without creating retained `DockSpace`.
    - 2026-05-19 twenty-sixth slice completed:
      - Moved left-button viewport pointer capture onto the public declarative dock-space host.
      - Added declarative viewport capture state in `fret-docking`, keeping viewport interaction
        policy/state out of `fret-ui`.
      - The declarative host now forwards `ViewportInputKind::PointerDown`, clamped captured
        `PointerMove`, `PointerUp`, and `PointerCancel` effects through the shared viewport helper
        path, and requests/releases pointer capture on the managed-surface host node.
      - Locked the public declarative viewport capture path with tests for captured moves outside
        the draw rect and pointer cancel release without creating retained `DockSpace`.
    - 2026-05-19 twenty-seventh slice completed:
      - Moved floating close/title-bar hover visual state onto the public declarative dock-space
        host.
      - Added declarative floating hover state in `fret-docking`, keeping hover policy/state out of
        `fret-ui`.
      - The declarative host now updates floating hover state from `PointerMove` hit-tests, applies
        that state at paint time so visuals use the latest event state, and preserves retained
        cursor hints for floating close/title-bar hover.
      - Locked the public declarative floating hover path with a test for title-bar hover
        background and close hover affordance painting without creating retained `DockSpace`.
    - 2026-05-19 twenty-eighth slice completed:
      - Moved the stale-hover cleanup part of raw `InternalDrag` arbitration onto the public
        declarative dock-space host.
      - The declarative host now clears `DockManager::hover` on
        `InternalDragKind::{Drop, Leave, Cancel}` and requests redraw only when stale hover state
        was present.
      - Preserved the retained robustness behavior where `Drop` can arrive after the runtime drag
        session has already been cleared by the runner or driver.
      - Locked the public declarative cleanup path with an event test that dispatches
        `InternalDragKind::Drop` without creating retained `DockSpace`.
    - 2026-05-19 twenty-ninth slice completed:
      - Moved drop-target resolution out of the retained `DockSpace::event` local function set and
        into docking-private `dock/drop_resolve.rs`.
      - Retained `DockSpace` now uses the shared resolver, preserving existing retained drag/drop
        behavior while reducing retained adapter coupling.
      - The public declarative dock-space host now resolves `InternalDragKind::{Enter, Over}` hover
        targets through the same shared resolver and updates `DockManager::hover`.
      - Added a mechanism-only `ManagedSurfaceEventCx::pointer_position_window(...)` helper so the
        declarative docking host can use window-local pointer positions without retained `EventCx`.
      - Locked the public declarative `Over` path with a test that resolves the root split
        outer-left hint rect without creating retained `DockSpace`.
    - 2026-05-19 thirtieth slice completed:
      - Moved drop-intent resolution/application out of the retained `DockSpace::event` local
        function set and into docking-private `dock/drop_resolve.rs`.
      - Retained `DockSpace` now uses the shared drop-intent helpers, preserving existing retained
        drop behavior while reducing retained adapter coupling.
      - The public declarative dock-space host now handles `InternalDragKind::Drop` for active
        dock-panel and dock-tabs drags: it resolves the drop target through the shared resolver,
        applies the shared `DockDropIntent` into `Effect::Dock(...)`, clears hover, invalidates
        layout when an op is emitted, and ends the active dock drag session.
      - Locked the public declarative `Drop` path with a test that emits `DockOp::MovePanel`,
        applies it to split a tabs node, and verifies the drag session is cancelled without
        creating retained `DockSpace`.
    - 2026-05-19 thirty-first slice completed:
      - Moved tab-bar drag auto-scroll for active dock drags onto the public declarative dock-space
        host.
      - Added a declarative interaction-service cache for tab widths measured during managed-surface
        paint, so event-side hit testing, drop target resolution, and auto-scroll use the same
        measured tab geometry as retained `DockSpace` after paint.
      - Preserved a fallback approximate-width path for the first frame before paint measurement is
        available.
      - Locked the public declarative auto-scroll path with a test that drives repeated
        `InternalDragKind::Over` events at the right tab-bar edge without creating retained
        `DockSpace`.
    - 2026-05-19 thirty-second slice completed:
      - Moved stable out-of-bounds tear-off debounce mutation onto the public declarative
        dock-space host.
      - Added declarative handling for `PlatformCapabilities.ui.window_tear_off`,
        `DockSpaceElementOptions::allow_multi_window_tear_off`, policy-gated tear-off checks,
        `tear_off_oob_start_frame`, and duplicate request suppression.
      - Locked the public declarative tear-off path with a test that requires the second stable
        OOB frame to emit `DockOp::RequestTearOffPanel` without creating retained `DockSpace`.
    - Result for `RBX-M1-075`:
      - The active-tab close, overflow-menu close/activation, overflow-menu wheel scroll,
        tab-strip wheel scroll, tab hover, panel-tab drag activation, tabs-group drag activation,
        viewport input capture, floating close/title-bar hover, stale internal-drag hover cleanup,
        internal-drag over hover resolution, internal-drag drop intent/end-drag, tab-bar drag
        auto-scroll, tear-off debounce mutation, and tab paint paths now exist on the declarative
        host.
      - App/demo call sites can now switch away from `create_dock_space_node*` during
        `RBX-M1-080`.
- [x] RBX-M1-080 Remove `fret-ui/unstable-retained-bridge` from `fret-docking`.
  - Scope:
    - `ecosystem/fret-docking/Cargo.toml`
    - remaining `fret_ui::retained_bridge` imports in `ecosystem/fret-docking/src/`
    - `tools/check_layering.py` retained-bridge allowlist.
  - Goal:
    - Delete or quarantine the retained docking adapter and remove `fret-docking` from the
      retained-bridge allowlist.
  - Validation:
    - `cargo check -p fret-docking`
    - `cargo nextest run -p fret-docking`
    - `python3 tools/check_layering.py`
  - Result:
    - Deleted the retained `DockSpace` adapter, retained panel registry, retained prelude, and
      retained-only docking tests after mapping their covered behaviors to public declarative or
      mechanism-level tests.
    - Removed `fret-ui/unstable-retained-bridge` from `ecosystem/fret-docking/Cargo.toml`.
    - Removed `fret-docking` from `tools/check_layering.py`'s retained-bridge allowlist.
    - Added public-surface policy coverage proving declarative docking entry points are exported
      and retained docking entry points are no longer public.
    - Evidence:
      `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-19---rbx-m1-080-retained-capability-parity-audit-and-docking-bridge-removal`.
- [x] RBX-M1-085 Move first-party docking demos and cookbook off retained docking entry points.
  - Scope:
    - `apps/fret-examples/src/docking_demo.rs`
    - `apps/fret-examples/src/container_queries_docking_demo.rs`
    - `apps/fret-examples/src/docking_arbitration_demo.rs`
    - `apps/fret-examples/src/imui_editor_proof_demo.rs`
    - `apps/fret-cookbook/examples/docking_basics.rs`
    - `ecosystem/fret-docking/tests/public_surface_policy.rs`
  - Goal:
    - Keep public examples and cookbook on `DockPanelElementRegistry` +
      `dock_space_element_from_registry(...)` / `dock_space_declarative_with(...)` after the
      retained public docking entry points were removed.
    - Preserve demo diagnostics harness nodes where scripted tests still need stable external
      anchors, without reintroducing `fret-docking` retained APIs.
  - Result:
    - First-party docking examples compile against the declarative dock-space host.
    - Cookbook docking now teaches the declarative registry/host path.
    - Added policy coverage preventing first-party docking examples from reintroducing the deleted
      retained public entry points.
  - Validation:
    - `cargo check -p fret-demo --bin docking_demo`
    - `cargo check -p fret-demo --bin container_queries_docking_demo`
    - `cargo check -p fret-demo --bin docking_arbitration_demo`
    - `cargo check -p fret-demo --bin imui_editor_proof_demo`
    - `cargo check -p fret-cookbook --features cookbook-docking --example docking_basics`
    - `cargo nextest run -p fret-docking public_docking_surface_prefers_declarative_entry_points retained_docking_entry_points_are_not_public first_party_docking_examples_use_declarative_entry_points`
- [ ] Add/upgrade `fretboard-dev diag` scripts to lock in docking drag + tear-off correctness.

### M2 — Node graph migration

- [x] RBX-M2-010 Narrow `fret-node` retained bridge entry to the canvas compatibility island.
  - Scope:
    - `ecosystem/fret-node/Cargo.toml`
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Remove the redundant `compat-retained-bridge` feature alias so consumers cannot opt into
      `fret-ui/unstable-retained-bridge` without naming the concrete retained canvas compatibility
      surface.
  - Result:
    - `compat-retained-canvas` remains the only `fret-node` feature that enables
      `fret-ui/unstable-retained-bridge`.
    - The crate surface-policy test now rejects reintroducing `compat-retained-bridge` and locks the
      direct `compat-retained-canvas = ["fret-ui", "fret-ui/unstable-retained-bridge"]` edge.
  - Validation:
    - `cargo fmt --check`
    - `cargo nextest run -p fret-node retained_compatibility_surface_stays_declarative_only`
    - `cargo check -p fret-node --no-default-features --features headless`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
- [x] RBX-M2-020 Migrate first-party gallery node graph pages off retained canvas.
  - Scope:
    - `apps/fret-ui-gallery/Cargo.toml`
    - `apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs`
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Remove UI Gallery's `fret-node/compat-retained-canvas` dependency by switching gallery node
      graph examples to `NodeGraphSurfaceBinding` plus declarative `node_graph_surface(...)`.
    - Preserve workflow zoom/fit/reset controls by replacing the retained `BoundsRecorder` with a
      declarative `LayoutQueryRegion` bounds query.
  - Result:
    - UI Gallery's `fret-node` dependency no longer enables `compat-retained-canvas`.
    - `node_graph_cull_torture` and `workflow_node_graph_demo` no longer use
      `RetainedSubtreeProps`, `retained_bridge`, `NodeGraphCanvas::new`, or
      `NodeGraphEditor::new`.
    - Added `fret-node` surface-policy coverage to keep first-party gallery node graph pages off
      retained canvas.
  - Validation:
    - `cargo fmt --check`
    - `cargo check -p fret-ui-gallery --features gallery-dev`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node workflow_gallery_surface_stays_binding_first_for_viewport_controls first_party_gallery_node_graph_pages_stay_off_retained_canvas retained_compatibility_surface_stays_declarative_only`
    - `cargo nextest run -p fret-node`
    - `rg -n "RetainedSubtreeProps|retained_bridge|NodeGraphCanvas::new|NodeGraphEditor::new|create_node_retained|retained_subtree|compat-retained-canvas" apps/fret-ui-gallery/Cargo.toml apps/fret-ui-gallery/src/ui/previews/pages/torture/node_graph_cull_torture.rs apps/fret-ui-gallery/src/ui/snippets/ai/workflow_node_graph_demo.rs`
    - `cargo tree -p fret-ui-gallery --features gallery-dev -e features -i fret-node | rg -n "compat-retained-canvas|fret-node feature|fret-ui-gallery|fret-node v"`
    - `cargo tree -p fret-ui-gallery --features gallery-dev -e features -i fret-ui | tail -60`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-030 Remove first-party legacy retained node graph demo entry points.
  - Scope:
    - `apps/fret-demo/Cargo.toml`
    - `apps/fret-demo/src/bin/*node_graph*_demo.rs` legacy retained bins
    - `apps/fret-examples/Cargo.toml`
    - `apps/fret-examples/src/*node_graph*_demo.rs` legacy retained modules
    - `apps/fret-examples/src/lib.rs`
    - `apps/fretboard/src/dev/native.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - node graph docs/source-policy tools that referenced the legacy examples
  - Goal:
    - Remove first-party `node-graph-demos-legacy` entry points instead of keeping retained canvas
      examples alive after the declarative `node_graph_demo` path exists.
  - Result:
    - First-party node graph demo feature surface is declarative-only.
    - `fretboard dev native --bin node_graph_demo` still maps to `node-graph-demos`.
    - `fret-node` policy tests now reject legacy demo feature names, modules, bins, and retained
      canvas feature usage in first-party app/demo sources.
  - Validation:
    - `cargo check -p fret-demo --features node-graph-demos --bin node_graph_demo`
    - `cargo check -p fret-examples --features node-graph-demos`
    - `cargo nextest run -p fret-node first_party_node_graph_demos_stay_declarative_only retained_compatibility_surface_stays_declarative_only first_party_gallery_node_graph_pages_stay_off_retained_canvas`
    - `rg -n "node-graph-demos-legacy|fret-node/compat-retained-canvas|node_graph_legacy_demo|node_graph_domain_demo|imui_node_graph_demo|node_graph_tuning_overlay" apps crates ecosystem tools docs --glob '!docs/workstreams/**' --glob '!docs/audits/**' --glob '!target/**'`
- [x] RBX-M2-040 Remove the node graph declarative retained-subtree compatibility entry point.
  - Scope:
    - `ecosystem/fret-node/src/ui/declarative/compat_retained.rs`
    - `ecosystem/fret-node/src/ui/declarative/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Delete the public declarative retained-subtree shim
      (`node_graph_surface_compat_retained(...)` / `NodeGraphSurfaceCompatRetainedProps`) so
      `fret-node`'s declarative authoring surface cannot be backed by `RetainedSubtreeProps`.
    - Keep the lower-level `compat-retained-canvas` feature compiling for the remaining retained
      canvas/editor implementation island until its behavior has been migrated or quarantined.
  - Result:
    - `fret-node::ui::declarative` exports only `NodeGraphSurfaceBinding`,
      `node_graph_surface(...)`, and related declarative paint-only configuration.
    - `fret-node::ui` no longer re-exports `node_graph_surface_compat_retained(...)` or
      `NodeGraphSurfaceCompatRetainedProps`.
    - The retained canvas/editor stack remains behind `compat-retained-canvas` as a private
      implementation island for follow-up M2 work.
    - Surface-policy coverage now rejects declarative retained-subtree compatibility symbols.
  - Validation:
    - `cargo fmt --check`
    - `cargo nextest run -p fret-node retained_compatibility_surface_stays_declarative_only`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `rg -n "node_graph_surface_compat_retained|NodeGraphSurfaceCompatRetainedProps|compat_retained|RetainedSubtreeProps" ecosystem/fret-node/src ecosystem/fret-node/Cargo.toml --glob '!target/**'`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-050 Quarantine retained node graph widget surface behind a crate-private compat island.
  - Scope:
    - `ecosystem/fret-node/src/ui/mod.rs`
    - retained canvas/editor/overlay/portal modules under `ecosystem/fret-node/src/ui/`
    - retained canvas conformance test imports
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Remove the public root `fret-node::ui` exports for retained node graph widgets, editors,
      overlays, panels, portals, and retained canvas middleware.
    - Keep the retained canvas/editor implementation available only as private compatibility
      plumbing for the explicit `compat-retained-canvas` feature and its conformance tests.
    - Prove no capability was deleted by running both the declarative default test set and the full
      `compat-retained-canvas` retained behavior matrix.
  - Result:
    - `canvas`, `a11y`, `diag_anchors`, `editor`, `editors`, `overlays`, `panel`, and `portal`
      are crate-private modules instead of public `fret-node::ui` modules.
    - Root exports such as `NodeGraphCanvas`, `NodeGraphCanvasWith`, `NodeGraphEditor`,
      retained overlays, retained panels, and retained portal helpers are no longer public API.
    - Retained canvas conformance tests use crate-private/test-only module access, so the
      compatibility island can still guard behavior while public authoring stays declarative-first.
    - Surface-policy coverage now rejects reintroducing the retained widget public exports.
  - Validation:
    - `cargo fmt --check`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `rg -n "fret_node::ui::(NodeGraphCanvas|NodeGraphCanvasWith|NodeGraphEditor|NodeGraphPanel|NodeGraphPortalHost|NodeGraphOverlayHost)|use fret_node::ui::\\{[^\\n]*(NodeGraphCanvas|NodeGraphCanvasWith|NodeGraphEditor|NodeGraphPanel|NodeGraphPortalHost|NodeGraphOverlayHost)" apps crates ecosystem tools docs --glob '!target/**' --glob '!docs/workstreams/**'`
    - `rg -n "pub use (canvas|editor|editors|overlays|panel|portal)::|pub mod (canvas|a11y|diag_anchors|editor|editors|overlays|panel|portal);|NodeGraphSurfaceCompatRetainedProps|node_graph_surface_compat_retained|RetainedSubtreeProps" ecosystem/fret-node/src/ui ecosystem/fret-node/src/lib.rs ecosystem/fret-node/Cargo.toml --glob '!target/**'`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-060 Move node graph overlay/panel policy tests onto the default declarative UI gate.
  - Scope:
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/screen_space_placement.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
    - `ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard_policy.rs`
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Compile overlay/panel/screen-space pure policy and layout modules under the default
      declarative `fret-ui` feature, without enabling `compat-retained-canvas` or
      `fret-ui/unstable-retained-bridge`.
    - Keep retained overlay widget/paint modules gated behind `compat-retained-canvas` until their
      host/paint behavior is replaced by declarative composition.
    - Expand the default `fret-node` test gate so overlay/panel layout, keyboard, pointer,
      minimap, toolbar, blackboard, rename, and screen-space placement policy is protected outside
      the retained compatibility island.
  - Result:
    - `overlays` and `screen_space_placement` now compile in the default `fret-ui` path.
    - Retained overlay widget modules (`blackboard`, `controls`, `minimap`, `toolbars`) and retained
      paint helpers remain behind `compat-retained-canvas`.
    - Default `cargo nextest run -p fret-node` coverage increased from 269 to 319 tests, adding 50
      overlay/panel/screen-space policy tests to the declarative gate.
    - Surface-policy coverage now rejects moving overlay policy modules back behind
      `compat-retained-canvas`.
  - Validation:
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node overlay_policy_modules_compile_without_retained_canvas_compat`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-070 Move node graph portal editor chrome tests onto the default declarative UI gate.
  - Scope:
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/editors/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Compile the portal editor chrome helpers under the default declarative `fret-ui` feature,
      because they render declarative element chrome and do not require retained bridge lifecycle.
    - Keep retained portal text/number editor command handlers behind `compat-retained-canvas`
      until portal command submission and host lifecycle move to declarative composition.
  - Result:
    - `editors/chrome.rs` now compiles and tests in the default `fret-ui` path.
    - `editors/portal_text.rs` and `editors/portal_number.rs` remain gated behind
      `compat-retained-canvas`.
    - Default `cargo nextest run -p fret-node` coverage increased from 319 to 324 tests, adding
      editor chrome policy coverage to the declarative gate.
    - Surface-policy coverage now rejects moving editor chrome back behind
      `compat-retained-canvas`.
  - Validation:
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node editor_chrome_compiles_without_retained_canvas_compat ui::editors::chrome`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
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
