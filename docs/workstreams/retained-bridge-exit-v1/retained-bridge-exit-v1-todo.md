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
- [x] RBX-M2-220 Isolate pointer-up finish retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/retained_widget_tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pointer-up finish tail behavior (release pointer capture + invalidate paint) behind the
      retained-agnostic widget tail seam.
    - Keep retained `EventCx` release-pointer-capture implementation in the retained tail adapter.
    - Extend the default source-policy gate so pointer-up finish helper files cannot re-import
      retained bridge Cx names.
  - Result:
    - Added `PointerCaptureReleaseCx` and `finish_pointer_capture_release(...)` to `widget_tail.rs`
      with unit coverage for release/redraw/paint-invalidation sequencing.
    - Implemented `PointerCaptureReleaseCx` for retained `EventCx` in `retained_widget_tail.rs`.
    - Moved `pointer_up_finish.rs` and `pointer_up_session/cleanup.rs` off direct retained
      `EventCx` signatures.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include the
      pointer-up finish helper files.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-230 Isolate sticky-wire finish retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move sticky-wire pointer-down finish behavior (release pointer capture + stop propagation +
      invalidate paint) behind the retained-agnostic widget tail seam.
    - Extend the default source-policy gate so the sticky-wire finish helper cannot re-import
      retained bridge Cx names.
  - Result:
    - Added `HandledPointerCaptureReleaseCx` and `finish_handled_pointer_capture_release(...)` to
      `widget_tail.rs` with unit coverage for release/stop/redraw/paint-invalidation sequencing.
    - Moved `sticky_wire_connect/finish.rs` off direct retained `EventCx` signatures.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include the
      sticky-wire finish helper.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge widget_tail retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-240 Isolate edge-insert drag tail retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move edge-insert drag move finish paint invalidation behind the retained-agnostic widget tail
      seam.
    - Extend the default source-policy gate so the edge-insert drag tail helper cannot re-import
      retained bridge Cx names.
  - Result:
    - Moved `edge_insert_drag/drag/tail.rs` off direct retained `EventCx` signatures.
    - Reused the existing `WidgetPaintInvalidationCx` seam and `invalidate_widget_paint(...)`
      helper for redraw plus paint invalidation.
    - Added a retained-agnostic unit test for edge-insert drag move tail invalidation.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include the
      edge-insert drag tail helper.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge finish_edge_insert_drag_move_invalidates_paint retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-250 Isolate cancel cleanup retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/cancel.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move cancel finish tail behavior (release pointer capture, optional stop propagation,
      redraw, and paint invalidation) behind the retained-agnostic widget tail seam.
    - Keep retained `cx.app` timer I/O in the retained caller instead of hiding host access inside
      a generic cancel cleanup trait.
    - Extend the default source-policy gate so `cancel_cleanup.rs` cannot re-import retained bridge
      Cx names.
  - Result:
    - Moved `cancel_cleanup.rs::finish_cancel(...)` off direct retained `EventCx` signatures.
    - Kept `canvas.stop_auto_pan_timer(cx.app)` in `cancel.rs`, the retained event caller.
    - Added retained-agnostic unit tests for consuming and non-consuming cancel finish tails.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include
      `cancel_cleanup.rs`.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas finish_cancel retained_canvas_tail_policy_helpers_stay_off_retained_bridge escape_cancel_releases_pointer_capture_during_panning escape_cancel_emits_connect_end_canceled escape_cancel_panning_emits_move_end_canceled node_drag_start_and_escape_cancel_emits_node_drag_end_canceled retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-260 Isolate sticky-wire target picker retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/retained_picker_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move sticky-wire target picker host/window access plus handled-event finish behavior behind a
      retained-agnostic picker Cx seam.
    - Keep the retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so `sticky_wire_targets/picker.rs` cannot re-import
      retained bridge Cx names.
  - Result:
    - Added `StickyWireTargetPickerCx` for host/window access plus handled-event tail behavior.
    - Added `sticky_wire_targets/retained_picker_cx.rs` as the retained `EventCx` adapter.
    - Moved `sticky_wire_targets/picker.rs` off direct retained `EventCx` signatures.
    - Added a retained-agnostic unit test for target picker stop-propagation plus paint
      invalidation.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include
      `sticky_wire_targets/picker.rs`.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas finish_sticky_wire_target_picker_stops_and_invalidates_paint retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-270 Isolate group preview tail retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_drag/tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_resize/tail.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move group drag/resize preview tail paint invalidation behind the retained-agnostic widget
      tail seam.
    - Keep retained `cx.app` auto-pan view-state I/O in the retained event callers instead of
      hiding host access inside the preview tail helpers.
    - Extend the default source-policy gate so the group drag/resize tail helpers cannot re-import
      retained bridge Cx names.
  - Result:
    - Moved `group_drag/tail.rs` and `group_resize/tail.rs` off direct retained `EventCx`
      signatures.
    - Reused `WidgetPaintInvalidationCx` and `invalidate_widget_paint(...)` for redraw plus paint
      invalidation.
    - Kept auto-pan view-state updates in `group_drag.rs` and `group_resize.rs`, the retained event
      callers that still own `cx.app` access.
    - Added retained-agnostic unit tests for group drag/resize preview state updates and no-op
      preview revision behavior.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include both
      group preview tail helpers.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas update_drag_preview_state update_resize_preview_state group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/paint_invalidation.rs ecosystem/fret-node/src/ui/canvas/widget/redraw_request.rs ecosystem/fret-node/src/ui/canvas/widget/widget_tail.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_finish.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect/finish.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/cancel_cleanup.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs ecosystem/fret-node/src/ui/canvas/widget/group_drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize/tail.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-280 Isolate group preview move retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move group drag/resize move handler host/bounds access behind a retained-agnostic
      `GroupPreviewMoveCx` seam.
    - Keep the retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so `group_drag.rs`, `group_resize.rs`, and the pure
      move Cx seam cannot re-import retained bridge Cx names.
  - Result:
    - Added `GroupPreviewMoveCx` for retained-agnostic host/bounds access plus widget paint
      invalidation.
    - Added `group_preview_move_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `group_drag.rs` and `group_resize.rs` off direct retained `EventCx` signatures.
    - Added `group_preview_move_handlers_stay_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas group_preview_move_handlers_stay_off_retained_bridge update_drag_preview_state update_resize_preview_state group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_canvas_tail_policy_helpers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-290 Isolate pending group activation retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pending group drag activation host access behind a retained-agnostic
      `PendingGroupActivationCx` seam.
    - Remove the unused retained Cx parameter from pending group resize activation.
    - Extend the default source-policy gate so pending group activation handlers and the pure Cx
      seam cannot re-import retained bridge Cx names.
  - Result:
    - Added `PendingGroupActivationCx` for retained-agnostic host access.
    - Added `pending_group_activation_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `pending_group_drag.rs` off direct retained `EventCx` signatures.
    - Removed the unused Cx parameter from `pending_group_resize.rs`.
    - Added `pending_group_activation_handlers_stay_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pending_group_activation_handlers_stay_off_retained_bridge group_preview_move_handlers_stay_off_retained_bridge pending_group_drag_release_clears_session_without_committing pending_group_resize_release_clears_session_without_committing group_header_click_selects_group_and_arms_pending_group_drag group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs`
    - `cargo fmt -p fret-node`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-300 Isolate pending release retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/group.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/node.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pending group drag, pending group resize, and pending node resize pointer-up release
      tail actions behind the retained-agnostic `PointerCaptureReleaseCx` seam.
    - Reuse the retained `EventCx` implementation that already lives in `retained_widget_tail.rs`
      instead of naming retained bridge Cx types in pending release helpers.
    - Extend the default source-policy gate so pending release helpers cannot re-import retained
      bridge Cx names.
  - Result:
    - Moved `pointer_up_session/release.rs`, `pointer_up_pending/release.rs`,
      `pointer_up_pending/release/group.rs`, and `pointer_up_pending/release/node.rs` off direct
      retained `EventCx` signatures.
    - Reused the existing `PointerCaptureReleaseCx` tail seam for release-capture plus paint
      invalidation behavior.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include the
      pending release helpers.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge pending_group_drag_release_clears_session_without_committing pending_group_resize_release_clears_session_without_committing pending_group_activation_handlers_stay_off_retained_bridge retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_session/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/group.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/release/node.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-310 Isolate pending wire release retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pending wire drag pointer-up release tail actions behind the retained-agnostic
      `PointerCaptureReleaseCx` seam.
    - Keep promotion logic retained-agnostic and reuse the retained `EventCx` implementation that
      already lives in `retained_widget_tail.rs`.
    - Extend the default source-policy gate so pending wire release helpers cannot re-import
      retained bridge Cx names.
  - Result:
    - Moved `pointer_up_pending/wire_drag.rs` off direct retained `EventCx` signatures.
    - Reused `PointerCaptureReleaseCx` for pointer capture release plus paint invalidation after
      pending wire drag release/promotion.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include the
      pending wire release helper.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_canvas_tail_policy_helpers_stay_off_retained_bridge should_promote_pending_wire_drag_requires_click_connect_and_new_drag click_connect_target_port_click_commits_wire_and_clears_click_connect_state retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/wire_drag.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-320 Isolate pending node drag click-select retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/click_select.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pending node drag click-select release view-state I/O plus pointer-up tail actions behind
      a retained-agnostic `PendingNodeDragReleaseCx` seam.
    - Keep retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so pending node drag click-select release helpers and
      the pure Cx seam cannot re-import retained bridge Cx names.
  - Result:
    - Added `PendingNodeDragReleaseCx` for retained-agnostic host access plus pointer capture
      release/paint invalidation.
    - Added `pending_node_drag_release_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `pointer_up_pending/click_select.rs` off direct retained `EventCx` signatures.
    - Added `pending_node_drag_release_handlers_stay_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pending_node_drag_release_handlers_stay_off_retained_bridge apply_pending_node_selection_toggles_selection_and_keeps_node_last_in_draw_order shift_clicking_a_node_does_not_clear_selection node_click_does_not_select_node_when_node_selectable_is_false retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_pending/click_select.rs ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_release_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-330 Isolate pointer-up commit retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/group_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/group.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/node.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move group drag, group resize, and node resize pointer-up commit host/window I/O plus
      release tail actions behind a retained-agnostic `PointerUpCommitCx` seam.
    - Keep retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so pointer-up commit handlers and the pure Cx seam
      cannot re-import retained bridge Cx names.
  - Result:
    - Added `PointerUpCommitCx` for retained-agnostic host/window access plus pointer capture
      release/paint invalidation.
    - Added `pointer_up_commit_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved group drag, group resize, and node resize pointer-up commit helpers off direct
      retained `EventCx` signatures.
    - Added `pointer_up_commit_handlers_stay_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_commit_handlers_stay_off_retained_bridge build_group_drag_ops_includes_group_and_moved_nodes_only build_node_resize_ops_collects_node_and_group_changes node_resize_expands_group_when_expand_parent_is_true group_resize_is_previewed_and_committed_on_pointer_up group_resize_clamps_to_children retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/group.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit/resize/node.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-340 Isolate node drag move tail retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag/tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move node drag move tail host I/O and paint invalidation behind a retained-agnostic
      `NodeDragMoveTailCx` seam.
    - Keep retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so node drag move tail helpers and the pure Cx seam
      cannot re-import retained bridge Cx names.
  - Result:
    - Added `NodeDragMoveTailCx` for retained-agnostic host access plus paint invalidation.
    - Added `node_drag_move_tail_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `node_drag/tail.rs` off direct retained `EventCx` signatures.
    - Added `node_drag_move_tail_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_move_tail_stays_off_retained_bridge node_drag_move_emits_on_node_drag child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true node_drag_records_single_history_entry_for_multi_node_move retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag/tail.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_tail_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-350 Isolate marquee begin/finish retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/marquee_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move marquee begin capture/paint invalidation and marquee finish view-state I/O/release tail
      actions behind a retained-agnostic `MarqueeCx` seam.
    - Keep retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so marquee begin/finish helpers and the pure Cx seam
      cannot re-import retained bridge Cx names.
  - Result:
    - Added `MarqueeCx` for retained-agnostic host access, self pointer capture, and pointer-up
      release/paint invalidation.
    - Added `marquee_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `marquee_begin.rs` and `marquee_finish.rs` off direct retained `EventCx` signatures.
    - Added `marquee_begin_finish_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas marquee_begin_finish_stays_off_retained_bridge background_click_starts_pending_marquee_and_clears_selection_on_up marquee_replace_mode_replaces_selection_even_with_ctrl_pressed marquee_selects_connected_edges_for_selected_nodes marquee_selects_connected_edges_for_selected_nodes_with_store retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/marquee_begin.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_cx.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_finish.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-360 Isolate node drag preview compute retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview/compute.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move node drag preview host/graph-read I/O behind a retained-agnostic
      `NodeDragPreviewCx` seam.
    - Keep retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so node drag preview wrapper/compute helpers and the
      pure Cx seam cannot re-import retained bridge Cx names.
  - Result:
    - Added `NodeDragPreviewCx` for retained-agnostic host access.
    - Added `node_drag_preview_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `node_drag_preview.rs` and `node_drag_preview/compute.rs` off direct retained
      `EventCx` signatures.
    - Added `node_drag_preview_compute_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_preview_compute_stays_off_retained_bridge node_drag_move_emits_on_node_drag child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true node_drag_records_single_history_entry_for_multi_node_move retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview/compute.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_preview_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-370 Isolate node drag geometry retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move node drag snapline geometry reads and multi-drag extent geometry reads behind a
      retained-agnostic `NodeDragGeometryCx` seam.
    - Keep retained `EventCx` implementation in a dedicated retained adapter module.
    - Extend the default source-policy gate so node drag geometry helpers and the pure Cx seam
      cannot re-import retained bridge Cx names.
  - Result:
    - Added `NodeDragGeometryCx` for retained-agnostic host access.
    - Added `node_drag_geometry_retained_cx.rs` as the retained `EventCx` adapter.
    - Moved `node_drag_snap.rs`, `node_drag_constraints.rs`, and
      `node_drag_constraints_extent.rs` off direct retained `EventCx` signatures.
    - Added `node_drag_geometry_helpers_stay_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas node_drag_geometry_helpers_stay_off_retained_bridge node_drag_move_emits_on_node_drag node_drag_respects_per_node_extent_rect multi_node_drag_clamps_by_selection_bounds_in_node_extent_rect child_node_drag_is_clamped_to_group_when_expand_parent_is_false child_node_drag_expands_group_when_expand_parent_is_true snap_delta_for_rects_snaps_left_edge snap_delta_for_rects_snaps_center_y retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_constraints_extent.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_geometry_cx.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_snap.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-380 Isolate keyboard pan activation retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move keyboard pan activation paint invalidation and stop-propagation side effects behind
      existing retained-agnostic `widget_tail` seams.
    - Extend the default source-policy gate so keyboard pan activation cannot re-import retained
      bridge Cx names.
  - Result:
    - Moved `keyboard_pan_activation.rs` off direct retained `EventCx` signatures.
    - Reused `WidgetHandledCx` for key-down stop-propagation plus paint invalidation.
    - Reused `WidgetPaintInvalidationCx` for key-up paint invalidation.
    - Added `keyboard_pan_activation_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas keyboard_pan_activation_stays_off_retained_bridge space_to_pan_starts_left_mouse_panning_and_updates_viewport pan_activation_key_code_must_match_to_enable_space_to_pan pan_activation_key_code_none_disables_space_to_pan_activation space_enables_pan_on_scroll_even_when_pan_on_scroll_is_disabled retained_bridge_source_usage_stays_on_the_migration_ledger retained_widget_compat_island_stays_crate_private_and_controller_bound`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/keyboard_pan_activation.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-390 Isolate feedback/motion retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/clipboard_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move clipboard feedback host/window/paint invalidation and timer-motion paint invalidation
      behind retained-agnostic seams.
    - Extend source-policy coverage so feedback/motion helpers cannot re-import retained bridge Cx
      names.
    - Backfill behavior coverage for clipboard-unavailable feedback so adapter isolation does not
      silently drop toast, timer, redraw, or paint invalidation side effects.
  - Result:
    - Added `ClipboardFeedbackCx` as the retained-agnostic feedback seam and kept retained
      `EventCx` usage isolated in `event_clipboard_feedback_retained_cx.rs`.
    - Moved `request_paste_feedback(...)`, `show_clipboard_unavailable_toast(...)`, and
      `invalidate_motion(...)` off direct retained bridge signatures.
    - Added `feedback_motion_helpers_stay_off_retained_bridge` source-policy coverage.
    - Added clipboard-unavailable behavior tests for matching and stale paste tokens.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(feedback_motion_helpers_stay_off_retained_bridge) | test(clipboard_unavailable_with_matching_token_shows_toast_and_invalidates_paint) | test(clipboard_unavailable_with_stale_token_has_no_feedback_side_effects) | test(pan_inertia_emits_move_end_after_inertia_stops) | test(wheel_zoom_emits_move_start_and_debounced_move_end) | test(pinch_zoom_emits_move_start_and_debounced_move_end) | test(wheel_pan_emits_move_start_and_debounced_move_end) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-400 Isolate toast timer retained Cx adapter.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_timer_toast.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/toast_timer_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move toast timer paint invalidation behind retained-agnostic `WidgetPaintInvalidationCx`.
    - Extend source-policy coverage so toast timer helpers cannot re-import retained bridge Cx
      names.
    - Backfill behavior coverage for matching and stale toast timer ticks.
  - Result:
    - `event_timer_toast.rs` no longer imports or names retained bridge Cx types.
    - Matching toast timer ticks clear the toast, request redraw, and invalidate paint through the
      retained-agnostic widget tail seam.
    - Stale toast timer ticks leave the toast and feedback side effects untouched.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(feedback_motion_helpers_stay_off_retained_bridge) | test(matching_toast_timer_clears_toast_and_invalidates_paint) | test(stale_toast_timer_keeps_toast_without_feedback_side_effects) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_timer_toast.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard_feedback_cx.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_shared.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-410 Remove unused pending resize retained Cx parameter.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/pending_resize_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Delete the unused retained `EventCx` parameter from pending node resize move handling instead
      of wrapping it in another adapter.
    - Extend source-policy coverage so pending node resize move cannot re-import retained bridge Cx
      names.
    - Backfill handler behavior coverage for below-threshold and activation paths.
  - Result:
    - `pending_resize.rs` no longer imports or names retained bridge Cx types.
    - Pending node resize move dispatch now calls the handler without a retained Cx parameter.
    - Added direct handler tests proving below-threshold moves stay pending and above-threshold
      moves activate node resize.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pending_node_resize_move_stays_off_retained_bridge) | test(pending_node_resize_move_below_threshold_keeps_pending_resize) | test(pending_node_resize_move_past_threshold_activates_resize) | test(should_activate_pending_node_resize_respects_threshold) | test(activate_pending_node_resize_moves_pending_into_active) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-420 Isolate edge double-click finish retained Cx adapter.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/finish.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move edge double-click finish stop-propagation plus paint invalidation behind the existing
      retained-agnostic `WidgetHandledCx` seam.
    - Extend source-policy coverage so edge double-click finish cannot re-import retained bridge Cx
      names.
  - Result:
    - `pointer_down_double_click_edge/finish.rs` no longer imports or names retained bridge Cx
      types.
    - Added a local tail test proving finish stops propagation, requests redraw, and invalidates
      paint.
    - Existing edge double-click reroute and insert-picker gesture tests remain green.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(double_click_edge_inserts_reroute_when_enabled) | test(alt_double_click_edge_opens_insert_node_picker) | test(alt_double_click_edge_prefers_picker_over_reroute_when_both_enabled) | test(edge_double_click_finish_stays_off_retained_bridge) | test(finish_double_click_stops_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/finish.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-430 Isolate searcher dismiss tail retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear/tests.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher dismiss release-capture, finish, and paint invalidation tails behind existing
      retained-agnostic `widget_tail` seams.
    - Extend source-policy coverage so searcher dismiss tail helpers cannot re-import retained
      bridge Cx names.
  - Result:
    - `searcher_activation_state/clear.rs`, `searcher_ui.rs`, and `searcher_ui/event.rs` no
      longer import or name retained bridge Cx types.
    - Added focused tests proving searcher dismiss clears overlay/pending drag state and releases
      capture without adding paint side effects at the dismiss layer.
    - Added focused tests proving searcher paint invalidation and handled finish still request
      redraw/paint invalidation and stop propagation.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(clear_pending_searcher_row_drag_reports_and_clears_state) | test(clear_searcher_overlay_clears_searcher_and_pending_drag) | test(dismiss_searcher_overlay_clears_state_and_releases_capture_without_painting) | test(invalidate_searcher_paint_requests_redraw_and_paint_invalidation) | test(finish_searcher_event_stops_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-440 Isolate searcher row-drag release retained Cx adapter.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release/tests.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher row-drag release activation, dismiss, release-capture, and finish coordination
      behind a retained-agnostic `SearcherReleaseCx` seam.
    - Keep retained `EventCx` row activation as an adapter-only implementation.
    - Extend source-policy coverage so `release.rs` cannot re-import retained bridge Cx names.
  - Result:
    - `searcher_activation_state/release.rs` no longer imports or names retained bridge Cx types.
    - Added `release_retained_cx.rs` as the only retained adapter for row activation during
      searcher release.
    - Added focused tests for no-pending-drag side-effect-free release, row activation release, and
      outside dismiss release.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_release_without_pending_drag_is_side_effect_free) | test(searcher_release_on_row_activates_and_finishes) | test(searcher_release_outside_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-450 Isolate searcher row-drag arm retained Cx adapter.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm/tests.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher row-drag arm pointer id, tick id, and pointer capture access behind a
      retained-agnostic `SearcherArmCx` seam.
    - Keep retained `EventCx` pointer/timer/capture access as an adapter-only implementation.
    - Extend source-policy coverage so `arm.rs` cannot re-import retained bridge Cx names.
  - Result:
    - `searcher_activation_state/arm.rs` no longer imports or names retained bridge Cx types.
    - Added `arm_retained_cx.rs` as the only retained adapter for searcher row-drag arming.
    - Added focused tests for unselectable-row no-side-effect behavior and selectable-row pending
      drag/capture behavior.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(arm_searcher_row_drag_rejects_unselectable_row_without_side_effects) | test(arm_searcher_row_drag_records_pending_drag_and_captures_pointer) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-460 Isolate searcher pointer-down retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher pointer-down routing off direct retained bridge Cx names by composing the
      retained-agnostic searcher arm/dismiss/finish seams.
    - Extend source-policy coverage so `pointer_down.rs` cannot re-import retained bridge Cx
      names.
  - Result:
    - `searcher_activation/pointer_down.rs` no longer imports or names retained bridge Cx types.
    - Added `SearcherPointerDownCx` as the narrow combined capability required by pointer-down
      routing.
    - Added focused tests for no-searcher side-effect-free behavior, row arm/finish, left outside
      dismiss/finish, and secondary-button dismiss/finish.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_down_without_searcher_is_side_effect_free) | test(searcher_left_pointer_down_on_row_arms_drag_and_finishes) | test(searcher_left_pointer_down_outside_dismisses_and_finishes) | test(searcher_secondary_pointer_down_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-470 Isolate searcher pointer-up retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher pointer-up routing off direct retained bridge Cx names by reusing the
      retained-agnostic searcher release seam.
    - Extend source-policy coverage so `pointer_up.rs` cannot re-import retained bridge Cx names.
  - Result:
    - `searcher_activation/pointer_up.rs` no longer imports or names retained bridge Cx types.
    - Pointer-up routing now takes `SearcherReleaseCx` directly and keeps hit-specific release
      completion in a focused helper for behavior tests.
    - Added focused tests for non-left button ignore, no-searcher pending-drag cleanup, row
      activation/finish, and outside dismiss/finish.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_up_ignores_non_left_button) | test(searcher_pointer_up_without_searcher_clears_pending_drag_only) | test(searcher_pointer_up_on_row_activates_and_finishes) | test(searcher_pointer_up_outside_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-480 Isolate outer searcher activation wrappers from retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the outer searcher activation wrapper off direct retained bridge Cx names after
      pointer-down and pointer-up helpers became retained-agnostic.
    - Extend source-policy coverage so `searcher_activation.rs` cannot re-import retained bridge
      Cx names.
  - Result:
    - `searcher_activation.rs` no longer imports or names retained bridge Cx types.
    - The pointer-down wrapper now takes the combined `SearcherPointerDownCx` capability, and the
      pointer-up wrapper now takes `SearcherReleaseCx` directly.
    - `SearcherPointerDownCx` is visible to the parent widget module while retained `EventCx`
      support remains only through existing retained adapter implementations.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_down_without_searcher_is_side_effect_free) | test(searcher_pointer_up_ignores_non_left_button) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-490 Isolate searcher pointer move/wheel retained Cx routes.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event/tests.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher pointer move/wheel routing off direct retained bridge Cx names by using the
      retained-agnostic paint invalidation seam.
    - Extend source-policy coverage so `searcher_pointer.rs`, `move_event.rs`, and
      `wheel_event.rs` cannot re-import retained bridge Cx names.
  - Result:
    - Searcher pointer move and wheel routes now take `WidgetPaintInvalidationCx`.
    - Added focused tests for no-searcher move/wheel side-effect-free behavior, hover update paint
      invalidation, repeated hover no-op behavior, wheel scroll paint invalidation, boundary wheel
      consumption without paint, and Ctrl-wheel pass-through.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_pointer_move_without_searcher_is_side_effect_free) | test(searcher_pointer_move_updates_hover_and_invalidates_paint) | test(searcher_pointer_move_same_hover_does_not_invalidate_paint_again) | test(searcher_wheel_without_searcher_is_side_effect_free) | test(searcher_wheel_scrolls_and_invalidates_paint) | test(searcher_wheel_at_scroll_boundary_consumes_plain_wheel_without_paint) | test(searcher_wheel_with_ctrl_does_not_consume_or_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-500 Isolate searcher key-down retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/activation_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch/tests.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher key-down routing off direct retained bridge Cx names by introducing a narrow
      retained-agnostic `SearcherInputCx` seam.
    - Keep retained row activation I/O as an adapter-only implementation.
    - Extend source-policy coverage so searcher key routing helpers cannot re-import retained
      bridge Cx names.
  - Result:
    - Searcher key dispatch now takes `SearcherInputCx`, which combines handled finish behavior
      with row activation I/O.
    - Added `searcher_input/activation_retained_cx.rs` as the only retained adapter for key-route
      row activation.
    - Added focused tests for Enter activation/finish, ArrowDown navigation/finish, query update,
      Ctrl text pass-through, and no-searcher no-op behavior.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(searcher_enter_activates_active_row_and_finishes) | test(searcher_arrow_down_steps_active_row_and_finishes) | test(searcher_text_key_updates_query_and_finishes) | test(searcher_ctrl_text_key_is_not_handled) | test(searcher_key_without_searcher_is_side_effect_free) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-510 Isolate the top-level searcher retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the top-level searcher escape/key/pointer/wheel route wrapper off direct retained bridge
      Cx names by introducing a narrow `SearcherCx` capability composed from the existing
      retained-agnostic searcher seams.
    - Keep retained pointer/timer/capture, row activation, and widget-tail I/O in the existing
      adapter-only implementations.
    - Extend source-policy coverage so `searcher.rs` cannot re-import retained bridge Cx names.
  - Result:
    - `searcher.rs` now routes Escape, key down, pointer down/up/move, and wheel through
      `impl SearcherCx` instead of `fret_ui::retained_bridge::EventCx`.
    - `SearcherCx` composes `SearcherPointerDownCx`, `SearcherReleaseCx`, and `SearcherInputCx`;
      retained `EventCx` satisfies it only through the existing adapter implementations.
    - Added focused top-level route tests for Escape dismiss/finish, Enter row activation through
      the input seam, and pointer-down row drag arming without retained Cx types.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_top_level_route_stays_off_retained_bridge) | test(searcher_top_level_escape_dismisses_and_finishes) | test(searcher_top_level_key_down_delegates_to_activation_seam) | test(searcher_top_level_pointer_down_arms_row_drag_without_retained_cx) | test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/arm.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/clear.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_activation_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_input_query.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_pointer/wheel_event.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-520 Isolate context menu UI retained Cx tails.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event_retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move context menu UI open/restore/dismiss/finish/invalidate helpers off direct retained
      bridge Cx names by using existing widget-tail seams plus a narrow context-menu focus seam.
    - Keep retained focus-self I/O as an adapter-only implementation.
    - Extend source-policy coverage so context menu UI tail helpers cannot re-import retained
      bridge Cx names.
  - Result:
    - `context_menu/ui.rs` and `context_menu/ui/event.rs` now use retained-agnostic
      `WidgetHandledCx`, `WidgetPaintInvalidationCx`, and `ContextMenuFocusCx`.
    - Added `context_menu/ui/event_retained_cx.rs` as the only retained adapter for context menu
      focus-self I/O.
    - Added focused tests for open/focus/finish, restore/finish without focus,
      dismiss/finish, no-menu dismiss no-op behavior, and paint invalidation.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_ui_tail_stays_off_retained_bridge) | test(open_context_menu_event_installs_menu_focuses_and_finishes) | test(restore_context_menu_event_restores_menu_and_finishes_without_focus) | test(dismiss_context_menu_event_clears_menu_and_finishes) | test(dismiss_context_menu_event_without_menu_is_side_effect_free) | test(invalidate_context_menu_paint_requests_redraw_and_paint_invalidation) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-530 Isolate context menu pointer-move retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/pointer_move.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move context menu pointer-move routing off direct retained bridge Cx names by using the
      retained-agnostic paint invalidation seam.
    - Extend source-policy coverage so the pointer-move helper cannot re-import retained bridge Cx
      names.
  - Result:
    - Context menu pointer-move routing now takes `WidgetPaintInvalidationCx`.
    - Added focused tests for no-menu no-op behavior, hover update paint invalidation, and repeated
      hover no-op invalidation behavior.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_pointer_move_route_stays_off_retained_bridge) | test(pointer_move_without_context_menu_is_side_effect_free) | test(pointer_move_updates_hover_and_invalidates_paint) | test(pointer_move_same_hover_does_not_invalidate_paint_again) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/pointer_move.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-540 Isolate context menu key-down retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move context menu key-down routing off direct retained bridge Cx names by introducing a
      retained-agnostic `ContextMenuKeyDownCx` capability.
    - Keep retained active-selection activation I/O in an adapter-only implementation.
    - Extend source-policy coverage so key navigation cannot re-import retained bridge Cx names.
  - Result:
    - `key_navigation.rs` and `key_navigation/key_down.rs` now route through
      `ContextMenuKeyDownCx`.
    - Added focused tests for no-menu no-op behavior, ArrowDown navigation/finish, Enter
      activation/close, Enter keep-open restore, typeahead, and Backspace typeahead pop behavior.
    - Superseded by `RBX-M2-550`: retained active-selection activation I/O now lives in the shared
      `selection_activation/retained_cx.rs` adapter rather than a key-down-specific adapter.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_key_down_route_stays_off_retained_bridge) | test(key_down_without_context_menu_is_side_effect_free) | test(key_down_arrow_down_advances_active_item_and_finishes) | test(key_down_enter_activates_active_item_and_closes_menu) | test(key_down_enter_keep_open_restores_menu_and_finishes) | test(key_down_typeahead_updates_active_item_and_finishes) | test(key_down_backspace_pops_typeahead_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-550 Isolate context menu selection activation and pointer-down retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/tests.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/pointer_down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move shared context menu active-selection and indexed-selection activation off direct retained
      bridge Cx names by introducing `ContextMenuSelectionActivationCx`.
    - Move context menu pointer-down routing off direct retained bridge Cx names by composing
      `ContextMenuPointerDownCx` from handled widget-tail behavior and selection activation.
    - Replace the key-down-specific retained activation adapter with the shared selection activation
      retained adapter.
  - Result:
    - `selection_activation.rs` and `selection_activation/pointer_down.rs` now route through
      retained-agnostic selection activation and pointer-down seams.
    - `selection_activation/retained_cx.rs` is the retained `EventCx` adapter for actual context
      menu item execution.
    - `key_navigation.rs` now composes `ContextMenuKeyDownCx` from `WidgetHandledCx` and
      `ContextMenuSelectionActivationCx`; `key_navigation/key_down_retained_cx.rs` was removed.
    - Added focused tests for pointer-down no-menu no-op behavior, left enabled-item activation and
      close, left disabled-item restore, left outside-menu close, and right-button replacement-menu
      pass-through behavior.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_selection_activation_route_stays_off_retained_bridge) | test(context_menu_key_down_route_stays_off_retained_bridge) | test(pointer_down_without_context_menu_is_side_effect_free) | test(pointer_down_left_inside_enabled_item_activates_and_closes_menu) | test(pointer_down_left_disabled_item_restores_menu_and_finishes) | test(pointer_down_left_outside_menu_closes_menu_and_finishes) | test(pointer_down_right_button_leaves_menu_taken_and_unfinished) | test(key_down_enter_activates_active_item_and_closes_menu) | test(key_down_enter_keep_open_restores_menu_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/selection_activation/pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/key_navigation/key_down.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-560 Isolate context menu top-level retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/input.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/pointer.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the context menu top-level escape/key/pointer route wrappers off direct retained bridge
      Cx names by composing the existing key-down and pointer-down seams through `ContextMenuCx`.
    - Move `context_menu/input.rs` and `context_menu/pointer.rs` wrapper signatures onto the
      narrow retained-agnostic seams they actually need.
    - Lock the top-level context menu route files with source-policy coverage.
  - Result:
    - `context_menu/mod.rs` now composes top-level routing through `ContextMenuCx`, mirroring the
      searcher top-level route pattern.
    - `context_menu/input.rs` now takes `WidgetHandledCx` for Escape and `ContextMenuKeyDownCx` for
      key-down routing.
    - `context_menu/pointer.rs` now takes `ContextMenuPointerDownCx` for pointer-down routing and
      `WidgetPaintInvalidationCx` for pointer-move routing.
    - Added focused top-level tests proving Escape dismiss/finish, Enter active-item activation,
      pointer-down item activation, and pointer-move hover/invalidation without retained Cx types.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_top_level_route_stays_off_retained_bridge) | test(context_menu_top_level_escape_dismisses_and_finishes) | test(context_menu_top_level_key_down_delegates_to_activation_seam) | test(context_menu_top_level_pointer_down_delegates_to_selection_activation) | test(context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint) | test(context_menu_selection_activation_route_stays_off_retained_bridge) | test(context_menu_key_down_route_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/input.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/pointer.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-570 Isolate context menu opening retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/background.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/edge.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/group.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/ui/event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/overlay_menu_searcher_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move context menu opening route helpers off direct retained bridge Cx names by introducing
      `ContextMenuOpeningCx` for host, bounds, window availability, focus, and handled finish
      capabilities.
    - Keep retained `EventCx` field access isolated in `context_menu/opening/retained_cx.rs`.
    - Lock the opening route files with source-policy coverage and prove background, group, and
      edge right-click menu behavior still opens the correct menu and selection target.
  - Result:
    - `context_menu/opening.rs` now routes through `ContextMenuOpeningCx`; background, group, and
      edge opening helpers no longer name retained bridge Cx types.
    - `ContextMenuFocusCx` visibility was raised from `context_menu`-internal to `widget`-internal
      so opening can compose focus behavior without widening public API.
    - Added `opening/retained_cx.rs` as the retained adapter for host/bounds/window/focus I/O.
    - Added retained-path regression tests for background, group, and edge right-click opening.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_opening_route_stays_off_retained_bridge) | test(right_click_background_opens_background_context_menu_with_paste_disabled_without_window) | test(right_click_group_opens_group_context_menu_and_selects_group) | test(right_click_edge_opens_edge_context_menu_and_selects_edge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/background.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/edge.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/opening/group.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-580 Isolate context menu action activation retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/command.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/target.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the context menu item action activation route off direct retained bridge Cx names by
      introducing a retained-agnostic `ContextMenuActionCx` seam for command dispatch, target
      selection, and target-specific action execution.
    - Keep retained `EventCx` command dispatch and target executor calls isolated in
      `context_menu/activate/retained_cx.rs`.
    - Lock activation route files with source-policy coverage and prove command, target, and
      ignored action dispatch behavior.
  - Result:
    - `context_menu/activate.rs`, `context_menu/activate/command.rs`, and
      `context_menu/activate/target.rs` now route through `ContextMenuActionCx` plus narrower
      command/target action seams.
    - Added `activate/retained_cx.rs` as the retained adapter for command dispatch, group
      selection sync, and existing background/edge/connection target executors.
    - Added focused tests for group command selection-before-dispatch, non-command target action
      delegation, and ignored target actions.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_activation_route_stays_off_retained_bridge) | test(command_items_select_group_before_dispatching_command) | test(non_command_items_delegate_to_target_action_executor) | test(ignored_target_actions_are_side_effect_free) | test(pointer_down_left_inside_enabled_item_activates_and_closes_menu) | test(key_down_enter_activates_active_item_and_closes_menu) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/command.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/activate/target.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-590 Isolate context menu background execution retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/activate.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/apply.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/tests.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move background insert context-menu execution off direct retained bridge Cx names by
      introducing `BackgroundInsertMenuCx` for host/window access.
    - Keep retained `EventCx` host/window field access isolated in
      `context_menu/background_execution/retained_cx.rs`.
    - Lock background execution files with source-policy coverage and prove candidate gating,
      ignored action, and rejection-toast behavior.
  - Result:
    - `background_execution.rs`, `background_execution/activate.rs`, and
      `background_execution/apply.rs` now use `BackgroundInsertMenuCx`.
    - Added `background_execution/retained_cx.rs` as the retained adapter for host/window access.
    - Added focused tests for missing candidate handled/no-op behavior, non-candidate action
      ignored behavior, and candidate rejection toast plus recent-kind recording.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_background_execution_stays_off_retained_bridge) | test(background_insert_menu_plan_surfaces_create_node_errors) | test(background_insert_action_with_missing_candidate_is_handled_without_side_effects) | test(background_insert_action_ignores_non_candidate_actions) | test(background_insert_action_records_candidate_and_surfaces_rejection_toast) | test(context_menu_activation_route_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/background_execution/apply.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-600 Isolate context menu edge execution retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/open_insert.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/reroute.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/delete.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/custom_action.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move edge context-menu execution off direct retained bridge Cx names by introducing
      `EdgeContextActionCx` for host/window access and edge insert menu opening.
    - Keep retained `EventCx` host/window/open-insert field access isolated in
      `context_menu/edge_execution/retained_cx.rs`.
    - Lock edge execution files with source-policy coverage and prove open-insert, reroute,
      delete, custom action, and ignored-action behavior.
  - Result:
    - `edge_execution.rs` plus its open-insert/reroute/delete/custom helpers now use
      `EdgeContextActionCx`.
    - Added `edge_execution/retained_cx.rs` as the retained adapter for host/window/open-insert
      access.
    - Added focused tests for edge insert menu delegation, edge deletion and selection cleanup,
      reroute insertion, custom presenter ops, and ignored non-edge actions.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_edge_execution_stays_off_retained_bridge) | test(open_insert_action_delegates_to_context_adapter) | test(delete_edge_action_removes_edge_and_selection) | test(insert_reroute_action_splits_edge_and_selects_inserted_node) | test(custom_edge_action_applies_presenter_ops) | test(ignored_edge_actions_are_side_effect_free) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/open_insert.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/reroute.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/delete.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/edge_execution/custom_action.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-610 Isolate context menu connection insert execution retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/activate.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/apply.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move connection insert context-menu execution off direct retained bridge Cx names by
      introducing `ConnectionInsertMenuCx` for host/window access and wire-drag recovery.
    - Keep retained `EventCx` host/window/capture/recovery field access isolated in
      `context_menu/connection_execution_insert/retained_cx.rs`.
    - Lock connection insert execution files with source-policy coverage and prove candidate
      gating, rejection restore, successful resume, and ignore restore behavior.
  - Result:
    - `connection_execution_insert.rs` plus its activate/apply/recovery helpers now use
      `ConnectionInsertMenuCx`.
    - Added `connection_execution_insert/retained_cx.rs` as the retained adapter for
      host/window/wire-drag recovery.
    - Added focused tests for missing candidate handled/no-op behavior, non-candidate ignored
      behavior, rejected candidate toast plus restore, successful resume, and ignore restore.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_connection_insert_execution_stays_off_retained_bridge) | test(connection_insert_action_with_missing_candidate_is_handled_without_side_effects) | test(connection_insert_action_ignores_non_candidate_actions) | test(connection_insert_action_records_candidate_and_restores_on_rejection) | test(connection_insert_apply_success_resumes_wire_drag) | test(connection_insert_apply_ignore_restores_wire_drag) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/apply.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_insert/recovery.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-620 Isolate context menu connection conversion execution retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move connection conversion context-menu execution off direct retained bridge Cx names by
      introducing `ConnectionConversionMenuCx` for host/window access and wire-drag restoration.
    - Keep retained `EventCx` host/window/capture/recovery field access isolated in
      `context_menu/connection_execution_conversion/retained_cx.rs`.
    - Lock connection conversion execution files with source-policy coverage and prove candidate
      gating, rejection restore, successful conversion apply/selection, and ignore restore
      behavior.
  - Result:
    - `connection_execution_conversion.rs` plus its activate/apply helpers now use
      `ConnectionConversionMenuCx`.
    - Added `connection_execution_conversion/retained_cx.rs` as the retained adapter for
      host/window/wire-drag recovery.
    - Added focused tests for missing candidate handled/no-op behavior, non-candidate ignored
      behavior, rejected candidate toast plus restore, successful apply clearing suspended drag and
      selecting the inserted node, and ignore restore.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(context_menu_connection_conversion_execution_stays_off_retained_bridge) | test(connection_conversion_action_with_missing_candidate_is_handled_without_side_effects) | test(connection_conversion_action_ignores_non_candidate_actions) | test(connection_conversion_action_records_candidate_and_restores_on_rejection) | test(connection_conversion_apply_success_clears_suspended_drag_and_selects_node) | test(connection_conversion_apply_ignore_restores_wire_drag) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/activate.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/connection_execution_conversion/apply.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-630 Isolate searcher row activation retained Cx route.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_logic.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation/retained_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move searcher row activation off direct retained bridge Cx names by introducing
      `SearcherRowActivationCx` for the context-menu item activation side effect.
    - Keep retained `EventCx` context-menu activation I/O isolated in
      `searcher_row_activation/retained_cx.rs`.
    - Lock the searcher row activation route with source-policy coverage and prove no-searcher,
      unactivatable-row restore, and candidate-row delegation behavior.
  - Result:
    - `searcher_row_activation.rs` now takes `SearcherRowActivationCx` instead of retained
      `EventCx`.
    - Added `searcher_row_activation/retained_cx.rs` as the retained adapter that delegates to the
      existing context-menu action seam.
    - Added focused tests for side-effect-free no-searcher activation, header/disabled-row
      restoration, and candidate row context-action delegation.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(searcher_row_activation_route_stays_off_retained_bridge) | test(searcher_row_activation_without_searcher_is_side_effect_free) | test(searcher_row_activation_restores_unactivatable_row) | test(searcher_row_activation_delegates_candidate_item_to_context_action) | test(searcher_dismiss_tail_helpers_stay_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/searcher_row_activation.rs ecosystem/fret-node/src/ui/canvas/widget/searcher_logic.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-640 Isolate right-click pending context-menu route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/right_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/right_click/pending.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the right-click context-menu pointer-down/up route off direct retained bridge Cx names
      by introducing `RightClickCx`, composed from the existing `ContextMenuOpeningCx` and
      `PointerCaptureReleaseCx` capabilities.
    - Lock the right-click route files with source-policy coverage.
    - Prove pending right-click release planning plus retained right-click context-menu behavior.
  - Result:
    - `right_click.rs` and `right_click/pending.rs` now use `RightClickCx` instead of retained
      `EventCx` signatures.
    - Pending right-click pointer-up routing is split into a small retained-agnostic plan that
      distinguishes ignored, release-only, and release-plus-open-menu outcomes.
    - The retained adapter remains the existing composition of `ContextMenuOpeningCx` and
      `PointerCaptureReleaseCx` implementations for retained `EventCx`; no new retained Cx file
      was needed.
    - Added focused tests for non-right-button ignore, missing-pending no-op, drag release
      clearing pending state, and click release requesting menu open.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(right_click_route_stays_off_retained_bridge) | test(pending_right_click_pointer_up_ignores_non_right_button) | test(pending_right_click_pointer_up_without_pending_state_is_side_effect_free) | test(pending_right_click_drag_release_clears_pending_and_releases_capture) | test(pending_right_click_click_release_requests_menu_open) | test(right_click_cancels_wire_drag_and_opens_context_menu) | test(right_pan_defers_context_menu_until_pointer_up) | test(right_pan_drag_does_not_open_context_menu) | test(right_click_background_opens_background_context_menu_with_paste_disabled_without_window) | test(right_click_group_opens_group_context_menu_and_selects_group) | test(right_click_edge_opens_edge_context_menu_and_selects_edge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/right_click.rs ecosystem/fret-node/src/ui/canvas/widget/right_click/pending.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-650 Isolate pointer-up guard dispatch retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pointer-up guard arbitration off direct retained bridge Cx names by introducing
      `PointerUpGuardCx`, composed from the existing right-click and searcher seams.
    - Keep the retained fallback pointer-up route in the retained caller layer until the full
      pointer-up commit/release path is migrated.
    - Lock the guard dispatch file with source-policy coverage and prove right-click/searcher guard
      behavior still passes.
  - Result:
    - `event_pointer_up/dispatch.rs` now only dispatches guard paths through
      `PointerUpGuardCx`.
    - `event_pointer_up.rs` calls the retained fallback `pointer_up::handle_pointer_up(...)`
      directly after guards decline, keeping the retained full pointer-up path explicit.
    - Added `pointer_up_guard_dispatch_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(right_click_route_stays_off_retained_bridge) | test(pending_right_click_click_release_requests_menu_open) | test(right_pan_defers_context_menu_until_pointer_up) | test(searcher_pointer_up_on_row_activates_and_finishes) | test(searcher_pointer_up_without_searcher_clears_pending_drag_only) | test(searcher_pointer_up_outside_dismisses_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
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
