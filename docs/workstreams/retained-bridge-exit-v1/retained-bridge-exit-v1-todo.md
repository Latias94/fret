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
- [x] RBX-M2-080 Record the node graph retained capability ledger and lock retained source usage.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/RBX_M2_080_NODE_RETAINED_CAPABILITY_LEDGER_2026-05-19.md`
    - workstream evidence/handoff docs
  - Goal:
    - Treat the remaining `compat-retained-canvas` island as a deletion oracle rather than a
      public authoring path.
    - Record the remaining retained capability families and the declarative/default tests required
      before deleting each family.
    - Add a source-policy gate proving code-level retained bridge usage cannot spread outside the
      explicit migration ledger.
  - Result:
    - Added `surface_policy_tests::retained_bridge_source_usage_stays_on_the_migration_ledger`.
    - Recorded remaining retained node graph capability families: canvas paint/cache, pan/zoom,
      interactions, overlays, portal editor commands, a11y/diagnostics anchors, and middleware.
    - Confirmed deletion should proceed by shrinking the ledger as default declarative tests replace
      retained conformance coverage, not by deleting retained oracle code without replacement.
  - Validation:
    - `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `rg -l "use fret_ui::retained_bridge|use fret_ui::\\{UiHost, retained_bridge|fret_ui::retained_bridge::|RetainedSubtreeProps|UiTreeRetainedExt" ecosystem/fret-node/src/ui -g '*.rs' | sort`
- [x] RBX-M2-085 Move node graph portal command protocol onto the default declarative UI gate.
  - Scope:
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/portal.rs`
    - `ecosystem/fret-node/src/ui/portal_commands.rs`
  - Goal:
    - Extract the submit/cancel/step command IDs, parser, step-mode enum, command enum, and command
      outcome type from the retained `NodeGraphPortalHost` module into a default-gated protocol
      module.
    - Keep the retained portal host and retained portal text/number command handlers gated for now,
      while making the command protocol available to future declarative portal command handling.
  - Result:
    - Added `ui/portal_commands.rs` under the default `fret-ui` path.
    - `ui/portal.rs` now re-exports the protocol for retained compat consumers instead of owning
      it.
    - Added default-gated roundtrip and malformed-command tests for the portal text command
      protocol.
  - Validation:
    - `cargo nextest run -p fret-node portal_text_command_protocol`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-090 Move portal editor command policy onto the default declarative UI gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/editors/mod.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_command_policy.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_text.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_number.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move portal text/number submit/cancel/step decision policy out of retained `CommandCx`
      handlers and into a default-gated service module.
    - Keep retained portal text/number handlers as compatibility adapters for session/model I/O
      until declarative portal editor hosting can replace them.
    - Add default declarative tests for text and number command policy behavior before deleting any
      retained editor command code.
  - Result:
    - Added `ui/editors/portal_command_policy.rs` under the default `fret-ui` path.
    - Moved `PortalTextEditSpec`, `PortalTextEditSubmit`, `PortalNumberEditSpec`, and
      `PortalNumberEditSubmit` into the default policy module.
    - Added default tests covering text cancel/submit/step planning and number cancel/submit/parse
      error/step planning without retained `CommandCx`.
    - `portal_text.rs` and `portal_number.rs` now consume default policy plans and remain retained
      session/model I/O adapters behind `compat-retained-canvas`.
  - Validation:
    - `cargo nextest run -p fret-node portal_command_policy editor_chrome_compiles_without_retained_canvas_compat`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas portal`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/editors/portal_command_policy.rs 2>&1); test -z "$out"`
- [x] RBX-M2-095 Move portal editor command session application onto the default declarative UI gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/editors/mod.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_command_session.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_text.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_number.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move portal text/number session command application out of retained `CommandCx` handlers and
      into a default-gated adapter module.
    - Keep retained portal text/number command handlers as thin model I/O adapters for the
      remaining retained portal host.
    - Add default tests proving cancel, submit, parse/error, normalization, and commit outcomes can
      be applied without retained `CommandCx`.
  - Result:
    - Added `ui/editors/portal_command_session.rs` under the default `fret-ui` path.
    - Added `PortalTextCommandSession` and `PortalNumberCommandSession` traits plus default command
      application functions that consume `portal_command_policy`.
    - Converted retained `PortalTextEditHandler` and `PortalNumberEditHandler` to provide retained
      model/session I/O adapters around the default session application functions.
    - Removed redundant retained handler-owned submit/cancel/step application code.
  - Validation:
    - `cargo nextest run -p fret-node without_retained_command_cx editor_chrome_compiles_without_retained_canvas_compat`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas portal`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/editors/portal_command_session.rs 2>&1); test -z "$out"`
- [x] RBX-M2-100 Add default declarative controls overlay composition.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move controls overlay composition coverage onto the default declarative `fret-ui` gate before
      deleting retained overlay widget code.
    - Preserve the retained controls overlay as the compatibility oracle for pointer/keyboard/focus
      behavior until the full overlay interaction path has declarative coverage.
  - Result:
    - Added `NodeGraphControlsOverlayElementProps` and
      `node_graph_controls_overlay_element(...)` under the default overlay module gate.
    - The declarative controls tree now builds a panel container, column, six pressable buttons,
      stable `node_graph.controls.*` test IDs, button labels, button a11y labels, enabled/disabled
      command binding state, and activation command dispatch hooks.
    - Added default tests for panel sizing, button roster/order, labels, a11y/test IDs, connection
      mode labels, command activation dispatch, and disabled command suppression without
      constructing the retained widget.
    - Added a source-policy assertion that the declarative controls composition does not take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node controls_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_declarative.rs 2>&1); test -z "$out"`
- [x] RBX-M2-105 Add default declarative blackboard overlay composition.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move blackboard overlay composition coverage onto the default declarative `fret-ui` gate before
      deleting retained overlay widget code.
    - Preserve the retained blackboard overlay as the compatibility oracle for transaction
      submission, rename sessions, keyboard/focus navigation, pointer hover/press state, and
      retained paint behavior until those families have default declarative coverage.
  - Result:
    - Added `NodeGraphBlackboardOverlayElementProps` and
      `node_graph_blackboard_overlay_element(...)` under the default overlay module gate.
    - The declarative blackboard tree now builds the panel container, header, add-symbol action,
      sorted symbol rows, insert/rename/delete pressables, stable `node_graph.blackboard.*` test
      IDs, visible labels, button a11y labels, and a mechanism-only action hook.
    - Added default tests for panel sizing, row order, root semantics, symbol action a11y/test IDs,
      and pointer activation through the declarative action hook without constructing the retained
      widget.
    - Added a source-policy assertion that the declarative blackboard composition does not take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node blackboard_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs 2>&1); test -z "$out"`
- [x] RBX-M2-106 Add default declarative minimap overlay composition.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move minimap overlay composition and paint-plan coverage onto the default declarative
      `fret-ui` gate before deleting retained minimap widget code.
    - Preserve the retained minimap overlay as the compatibility oracle for keyboard pan/zoom/focus,
      pointer drag panning, focus/capture propagation, retained hit testing, and store/controller
      viewport updates until those families have default declarative coverage.
  - Result:
    - Added `NodeGraphMiniMapOverlayElementProps`, `NodeGraphMiniMapSnapshot`, and
      `node_graph_minimap_overlay_element(...)` under the default overlay module gate.
    - The declarative minimap tree now builds a root panel container, declarative `Canvas`, stable
      `node_graph.minimap` semantics/test ID, and a paint plan for the minimap panel, projected node
      markers, and viewport marker without constructing the retained minimap widget.
    - Added default tests for panel sizing, root semantics, declarative canvas composition, and
      panel/node/viewport paint ops without constructing the retained widget.
    - Added a source-policy assertion that the declarative minimap composition does not take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node minimap_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs 2>&1); test -z "$out"`
- [x] RBX-M2-107 Add default declarative toolbar overlay composition.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move node/edge toolbar placement and composition coverage onto the default declarative
      `fret-ui` gate before deleting retained toolbar widget code.
    - Preserve the retained toolbar widgets as the compatibility oracle for child measurement,
      retained child-root layout/paint, hit testing, and model/internals-driven target resolution
      until those families have default declarative coverage.
  - Result:
    - Added `NodeGraphNodeToolbarElementProps`, `NodeGraphEdgeToolbarElementProps`, target structs,
      `node_graph_node_toolbar_element(...)`, and `node_graph_edge_toolbar_element(...)` under the
      default overlay module gate.
    - The declarative toolbar path now plans retained-compatible node-rect and edge-center
      placement, honors `WhenSelected`/`Always` visibility, emits an absolute declarative toolbar
      container, stamps toolbar semantics/test IDs, and preserves caller-supplied declarative
      children without constructing retained toolbar widgets.
    - Added default tests for node toolbar placement/visibility, edge toolbar placement/visibility,
      node toolbar declarative composition, and edge toolbar declarative composition.
    - Added a source-policy assertion that the declarative toolbar composition does not take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node toolbars_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs 2>&1); test -z "$out"`
- [x] RBX-M2-108 Add default declarative rename overlay composition.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move inline rename overlay composition and text-input submit/cancel command wiring onto the
      default declarative `fret-ui` gate before deleting retained rename host code.
    - Preserve the retained rename host as the compatibility oracle for seed-text ownership,
      focus-loss close, focus request/restore, keyboard submit/cancel event routing, graph/edit
      queue transaction submission, blackboard rename handoff, and retained paint/hit testing until
      those families have default declarative coverage.
  - Result:
    - Added `NodeGraphRenameOverlayElementProps` and `node_graph_rename_overlay_element(...)`
      under the default overlay module gate, backed by default rename command protocol helpers.
    - The declarative rename path now consumes shared `plan_rename_host_layout(...)`, emits an
      absolute panel container, builds a declarative `TextInput`, preserves caller-owned bound text
      models, stamps root/input semantics and stable test IDs, and wires submit/cancel commands
      without constructing the retained `NodeGraphOverlayHost` widget.
    - Added default tests for hidden/no-session behavior, group and symbol text input composition,
      shared layout-policy consumption, caller-owned text-model preservation, and submit/cancel
      command protocol roundtrips.
    - Extended the source-policy assertion so declarative rename composition cannot take a retained
      bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node rename_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node rename_declarative minimap_declarative toolbars_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_declarative.rs 2>&1); test -z "$out"`
- [x] RBX-M2-109 Move rename command/session application onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_command.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
    - `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move rename submit/cancel command parsing, keyboard submit/cancel decision, and active
      session application onto the default declarative `fret-ui` gate.
    - Leave retained `NodeGraphOverlayHost` as an I/O adapter that reads models, submits any
      produced transaction through the existing controller/edit-queue transport, and restores focus.
  - Result:
    - Added default `rename_command.rs` with `RenameTextCommand`, `RenameCommandOutcome`,
      `RenameHostKeyDecision`, command ID helpers, command parsing, text-command application, and
      keyboard-decision application.
    - Moved rename submit/cancel command protocol out of `rename_declarative.rs` so declarative
      composition only wires commands onto `TextInput` while default command/session policy owns
      command semantics.
    - Updated retained `rename_host_event.rs` to delegate active-session close/commit decisions to
      default rename command/session policy; it now only performs retained model I/O and retained
      transaction submission transport.
    - Added default tests for malformed command rejection, stale-session no-op behavior, active
      group/symbol submit/cancel application, and keyboard Enter/Escape/ignore application without
      retained `EventCx`.
    - Extended the source-policy assertion so default rename command/session policy cannot take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node rename_command rename_declarative rename_host_event overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas rename_command rename_host_event overlay_group_rename_conformance`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_command.rs 2>&1); test -z "$out"`
- [x] RBX-M2-110 Move rename lifecycle planning onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs`
    - `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move rename seed-text ownership, first-focus request, focus-loss close planning, and
      focus-restore planning out of retained `NodeGraphOverlayHost` layout code and into a
      default-gated policy module.
    - Keep the retained host as a compatibility I/O adapter that applies the default lifecycle
      plan to the retained tree until a declarative managed host owns the same side effects.
  - Result:
    - Added `rename_lifecycle.rs` with `RenameHostLifecyclePlan` and
      `plan_rename_host_lifecycle(...)` under the default overlay module gate.
    - Added default tests for group/symbol seed text, first-open focus request, no reseed/refocus
      for an already-open session, focus-loss close without stealing the new focus owner, and
      focus restoration when a hidden rename input still owns focus.
    - Updated retained `NodeGraphOverlayHost::layout` to consume the default lifecycle plan, so
      retained code now applies model/tree side effects around default policy decisions instead of
      owning those decisions.
    - Extended source-policy assertions so default rename lifecycle policy cannot take a retained
      bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node rename_lifecycle rename_host_event rename_command rename_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node --features compat-retained-canvas rename_lifecycle rename_host_event overlay_group_rename_conformance overlay_symbol_rename_conformance overlay_blackboard_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/rename_lifecycle.rs 2>&1); test -z "$out"`
- [x] RBX-M2-111 Move minimap keyboard/pointer interaction planning onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/minimap.rs`
    - `ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move minimap keyboard pan/zoom/focus decisions plus pointer down/up focus/capture/repaint
      planning out of the retained minimap widget event handler and into a default-gated policy
      module.
    - Keep the retained minimap widget as a compatibility I/O adapter that reads store state,
      applies viewport updates, and performs retained event side effects until a declarative
      minimap host owns the same behavior.
  - Result:
    - Added `minimap_interaction_policy.rs` with `MiniMapKeyboardInteractionPlan`,
      `MiniMapPointerDownInteractionPlan`, `MiniMapPointerUpInteractionPlan`, and default
      planning functions for keyboard pan/zoom/focus plus pointer drag start/end side effects.
    - Updated retained `NodeGraphMiniMapOverlay::event` to consume the default interaction plans,
      leaving retained code to perform model/view-state I/O, focus/capture calls, and repaint
      completion.
    - Added default tests for keyboard pan/zoom/focus/ignore behavior, pointer down
      focus/capture/stop-propagation/repaint behavior, non-left/outside pointer rejection, and
      pointer-up capture release/finish gating.
    - Extended source-policy assertions so default minimap interaction policy cannot take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node minimap_interaction_policy minimap_drag_policy minimap_policy overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node --features compat-retained-canvas minimap_interaction_policy overlay_minimap_controls_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/minimap_interaction_policy.rs 2>&1); test -z "$out"`
- [x] RBX-M2-112 Move toolbar layout/hit-test planning onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars_layout.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move toolbar visible-target filtering, node/edge child rect planning, empty-size hiding, and
      child-bound hit-test decisions out of the retained toolbar widget and into a default-gated
      policy module.
    - Keep retained toolbar widgets as compatibility I/O adapters that resolve internals targets,
      measure retained children, apply `layout_in`, and paint retained child roots until a
      declarative toolbar host owns those side effects.
  - Result:
    - Added `toolbar_layout_policy.rs` with `ToolbarChildLayoutPlan`,
      `visible_toolbar_anchor(...)`, node/edge child layout planning, and child-bound hit-test
      policy.
    - Updated retained node/edge toolbar widgets to consume the default layout/hit-test policy;
      retained code now performs target/model I/O, retained child measurement, retained layout
      application, and retained child painting.
    - Updated declarative toolbar composition to reuse the same default layout policy instead of
      carrying a separate rect/visibility implementation.
    - Moved retained-only positioning math tests to default layout-policy tests and kept retained
      toolbar oracle tests green for pointer fallthrough and focus release.
    - Extended source-policy assertions so default toolbar layout policy cannot take a retained
      bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node toolbar_layout_policy toolbars_declarative toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node --features compat-retained-canvas toolbar_layout_policy toolbars_declarative overlay_toolbars_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/toolbar_layout_policy.rs 2>&1); test -z "$out"`
- [x] RBX-M2-113 Move controls overlay interaction planning onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls_interaction_policy.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move controls overlay keyboard select/activate/focus-canvas planning plus pointer
      hover/down/up focus/capture/repaint/activation planning out of the retained controls widget
      and into a default-gated policy module.
    - Keep the retained controls widget as a compatibility side-effect adapter that applies focus,
      cursor, pointer capture, repaint completion, and command dispatch until a declarative host
      owns those side effects.
  - Result:
    - Added `controls_interaction_policy.rs` with `ControlsInteractionState` and default planning
      functions for keyboard, hover, pointer-down, and pointer-up interactions.
    - Updated retained `NodeGraphControlsOverlay::event` to consume default interaction plans while
      retaining only side-effect application and command dispatch.
    - Added default tests for keyboard navigation/activation/focus/ignore, hover repaint
      transitions, pointer-down keyboard promotion/capture gating, and pointer-up activation
      gating.
    - Extended source-policy assertions so default controls interaction policy cannot take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node controls_interaction_policy controls_declarative controls_layout controls_policy panel_navigation_policy panel_pointer_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_interaction_policy.rs 2>&1); test -z "$out"`
- [x] RBX-M2-114 Move blackboard overlay interaction planning onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard_interaction_policy.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move blackboard keyboard select/activate/focus-canvas planning plus pointer
      hover/down/up focus/capture/repaint/activation planning out of the retained blackboard widget
      and into a default-gated policy module.
    - Keep the retained blackboard widget as a compatibility side-effect adapter that applies
      focus, cursor, pointer capture, repaint, and transaction/rename dispatch until a declarative
      host owns those side effects.
  - Result:
    - Added `blackboard_interaction_policy.rs` with `BlackboardInteractionState` and default
      planning functions for keyboard, hover, pointer-down, and pointer-up interactions.
    - Updated retained `NodeGraphBlackboardOverlay::event` to consume default interaction plans
      while retaining only side-effect application and action dispatch.
    - Added default tests for keyboard navigation/activation/focus/ignore, hover repaint
      transitions, pointer-down panel/action gating, and pointer-up capture/repaint/activation
      gating.
    - Extended source-policy assertions so default blackboard interaction policy cannot take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node blackboard_interaction_policy blackboard_declarative blackboard_layout blackboard_policy panel_navigation_policy panel_pointer_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat`
    - `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_interaction_policy blackboard_declarative overlay_blackboard_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_interaction_policy.rs 2>&1); test -z "$out"`
- [x] RBX-M2-115 Move blackboard retained paint decisions onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard_paint_plan.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard_paint.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move blackboard panel/button/label paint ordering, text constraints, active-action
      background selection, and missing-symbol label fallback into a default-gated paint plan.
    - Keep the retained blackboard paint module as a compatibility adapter that prepares text
      blobs and pushes `SceneOp`s into retained `PaintCx`.
  - Result:
    - Added `blackboard_paint_plan.rs` with `BlackboardPaintPlan`, panel/item plans,
      `BlackboardPaintState`, and default tests for panel/header/buttons/rows, active-state
      backgrounds, text constraints, and missing symbol fallback.
    - Updated retained `blackboard_paint.rs` to consume the default paint plan and retain only
      `PaintCx`/text-blob/scene-op side effects.
    - Extended source-policy assertions so default blackboard paint planning cannot take a retained
      bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node blackboard_paint_plan blackboard_layout blackboard_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat`
    - `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_paint_plan blackboard_declarative overlay_blackboard_conformance blackboard_paint`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/blackboard_paint_plan.rs 2>&1); test -z "$out"`
- [x] RBX-M2-116 Move controls retained paint decisions onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls_paint_plan.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move controls panel/button paint ordering, text constraints, connection-mode labels, pressed
      / hovered / keyboard-active background selection, and focus-gated keyboard highlight rules
      into a default-gated paint plan.
    - Keep retained `NodeGraphControlsOverlay::paint` as a compatibility adapter that reads
      models, prepares text blobs, and pushes `SceneOp`s into retained `PaintCx`.
  - Result:
    - Added `controls_paint_plan.rs` with `ControlsPaintPlan`, panel/button plans,
      `ControlsPaintState`, and default tests for panel draw decisions, text constraints,
      connection-mode labels, active/hover/keyboard/idle backgrounds, and pointer-active keyboard
      suppression.
    - Updated retained `NodeGraphControlsOverlay::paint` to consume the default paint plan and
      retain only `PaintCx`/text-blob/scene-op side effects.
    - Extended source-policy assertions so default controls paint planning cannot take a retained
      bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node controls_paint_plan controls_layout controls_policy panel_item_state overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_paint_plan controls_declarative overlay_minimap_controls_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_paint_plan.rs 2>&1); test -z "$out"`
- [x] RBX-M2-117 Move controls host hit-test and panel pointer-down side-effect planning onto the default overlay gate.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls_host_policy.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move controls panel hit-testing, retained pointer-down host side-effect planning, and
      declarative panel blank-area pointer-down handling into a default-gated host policy.
    - Preserve button pressable activation by making the declarative panel handler ignore
      descendant pressable hits.
    - Keep retained `NodeGraphControlsOverlay` as a compatibility adapter that applies the shared
      host plan through retained `EventCx` focus, propagation, capture, and repaint side effects.
  - Result:
    - Added `controls_host_policy.rs` with shared controls panel hit-test and pointer-down host
      plans for retained and declarative hosts.
    - Updated retained `NodeGraphControlsOverlay::hit_test` and pointer-down handling to consume
      the default host policy while preserving panel blocking, button capture, and repaint
      behavior.
    - Wrapped declarative controls composition in `PointerRegion` so blank panel pointer-downs
      focus and stop propagation without dispatching a controls command, while button descendants
      continue to activate through `Pressable`.
    - Added a default declarative integration test for blank panel pointer-down focus/no-command
      behavior and extended source-policy assertions so default controls host policy cannot take a
      retained bridge, retained subtree, or retained `Widget` dependency.
  - Validation:
    - `cargo nextest run -p fret-node controls_declarative_panel_blank_pointer_down_focuses_overlay_without_command controls_host_policy controls_interaction_policy controls_declarative overlay_policy_modules_compile_without_retained_canvas_compat controls_overlay_requires_explicit_editor_config_model`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `out=$(git diff --check --no-index /dev/null ecosystem/fret-node/src/ui/overlays/controls_host_policy.rs 2>&1); test -z "$out"`
- [x] RBX-M2-118 Prove controls declarative pointer-up/capture/command completion parity.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Prove the default declarative controls button path completes the retained controls button
      pointer-up behavior family without constructing the retained controls widget.
    - Cover pointer-down capture, no early command dispatch, pointer-up capture release, focus
      transfer to the activated button, command dispatch on in-bounds release, and capture
      completion without command dispatch on out-of-bounds release.
    - Treat this as a proof of the existing declarative `Pressable` mechanism, not a new controls
      policy extraction.
  - Result:
    - Added
      `controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch`.
    - The default declarative controls composition now directly proves the retained controls
      pointer-up/capture completion and command-dispatch behavior family through `Pressable`.
    - The remaining controls retained-widget deletion gap is now retained paint/oracle removal
      logistics and broader overlay integration, not pointer-up/capture command completion.
  - Validation:
    - `cargo nextest run -p fret-node controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_declarative_activation_dispatches_commands_and_honors_disabled_bindings controls_host_policy controls_interaction_policy`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-119 Prove toolbar declarative child measurement and child-root host placement parity.
  - Scope:
    - `crates/fret-ui/src/managed_surface.rs`
    - `crates/fret-ui/src/declarative/tests/managed_surface.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Add the narrow mechanism needed by declarative toolbar hosts to measure a child before
      choosing its final placement.
    - Prove node and edge toolbar declarative hosts can use Auto child measurement to compute the
      same child rects as the retained toolbar layout policy, then layout and paint that child
      through the declarative managed-surface path.
    - Keep retained toolbar widgets as the oracle for pointer fallthrough/interception, focus
      release, and model/internals-driven target resolution until those behavior families have
      dedicated default declarative coverage.
  - Result:
    - Added `ManagedSurfaceLayoutCx::measure_child(...)` and a `fret-ui` managed-surface mechanism
      test proving a host can measure a declarative child before final placement.
    - Added node and edge toolbar declarative managed-host entry points for Auto/Fixed child size
      planning, child layout, and child paint without constructing retained toolbar widgets.
    - Added default declarative toolbar tests proving Auto child measurement feeds retained
      placement policy for visible node/edge toolbars and hidden edge toolbars.
    - Did not delete retained toolbar widgets in this slice because retained pointer
      fallthrough/interception and focus-release behavior remain covered only by the retained
      oracle tests.
  - Validation:
    - `cargo nextest run -p fret-ui managed_surface`
    - `cargo nextest run -p fret-node node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat`
    - `cargo nextest run -p fret-node --features compat-retained-canvas node_toolbar_declarative_host_auto_measures_and_places_child_without_retained_widget edge_toolbar_declarative_host_auto_measures_and_hides_child_without_retained_widget toolbars_declarative toolbar_layout_policy overlay_toolbars_conformance`
- [x] RBX-M2-120 Prove toolbar model/internals-driven target resolution parity.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/toolbar_policy.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move node/edge toolbar target resolution from retained-widget-local logic to a default-gated
      policy helper.
    - Prove default declarative node/edge toolbar targets are resolved from the same
      `NodeGraphViewState` selected fallback/requested-id rules plus `NodeGraphInternalsStore`
      window geometry as the retained widgets.
    - Keep retained toolbar widgets as the compatibility oracle until a separate deletion slice
      removes the retained widget files/exports and reruns the default and compatibility gates.
  - Result:
    - Added `resolve_node_toolbar_window_target(...)` and `resolve_edge_toolbar_window_target(...)`
      in default-gated `toolbar_policy.rs`.
    - Retained node/edge toolbar widgets now consume those helpers instead of duplicating
      `view_state + internals.snapshot()` resolution locally.
    - Added declarative target wrappers and default tests covering selected fallback, requested
      selected/unselected targets, and missing internals geometry for node and edge toolbars.
    - Fresh default and compat retained toolbar gates also cover declarative pointer
      fallthrough/interception, focus release when hidden, child measurement/layout/paint, and the
      retained toolbar oracle after the helper extraction.
  - Validation:
    - `cargo nextest run -p fret-node node_toolbar_declarative_target_resolution_uses_view_state_and_internals edge_toolbar_declarative_target_resolution_uses_view_state_and_internals`
    - `cargo nextest run -p fret-node toolbar_policy node_toolbar_declarative_target_resolution_uses_view_state_and_internals edge_toolbar_declarative_target_resolution_uses_view_state_and_internals`
    - `cargo nextest run -p fret-node toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat`
    - `cargo nextest run -p fret-node --features compat-retained-canvas toolbars_declarative toolbar_layout_policy toolbar_policy overlay_toolbars_conformance`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `git diff --check`
- [x] RBX-M2-121 Delete retained toolbar widgets after default parity proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars.rs`
    - `ecosystem/fret-node/src/ui/overlays/toolbars_layout.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_toolbars_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete the retained node/edge toolbar widget files and their retained oracle test module now
      that default declarative coverage proves placement/composition, layout/hit-test policy, Auto
      child measurement and child-root layout/paint, pointer fallthrough/interception, focus release
      when hidden, and model/internals-driven target resolution.
    - Remove the retained toolbar exports and source-policy allowlist entries so toolbar retained
      code cannot re-enter the compatibility island accidentally.
  - Result:
    - Deleted `toolbars.rs`, `toolbars_layout.rs`, and
      `overlay_toolbars_conformance.rs`.
    - Removed retained toolbar test-only exports from `ui/mod.rs` and `overlays/mod.rs`.
    - Removed toolbar retained files from the explicit retained bridge source allowlist.
    - Kept default `toolbar_policy.rs`, `toolbar_layout_policy.rs`, and `toolbars_declarative.rs`
      as the canonical toolbar behavior surface.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo nextest run -p fret-node toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node --features compat-retained-canvas toolbars_declarative toolbar_layout_policy toolbar_policy overlay_policy_modules_compile_without_retained_canvas_compat retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `rg -n "\\bNodeGraphNodeToolbar\\b|\\bNodeGraphEdgeToolbar\\b|overlay_toolbars_conformance|toolbars_layout|mod toolbars;|src/ui/overlays/toolbars\\.rs|src/ui/overlays/toolbars_layout\\.rs" ecosystem/fret-node/src -g '*.rs'`
- [x] RBX-M2-122 Prove declarative controls activation restores focus to the node graph surface.
  - Scope:
    - `crates/fret-ui/src/action.rs`
    - `crates/fret-ui/src/elements/cx.rs`
    - `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
    - `crates/fret-ui/src/declarative/tests/interactions/pressable.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Add a narrow focus-capable `Pressable` activation hook without changing the existing
      `pressable_on_activate` contract.
    - Prove controls button pointer and keyboard activation can dispatch the bound command and then
      restore focus to the node graph surface/canvas target on the default declarative path.
    - Keep retained controls as the compatibility oracle until deletion logistics and remaining
      retained-only behavior families have default coverage.
  - Result:
    - Added `pressable_on_activate_focus(...)` / `pressable_add_on_activate_focus(...)` and matching
      `*_for` registration helpers backed by `UiFocusActionHost`.
    - Added `fret-ui` mechanism tests proving focus-capable activation hooks can override pointer
      default focus and restore focus after keyboard activation.
    - Added `focus_target` to the declarative controls overlay props and default controls tests for
      pointer and keyboard activation restoring focus to a surface target while still dispatching
      commands.
    - Retained controls were not deleted in this slice.
  - Validation:
    - `cargo nextest run -p fret-ui pressable_focus_activation_hook_can_restore_focus_after_pointer_activation pressable_focus_activation_hook_can_restore_focus_after_keyboard_activation pressable_on_activate_hook_runs_on_pointer_activation pressable_on_activate_hook_runs_on_keyboard_activation`
    - `cargo nextest run -p fret-node controls_declarative_button_activation_restores_focus_to_surface_target controls_declarative_keyboard_activation_restores_focus_to_surface_target controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_declarative_activation_dispatches_commands_and_honors_disabled_bindings controls_host_policy controls_interaction_policy`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative_button_activation_restores_focus_to_surface_target controls_declarative_keyboard_activation_restores_focus_to_surface_target controls_declarative_button_pointer_up_completes_capture_focus_and_command_dispatch controls_host_policy controls_interaction_policy controls_declarative overlay_minimap_controls_conformance`
    - `cargo fmt -p fret-ui -p fret-node`
- [x] RBX-M2-123 Prove declarative controls root keyboard semantics and Escape parity.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move retained controls root semantics intent onto the default declarative path:
      `node_graph.controls`, `Controls` panel role/label, focusability, and active button value.
    - Prove pointer-down and keyboard navigation update the root active semantics value without
      constructing the retained controls widget.
    - Prove root-level keyboard activation dispatches the selected command and restores focus to
      the node graph surface target.
    - Prove Escape restores focus to the node graph surface target without dispatching a command
      and clears the active semantics value back to the retained-compatible default.
  - Result:
    - Added a focusable declarative controls semantics root with stable `node_graph.controls`
      test ID and retained-compatible value fallback to the first controls button.
    - Added declarative controls root key handling backed by `ControlsInteractionState` and
      `plan_controls_keyboard_interaction(...)`.
    - Added button pointer-down state promotion so default declarative controls expose the same
      active semantics value after pointer targeting as the retained oracle.
    - Retained controls were not deleted in this slice; this removes the retained-only root
      keyboard/semantics/Escape gap needed before a deletion slice.
  - Validation:
    - `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-124 Prove declarative controls overlay integration parity before retained deletion.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/controls_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Backfill default declarative integration tests for the retained controls oracle behavior that
      still crossed the overlay/surface boundary before deleting the retained controls widget.
    - Prove pointer-down outside the controls panel falls through to the node graph surface.
    - Prove blank pointer-down inside the controls panel blocks surface input and focuses the
      controls root for keyboard follow-up.
    - Prove focus traversal can move from the node graph surface to the focusable controls root,
      and Escape returns focus to the surface without dispatching commands.
    - Keep retained controls as the compatibility oracle in this slice; deletion belongs in the
      next narrow cleanup task after default and compat evidence are recorded.
  - Result:
    - Added `render_controls_with_recording_surface(...)` as a declarative stack fixture with a
      focusable recording surface and the declarative controls overlay.
    - Added
      `controls_declarative_pointer_events_fall_through_outside_panel_to_surface`,
      `controls_declarative_blocks_surface_input_within_panel_even_off_button`, and
      `controls_declarative_focus_traversal_reaches_controls_from_surface`.
    - Confirmed the default declarative controls path now covers retained controls pointer
      fallthrough, panel interception, focus traversal into controls, and Escape focus return.
    - Retained `NodeGraphControlsOverlay` was not deleted in this slice; it is now ready for a
      separate controls-only deletion task that leaves retained minimap coverage intact.
  - Validation:
    - `cargo nextest run -p fret-node controls_declarative_pointer_events_fall_through_outside_panel_to_surface controls_declarative_blocks_surface_input_within_panel_even_off_button controls_declarative_focus_traversal_reaches_controls_from_surface`
    - `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-125 Delete the retained controls widget after default integration parity proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/overlays/controls.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete `NodeGraphControlsOverlay` and retained controls test-only exports now that default
      declarative coverage proves composition, command dispatch, pointer capture/up completion,
      panel hit-test/blocking, pointer fallthrough, focus restore, root semantics, keyboard
      activation, Escape, and focus traversal behavior.
    - Split or trim the combined retained `overlay_minimap_controls_conformance` oracle so retained
      minimap tests remain under `compat-retained-canvas` while controls retained tests disappear.
    - Remove `src/ui/overlays/controls.rs` from the retained bridge source migration ledger.
  - Result:
    - Deleted `ecosystem/fret-node/src/ui/overlays/controls.rs`.
    - Removed the retained controls module/test-only export from `ui/overlays/mod.rs` and the
      crate-private test re-export from `ui/mod.rs`.
    - Trimmed `overlay_minimap_controls_conformance.rs` to minimap-only retained oracle coverage;
      retained minimap pointer fallthrough, drag, keyboard pan/zoom, controller binding, store/view
      sync, focus behavior, and semantics test ID coverage remain under `compat-retained-canvas`.
    - Removed `src/ui/overlays/controls.rs` from
      `retained_bridge_source_usage_stays_on_the_migration_ledger` and renamed the default source
      policy test to `default_overlay_policy_surfaces_stay_off_retained_bridge`.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `rg -n "\\bNodeGraphControlsOverlay\\b|src/ui/overlays/controls\\.rs|mod controls;|pub use controls::" ecosystem/fret-node/src -g '*.rs'`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-126 Prove declarative minimap managed-host side-effect parity.
  - Scope:
    - `crates/fret-ui/src/managed_surface.rs`
    - `crates/fret-ui/src/widget.rs`
    - `ecosystem/fret-node/src/ui/overlays/minimap_declarative.rs`
    - `ecosystem/fret-node/src/ui/overlays/minimap_navigation_policy.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Upgrade the default declarative minimap from paint-only composition to a managed host that
      owns the retained minimap's host side effects: minimap-only hit testing, pointer capture,
      focus return to the node graph surface, drag pan updates, keyboard pan/zoom, Escape focus
      return, redraw, and notify.
    - Keep retained `minimap.rs` as the `compat-retained-canvas` oracle in this slice; deletion
      must be a narrow follow-up after default and compat evidence are green.
  - Result:
    - Declarative minimap now renders a focusable `node_graph.minimap` semantics root backed by a
      `ManagedSurface` host and declarative canvas child.
    - The managed host publishes the minimap hit-test rect so pointer-downs outside the minimap
      fall through to the node graph surface while pointer-downs inside focus the surface, capture
      the pointer on the minimap host, and stop propagation.
    - Added default declarative minimap integration tests for pointer fallthrough, drag pan
      view/store updates without surface leakage, keyboard pan/zoom, and Escape focus return.
    - Added an object-safe minimap viewport update adapter for action hooks so focus-root keyboard
      handling and managed-surface pointer handling share the same navigation semantics.
    - Retained minimap remains in place under `compat-retained-canvas`; its minimap-only oracle
      tests stayed green beside the new default declarative tests.
  - Validation:
    - `cargo nextest run -p fret-node minimap_declarative`
    - `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo fmt -p fret-ui -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-127 Delete the retained minimap widget after default managed-host parity proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/minimap.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete the retained minimap widget and minimap retained test-only exports now that
      `RBX-M2-126` proves the default declarative host covers retained minimap hit-test,
      keyboard, pointer, focus, capture, redraw/notify, and store/controller viewport behavior.
    - Remove `src/ui/overlays/minimap.rs` from the retained bridge source migration ledger only
      after default declarative tests and retained compat oracle tests are both green in the same
      deletion diff.
  - Result:
    - Deleted `ecosystem/fret-node/src/ui/overlays/minimap.rs`.
    - Deleted the retained minimap oracle module
      `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_minimap_controls_conformance.rs`
      after first running the oracle in the current worktree.
    - Removed the retained minimap module/export from `ui/overlays/mod.rs` and the crate-private
      test re-export from `ui/mod.rs`.
    - Removed `src/ui/overlays/minimap.rs` from
      `retained_bridge_source_usage_stays_on_the_migration_ledger` and removed the now-obsolete
      source-policy assertions that read the deleted retained minimap source.
  - Validation:
    - deletion-preflight:
      `cargo nextest run -p fret-node --features compat-retained-canvas minimap_declarative minimap_interaction_policy overlay_minimap_controls_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo nextest run -p fret-node controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas controls_declarative controls_host_policy controls_interaction_policy minimap_declarative minimap_interaction_policy retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `rg -n "\\bNodeGraphMiniMapOverlay\\b|overlay_minimap_controls_conformance|src/ui/overlays/minimap\\.rs|include_str!\\(\\\"ui/overlays/minimap\\.rs\\\"\\)|mod minimap;|pub use minimap|MINIMAP_RS|minimap_navigation_surface_stays" ecosystem/fret-node/src -g '*.rs'`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-128 Prove declarative blackboard host side-effect parity.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Upgrade the default declarative blackboard composition into a focusable host path that owns
      retained blackboard host side effects for pointer fallthrough/blocking, panel focus,
      pointer capture/up completion, root keyboard navigation/activation, Escape focus return, and
      action dispatch hook routing without constructing `NodeGraphBlackboardOverlay`.
    - Keep retained `blackboard.rs` and `blackboard_paint.rs` as the `compat-retained-canvas`
      oracle until the remaining graph/controller transaction adapter and symbol-rename handoff
      are covered on the default declarative surface.
  - Result:
    - Declarative blackboard now renders a focusable `node_graph.blackboard` semantics root with a
      retained-compatible active action value.
    - The panel is wrapped in a pointer region so blank panel pointer-downs focus the blackboard
      root and stop propagation, while pointer-downs outside the panel fall through to the surface.
    - Blackboard action buttons now use pressable pointer-down/up hooks to update shared
      interaction state, capture/release through the default pressable mechanism, restore focus to
      the blackboard root, and route pointer/keyboard activation through the existing action hook.
    - Added default declarative tests for pointer capture/up completion, root keyboard
      navigation/activation, Escape focus return, pointer fallthrough, and panel blocking.
    - Retained blackboard remains in place under `compat-retained-canvas`; its oracle tests stayed
      green beside the new default declarative tests.
  - Validation:
    - `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_blackboard_conformance retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-129 Wire declarative blackboard actions to the default binding and overlay state.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/blackboard_declarative.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the remaining retained blackboard transaction submission and symbol-rename handoff
      capability onto the default declarative blackboard path without exposing
      `NodeGraphEditQueue` or retained widget APIs.
    - Use `NodeGraphSurfaceBinding`, store-first graph/view snapshots, `plan_blackboard_action`,
      and `NodeGraphOverlayState` for the default path.
  - Result:
    - Added `NodeGraphBlackboardActionIntegration`, carrying a `NodeGraphSurfaceBinding`,
      `NodeGraphOverlayState` model, and current surface bounds.
    - Declarative blackboard activation now can commit Add Symbol, Insert Symbol Ref, and Delete
      Symbol transactions through the binding/controller/store path.
    - Declarative blackboard Rename now opens `NodeGraphOverlayState.symbol_rename` without
      queueing a graph transaction.
    - Default tests cover Add/Insert/Delete/Rename integration without constructing the retained
      `NodeGraphBlackboardOverlay` or using `NodeGraphEditQueue`.
    - Retained blackboard remains available only as the `compat-retained-canvas` oracle for the
      next deletion slice.
  - Validation:
    - `cargo nextest run -p fret-node blackboard_declarative`
    - `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan`
    - `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_blackboard_conformance`
- [x] RBX-M2-130 Delete the retained blackboard widget after default integration parity proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/blackboard.rs`
    - `ecosystem/fret-node/src/ui/overlays/blackboard_paint.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_blackboard_conformance.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Remove the retained blackboard widget/paint adapter/oracle only after the retained oracle
      passes in the current worktree and the default declarative path covers host side effects,
      transaction submission, and symbol-rename handoff.
  - Result:
    - Ran a deletion-preflight compat retained oracle before deleting retained blackboard source.
    - Deleted the retained `NodeGraphBlackboardOverlay`, retained blackboard paint adapter, and
      retained `overlay_blackboard_conformance` module.
    - Removed retained blackboard exports/module declarations and removed blackboard files from the
      retained bridge source migration ledger.
    - Kept default declarative blackboard composition, interaction policy, paint-plan, binding
      transaction, and rename-handoff tests as the behavior contract.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas overlay_blackboard_conformance blackboard_declarative blackboard_interaction_policy blackboard_paint_plan`
    - `cargo nextest run -p fret-node blackboard_declarative blackboard_interaction_policy blackboard_paint_plan overlay_policy_modules_compile_without_retained_canvas_compat default_overlay_policy_surfaces_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node --features compat-retained-canvas blackboard_declarative blackboard_interaction_policy blackboard_paint_plan retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-node --no-default-features --features fret-ui`
    - `rg -n "\\bNodeGraphBlackboardOverlay\\b|overlay_blackboard_conformance|ui/overlays/blackboard\\.rs|blackboard_paint\\.rs|mod blackboard;|mod blackboard_paint;|pub use blackboard" ecosystem/fret-node/src -g '*.rs'`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-131 Prove declarative rename managed-host parity.
  - Scope:
    - `crates/fret-ui/src/managed_surface.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_declarative.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_host_layout.rs`
    - `ecosystem/fret-node/src/ui/overlays/rename_policy.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move retained rename host side effects onto the default declarative managed-host path before
      deleting `NodeGraphOverlayHost`.
    - Prove seed-text ownership, first-open focus, submit/cancel command routing, graph/store
      transaction submission, focus restore, focus-loss close, and retained-compatible hit-test
      masking without constructing the retained rename host.
    - Keep the retained rename host and retained oracle tests in place until this proof and the
      compat retained oracle are both green in the same worktree.
  - Result:
    - Added `NodeGraphRenameOverlayHostProps` and
      `node_graph_rename_overlay_host_element(...)` backed by `ManagedSurface`.
    - Extended `ManagedSurface` with the mechanism hooks rename needs: layout focus access,
      element focus requests, host-selected hit-test rects, command-time element focus restore, and
      event notify support.
    - Added default declarative rename managed-host tests for seed/focus/hit-test masking,
      submit-through-`NodeGraphSurfaceBinding`, Escape cancel/focus restore, and focus-loss close
      without transaction or focus steal.
    - Kept retained group/symbol rename conformance tests green under `compat-retained-canvas`;
      retained rename source was deliberately left in place for the next narrow deletion slice.
  - Validation:
    - `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout`
    - `cargo nextest run -p fret-ui managed_surface`
    - `cargo nextest run -p fret-node --features compat-retained-canvas overlay_group_rename_conformance overlay_symbol_rename_conformance rename_declarative rename_lifecycle rename_command`
    - `cargo fmt --check`
    - `git diff --check -- crates/fret-ui/src/managed_surface.rs ecosystem/fret-node/src/ui/overlays/rename_declarative.rs ecosystem/fret-node/src/ui/overlays/mod.rs ecosystem/fret-node/src/ui/overlays/rename_host_layout.rs ecosystem/fret-node/src/ui/overlays/rename_policy.rs`
- [x] RBX-M2-132 Delete retained rename host after parity proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/overlays/group_rename.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - deleted `ecosystem/fret-node/src/ui/overlays/rename_host_event.rs`
    - deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_group_rename_conformance.rs`
    - deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_symbol_rename_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Remove the retained `NodeGraphOverlayHost` adapter after the default declarative managed host
      and deletion-preflight retained oracle both proved rename behavior.
    - Delete the retained-only rename event adapter and retained group/symbol rename oracle tests.
    - Shrink the retained bridge source usage ledger so `group_rename.rs` and `overlays/mod.rs` can
      no longer carry retained bridge types.
  - Result:
    - `group_rename.rs` now only defines `NodeGraphOverlayState`, `GroupRenameOverlay`, and
      `SymbolRenameOverlay`.
    - Removed retained rename host exports and test-only re-exports.
    - Removed the retained rename oracle modules from the compat canvas test module list.
    - Removed `rename_host_event.rs`, the orphaned retained hidden-child layout helper, and the
      group-rename/overlay-mod retained source allowlist entries.
    - Default declarative rename managed-host tests now own the rename behavior contract.
  - Validation:
    - deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas overlay_group_rename_conformance overlay_symbol_rename_conformance rename_declarative rename_lifecycle rename_command`
    - deletion-preflight `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout`
    - post-delete `cargo nextest run -p fret-node rename_declarative rename_lifecycle rename_command rename_host_layout`
    - post-delete `cargo check -p fret-node --features compat-retained-canvas`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas rename_declarative rename_lifecycle rename_command retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge overlay_policy_modules_compile_without_retained_canvas_compat`
    - post-delete `cargo nextest run -p fret-ui managed_surface`
    - post-delete `cargo check -p fret-node --no-default-features --features fret-ui`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "NodeGraphOverlayHost|rename_host_event|overlay_group_rename_conformance|overlay_symbol_rename_conformance|layout_hidden_child_and_release_focus|src/ui/overlays/group_rename\\.rs" ecosystem/fret-node/src -g '*.rs'`
- [x] RBX-M2-133 Delete no-user retained diagnostics anchor widgets.
  - Scope:
    - `ecosystem/fret-node/src/ui/diag_anchors.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/builders.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/widget_surface/construct.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_children.rs`
    - deleted `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_layout_publish.rs`
    - `docs/ui-diagnostics-and-scripted-tests.md`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete retained diagnostics-only semantics anchor widgets that have no callers.
    - Remove the dead retained canvas diagnostics-anchor child layout plumbing that only existed to
      support those widgets.
    - Shrink the retained bridge source usage ledger by removing `diag_anchors.rs`.
  - Result:
    - Deleted `NodeGraphDiagAnchor` and `NodeGraphDiagConnectingFlag`.
    - Removed `mod diag_anchors`, the retained bridge source allowlist entry, and stale diagnostic
      anchor docs that pointed users at retained anchor widgets.
    - Removed `with_diagnostics_anchor_ports`, `diagnostics_anchor_ports`, and the retained layout
      publish helper because no retained or declarative caller remains.
    - Kept `a11y.rs` for follow-up `RBX-M2-134` because retained canvas active-descendant child
      semantics still needed a default declarative proof before deletion.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "DiagnosticsAnchorPorts|diagnostics_anchor_ports|with_diagnostics_anchor_ports|retained_widget_layout_publish|publish_diagnostics_derived_outputs|NodeGraphDiagAnchor|NodeGraphDiagConnectingFlag|diag_anchors" ecosystem/fret-node/src docs/ui-diagnostics-and-scripted-tests.md docs/workstreams/retained-bridge-exit-v1 -g '*.rs' -g '*.md'`
- [x] RBX-M2-134 Delete retained a11y anchors after declarative active-descendant proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - `ecosystem/fret-node/src/ui/binding.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_frame.rs`
    - deleted `ecosystem/fret-node/src/ui/a11y.rs`
    - deleted `ecosystem/fret-node/src/ui/canvas/widget/tests/a11y_active_descendant_conformance.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move node graph active-descendant semantics to the default declarative surface.
    - Cover focused port, edge, and node active-descendant mapping without enabling
      `compat-retained-canvas`.
    - Delete the retained a11y child-anchor widgets and retained oracle tests after retained and
      default paths agree.
    - Shrink the retained bridge source usage ledger by removing `a11y.rs`.
  - Result:
    - `NodeGraphSurfaceBinding` now owns a UI-only `NodeGraphInternalsStore`, and
      `binding.surface_props()` / `NodeGraphSurfaceProps::new(...)` automatically wire it into the
      default declarative surface.
    - `node_graph_surface(...)` exposes the canvas as a focusable `Viewport` semantics node,
      syncs binding internals from the default surface frame, creates zero-size semantics-only
      children for focused port/edge/node labels, and resolves `active_descendant` through
      declarative element-id relations.
    - Default declarative tests now prove the retained priority order: focused port, then focused
      edge, then focused node.
    - Default declarative tests also cover presenter-derived active-descendant labels and the
      no-stale-descendant case for selected items that are missing from current graph geometry.
    - Deleted `NodeGraphA11yFocusedPort`, `NodeGraphA11yFocusedEdge`, `NodeGraphA11yFocusedNode`,
      their retained module, and their retained conformance test module.
  - Validation:
    - deletion-preflight `cargo nextest run -p fret-node node_graph_surface_active_descendant`
    - deletion-preflight `cargo nextest run -p fret-node --features compat-retained-canvas a11y_active_descendant_conformance node_graph_surface_active_descendant`
    - post-delete `cargo nextest run -p fret-node node_graph_surface_active_descendant retained_bridge_source_usage_stays_on_the_migration_ledger`
    - post-delete `cargo check -p fret-node --features compat-retained-canvas`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound node_graph_surface_active_descendant`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^\\s*(pub\\s+)?mod a11y;|NodeGraphA11yActiveDescendant|NodeGraphA11yFocused|a11y_active_descendant_conformance" ecosystem/fret-node/src -S`
- [x] RBX-M2-135 Prove declarative portal subtree lifecycle and measurement parity.
  - Scope:
    - `ecosystem/fret-node/src/ui/declarative/paint_only/portals.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the retained portal subtree lifecycle key contract onto the default declarative
      visible-subset portal path.
    - Prove default portal subtree identity persists across frames and resets when node
      `kind_version` or `kind` changes, matching the retained `NodeGraphPortalHost` lifecycle
      oracle.
    - Backfill default measurement tests for growth-only node-size hints and removed-node cleanup
      before deleting retained portal host code.
    - Keep retained portal files in place as the oracle because arbitrary per-kind renderer subtree
      hosting and retained command adapters are not deleted in this slice.
  - Result:
    - Added `DeclarativePortalNodeKey` and keyed declarative visible-subset portal labels by
      `(node id, node kind hash, node kind_version)` instead of only node id.
    - Added a default declarative surface test that renders real surface frames, waits for the
      frame-lagged portal layer, proves stable semantics identity across frames, and proves identity
      reset on `kind_version` and `kind` changes.
    - Added a default portal measured-geometry flush test covering growth-only hint behavior and
      cleanup when a previously-published node is removed from the graph.
    - Verified retained portal lifecycle, measured-geometry, and measured-internals oracle tests
      still pass under `compat-retained-canvas`.
  - Validation:
    - `cargo nextest run -p fret-node declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes`
    - `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-140 Host per-kind portal renderers on the default declarative surface.
  - Scope:
    - `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/portals.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_content.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/surface_shell.rs`
    - `ecosystem/fret-node/src/ui/registry.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Replace the retained `NodeGraphPortalHost` arbitrary per-kind renderer callback with a
      default declarative authoring path.
    - Let `NodeGraphNodeTypes` host per-kind portal subtrees through `node_graph_surface(...)`
      family APIs without enabling `compat-retained-canvas`.
    - Preserve the retained portal lifecycle key contract from `RBX-M2-135` and prove custom
      subtree measurement flows into `MeasuredGeometryStore`.
    - Keep retained portal files in place only as command-adapter/deletion-preflight oracle code.
  - Result:
    - Added `NodeGraphDeclarativePortalRenderer` plus
      `node_graph_surface_with_portal_renderer(...)` /
      `node_graph_surface_with_portal_renderer_in(...)`.
    - Implemented `NodeGraphDeclarativePortalRenderer` for `NodeGraphNodeTypes`, so existing
      ReactFlow-style per-kind registries can render directly through the default declarative
      visible-subset portal layer.
    - Changed the declarative portal label host so custom portal subtrees replace the built-in
      lightweight label when they return elements, and empty custom output falls back to the
      built-in label.
    - Kept all hosted portal subtrees keyed by `(node id, node kind, node kind_version)` and made
      custom renderer measurements participate in the same portal measured-geometry pipeline.
    - Verified retained portal lifecycle, measured-geometry, and measured-internals oracle tests
      still pass under `compat-retained-canvas`.
  - Validation:
    - `cargo nextest run -p fret-node declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
    - `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-145 Host portal commands on the default declarative surface.
  - Scope:
    - `ecosystem/fret-node/src/ui/declarative/paint_only.rs`
    - `ecosystem/fret-node/src/ui/declarative/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Replace the retained `NodeGraphPortalHost::command` adapter responsibility with a default
      declarative command host on `node_graph_surface(...)`.
    - Keep command policy/component authorship outside `fret-ui` and avoid importing retained
      `CommandCx` into the default path.
    - Submit `PortalCommandOutcome::Commit(...)` through `NodeGraphSurfaceBinding` so the
      authoritative store and graph/view mirrors remain synchronized.
    - Prove unclaimed portal commands keep bubbling instead of being swallowed.
  - Result:
    - Added `NodeGraphDeclarativePortalCommandHandler` and
      `NodeGraphDeclarativePortalCommandHandlerRef` as the default declarative portal command seam.
    - Added `NodeGraphSurfaceProps::portal_command_handler` and wired surface-root command
      availability, action-route fallback, command parsing, binding-backed transaction submission,
      focus return, redraw, and notify side effects.
    - Re-exported the portal command protocol from the declarative/default API surface so downstream
      handlers can name `PortalTextCommand`, `PortalTextStepMode`, `PortalCommandOutcome`, and the
      command builders/parsers without importing the retained portal module.
    - Added default declarative coverage proving `portal_submit_text_command(node)` commits a graph
      transaction through the surface binding without constructing `NodeGraphPortalHost`, and that
      commands for unhandled nodes are not swallowed.
    - Verified retained portal lifecycle, measured-geometry, and measured-internals oracle tests
      still pass under `compat-retained-canvas`.
  - Validation:
    - `cargo nextest run -p fret-node declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
    - `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node declarative_portal_command_host_submits_transactions_without_retained_portal_host retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-150 Delete retained portal text/number command adapters after default handler proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/editors/portal_text.rs`
    - `ecosystem/fret-node/src/ui/editors/portal_number.rs`
    - `ecosystem/fret-node/src/ui/editors/mod.rs`
    - `ecosystem/fret-node/src/ui/declarative/mod.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/declarative/paint_only/tests.rs`
    - `ecosystem/fret-node/src/ui/portal.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move first-party portal text/number editor command handlers onto the default declarative
      command-handler seam.
    - Delete the retained `CommandCx` adapter implementations from `portal_text.rs` and
      `portal_number.rs`.
    - Keep retained `NodeGraphPortalHost` only as lifecycle/measurement oracle code while default
      text/number editor handlers own command policy and session model I/O.
  - Result:
    - `PortalTextEditor` and `PortalNumberEditor` now own cloneable, model-backed session state
      instead of storing editor sessions in host globals.
    - `PortalTextEditHandler` and `PortalNumberEditHandler` implement
      `NodeGraphDeclarativePortalCommandHandler`, expose `with_editor(...)` for explicit renderer /
      command-handler state sharing, and are re-exported from default `fret-node::ui` /
      `fret-node::ui::declarative`.
    - Deleted retained `NodeGraphPortalCommandHandler` impls for text/number editor handlers and
      removed `portal_text.rs` / `portal_number.rs` from the retained bridge source migration
      ledger.
    - Added default declarative surface tests proving both text and number editor handlers submit
      binding-backed graph transactions without retained `CommandCx`.
    - Kept retained portal lifecycle/measurement oracle tests green after deletion.
  - Validation:
    - pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx`
    - post-delete `cargo nextest run -p fret-node declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx retained_bridge_source_usage_stays_on_the_migration_ledger editor_chrome_compiles_without_retained_canvas_compat portal_command_session`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-160 Delete retained portal host after default lifecycle/renderer/command parity proof.
  - Scope:
    - `ecosystem/fret-node/src/ui/portal.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_lifecycle_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_geometry_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/portal_measured_internals_conformance.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete retained `NodeGraphPortalHost` after the default declarative path has parity coverage
      for portal subtree lifecycle keys, measured-geometry cleanup/publishing, arbitrary per-kind
      renderer hosting, portal command routing, and first-party text/number editor command
      submission.
    - Remove the retained portal oracle modules and shrink the retained bridge source migration
      ledger so portal code cannot re-enter the retained compatibility island.
  - Result:
    - Deleted `ui/portal.rs`, including retained `NodeGraphPortalHost`,
      `NodeGraphPortalCommandHandler`, `PortalNoopCommandHandler`, and
      `PortalCommandHandlerChain`.
    - Deleted retained portal lifecycle/measured-geometry/measured-internals oracle test modules.
    - Removed the compat-gated `mod portal;` entry and the unused retained-canvas `node_order`
      re-export that only the retained portal host consumed.
    - Updated the retained bridge source policy test and capability ledger so `src/ui/portal.rs` is
      no longer an allowed retained source.
    - Default declarative portal lifecycle, renderer, measurement, command, and text/number editor
      tests now carry the portal contract after retained host deletion.
  - Validation:
    - pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas portal_lifecycle_conformance portal_measured_geometry_conformance portal_measured_internals_conformance declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx`
    - post-delete `cargo check -p fret-node --features compat-retained-canvas`
    - post-delete `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo nextest run -p fret-node declarative_visible_subset_portal_identity_persists_and_resets_on_kind_or_version_change flush_portal_measured_geometry_state_keeps_growth_only_and_removes_missing_nodes declarative_portal_renderer_hosts_custom_subtrees_by_node_kind_with_default_fallback declarative_surface_hosts_node_type_registry_without_retained_portal_host declarative_portal_renderer_publishes_custom_subtree_measurements declarative_portal_command_host_submits_transactions_without_retained_portal_host declarative_portal_text_editor_handler_submits_transactions_without_retained_command_cx declarative_portal_number_editor_handler_submits_transactions_without_retained_command_cx`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-170 Delete unused retained editor/panel composition wrappers.
  - Scope:
    - `ecosystem/fret-node/src/ui/editor.rs`
    - `ecosystem/fret-node/src/ui/panel.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete retained `NodeGraphEditor` and `NodeGraphPanel` wrappers after proving there are no
      live source consumers outside their own files.
    - Keep the actual window-space placement math on the default `screen_space_placement` module
      rather than retaining a dead `Widget` wrapper.
    - Shrink the retained bridge source migration ledger so editor/panel wrappers cannot re-enter
      the compatibility island.
  - Result:
    - Deleted `ui/editor.rs` and `ui/panel.rs`.
    - Removed their compat-gated module entries and retained-source ledger allowlist entries.
    - Updated the overlay placement comment to describe window-space panel bounds without naming
      the deleted retained wrapper.
    - Kept `screen_space_placement::rect_in_bounds` as the default placement contract.
  - Validation:
    - pre-delete `rg -n "\bNodeGraphEditor\b|\bNodeGraphPanel\b|\bNodeGraphPanelPosition\b|\bNodeGraphPanelSize\b" ecosystem/fret-node/src apps crates ecosystem tools --glob '!target/**' --glob '!ecosystem/fret-node/src/ui/editor.rs' --glob '!ecosystem/fret-node/src/ui/panel.rs' --glob '!ecosystem/fret-node/src/lib.rs'`
    - pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas positioned_rect_top_right_respects_margin rect_in_bounds_top_right_respects_margin retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo nextest run -p fret-node rect_in_bounds_top_right_respects_margin retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-180 Delete no-user retained submit/tail/panel paint helpers.
  - Scope:
    - `ecosystem/fret-node/src/ui/retained_submit.rs`
    - `ecosystem/fret-node/src/ui/retained_event_tail.rs`
    - `ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs`
    - `ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `ecosystem/fret-node/src/ui/overlays/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete retained helper modules that no longer have live consumers after the overlay, portal,
      editor, and panel wrapper deletions.
    - Move `panel_pointer_policy.rs` fully onto the default policy path by deleting its retained
      `begin_panel_press` adapter.
    - Remove overlay retained bridge source entries from the `fret-node` retained source ledger.
  - Result:
    - Deleted `retained_submit.rs`, `retained_event_tail.rs`, and `panel_button_paint.rs`.
    - Removed their module entries from `ui/mod.rs` / `ui/overlays/mod.rs`.
    - Deleted retained-only `begin_panel_press(...)`; `panel_pointer_policy.rs` now contains only
      default hover/release policy shared by controls and blackboard.
    - Shrank the retained bridge source policy allowlist so only the retained canvas widget root,
      middleware, and `canvas/widget/**` remain.
  - Validation:
    - pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target centered_text_origin_centers_within_button_rect leading_text_origin_keeps_padding_and_vertical_centering retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - pre-delete `rg -n "\b(retained_submit|submit_graph_transaction|submit_graph_and_view_transaction|retained_event_tail|request_paint_repaint|finish_paint_event|focus_canvas_and_finish_paint_event|focus_canvas_and_finish_layout_event|finish_portal_command|begin_panel_press|paint_panel_button|paint_panel_label|centered_text_origin|leading_text_origin)\b" ecosystem/fret-node/src apps crates ecosystem tools --glob '!target/**' --glob '!ecosystem/fret-node/src/lib.rs' --glob '!ecosystem/fret-node/src/ui/retained_submit.rs' --glob '!ecosystem/fret-node/src/ui/retained_event_tail.rs' --glob '!ecosystem/fret-node/src/ui/overlays/panel_button_paint.rs' --glob '!ecosystem/fret-node/src/ui/overlays/panel_pointer_policy.rs'`
    - post-delete `cargo check -p fret-node --features compat-retained-canvas`
    - post-delete `cargo nextest run -p fret-node sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas sync_panel_hover_only_reports_real_changes release_panel_press_only_activates_on_matching_release_target retained_bridge_source_usage_stays_on_the_migration_ledger default_overlay_policy_surfaces_stay_off_retained_bridge retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-190 Remove retained event/command hooks from canvas middleware.
  - Scope:
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
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Remove the retained `EventCx` / `CommandCx` middleware hook surface while preserving
      `before_commit` as the remaining transaction guard.
    - Move `canvas/middleware.rs` out of the retained bridge source migration ledger.
  - Result:
    - Deleted `NodeGraphCanvasEventOutcome`, `NodeGraphCanvasCommandOutcome`, middleware
      `handle_event(...)`, and middleware `handle_command(...)`.
    - Removed retained runtime dispatch through middleware event/command hooks.
    - Kept `NodeGraphCanvasMiddleware::before_commit(...)` and the commit-rejection conformance
      test green.
    - `canvas/middleware.rs` no longer imports or names `retained_bridge`, `EventCx`, or
      `CommandCx`; the retained bridge source allowlist now contains only `src/ui/canvas/widget.rs`
      and `src/ui/canvas/widget/**`.
  - Validation:
    - pre-delete `cargo nextest run -p fret-node --features compat-retained-canvas middleware_can_override_select_all_command middleware_can_reject_commits_before_apply retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo check -p fret-node --features compat-retained-canvas`
    - post-delete `cargo nextest run -p fret-node retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - post-delete `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound middleware_can_reject_commits_before_apply`
    - post-delete `rg -n "retained_bridge|CommandCx|EventCx|NodeGraphCanvasCommandOutcome|NodeGraphCanvasEventOutcome|handle_event\\(|handle_command\\(" ecosystem/fret-node/src/ui/canvas/middleware.rs ecosystem/fret-node/src/ui/canvas/middleware -g '*.rs'`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-200 Isolate retained canvas widget tail Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_runtime_shared.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/command_ui.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move retained canvas widget tail actions (`request_redraw`, paint invalidation, and
      handled-event propagation stop) behind retained-agnostic internal traits.
    - Keep retained `EventCx` / `CommandCx` / `LayoutCx` / `PaintCx` implementations in one
      retained adapter module instead of leaking those Cx types into policy helpers.
    - Lock the extracted pure helper files with a default source-policy gate.
  - Result:
    - Added `widget_tail.rs` as a retained-agnostic tail action seam with unit tests.
    - Added `retained_widget_tail.rs` as the only new retained Cx adapter for those tail actions.
    - Moved `paint_invalidation.rs` and `redraw_request.rs` off direct retained bridge imports.
    - Removed the generic retained Cx tail trait from `retained_widget_runtime_shared.rs`, leaving
      that file responsible only for retained runtime theme/service sync.
    - Added `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` so
      `paint_invalidation.rs`, `redraw_request.rs`, and `widget_tail.rs` cannot reintroduce
      retained bridge/Cx imports.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `cargo nextest run -p fret-node --features compat-retained-canvas widget_tail retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-210 Isolate wire-drag commit retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Keep `wire_drag/commit_cx.rs` as the retained-agnostic wire commit side-effect seam.
    - Move retained `EventCx` / `CommandCx` implementations into a retained adapter module.
    - Extend the default source-policy gate so the pure commit seam cannot re-import retained
      bridge Cx types.
  - Result:
    - Added `wire_drag/retained_commit_cx.rs` for retained `EventCx` and `CommandCx`
      `WireCommitCx` implementations.
    - Removed direct retained bridge imports and impls from `wire_drag/commit_cx.rs`.
    - Added a retained-agnostic unit test for `invalidate_commit_paint(...)` sequencing.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include
      `wire_drag/commit_cx.rs`.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail commit_cx retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs`
    - `cargo fmt -p fret-node`
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
