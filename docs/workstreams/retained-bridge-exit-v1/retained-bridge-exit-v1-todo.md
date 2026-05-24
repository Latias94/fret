# Retained Bridge Exit Plan v1 — TODO Tracker

Status: Closed for runtime retained-bridge exit (fearless refactor friendly; pre-1.0)

Related plan:

- `docs/workstreams/retained-bridge-exit-v1/retained-bridge-exit-v1.md`

## Milestones

### M0 — Governance gates (blast radius control)

- [x] CI: reject `crates/* -> ecosystem/*` reverse dependencies (`tools/check_layering.py`).
- [x] CI: restrict `fret-ui/unstable-retained-bridge` to an explicit allowlist (`tools/check_layering.py`).
- [x] Document the current allowlist and rationale per crate.
  - Source of truth: `tools/check_layering.py`
    (`unstable_retained_bridge_dependency_allowlist` and
    `unstable_retained_bridge_feature_mapping_allowlist`).
  - Direct dependency feature allowlist:
    - Empty. No workspace crate may enable `fret-ui/unstable-retained-bridge` directly from its
      `fret-ui` dependency entry.
  - Explicit compatibility feature mapping allowlist (workspace crate names/features):
    - Empty. No workspace crate feature may map to `fret-ui/unstable-retained-bridge`.
  - Removed from allowlist:
    - `fret-node`
      - Result: removed in `RBX-M4-043`; `fret-node/compat-retained-canvas` remains a node-local
        legacy implementation gate but maps only to `fret-ui`, not to the deleted retained bridge.
    - `fret-docking`
      - Result: removed in `RBX-M1-080`; docking now uses public declarative dock-space entry
        points and no longer depends on `fret-ui/unstable-retained-bridge`.
    - `fret-plot3d`
      - Result: removed in `RBX-M3-010`; Plot3D now exposes a declarative viewport-surface panel
        and the first-party Plot3D demos mount it through `declarative::render_root(...)`.
    - `fret-plot`
      - Result: removed in `RBX-M3-316`; `compat-retained-canvas = []` is now a no-op transition
        alias and the crate root no longer compiles retained plot source.
    - `fret-chart`
      - Result: removed in `RBX-M4-032`; `compat-retained-canvas = []` is now a no-op transition
        alias and the crate root no longer compiles retained chart source.

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
- [x] Add/upgrade `fretboard-dev diag` scripts to lock in docking drag + tear-off correctness.
  - Result:
    - Added the lightweight multi-window tear-off smoke to `diag-hardening-smoke-docking`, so the promoted registry keeps a minimal tear-off gate alongside the existing drag/merge coverage.
  - Validation:
    - `cargo run -p fretboard-dev -- diag registry check`
    - `python3 tools/check_diag_scripts_registry.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`

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
- [x] RBX-M2-261 Isolate sticky-wire non-port target retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/retained_picker_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move `handle_sticky_wire_non_port_target(...)` off a direct retained `EventCx` signature.
    - Reuse the sticky-wire target picker seam for host/window, pointer-capture release, and
      handled-event tail behavior.
    - Keep the retained `EventCx` implementation in the dedicated retained adapter module.
  - Result:
    - `sticky_wire_targets.rs` now accepts `impl StickyWireTargetPickerCx<H>` and reads host access
      through `cx.host()` instead of retained `cx.app`.
    - `StickyWireTargetPickerCx` now composes `PointerCaptureReleaseCx + WidgetHandledCx`, allowing
      non-port target handling to release pointer capture without naming retained bridge Cx types.
    - Extended `retained_canvas_tail_policy_helpers_stay_off_retained_bridge` to include
      `sticky_wire_targets.rs` in addition to `sticky_wire_targets/picker.rs`.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs ecosystem/fret-node/src/lib.rs`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(finish_sticky_wire_target_picker_stops_and_invalidates_paint) | test(retained_canvas_tail_policy_helpers_stay_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-262 Isolate sticky-wire pointer-down/connect retained Cx adapters.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/retained_picker_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the sticky-wire pointer-down and connect-target route wrappers off direct retained
      `EventCx` signatures.
    - Reuse `StickyWireTargetPickerCx` as the host/window + release/handled seam for both connected
      and non-port sticky-wire target paths.
    - Keep the retained `EventCx` implementation in the dedicated retained adapter module.
  - Result:
    - `sticky_wire.rs::handle_sticky_wire_pointer_down(...)` now accepts
      `impl StickyWireTargetPickerCx<H>` and calls `cx.host()` instead of retained `cx.app`.
    - `sticky_wire_connect.rs::handle_sticky_wire_connect_target(...)` now accepts the same seam and
      uses `cx.host()` / `cx.window()` for apply/toast side effects.
    - Added `sticky_wire_pointer_down_route_stays_off_retained_bridge` source-policy coverage for
      `sticky_wire.rs` and `sticky_wire_connect.rs`.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/ui/canvas/widget/sticky_wire.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs ecosystem/fret-node/src/lib.rs`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(sticky_wire_pointer_down_route_stays_off_retained_bridge) | test(finish_sticky_wire_target_picker_stops_and_invalidates_paint) | test(retained_canvas_tail_policy_helpers_stay_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_widget_compat_island_stays_crate_private_and_controller_bound)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/sticky_wire.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_connect.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets.rs ecosystem/fret-node/src/ui/canvas/widget/sticky_wire_targets/picker.rs || test $? -eq 1`
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
- [x] RBX-M2-660 Isolate pointer-up release retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up/release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_state/release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pointer-up sticky ignored release and pan release helpers off direct retained bridge Cx
      names.
    - Keep retained I/O explicit in a small `PointerUpReleaseCx` retained adapter while release
      route/state helpers use retained-agnostic capabilities.
    - Prove sticky-wire ignored release, pan inertia release, and right-pan context-menu behavior
      remain green.
  - Result:
    - Added `PointerUpReleaseCx` for host/window access plus pointer-capture release and paint
      invalidation.
    - `pointer_up/release.rs` and `pointer_up_state/release.rs` now use retained-agnostic release
      capabilities.
    - Added `pointer_up_release_route_stays_off_retained_bridge` source-policy coverage.
    - Added `sticky_wire_ignored_left_pointer_up_clears_ignore_and_invalidates_paint` behavior
      coverage.
  - Validation:
    - Red: `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_release_route_stays_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_release_route_stays_off_retained_bridge) | test(sticky_wire_ignored_left_pointer_up_clears_ignore_and_invalidates_paint) | test(pan_inertia_emits_move_end_after_inertia_stops) | test(right_pan_defers_context_menu_until_pointer_up) | test(right_pan_drag_does_not_open_context_menu) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_state/release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_release_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-670 Isolate pointer-up left double-click retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/double_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_insert_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the plain double-click edge-insert pointer-up subroute off direct retained bridge Cx
      names.
    - Reuse the retained-agnostic `PointerUpReleaseCx` seam for host/window access, pointer capture
      release, and paint invalidation.
    - Prove the real pointer-up path still opens the edge insert picker and invalidates paint.
  - Result:
    - `pointer_up_left_route/double_click.rs` now uses `PointerUpReleaseCx` instead of retained
      `EventCx`.
    - Added `pointer_up_left_double_click_route_stays_off_retained_bridge` source-policy coverage.
    - Added `plain_double_click_edge_insert_left_up_opens_picker_and_invalidates_paint` behavior
      coverage through `pointer_up::handle_pointer_up`.
  - Validation:
    - Red: `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_left_double_click_route_stays_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_left_double_click_route_stays_off_retained_bridge) | test(plain_double_click_edge_insert_left_up_opens_picker_and_invalidates_paint) | test(should_open_edge_insert_picker_requires_plain_double_click) | test(edge_insert_left_up_does_not_open_picker_when_searcher_is_open) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/double_click.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-680 Isolate pointer-up commit dispatch retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the pointer-up commit dispatch chain and node-drag commit release helper off direct
      retained bridge Cx names.
    - Reuse the existing retained-agnostic `PointerUpCommitCx` seam for host/window access,
      pointer capture release, and paint invalidation.
    - Prove node-drag release and group resize commit behavior remain green.
  - Result:
    - `pointer_up_commit.rs`, `pointer_up_node_drag.rs`, and
      `pointer_up_left_route/dispatch/commit.rs` now accept `PointerUpCommitCx` instead of naming
      retained `EventCx`.
    - Extended `pointer_up_commit_handlers_stay_off_retained_bridge` source-policy coverage to
      lock those files off retained bridge Cx names.
    - Retained `EventCx` adaptation stays isolated in `pointer_up_commit_retained_cx.rs`.
  - Validation:
    - Red: `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_commit_handlers_stay_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_commit_handlers_stay_off_retained_bridge) | test(node_drag_pointer_up_emits_node_drag_end_committed) | test(node_drag_end_batches_group_rect_ops_in_sorted_group_id_order) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(group_resize_clamps_to_children) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_commit.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_node_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/commit.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-690 Isolate pointer-up pending dispatch retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the left pointer-up pending release dispatch chain off direct retained bridge Cx names.
    - Reuse the existing retained-agnostic `PendingNodeDragReleaseCx` seam as the composed
      capability for pending node selection, pending group release, pending node resize release, and
      pending wire release.
    - Prove the real pointer-up path still completes pending group drag/resize release, pending
      node click-select release, and pending wire-drag promotion.
  - Result:
    - `pointer_up_left_route/dispatch/pending.rs` now accepts `PendingNodeDragReleaseCx` instead of
      naming retained `EventCx`.
    - Added `pointer_up_pending_dispatch_stays_off_retained_bridge` source-policy coverage.
    - Added real `pointer_up::handle_pointer_up` behavior coverage for pending node drag
      click-select release and pending wire-drag release promotion.
  - Validation:
    - Red: `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_pending_dispatch_stays_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_pending_dispatch_stays_off_retained_bridge) | test(pending_group_drag_release_clears_session_without_committing) | test(pending_group_resize_release_clears_session_without_committing) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(pending_wire_drag_release_promotes_to_active_wire_drag_and_finishes) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/pending.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-700 Isolate pointer-up active dispatch retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/active.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/active.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/pointer_up.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edge_drag_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the left pointer-up active release dispatch chain off direct retained bridge Cx names.
    - Migrate its direct edge-insert and edge-drag pointer-up leaf helpers behind retained-agnostic
      release capabilities so the source-policy gate covers a real leaf boundary.
    - Prove wire left-up, edge-insert left-up, and edge-drag left-up behavior remain green.
  - Result:
    - `pointer_up_left_route/dispatch/active.rs` now accepts `WireCommitCx + PointerUpReleaseCx`
      instead of naming retained `EventCx`.
    - `edge_insert_drag/pointer_up*.rs` now uses `PointerUpReleaseCx` /
      `PointerCaptureReleaseCx` for host/window access and pointer-up finish side effects.
    - `edge_drag/pointer_up.rs` now uses `PointerUpReleaseCx`.
    - Added `pointer_up_active_dispatch_stays_off_retained_bridge` source-policy coverage and a
      focused edge-drag left-up behavior test.
  - Validation:
    - Red: `cargo nextest run -p fret-node --features compat-retained-canvas pointer_up_active_dispatch_stays_off_retained_bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_active_dispatch_stays_off_retained_bridge) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_insert_left_up_does_not_open_picker_when_searcher_is_open) | test(plain_double_click_edge_insert_left_up_opens_picker_and_invalidates_paint) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch/active.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/active.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pointer_up/pending.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/pointer_up.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-710 Isolate pointer-up route wrappers retained Cx names.
  - Scope:
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
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the top-level pointer-up fallback route wrappers off direct retained bridge Cx names by
      composing the already-migrated release, commit, pending, wire, and marquee capabilities.
    - Finish the marquee move path that `marquee.rs` forwards into so the wrapper gate covers a
      real behavior path instead of only hiding the retained type name one frame higher.
    - Extract a narrow pan-begin Cx seam for marquee-to-pan promotion without widening the slice to
      the still-retained pan-move route.
    - Prove pointer-up, marquee selection, and pan-begin behavior remain green.
  - Result:
    - Added `PointerUpCx` as a composed capability over `PointerUpReleaseCx`,
      `PointerUpCommitCx`, `PendingNodeDragReleaseCx`, `WireCommitCx`, and `MarqueeCx`.
    - `pointer_up.rs`, `pointer_up/left.rs`, `pointer_up_left_route.rs`, and
      `pointer_up_left_route/dispatch.rs` now accept `PointerUpCx` instead of naming retained
      `EventCx`.
    - `marquee.rs`, `marquee_pending.rs`, and `marquee_selection.rs` now use `MarqueeCx` for host,
      capture, release, and paint invalidation side effects; the old `marquee_retained_cx.rs`
      adapter was deleted.
    - Added `PanZoomBeginCx` plus `pan_zoom_begin_retained_cx.rs`, and moved
      `pan_zoom_begin.rs` behind that seam. `pan_zoom.rs` still has a retained move wrapper and is
      intentionally not source-policy gated as a whole in this slice.
    - Added source-policy coverage for pointer-up route wrappers, marquee move handlers, and
      pan-zoom begin helpers.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(marquee_begin_finish_stays_off_retained_bridge) | test(marquee_move_handlers_stay_off_retained_bridge) | test(pan_zoom_begin_helpers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(marquee_replace_mode_replaces_selection_even_with_ctrl_pressed) | test(middle_mouse_panning_tracks_screen_delta_under_render_transform) | test(panning_emits_move_start_and_move_end) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up/left.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_up_left_route/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/marquee.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-720 Isolate pointer-up event entry retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the retained fallback pointer-up event entry off direct retained bridge Cx names by
      composing the existing guard and route capabilities.
    - Keep the upper pointer-event parser (`event_router_pointer_button/up.rs`) as a later route
      isolation slice so this task stays focused on the pointer-up handler entry.
    - Prove the real pointer-up path still handles guard dispatch, marquee release, pending node
      click-select release, edge reconnect release, and edge-drag left-up cleanup.
  - Result:
    - Added `PointerUpRouteCx` as the event-entry capability over `PointerUpGuardCx` and
      `PointerUpCx`.
    - `NodeGraphCanvasWith::handle_pointer_up(...)` now accepts the retained-agnostic composed
      capability instead of naming retained `EventCx`.
    - Widened `PointerUpGuardCx` to widget-internal visibility so the composed route seam does not
      trigger private-bound warnings.
    - Added `pointer_up_event_entry_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - Red: `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_event_entry_stays_off_retained_bridge) | test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_event_entry_stays_off_retained_bridge) | test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-730 Isolate pointer-up button router retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/up.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the `PointerEvent::Up` parser/forwarder off direct retained bridge Cx names by
      reusing `PointerUpRouteCx`.
    - Keep `event_router_pointer_button.rs` and its down/move branches as later route isolation
      slices.
    - Prove the real pointer-up route still handles guard dispatch, marquee release, pending node
      click-select release, edge reconnect release, and edge-drag left-up cleanup.
  - Result:
    - `event_router_pointer_button/up.rs` now accepts `PointerUpRouteCx` instead of naming retained
      `EventCx`.
    - Added `pointer_up_button_router_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_up_button_router_stays_off_retained_bridge) | test(pointer_up_event_entry_stays_off_retained_bridge) | test(pointer_up_guard_dispatch_stays_off_retained_bridge) | test(pointer_up_route_wrappers_stay_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(pending_node_drag_click_select_release_toggles_selection_and_finishes) | test(edge_reconnect_drop_on_empty_can_disconnect_edge) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/up.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_up/dispatch.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-740 Isolate pan-zoom move retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Split the pan-begin-only capture capability from the shared pan host/bounds/paint capability
      so the panning move path no longer names retained bridge Cx types.
    - Keep retained `EventCx` adaptation isolated in `pan_zoom_begin_retained_cx.rs`.
    - Prove middle-mouse panning, space-to-pan, move start/end callbacks, pan inertia end, and
      source-policy gates stay green.
  - Result:
    - Added `PanZoomCx` for host, bounds, and paint invalidation.
    - Kept `PanZoomBeginCx` as the begin-only extension for pointer capture.
    - `pan_zoom.rs` and `pan_zoom_move.rs` now accept retained-agnostic pan capabilities instead of
      naming retained `EventCx`.
    - Added `pan_zoom_move_helpers_stay_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pan_zoom_move_helpers_stay_off_retained_bridge) | test(pan_zoom_begin_helpers_stay_off_retained_bridge) | test(middle_mouse_panning_tracks_screen_delta_under_render_transform) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(panning_emits_move_start_and_move_end) | test(pan_inertia_emits_move_end_after_inertia_stops) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_begin_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-750 Isolate pointer-move primary surface retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/surface.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the primary surface pointer-move route off direct retained bridge Cx names by reusing the
      existing `MarqueeCx` capability.
    - Keep the surrounding primary pointer-move router plus group/node/connection branches as later
      route isolation slices.
    - Prove panning move and marquee move behavior remain green through the surface route.
  - Result:
    - `pointer_move_dispatch/primary/surface.rs` now accepts `MarqueeCx` instead of naming retained
      `EventCx`.
    - Because `MarqueeCx` includes `PanZoomBeginCx` and `PanZoomBeginCx` extends `PanZoomCx`, the
      surface route covers both pan move and marquee move without adding a duplicate seam.
    - Added `pointer_move_primary_surface_route_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_surface_route_stays_off_retained_bridge) | test(pan_zoom_move_helpers_stay_off_retained_bridge) | test(marquee_move_handlers_stay_off_retained_bridge) | test(middle_mouse_panning_tracks_screen_delta_under_render_transform) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(marquee_replace_mode_replaces_selection_even_with_ctrl_pressed) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/surface.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom.rs ecosystem/fret-node/src/ui/canvas/widget/pan_zoom_move.rs ecosystem/fret-node/src/ui/canvas/widget/marquee.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_pending.rs ecosystem/fret-node/src/ui/canvas/widget/marquee_selection.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-760 Isolate pointer-move primary group retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the primary group pointer-move route off direct retained bridge Cx names by composing the
      existing pending-group activation and group-preview move capabilities.
    - Keep the surrounding primary pointer-move router plus node/connection branches as later route
      isolation slices.
    - Prove pending group drag activation and group resize preview/commit behavior remain green.
  - Result:
    - `pointer_move_dispatch/primary/group.rs` now accepts
      `PendingGroupActivationCx + GroupPreviewMoveCx` instead of naming retained `EventCx`.
    - The route reuses existing seams from `RBX-M2-280` and `RBX-M2-290`; no duplicate group route
      seam was added.
    - Added `pointer_move_primary_group_route_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_group_route_stays_off_retained_bridge) | test(group_preview_move_handlers_stay_off_retained_bridge) | test(pending_group_activation_handlers_stay_off_retained_bridge) | test(group_header_click_selects_group_and_arms_pending_group_drag) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(group_resize_clamps_to_children) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/group.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_group_activation_cx.rs ecosystem/fret-node/src/ui/canvas/widget/group_drag.rs ecosystem/fret-node/src/ui/canvas/widget/group_resize.rs ecosystem/fret-node/src/ui/canvas/widget/group_preview_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-770 Isolate pointer-move primary node retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_drag_session/node.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_activation_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_activation_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the primary node pointer-move route off direct retained bridge Cx names by introducing a
      narrow `PendingNodeDragActivationCx` capability for host access plus pointer-capture release.
    - Keep retained `EventCx` adaptation isolated in `pending_node_drag_activation_retained_cx.rs`.
    - Keep pending node resize move on its existing retained-free path and leave the connection
      branch as a later route isolation slice.
    - Prove pending node drag activation/cancel, node drag threshold, and pending node resize move
      behavior remain green.
  - Result:
    - `pending_drag.rs` now accepts `PendingNodeDragActivationCx` instead of naming retained
      `EventCx`.
    - `pending_drag_session::abort_pending_node_drag(...)` now only needs the existing
      `PointerCaptureReleaseCx` tail seam.
    - `pointer_move_dispatch/primary/node.rs` now accepts `PendingNodeDragActivationCx` and routes
      pending node drag activation plus retained-free pending node resize move without retained Cx
      names.
    - Added `pending_node_drag_activation_handlers_stay_off_retained_bridge` and
      `pointer_move_primary_node_route_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_node_route_stays_off_retained_bridge) | test(pending_node_drag_activation_handlers_stay_off_retained_bridge) | test(pending_node_resize_move_stays_off_retained_bridge) | test(node_drag_does_not_start_when_nodes_draggable_is_false) | test(node_drag_start_and_escape_cancel_emits_node_drag_end_canceled) | test(node_drag_threshold_is_zoom_invariant_in_screen_space) | test(pending_node_resize_move_past_threshold_activates_resize) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/node.rs ecosystem/fret-node/src/ui/canvas/widget/pending_drag.rs ecosystem/fret-node/src/ui/canvas/widget/pending_node_drag_activation_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pending_resize.rs ecosystem/fret-node/src/ui/canvas/widget/pending_drag_session/node.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-780 Isolate pointer-move primary connection retained Cx names.
  - Scope:
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
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the primary connection pointer-move route off direct retained bridge Cx names by
      introducing a narrow `WireDragMoveCx` capability for host, bounds, and paint invalidation.
    - Keep retained `EventCx` adaptation isolated in `wire_drag_move_retained_cx.rs`.
    - Move pending/active edge-insert move helpers onto the existing `WidgetPaintInvalidationCx`
      seam because they only need paint invalidation.
    - Prove wire drag hover/threshold and pending edge-insert threshold behavior remain green.
  - Result:
    - `pending_wire_drag.rs` and `wire_drag/move_update/**` now accept `WireDragMoveCx` instead of
      naming retained `EventCx`.
    - `edge_insert_drag/{drag,pending,pending/activate}.rs` now accept
      `WidgetPaintInvalidationCx` instead of naming retained `EventCx`.
    - `pointer_move_dispatch/primary/connection.rs` now accepts `WireDragMoveCx` and routes
      pending wire drag plus pending edge-insert drag without retained Cx names.
    - Added source-policy coverage for the primary connection route, wire-drag move helpers, and
      edge-insert move helpers.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_connection_route_stays_off_retained_bridge) | test(wire_drag_move_handlers_stay_off_retained_bridge) | test(edge_insert_move_handlers_stay_off_retained_bridge) | test(wire_drag_hover_marks_valid_target_port_as_valid) | test(wire_drag_hover_tracks_invalid_port_in_strict_mode) | test(connection_drag_threshold_is_zoom_invariant_in_screen_space) | test(edge_insert_drag_threshold_is_zoom_invariant_in_screen_space) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary/connection.rs ecosystem/fret-node/src/ui/canvas/widget/pending_wire_drag.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag_move_cx.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/mod.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/auto_pan.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/move_update/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/pending/activate.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert_drag/drag/tail.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-790 Isolate pointer-move primary route wrapper retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/primary_pointer_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the primary pointer-move wrapper off direct retained bridge Cx names now that its
      surface, group, node, and connection branches all have retained-agnostic seams.
    - Introduce only a composed `PrimaryPointerMoveCx` capability over existing branch seams, with
      no new side-effect surface.
    - Prove branch source-policy gates and representative primary pointer-move behavior stay green.
  - Result:
    - Added `PrimaryPointerMoveCx` as a composition of `MarqueeCx`,
      `PendingGroupActivationCx`, `GroupPreviewMoveCx`, `PendingNodeDragActivationCx`, and
      `WireDragMoveCx`.
    - `pointer_move_dispatch/primary.rs` now accepts `PrimaryPointerMoveCx` instead of naming
      retained `EventCx`.
    - Added `pointer_move_primary_route_wrapper_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_primary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_primary_surface_route_stays_off_retained_bridge) | test(pointer_move_primary_group_route_stays_off_retained_bridge) | test(pointer_move_primary_node_route_stays_off_retained_bridge) | test(pointer_move_primary_connection_route_stays_off_retained_bridge) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(group_header_click_selects_group_and_arms_pending_group_drag) | test(node_drag_threshold_is_zoom_invariant_in_screen_space) | test(connection_drag_threshold_is_zoom_invariant_in_screen_space) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/primary.rs ecosystem/fret-node/src/ui/canvas/widget/primary_pointer_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-800 Isolate pointer-move secondary node retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_resize/move_update.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_resize_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/node_resize_move_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/node.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the secondary node pointer-move route off direct retained bridge Cx names by introducing
      narrow node drag and node resize move capabilities.
    - Keep retained `EventCx` adaptation isolated in `node_drag_move_retained_cx.rs` and
      `node_resize_move_retained_cx.rs`.
    - Leave secondary connection/insert routing as later isolation slices because those branches
      still have retained-bound leaf helpers.
    - Prove node drag move and node resize move behavior remain green through the retained
      compatibility island.
  - Result:
    - Added `NodeDragMoveCx` as a composition of node drag geometry, preview, and move-tail
      capabilities plus bounds access.
    - Added `NodeResizeMoveCx` for host access plus paint invalidation.
    - `node_drag::handle_node_drag_move(...)`, `node_resize::handle_node_resize_move(...)`, and
      `pointer_move_dispatch/secondary/node.rs` now accept retained-agnostic capabilities instead
      of naming retained `EventCx`.
    - Added source-policy coverage for node drag move handlers, node resize move handlers, and the
      secondary node pointer-move route.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_node_route_stays_off_retained_bridge) | test(node_drag_move_handlers_stay_off_retained_bridge) | test(node_resize_move_handlers_stay_off_retained_bridge) | test(node_drag_move_emits_on_node_drag) | test(node_drag_respects_per_node_extent_rect) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(node_resize_expands_group_when_expand_parent_is_true) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/node.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag.rs ecosystem/fret-node/src/ui/canvas/widget/node_drag_move_cx.rs ecosystem/fret-node/src/ui/canvas/widget/node_resize/move_update.rs ecosystem/fret-node/src/ui/canvas/widget/node_resize_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-810 Isolate pointer-move secondary connection retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/move_start.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag/prelude.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_drag_move_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the secondary connection pointer-move route off direct retained bridge Cx names by
      introducing a narrow edge-drag move capability.
    - Reuse the existing `WireDragMoveCx` and `WidgetPaintInvalidationCx` seams for active wire
      drag and edge-insert drag movement instead of adding duplicate connection capabilities.
    - Keep retained `EventCx` adaptation isolated in `edge_drag_move_retained_cx.rs`.
    - Prove wire move, edge-insert move, and edge reconnect drag behavior remain green through the
      retained compatibility island.
  - Result:
    - Added `EdgeDragMoveCx` for host access plus paint invalidation.
    - `edge_drag::handle_edge_drag_move(...)` and its move-start helper now accept
      `EdgeDragMoveCx` instead of naming retained `EventCx`.
    - `pointer_move_dispatch/secondary/connection.rs` now accepts
      `WireDragMoveCx + EdgeDragMoveCx` and routes active wire drag, active edge-insert drag, and
      edge reconnect drag without retained Cx names.
    - Added source-policy coverage for edge drag move handlers and the secondary connection
      pointer-move route.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_connection_route_stays_off_retained_bridge) | test(edge_drag_move_handlers_stay_off_retained_bridge) | test(wire_drag_move_handlers_stay_off_retained_bridge) | test(edge_insert_move_handlers_stay_off_retained_bridge) | test(edge_reconnect_requires_drag_threshold_before_starting_wire_drag) | test(edge_reconnect_drag_cancels_when_endpoint_not_reconnectable) | test(edge_reconnect_radius_is_zoom_invariant_in_screen_space) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/connection.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/mod.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/move_start.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_drag_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-820 Isolate pointer-move secondary insert retained Cx names.
  - Scope:
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
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the secondary insert pointer-move route and pending insert-node drag move helper off
      direct retained bridge Cx names.
    - Introduce a narrow `InsertNodeDragMoveCx` capability for pointer/window/bounds/tick/host
      access plus pointer-capture release and paint invalidation.
    - Keep retained `EventCx` adaptation isolated in `insert_node_drag_move_retained_cx.rs`.
    - Move the retained internal drag enter/over/drop entry into `insert_node_drag/internal_event.rs`
      so `insert_node_drag/mod.rs` can stay retained-free for the pending move wrapper.
  - Result:
    - Added `InsertNodeDragMoveCx`.
    - `insert_node_drag::handle_pending_insert_node_drag_move(...)`,
      `insert_node_drag/pending.rs`, `insert_node_drag/session.rs`, and
      `pointer_move_dispatch/secondary/insert.rs` now accept retained-agnostic capabilities instead
      of naming retained `EventCx`.
    - `insert_node_drag/internal_event.rs` owns the still-retained internal drag event entry for
      enter/over/leave/cancel/drop; internal move/drop retained I/O remains a later slice.
    - Added source-policy coverage for insert-node drag move handlers and the secondary insert
      pointer-move route.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_insert_route_stays_off_retained_bridge) | test(insert_node_drag_move_handlers_stay_off_retained_bridge) | test(insert_node_drag_does_not_start_until_threshold) | test(insert_node_drag_starts_after_threshold) | test(insert_node_drag_start_clears_searcher_overlay_state) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary/insert.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/mod.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/pending.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/session.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-830 Isolate pointer-move secondary route wrapper retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/secondary_pointer_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the secondary pointer-move wrapper off direct retained bridge Cx names now that its node,
      connection, and insert branches all have retained-agnostic seams.
    - Introduce only a composed `SecondaryPointerMoveCx` capability over existing branch seams, with
      no new side-effect surface.
    - Prove branch source-policy gates and representative secondary pointer-move behavior stay
      green.
  - Result:
    - Added `SecondaryPointerMoveCx` as a composition of `NodeResizeMoveCx`, `NodeDragMoveCx`,
      `WireDragMoveCx`, `EdgeDragMoveCx`, and `InsertNodeDragMoveCx`.
    - `pointer_move_dispatch/secondary.rs` now accepts `SecondaryPointerMoveCx` instead of naming
      retained `EventCx`.
    - Added `pointer_move_secondary_route_wrapper_stays_off_retained_bridge` source-policy
      coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_secondary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_secondary_node_route_stays_off_retained_bridge) | test(pointer_move_secondary_connection_route_stays_off_retained_bridge) | test(pointer_move_secondary_insert_route_stays_off_retained_bridge) | test(node_drag_move_emits_on_node_drag) | test(edge_reconnect_requires_drag_threshold_before_starting_wire_drag) | test(insert_node_drag_starts_after_threshold) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/secondary.rs ecosystem/fret-node/src/ui/canvas/widget/secondary_pointer_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-840 Isolate pointer-move overlay route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/overlay.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the overlay pointer-move route off direct retained bridge Cx names.
    - Narrow searcher and context-menu pointer-move facades to the paint invalidation capability
      they already need, instead of requiring the broader key/down/up route traits.
    - Prove source policy and representative searcher/context-menu hover behavior remain green.
  - Result:
    - `pointer_move_dispatch/overlay.rs` now accepts `WidgetPaintInvalidationCx` instead of naming
      retained `EventCx`.
    - `searcher::handle_searcher_pointer_move(...)` now requires only
      `WidgetPaintInvalidationCx`.
    - `context_menu::handle_context_menu_pointer_move(...)` now requires only
      `WidgetPaintInvalidationCx`.
    - Added `pointer_move_overlay_route_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_overlay_route_stays_off_retained_bridge) | test(searcher_top_level_route_stays_off_retained_bridge) | test(context_menu_top_level_route_stays_off_retained_bridge) | test(context_menu_pointer_move_route_stays_off_retained_bridge) | test(searcher_pointer_move_updates_hover_and_invalidates_paint) | test(context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint) | test(pointer_move_updates_hover_and_invalidates_paint) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch/overlay.rs ecosystem/fret-node/src/ui/canvas/widget/searcher.rs ecosystem/fret-node/src/ui/canvas/widget/context_menu/mod.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-850 Isolate pointer-move hover fallback retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/hover.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/hover_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/hover_move_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_hover_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move fallback hover edge/anchor pointer-move handling off direct retained bridge Cx names.
    - Introduce a narrow `HoverMoveCx` capability for host access plus paint invalidation.
    - Keep retained `EventCx` adaptation isolated in `hover_move_retained_cx.rs`.
    - Prove real hover fallback behavior still updates edge hover and invalidates paint only on
      change.
  - Result:
    - Added `HoverMoveCx`.
    - `hover::update_hover_edge(...)` now accepts `HoverMoveCx` instead of naming retained
      `EventCx`.
    - Added `pointer_move_hover_fallback_stays_off_retained_bridge` source-policy coverage.
    - Added retained compatibility behavior coverage for fallback edge hover invalidation.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_hover_fallback_stays_off_retained_bridge) | test(hover_fallback_updates_hover_edge_and_invalidates_paint_once) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/hover.rs ecosystem/fret-node/src/ui/canvas/widget/hover_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-860 Isolate top-level pointer-move route wrapper retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the top-level pointer-move dispatch wrapper off direct retained bridge Cx names now
      that primary, secondary, overlay, and hover fallback branches are retained-agnostic.
    - Introduce only a composed `PointerMoveCx` capability over existing branch seams, with no new
      side-effect methods.
    - Prove branch source-policy gates and representative pointer-move behavior stay green.
  - Result:
    - Added `PointerMoveCx` as a composition of `PrimaryPointerMoveCx`,
      `SecondaryPointerMoveCx`, and `HoverMoveCx`.
    - `pointer_move_dispatch.rs` now accepts `PointerMoveCx` instead of naming retained `EventCx`.
    - Added `pointer_move_route_wrapper_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_route_wrapper_stays_off_retained_bridge) | test(pointer_move_primary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_secondary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_overlay_route_stays_off_retained_bridge) | test(pointer_move_hover_fallback_stays_off_retained_bridge) | test(node_drag_move_emits_on_node_drag) | test(edge_reconnect_requires_drag_threshold_before_starting_wire_drag) | test(searcher_pointer_move_updates_hover_and_invalidates_paint) | test(context_menu_top_level_pointer_move_updates_hover_and_invalidates_paint) | test(hover_fallback_updates_hover_edge_and_invalidates_paint_once) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-870 Isolate pointer-move cursor update retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/cursor.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/cursor_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/cursor_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/cursor.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_cursor_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pointer-move cursor update helpers off direct retained bridge Cx names.
    - Introduce a narrow `CanvasCursorCx` capability for host access plus cursor icon side effects.
    - Keep retained `EventCx` adaptation isolated in `cursor_retained_cx.rs`.
    - Prove real close-button pointer-move cursor behavior remains green.
  - Result:
    - Added `CanvasCursorCx`.
    - `cursor::update_cursors(...)` and `event_pointer_move_tail/cursor.rs` now accept
      `CanvasCursorCx` instead of naming retained `EventCx`.
    - Added `pointer_move_cursor_update_stays_off_retained_bridge` source-policy coverage.
    - Added retained compatibility behavior coverage for close-button pointer cursor updates.
  - Validation:
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_cursor_update_stays_off_retained_bridge) | test(pointer_move_cursor_update_sets_close_button_cursor) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/cursor.rs ecosystem/fret-node/src/ui/canvas/widget/cursor_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/cursor.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-880 Isolate pointer-move auto-pan timer retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/auto_pan_timer_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/auto_pan_timer_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/timer.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/pointer_move_timer_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move pointer-move auto-pan timer sync off direct retained bridge Cx names.
    - Introduce a narrow `AutoPanTimerCx` capability for host, window, and bounds access.
    - Keep retained `EventCx` adaptation isolated in `auto_pan_timer_retained_cx.rs`.
    - Prove real pointer-move node-drag auto-pan timer behavior remains green.
  - Result:
    - Added `AutoPanTimerCx`.
    - `event_pointer_move_tail/timer.rs` now accepts `AutoPanTimerCx` instead of naming retained
      `EventCx`.
    - Added `pointer_move_auto_pan_timer_stays_off_retained_bridge` source-policy coverage.
    - Added retained compatibility behavior coverage for starting a repeating auto-pan timer during
      a node drag near the viewport edge.
  - Validation:
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_auto_pan_timer_stays_off_retained_bridge) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/auto_pan_timer_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail/timer.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-890 Isolate pointer-move tail wrapper retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_tail_cx.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the pointer-move tail wrapper off direct retained bridge Cx names.
    - Compose the already-isolated cursor, pointer-move dispatch, and auto-pan timer capabilities
      through `PointerMoveTailCx`.
    - Prove representative cursor and auto-pan pointer-move behavior remains green.
  - Result:
    - Added `PointerMoveTailCx`.
    - `event_pointer_move_tail.rs` now accepts `PointerMoveTailCx` instead of naming retained
      `EventCx`.
    - Added `pointer_move_tail_wrapper_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_tail_wrapper_stays_off_retained_bridge) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(pointer_move_cursor_update_sets_close_button_cursor) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move_tail.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_tail_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-900 Isolate pointer-move missing-left-release retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_left.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the missing-left-release pointer-move helper off direct retained bridge Cx names.
    - Reuse the existing `PointerUpCx` / `PointerUpReleaseCx` seams instead of adding another
      retained adapter.
    - Prove missed left pointer-up inference still commits/cancels active drag families correctly.
  - Result:
    - `pointer_move_release_left.rs` now accepts `PointerUpCx` instead of naming retained `EventCx`.
    - Host access for snapshot sync now uses `PointerUpReleaseCx::host(...)`.
    - Added `pointer_move_missing_left_release_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_missing_left_release_stays_off_retained_bridge) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state_for_wire_reconnect_drag) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state_for_new_wire_drag) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_left.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-910 Isolate pointer-move release route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/tail.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/missing_release.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/interaction_conformance.rs`
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the pointer-move release guard wrappers and pan-release helpers off direct retained bridge
      Cx names.
    - Introduce a composed `PointerMoveReleaseCx` over existing pointer-up and pan-begin
      capabilities instead of adding another retained adapter.
    - Prove missed pan pointer-up inference and right-button pan start behavior remain green.
  - Result:
    - `event_pointer_move.rs`, `event_pointer_move/release.rs`, `event_pointer_move/tail.rs`,
      `pointer_move_release.rs`, `pointer_move_release_pan.rs`,
      `pointer_move_release_pan/missing_release.rs`, and
      `pointer_move_release_pan/pending_right_click.rs` now stay retained-Cx agnostic.
    - `PointerMoveReleaseCx` composes `PointerUpCx` and `PanZoomBeginCx`; retained `EventCx`
      satisfies the release route only through those existing adapter seams.
    - Added `pointer_move_release_route_stays_off_retained_bridge` source-policy coverage and
      `missing_pan_pointer_up_can_be_inferred_from_mouse_buttons_state` behavior coverage.
  - Validation:
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_release_route_stays_off_retained_bridge) | test(pointer_move_missing_left_release_stays_off_retained_bridge) | test(missing_pan_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(right_pan_drag_does_not_open_context_menu) | test(right_pan_defers_context_menu_until_pointer_up) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/release.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/tail.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/missing_release.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_release_pan/pending_right_click.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
- [x] RBX-M2-930 Isolate pointer-wheel and timer-motion retained Cx routes.
  - Scope:
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
    - workstream evidence/handoff/ledger docs
  - Goal:
    - Move the pointer-wheel and timer-motion route wrappers off direct retained bridge Cx names.
    - Compose the wheel route through retained-agnostic viewport-motion plus searcher/platform
      seams.
    - Compose the timer route through retained-agnostic viewport-motion plus pointer-move-tail
      seams.
    - Thread explicit `Platform` through the wheel route chain and delete
      `pointer_wheel_retained_cx.rs`.
    - Prove wheel zoom/pan, viewport animation, auto-pan, and timer motion behavior remain green.
  - Result:
    - `event_pointer_wheel.rs`, `event_pointer_wheel_route.rs`, `pointer_wheel_motion.rs`,
      `pointer_wheel_pan.rs`, `pointer_wheel_pan/apply.rs`, `pointer_wheel_viewport.rs`,
      `pointer_wheel_zoom.rs`, `pointer_wheel_zoom/apply.rs`, `pointer_wheel_zoom/pinch.rs`,
      `pointer_wheel_zoom/wheel.rs`, `event_timer.rs`, `event_timer_route.rs`, `timer_motion.rs`,
      `timer_motion_auto_pan.rs`, `timer_motion_auto_pan/dispatch.rs`,
      `timer_motion_pan_inertia.rs`, `timer_motion_viewport.rs`,
      `timer_motion_viewport/animation.rs`, and `timer_motion_viewport/debounce.rs` now stay
      retained-Cx agnostic.
    - Added `ViewportMotionCx`, `PointerWheelCx`, and `TimerMotionCx` seams; the wheel route now
      threads explicit `Platform` through to `pointer_wheel_pan.rs`, and
      `pointer_wheel_retained_cx.rs` has been deleted.
    - Added `pointer_wheel_route_stays_off_retained_bridge` and
      `timer_motion_route_stays_off_retained_bridge` source-policy coverage.
  - Validation:
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_wheel_route_stays_off_retained_bridge) | test(timer_motion_route_stays_off_retained_bridge) | test(wheel_zoom_emits_move_start_and_debounced_move_end) | test(pinch_zoom_emits_move_start_and_debounced_move_end) | test(wheel_pan_emits_move_start_and_debounced_move_end) | test(wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom) | test(frame_view_animates_over_timer_ticks_and_reaches_target) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(pinch_gesture_zooms_in_about_pointer) | test(wheel_zoom_zooms_about_pointer) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "retained_bridge|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_route.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_motion.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan/apply.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_viewport.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/apply.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/pinch.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom/wheel.rs ecosystem/fret-node/src/ui/canvas/widget/event_timer.rs ecosystem/fret-node/src/ui/canvas/widget/event_timer_route.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_pan_inertia.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_auto_pan.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_auto_pan/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport/animation.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_viewport/debounce.rs ecosystem/fret-node/src/ui/canvas/widget/viewport_motion_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_cx.rs ecosystem/fret-node/src/ui/canvas/widget/timer_motion_cx.rs`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - 2026-05-24 rerun:
      - `cargo nextest run -p fret-node`
      - `cargo check -p fret-node --features compat-retained-canvas`
      - `cargo fmt --all -- --check`
      - `python3 tools/check_layering.py`
      - `python3 tools/check_workstream_catalog.py`
      - `git diff --check`
- [x] Split full node graph implementation into follow-on work:
  - declarative composition for chrome/overlays/panels,
  - `Canvas`/`ViewportSurface`-style leaf for heavy rendering where needed.
  - Closeout decision:
    - The runtime retained bridge exit no longer requires this broad split to be completed in this
      lane. `RBX-M4-043` deleted `fret-ui/unstable-retained-bridge` and the
      `fret_ui::compat_retained_canvas` facade; `fret-node/compat-retained-canvas` is now a
      node-local legacy implementation gate that maps only to stable `fret-ui` exports.
    - Continue this as a future node-graph architecture lane rather than reopening the deleted
      runtime bridge.
  - Evidence:
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---closeout-audit-runtime-retained-bridge-exit`
- [x] Remove `unstable-retained-bridge` from `ecosystem/fret-node` dependencies.
  - Completed by `RBX-M4-043`: `fret-node/compat-retained-canvas` now maps to `fret-ui` only,
    and `crates/fret-ui` no longer defines `unstable-retained-bridge`.

- [x] RBX-M2-940 Un-gate pure node geometry and route-math helpers from the retained bridge island.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/geometry/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/route_math.rs`
    - `ecosystem/fret-node/src/lib.rs`
  - Goal:
    - Make the node graph's pure geometry and route-math helpers available on the default build
      instead of only behind the retained compat island.
    - Keep the retained widget island intact while shrinking its surface area.
    - Prove the helper exports no longer need compat gating even though the retained widget island
      still exists.
  - Validation:
    - `cargo test -p fret-node pure_geometry_and_route_math_helpers_are_available_without_compat_gating -- --nocapture`
    - `cargo check -p fret-node`
    - `python3 tools/check_layering.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-node/src/ui/canvas/geometry/mod.rs ecosystem/fret-node/src/ui/canvas/route_math.rs ecosystem/fret-node/src/lib.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/geometry/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/route_math.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-940-un-gate-pure-node-geometry-and-route-math-helpers-from-the-retained-bridge-island`
  - Result:
    - Pure geometry helpers `group_order`, `node_anchor_from_rect_origin`, and
      `node_rect_origin_from_anchor` are now available without `compat-retained-canvas`.
    - Pure route-math helpers `cubic_bezier`, `normal_from_tangent`, `edge_route_start_tangent`,
      and `edge_route_end_tangent` are now available without `compat-retained-canvas`.
    - Default package check and the new source-policy regression pass; the retained widget island
      remains intact for the heavier adapter work.

- [x] RBX-M2-950 Isolate left-click hit routing retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/handlers.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/hit.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits/**`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits/**`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/group_background.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/group_background/**`
  - Goal:
    - Move the left-click hit-routing subtree off direct retained `EventCx` names by composing
      the existing `MarqueeCx` and `WireCommitCx` seams.
    - Keep host access explicit through a narrow `LeftClickCx` seam and reuse the existing
      retained adapters for capture, paint, and commit behavior.
    - Add a source-policy gate so the left-click route cannot regrow direct retained Cx names.
  - Validation:
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(left_click_route_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(background_click_starts_pending_marquee_and_clears_selection_on_up) | test(group_header_click_selects_group_and_arms_pending_group_drag) | test(group_resize_is_previewed_and_committed_on_pointer_up) | test(shift_clicking_a_node_does_not_clear_selection) | test(node_click_does_not_select_node_when_node_selectable_is_false) | test(click_connect_target_port_click_commits_wire_and_clears_click_connect_state) | test(edge_drag_left_up_clears_edge_drag_and_invalidates_paint) | test(node_drag_threshold_is_zoom_invariant_in_screen_space) | test(connection_drag_threshold_is_zoom_invariant_in_screen_space)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/left_click -g '*.rs' || test $? -eq 1`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/handlers.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/hit.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/connection_hits.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/element_hits.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/group_background.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-950-isolate-left-click-hit-routing-retained-cx-names`
  - Result:
    - `left_click/**` now routes through `LeftClickCx` rather than naming retained `EventCx`
      directly.
    - The retained `EventCx` adapters remain shared via the existing `MarqueeCx` / `WireCommitCx`
      / pan-begin seams.
    - The new source-policy gate and focused behavior tests keep the route honest.

- [x] RBX-M2-960 Isolate keyboard shortcut command dispatch retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move modifier, Tab, arrow-nudge, and delete shortcut command dispatch off direct retained
      `EventCx` signatures.
    - Keep retained event adaptation isolated in `keyboard_shortcuts.rs` behind the narrow
      `KeyboardShortcutCommandSink` seam.
    - Add a source-policy gate so `keyboard_shortcuts_commands.rs` cannot regrow direct retained
      Cx names.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(keyboard_shortcut_command_helpers_stay_off_retained_cx) | test(disable_keyboard_a11y_does_not_block_delete_shortcut) | test(disable_keyboard_a11y_blocks_tab_focus_traversal) | test(nudge_moves_selection_and_records_history_entry) | test(nudge_multi_selection_respects_node_extent_by_selection_bounds) | test(nudge_respects_per_node_extent_rect)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_commands.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-960-isolate-keyboard-shortcut-command-dispatch-retained-cx-names`
  - Result:
    - `keyboard_shortcuts_commands.rs` now dispatches through `KeyboardShortcutCommandSink` rather
      than directly naming `EventCx`.
    - The retained `EventCx` adapter remains isolated in `keyboard_shortcuts.rs`, while overlay
      keyboard and Escape routing can be handled in later slices.
    - Focused keyboard behavior tests and the new source-policy gate pass.

- [x] RBX-M2-970 Isolate keyboard overlay/Escape retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move Escape, searcher-key, and context-menu-key overlay keyboard routing off direct retained
      `EventCx` signatures in the helper layer.
    - Compose the already-isolated searcher, context-menu, and cancel seams through a narrow
      `KeyboardOverlayCx` trait.
    - Keep the retained keyboard event adapter explicit in `keyboard_shortcuts.rs` and leave the
      top-level `event_keyboard_route.rs` retained `EventCx` entry for a later slice.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(keyboard_overlay_helpers_stay_off_retained_cx) | test(searcher_top_level_escape_dismisses_and_finishes) | test(context_menu_top_level_escape_dismisses_and_finishes) | test(escape_cancel_releases_pointer_capture_during_panning)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-970-isolate-keyboard-overlayescape-retained-cx-names`
  - Result:
    - `keyboard_shortcuts_overlay.rs` now owns the retained-agnostic overlay/Escape helper logic.
    - `KeyboardOverlayCx` composes `SearcherCx`, `ContextMenuCx`, and `CancelGestureCx` without
      naming retained context types.
    - Behavior coverage proves Escape still dismisses the searcher/context menu before falling
      through to cancel active pan/gesture state.

- [x] RBX-M2-980 Isolate top-level keyboard event route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the top-level keyboard route helper off direct retained `EventCx` signatures.
    - Compose the existing keyboard overlay, command, and pan-activation side-effect seams through
      a narrow `KeyboardRouteCx` trait.
    - Keep `event_keyboard.rs` as the explicit retained event adapter for now.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(keyboard_event_route_stays_off_retained_cx) | test(keyboard_overlay_helpers_stay_off_retained_cx) | test(keyboard_shortcut_command_helpers_stay_off_retained_cx) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(pan_activation_key_code_must_match_to_enable_space_to_pan) | test(pan_activation_key_code_none_disables_space_to_pan_activation) | test(disable_keyboard_a11y_does_not_block_delete_shortcut) | test(disable_keyboard_a11y_blocks_tab_focus_traversal)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_overlay.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-980-isolate-top-level-keyboard-event-route-retained-cx-names`
  - Result:
    - `event_keyboard_route.rs` now receives `KeyboardRouteCx` / `WidgetPaintInvalidationCx`
      instead of retained `EventCx`.
    - `KeyboardRouteCx` composes keyboard overlay, shortcut command, and handled/paint tail
      capabilities without naming retained context types.
    - `event_keyboard.rs` remains the retained event adapter and is the next keyboard entry-point
      shrink target.

- [x] RBX-M2-990 Isolate keyboard system input route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move `event_router_system_input.rs` off direct retained `EventCx` signatures for keyboard
      input routing.
    - Keep retained `EventCx` adaptation in `event_keyboard.rs` behind a narrow
      `KeyboardInputSink` seam that exposes only text-input focus plus the already-isolated
      keyboard route capabilities.
    - Preserve key-down ignore behavior for text-input focus and existing keyboard command /
      space-to-pan behavior.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(keyboard_system_input_route_stays_off_retained_cx) | test(keyboard_event_route_stays_off_retained_cx) | test(should_ignore_key_down_tracks_text_input_focus) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(disable_keyboard_a11y_does_not_block_delete_shortcut) | test(disable_keyboard_a11y_blocks_tab_focus_traversal)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_route.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-990-isolate-keyboard-system-input-route-retained-cx-names`
  - Result:
    - `event_router_system_input.rs` now routes key events through `KeyboardInputSink` rather than
      retained `EventCx`.
    - `event_keyboard.rs` is the explicit retained adapter for text-input focus plus the composed
      keyboard route seams.
    - The next natural M2 slice is `event_router_system.rs` / lifecycle routing, where clipboard,
      focus-loss cancel, pointer-cancel, internal drag, and timer events still receive retained
      `EventCx`.

- [x] RBX-M2-1000 Isolate system lifecycle event route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/internal_drag_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/internal_drag_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_move.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_drop.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move `event_router_system_lifecycle.rs` off direct retained `EventCx` signatures.
    - Compose existing clipboard feedback, cancel, timer-motion, and a new internal-drag seam
      through a narrow `SystemLifecycleCx` trait.
    - Keep retained internal-drag payload/window/host adaptation explicit in
      `internal_drag_retained_cx.rs`.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs ecosystem/fret-node/src/ui/canvas/widget/internal_drag_cx.rs ecosystem/fret-node/src/ui/canvas/widget/internal_drag_retained_cx.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_event.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_move.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_drop.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(system_lifecycle_route_helpers_stay_off_retained_cx) | test(clipboard_unavailable_with_matching_token_shows_toast_and_invalidates_paint) | test(matching_toast_timer_clears_toast_and_invalidates_paint) | test(window_focus_lost_cancels_wire_drag) | test(pointer_left_cancels_wire_drag) | test(internal_drag_drop_candidate_on_edge_splits_edge) | test(internal_drag_drop_candidate_off_edge_creates_node)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs ecosystem/fret-node/src/ui/canvas/widget/event_clipboard.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_event.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_move.rs ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_drop.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_clipboard.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/internal_drag_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/internal_drag_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_move.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/insert_node_drag/internal_drop.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1000-isolate-system-lifecycle-event-route-retained-cx-names`
  - Result:
    - `event_router_system_lifecycle.rs` now routes through `SystemLifecycleCx` rather than
      retained `EventCx`.
    - `event_clipboard.rs` now uses `ClipboardTextCx`; retained clipboard feedback adaptation
      remains in `event_clipboard_feedback_retained_cx.rs`.
    - Insert-node internal drag move/drop/event helpers now use `InternalDragCx`; retained drag
      session/window/host adaptation remains in `internal_drag_retained_cx.rs`.
    - The next natural slice is `event_router_system.rs`, which still composes lifecycle/input
      subroutes from a retained `EventCx` entry.

- [x] RBX-M2-1010 Isolate top-level non-pointer system event route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_system.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move `event_router_system.rs` off direct retained `EventCx` signatures now that keyboard
      input and lifecycle subroutes are retained-agnostic.
    - Compose the subroutes through a narrow `SystemRouteCx` seam.
    - Keep retained `EventCx` adaptation at the broader `event_router.rs` / pointer-router boundary
      for later slices.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(system_non_pointer_route_stays_off_retained_cx) | test(system_lifecycle_route_helpers_stay_off_retained_cx) | test(keyboard_system_input_route_stays_off_retained_cx) | test(window_focus_lost_cancels_wire_drag) | test(pointer_left_cancels_wire_drag) | test(space_to_pan_starts_left_mouse_panning_and_updates_viewport) | test(internal_drag_drop_candidate_off_edge_creates_node)'`
    - `rg -n "retained_bridge|compat_retained_canvas|EventCx|CommandCx|LayoutCx|PaintCx" ecosystem/fret-node/src/ui/canvas/widget/event_router_system.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system_input.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_system_lifecycle.rs || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_system.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1010-isolate-top-level-non-pointer-system-event-route-retained-cx-names`
  - Result:
    - `event_router_system.rs` now receives `SystemRouteCx` rather than retained `EventCx`.
    - `SystemRouteCx` composes `SystemLifecycleCx` and `KeyboardInputSink` without naming retained
      context types.
    - The remaining retained `EventCx` route boundary is now the broader `event_router.rs` /
      pointer-event router path.

- [x] RBX-M2-1020 Isolate pointer-down double-click route retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/hit.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/apply.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/insert_picker.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/target.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/reroute.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move pointer-down background zoom, edge-insert picker, and edge-reroute double-click route
      helpers off direct retained `EventCx` signatures.
    - Compose the host/window and handled-tail requirements through a narrow
      `PointerDownDoubleClickCx` seam.
    - Keep retained `EventCx` adaptation isolated in `pointer_down_double_click_retained_cx.rs`.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_retained_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/hit.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background/apply.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/insert_picker.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/target.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge/reroute.rs`
    - `rg -n "EventCx|CommandCx|LayoutCx|PaintCx|retained_bridge|compat_retained_canvas" ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_double_click_route_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_down_double_click_route_stays_off_retained_bridge) | test(edge_double_click_finish_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(background_double_click_zoom) | test(edge_insert) | test(reroute)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_retained_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1020-isolate-pointer-down-double-click-route-retained-cx-names`
  - Result:
    - `pointer_down_double_click.rs`, `pointer_down_double_click_background/**`, and
      `pointer_down_double_click_edge/**` now receive `PointerDownDoubleClickCx` instead of
      retained `EventCx`.
    - The retained adapter is isolated to `pointer_down_double_click_retained_cx.rs`.
    - The full compat-retained `fret-node` package gate passes after the route seam.

- [x] RBX-M2-1030 Isolate pointer wheel event router retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_retained_cx.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the pointer wheel/pinch event router off direct retained `EventCx` signatures.
    - Compose existing wheel behavior requirements through `PointerWheelRouteCx` plus a narrow
      platform accessor.
    - Keep retained `EventCx::input_ctx.platform` adaptation isolated in
      `event_router_pointer_wheel_retained_cx.rs`.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_retained_cx.rs`
    - `rg -n "EventCx|CommandCx|LayoutCx|PaintCx|retained_bridge|compat_retained_canvas" ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_wheel_route.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_motion.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_pan.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_viewport.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_wheel_zoom.rs -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-node`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_wheel_route_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_wheel_route_stays_off_retained_bridge) | test(wheel_zoom_emits_move_start_and_debounced_move_end) | test(pinch_zoom_emits_move_start_and_debounced_move_end) | test(wheel_pan_emits_move_start_and_debounced_move_end) | test(wheel_pan_then_wheel_zoom_ends_pan_and_starts_zoom) | test(pinch_gesture_zooms_in_about_pointer) | test(wheel_zoom_zooms_about_pointer) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_wheel_retained_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1030-isolate-pointer-wheel-event-router-retained-cx-names`
  - Result:
    - `event_router_pointer_wheel.rs` now receives `PointerWheelRouteCx` rather than retained
      `EventCx`.
    - The wheel router policy test now covers the top-level wheel event router and the route Cx
      seam, not just the lower-level wheel handlers.
    - The retained adapter is isolated to `event_router_pointer_wheel_retained_cx.rs`.


- [x] RBX-M2-1040 Isolate pointer-move button router retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/move_event.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the pointer-move button router wrapper off direct retained `EventCx` signatures.
    - Reuse the existing retained-agnostic `PointerMoveReleaseCx + PointerMoveTailCx` route bounds
      already required by `handle_pointer_move(...)`.
    - Extend source-policy coverage so the router wrapper cannot re-import retained Cx names.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/move_event.rs`
    - `rg -n "EventCx|CommandCx|LayoutCx|PaintCx|retained_bridge|compat_retained_canvas" ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/move_event.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/release.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_move/tail.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_move_cx.rs -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_move_route_wrapper_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_move_route_wrapper_stays_off_retained_bridge) | test(pointer_move_release_route_stays_off_retained_bridge) | test(pointer_move_primary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_secondary_route_wrapper_stays_off_retained_bridge) | test(pointer_move_auto_pan_timer_starts_for_node_drag_near_viewport_edge) | test(missing_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(missing_pan_pointer_up_can_be_inferred_from_mouse_buttons_state) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/move_event.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1040-isolate-pointer-move-button-router-retained-cx-names`
  - Result:
    - `event_router_pointer_button/move_event.rs` now receives retained-agnostic pointer-move route
      bounds instead of retained `EventCx`.
    - Pointer-move behavior remains covered by focused route, release inference, primary/secondary,
      auto-pan, and full compat package gates.

- [x] RBX-M2-1050 Isolate pointer-down double-click route wrapper retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/double_click.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the pointer-down double-click route wrapper off direct retained `EventCx` signatures.
    - Reuse the existing retained-agnostic `PointerDownDoubleClickCx` route seam created by
      `RBX-M2-1020`.
    - Extend source-policy coverage so the wrapper cannot re-import retained Cx names.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_double_click_route_stays_off_retained_bridge`
      first failed after adding `event_pointer_down_route/double_click.rs` to the source-policy
      coverage, proving the gate caught the intended remaining retained Cx boundary.
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/double_click.rs ecosystem/fret-node/src/lib.rs`
    - `rg -n "EventCx|CommandCx|LayoutCx|PaintCx|retained_bridge|compat_retained_canvas" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/double_click.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_background ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click_edge -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_double_click_route_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(pointer_down_double_click_route_stays_off_retained_bridge) | test(edge_double_click_finish_stays_off_retained_bridge) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(background_double_click_zoom) | test(edge_insert) | test(reroute)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `git diff --check -- ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/double_click.rs`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/double_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_double_click.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1050-isolate-pointer-down-double-click-route-wrapper-retained-cx-names`
  - Result:
    - `event_pointer_down_route/double_click.rs` now receives `PointerDownDoubleClickCx` instead
      of retained `EventCx`.
    - The pointer-down double-click policy test now covers the route wrapper as well as the helper
      subtrees.
    - The remaining pointer-down preflight/start/tail wrappers are still retained-bound and are the
      next M2 seam candidates.

- [x] RBX-M2-1060 Isolate pointer-down preflight retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/preflight.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/close_button.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_retained_cx.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the pointer-down preflight route off direct retained `EventCx` signatures.
    - Reuse the retained-agnostic `SearcherPointerDownCx`, `PointerDownCloseButtonCx`, and
      `PointerDownDoubleClickCx` seams instead of carrying retained context directly through the
      route wrapper.
    - Keep retained `EventCx` adaptation isolated in the dedicated adapter files.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_preflight_route_stays_off_retained_bridge`
      first failed after adding `event_pointer_down_route/preflight.rs` and
      `pointer_down_gesture_start/close_button.rs` to the source-policy coverage, proving the gate
      caught the intended preflight boundary.
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/searcher.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/preflight.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/close_button.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_retained_cx.rs ecosystem/fret-node/src/lib.rs`
    - `rg -n "EventCx|CommandCx|LayoutCx|PaintCx|retained_bridge|compat_retained_canvas" ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/preflight.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/close_button.rs -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_preflight_route_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `git diff --check -- ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/preflight.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/close_button.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_retained_cx.rs`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/preflight.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/close_button.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_close_button_retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/searcher.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1060-isolate-pointer-down-preflight-retained-cx-names`
  - Result:
    - `event_pointer_down_route/preflight.rs` now routes through `PointerDownPreflightCx` rather
      than retained `EventCx`.
    - Close-button pointer-down dispatch now uses `PointerDownCloseButtonCx`, with retained
      dispatch isolated in `pointer_down_close_button_retained_cx.rs`.
    - `searcher.rs` now narrows pointer-down routing to `SearcherPointerDownCx`, leaving the
      broader retained pointer-down boundary in the remaining start/tail wrappers.

- [x] RBX-M2-1070 Isolate pointer-down starts retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/starts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/menu.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/pending_right_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/pan_start.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/sticky.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the pointer-down starts route off direct retained `EventCx` signatures.
    - Reuse the retained-agnostic `ContextMenuCx`, `PointerDownStartCx`,
      `StickyWireTargetPickerCx`, and `PanZoomBeginCx` seams instead of carrying retained context
      directly through the route wrapper.
    - Keep retained `EventCx` adaptation isolated in the existing adapter files for those seams.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_starts_route_stays_off_retained_bridge`
      first failed after adding `event_pointer_down_route/starts.rs` and the
      `pointer_down_gesture_start` subtree to the source-policy coverage, proving the gate caught
      the intended start boundary.
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/starts.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/menu.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/pending_right_click.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/sticky.rs ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/pan_start.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_starts_route_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/starts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/menu.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/pending_right_click.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/sticky.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/pointer_down_gesture_start/pan_start.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1070-isolate-pointer-down-starts-retained-cx-names`
  - Result:
    - `event_pointer_down_route/starts.rs` now routes through `PointerDownStartCx` instead of
      retained `EventCx`.
    - The pointer-down start policy test now covers the route wrapper and the start helper
      subtree.
    - Remaining pointer-down tail wrappers still carry the retained boundary and are the next M2
      seam candidates.

- [x] RBX-M2-1080 Isolate pointer-down tail retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down/prelude.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down/dispatch.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/dispatch.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the remaining pointer-down tail route off direct retained `EventCx` signatures.
    - Reuse the retained-agnostic `PointerDownRouteCx` seam instead of carrying retained context
      directly through the pointer-down entry, route, and tail wrappers.
    - Keep retained `EventCx` adaptation isolated in the existing adapter files for the underlying
      tail traits.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_tail_route_stays_off_retained_bridge`
      first failed after adding `event_pointer_down.rs`, `event_pointer_down_route.rs`,
      `event_pointer_down_route/dispatch.rs`, and `event_router_pointer_button/down.rs` to the
      source-policy coverage, proving the gate caught the intended tail boundary.
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route.rs ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/dispatch.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/down.rs ecosystem/fret-node/src/ui/canvas/widget/left_click/mod.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas pointer_down_tail_route_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down/prelude.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down/dispatch.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_pointer_down_route/dispatch.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button/down.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/left_click/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1080-isolate-pointer-down-tail-retained-cx-names`
  - Result:
    - `event_pointer_down.rs`, `event_pointer_down_route.rs`, `event_pointer_down_route/dispatch.rs`,
      `event_pointer_down/prelude.rs`, `event_pointer_down/dispatch.rs`, and
      `event_router_pointer_button/down.rs` now route through `PointerDownRouteCx` instead of
      retained `EventCx`.
    - The pointer-down tail policy test now covers the route entry, wrapper, dispatch, and router
      files.
    - The broader `event_router.rs` entry remains the next retained boundary outside this tail
      slice.

- [x] RBX-M2-1090 Isolate top-level event router retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the top-level event router off direct retained `EventCx` signatures.
    - Compose already-isolated system, pointer-down, pointer-move, pointer-up, and wheel route
      seams behind a retained-agnostic `EventRouteCx`.
    - Keep retained `EventCx` adaptation isolated in existing lower-level adapter files.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas top_level_event_router_stays_off_retained_bridge`
      first failed after adding `event_router.rs`, `event_router_pointer.rs`,
      `event_router_pointer_button.rs`, and `event_router_pointer_button/down.rs` to the
      source-policy coverage, proving the gate caught the intended top-level boundary.
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_cx.rs ecosystem/fret-node/src/ui/canvas/widget/event_router.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer.rs ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas top_level_event_router_stays_off_retained_bridge`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_router_pointer_button.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1090-isolate-top-level-event-router-retained-cx-names`
  - Result:
    - `event_router.rs`, `event_router_pointer.rs`, and `event_router_pointer_button.rs` now route
      through retained-agnostic `EventRouteCx` / `PointerEventRouteCx` /
      `PointerButtonRouteCx`.
    - The top-level event router source-policy gate now covers the outer router wrappers.
    - Remaining direct retained event boundaries should now be adapter files or explicitly
      retained widget runtime internals.

- [x] RBX-M2-1100 Isolate edge-insert menu/insert retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/prelude.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/context_menu.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/insert.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move edge-insert menu opening and insertion dispatch off direct retained `EventCx`
      signatures.
    - Introduce a narrow retained-agnostic `EdgeInsertCx` seam for host, window, and bounds access.
    - Keep retained `EventCx` adaptation isolated in `edge_insert/retained_cx.rs`.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas edge_insert_menu_and_insert_routes_stay_off_retained_bridge`
      first failed after adding `edge_insert/prelude.rs`, `insert.rs`, and `context_menu.rs` to
      source-policy coverage, proving the gate caught the intended edge-insert boundary.
    - `cargo nextest run -p fret-node --features compat-retained-canvas edge_insert_menu_and_insert_routes_stay_off_retained_bridge`
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert/mod.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert/prelude.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert/cx.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert/retained_cx.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert/insert.rs ecosystem/fret-node/src/ui/canvas/widget/edge_insert/context_menu.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/prelude.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/retained_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/context_menu.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/edge_insert/insert.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1100-isolate-edge-insert-menuinsert-retained-cx-names`
  - Result:
    - `open_edge_insert_context_menu(...)`, `activate_edge_insert_picker_action(...)`, and
      `insert_node_on_edge(...)` now route through `EdgeInsertCx` instead of direct retained
      `EventCx`.
    - `EdgeInsertCx` exposes only the host, window, and bounds required by edge-insert policy.
    - The retained adapter is isolated in `edge_insert/retained_cx.rs`, and the source-policy gate
      locks the menu/insert route files as retained-Cx agnostic.

- [x] RBX-M2-1110 Isolate wire-drag helper retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers_retained_cx.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move wire-drag helper capture / repaint plumbing off direct retained `EventCx` signatures.
    - Introduce a narrow retained-agnostic `WireDragStartCx` seam for pointer capture and paint
      invalidation.
    - Keep retained `EventCx` adaptation isolated in `wire_drag_helpers_retained_cx.rs`.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas wire_drag_helpers_stay_off_retained_bridge`
      first failed after adding `wire_drag_helpers.rs` to source-policy coverage, proving the gate
      caught the intended wire-drag boundary.
    - `cargo nextest run -p fret-node --features compat-retained-canvas wire_drag_helpers_stay_off_retained_bridge`
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers_cx.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers_retained_cx.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag_helpers_retained_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1110-isolate-wire-drag-helper-retained-cx-names`
  - Result:
    - `start_sticky_wire_drag_from_port(...)` and `restore_suspended_wire_drag(...)` now route
      through `WireDragStartCx` instead of direct retained `EventCx`.
    - `WireDragStartCx` only exposes the self-pointer capture behavior needed by wire-drag start
      and restore.
    - The retained adapter is isolated in `wire_drag_helpers_retained_cx.rs`, and the source-policy
      gate locks the helper file as retained-Cx agnostic.

- [x] RBX-M2-1120 Isolate keyboard input focus retained Cx names.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_retained_cx.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the remaining keyboard input focus check off direct retained `EventCx` names.
    - Introduce a narrow retained-agnostic `KeyboardInputFocusCx` seam for the text-input focus
      predicate used by keyboard routing.
    - Keep retained `EventCx` adaptation isolated in `event_keyboard_retained_cx.rs`.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas keyboard_input_focus_helper_stays_off_retained_cx`
      first failed while the retained adapter file was missing, proving the new gate required an
      explicit retained adapter module before implementation.
    - `cargo nextest run -p fret-node --features compat-retained-canvas keyboard_input_focus_helper_stays_off_retained_cx`
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_retained_cx.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/event_keyboard_retained_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1120-isolate-keyboard-input-focus-retained-cx-names`
  - Result:
    - `event_keyboard.rs` now exposes `KeyboardInputFocusCx` and composes it into
      `KeyboardInputSink` without naming retained `EventCx`.
    - The retained `EventCx` implementation for `focus_is_text_input()` is isolated in
      `event_keyboard_retained_cx.rs`.
    - The source-policy gate locks `event_keyboard.rs` as retained-Cx agnostic while keeping the
      retained adapter explicit.

- [x] RBX-M2-1130 Isolate keyboard shortcut command retained Cx adapter.
  - Scope:
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_retained_cx.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the retained `EventCx` implementation of `KeyboardShortcutCommandSink` out of
      `keyboard_shortcuts.rs`.
    - Keep `keyboard_shortcuts.rs` as retained-Cx-agnostic wrapper/seam code.
    - Keep retained command dispatch adaptation isolated in `keyboard_shortcuts_retained_cx.rs`.
  - Validation:
    - `cargo nextest run -p fret-node --features compat-retained-canvas keyboard_shortcut_wrapper_stays_off_retained_cx`
      first failed while `keyboard_shortcuts_retained_cx.rs` was missing, proving the new gate
      required an explicit retained adapter module before implementation.
    - `cargo nextest run -p fret-node --features compat-retained-canvas keyboard_shortcut_wrapper_stays_off_retained_cx`
    - `rustfmt --edition 2024 --check ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_retained_cx.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `cargo nextest run -p fret-node --features compat-retained-canvas`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/keyboard_shortcuts_retained_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m2-1130-isolate-keyboard-shortcut-command-retained-cx-adapter`
  - Result:
    - `keyboard_shortcuts.rs` now defines `KeyboardShortcutCommandSink` and helper wrappers without
      naming retained `EventCx`.
    - The retained `EventCx` command dispatch implementation is isolated in
      `keyboard_shortcuts_retained_cx.rs`.
    - The source-policy gate locks the wrapper file as retained-Cx agnostic while keeping the
      retained adapter explicit.

### M3 — Charts/plots migration

- [x] RBX-M3-010 Migrate `fret-plot3d` to a declarative viewport-surface panel.
  - Scope:
    - `ecosystem/fret-plot3d/`
    - `apps/fret-examples/src/plot3d_demo.rs`
    - `apps/fret-examples/src/gizmo3d_demo.rs`
    - `tools/check_layering.py`
  - Goal:
    - Delete the retained `Plot3dCanvas` widget surface.
    - Keep the portable Plot3D model/style/viewport contract.
    - Replace first-party retained demo mounting with public declarative `plot3d_panel(...)`.
    - Remove `fret-plot3d` from the `fret-ui/unstable-retained-bridge` allowlist.
  - Validation:
    - `cargo check -p fret-plot3d`
    - `cargo check -p fret-demo --bin plot3d_demo`
    - `cargo check -p fret-demo --bin gizmo3d_demo`
    - `cargo nextest run -p fret-plot3d`
    - `python3 tools/check_layering.py`
  - Evidence:
    - `ecosystem/fret-plot3d/src/declarative.rs`
    - `ecosystem/fret-plot3d/src/lib.rs`
    - `ecosystem/fret-plot3d/Cargo.toml`
    - `apps/fret-examples/src/plot3d_demo.rs`
    - `apps/fret-examples/src/gizmo3d_demo.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-010-plot3d-declarative-viewport-panel`
  - Result:
    - `fret-plot3d` no longer enables `fret-ui/unstable-retained-bridge`.
    - `Plot3dCanvas` and `src/retained.rs` were deleted.
    - Public Plot3D authoring is now `plot3d_panel(...)` / `Plot3dPanelProps`, backed by
      `fret-ui-kit`'s declarative `viewport_surface_panel(...)`.
    - First-party Plot3D demos now mount through `declarative::RenderRootContext::render_root(...)`.
- [x] RBX-M3-020 Add a declarative `fret-chart` canvas capability baseline before deleting
  retained chart code.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
  - Goal:
    - Prove `chart_canvas_panel(...)` can render a controlled `Model<ChartEngine>` through a real
      declarative UI frame without constructing `retained::ChartCanvas`.
    - Lock the migration-critical behavior before moving public/demo/gallery consumers off retained
      chart entry points.
  - Validation:
    - `cargo nextest run -p fret-chart chart_canvas_panel_paints_seeded_chart_marks_on_declarative_path`
    - `cargo nextest run -p fret-chart`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-020-chart-declarative-canvas-capability-baseline`
  - Result:
    - The new behavior test seeds a bar-chart dataset into a controlled `ChartEngine`, renders
      `chart_canvas_panel(...)` through `fret_ui::declarative::render_root(...)`, lays out and
      paints the real `UiTree`, and asserts:
      - the declarative chart subtree has non-zero layout,
      - the engine records the viewport bounds,
      - delinea produces rect marks for the seeded bar chart,
      - the declarative canvas emits non-background, non-zero chart mark quads.
    - This is a pre-delete baseline only; retained chart public/demo/gallery entry points still
      need consumer migration before `fret-chart` can leave the retained bridge allowlist.
- [x] RBX-M3-030 Extend the declarative `fret-chart` baseline to line and scatter marks.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
  - Goal:
    - Prove the declarative `chart_canvas_panel(...)` path paints non-bar chart families before
      migrating line/scatter demos from retained `ChartCanvas`.
  - Validation:
    - `cargo nextest run -p fret-chart chart_canvas_panel_paints_line_and_scatter_marks_on_declarative_path`
    - `cargo nextest run -p fret-chart`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-030-chart-declarative-linescatter-capability-baseline`
  - Result:
    - The new behavior test seeds a controlled line + scatter chart engine, renders it through
      `chart_canvas_panel(...)`, and asserts:
      - delinea produces `Polyline` marks for the line series,
      - delinea produces `Points` marks for the scatter series,
      - the declarative canvas paints the line as a `SceneOp::Path`,
      - the declarative canvas paints scatter points as non-background, non-zero quads.
    - This gives `category_line_demo`-style consumer migration a stronger regression gate, while
      retained axes/visual-map/data-zoom/output gaps remain tracked before broad deletion.
- [x] RBX-M3-040 Move `ChartCanvasOutput` publication onto the declarative chart path.
  - Scope:
    - `ecosystem/fret-chart/src/output.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/retained/output.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
  - Goal:
    - Make `ChartCanvasOutput` a shared, non-retained chart contract and let
      `chart_canvas_panel(...)` publish output snapshots without constructing `retained::ChartCanvas`.
    - Preserve retained output behavior by routing the retained widget through the same shared
      snapshot/update helper.
  - Validation:
    - `cargo nextest run -p fret-chart chart_canvas_panel_publishes_output_model_on_declarative_path`
    - `cargo nextest run -p fret-chart`
  - Evidence:
    - `ecosystem/fret-chart/src/output.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-040-chart-declarative-output-model-publication`
  - Result:
    - Added top-level `ChartCanvasOutput` / `ChartCanvasOutputSnapshot` plus shared helpers for
      link-event batch retention, engine snapshot derivation, and revision updates.
    - Kept `retained::ChartCanvasOutput` as a compatibility re-export while moving the actual type
      out of `retained`.
    - Added `ChartCanvasPanelProps::output_model(...)` and `link_axis_map(...)` so declarative chart
      panels can publish domain windows, brush state, link events, tooltip lines, and output
      revisions.
    - Added a default-feature declarative test that seeds a controlled engine with domain windows,
      brush selection, and link events, renders the real `UiTree`, and asserts output model
      publication without constructing `retained::ChartCanvas`.
    - Existing retained output/linking/tooltip tests still pass through the shared helper, so this
      narrows retained-only policy without dropping current chart capabilities.
- [x] RBX-M3-045 Migrate Gallery chart usage/demo snippets off retained `ChartCanvas` authoring.
  - Scope:
    - `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/grid_axis.rs`
    - `apps/fret-ui-gallery/src/ui/pages/chart.rs`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
  - Goal:
    - Stop first-party shadcn-style chart docs from teaching `RetainedSubtreeProps` /
      `ChartCanvas::new(...)` for ordinary chart body authoring.
    - Use `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)` with a controlled
      `Model<ChartEngine>` and shared `ChartCanvasOutput` instead.
  - Validation:
    - `cargo check -p fret-ui-gallery --features gallery-chart`
    - `cargo nextest run -p fret-ui-gallery --features gallery-chart chart_snippets_prefer_declarative_canvas_panel`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
  - Evidence:
    - `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/demo.rs`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-045-gallery-chart-snippets-use-declarative-panel`
  - Result:
    - The Gallery "First Chart" and demo-card snippets now seed `ChartEngine` models directly and
      render through `chart_canvas_panel_in(...)`.
    - The shared output-model path remains intact for tooltip/legend recipes through
      `ChartCanvasOutput`.
    - A source-policy test prevents `usage.rs` and `demo.rs` from reintroducing retained chart
      authoring markers.
    - Accessibility-specific docs still mention retained `ChartCanvas` because keyboard point
      navigation remains a separate declarative parity slice before that helper can be migrated.
- [x] RBX-M3-050 Move chart accessibility keyboard navigation onto the declarative panel.
  - Scope:
    - `ecosystem/fret-chart/src/a11y.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/accessibility.rs`
    - `apps/fret-ui-gallery/src/ui/pages/chart.rs`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
  - Goal:
    - Give `chart_canvas_panel(...)` retained-compatible focusable chart semantics and arrow-key
      point navigation before deleting retained chart code.
    - Migrate Gallery chart accessibility docs away from retained `ChartCanvas` helper authoring.
  - Validation:
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-ui-gallery --features gallery-chart chart_first_chart_keyboard_navigation_shows_auto_wired_tooltip_under_default_cache_policy chart_snippets_prefer_declarative_canvas_panel`
    - `cargo check -p fret-ui-gallery --features gallery-chart`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
  - Evidence:
    - `ecosystem/fret-chart/src/a11y.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/usage.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/chart/accessibility.rs`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-050-chart-declarative-accessibility-navigation`
  - Result:
    - Added shared crate-private `ChartA11yIndex` so retained and declarative chart paths use the
      same mark-to-data-index accessibility index.
    - Added `ChartCanvasPanelProps::accessibility_layer(true)`, `test_id(...)`, and
      `input_map(...)`.
    - Declarative chart panels now expose a focusable viewport semantics node, collection
      position, tooltip value, and arrow-key navigation that updates hover/axis-pointer output.
    - The new declarative test mirrors the retained keyboard-navigation oracle: `ArrowRight`
      advances `pos_in_set` from 1 to 2 and publishes non-empty tooltip lines to
      `ChartCanvasOutput`.
    - Gallery's first-chart snippet now opts into declarative chart accessibility, and Gallery
      accessibility docs/source-policy no longer teach retained `ChartCanvas` authoring.
- [x] RBX-M3-060 Migrate UI Gallery chart torture off retained `ChartCanvas`.
  - Scope:
    - `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`
    - `apps/fret-ui-gallery/src/harness.rs`
    - `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
  - Goal:
    - Stop the first-party chart torture surface from mounting `ChartCanvas::new_shared(...)`
      through `RetainedSubtreeProps` / `cx.retained_subtree(...)`.
    - Preserve the torture page's shared engine/output diagnostics, explicit Y link-map fixture,
      520px chart viewport, and chart-output snapshot collection through declarative
      `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)`.
  - Validation:
    - `cargo check -p fret-ui-gallery --features gallery-dev`
    - `cargo nextest run -p fret-ui-gallery --features gallery-dev chart_torture_preview_uses_declarative_chart_panel`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-ui-gallery --features gallery-chart -E 'test(chart_snippets_prefer_declarative_canvas_panel) | test(chart_first_chart_keyboard_navigation_shows_auto_wired_tooltip_under_default_cache_policy)'`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
  - Evidence:
    - `apps/fret-ui-gallery/src/ui/previews/pages/torture/chart_torture.rs`
    - `apps/fret-ui-gallery/src/harness.rs`
    - `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_internal_previews.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-060-gallery-chart-torture-uses-declarative-panel`
  - Result:
    - Chart torture now stores the stress `ChartEngine` as a `Model<ChartEngine>` and passes that
      model directly to `ChartCanvasPanelProps`.
    - `UiGalleryChartTortureOutputHandle` keeps `Model<ChartCanvasOutput>` plus the shared engine
      model, so diagnostics still read data zoom, axis output, domain-window, and tooltip state
      from the same live chart instance.
    - The explicit Y link-map fixture still publishes the ambiguous Y domain window through the
      declarative output path.
    - A new internal preview source-policy test rejects reintroducing retained chart torture
      authoring markers.
- [x] RBX-M3-070 Migrate cookbook chart interactions off retained `ChartCanvas`.
  - Scope:
    - `apps/fret-cookbook/examples/chart_interactions_basics.rs`
    - `apps/fret-cookbook/src/lib.rs`
  - Goal:
    - Stop the cookbook interactions example from teaching `ChartCanvas::new_shared(...)` through
      `RetainedSubtreeProps` / `cx.retained_subtree(...)`.
    - Preserve the same app-owned zoom/reset/selection workflow, chart test id, accessibility
      layer, default chart input map, and single live chart engine while mounting the chart through
      `ChartCanvasPanelProps` + `chart_canvas_panel_in(...)`.
  - Validation:
    - `cargo check -p fret-cookbook --example chart_interactions_basics --features cookbook-chart`
    - `cargo nextest run -p fret-cookbook --features cookbook-chart chart_interactions_example_prefers_declarative_chart_panel`
    - `cargo nextest run -p fret-chart`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
  - Evidence:
    - `apps/fret-cookbook/examples/chart_interactions_basics.rs`
    - `apps/fret-cookbook/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-070-cookbook-chart-interactions-use-declarative-panel`
  - Result:
    - Cookbook chart interactions now store the seeded `ChartEngine` as a `Model<ChartEngine>` and
      pass it to `ChartCanvasPanelProps`.
    - The example keeps a shared `ChartCanvasOutput` model, focusable accessibility layer,
      default input map, and stable `cookbook.chart_interactions_basics.canvas` test id on the
      declarative chart panel.
    - App-owned zoom/reset commands update the same engine model that the panel renders, and
      "Select hovered" still reads hover state from the live chart engine.
    - A cookbook source-policy test now requires declarative chart panel markers and rejects
      retained chart authoring markers.
- [x] RBX-M3-080 Migrate basic first-party chart demos off retained `ChartCanvas`.
  - Scope:
    - `apps/fret-examples/src/chart_demo.rs`
    - `apps/fret-examples/src/category_line_demo.rs`
    - `apps/fret-examples/src/horizontal_bars_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
  - Goal:
    - Stop the basic first-party chart demos from creating retained `ChartCanvas` widgets through
      `ChartCanvas::new(...)` / `ChartCanvas::create_node(...)`.
    - Preserve the seeded chart specs and datasets while mounting each demo through
      `fret_ui::declarative::render_root(...)` and `ChartCanvasPanelProps` +
      `chart_canvas_panel(...)`.
  - Validation:
    - `cargo check -p fret-demo --bin chart_demo --bin category_line_demo --bin horizontal_bars_demo`
    - `cargo nextest run -p fret-examples basic_chart_demos_use_declarative_canvas_panel`
    - `cargo nextest run -p fret-chart`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
  - Evidence:
    - `apps/fret-examples/src/chart_demo.rs`
    - `apps/fret-examples/src/category_line_demo.rs`
    - `apps/fret-examples/src/horizontal_bars_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-080-basic-chart-demos-use-declarative-panel`
  - Result:
    - Each demo now stores its seeded `ChartEngine` as a `Model<ChartEngine>` and keeps the
      `ChartSpec` in window state.
    - Each render pass rebuilds the declarative root and observes the chart engine for paint
      invalidation before constructing `ChartCanvasPanelProps`.
    - A source-policy test requires the declarative chart panel markers and rejects retained
      `ChartCanvas` widget authoring markers in these demos.
- [x] RBX-M3-090 Migrate `chart_stress_demo` off retained `ChartCanvas`.
  - Scope:
    - `apps/fret-examples/src/chart_stress_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
  - Goal:
    - Remove the retained `ChartStressCanvas` wrapper and retained `ChartCanvas` node creation from
      the stress demo.
    - Preserve seeded LOD/progressive stress data, continuous redraw, and periodic delinea engine
      stats reporting while mounting through `ChartCanvasPanelProps` + `chart_canvas_panel(...)`.
  - Validation:
    - `cargo check -p fret-demo --bin chart_stress_demo`
    - `cargo nextest run -p fret-examples -E 'test(basic_chart_demos_use_declarative_canvas_panel) | test(chart_stress_demo_uses_declarative_canvas_panel)'`
    - `cargo nextest run -p fret-chart`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
  - Evidence:
    - `apps/fret-examples/src/chart_stress_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-090-chart-stress-demo-uses-declarative-panel`
  - Result:
    - The stress demo now seeds `(ChartEngine, ChartSpec)`, stores the engine as a
      `Model<ChartEngine>`, and renders the chart via `fret_ui::declarative::render_root(...)`.
    - The retained wrapper's delinea stage/emitted stats report is preserved as a declarative
      render report read from the same live engine model.
    - A source-policy test rejects the deleted retained stress wrapper, retained chart node
      creation, and old `avg_canvas_paint` metric.
- [x] RBX-M3-100 Migrate `chart_multi_axis_demo` off retained `ChartCanvas`.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `apps/fret-examples/src/chart_multi_axis_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
  - Goal:
    - Move the multi-axis linked chart demo from retained `ChartCanvas::new_shared(...)`,
      retained chart node creation, and retained `FixedSplit` node composition to declarative
      `ChartCanvasPanelProps` + `chart_canvas_panel(...)`.
    - Preserve linked brush, linked axis pointer, linked domain windows, output model publication,
      diagnostics snapshots, and deterministic diag auto-zoom behavior before deleting retained
      chart code.
  - Validation:
    - `cargo check -p fret-chart`
    - `cargo check -p fret-demo --bin chart_multi_axis_demo`
    - `cargo nextest run -p fret-chart explicit_y_domain_window_propagates_to_second_declarative_chart_output_model`
    - `cargo nextest run -p fret-examples --test basic_chart_demos_surface`
    - `cargo nextest run -p fret-chart`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `apps/fret-examples/src/chart_multi_axis_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-100-chart-multi-axis-demo-uses-declarative-panel`
  - Result:
    - `ChartCanvasPanelProps` now accepts linked brush, linked axis pointer, and linked domain
      window models, and the declarative panel consumes those shared inputs before stepping and
      publishing chart output.
    - A new declarative parity test proves an explicit linked Y domain window can propagate from a
      source output model through `LinkedChartGroup` into a second declarative chart panel output
      model.
    - `chart_multi_axis_demo` now builds `(ChartEngine, ChartSpec, ChartLinkRouter)` pairs, stores
      each engine as a `Model<ChartEngine>`, constructs `LinkedChartGroup` once in window state,
      and renders both charts through a declarative vertical flex root.
    - The demo source-policy gate now rejects retained chart widget authoring, retained split node
      composition, and `Rc<RefCell<ChartEngine>>` in the multi-axis demo.
- [x] RBX-M3-110 Migrate `echarts_multi_grid_demo` off retained multi-grid helpers.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
    - `crates/fret-ui/src/declarative/host_widget.rs`
    - `crates/fret-ui/src/declarative/tests/managed_surface.rs`
    - `apps/fret-examples/src/echarts_multi_grid_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
  - Goal:
    - Move the ECharts multi-grid demo from retained `UniformGrid` /
      `create_multi_grid_chart_canvas_nodes(...)` helpers to model-backed declarative chart panels.
    - Add enough declarative chart panel capability to preserve multi-grid rendering and overlay
      interaction before deleting retained multi-grid chart code.
    - Ensure overlay-only chart panels do not block input outside their visible legend panel.
  - Validation:
    - `cargo check -p fret-chart`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-ui declarative::tests::managed_surface`
    - `cargo check -p fret-demo --bin echarts_multi_grid_demo`
    - `cargo nextest run -p fret-examples --test basic_chart_demos_surface`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
    - `crates/fret-ui/src/declarative/host_widget.rs`
    - `crates/fret-ui/src/declarative/tests/managed_surface.rs`
    - `apps/fret-examples/src/echarts_multi_grid_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-110-echarts-multi-grid-demo-uses-declarative-panels`
  - Result:
    - `ChartCanvasPanelProps` now supports full, per-grid, and overlay-only modes.
    - Per-grid declarative panels publish `plot_viewports_by_grid` without overwriting the shared
      engine's global viewport and paint only series attached to their grid.
    - Overlay-only declarative panels keep legend/tooltip overlay tools while suppressing mark
      rendering and use `ManagedSurface` hit-test masking so input outside the legend falls through
      to underlying grid panels.
    - `echarts_multi_grid_demo` now stores one shared `Model<ChartEngine>`, renders one
      declarative panel per grid plus a top-level overlay-only panel, and no longer teaches retained
      multi-grid chart helper authoring.
- [x] RBX-M3-120 Delete retained chart multi-grid helper surface.
  - Scope:
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/retained/multi_grid.rs`
  - Goal:
    - Delete the retained `UniformGrid` / `create_multi_grid_chart_canvas_nodes(...)` helper island
      after `RBX-M3-110` proved the declarative multi-grid replacement.
    - Remove no-user retained multi-surface constructors (`ChartCanvas::new_grid_view(...)` and
      `ChartCanvas::new_overlay(...)`) plus their shared-engine/mode branches from retained
      `ChartCanvas`.
    - Keep ordinary retained `ChartCanvas` behavior green as the remaining oracle while other chart
      interactions migrate.
  - Validation:
    - `cargo check -p fret-chart`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-demo --bin echarts_multi_grid_demo`
    - `cargo nextest run -p fret-examples --test basic_chart_demos_surface echarts_multi_grid_demo_uses_declarative_grid_panels_and_overlay`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - deleted `ecosystem/fret-chart/src/retained/multi_grid.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-120-retained-chart-multi-grid-helper-deletion`
  - Result:
    - Deleted `retained/multi_grid.rs` and removed its retained public re-export.
    - Removed retained `ChartCanvas::new_grid_view(...)` / `ChartCanvas::new_overlay(...)` and the
      now-unneeded shared-engine/mode/grid-filter branches from retained `ChartCanvas`.
    - Added a `fret-chart` public-surface policy test preventing retained multi-grid helper
      reintroduction.
    - The full `fret-chart` package gate still passes, including retained chart tooltip, legend,
      visual-map, slider, output/linking, keyboard, and accessibility oracle tests.
- [x] RBX-M3-130 Move chart style/tooltip shared contracts out of the retained namespace.
  - Scope:
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/style.rs`
    - `ecosystem/fret-chart/src/tooltip.rs`
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
    - `ecosystem/fret-chart/src/declarative/tooltip_overlay.rs`
    - `ecosystem/fret-chart/src/output.rs`
  - Goal:
    - Move shared chart style and tooltip contracts to top-level `fret-chart` modules so
      declarative chart code no longer depends on `retained::*` naming for non-retained contracts.
    - Keep retained `ChartCanvas` as the remaining behavior oracle while making it consume the same
      top-level style/tooltip contracts.
    - Add a source-policy test that prevents declarative shared contracts from importing these
      contracts through the retained namespace again.
  - Validation:
    - `cargo fmt --check`
    - `cargo check -p fret-chart`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/style.rs`
    - `ecosystem/fret-chart/src/tooltip.rs`
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
    - `ecosystem/fret-chart/src/declarative/tooltip_overlay.rs`
    - `ecosystem/fret-chart/src/output.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-130-chart-style-and-tooltip-contracts-moved-out-of-retained-namespace`
  - Result:
    - Moved `retained/style.rs` and `retained/tooltip.rs` to top-level `style.rs` and
      `tooltip.rs`, then exported them from the crate root.
    - Removed the retained module ownership/re-export of `style` and `tooltip`; retained
      `ChartCanvas` now imports those shared contracts through the top-level modules.
    - Declarative chart panel, legend overlay, tooltip overlay, and output publication now import
      style/tooltip contracts from top-level `fret-chart` modules instead of `crate::retained::*`.
    - The full `fret-chart` package gate still passes, including declarative chart baselines,
      top-level tooltip/style tests, and ordinary retained chart oracle tests.
- [x] RBX-M3-135 Move chart linking output contract off the retained namespace.
  - Scope:
    - `ecosystem/fret-chart/src/linking.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Make `LinkedChartMember` and `LinkedChartGroup` consume top-level `ChartCanvasOutput`
      directly instead of naming the shared output contract through `crate::retained::*`.
    - Add a red/green source-policy test that prevents linking from depending on retained output
      namespace markers again.
  - Validation:
    - `cargo nextest run -p fret-chart chart_linking_does_not_depend_on_retained_output_namespace`
    - `cargo fmt --check`
    - `cargo check -p fret-chart`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/linking.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-135-chart-linking-output-contract-moved-off-retained-namespace`
  - Result:
    - `LinkedChartMember::output` and the linked output snapshot cache now use top-level
      `ChartCanvasOutput`.
    - The new public-surface policy test first failed on `crate::retained::ChartCanvasOutput`, then
      passed after the import was moved to `crate::ChartCanvasOutput`.
    - The full `fret-chart` package gate now passes with 50 tests.
- [x] RBX-M3-140 Delete retained chart output compatibility re-export.
  - Scope:
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/output.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Remove the no-user `retained::ChartCanvasOutput` / `retained::ChartCanvasOutputSnapshot`
      compatibility re-export now that output contracts live at the crate root.
    - Make retained `ChartCanvas` consume top-level `ChartCanvasOutput` directly while keeping it as
      the remaining chart behavior oracle.
    - Add a public-surface policy test that prevents the retained output re-export from returning.
  - Validation:
    - `cargo nextest run -p fret-chart retained_output_reexport_is_removed_from_public_surface`
    - `cargo fmt --check`
    - `cargo check -p fret-chart`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - deleted `ecosystem/fret-chart/src/retained/output.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-140-retained-chart-output-re-export-deletion`
  - Result:
    - Deleted `retained/output.rs` and removed its retained public re-export.
    - Retained `ChartCanvas` now imports `ChartCanvasOutput` from the top-level output module.
    - The full `fret-chart` package gate now passes with 51 tests.
- [x] RBX-M3-145 Remove retained chart widget glob re-export from the crate root.
  - Scope:
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Stop exporting retained chart widgets through the default `fret_chart::*` crate-root surface.
    - Keep the retained module available explicitly as `fret_chart::retained` while remaining
      retained behavior is used as an oracle.
    - Add a public-surface policy test that prevents the crate-root retained glob re-export from
      returning.
  - Validation:
    - `cargo nextest run -p fret-chart retained_widgets_are_not_glob_reexported_from_crate_root`
    - `cargo fmt --check`
    - `cargo check -p fret-chart`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-145-retained-chart-widget-crate-root-glob-re-export-removal`
  - Result:
    - Removed `pub use retained::*` from the crate root, so retained chart widgets require explicit
      `fret_chart::retained` imports.
    - The new policy test initially self-matched its marker string; after switching to a dynamic
      marker, the intended crate-root surface check passed.
    - The full `fret-chart` package gate now passes with 52 tests.
- [x] RBX-M3-150 Move chart legend scroll policy onto shared/default chart logic.
  - Scope:
    - `ecosystem/fret-chart/src/legend_logic.rs`
    - `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Remove duplicated retained/declarative legend scroll max/clamp/wheel policy.
    - Make both retained `ChartCanvas` and declarative legend overlay consume the same shared
      `legend_logic` scroll policy.
    - Keep retained legend scroll oracle coverage green while adding direct shared-policy tests and
      a source-policy guard.
  - Validation:
    - `cargo nextest run -p fret-chart legend_scroll_policy`
    - `cargo nextest run -p fret-chart legend_scroll_clamps_to_content_height`
    - `cargo check -p fret-chart`
    - `cargo fmt --check`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/legend_logic.rs`
    - `ecosystem/fret-chart/src/declarative/legend_overlay.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-150-chart-legend-scroll-policy-moved-to-shared-logic`
  - Result:
    - Added shared `legend_max_scroll_y`, `legend_clamp_scroll_y`, and
      `legend_scroll_after_wheel` policy functions in `legend_logic`.
    - Retained `ChartCanvas` and declarative legend overlay now route wheel scrolling and layout
      scroll clamping through the shared policy.
    - Added shared policy tests for clamp/wheel behavior and content-fits reset behavior, plus a
      source-policy test preventing duplicated legend wheel speed policy in retained/declarative
      paths.
    - The full `fret-chart` package gate now passes with 55 tests.
- [x] RBX-M3-160 Move chart slider math onto shared/default chart logic.
  - Scope:
    - `ecosystem/fret-chart/src/slider_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Remove retained-only ownership of pure data-zoom / visual-map slider math.
    - Keep retained `ChartCanvas` event routing and engine actions as the current oracle, but make
      it consume shared slider policy for norm/value/window calculations.
    - Add shared slider tests and a source-policy guard preventing slider math from moving back
      into retained `ChartCanvas`.
  - Validation:
    - `cargo nextest run -p fret-chart slider_`
    - `cargo nextest run -p fret-chart slider_math_policy_lives_in_shared_logic`
    - `cargo check -p fret-chart`
    - `cargo fmt --check`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/slider_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-160-chart-slider-math-policy-moved-to-shared-logic`
  - Result:
    - Added shared `SliderDragKind`, `slider_norm`, `slider_value_at_x`,
      `slider_value_at_y`, and `slider_window_after_delta` in `slider_logic`.
    - Retained `ChartCanvas` now consumes shared slider math for data-zoom sliders and visual-map
      range dragging while retaining event/state/action orchestration as the oracle.
    - Added shared slider tests and a public-surface policy test that prevents the pure slider math
      functions from returning to retained `ChartCanvas`.
    - The full `fret-chart` package gate now passes with 58 tests.
- [x] RBX-M3-170 Move chart visual-map layout/mapping policy onto shared/default chart logic.
  - Scope:
    - `ecosystem/fret-chart/src/visual_map_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Remove retained-only ownership of pure visual-map track layout, hit-test routing input, domain
      window conversion, and value-to-y mapping policy.
    - Keep retained `ChartCanvas` paint/event/engine orchestration as the current oracle, but make
      it consume shared visual-map geometry and mapping policy.
    - Add shared visual-map tests and a source-policy guard preventing pure visual-map policy from
      moving back into retained `ChartCanvas`.
  - Validation:
    - `cargo nextest run -p fret-chart visual_map`
    - `cargo check -p fret-chart`
    - `cargo fmt --check`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/visual_map_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-170-chart-visual-map-policy-moved-to-shared-logic`
  - Result:
    - Added shared `VisualMapTrackLayout`, `visual_map_track_layouts`,
      `visual_map_track_at`, `visual_map_domain_window`, and `visual_map_y_at_value` in
      `visual_map_logic`.
    - Retained `ChartCanvas` now consumes shared visual-map track layout, hit-test selection,
      domain-window conversion, and value-to-y mapping while retaining paint/event/action
      orchestration as the oracle.
    - Added shared visual-map tests for endpoint y mapping, padding/gap track layout, and track
      hit selection, plus a public-surface policy test that prevents those pure functions from
      returning to retained `ChartCanvas`.
    - The full `fret-chart` package gate now passes with 62 tests.
- [x] RBX-M3-180 Move chart visual-map interaction decision policy onto shared/default chart logic.
  - Scope:
    - `ecosystem/fret-chart/src/visual_map_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Remove retained-only ownership of visual-map piecewise mask/reset/shift-range decision
      policy.
    - Remove retained-only ownership of continuous visual-map handle-vs-pan-vs-jump drag-start
      decision policy.
    - Keep retained `ChartCanvas` as the current event/action oracle while making it consume
      shared visual-map interaction decisions.
  - Validation:
    - `cargo nextest run -p fret-chart visual_map`
    - `cargo check -p fret-chart`
    - `cargo fmt --check`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/visual_map_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-180-chart-visual-map-interaction-policy-moved-to-shared-logic`
  - Result:
    - Added shared `visual_map_full_piece_mask`, `visual_map_current_piece_mask`,
      `visual_map_piece_mask_after_click`, and `visual_map_continuous_drag_start`.
    - Retained `ChartCanvas` now delegates visual-map piecewise mask toggles, reset, shift-range
      selection, and continuous drag-start choice to shared logic.
    - Added shared visual-map tests for piecewise toggle/range/reset and continuous handle/pan/jump
      starts, and extended the source-policy guard so those pure decisions cannot move back into
      retained canvas.
    - The full `fret-chart` package gate now passes with 64 tests.
- [x] RBX-M3-190 Move chart data-zoom slider interaction policy onto shared/default chart logic.
  - Scope:
    - `ecosystem/fret-chart/src/slider_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
  - Goal:
    - Remove retained-only ownership of data-zoom slider handle-vs-pan-vs-jump drag-start policy.
    - Remove retained-only ownership of data-zoom slider drag-update projection and span-anchor
      policy.
    - Keep retained `ChartCanvas` as the current event/action oracle while making it consume
      shared slider interaction decisions.
  - Validation:
    - `cargo nextest run -p fret-chart slider_`
    - `cargo check -p fret-chart`
    - `cargo fmt --check`
    - `cargo nextest run -p fret-chart`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-chart/src/slider_logic.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-190-chart-data-zoom-slider-interaction-policy-moved-to-shared-logic`
  - Result:
    - Added shared `SliderDragPermissions`, `SliderDragStart`, `SliderDragUpdate`,
      `slider_anchor_for_drag_kind`, `slider_drag_start_at_x`, `slider_drag_start_at_y`,
      `slider_drag_update_at_x`, and `slider_drag_update_at_y`.
    - Retained `ChartCanvas` now delegates data-zoom slider drag-start selection, jump-to-window
      start windows, drag update projection, and window-span anchor choice to shared logic.
    - Added shared slider tests for x/y handle, pan, jump, lock, and drag-update behavior, and
      extended the source-policy guard so those pure interaction decisions cannot move back into
      retained canvas.
    - The full `fret-chart` package gate now passes with 67 tests.
- [x] RBX-M3-200 Establish default declarative `fret-plot` line plot baseline and isolate retained plot bridge.
  - Scope:
    - `ecosystem/fret-plot/Cargo.toml`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/{models,state,style}.rs`
    - `ecosystem/fret-plot/src/retained/`
    - `ecosystem/fret-plot/src/lib.rs`
  - Goal:
    - Stop enabling `fret-ui/unstable-retained-bridge` from the default `fret-plot` dependency.
    - Move shared plot data/state/style contracts out of the retained namespace so declarative
      authoring has a default-gated contract surface.
    - Add a default declarative `line_plot_panel(...)` baseline that paints real line-series data
      through `fret_ui::ElementContext::canvas(...)` without constructing retained `PlotCanvas`.
    - Keep retained plot canvases available only behind explicit `compat-retained-canvas` as the
      migration oracle for remaining plot demos and interactions.
  - Validation:
    - `cargo check -p fret-plot`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_seeded_line_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-examples`
    - `cargo metadata --no-deps --format-version 1 | ... fret-plot fret-ui dependency features`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-plot/Cargo.toml`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `ecosystem/fret-plot/src/models.rs`
    - `ecosystem/fret-plot/src/state.rs`
    - `ecosystem/fret-plot/src/style.rs`
    - `apps/fret-examples/Cargo.toml`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-200-default-declarative-fret-plot-line-plot-baseline-and-retained-bridge-isolation`
  - Result:
    - Default `fret-plot` now depends on `fret-ui` without dependency features; the retained bridge
      is reachable only through the explicit `compat-retained-canvas` feature.
    - `fret_plot::{models,state,style}` expose the shared plot contracts on the default surface,
      while `fret_plot::retained` stays compat-gated and `LineChart` remains model-only on the
      default surface.
    - Added `LinePlotPanelProps` and `line_plot_panel(...)` plus a real render/layout/paint test
      that proves seeded line data emits a declarative canvas path without retained `PlotCanvas`.
    - `apps/fret-examples` now opts into `fret-plot/compat-retained-canvas` explicitly so retained
      plot demos remain compile-checked migration oracles without reopening the `fret-plot`
      default dependency feature.
    - The default `fret-plot` package gate passes with 23 tests, and the compat retained oracle
      still compiles under `compat-retained-canvas`.
- [x] RBX-M3-210 Add a first-party declarative line plot demo while keeping retained plot demos as oracles.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `apps/fret-examples/src/plot_declarative_demo.rs`
    - `apps/fret-examples/src/lib.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `apps/fret-demo/src/main.rs`
    - `apps/fret-demo/src/bin/plot_declarative_demo.rs`
  - Goal:
    - Provide an app-facing first-party plot example that uses the default declarative
      `LinePlotPanelProps` / `line_plot_panel` surface.
    - Keep the old retained `plot_demo` and other plot demos intact as migration oracles until
      axes, legend, tooltip/readout, pan/zoom/box/query, and non-line layers have declarative
      parity.
  - Validation:
    - `cargo nextest run -p fret-examples plot_declarative_demo_uses_default_declarative_line_plot_panel`
    - `cargo check -p fret-demo --bin plot_declarative_demo`
    - `cargo check -p fret-demo --bin fret-demo`
    - `cargo check -p fret-plot`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `apps/fret-examples/src/plot_declarative_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-210-first-party-declarative-line-plot-demo`
  - Result:
    - Added `line_plot_panel_in(...)` as an `ElementContextAccess` adapter so app-facing
      `fret::AppUi` code can consume the default declarative plot panel without spelling raw
      `ElementContext`.
    - Added `plot_declarative_demo` through `FretApp` with seeded `LinePlotModel` data and
      `LinePlotPanelProps`, plus demo launcher and standalone bin entries.
    - Added a source-policy test that requires the new declarative plot demo to use
      `fret_plot::declarative` APIs and rejects retained plot widget authoring in that demo.
- [x] RBX-M3-220 Add declarative line plot axes/grid paint baseline.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Move the default declarative line plot beyond series-only painting by adding tick-derived
      grid lines and x/y axis line painting without constructing retained `PlotCanvas`.
    - Keep labels, legend, readout, pan/zoom/query, and non-line layers for later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_axes_and_grid_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_seeded_line_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-220-declarative-line-plot-axesgrid-paint-baseline`
  - Result:
    - Added default declarative axis/grid painting using shared `axis_ticks_scaled(...)` and
      `PlotTransform` data-to-pixel mapping.
    - Added a render/layout/paint test that proves the declarative path emits x/y axis quads,
      tick-derived grid quads, and keeps series paths above those guide layers.
    - Full default `fret-plot` package tests now pass with 24 tests.
- [x] RBX-M3-230 Add declarative line plot legend paint baseline.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Move the default declarative line plot beyond axes/grid by painting per-series legend
      swatches and labels without constructing retained `PlotCanvas`.
    - Keep legend hover/pin/toggle, tooltip/readout, pan/zoom/query, overlays, and non-line layers
      as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_series_legend_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-230-declarative-line-plot-legend-paint-baseline`
  - Result:
    - Added default declarative legend painting for line plots using per-series palette/override
      colors, stable text cache keys, and label text through `CanvasPainter::text(...)`.
    - Added a render/layout/paint test that proves the declarative path emits legend swatch quads,
      legend text ops, and keeps seeded series paths intact.
    - Full default `fret-plot` package tests now pass with 25 tests.
- [x] RBX-M3-240 Add declarative line plot axis tick label baseline.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/axis.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
  - Goal:
    - Move the default declarative line plot beyond guide lines and legend by painting x/y axis
      tick labels without constructing retained `PlotCanvas`.
    - Keep log10 formatting shared with retained plot and leave tooltip/readout, pan/zoom/query,
      overlays, and non-line layers for later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_axis_tick_labels_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/axis.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-240-declarative-line-plot-axis-tick-label-baseline`
  - Result:
    - Added default declarative x/y tick label painting using shared axis label formatting and
      shared log10 decade label helpers.
    - Added a render/layout/paint test that proves the declarative path emits x/y axis tick label
      text ops and keeps seeded series paths intact.
    - `fret-plot` still passes the full package gate and the explicit `compat-retained-canvas`
      check after the shared helper extraction.
- [x] RBX-M3-250 Add declarative line plot managed-host pointer/output proof.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Move the default declarative line plot one step beyond paint-only composition by proving the
      managed-surface host can publish pointer-derived cursor/output state without constructing
      retained `PlotCanvas`.
    - Keep retained tooltip/readout, pan/zoom/query, overlays, and non-line layers for later parity
      slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_updates_output_cursor_on_pointer_move`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-250-declarative-line-plot-managed-host-pointeroutput-proof`
  - Result:
    - Added a declarative managed-surface wrapper around `line_plot_panel(...)` so it can handle
      pointer-move events without going through retained `PlotCanvas`.
    - Added a `PlotOutput` publication path that stores a data-space cursor snapshot derived from
      the pointer position and invalidates paint only when the snapshot changes.
    - Added a focused test that proves pointer moves inside the plot region publish cursor data,
      pointer moves outside clear cursor data, and the declarative series paint path remains intact.
    - The full default `fret-plot` package gate and explicit compat retained check remain green
      after the pointer/output proof.
- [x] RBX-M3-251 Extend declarative line plot cursor readout overlay on the managed-host path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Use the managed-host cursor snapshot to paint the declarative line plot crosshair and
      mouse-coordinate readout overlay without constructing retained `PlotCanvas`.
    - Keep the output publication path intact so the caller-owned `PlotOutput` still tracks cursor
      changes, but do not require an output model just to render the readout.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_updates_output_cursor_on_pointer_move`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-251-declarative-line-plot-cursor-readout-overlay`
  - Result:
    - The declarative line plot now paints cursor crosshair guides and the mouse-coordinate
      readout overlay directly from the managed-host snapshot.
    - The readout rendering works even when no caller-owned `PlotOutput` model is attached, while
      the pointer/output publication path still updates a caller-owned output model when present.
    - The overlay test and the existing pointer/output test both remain green on the default
      `fret-plot` gate, and the compat retained canvas path still compiles.
- [x] RBX-M3-252 Extend declarative line plot linked cursor readout on the managed-host path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Use the managed-host `PlotState` snapshot to paint the declarative line plot linked cursor
      crosshair and linked cursor readout overlay without constructing retained `PlotCanvas`.
    - Keep local cursor readout taking precedence when the pointer is inside the plot region.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-252-declarative-line-plot-linked-cursor-readout-overlay`
  - Result:
    - The declarative line plot now reads `PlotState.linked_cursor_x` and paints the linked cursor
      crosshair and overlay when no local cursor is active.
    - Local pointer hover still takes precedence over linked cursor rendering.
    - The focused linked-cursor test, full `fret-plot` gate, compat retained check, formatting,
      layering, catalog, conflict-marker scan, and whitespace checks all passed.
- [x] RBX-M3-253 Move line plot rich cursor readout rows onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/readout.rs`
    - `ecosystem/fret-plot/src/plot/mod.rs`
    - `ecosystem/fret-plot/src/retained/layers.rs`
  - Goal:
    - Paint per-series cursor readout rows on the default declarative line plot cursor overlays
      without constructing retained `PlotCanvas`.
    - Extract the pure line plot cursor readout interpolation/nearest-point policy out of the
      retained namespace so retained and declarative paths share the same value selection logic.
    - Keep legend hide/pin policy, pan/zoom/query, overlays, non-line layer readouts, and retained
      source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_series_readout_rows_on_declarative_cursor_overlay`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/readout.rs`
    - `ecosystem/fret-plot/src/retained/layers.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-253-declarative-line-plot-rich-cursor-readout-rows`
  - Result:
    - Added a shared default `plot::readout` module for line plot cursor readout rows and Y-value
      lookup policy.
    - The declarative cursor and linked-cursor overlays now append per-series rows such as
      `Alpha: y=...` using the same value lookup policy as the retained line layer.
    - The retained line layer now delegates to the shared default readout helper, while other
      retained layer readouts continue to use the shared Y lookup helper until their declarative
      parity slices are implemented.
    - The focused rich-readout test, full `fret-plot` gate, compat retained check, formatting,
      layering, catalog, conflict-marker scan, and whitespace checks all passed.
- [x] RBX-M3-254 Move line plot legend swatch visibility toggles onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` read `PlotState.hidden_series`, skip
      hidden line series during painting/readout, and update that state when the user clicks a
      legend swatch column.
    - Preserve retained-compatible "do not hide the last visible series" behavior.
    - Keep label-area pin/unpin, shift-click solo/restore, hover emphasis, pan/zoom/query,
      overlays, non-line layers, and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-254-declarative-line-plot-legend-swatch-visibility-toggle`
  - Result:
    - The declarative managed-surface prepaint path now mirrors `PlotState.hidden_series` for
      paint/readout.
    - Declarative legend swatch-column clicks toggle `PlotState.hidden_series`, clear a matching
      pinned series, and stop propagation after requesting a repaint.
    - Hidden series are omitted from declarative line painting and per-series readout rows.
    - The focused legend-toggle test, full `fret-plot` gate, compat retained check, formatting,
      layering, catalog, conflict-marker scan, and whitespace checks all passed.
- [x] RBX-M3-255 Move line plot legend label pin/unpin onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` update `PlotState.pinned_series` when the
      user clicks a legend label area.
    - Preserve retained-compatible label pin/unpin behavior: clicking an unpinned label pins that
      visible series, clicking the pinned label clears the pin, and pinning a hidden series restores
      it to visible.
    - Apply the pinned-series readout filter to declarative local and linked cursor readout rows.
    - Keep shift-click solo/restore, hover emphasis, pan/zoom/query, overlays, non-line layers, and
      retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_legend_label_click_pins_and_unpins_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-255-declarative-line-plot-legend-label-pin-unpin`
  - Result:
    - Declarative legend label clicks now pin/unpin `PlotState.pinned_series` without constructing
      retained `PlotCanvas`.
    - The declarative prepaint path mirrors visible pinned state, clears stale pins for hidden
      series, and local/linked cursor readout rows now follow retained-style pinned-series
      filtering.
    - The focused pin/unpin test started red, then passed along with the full `fret-plot` gate,
      compat retained check, formatting, layering, catalog, conflict-marker scan, and whitespace
      checks.
- [x] RBX-M3-256 Move line plot legend shift-click solo/restore onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` apply retained-compatible
      legend-row `Shift+Click` policy before swatch/label hit semantics.
    - `Shift+Click` on a non-solo row hides every other line series; `Shift+Click` on the already
      solo visible row restores all line series.
    - Keep hover emphasis, pan/zoom/query, overlays, non-line layers, and retained source deletion
      as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-256-declarative-line-plot-legend-shift-click-solo-restore`
  - Result:
    - Declarative legend row `Shift+Click` now solos/restores line series without constructing
      retained `PlotCanvas`.
    - The focused solo/restore test started red, then passed along with the full `fret-plot` gate,
      compat retained check, formatting, layering, catalog, conflict-marker scan, and whitespace
      checks.
- [x] RBX-M3-257 Move line plot legend hover emphasis onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Track legend-row hover on the default declarative `line_plot_panel(...)` managed host without
      constructing retained `PlotCanvas`.
    - Paint retained-style legend row highlight for the hovered row and dim non-hovered line series
      when `LinePlotStyle::emphasize_hovered_series` is enabled.
    - Preserve plot-region pointer move cursor/output behavior when the pointer is not over a
      legend row.
    - Keep pan/zoom/query, overlays, non-line layers, and retained source deletion as later parity
      slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_legend_hover_emphasizes_series_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path line_plot_panel_paints_series_readout_rows_on_declarative_cursor_overlay line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path line_plot_panel_legend_label_click_pins_and_unpins_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-257-declarative-line-plot-legend-hover-emphasis`
  - Result:
    - Declarative legend row hover now paints a row highlight and dims non-hovered line series
      without constructing retained `PlotCanvas`.
    - The integration regression suite caught and fixed an initial hover-event early-return bug
      that had blocked plot-region cursor/output updates.
    - The focused hover-emphasis test, related cursor/readout/legend regression tests, full
      `fret-plot` gate, compat retained check, formatting, layering, catalog, conflict-marker
      scan, and whitespace checks all passed.
- [x] RBX-M3-258 Move line plot controlled view bounds onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` consume caller-owned
      `PlotState.view_is_auto` / `PlotState.view_bounds` before pan/zoom/query behavior migrates
      off retained `PlotCanvas`.
    - Use the same current view bounds for paint, pointer-derived `PlotOutput`, cursor readouts,
      and linked cursor overlays.
    - Preserve default auto-fit behavior with `LinePlotStyle::clamp_to_data_bounds` and
      `overscroll_fraction` when no controlled state is supplied.
    - Keep actual pan/zoom/box/query gestures, overlays, non-line layers, and retained source
      deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_uses_controlled_view_bounds_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-258-declarative-line-plot-controlled-view-bounds`
  - Result:
    - Declarative line plot paint/output/readout now share current view bounds derived from
      caller-owned `PlotState` when `view_is_auto == false`.
    - The focused controlled-view test started red because the event/output path still used
      auto/data bounds, then passed after the event path reads the current `PlotState` for the
      same pointer frame.
    - The focused controlled-view/cursor regression tests, full `fret-plot` gate, compat retained
      check, formatting, layering, catalog, conflict-marker scan, and whitespace checks all passed.
- [x] RBX-M3-259 Move basic line plot pan gesture onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` start a plot-region pan session on plain
      left-button pointer down, update `PlotState.view_bounds` on pointer move, and end the session
      on pointer up/missing button.
    - Keep legend interactions higher priority than pan start, and preserve existing pointer
      output/readout behavior for non-drag pointer moves.
    - Keep wheel zoom, box zoom, query drag, axis locks, overlays, non-line layers, and retained
      source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_pans_controlled_view_bounds_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-259-declarative-line-plot-basic-pan-gesture`
  - Result:
    - Declarative line plot pan now updates caller-owned `PlotState.view_bounds` without
      constructing retained `PlotCanvas`.
    - The focused pan test passed after adding a managed-host pan session; a regression check then
      caught and fixed a legend-swatch priority bug by excluding legend rows from pan start.
    - The focused pan/controlled-view/pointer/legend regression tests, full `fret-plot` gate,
      compat retained check, formatting, layering, catalog, conflict-marker scan, and whitespace
      checks all passed.
- [x] RBX-M3-260 Move basic line plot wheel zoom onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` handle plot-region wheel events without
      constructing retained `PlotCanvas`.
    - Apply the retained default wheel zoom speed and plot-region axis-only modifiers
      (`Shift` for X-only, `Ctrl` for Y-only) to caller-owned `PlotState.view_bounds`.
    - Keep basic pan and legend interactions green while leaving box zoom, query drag, axis regions,
      explicit axis locks, overlays, non-line layers, and retained source deletion as later parity
      slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-260-declarative-line-plot-basic-wheel-zoom`
  - Result:
    - Declarative plot-region wheel events now update caller-owned `PlotState.view_bounds` using
      default retained-compatible wheel zoom speed and axis-only modifier policy.
    - The focused wheel-zoom test started red because wheel events previously left the declarative
      view unchanged, then passed after adding managed-host wheel zoom handling.
    - The focused all-axis/Shift-X/Ctrl-Y wheel tests, wheel/pan/controlled-view/pointer/legend
      regression tests, full `fret-plot` gate, compat retained check, formatting, layering,
      catalog, conflict-marker scan, and whitespace checks all passed.
- [x] RBX-M3-261 Move line plot axis-region wheel zoom onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` handle wheel events over its X-axis and
      Y-axis regions without constructing retained `PlotCanvas`.
    - Preserve retained-compatible routing: wheel over the X-axis zooms only the X range; wheel
      over the Y-axis zooms only the Y range.
    - Keep plot-region wheel zoom, axis-only modifiers, basic pan, and legend interactions green
      while leaving explicit axis locks, box zoom, query drag, overlays, non-line layers, and
      retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-261-declarative-line-plot-axis-region-wheel-zoom`
  - Result:
    - Declarative wheel hit testing now recognizes plot, X-axis, and Y-axis regions.
    - The X-axis focused test started red because axis-region wheel events were previously ignored,
      then passed after routing X-axis wheel events to X-only zoom.
    - X-axis, Y-axis, plot-region all-axis, Shift-X, Ctrl-Y, pan, controlled-view, pointer output,
      and legend regression tests passed before the package and closeout gates.
- [x] RBX-M3-262 Move line plot wheel zoom axis-lock handling onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` honor caller-owned
      `PlotState.axis_locks.x.zoom` and `PlotState.axis_locks.y.zoom` during wheel zoom without
      constructing retained `PlotCanvas`.
    - Preserve retained-compatible behavior: X zoom lock preserves the X range while still allowing
      Y zoom, Y zoom lock preserves the Y range while still allowing X zoom, and locking both axes
      makes wheel zoom a no-op.
    - Keep plot-region wheel zoom, axis-region wheel zoom, axis-only modifiers, basic pan, and
      legend interactions green while leaving pan locks, box zoom, query drag, overlays, non-line
      layers, and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-262-declarative-line-plot-wheel-zoom-axis-locks`
  - Result:
    - Declarative wheel zoom now reads caller-owned `PlotState.axis_locks` and gates the X/Y zoom
      factors before computing the next view bounds.
    - The focused X-lock test started red because declarative wheel zoom changed both axes despite
      `axis_locks.x.zoom == true`, then passed after adding lock gating.
    - X-lock, Y-lock, both-lock no-op, axis-region wheel, plot-region wheel, axis-only modifier,
      pan, controlled-view, pointer output, and legend regression tests passed along with the full
      `fret-plot` and compat retained gates.
- [x] RBX-M3-263 Move line plot pan axis-lock handling onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` honor caller-owned
      `PlotState.axis_locks.x.pan` and `PlotState.axis_locks.y.pan` during the basic pan gesture
      without constructing retained `PlotCanvas`.
    - Preserve retained-compatible behavior: X pan lock preserves the X range while still allowing
      Y pan, Y pan lock preserves the Y range while still allowing X pan, and locking both axes
      makes the pan move a no-op.
    - Keep wheel zoom, wheel zoom axis locks, axis-region wheel zoom, basic pan, controlled view,
      pointer output, and legend interactions green while leaving box zoom, query drag, overlays,
      non-line layers, and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_pan_respects_x_pan_lock_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-263-declarative-line-plot-pan-axis-locks`
  - Result:
    - Declarative pan now reads caller-owned `PlotState.axis_locks` and restores locked X/Y ranges
      after computing the next panned view bounds.
    - The focused X-lock test started red because diagonal declarative panning moved X despite
      `axis_locks.x.pan == true`, then passed after adding pan lock gating.
    - X pan lock, Y pan lock, both-lock no-op, basic pan, wheel zoom lock, axis-region wheel, plot
      wheel, controlled-view, pointer output, and legend regression tests passed along with the full
      `fret-plot` and compat retained gates.
- [x] RBX-M3-264 Move basic line plot box zoom onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` start a retained-compatible right-button
      box-zoom session in the plot region, track the selection on pointer move, and update
      caller-owned `PlotState.view_bounds` on release without constructing retained `PlotCanvas`.
    - Reuse the shared scaled data-rect projection, retained box-select expand modifiers, data
      clamping, and caller-owned zoom locks for the primary X/Y view.
    - Keep wheel zoom, wheel zoom axis locks, axis-region wheel zoom, pan locks, controlled view,
      pointer output, and legend interactions green while leaving query drag, selection painting,
      overlays, non-line layers, and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-264-declarative-line-plot-basic-box-zoom`
  - Result:
    - Declarative box zoom now updates caller-owned `PlotState.view_bounds` from a right-button
      plot-region selection without constructing retained `PlotCanvas`.
    - The focused box-zoom test started red because the declarative path ignored right-button drag
      selection and left the view unchanged, then passed after adding managed-host box-zoom session
      handling.
    - Box zoom, pan locks, wheel zoom locks, axis-region wheel, plot wheel, controlled-view,
      pointer output, and legend regression tests passed along with the full `fret-plot` and compat
      retained gates.
- [x] RBX-M3-265 Move basic line plot query drag onto the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let the default declarative `line_plot_panel(...)` start the retained-compatible
      `Alt+Left` query-drag chord in the plot region, track the selection on pointer move, and
      write caller-owned `PlotState.query` on release without constructing retained `PlotCanvas`.
    - Reuse retained-style raw query projection from plot-local selection points into the current
      primary X/Y data view.
    - Keep box zoom, wheel zoom, wheel zoom axis locks, axis-region wheel zoom, pan locks,
      controlled view, pointer output, and legend interactions green while leaving selection
      painting, query output publication, overlays, non-line layers, and retained source deletion
      as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_query_drag_updates_query_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_query_drag_updates_query_on_declarative_path line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-265-declarative-line-plot-basic-query-drag`
  - Result:
    - Declarative query drag now writes caller-owned `PlotState.query` from an `Alt+Left`
      plot-region selection without constructing retained `PlotCanvas`.
    - The focused query-drag test started red because the declarative path left `PlotState.query`
      empty, then passed after adding managed-host query-drag session handling.
    - Query drag, box zoom, pan locks, wheel zoom locks, axis-region wheel, plot wheel,
      controlled-view, pointer output, and legend regression tests passed along with the full
      `fret-plot` and compat retained gates.
- [x] RBX-M3-266 Publish line plot query output on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Let declarative query drag publish the selected query rectangle through caller-owned
      `PlotOutputSnapshot.query` without constructing retained `PlotCanvas`.
    - Keep subsequent pointer output snapshots carrying caller-owned `PlotState.query` so ordinary
      pointer movement does not clear the observed query selection.
    - Keep query drag, box zoom, wheel zoom, wheel zoom axis locks, axis-region wheel zoom,
      pan locks, controlled view, pointer output, and legend interactions green while leaving
      selection painting, overlays, non-line layers, and retained source deletion as later parity
      slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_query_drag_updates_output_query_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_query_drag_updates_output_query_on_declarative_path line_plot_panel_query_drag_updates_query_on_declarative_path line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-266-declarative-line-plot-query-output-publication`
  - Result:
    - Declarative query drag now publishes the selected query rect through `PlotOutputSnapshot`
      without constructing retained `PlotCanvas`.
    - The focused output test started red because the declarative output snapshot kept `query:
      None`, then passed after query drag release publishes a snapshot with caller-owned
      `PlotState.query`.
    - Query output publication, query drag, box zoom, pan locks, wheel zoom locks, axis-region
      wheel, plot wheel, controlled-view, pointer output, and legend regression tests passed along
      with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-267 Paint line plot query/box selection rectangles on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint active query-drag and active box-zoom selection rectangles on the default declarative
      line plot path without constructing retained `PlotCanvas`.
    - Keep persisted caller-owned `PlotState.query` visible after query release so declarative
      query selection state has the same visual feedback loop as retained plot output/state.
    - Keep query output publication, query drag, box zoom, wheel zoom, wheel zoom axis locks,
      axis-region wheel zoom, pan locks, controlled view, pointer output, and legend interactions
      green while leaving selection tooltips, overlays, non-line layers, and retained source
      deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_query_selection_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_box_zoom_selection_on_declarative_path line_plot_panel_paints_query_selection_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_query_selection_on_declarative_path line_plot_panel_paints_box_zoom_selection_on_declarative_path line_plot_panel_query_drag_updates_output_query_on_declarative_path line_plot_panel_query_drag_updates_query_on_declarative_path line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-267-declarative-line-plot-selection-painting`
  - Result:
    - Declarative line plot now paints active query-drag and active box-zoom selection rectangles
      with retained-style order/bounds without constructing retained `PlotCanvas`.
    - Persisted `PlotState.query` now paints as a declarative selection rectangle after query
      release.
    - The focused query selection test started red because no declarative selection quad was
      painted, then passed after adding active/persisted selection paint state.
    - Selection painting, query output publication, query drag, box zoom, pan locks, wheel zoom
      locks, axis-region wheel, plot wheel, controlled-view, pointer output, and legend regression
      tests passed along with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-268 Paint line plot query/box selection tooltips on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint active query-drag and active box-zoom selection tooltips on the default declarative
      line plot path without constructing retained `PlotCanvas`.
    - Preserve the retained tooltip semantics: active query drag shows `query` plus selected X/Y
      ranges, active box zoom shows `zoom` plus selected X/Y ranges, both using the retained-style
      raw projection from plot-local selection points into the current primary X/Y view.
    - Keep selection painting, query output publication, query drag, box zoom, wheel zoom, wheel
      zoom axis locks, axis-region wheel zoom, pan locks, controlled view, cursor/readout overlays,
      pointer output, and legend interactions green while leaving non-line layers, first-party
      retained plot consumers, and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_query_selection_tooltip_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_query_selection_tooltip_on_declarative_path line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_query_selection_tooltip_on_declarative_path line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path line_plot_panel_paints_query_selection_on_declarative_path line_plot_panel_paints_box_zoom_selection_on_declarative_path line_plot_panel_query_drag_updates_output_query_on_declarative_path line_plot_panel_query_drag_updates_query_on_declarative_path line_plot_panel_box_zoom_updates_controlled_view_bounds_on_declarative_path line_plot_panel_pan_respects_x_pan_lock_on_declarative_path line_plot_panel_pan_respects_y_pan_lock_on_declarative_path line_plot_panel_pan_noops_when_both_axes_locked_on_declarative_path line_plot_panel_pans_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_respects_x_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_respects_y_zoom_lock_on_declarative_path line_plot_panel_wheel_zoom_noops_when_both_axes_locked_on_declarative_path line_plot_panel_wheel_zoom_on_x_axis_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_on_y_axis_zooms_y_only_on_declarative_path line_plot_panel_wheel_zooms_controlled_view_bounds_on_declarative_path line_plot_panel_wheel_zoom_shift_modifier_zooms_x_only_on_declarative_path line_plot_panel_wheel_zoom_ctrl_modifier_zooms_y_only_on_declarative_path line_plot_panel_uses_controlled_view_bounds_on_declarative_path line_plot_panel_updates_output_cursor_on_pointer_move line_plot_panel_legend_swatch_click_toggles_series_visibility_on_declarative_path line_plot_panel_legend_shift_click_solos_and_restores_series_on_declarative_path line_plot_panel_legend_hover_emphasizes_series_on_declarative_path line_plot_panel_paints_cursor_readout_without_output_model_on_declarative_path line_plot_panel_paints_linked_cursor_readout_from_state_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-22---rbx-m3-268-declarative-line-plot-selection-tooltips`
  - Result:
    - Declarative line plot now paints active query-drag and active box-zoom selection tooltips
      with retained-style labels and selected X/Y ranges without constructing retained
      `PlotCanvas`.
    - Selection tooltips use the same raw selection-to-data projection as query drag and box zoom,
      and active selection tooltips take priority over cursor/linked readout overlays while the
      drag is active.
    - The focused query tooltip test started red because active declarative query drag only
      prepared axis/legend text, then passed after adding typed active selection tooltip state and
      painting.
    - Selection tooltips, selection painting, query output publication, query drag, box zoom, pan
      locks, wheel zoom locks, axis-region wheel, plot wheel, controlled-view, cursor/readout
      overlays, pointer output, and legend regression tests passed along with the full `fret-plot`
      and compat retained gates.
- [x] RBX-M3-269 Paint line plot reference-line overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned `PlotState.overlays.inf_lines_x` and `inf_lines_y` on the default
      declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible placement: X reference lines are data-X anchored vertical
      strips spanning the plot region, Y reference lines are data-Y anchored horizontal strips
      spanning the plot region, custom widths/colors are honored, and non-finite or out-of-scale
      values are skipped.
    - Keep selection tooltips, selection painting, query output publication, query drag, box zoom,
      wheel zoom, wheel zoom axis locks, axis-region wheel zoom, pan locks, controlled view,
      cursor/readout overlays, pointer output, and legend interactions green while leaving
      draggable overlays, tags/text/images, non-line layers, first-party retained plot consumers,
      and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_query_selection_tooltip_on_declarative_path line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-269-declarative-line-plot-reference-line-overlays`
  - Result:
    - Declarative line plot now reads caller-owned `PlotState.overlays` and paints X/Y reference
      lines before series painting, without constructing retained `PlotCanvas`.
    - The focused reference-line test initially failed before the implementation produced the
      retained-compatible X/Y line rects, then passed after adding overlay state capture and
      declarative reference-line painting.
    - Reference-line overlays, selection tooltips, selection painting, query output publication,
      query drag, box zoom, pan locks, wheel zoom locks, axis-region wheel, plot wheel,
      controlled-view, cursor/readout overlays, pointer output, and legend regression tests passed
      along with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-270 Paint line plot draggable line overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned `PlotState.overlays.drag_lines_x` and `drag_lines_y` on the default
      declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible initial placement: draggable X lines are data-X anchored
      vertical strips spanning the plot region, draggable left-axis Y lines are data-Y anchored
      horizontal strips spanning the plot region, custom widths/colors are honored, and non-finite
      or out-of-scale values are skipped.
    - Keep reference-line overlays, selection tooltips, selection painting, query output
      publication, query drag, box zoom, wheel zoom, wheel zoom axis locks, axis-region wheel zoom,
      pan locks, controlled view, cursor/readout overlays, pointer output, and legend interactions
      green while leaving draggable overlay interaction/output, right-side axis overlays,
      tags/text/images, non-line layers, first-party retained plot consumers, and retained source
      deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_draggable_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_reference_lines_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_query_selection_tooltip_on_declarative_path line_plot_panel_paints_box_zoom_selection_tooltip_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all && cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-270-declarative-line-plot-draggable-line-overlays`
  - Result:
    - Declarative line plot now paints caller-owned `drag_lines_x` and left-axis `drag_lines_y`
      using the same retained-compatible strip placement helper as reference lines, without
      constructing retained `PlotCanvas`.
    - The focused draggable-line test passed after adding the declarative paint path and locks the
      initial X/Y line positions.
    - Draggable line overlay painting, reference-line overlays, selection tooltips, selection
      painting, query output publication, query drag, box zoom, pan locks, wheel zoom locks,
      axis-region wheel, plot wheel, controlled-view, cursor/readout overlays, pointer output, and
      legend regression tests passed along with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-271 Paint line plot draggable point/rect overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned `PlotState.overlays.drag_points` and left-axis `drag_rects` on the default
      declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible initial placement: draggable points are data-anchored rounded
      quads using the caller radius/color, and draggable rects are data-anchored filled/bordered
      rects using caller border/fill policy.
    - Keep draggable line overlays, reference-line overlays, selection tooltips, selection
      painting, query output publication, query drag, box zoom, wheel zoom, wheel zoom axis locks,
      axis-region wheel zoom, pan locks, controlled view, cursor/readout overlays, pointer output,
      and legend interactions green while leaving draggable overlay interaction/output, right-side
      axis overlays, tags/text/images, non-line layers, first-party retained plot consumers, and
      retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_draggable_point_and_rect_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all && cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-271-declarative-line-plot-draggable-pointrect-overlays`
  - Result:
    - Declarative line plot now paints caller-owned left-axis `drag_points` and `drag_rects`
      before series painting, without constructing retained `PlotCanvas`.
    - The focused point/rect test initially failed on retained-compatible rounded point placement,
      then passed after locking the retained `round` behavior.
    - Draggable point/rect overlay painting, draggable line overlay painting, reference-line
      overlays, selection tooltips, selection painting, query output publication, query drag, box
      zoom, pan locks, wheel zoom locks, axis-region wheel, plot wheel, controlled-view,
      cursor/readout overlays, pointer output, and legend regression tests passed along with the
      full `fret-plot` and compat retained gates.
- [x] RBX-M3-272 Paint line plot PlotText overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned `PlotState.overlays.text` on the default declarative line plot path
      without constructing retained `PlotCanvas`.
    - Preserve retained-compatible left-axis placement: data-anchored text honors caller offset,
      optional background/border/padding/corner policy, and skips non-finite data coordinates.
    - Keep draggable point/rect and line overlays, reference-line overlays, selection tooltips,
      selection painting, query output publication, query drag, box zoom, wheel zoom, wheel zoom
      axis locks, axis-region wheel zoom, pan locks, controlled view, cursor/readout overlays,
      pointer output, and legend interactions green while leaving draggable overlay
      interaction/output, right-side axis overlays, tags/images, non-line layers, first-party
      retained plot consumers, and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_plot_text_overlay_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_plot_text_overlay_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all && cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `ecosystem/fret-plot/src/theme_tokens.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-272-declarative-line-plot-plottext-overlays`
  - Result:
    - Declarative line plot now paints caller-owned left-axis `PlotState.overlays.text` before
      series painting, with retained-compatible background/border/text placement and no retained
      `PlotCanvas`.
    - The focused PlotText test initially failed because no declarative path prepared the overlay
      text, then passed after adding the paint-only overlay helper and exposing the shared theme
      token helpers to the default build.
    - PlotText overlay painting, draggable point/rect overlay painting, draggable line overlay
      painting, reference-line overlays, selection tooltips, selection painting, query output
      publication, query drag, box zoom, pan locks, wheel zoom locks, axis-region wheel, plot
      wheel, controlled-view, cursor/readout overlays, pointer output, and legend regression
      tests passed along with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-273 Paint line plot TagX/TagY overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned `PlotState.overlays.tags_x` and left-axis `tags_y` on the default
      declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible label/marker placement: TagX labels anchor near the bottom of
      the plot with a vertical marker, left-axis TagY labels anchor at the left side with a
      horizontal marker, caller colors drive marker paint, and label text respects label/value
      composition.
    - Keep PlotText overlay painting, draggable point/rect and line overlays, reference-line
      overlays, selection tooltips, selection painting, query output publication, query drag, box
      zoom, wheel zoom, wheel zoom axis locks, axis-region wheel zoom, pan locks, controlled view,
      cursor/readout overlays, pointer output, and legend interactions green while leaving
      draggable overlay labels/interaction/output, right-side axis overlays, images, non-line
      layers, first-party retained plot consumers, and retained source deletion as later parity
      slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path line_plot_panel_paints_plot_text_overlay_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all && cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-273-declarative-line-plot-tagxtagy-overlays`
  - Result:
    - Declarative line plot now paints caller-owned `tags_x` and left-axis `tags_y` before series
      painting, with retained-compatible label backgrounds, text, and bottom/left marker strips,
      without retained `PlotCanvas`.
    - The focused TagX/TagY test initially failed because no declarative path prepared the tag
      label text, then passed after adding the paint-only tag overlay helper.
    - Tag overlay painting, PlotText overlay painting, draggable point/rect overlay painting,
      draggable line overlay painting, reference-line overlays, selection tooltips, selection
      painting, query output publication, query drag, box zoom, pan locks, wheel zoom locks,
      axis-region wheel, plot wheel, controlled-view, cursor/readout overlays, pointer output, and
      legend regression tests passed along with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-274 Paint line plot draggable overlay labels on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint labels for caller-owned `drag_lines_x`, left-axis `drag_lines_y`, and left-axis
      `drag_points` on the default declarative line plot path without constructing retained
      `PlotCanvas`.
    - Preserve retained-compatible label placement: draggable X/Y line labels reuse the TagX/TagY
      label+marker placement, and draggable point labels paint near the point with annotation
      background/border/text styling.
    - Keep TagX/TagY overlays, PlotText overlay painting, draggable point/rect and line overlay
      shapes, reference-line overlays, selection tooltips, selection painting, query output
      publication, query drag, box zoom, wheel zoom, wheel zoom axis locks, axis-region wheel zoom,
      pan locks, controlled view, cursor/readout overlays, pointer output, and legend interactions
      green while leaving draggable overlay interaction/output, right-side axis overlays/labels,
      images, non-line layers, first-party retained plot consumers, and retained source deletion as
      later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_draggable_overlay_labels_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_draggable_overlay_labels_on_declarative_path line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path line_plot_panel_paints_plot_text_overlay_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all && cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-274-declarative-line-plot-draggable-overlay-labels`
  - Result:
    - Declarative line plot now paints labels for `drag_lines_x`, left-axis `drag_lines_y`, and
      left-axis `drag_points` before series painting, without retained `PlotCanvas`.
    - The focused draggable-label test initially failed because no declarative path prepared the
      draggable label text, then passed after adding the paint-only label helper.
    - Draggable overlay label painting, TagX/TagY overlay painting, PlotText overlay painting,
      draggable point/rect overlay painting, draggable line overlay painting, reference-line
      overlays, selection tooltips, selection painting, query output publication, query drag, box
      zoom, pan locks, wheel zoom locks, axis-region wheel, plot wheel, controlled-view,
      cursor/readout overlays, pointer output, and legend regression tests passed along with the
      full `fret-plot` and compat retained gates.
- [x] RBX-M3-275 Paint line plot image overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned left-axis `PlotState.overlays.images` on the default declarative line
      plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible image projection for the primary axis: data rects become
      `SceneOp::ImageRegion` rects, image opacity is clamped, UVs are preserved, and image
      painting is clipped to the plot viewport.
    - Preserve retained layer intent for the primary axis by painting `BelowGrid` images before
      grid/axes and `AboveGrid` images after grid/axes but before overlay/series work.
    - Keep draggable overlay labels, TagX/TagY overlays, PlotText overlay painting, draggable
      point/rect and line overlay shapes, reference-line overlays, selection tooltips, selection
      painting, query output publication, query drag, box zoom, wheel zoom, wheel zoom axis locks,
      axis-region wheel zoom, pan locks, controlled view, cursor/readout overlays, pointer output,
      and legend interactions green while leaving draggable overlay interaction/output,
      right-side axis overlays/labels/images, non-line layers, first-party retained plot consumers,
      and retained source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_plot_image_overlay_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_plot_image_overlay_on_declarative_path line_plot_panel_paints_draggable_overlay_labels_on_declarative_path line_plot_panel_paints_tag_x_and_y_overlays_on_declarative_path line_plot_panel_paints_plot_text_overlay_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_reference_lines_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all && cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref'`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-275-declarative-line-plot-image-overlays`
  - Result:
    - Declarative line plot now paints caller-owned left-axis `PlotState.overlays.images` as
      clipped `ImageRegion` scene ops without retained `PlotCanvas`.
    - The focused image-overlay test initially failed because no declarative path emitted image
      regions, then passed after adding the paint-only image helper.
    - Image overlay painting, draggable overlay label painting, TagX/TagY overlay painting,
      PlotText overlay painting, draggable point/rect overlay painting, draggable line overlay
      painting, reference-line overlays, selection tooltips, selection painting, query output
      publication, query drag, box zoom, pan locks, wheel zoom locks, axis-region wheel, plot
      wheel, controlled-view, cursor/readout overlays, pointer output, and legend regression tests
      passed along with the full `fret-plot` and compat retained gates.
- [x] RBX-M3-280 Paint right-axis line-plot overlays on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned right-axis `PlotImage`, `TagY`, and `PlotText` overlays on the default
      declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible right-axis placement semantics: right-axis images use matching
      right-axis bounds, right-axis TagY labels and markers anchor on the right edge, right-axis
      PlotText uses the matching right-axis transform and right-anchored placement, and primary-axis
      overlay behavior stays intact.
    - Keep left-axis overlays, right-side series projection, first-party examples, and compat
      retained gates green while leaving right-axis draggable labels/interaction/output,
      right-side labels/ticks/readouts beyond series projection, non-line layers, and retained
      source deletion as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_plot_image_overlays_on_declarative_path line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path line_plot_panel_paints_right2_and_right3_axis_series_with_axis_bounds_on_declarative_path line_plot_panel_paints_right_axis_series_with_right_axis_bounds_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-280-declarative-right-axis-line-plot-overlays`
  - Result:
    - Declarative line plot now paints right-axis `PlotImage`, `TagY`, and `PlotText` overlays using
      the matching right-axis transforms and right-anchored placement rules, without retained
      `PlotCanvas`.
    - The focused right-axis overlay test initially failed because the declarative path only handled
      left-axis overlay painting, then passed after adding axis-aware tag/text overlay helpers.
    - Right-axis overlay painting, right-axis image overlay painting, right-axis series projection,
      related overlay/legend smoke tests, the full `fret-plot` package gate, compat retained check,
      formatting, layering, workstream-catalog, diff, and conflict-marker checks passed. The only
      warning observed remains the pre-existing `fret-ui` `current_effective_opacity` dead-code
      warning.

- [x] RBX-M3-281 Paint right-axis draggable overlay labels on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned right-axis `drag_lines_y` and `drag_points` labels on the default
      declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible right-axis label placement: draggable Y-line labels use the
      matching right-axis transform and right-edge TagY placement, and draggable point labels use
      the matching right-axis transform for point-adjacent annotation placement.
    - Keep existing left-axis draggable labels, right-axis TagY/PlotText/image overlays, right-axis
      series projection, full `fret-plot`, and compat retained gates green while leaving right-axis
      draggable interaction/output and non-line layers as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path line_plot_panel_paints_draggable_overlay_labels_on_declarative_path line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path line_plot_panel_paints_right_axis_plot_image_overlays_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-281-declarative-right-axis-draggable-overlay-labels`
  - Result:
    - Declarative line plot now paints right-axis draggable Y-line and point labels using matching
      right-axis transforms and retained-compatible right-edge/point-adjacent placement, without
      retained `PlotCanvas`.
    - The focused test initially failed because the declarative draggable-label painter filtered
      `drag_lines_y` and `drag_points` to `YAxis::Left`; it passed after making that painter
      axis-aware.
    - Focused, related overlay smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-282 Paint right-axis draggable overlay shapes on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Paint caller-owned right-axis `drag_lines_y`, `drag_points`, and `drag_rects` shapes on the
      default declarative line plot path without constructing retained `PlotCanvas`.
    - Preserve retained-compatible right-axis geometry by projecting line/point/rect overlays
      through the matching right-axis bounds while keeping primary-axis draggable overlays green.
    - Move right-axis `InfLineY` reference-line projection through the same axis-aware line painter
      because it shares the same y-axis overlay surface.
    - Keep right-axis draggable labels, TagY/PlotText/image overlays, right-axis series projection,
      full `fret-plot`, and compat retained gates green while leaving right-axis draggable
      interaction/output and non-line layers as later parity slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-282-declarative-right-axis-draggable-overlay-shapes`
  - Result:
    - Declarative line plot now paints right-axis draggable Y lines, points, and rects using
      matching right-axis transforms without retained `PlotCanvas`.
    - The focused test initially failed because the declarative line/shape painters filtered
      y-axis overlays to `YAxis::Left`; it passed after making those painters axis-aware.
    - Focused, related overlay smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-283 Publish right-axis draggable Y-line output on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Add the first declarative draggable-overlay interaction/output slice without constructing
      retained `PlotCanvas`.
    - Hit-test caller-owned right-axis `drag_lines_y` using the matching right-axis transform,
      capture a minimal declarative drag session, and publish `PlotDragOutput::LineY` start/update/
      end snapshots through the existing `PlotOutput` model.
    - Keep pan/query/box interactions from stealing the same plain-left drag-line gesture, while
      leaving `LineX`, point, and rect drag output for follow-up slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_drags_right_axis_y_line_output_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path line_plot_panel_drags_right_axis_y_line_output_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-283-declarative-right-axis-draggable-y-line-output`
  - Result:
    - Declarative line plot now publishes right-axis draggable Y-line output through
      `PlotOutputSnapshot::drag`, including `Start`, `Update`, and `End` phases.
    - The focused test initially failed because the declarative event path did not have a draggable
      overlay session and always published `drag: None`; it passed after adding a minimal
      `LinePlotDragSession::LineY` path ahead of query/box/pan handling.
    - Focused, related smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-284 Publish draggable X-line output on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the declarative draggable-overlay interaction/output slice from right-axis Y lines to
      caller-owned `drag_lines_x`, without constructing retained `PlotCanvas`.
    - Hit-test draggable X lines using the current X view transform, capture a declarative
      `LineX` drag session, and publish `PlotDragOutput::LineX` start/update/end snapshots through
      the existing `PlotOutput` model.
    - Keep right-axis Y-line drag output, draggable line painting, and right-axis draggable shape
      painting green while leaving point and rect drag output for follow-up slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_drags_x_line_output_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_drags_x_line_output_on_declarative_path line_plot_panel_drags_right_axis_y_line_output_on_declarative_path line_plot_panel_paints_draggable_lines_on_declarative_path line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-284-declarative-draggable-x-line-output`
  - Result:
    - Declarative line plot now publishes draggable X-line output through
      `PlotOutputSnapshot::drag`, including `Start`, `Update`, and `End` phases.
    - The focused test initially failed because the declarative drag session only handled Y lines;
      it passed after adding `LinePlotDragSession::LineX` hit testing, movement, and output
      publication.
    - Focused, related smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-285 Publish right-axis draggable point output on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend declarative draggable-overlay interaction/output from line sessions to caller-owned
      `drag_points`, without constructing retained `PlotCanvas`.
    - Prefer point hit-testing over line hit-testing, use the matching y-axis transform for
      right-axis points, and publish `PlotDragOutput::Point` start/update/end snapshots through the
      existing `PlotOutput` model.
    - Keep LineX/LineY drag output, draggable point/rect painting, and right-axis draggable shape
      painting green while leaving rect drag output for a follow-up slice.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_drags_right_axis_point_output_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_drags_right_axis_point_output_on_declarative_path line_plot_panel_drags_x_line_output_on_declarative_path line_plot_panel_drags_right_axis_y_line_output_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-285-declarative-right-axis-draggable-point-output`
  - Result:
    - Declarative line plot now publishes right-axis draggable point output through
      `PlotOutputSnapshot::drag`, including `Start`, `Update`, and `End` phases.
    - The focused test initially failed because the declarative drag session only handled line
      overlays; it passed after adding `LinePlotDragSession::Point` hit testing, movement, and
      output publication.
    - Focused, related smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-286 Publish right-axis draggable rect output on the declarative path.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Complete the declarative draggable-overlay output family by extending line/point sessions to
      caller-owned `drag_rects`, without constructing retained `PlotCanvas`.
    - Hit-test right-axis draggable rect interiors and edge handles using the matching right-axis
      transform, capture a declarative `Rect` drag session, and publish
      `PlotDragOutput::Rect` start/update/end snapshots through the existing `PlotOutput` model.
    - Keep LineX/LineY/Point drag output, draggable point/rect painting, and right-axis draggable
      shape painting green while leaving non-line layers, first-party consumers, and retained
      source deletion for follow-up slices.
  - Validation:
    - `cargo nextest run -p fret-plot line_plot_panel_drags_right_axis_rect_output_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_drags_right_axis_rect_output_on_declarative_path line_plot_panel_drags_right_axis_point_output_on_declarative_path line_plot_panel_drags_x_line_output_on_declarative_path line_plot_panel_drags_right_axis_y_line_output_on_declarative_path line_plot_panel_paints_draggable_point_and_rect_on_declarative_path line_plot_panel_paints_right_axis_draggable_shapes_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-286-declarative-right-axis-draggable-rect-output`
  - Result:
    - Declarative line plot now publishes right-axis draggable rect output through
      `PlotOutputSnapshot::drag`, including `Start`, `Update`, and `End` phases.
    - The focused test initially failed because the declarative drag session only handled lines and
      points; it passed after adding `LinePlotDragSession::Rect` hit testing, movement, and output
      publication.
    - Focused, related smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-287 Migrate first-party `drag_demo` to the declarative line plot panel.
  - Scope:
    - `apps/fret-examples/src/drag_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
  - Goal:
    - Move the first-party draggable overlay demo off `LinePlotCanvas` / `fret_plot::retained::*`
      and onto `LinePlotPanelProps` plus `line_plot_panel_in(...)`, while preserving the existing
      low-level runner shell.
    - Keep the demo's `PlotOutputSnapshot::drag` feedback loop that applies `LineX`, `LineY`,
      `Point`, and `Rect` updates back into caller-owned `PlotState`.
    - Add a source-policy regression so `drag_demo` cannot teach retained plot authoring again.
  - Validation:
    - RED: `cargo nextest run -p fret-examples drag_demo_uses_default_declarative_line_plot_panel`
    - GREEN: `cargo nextest run -p fret-examples drag_demo_uses_default_declarative_line_plot_panel`
    - `cargo nextest run -p fret-examples plot_declarative_demo_uses_default_declarative_line_plot_panel tags_demo_uses_default_declarative_line_plot_panel plot_image_demo_uses_default_declarative_line_plot_panel drag_demo_uses_default_declarative_line_plot_panel`
    - `cargo check -p fret-examples --lib`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/drag_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-287-declarative-drag-demo`
  - Result:
    - `drag_demo` now renders its plot through `declarative::RenderRootContext`,
      `LinePlotPanelProps`, and `line_plot_panel_in(...)` instead of constructing
      `LinePlotCanvas`.
    - The existing output feedback loop now reads drag output after pointer down/move/up and
      applies `LineX`, `LineY`, `Point`, and `Rect` outputs back to the caller-owned overlay state.
    - Focused, related first-party source-policy, `fret-examples` library compile, formatting,
      layering, workstream-catalog, diff, and conflict-marker gates passed.

- [x] RBX-M3-288 Migrate first-party `plot_stress_demo` to the declarative line plot panel.
  - Scope:
    - `apps/fret-examples/src/plot_stress_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
  - Goal:
    - Move the pure line-plot stress harness off `LinePlotCanvas` / `fret_plot::retained::*` and
      onto `LinePlotPanelProps` plus `line_plot_panel_in(...)`, while preserving the existing
      low-level runner, perf reporting, and animated-bounds model updates.
    - Keep the stress model on default-gated `LinePlotModel` / `LineSeries` authoring so this
      first-party perf harness no longer teaches retained plot canvas construction.
    - Extend the first-party source-policy regression so `plot_stress_demo` cannot regress to
      retained plot authoring.
  - Validation:
    - RED: `cargo nextest run -p fret-examples plot_stress_demo_uses_default_declarative_line_plot_panel`
    - GREEN: `cargo nextest run -p fret-examples plot_stress_demo_uses_default_declarative_line_plot_panel`
    - `cargo nextest run -p fret-examples plot_declarative_demo_uses_default_declarative_line_plot_panel tags_demo_uses_default_declarative_line_plot_panel plot_image_demo_uses_default_declarative_line_plot_panel drag_demo_uses_default_declarative_line_plot_panel plot_stress_demo_uses_default_declarative_line_plot_panel`
    - `cargo check -p fret-examples --lib`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/plot_stress_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-288-declarative-plot-stress-demo`
  - Result:
    - `plot_stress_demo` now renders its plot through `declarative::RenderRootContext`,
      `LinePlotPanelProps`, and `line_plot_panel_in(...)` instead of constructing
      `LinePlotCanvas`.
    - The stress model and animated bounds still use the same `Model<LinePlotModel>` path, while
      plot authoring now stays on default declarative APIs.
    - Focused, related first-party source-policy, `fret-examples` library compile, formatting,
      layering, workstream-catalog, diff, and conflict-marker gates passed.

- [x] RBX-M3-289 Add declarative right-axis tick label formatter props.
  - Scope:
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Add `LinePlotPanelProps::y2_axis_labels(...)`, `y3_axis_labels(...)`, and
      `y4_axis_labels(...)` so declarative line plots can consume caller-owned right-axis
      formatter policy instead of requiring retained `LinePlotCanvas`.
    - Paint right-axis tick label text for `YAxis::Right`, `YAxis::Right2`, and `YAxis::Right3`
      using the matching right-axis bounds and caller formatter.
    - Keep existing primary axis labels, right-axis series projection, right-axis overlays, and
      draggable output behavior green while leaving right-side layout/readout parity and demo
      migration for follow-up slices.
  - Validation:
    - RED: `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_tick_labels_with_custom_formatters_on_declarative_path`
    - GREEN: `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_tick_labels_with_custom_formatters_on_declarative_path`
    - `cargo nextest run -p fret-plot line_plot_panel_paints_right_axis_tick_labels_with_custom_formatters_on_declarative_path line_plot_panel_paints_axis_tick_labels_on_declarative_path line_plot_panel_paints_right_axis_series_with_right_axis_bounds_on_declarative_path line_plot_panel_paints_right2_and_right3_axis_series_with_axis_bounds_on_declarative_path line_plot_panel_paints_right_axis_draggable_overlay_labels_on_declarative_path line_plot_panel_paints_right_axis_tag_y_and_plot_text_overlays_on_declarative_path`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-289-declarative-right-axis-label-formatters`
  - Result:
    - Declarative line plots now expose `y2_axis_labels(...)`, `y3_axis_labels(...)`, and
      `y4_axis_labels(...)` builder methods and use those formatters when preparing right-side
      tick label text.
    - The focused test initially failed at compile time because the methods did not exist; it
      passed after adding the props and right-axis tick label paint helper.
    - Focused, related smoke, full `fret-plot`, compat retained, formatting, layering,
      workstream-catalog, diff, and conflict-marker gates passed. The only warning observed remains
      the pre-existing `fret-ui` `current_effective_opacity` dead-code warning.

- [x] RBX-M3-290 Migrate first-party `inf_lines_demo` to the declarative line plot panel.
  - Scope:
    - `apps/fret-examples/src/inf_lines_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
  - Goal:
    - Move the first-party inf-line demo off `LinePlotCanvas` / `fret_plot::retained::*` and onto
      `LinePlotPanelProps` plus `line_plot_panel_in(...)`, while preserving the existing low-level
      runner, inf-line overlay state, output logging, and custom right-axis label formatters.
    - Keep the demo on default-gated `LinePlotModel` / `LineSeries` authoring so it no longer
      teaches retained plot canvas construction.
    - Extend the first-party source-policy regression so `inf_lines_demo` cannot regress to
      retained plot authoring.
  - Validation:
    - RED: `cargo nextest run -p fret-examples inf_lines_demo_uses_default_declarative_line_plot_panel`
    - GREEN: `cargo nextest run -p fret-examples inf_lines_demo_uses_default_declarative_line_plot_panel`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" apps/fret-examples/src/inf_lines_demo.rs apps/fret-examples/tests/basic_plot_demos_surface.rs docs/workstreams/retained-bridge-exit-v1 ecosystem/fret-plot/src/declarative.rs ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/theme_tokens.rs`
  - Evidence:
    - `apps/fret-examples/src/inf_lines_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-290-declarative-inf-lines-demo-migration`
  - Result:
    - `inf_lines_demo` now renders its plot through `declarative::RenderRootContext`,
      `LinePlotPanelProps`, and `line_plot_panel_in(...)` instead of constructing
      `LinePlotCanvas`.
    - The demo keeps its caller-owned inf-line overlays, pointer/key output logging, and custom
      y2/y3/y4 axis label formatters on the declarative path.
    - `basic_plot_demos_surface` now prevents `inf_lines_demo` from reintroducing retained plot
      imports or `PlotCanvas` authoring while requiring the declarative panel path to stay visible
      in the source.
    - Focused source-policy, `fret-examples` library compile, formatting, layering, workstream
      catalog, diff, and conflict-marker gates passed.

- [x] RBX-M3-291 Migrate first-party `plot_demo` to the declarative line plot panel.
  - Scope:
    - `apps/fret-examples/src/plot_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
  - Goal:
    - Move the first-party LogX / multi-axis plot demo off `LinePlotCanvas` / `fret_plot::retained::*`
      and onto `LinePlotPanelProps` plus `line_plot_panel_in(...)`, while preserving the existing
      low-level runner, query output logging, and custom right-axis label formatters.
    - Keep the demo on default-gated `LinePlotModel` / `LineSeries` authoring so it no longer
      teaches retained plot canvas construction.
    - Extend the first-party source-policy regression so `plot_demo` cannot regress to retained
      plot authoring.
  - Validation:
    - RED: `cargo nextest run -p fret-examples plot_demo_uses_default_declarative_line_plot_panel`
    - GREEN: `cargo nextest run -p fret-examples plot_demo_uses_default_declarative_line_plot_panel`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" apps/fret-examples/src/plot_demo.rs apps/fret-examples/tests/basic_plot_demos_surface.rs docs/workstreams/retained-bridge-exit-v1 ecosystem/fret-plot/src/declarative.rs ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/theme_tokens.rs`
  - Evidence:
    - `apps/fret-examples/src/plot_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-291-declarative-plot-demo-migration`
  - Result:
    - `plot_demo` now renders its plot through `declarative::RenderRootContext`,
      `LinePlotPanelProps`, and `line_plot_panel_in(...)` instead of constructing
      `LinePlotCanvas`.
    - The demo keeps its LogX scale, multi-axis formatting, and query output logging on the
      declarative path.
    - `basic_plot_demos_surface` now prevents `plot_demo` from reintroducing retained plot imports
      or `PlotCanvas` authoring while requiring the declarative panel path to stay visible in the
      source.
    - Focused source-policy, `fret-examples` library compile, formatting, layering, workstream
      catalog, diff, and conflict-marker gates passed.

- [x] RBX-M3-292 Remove the default `unstable-retained-bridge` dependency from `fret-chart`.
  - Scope:
    - `ecosystem/fret-chart/Cargo.toml`
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/lib.rs` public-surface policy tests
    - `tools/check_layering.py`
  - Goal:
    - Remove `fret-ui/unstable-retained-bridge` from the default `fret-chart` dependency and make
      the retained canvas path available only through an explicit `compat-retained-canvas`
      feature.
    - Keep the retained chart oracle module feature-gated rather than compiled into the default
      chart crate surface.
    - Add a source-policy regression that prevents the default chart crate dependency from turning
      the retained bridge back on.
    - Remove `fret-chart` from the direct retained-bridge dependency allowlist after the default
      dependency no longer enables the bridge; keep only an explicit compatibility feature mapping
      until the retained chart oracle can be deleted.
  - Validation:
    - RED: `cargo nextest run -p fret-chart default_chart_dependency_does_not_enable_unstable_retained_bridge`
    - GREEN: `cargo nextest run -p fret-chart default_chart_dependency_does_not_enable_unstable_retained_bridge`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/Cargo.toml ecosystem/fret-chart/src/lib.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/Cargo.toml`
    - `ecosystem/fret-chart/src/lib.rs`
    - `tools/check_layering.py`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-292-fret-chart-default-retained-bridge-removal`
  - Result:
    - `fret-chart` now keeps the retained canvas path behind an explicit
      `compat-retained-canvas` feature and no longer enables `fret-ui/unstable-retained-bridge`
      from the default dependency.
    - The retained module is feature-gated in `lib.rs`, so the default public surface stays on the
      declarative chart APIs.
    - `fret-chart` is no longer permitted to enable the retained bridge from its default
      `fret-ui` dependency; it remains listed only under the explicit
      `compat-retained-canvas` feature-mapping allowlist until retained chart source is deleted.
      `RBX-M4-032` later turned that compat feature into a no-op and removed the chart mapping.
    - The new default policy regression prevents the default chart crate dependency from silently
      re-enabling the retained bridge.
    - Default and compat chart crate checks, package tests, formatting, layering, workstream
      catalog, diff, and conflict-marker gates passed.

- [x] RBX-M3-293 Migrate first-party `bars_demo` from retained plot bars to the declarative chart
      panel.
  - Scope:
    - `apps/fret-examples/src/bars_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
  - Goal:
    - Remove `bars_demo`'s dependency on `fret_plot::retained::BarsPlotCanvas` /
      `BarsPlotModel` / `PlotState` / `PlotOutput`.
    - Rebuild the demo as a `delinea::ChartEngine` / `ChartSpec` bar chart rendered through
      `ChartCanvasPanelProps` plus `chart_canvas_panel(...)`.
    - Preserve the low-level runner shell and keep output visibility by logging published
      `ChartCanvasOutput.tooltip_lines`.
    - Add a first-party source-policy regression so `bars_demo` cannot teach retained plot bar
      authoring again.
  - Validation:
    - RED/GREEN: `cargo nextest run -p fret-examples bars_demo_uses_declarative_canvas_panel`
    - `cargo nextest run -p fret-examples --test basic_chart_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/bars_demo.rs`
    - `apps/fret-examples/tests/basic_chart_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-293-declarative-bars-demo-migration`
  - Result:
    - `bars_demo` now renders a `SeriesKind::Bar` chart through a caller-owned
      `ChartEngine` model and `chart_canvas_panel(...)` instead of constructing retained
      `BarsPlotCanvas`.
    - The demo publishes and logs declarative chart tooltip output through `ChartCanvasOutput`.
    - `basic_chart_demos_surface` prevents `bars_demo` from reintroducing retained `fret-plot`
      bar authoring or retained plot output/state types.

- [x] RBX-M3-294 Migrate the top line plot in `linked_cursor_demo` to the declarative line plot
      panel.
  - Scope:
    - `apps/fret-examples/src/linked_cursor_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
  - Goal:
    - Move the top half of `linked_cursor_demo` off retained `LinePlotCanvas` authoring and onto
      `LinePlotPanelProps` plus `line_plot_panel_in(...)`.
    - Preserve the split demo shell, linked cursor group, and the bottom retained area plot while
      the remaining area migration stays on the retained compatibility path.
    - Add a source-policy regression so the top line plot cannot regress to retained line plot
      authoring.
  - Validation:
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" apps/fret-examples/src/linked_cursor_demo.rs apps/fret-examples/tests/basic_plot_demos_surface.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/linked_cursor_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-294-linked-cursor-top-line-plot-declarative-migration`
  - Result:
    - The top line plot in `linked_cursor_demo` now renders through `declarative::RenderRootContext`
      and `line_plot_panel_in(...)` instead of `LinePlotCanvas`.
    - The demo keeps the bottom retained area plot and split-shell behavior intact.
    - `basic_plot_demos_surface` now requires the top line plot to stay on the declarative path and
      rejects retained line plot authoring for that half of the demo.

- [x] RBX-M3-295 Migrate the area plot in `area_demo` and the bottom half of `linked_cursor_demo`
      to the declarative area plot panel.
  - Scope:
    - `apps/fret-examples/src/area_demo.rs`
    - `apps/fret-examples/src/linked_cursor_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/readout.rs`
  - Goal:
    - Extract a reusable declarative area plot panel/model/series skeleton that can share the
      existing line-plot interaction/readout machinery.
    - Move `area_demo` off retained `AreaPlotCanvas` authoring and onto `AreaPlotPanelProps` plus
      `area_plot_panel_in(...)`.
    - Move the bottom retained area half of `linked_cursor_demo` onto the same declarative area
      path while preserving the split-shell and linked-cursor behavior.
    - Add source-policy coverage so both demos keep teaching the declarative area path instead of
      retained area plot construction.
  - Validation:
    - `cargo nextest run -p fret-plot area_plot_panel_paints_area_fill_and_stroke_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/area_demo.rs`
    - `apps/fret-examples/src/linked_cursor_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/readout.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-295-declarative-area-plot-panel-migration`
  - Result:
    - `AreaPlotPanelProps`, `area_plot_panel(...)`, and `area_plot_panel_in(...)` now exist on the
      default declarative path.
    - `plot_cursor_readout(...)` now accepts a reusable series iterator so line and area panels can
      share the same readout skeleton.
    - `area_demo` and the bottom area half of `linked_cursor_demo` now render through the
      declarative area plot panel, and the example source-policy tests reject retained area plot
      authoring for both demos.

- [x] RBX-M3-296 Migrate `stems_demo` to the declarative stems plot panel.
  - Scope:
    - `apps/fret-examples/src/stems_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extract a reusable declarative stems panel that can share the existing plot model, readout,
      and interaction skeleton.
    - Move `stems_demo` off retained `StemsPlotCanvas` authoring and onto
      `StemsPlotPanelProps` plus `stems_plot_panel_in(...)`.
    - Preserve the demo's drag-zoom/query shell and logging behavior while teaching the
      declarative stems path.
    - Add source-policy coverage so the demo keeps teaching the declarative stems path instead of
      retained stems plot construction.
  - Validation:
    - `cargo nextest run -p fret-plot stems_plot_panel_paints_stems_from_baseline_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/stems_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-296-declarative-stems-plot-panel-migration`
  - Result:
    - `StemsPlotPanelProps`, `stems_plot_panel(...)`, and `stems_plot_panel_in(...)` now exist on
      the default declarative path.
    - The declarative stems panel reuses the shared plot-model/readout skeleton and renders the
      baseline-to-point stem strokes without retained canvas construction.
    - `stems_demo` and the example source-policy tests now teach the declarative stems path and
      reject retained stems plot authoring.

- [x] RBX-M3-297 Migrate `stairs_demo` to the declarative line plot panel with step mode.
  - Scope:
    - `apps/fret-examples/src/stairs_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative line plot panel with a plot-wide `StepMode` knob.
    - Move `stairs_demo` off retained `StairsPlotCanvas` authoring and onto
      `LinePlotPanelProps` plus `line_plot_panel_in(...)` with `StepMode::Post`.
    - Preserve the demo's drag-zoom/query shell and logging behavior while teaching the
      declarative stair-step path.
    - Add source-policy coverage so the demo keeps teaching the declarative step-mode path instead
      of retained stairs plot construction.
  - Validation:
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/stairs_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-297-declarative-stairs-plot-panel-migration`
  - Result:
    - `LinePlotPanelProps` now accepts `StepMode` and the shared declarative panel turns the line
      polyline into step commands before stroke painting.
    - `stairs_demo` now renders through `RenderRootContext` plus `line_plot_panel_in(...)` instead
      of retained `StairsPlotCanvas`.
    - The example source-policy tests now teach the declarative step-mode path and reject retained
      stairs plot authoring.

- [x] RBX-M3-298 Migrate `shaded_demo` to the declarative shaded plot panel.
  - Scope:
    - `apps/fret-examples/src/shaded_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/readout.rs`
  - Goal:
    - Extend the reusable declarative plot panel with a shaded-band series shape backed by
      `ShadedPlotModel` / `ShadedSeries`.
    - Paint a closed upper/lower band fill plus upper and lower strokes without constructing
      retained `ShadedPlotCanvas`.
    - Preserve the demo's drag-zoom/query shell, output logging, and time-axis labels while
      teaching the declarative shaded path.
    - Add source-policy coverage so the demo keeps teaching declarative shaded plot authoring
      instead of retained shaded plot construction.
  - Validation:
    - `cargo nextest run -p fret-plot shaded_plot_panel_paints_band_fill_and_two_strokes_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "ShadedPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/shaded_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/shaded_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `ecosystem/fret-plot/src/plot/readout.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-298-declarative-shaded-plot-panel-migration`
  - Result:
    - `ShadedPlotPanelProps`, `shaded_plot_panel(...)`, and `shaded_plot_panel_in(...)` now exist
      on the default declarative path.
    - The shared declarative plot panel paints shaded bands as one closed fill path plus upper and
      lower stroke paths.
    - Shaded readout rows now preserve retained-compatible upper/lower labels by owning readout
      label text.
    - `shaded_demo` now renders through `RenderRootContext` plus `shaded_plot_panel_in(...)`
      instead of retained `ShadedPlotCanvas`.
    - The example source-policy tests now teach the declarative shaded path and reject retained
      shaded plot authoring.

- [x] RBX-M3-299 Migrate `error_bars_demo` to the declarative error-bars plot panel.
  - Scope:
    - `apps/fret-examples/src/error_bars_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative plot panel with an error-bars series shape backed by
      `ErrorBarsPlotModel` / `ErrorBarsSeries`.
    - Paint X/Y error lines, caps, and point markers without constructing retained
      `ErrorBarsPlotCanvas`.
    - Preserve the demo's drag-zoom/query shell and output logging while teaching the declarative
      error-bars path.
    - Add source-policy coverage so the demo keeps teaching declarative error-bars plot authoring
      instead of retained error-bars plot construction.
  - Validation:
    - `cargo nextest run -p fret-plot error_bars_plot_panel_paints_x_y_errors_caps_and_markers_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "ErrorBarsPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/error_bars_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/error_bars_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-299-declarative-error-bars-plot-panel-migration`
  - Result:
    - `ErrorBarsPlotPanelProps`, `error_bars_plot_panel(...)`, and
      `error_bars_plot_panel_in(...)` now exist on the default declarative path.
    - The shared declarative plot panel paints error bars as open stroke commands covering X/Y
      error segments, caps, and configurable point markers.
    - `error_bars_demo` now renders through `RenderRootContext` plus
      `error_bars_plot_panel_in(...)` instead of retained `ErrorBarsPlotCanvas`.
    - The example source-policy tests now teach the declarative error-bars path and reject retained
      error-bars plot authoring.

- [x] RBX-M3-300 Migrate `histogram_demo` to the declarative histogram plot panel.
  - Scope:
    - `apps/fret-examples/src/histogram_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative plot panel with a 1D histogram series shape backed by
      `HistogramPlotModel` / `HistogramSeries`.
    - Paint histogram bins as closed fill path rectangles without constructing retained
      `HistogramPlotCanvas`.
    - Preserve the demo's drag-zoom/query shell, output logging, and nearest-at-cursor style while
      teaching the declarative histogram path.
    - Add source-policy coverage so the demo keeps teaching declarative histogram plot authoring
      instead of retained histogram plot construction.
  - Validation:
    - `cargo nextest run -p fret-plot histogram_plot_panel_paints_closed_bin_fill_paths_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "HistogramPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/histogram_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/histogram_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-300-declarative-histogram-plot-panel-migration`
  - Result:
    - `HistogramPlotPanelProps`, `histogram_plot_panel(...)`, and
      `histogram_plot_panel_in(...)` now exist on the default declarative path.
    - The shared declarative plot panel converts histogram bins into a sorted series for readout
      and paints each non-empty bin as a closed filled rectangle path using the real bin width.
    - `histogram_demo` now renders through `RenderRootContext` plus `histogram_plot_panel_in(...)`
      instead of retained `HistogramPlotCanvas`.
    - The example source-policy tests now teach the declarative histogram path and reject retained
      histogram plot authoring.

- [x] RBX-M3-301 Migrate grouped/stacked bars demos to the declarative bars plot panel.
  - Scope:
    - `apps/fret-examples/src/grouped_bars_demo.rs`
    - `apps/fret-examples/src/stacked_bars_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative plot panel with a bars series shape backed by
      `BarsPlotModel` / `BarSeries`.
    - Paint grouped and stacked bars as closed fill path rectangles without constructing retained
      `BarsPlotCanvas`.
    - Preserve both demos' drag-zoom/query shells, output logging, and nearest-at-cursor style
      while teaching the declarative bars path.
    - Add source-policy coverage so both demos keep teaching declarative bars plot authoring
      instead of retained bars plot construction.
  - Validation:
    - `cargo nextest run -p fret-plot bars_plot_panel_paints_grouped_and_stacked_closed_fill_paths_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "BarsPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/grouped_bars_demo.rs apps/fret-examples/src/stacked_bars_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/grouped_bars_demo.rs`
    - `apps/fret-examples/src/stacked_bars_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-301-declarative-grouped-and-stacked-bars-plot-panel-migration`
  - Result:
    - `BarsPlotPanelProps`, `bars_plot_panel(...)`, and `bars_plot_panel_in(...)` now exist on the
      default declarative path.
    - The shared declarative plot panel paints grouped bars and stacked bars, including
      per-index stacked baselines, as closed filled rectangle path commands.
    - `grouped_bars_demo` and `stacked_bars_demo` now render through `RenderRootContext` plus
      `bars_plot_panel_in(...)` instead of retained `BarsPlotCanvas`.
    - The example source-policy tests now teach the declarative bars path and reject retained bars
      plot authoring.

- [x] RBX-M3-302 Migrate `candlestick_demo` to the declarative candlestick plot panel.
  - Scope:
    - `apps/fret-examples/src/candlestick_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative plot panel with a candlestick series shape backed by
      `CandlestickPlotModel` / `CandlestickSeries`.
    - Paint wick strokes plus up/down candle bodies as declarative paths without constructing
      retained `CandlestickPlotCanvas`.
    - Preserve the demo's drag-zoom/query shell, output logging, and OHLC model setup while
      teaching the declarative candlestick path.
    - Add source-policy coverage so `candlestick_demo` keeps teaching declarative candlestick plot
      authoring instead of retained candlestick canvas construction.
  - Validation:
    - `cargo nextest run -p fret-plot candlestick_plot_panel_paints_wicks_and_up_down_bodies_on_declarative_path`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "CandlestickPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/candlestick_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/candlestick_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-302-declarative-candlestick-plot-panel-migration`
  - Result:
    - `CandlestickPlotPanelProps`, `candlestick_plot_panel(...)`, and
      `candlestick_plot_panel_in(...)` now exist on the default declarative path.
    - The shared declarative plot panel carries optional candlestick metadata, uses close-series
      data for cursor/readout rows, and paints wick strokes plus up/down closed candle body paths.
    - `candlestick_demo` now renders through `RenderRootContext` plus
      `candlestick_plot_panel_in(...)` instead of retained `CandlestickPlotCanvas`.
    - The example source-policy test now teaches the declarative candlestick path and rejects
      retained candlestick plot authoring.

- [x] RBX-M3-303 Migrate `heatmap_demo` to the declarative heatmap plot panel.
  - Scope:
    - `apps/fret-examples/src/heatmap_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative plot panel with heatmap grid support backed by
      `HeatmapPlotModel`.
    - Paint finite heatmap cells as declarative quads and paint the default retained-compatible
      colorbar without constructing retained `HeatmapPlotCanvas`.
    - Preserve the demo's drag-zoom/query shell, output logging, and generated grid model while
      teaching the declarative heatmap path.
    - Add source-policy coverage so `heatmap_demo` keeps teaching declarative heatmap plot
      authoring instead of retained heatmap canvas construction.
  - Validation:
    - `cargo nextest run -p fret-plot 'heatmap_plot_panel_paints'`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "HeatmapPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/heatmap_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/heatmap_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-303-declarative-heatmap-plot-panel-migration`
  - Result:
    - `HeatmapPlotPanelProps`, `heatmap_plot_panel(...)`, and `heatmap_plot_panel_in(...)` now
      exist on the default declarative path.
    - The shared declarative plot panel carries optional heatmap metadata, paints visible finite
      cells as declarative quads, and paints a default colorbar with min/max labels.
    - `heatmap_demo` now renders through `RenderRootContext` plus `heatmap_plot_panel_in(...)`
      instead of retained `HeatmapPlotCanvas`.
    - The example source-policy test now teaches the declarative heatmap path and rejects retained
      heatmap plot authoring.

- [x] RBX-M3-304 Migrate `histogram2d_demo` to the declarative histogram2d plot panel.
  - Scope:
    - `apps/fret-examples/src/histogram2d_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
  - Goal:
    - Extend the reusable declarative plot panel with a histogram2d grid entry point backed by
      `Histogram2DPlotModel`.
    - Reuse the default declarative grid/colorbar path to paint finite histogram2d bins as quads
      and the default colorbar without constructing retained `Histogram2DPlotCanvas`.
    - Preserve the demo's deterministic point generation, `histogram2d_counts(...)` model setup,
      fixed axis labels, and native shell while teaching the declarative histogram2d path.
    - Add source-policy coverage so `histogram2d_demo` keeps teaching declarative histogram2d plot
      authoring instead of retained histogram2d canvas construction.
  - Validation:
    - `cargo nextest run -p fret-plot histogram2d_plot_panel_paints_grid_cells_and_default_colorbar_on_declarative_path`
    - `cargo nextest run -p fret-plot heatmap_plot_panel_paints`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-examples --lib`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" . -g '!target' -g '!repo-ref' || test $? -eq 1`
    - `rg -n "Histogram2DPlotCanvas|HeatmapPlotCanvas|fret_plot::retained|use fret_plot::retained|create_node_retained\(" apps/fret-examples/src/histogram2d_demo.rs apps/fret-examples/src/heatmap_demo.rs || test $? -eq 1`
  - Evidence:
    - `apps/fret-examples/src/histogram2d_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `ecosystem/fret-plot/src/declarative.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-304-declarative-histogram2d-plot-panel-migration`
  - Result:
    - `Histogram2DPlotPanelProps`, `histogram2d_plot_panel(...)`, and
      `histogram2d_plot_panel_in(...)` now exist on the default declarative path.
    - The shared declarative plot panel maps `Histogram2DPlotModel` onto the same grid metadata
      path as heatmap plots, painting visible finite bins as declarative quads and a default
      colorbar with min/max labels.
    - `histogram2d_demo` now renders through `RenderRootContext` plus
      `histogram2d_plot_panel_in(...)` instead of retained `Histogram2DPlotCanvas`.
    - The example source-policy test now teaches the declarative histogram2d path and rejects
      retained histogram2d plot authoring.

- [x] RBX-M3-305 Remove `apps/fret-examples`' `fret-plot/compat-retained-canvas` dependency.
  - Scope:
    - `apps/fret-examples/Cargo.toml`
    - `apps/fret-examples/src/docking_demo.rs`
    - `apps/fret-examples/src/container_queries_docking_demo.rs`
    - `apps/fret-examples/src/docking_arbitration_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
  - Goal:
    - Drop the first-party example crate's direct dependency on retained plot compatibility and
      keep the docking diagnostics demos on declarative-managed-surface anchors instead of
      retained widgets.
    - Preserve source-policy coverage that teaches first-party demos to stay on the default
      declarative `fret-plot` path.
  - Validation:
    - `cargo check -p fret-examples --lib`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface fret_examples_does_not_enable_fret_plot_retained_compat_feature`
    - `cargo nextest run -p fret-examples --test basic_plot_demos_surface`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "retained_bridge|create_node_retained|UiTreeRetainedExt|Widget|LayoutCx|PaintCx|SemanticsCx|compat-retained-canvas" apps/fret-examples/src/docking_demo.rs apps/fret-examples/src/container_queries_docking_demo.rs apps/fret-examples/src/docking_arbitration_demo.rs apps/fret-examples/Cargo.toml apps/fret-examples/tests/basic_plot_demos_surface.rs || true`
  - Evidence:
    - `apps/fret-examples/Cargo.toml`
    - `apps/fret-examples/src/docking_demo.rs`
    - `apps/fret-examples/src/container_queries_docking_demo.rs`
    - `apps/fret-examples/src/docking_arbitration_demo.rs`
    - `apps/fret-examples/tests/basic_plot_demos_surface.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-305-remove-app-dependency-on-fret-plot-compat-retained-canvas`
  - Result:
    - `apps/fret-examples/Cargo.toml` now depends on `fret-plot` without
      `compat-retained-canvas`.
    - The docking demos now use declarative-managed-surface diagnostics anchors rather than
      retained widgets or retained bridge imports.
    - `basic_plot_demos_surface` keeps proving that the first-party examples do not enable the
      retained plot compatibility feature.

- [x] RBX-M3-306 Move chart visual-map overlay paint/interactions onto the default declarative
      chart panel.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/visual_map_overlay.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/mod.rs`
  - Goal:
    - Add a declarative visual-map overlay tool that paints visual-map tracks, ramps, continuous
      selection ranges, handles, and piecewise bucket state without constructing retained
      `ChartCanvas`.
    - Route continuous visual-map drag and piecewise click toggles through `CanvasToolRouter` while
      reusing shared `visual_map_logic` and `slider_logic` policy.
    - Keep the explicit retained chart compatibility oracle compiling while the default chart path
      grows parity coverage.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/mod.rs ecosystem/fret-chart/src/declarative/panel.rs ecosystem/fret-chart/src/declarative/visual_map_overlay.rs`
    - `cargo check -p fret-chart --tests`
    - `cargo nextest run -p fret-chart --lib`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/declarative/mod.rs ecosystem/fret-chart/src/declarative/panel.rs ecosystem/fret-chart/src/declarative/visual_map_overlay.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/visual_map_overlay.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-306-declarative-chart-visual-map-overlay`
  - Result:
    - The default declarative chart panel now installs a visual-map overlay tool beside the legend
      and tooltip overlay tools.
    - Continuous visual-map drag updates the shared `ChartEngine` visual-map range, keeps pointer
      capture/release behavior on the declarative path, and paints the updated range immediately
      from overlay-local state after event-driven redraws.
    - Piecewise visual-map clicks update the shared `ChartEngine` piece mask and repaint bucket
      selection state on the declarative path.
    - Default `fret-chart` tests now cover continuous visual-map paint/drag and piecewise visual-map
      mask toggling without constructing retained `ChartCanvas`.

- [x] RBX-M3-307 Move chart data-zoom slider overlay paint/interactions onto the default declarative
      chart panel.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/data_zoom_overlay.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/mod.rs`
  - Goal:
    - Add a declarative data-zoom overlay tool that derives x/y slider tracks, paints the selected
      window and handles, and routes drag start/move/up through shared `slider_logic` without
      constructing retained `ChartCanvas`.
    - Keep the explicit retained chart compatibility oracle compiling while the default chart path
      grows parity coverage.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/mod.rs ecosystem/fret-chart/src/declarative/panel.rs ecosystem/fret-chart/src/declarative/data_zoom_overlay.rs`
    - `cargo check -p fret-chart --tests`
    - `cargo nextest run -p fret-chart --lib`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/declarative/mod.rs ecosystem/fret-chart/src/declarative/panel.rs ecosystem/fret-chart/src/declarative/data_zoom_overlay.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/data_zoom_overlay.rs`
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `ecosystem/fret-chart/src/declarative/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-307-declarative-chart-data-zoom-overlay`
  - Result:
    - The default declarative chart panel now installs a data-zoom overlay tool beside the legend,
      tooltip, and visual-map tools.
    - `data_zoom_overlay.rs` derives x/y slider tracks from the active chart layout, including
      axis-band and visual-map offsets, and paints the selection window plus handles from
      overlay-local state.
    - The pointer handlers route x/y drag start, drag update, and release through shared
      `slider_logic`, update the shared `ChartEngine` zoom state, and refresh overlay-local state
      so event-driven redraws stay in sync.
    - The new declarative test covers x-track paint/drag/capture/release and proves the y-track is
      derived in the declarative layout.

- [x] RBX-M3-308 Remove the retained `LineChart::into_canvas` builder from the default `fret-plot`
      surface.
  - Scope:
    - `ecosystem/fret-plot/src/chart/line_chart.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`
  - Goal:
    - Keep `LineChart` as a model builder on the default surface and remove the retained canvas
      convenience method now that there are no real consumers.
    - Leave the explicit retained `PlotCanvas` oracle behind `compat-retained-canvas` for the
      migration checks that still need it.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/chart/line_chart.rs ecosystem/fret-plot/src/lib.rs`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot`
    - `python3 tools/check_layering.py`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/src/chart/line_chart.rs ecosystem/fret-plot/src/lib.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/chart/line_chart.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-308-remove-the-retained-linechartintocanvas-builder-from-the-default-fret-plot-surface`
  - Result:
    - `LineChart` is now model-only on the default `fret-plot` surface; the retained canvas
      convenience builder is gone.
    - A default-surface policy test now scans `chart/line_chart.rs` to keep `into_canvas(` and
      `LinePlotCanvas` out of the non-compat surface.
    - The retained plot oracle remains behind `compat-retained-canvas` for the retained canvas
      path that still needs compile coverage.

- [x] RBX-M3-309 Add a declarative `LineChart::into_element` convenience helper while keeping the
      retained canvas builder removed.
  - Scope:
    - `ecosystem/fret-plot/src/chart/line_chart.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/HANDOFF.md`
  - Goal:
    - Give `LineChart` a first-class declarative entry point that mounts the default
      `line_plot_panel(...)` surface directly from an `ElementContextAccess`.
    - Keep the retained canvas builder absent from the default surface.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/chart/line_chart.rs ecosystem/fret-plot/src/lib.rs`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot`
    - `python3 tools/check_layering.py`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/src/chart/line_chart.rs ecosystem/fret-plot/src/lib.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/chart/line_chart.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m3-309-add-a-declarative-linechartintoelement-convenience-helper-while-keeping-the-retained-canvas-builder-removed`
  - Result:
    - `LineChart::into_element(...)` now installs the model and mounts
      `line_plot_panel_in(...)` directly from an `ElementContextAccess`.
    - The retained `into_canvas` convenience builder remains absent.
    - The source-policy test now asserts both the declarative helper and the retained-builder
      removal.

- [x] RBX-M3-310 Stop re-exporting default plot model/state/style surfaces through
      `fret_plot::retained`.
  - Scope:
    - `ecosystem/fret-plot/src/retained/mod.rs`
    - `ecosystem/fret-plot/src/retained/layout.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Keep the retained plot module as an explicit compat oracle for retained canvas/layer types
      only.
    - Stop teaching stable model/state/style APIs through the retained namespace now that default
      authors should import them from `fret_plot::{models,state,style}`.
    - Preserve retained plot compat compilation by replacing internal reliance on the removed
      re-export with explicit default-surface imports.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/retained/mod.rs ecosystem/fret-plot/src/retained/layout.rs`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot retained_plot_module_does_not_reexport_default_model_state_or_style_surfaces retained_plot_surface_requires_explicit_compat_feature line_chart_builder_stays_model_only_on_default_surface`
    - `cargo nextest run -p fret-plot`
    - `rg -n "pub use crate::(models|state|style)::\\*;" ecosystem/fret-plot/src/retained/mod.rs || test $? -eq 1`
    - `rg -n "use super::YAxis" ecosystem/fret-plot/src/retained --glob '*.rs' || test $? -eq 1`
    - `rg -n "fret_plot::retained::(LinePlotModel|PlotState|LinePlotStyle|PlotOutput|SeriesTooltipMode|AxisScale|LineSeries)" apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/retained/mod.rs ecosystem/fret-plot/src/retained/layout.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/retained/mod.rs`
    - `ecosystem/fret-plot/src/retained/layout.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-310-stop-re-exporting-default-plot-modelstatestyle-surfaces-through-fret_plotretained`
  - Result:
    - `fret_plot::retained` no longer re-exports `crate::models::*`, `crate::state::*`, or
      `crate::style::*`.
    - The retained layout helper now imports `YAxis` from `crate::models::YAxis` explicitly.
    - A source-policy test keeps default plot model/state/style surfaces out of the retained
      namespace while preserving retained canvas/layer oracle exports.

- [x] RBX-M3-311 Stop re-exporting raw retained plot layer authoring surfaces through
      `fret_plot::retained`.
  - Scope:
    - `ecosystem/fret-plot/src/retained/mod.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Keep `fret_plot::retained` as an explicit retained canvas oracle namespace rather than a
      glob-exported retained layer authoring surface.
    - Preserve the concrete retained `*PlotCanvas` aliases and `AxisConstraints` needed by
      compatibility oracle checks.
    - Stop exposing raw retained `PlotCanvas`, `PlotLayer`, concrete `*PlotLayer` types, and retained
      paint/hit/readout helper structs from the retained module root.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/retained/mod.rs ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `cargo nextest run -p fret-plot retained_plot_module_exports_only_explicit_canvas_oracles retained_plot_module_does_not_reexport_default_model_state_or_style_surfaces`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot --features compat-retained-canvas retained_plot_module_exports_only_explicit_canvas_oracles retained_plot_module_does_not_reexport_default_model_state_or_style_surfaces box_select_modifiers_expand_to_edges`
    - `cargo nextest run -p fret-plot`
    - `cargo nextest run -p fret-plot --features compat-retained-canvas`
    - `rg -n "pub use (canvas|layers)::\*|pub use crate::(models|state|style)::\*|pub use canvas::PlotCanvas|pub use layers::PlotLayer|LinePlotLayer|PlotPaintArgs|PlotHitTestArgs|PlotHover|SeriesMeta" ecosystem/fret-plot/src/retained/mod.rs || test $? -eq 1`
    - `rg -n "crate::retained::(LinePlotLayer|PlotCanvas|PlotLayer|PlotPaintArgs|PlotHitTestArgs|PlotHover|SeriesMeta)|use fret_plot::retained::(LinePlotLayer|PlotCanvas|PlotLayer|PlotPaintArgs|PlotHitTestArgs|PlotHover|SeriesMeta)" apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/retained/mod.rs ecosystem/fret-plot/src/retained/canvas/mod.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/retained/mod.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `ecosystem/fret-plot/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-311-stop-re-exporting-raw-retained-plot-layer-authoring-surfaces-through-fret_plotretained`
  - Result:
    - `fret_plot::retained` now explicitly re-exports only `AxisConstraints` plus concrete retained
      `*PlotCanvas` aliases.
    - The retained module root no longer uses `pub use canvas::*` / `pub use layers::*`.
    - Internal retained canvas tests use a sibling-private layer path instead of relying on the
      public retained module root to expose `LinePlotLayer`.
    - A source-policy test prevents the retained module root from regrowing raw retained layer
      authoring exports.

- [x] RBX-M3-315 Keep `fret_plot::retained` crate-private while retaining the compat oracle.
  - Scope:
    - `ecosystem/fret-plot/src/lib.rs`
    - `ecosystem/fret-plot/src/retained/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Keep the retained plot oracle source available for explicit compat checks without exposing
      `fret_plot::retained` as a public crate-root module.
    - Remove the remaining public retained plot root re-export lines now that no first-party plot
      demo consumers rely on retained authoring.
    - Preserve the explicit `compat-retained-canvas` feature mapping while keeping the default plot
      dependency bridge-free.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/retained/mod.rs`
    - `cargo test -p fret-plot retained_plot_surface_requires_explicit_compat_feature -- --nocapture`
    - `cargo test -p fret-plot retained_plot_module_stays_private_compat_oracle_only -- --nocapture`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot`
    - `cargo nextest run -p fret-plot --features compat-retained-canvas`
    - source scan proving no forbidden retained plot public-root markers in `lib.rs` public surface
      or `retained/mod.rs`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/src/lib.rs ecosystem/fret-plot/src/retained/mod.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/lib.rs`
    - `ecosystem/fret-plot/src/retained/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-315-keep-fret_plotretained-crate-private-while-retaining-the-compat-oracle`
  - Result:
    - `fret_plot::retained` now stays crate-private behind `#[cfg(feature = "compat-retained-canvas")] mod retained;`.
    - `retained/mod.rs` keeps only private oracle submodules and no longer re-exports
      `AxisConstraints` or concrete retained `*PlotCanvas` aliases publicly.
    - Default and explicit compat `fret-plot` package tests remain green.

- [x] RBX-M3-316 Remove `fret-plot` retained bridge feature mapping.
  - Scope:
    - `ecosystem/fret-plot/Cargo.toml`
    - `ecosystem/fret-plot/src/lib.rs`
    - `tools/check_layering.py`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Turn `fret-plot/compat-retained-canvas` into a no-op transition alias now that no
      first-party plot consumers rely on the retained plot module.
    - Stop compiling the retained plot oracle source from the package feature and keep the source
      quarantined/uncompiled until a deletion slice removes it.
    - Remove `fret-plot/compat-retained-canvas` from the retained-bridge feature-mapping allowlist,
      leaving only the remaining chart/node compat islands mapped to
      `fret-ui/unstable-retained-bridge`.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/lib.rs`
    - `cargo fmt --manifest-path ecosystem/fret-plot/Cargo.toml --check`
    - `cargo test -p fret-plot retained_plot_compat_feature_no_longer_enables_bridge_or_module -- --nocapture`
    - `cargo check -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot`
    - `cargo nextest run -p fret-plot --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/Cargo.toml ecosystem/fret-plot/src/lib.rs tools/check_layering.py docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/Cargo.toml`
    - `ecosystem/fret-plot/src/lib.rs`
    - `tools/check_layering.py`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-316-remove-fret-plot-retained-bridge-feature-mapping`
  - Result:
    - `compat-retained-canvas = []` is now a no-op transition alias in `fret-plot`.
    - `fret-plot` no longer maps any package feature to `fret-ui/unstable-retained-bridge`.
    - `fret-plot` no longer compiles `src/retained` from the crate root; retained plot source
      remains quarantined/uncompiled for a later deletion slice.
    - At `RBX-M3-316` time, the retained-bridge feature-mapping allowlist contained only
      `fret-node/compat-retained-canvas` and `fret-chart/compat-retained-canvas`; `RBX-M4-032`
      later removed the `fret-chart` mapping, so the current allowlist contains only
      `fret-node/compat-retained-canvas`.

- [x] RBX-M3-312 Stop glob-re-exporting retained chart canvas internals through
      `fret_chart::retained`.
  - Scope:
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Keep retained chart compatibility as an explicit `ChartCanvas` oracle namespace rather than a
      module-root glob export of all retained canvas public items.
    - Preserve the currently public retained chart support types that are part of the retained
      `ChartCanvas` API (`ChartStyleSource`, `ChartTextCachePruneTuning`).
    - Prevent legacy/internal retained chart surface from regrowing through the retained module
      root.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/lib.rs ecosystem/fret-chart/src/retained/mod.rs`
    - `cargo nextest run -p fret-chart retained_chart_module_exports_only_explicit_canvas_oracles retained_widgets_are_not_glob_reexported_from_crate_root`
    - `cargo check -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `rg -n "pub use canvas::\*|pub use text_cache::\*|ChartCanvasMode|ChartLayout|AxisRegion|AxisBandLayout" ecosystem/fret-chart/src/retained/mod.rs || test $? -eq 1`
    - `rg -n "fret_chart::retained::(ChartLayout|AxisRegion|AxisBandLayout|ChartCanvasMode)|use fret_chart::retained::(ChartLayout|AxisRegion|AxisBandLayout|ChartCanvasMode)" apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/lib.rs ecosystem/fret-chart/src/retained/mod.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-312-stop-glob-re-exporting-retained-chart-canvas-internals-through-fret_chartretained`
  - Result:
    - `fret_chart::retained` now explicitly re-exports only `ChartCanvas`, `ChartStyleSource`, and
      `ChartTextCachePruneTuning`.
    - The retained chart module root no longer uses `pub use canvas::*`.
    - A source-policy test requires exactly one explicit retained chart root re-export line and
      blocks legacy/internal chart root names such as `ChartCanvasMode`, `ChartLayout`,
      `AxisRegion`, and `AxisBandLayout`.

- [x] RBX-M3-313 Remove no-user retained chart support knobs from public retained API.
  - Scope:
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Shrink the explicit `fret_chart::retained` oracle namespace to `ChartCanvas` only.
    - Remove no-user public support knobs from retained `ChartCanvas` (`ChartStyleSource`,
      `ChartTextCachePruneTuning`, `set_style_source(...)`, `set_text_cache_prune_tuning(...)`).
    - Keep retained chart compatibility tests green while reducing retained public API area.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/lib.rs ecosystem/fret-chart/src/retained/mod.rs ecosystem/fret-chart/src/retained/canvas.rs`
    - `cargo nextest run -p fret-chart retained_chart_module_exports_only_explicit_canvas_oracles`
    - `cargo check -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `rg -n "pub struct ChartTextCachePruneTuning|pub enum ChartStyleSource|pub fn set_text_cache_prune_tuning|pub fn set_style_source" ecosystem/fret-chart/src/retained/canvas.rs || test $? -eq 1`
    - `rg -n "fret_chart::retained::(ChartStyleSource|ChartTextCachePruneTuning)|use fret_chart::retained::(ChartStyleSource|ChartTextCachePruneTuning)|set_style_source\(|set_text_cache_prune_tuning\(" apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/lib.rs ecosystem/fret-chart/src/retained/mod.rs ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `ecosystem/fret-chart/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-313-remove-no-user-retained-chart-support-knobs-from-public-retained-api`
  - Result:
    - `fret_chart::retained` now explicitly re-exports only `ChartCanvas`.
    - `ChartStyleSource` and `ChartTextCachePruneTuning` are retained-canvas-private implementation
      details.
    - `set_style_source(...)` and `set_text_cache_prune_tuning(...)` were deleted from the public
      retained `ChartCanvas` API.
    - The retained chart surface-policy test now locks the root export and no-user support knob
      removal.

- [x] RBX-M3-314 Keep `fret_chart::retained` crate-private while retaining the compat oracle.
  - Scope:
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Keep the retained chart oracle module available for explicit compat checks without exposing
      `fret_chart::retained` as a public crate-root module.
    - Preserve the explicit `compat-retained-canvas` feature mapping while keeping the default
      chart dependency bridge-free.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/lib.rs ecosystem/fret-chart/src/retained/mod.rs ecosystem/fret-chart/src/retained/canvas.rs`
    - `cargo test -p fret-chart retained_chart_module_stays_private_compat_oracle_only -- --nocapture`
    - `cargo test -p fret-chart default_chart_dependency_does_not_enable_unstable_retained_bridge -- --nocapture`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/lib.rs ecosystem/fret-chart/src/retained/mod.rs ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/lib.rs`
    - `ecosystem/fret-chart/src/retained/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-314-keep-fret_chartretained-crate-private-while-retaining-the-compat-oracle`
  - Result:
    - `fret_chart::retained` now stays crate-private behind `#[cfg(feature = "compat-retained-canvas")] mod retained;`.
    - The retained chart oracle still exists for explicit compat checks, but public crate-root access
      is gone.
    - The default chart dependency still does not enable `fret-ui/unstable-retained-bridge`.

- [x] RBX-M3-317 Move chart series-order palette oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `series_color_follows_series_order_not_series_id` behavior on the
      default declarative chart panel before removing more retained chart compatibility surface.
    - Use the chart model's declared `series_order` / grid-filtered series order for palette slot
      assignment instead of deriving palette slots from raw `SeriesId`.
    - Keep line-family declarative paint compatible with retained chart behavior: line marks with
      `source_series` use the series palette slot even when `delinea` emits a default
      `PaintId(0)` stroke payload.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_uses_series_order_for_palette_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_uses_series_order_for_palette_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/declarative/panel.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-317-move-chart-series-order-palette-oracle-to-the-declarative-path`
  - Result:
    - Added `compute_series_rank_by_id(...)` for the declarative chart panel and routed line,
      rect, point, tooltip, and a11y palette-rank consumers through the same grid-aware series rank.
    - Changed declarative polyline paint to prefer `source_series` palette color over mark-local
      stroke `PaintId`, matching retained chart behavior for line-family marks.
    - Added
      `chart_canvas_panel_uses_series_order_for_palette_on_declarative_path`, which renders the
      real declarative panel, maps emitted `SceneOp::Path` entries back to `delinea` polyline marks,
      and proves series id `42` receives palette slot 0 while later-declared series id `1` receives
      palette slot 1.
    - Default and explicit compat `fret-chart` package tests remain green; this does not yet remove
      the `fret-chart/compat-retained-canvas` retained bridge mapping.

- [x] RBX-M3-318 Move first-chart bar pointer-hover tooltip output oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `first_chart_bar_hover_publishes_tooltip_lines_to_output_model` and
      `first_chart_bar_hover_publishes_tooltip_lines_with_nonzero_bounds_origin` behavior on the
      default declarative chart panel before removing more retained chart compatibility surface.
    - Prove a real declarative `PointerEvent::Move` over the first Desktop bar publishes shared
      `ChartCanvasOutput` tooltip lines without constructing retained `ChartCanvas`.
    - Keep the non-zero canvas-origin regression explicit because this was a retained oracle for
      hover-point/view-origin correctness.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_pointer_hover_publishes_tooltip_lines_to_output_model_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_pointer_hover_publishes_tooltip_lines_to_output_model_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/declarative/panel.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-318-move-first-chart-bar-pointer-hover-tooltip-output-oracle-to-the-declarative-path`
  - Result:
    - Added a declarative first-chart bar fixture matching the retained Desktop/Mobile bar oracle.
    - Added
      `chart_canvas_panel_pointer_hover_publishes_tooltip_lines_to_output_model_on_declarative_path`,
      which renders `chart_canvas_panel(...)` at a non-zero origin, dispatches a real mouse move to
      the first Desktop bar hover point, propagates model changes, and proves the shared output
      model advances with non-empty tooltip lines headed by `TooltipTextLineKind::AxisHeader`.
    - This is a proof slice: the default declarative hover/output path already carried the behavior,
      so no retained chart source was deleted and `fret-chart/compat-retained-canvas` remains mapped
      to `fret-ui/unstable-retained-bridge` for other oracle families.

- [x] RBX-M3-319 Move active-axis last-hovered-band oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `active_axes_prefer_last_hovered_band` behavior on the default
      declarative chart panel before deleting more retained chart compatibility surface.
    - After hovering a right-side y-axis band, plot-region pan and wheel interactions should keep
      using the last hovered right y-axis instead of falling back to the first visible series' left
      y-axis.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_plot_pan_prefers_last_hovered_axis_band_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_plot_pan_prefers_last_hovered_axis_band_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-319-move-active-axis-last-hovered-band-oracle-to-the-declarative-path`
  - Result:
    - Added a declarative multi-axis layout fixture with explicit bottom/left/right axis positions.
    - Added `chart_canvas_panel_plot_pan_prefers_last_hovered_axis_band_on_declarative_path`, which
      hovers the right y-axis band, dispatches a real plot-region pan through the declarative panel,
      and proves the shared engine updates the right-side y window without creating a left-side
      y-window after the hover.
    - The default declarative chart path now carries the retained active-axis oracle, but
      `fret-chart/compat-retained-canvas` remains mapped to `fret-ui/unstable-retained-bridge` for
      the remaining chart oracle families.

- [x] RBX-M3-320 Move axis-pointer axis-band clamp oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `axis_pointer_hover_point_clamps_axis_band_into_plot` behavior on the
      default declarative chart panel before deleting more retained chart compatibility surface.
    - Prove bottom x-axis, left y-axis, and right y-axis band hover positions are projected back
      inside the plot rect with retained-compatible 1px in-plot clamps.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_axis_pointer_hover_point_clamps_axis_band_into_plot_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_axis_pointer_hover_point_clamps_axis_band_into_plot_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-320-move-axis-pointer-axis-band-clamp-oracle-to-the-declarative-path`
  - Result:
    - Added
      `chart_canvas_panel_axis_pointer_hover_point_clamps_axis_band_into_plot_on_declarative_path`,
      which exercises the declarative `axis_pointer_hover_point_for_layout(...)` helper with a
      bottom x-axis band plus left/right y-axis bands and proves the returned hover point remains
      inside the plot rect at `(50, 99)`, `(1, 25)`, and `(99, 75)` respectively.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      helper already carries the retained clamp semantics used by the event path.

- [x] RBX-M3-321 Move primary-axes hidden-series oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `primary_axes_skip_hidden_series` behavior on the default declarative
      chart panel before deleting more retained chart compatibility surface.
    - Prove plot-region pan uses the first visible series' axes when the first declared series is
      hidden, instead of continuing to route through the hidden series' y-axis.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_plot_pan_primary_axes_skip_hidden_series_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_plot_pan_primary_axes_skip_hidden_series_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-321-move-primary-axes-hidden-series-oracle-to-the-declarative-path`
  - Result:
    - Added
      `chart_canvas_panel_plot_pan_primary_axes_skip_hidden_series_on_declarative_path`, which
      hides the first declared multi-axis series, renders the real declarative chart panel, performs
      a plot-region pan, and proves the pan still updates the shared x-axis while skipping the
      hidden left y-axis and updating the first visible right y-axis.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      event path already carries the retained primary-axis visible-series semantics.

- [x] RBX-M3-322 Move legend double-click isolate/restore oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `legend_double_click_isolates_and_restores_all_series` behavior on
      the default declarative chart panel before deleting more retained chart compatibility surface.
    - Prove double-clicking a legend row isolates that series and double-clicking the already
      isolated row restores all series through the real declarative pointer event path.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_legend_double_click_isolates_and_restores_all_series_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_legend_double_click_isolates_and_restores_all_series_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-322-move-legend-double-click-isolaterestore-oracle-to-the-declarative-path`
  - Result:
    - Added
      `chart_canvas_panel_legend_double_click_isolates_and_restores_all_series_on_declarative_path`,
      which renders a two-series declarative chart panel, dispatches a real double-click pointer
      down on the second legend row, proves the first series is hidden while the second remains
      visible, then double-clicks the isolated row again and proves both series are visible.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      legend chrome path already carries the retained double-click isolate/restore semantics.

- [x] RBX-M3-323 Move legend selector All/None/Invert oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `legend_select_all_none_invert_update_series_visibility` behavior on
      the default declarative chart panel before deleting more retained chart compatibility surface.
    - Prove the declarative legend selector row updates series visibility for None, All, and Invert
      through the real pointer event path.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_legend_selectors_update_series_visibility_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_legend_selectors_update_series_visibility_on_declarative_path`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-323-move-legend-selector-allnoneinvert-oracle-to-the-declarative-path`
  - Result:
    - Added
      `chart_canvas_panel_legend_selectors_update_series_visibility_on_declarative_path`, which
      renders a two-series declarative chart panel, dispatches real pointer-down events on the
      legend selector row, and proves None hides all series, All shows all series, and Invert flips
      current series visibility.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      legend selector chrome path already carries the retained All/None/Invert semantics.

- [x] RBX-M3-324 Move legend scroll clamp oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `legend_scroll_clamps_to_content_height` behavior on the default
      declarative chart panel before deleting more retained chart compatibility surface.
    - Prove real declarative legend wheel events clamp scrolling to the legend content height,
      expose bottom rows at the maximum scroll offset, and clamp back to the top.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_legend_scroll_clamps_to_content_height_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_legend_scroll_clamps_to_content_height_on_declarative_path`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-324-move-legend-scroll-clamp-oracle-to-the-declarative-path`
  - Result:
    - Added `chart_canvas_panel_legend_scroll_clamps_to_content_height_on_declarative_path`, which
      renders a forty-series declarative chart panel, dispatches real legend wheel events, verifies
      the retained scroll-policy maximum of `422px` for the test fixture, and proves intermediate,
      bottom-clamped, and top-clamped scroll positions by clicking the exposed rows for series 13,
      31, and 1 respectively.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      legend wheel chrome path already carries the retained content-height clamp semantics.

- [x] RBX-M3-325 Move visual-map track style padding oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `visual_map_track_applies_style_padding` behavior on the default
      declarative chart panel before deleting more retained chart compatibility surface.
    - Prove the declarative visual-map track helper applies style padding to the track rect on the
      default declarative chart panel.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_visual_map_track_applies_style_padding_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_visual_map_track_applies_style_padding_on_declarative_path`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-325-move-visual-map-track-style-padding-oracle-to-the-declarative-path`
  - Result:
    - Added `chart_canvas_panel_visual_map_track_applies_style_padding_on_declarative_path`, which
      renders a visual-map-enabled declarative chart panel with explicit track padding and proves
      the helper track rect is inset by the requested padding on both x and y axes.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      visual-map helper already carries the retained style-padding semantics.

- [x] RBX-M3-326 Move data-zoom slider clamp/no-invert oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `slider_window_after_delta_clamps_and_never_inverts` behavior on the
      default declarative chart panel before deleting more retained chart compatibility surface.
    - Prove real declarative data-zoom slider drags clamp pan windows to the axis extent and keep
      min/max handle drags non-inverted when dragged past the opposite handle.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_data_zoom_slider_clamps_and_never_inverts_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_data_zoom_slider_clamps_and_never_inverts_on_declarative_path`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-326-move-data-zoom-slider-clampno-invert-oracle-to-the-declarative-path`
  - Result:
    - Added `chart_canvas_panel_data_zoom_slider_clamps_and_never_inverts_on_declarative_path`,
      which mounts the real declarative chart panel, dispatches data-zoom pointer drags through the
      actual x slider, proves pan drags clamp to `0.0..0.1` and `0.9..1.0`, then drags both handles
      past their opposite edge and proves the resulting windows remain bounded and non-inverted.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      data-zoom slider path carries the retained clamp/no-invert semantics.

- [x] RBX-M3-327 Move visual-map domain-endpoint mapping oracle to the declarative path.
  - Scope:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Preserve retained chart `visual_map_y_mapping_respects_domain_endpoints` behavior on the
      default declarative chart panel before deleting more retained chart compatibility surface.
    - Prove the declarative visual-map track maps the domain minimum to the track bottom and the
      domain maximum to the track top, and that a full-domain continuous selection fills the track.
  - Validation:
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/declarative/panel.rs`
    - `cargo test -p fret-chart chart_canvas_panel_visual_map_y_mapping_respects_domain_endpoints_on_declarative_path -- --nocapture`
    - `cargo nextest run -p fret-chart chart_canvas_panel_visual_map_y_mapping_respects_domain_endpoints_on_declarative_path`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `python3 - <<'PY' ... PY` conflict-marker scan over the touched chart panel and workstream docs
  - Evidence:
    - `ecosystem/fret-chart/src/declarative/panel.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-327-move-visual-map-domain-endpoint-mapping-oracle-to-the-declarative-path`
  - Result:
    - Added
      `chart_canvas_panel_visual_map_y_mapping_respects_domain_endpoints_on_declarative_path`,
      which mounts a full-domain continuous visual-map declarative panel, verifies the helper maps
      `domain.min` to the track bottom and `domain.max` to the track top, and proves the painted
      selection quad fills the declarative track.
    - No retained chart source was modified; this is an oracle migration proving the declarative
      visual-map paint/helper path carries the retained domain-endpoint mapping semantics.

- [x] RBX-M3-328 Extract plot axis-lock helpers into the shared `plot/view` module.
  - Scope:
    - `ecosystem/fret-plot/src/plot/view.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Move the pure axis-lock helper logic out of the retained plot canvas and into the shared
      plot view module so the retained adapter only carries the retained-specific fit-view glue.
    - Keep `fit_view_bounds_with_zoom_locks(...)` in the retained adapter, but make it delegate to
      the shared helper boundary.
  - Validation:
    - `cargo test -p fret-plot apply_axis_locks_preserves_only_locked_axes -- --nocapture`
    - `rustfmt --edition 2024 --check ecosystem/fret-plot/src/plot/view.rs ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `cargo nextest run -p fret-plot`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo nextest run -p fret-plot --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-plot/src/plot/view.rs ecosystem/fret-plot/src/retained/canvas/mod.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-plot/src/plot/view.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m3-328-extract-plot-axis-lock-helpers-into-shared-view-module`
  - Result:
    - Added shared `plot::view::{apply_axis_locks, all_visible_axes_zoom_locked}` plus direct unit
      coverage.
    - Retained `fit_view_bounds_with_zoom_locks(...)` now delegates to the shared helper instead of
      holding the pure axis-lock logic inline.
    - The default and compat `fret-plot` gates remain green after the helper extraction.

- [x] RBX-M4-020 Delete the retained legend scroll oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `legend_scroll_clamps_to_content_height` oracle now that the
      declarative chart panel covers the same legend scroll clamp behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-020-delete-the-retained-legend-scroll-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `legend_scroll_clamps_to_content_height` test from the compat oracle
      suite.
    - The declarative chart panel still proves the same legend scroll clamp behavior, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-021 Delete the retained visual-map domain-endpoint oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `visual_map_y_mapping_respects_domain_endpoints` oracle now
      that the declarative chart panel covers the same visual-map domain-endpoint behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-021-delete-the-retained-visual-map-domain-endpoint-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `visual_map_y_mapping_respects_domain_endpoints` test from the compat
      oracle suite.
    - The declarative chart panel still proves the same visual-map domain-endpoint behavior via
      `chart_canvas_panel_visual_map_y_mapping_respects_domain_endpoints_on_declarative_path`, so
      the retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-022 Delete the retained axis-pointer hover oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `axis_pointer_hover_point_clamps_axis_band_into_plot` oracle
      now that the declarative chart panel covers the same axis-pointer hover clamping behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-022-delete-the-retained-axis-pointer-hover-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `axis_pointer_hover_point_clamps_axis_band_into_plot` test from the
      compat oracle suite.
    - The declarative chart panel still proves the same axis-pointer hover clamping behavior via
      `chart_canvas_panel_axis_pointer_hover_point_clamps_axis_band_into_plot_on_declarative_path`,
      so the retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-023 Delete the retained legend double-click oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `legend_double_click_isolates_and_restores_all_series`
      oracle now that the declarative chart panel covers the same legend double-click behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-023-delete-the-retained-legend-double-click-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `legend_double_click_isolates_and_restores_all_series` test from the
      compat oracle suite.
    - The declarative chart panel still proves the same legend double-click behavior via
      `chart_canvas_panel_legend_double_click_isolates_and_restores_all_series_on_declarative_path`,
      so the retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-024 Delete the retained legend select-all/none/invert oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `legend_select_all_none_invert_update_series_visibility`
      oracle now that the declarative chart panel covers the same legend visibility behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-024-delete-the-retained-legend-select-allnoneinvert-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `legend_select_all_none_invert_update_series_visibility` test from the
      compat oracle suite.
    - The declarative chart panel still proves the same legend visibility behavior via
      `chart_canvas_panel_legend_selectors_update_series_visibility_on_declarative_path`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-025 Delete the retained legend selector hit-test oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `legend_selector_hit_test_returns_action` oracle now that the
      declarative chart panel covers the same selector hit-testing behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-025-delete-the-retained-legend-selector-hit-test-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `legend_selector_hit_test_returns_action` test from the compat oracle
      suite.
    - The declarative chart panel still proves the same selector hit-testing behavior via
      `chart_canvas_panel_legend_selectors_update_series_visibility_on_declarative_path`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-026 Delete the retained explicit Y-domain output propagation oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `explicit_y_domain_window_propagates_to_second_linked_chart_output_model`
      oracle now that the declarative chart panel covers the same explicit Y-domain output
      propagation behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-026-delete-the-retained-explicit-y-domain-output-propagation-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `explicit_y_domain_window_propagates_to_second_linked_chart_output_model`
      test from the compat oracle suite.
    - The declarative chart panel still proves the same explicit Y-domain output propagation via
      `explicit_y_domain_window_propagates_to_second_declarative_chart_output_model`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-027 Delete the retained visual-map track padding oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `visual_map_track_applies_style_padding` oracle now that the
      declarative chart panel covers the same visual-map track padding behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-027-delete-the-retained-visual-map-track-padding-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `visual_map_track_applies_style_padding` test from the compat oracle
      suite.
    - The declarative chart panel still proves the same visual-map track padding behavior via
      `chart_canvas_panel_visual_map_track_applies_style_padding_on_declarative_path`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-028 Delete the retained primary-axes hidden-series oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `primary_axes_skip_hidden_series` oracle now that the
      declarative chart panel covers the same primary-axis selection behavior when hidden series
      are skipped.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-028-delete-the-retained-primary-axes-hidden-series-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `primary_axes_skip_hidden_series` test from the compat oracle suite.
    - The declarative chart panel still proves the same hidden-series primary-axis behavior via
      `chart_canvas_panel_plot_pan_primary_axes_skip_hidden_series_on_declarative_path`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-029 Delete the retained active-axes hovered-band oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `active_axes_prefer_last_hovered_band` oracle now that the
      declarative chart panel covers the same active-axis selection behavior when hovering the
      latest band.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-029-delete-the-retained-active-axes-hovered-band-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `active_axes_prefer_last_hovered_band` test from the compat oracle
      suite.
    - The declarative chart panel still proves the same active-axis hovered-band behavior via
      `chart_canvas_panel_plot_pan_prefers_last_hovered_axis_band_on_declarative_path`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-030 Delete the retained first-chart tooltip-output oracles now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained first-chart tooltip/output oracles now that the declarative
      chart panel covers the same pointer-hover and keyboard-navigation output behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-030-delete-the-retained-first-chart-tooltip-output-oracles-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `first_chart_bar_hover_publishes_tooltip_lines_to_output_model`,
      `first_chart_bar_hover_publishes_tooltip_lines_with_nonzero_bounds_origin`, and
      `ui_tree_keyboard_navigation_publishes_tooltip_lines_to_output_model` tests from the compat
      oracle suite.
    - The declarative chart panel still proves the same pointer-hover and keyboard-navigation
      tooltip/output behavior via
      `chart_canvas_panel_pointer_hover_publishes_tooltip_lines_to_output_model_on_declarative_path`
      and `chart_canvas_panel_keyboard_navigation_publishes_tooltip_lines_on_declarative_path`, so
      the retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-031 Delete the retained explicit-link-axis-map domain-window oracle now covered by declarative coverage.
  - Scope:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the redundant retained `explicit_link_axis_map_publishes_ambiguous_y_domain_window_to_output_model`
      oracle now that the declarative chart panel covers the same explicit Y-domain output
      propagation behavior.
    - Trim the retained `fret-chart` compat test surface without losing the behavioral proof.
  - Validation:
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-031-delete-the-retained-explicit-link-axis-map-domain-window-oracle-now-covered-by-declarative-coverage`
  - Result:
    - Removed the retained `explicit_link_axis_map_publishes_ambiguous_y_domain_window_to_output_model`
      test from the compat oracle suite.
    - The declarative chart panel still proves the same explicit Y-domain output propagation via
      `explicit_y_domain_window_propagates_to_second_declarative_chart_output_model`, so the
      retained deletion does not leave the behavior uncovered.
    - Default and compat `fret-chart` gates remain green.

- [x] RBX-M4-032 Remove the `fret-chart` retained-bridge feature mapping.
  - Scope:
    - `ecosystem/fret-chart/Cargo.toml`
    - `ecosystem/fret-chart/src/lib.rs`
    - `tools/check_layering.py`
    - `tools/test_check_layering.py`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Turn `fret-chart/compat-retained-canvas` into a no-op transition alias after the retained
      chart oracle surface was migrated/deleted.
    - Stop compiling `ecosystem/fret-chart/src/retained` through the crate root.
    - Remove `fret-chart` from the retained-bridge feature-mapping allowlist so only the remaining
      `fret-node/compat-retained-canvas` island may map to `fret-ui/unstable-retained-bridge`.
  - Validation:
    - `cargo test -p fret-chart retained_chart_compat_feature_is_noop_and_module_is_quarantined -- --nocapture`
      - failed before implementation, proving the policy test caught the still-compiled retained
        module.
    - `cargo test -p fret-chart public_surface_policy -- --nocapture`
    - `rustfmt --edition 2024 --check ecosystem/fret-chart/src/lib.rs`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-chart`
    - `cargo nextest run -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" ecosystem/fret-chart/Cargo.toml ecosystem/fret-chart/src/lib.rs tools/check_layering.py tools/test_check_layering.py docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `ecosystem/fret-chart/Cargo.toml`
    - `ecosystem/fret-chart/src/lib.rs`
    - `tools/check_layering.py`
    - `tools/test_check_layering.py`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-032-remove-the-fret-chart-retained-bridge-feature-mapping`
  - Result:
    - `fret-chart/compat-retained-canvas` is now `[]` and no longer enables
      `fret-ui/unstable-retained-bridge`.
    - `fret-chart` no longer compiles `src/retained` from the crate root; retained chart source is
      quarantined/uncompiled for a later source-deletion cleanup.
    - The retained-bridge feature-mapping allowlist now contains only
      `fret-node/compat-retained-canvas`.

- [x] Convert `fret-chart` retained surfaces to `Canvas`-first declarative authoring.
  - Result: the quarantined retained chart source tree was deleted after the declarative path and
    no-op compat gate proved there are no remaining first-party chart consumers for retained
    source files.
- [x] Convert `fret-plot` retained surfaces to `Canvas`-first declarative authoring.
  - Current proof: `apps/fret-examples/src/tags_demo.rs`,
    `apps/fret-examples/src/plot_image_demo.rs`, `apps/fret-examples/src/drag_demo.rs`,
    `apps/fret-examples/src/plot_stress_demo.rs`, `apps/fret-examples/src/inf_lines_demo.rs`,
    `apps/fret-examples/src/plot_demo.rs`, `apps/fret-examples/src/stems_demo.rs`,
    `apps/fret-examples/src/stairs_demo.rs`, `apps/fret-examples/src/shaded_demo.rs`,
    `apps/fret-examples/src/error_bars_demo.rs`, and
    `apps/fret-examples/src/histogram_demo.rs`,
    `apps/fret-examples/src/grouped_bars_demo.rs`,
    `apps/fret-examples/src/stacked_bars_demo.rs`,
    `apps/fret-examples/src/candlestick_demo.rs`,
    `apps/fret-examples/src/heatmap_demo.rs`, and
    `apps/fret-examples/src/histogram2d_demo.rs` now use `line_plot_panel_in(...)` /
    `stems_plot_panel_in(...)` / `shaded_plot_panel_in(...)` /
    `error_bars_plot_panel_in(...)` / `histogram_plot_panel_in(...)` /
    `bars_plot_panel_in(...)` / `candlestick_plot_panel_in(...)` /
    `heatmap_plot_panel_in(...)` / `histogram2d_plot_panel_in(...)` for
    TagX/TagY/PlotText overlays, a `PlotImage` underlay, draggable overlay output, the pure
    line-plot stress harness, caller-owned inf-line overlays with custom y2/y3/y4 axis label
    formatters, the LogX/multi-axis query-output demo, declarative stems/area examples, and
    shaded band fills/strokes plus X/Y error bars, caps, point markers, 1D histogram bin fills,
    grouped bars, stacked bars, heatmap grid cells plus the default colorbar, and histogram2d
    bins plus the default colorbar.
    `apps/fret-examples/src/bars_demo.rs` now uses the declarative `fret-chart` panel instead of
    retained `fret-plot` bar authoring, and `apps/fret-examples/src/area_demo.rs` plus both halves
    of `apps/fret-examples/src/linked_cursor_demo.rs` now use the declarative area/line panels.
    Primary and y2/y3/y4 y-axis label formatter support exists on the declarative panel,
    `YAxis::Right`/`YAxis::Right2`/`YAxis::Right3` line series now use their matching right-axis
    bounds, and right-axis `PlotImage`, `TagY`, and `PlotText` overlays now paint on the
    declarative path, right-axis draggable line/point labels now paint on the declarative path,
    right-axis draggable line/point/rect shapes now paint on the declarative path, and right-axis
    draggable Y-line, draggable X-line, and right-axis draggable point/rect start/update/end output
    now publish through the declarative event path. The quarantined retained plot source tree has
    now been deleted, and no known first-party retained plot demo consumers remain.
- [x] Remove `unstable-retained-bridge` from `ecosystem/fret-chart`; `ecosystem/fret-plot`
  is already a no-op transition alias via `RBX-M3-316` and no longer maps to the bridge.
  - Current proof: `RBX-M4-032` makes `fret-chart/compat-retained-canvas` a no-op transition alias,
    removes the crate-root retained module gate, and narrows the feature-mapping allowlist to
    `fret-node/compat-retained-canvas` only.

### M4 — Bridge shrink and delete (or quarantine)

- [x] RBX-M4-010 Audit retained bridge exports and tighten retained-bridge feature allowlist semantics.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `tools/check_layering.py`
    - `tools/test_check_layering.py`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Record the current `retained_bridge.rs` export usage map before deleting or quarantining the
      remaining bridge surface.
    - Split the retained-bridge allowlist into two enforceable concepts:
      - direct `fret-ui` dependency features: no crate may enable
        `unstable-retained-bridge` directly;
      - explicit compatibility feature mappings: at `RBX-M4-010` time,
        `fret-node/compat-retained-canvas`, `fret-plot/compat-retained-canvas`, and
        `fret-chart/compat-retained-canvas` were the only mapped features; `RBX-M3-316` later
        removed the `fret-plot` mapping, so the current mapped set is `fret-node` plus
        `fret-chart` only.
    - Add unit coverage for the layering checker so future crates cannot grow the retained bridge
      by adding a new package feature mapping, and allowed crates cannot smuggle the bridge through
      a default feature or direct dependency feature.
  - Validation:
    - `python3 tools/audit_crate.py --crate fret-ui`
    - retained bridge export scan over `apps/`, `crates/`, and `ecosystem/`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" tools/check_layering.py tools/test_check_layering.py docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `tools/check_layering.py`
    - `tools/test_check_layering.py`
    - `crates/fret-ui/src/retained_bridge.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-010-retained-bridge-export-audit-and-feature-allowlist-semantics`
  - Result:
    - The current direct dependency allowlist is empty: no workspace crate should enable
      `fret-ui/unstable-retained-bridge` directly from a `fret-ui` dependency entry.
    - The direct dependency allowlist is still empty.
    - This slice originally allowed the three active compatibility feature mappings:
      `fret-node/compat-retained-canvas`, `fret-plot/compat-retained-canvas`, and
      `fret-chart/compat-retained-canvas`; `RBX-M3-316` later removed the `fret-plot` mapping, so
      the current allowlist contains only `fret-node/compat-retained-canvas` and
      `fret-chart/compat-retained-canvas`. `RBX-M4-032` later removed the `fret-chart` mapping, so
      the current allowlist contains only `fret-node/compat-retained-canvas`.
    - Export audit summary:
      - Required by remaining compat islands / harnesses now: `Widget`, `EventCx`, `CommandCx`,
        `CommandAvailability`, `CommandAvailabilityCx`, `Invalidation`, `LayoutCx`, `PaintCx`,
        `PrepaintCx`, `SemanticsCx`, and `UiTreeRetainedExt`.
      - Required by the feature-gated retained-subtree declarative bridge shape:
        `RetainedSubtreeProps` and its `RetainedSubtreeFactory` field type, though current
        workspace runtime usage is mostly policy tests / cookbook source-policy strings rather than
        first-party runtime mounting.
      - No current workspace source users were found for retained-bridge exports
        `BoundTextInput`, `TextInput`, `MeasureCx`, `ViewportInputCapture`, or
        `viewport_surface::handle_viewport_surface_input`; these are the next deletion candidates
        once ADR/doc references are updated or intentionally retained as external-compat debt.
    - Layering now catches both accidental direct dependency feature reintroduction and new package
      feature mappings to `fret-ui/unstable-retained-bridge`.

- [x] RBX-M4-011 Delete no-user retained bridge exports and remove the retained viewport helper.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `docs/adr/0098-plot-architecture-and-performance.md`
    - `docs/adr/0052-ui-host-runtime-boundary.md`
    - `docs/adr/0077-resizable-panel-groups-and-docking-split-sizing.md`
    - `docs/adr/0096-plot-widgets-and-crate-placement.md`
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
    - `docs/crate-usage-guide.md`
    - `docs/roadmap.md`
    - `docs/shadcn-declarative-progress.md`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the retained bridge exports that the audit found to have no current workspace users:
      `BoundTextInput`, `TextInput`, `MeasureCx`, `ViewportInputCapture`, and
      `handle_viewport_surface_input`.
    - Keep the retained subtree bridge and the `Widget`/context exports needed by the remaining
      compat islands.
    - Update ADR references that still pointed at the deleted retained viewport helper so the docs
      reflect the current bridge surface.
  - Validation:
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/retained_bridge.rs docs/adr docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `docs/adr/0098-plot-architecture-and-performance.md`
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-011-delete-no-user-retained-bridge-exports`
  - Result:
    - The retained bridge no longer re-exports `BoundTextInput`, `TextInput`, or `MeasureCx`.
    - The viewport retained helper module has been removed entirely, so there is no remaining
      `ViewportInputCapture` or `handle_viewport_surface_input` export in `fret_ui::retained_bridge`.
    - `fret-node`, `fret-plot`, `fret-chart`, and `fret-ui` compatibility checks still compile
      after the shrink.
    - ADR references now describe viewport forwarding through declarative / core mapping paths
      instead of the deleted retained helper.

- [x] RBX-M4-012 Narrow retained subtree props to constructor/accessor-only public API.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `crates/fret-ui/src/declarative/frame.rs`
    - `crates/fret-ui/src/declarative/host_widget/layout.rs`
    - `crates/fret-ui/src/declarative/host_widget/measure.rs`
    - `crates/fret-ui/src/declarative/host_widget/paint.rs`
    - `crates/fret-ui/src/declarative/mount.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Stop exposing `RetainedSubtreeProps` as a public struct-field construction surface.
    - Keep the existing constructor-style API (`new(...)` plus `with_layout(...)`) while moving
      runtime-only `layout` / `factory` access behind crate-private accessors.
    - Preserve the feature-gated retained-subtree bridge for the remaining migration/quarantine
      decision without letting downstream code depend on its internal layout/factory fields.
  - Validation:
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `git diff --no-index --check /dev/null tools/test_check_layering.py || test $? -eq 1`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/retained_bridge.rs crates/fret-ui/src/declarative/frame.rs crates/fret-ui/src/declarative/host_widget/layout.rs crates/fret-ui/src/declarative/host_widget/measure.rs crates/fret-ui/src/declarative/host_widget/paint.rs crates/fret-ui/src/declarative/mount.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `crates/fret-ui/src/declarative/frame.rs`
    - `crates/fret-ui/src/declarative/host_widget/layout.rs`
    - `crates/fret-ui/src/declarative/host_widget/measure.rs`
    - `crates/fret-ui/src/declarative/host_widget/paint.rs`
    - `crates/fret-ui/src/declarative/mount.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-012-narrow-retained-subtree-props-public-surface`
  - Result:
    - `RetainedSubtreeProps::layout` and `RetainedSubtreeProps::factory` are now private fields.
    - The declarative runtime consumes them through crate-private `layout()` / `factory()`
      accessors.
    - Remaining `fret-ui`, `fret-node`, `fret-plot`, and `fret-chart` retained compatibility
      checks still compile with the narrower public surface.

- [x] RBX-M4-013 Remove `RetainedSubtreeFactory` from the public retained bridge surface.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Turn `RetainedSubtreeFactory` into a crate-visible implementation detail now that
      `RetainedSubtreeProps::factory` is no longer a public field.
    - Keep `RetainedSubtreeProps::new(...)` as the only public entry point that constructs the
      retained-subtree build closure.
    - Preserve all remaining compat islands while removing another public retained bridge type.
  - Validation:
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `git diff --no-index --check /dev/null tools/test_check_layering.py || test $? -eq 1`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/retained_bridge.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-013-remove-retainedsubtreefactory-from-the-public-surface`
  - Result:
    - `RetainedSubtreeFactory` is now `pub(crate)` and `RetainedSubtreeFactory::new(...)` is no
      longer public.
    - External users can still build retained subtrees only through `RetainedSubtreeProps::new(...)`
      and optional `with_layout(...)`.
    - The remaining compat islands still compile after removing the public factory type.

- [x] RBX-M4-014 Delete the retained-subtree declarative bridge.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `crates/fret-ui/src/element.rs`
    - `crates/fret-ui/src/elements/cx.rs`
    - `crates/fret-ui/src/declarative/frame.rs`
    - `crates/fret-ui/src/declarative/mount.rs`
    - `crates/fret-ui/src/declarative/host_widget/layout.rs`
    - `crates/fret-ui/src/declarative/host_widget/measure.rs`
    - `crates/fret-ui/src/declarative/host_widget/paint.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove `RetainedSubtreeProps`, the retained subtree factory implementation, and
      `AppComponentCx::retained_subtree(...)` after scanning found no first-party runtime users.
    - Remove `ElementKind::RetainedSubtree`, `ElementInstance::RetainedSubtree`, and the
      feature-gated mount/layout/measure/paint handling for that leaf.
    - Keep the lower-level retained widget/context bridge intact for the remaining explicit
      `fret-node`, `fret-plot`, and `fret-chart` compatibility islands.
  - Validation:
    - retained-subtree source scan over `crates/fret-ui/src`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `git diff --no-index --check /dev/null tools/test_check_layering.py || test $? -eq 1`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/retained_bridge.rs crates/fret-ui/src/element.rs crates/fret-ui/src/elements/cx.rs crates/fret-ui/src/declarative/frame.rs crates/fret-ui/src/declarative/host_widget/layout.rs crates/fret-ui/src/declarative/host_widget/measure.rs crates/fret-ui/src/declarative/host_widget/paint.rs crates/fret-ui/src/declarative/mount.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `crates/fret-ui/src/element.rs`
    - `crates/fret-ui/src/elements/cx.rs`
    - `crates/fret-ui/src/declarative/frame.rs`
    - `crates/fret-ui/src/declarative/mount.rs`
    - `crates/fret-ui/src/declarative/host_widget/layout.rs`
    - `crates/fret-ui/src/declarative/host_widget/measure.rs`
    - `crates/fret-ui/src/declarative/host_widget/paint.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-014-delete-the-retained-subtree-declarative-bridge`
  - Result:
    - There is no remaining `RetainedSubtreeProps`, `RetainedSubtreeFactory`,
      `AppComponentCx::retained_subtree(...)`, `ElementKind::RetainedSubtree`, or
      `ElementInstance::RetainedSubtree` implementation in `crates/fret-ui/src`.
    - The retained bridge feature now exposes only the lower-level retained widget/context
      compatibility surface needed by the remaining compat islands.
    - `fret-node`, `fret-plot`, and `fret-chart` retained compatibility checks still compile.

- [x] RBX-M4-015 Remove `CommandAvailability` from the retained bridge re-export surface.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Stop exposing the stable command availability enum through
      `fret_ui::retained_bridge::CommandAvailability`.
    - Keep retained-only `CommandAvailabilityCx` in the bridge while callers use the normal
      `fret_ui::CommandAvailability` export for the enum.
    - Preserve the retained command-availability oracle in `fret-node`.
  - Validation:
    - `rg -n "retained_bridge::CommandAvailability\\b|retained_bridge::\\{[^\\n]*\\bCommandAvailability\\b" apps crates ecosystem -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas edit_command_availability`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `git diff --no-index --check /dev/null tools/test_check_layering.py || test $? -eq 1`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/retained_bridge.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-015-remove-commandavailability-from-the-retained-bridge-re-export-surface`
  - Result:
    - `retained_bridge.rs` no longer re-exports `CommandAvailability`.
    - `fret-node` retained command availability code imports the enum from `fret_ui::CommandAvailability`.
    - No workspace Rust source refers to `retained_bridge::CommandAvailability`.

- [x] RBX-M4-016 Remove `Invalidation` from the retained bridge re-export surface.
  - Scope:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Stop exposing the stable invalidation enum through
      `fret_ui::retained_bridge::Invalidation`.
    - Keep retained context types in the bridge while callers use the normal
      `fret_ui::Invalidation` export for the enum.
    - Preserve retained node/plot/chart invalidation behavior.
  - Validation:
    - `rg -n "retained_bridge::Invalidation\\b|retained_bridge::\\{[^\\n]*\\bInvalidation\\b" apps crates ecosystem -g '*.rs' || test $? -eq 1`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas invalidation_ordering_conformance geometry_overrides_invalidation_conformance`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo fmt --all -- --check`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `git diff --no-index --check /dev/null tools/test_check_layering.py || test $? -eq 1`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/retained_bridge.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs ecosystem/fret-plot/src/retained/canvas/mod.rs ecosystem/fret-chart/src/retained/canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/retained_bridge.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-016-remove-invalidation-from-the-retained-bridge-re-export-surface`
  - Result:
    - `retained_bridge.rs` no longer re-exports `Invalidation`.
    - Remaining retained node/plot/chart code imports the enum from `fret_ui::Invalidation`.
    - No workspace Rust source refers to `retained_bridge::Invalidation`.

- [x] RBX-M4-017 Delete the legacy `fret_ui::retained_bridge` alias module.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/retained_bridge.rs`
    - `apps/fret-cookbook/src/lib.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the deprecated `fret_ui::retained_bridge` module now that current workspace code uses
      the explicit `fret_ui::compat_retained_canvas` facade.
    - Preserve remaining retained widget/context compatibility for `fret-node`, `fret-plot`, and
      `fret-chart` through `compat_retained_canvas`.
    - Keep policy tests rejecting old `retained_bridge` imports in first-party examples and node UI
      surfaces.
  - Validation:
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo check -p fret-cookbook`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `cargo nextest run -p fret-examples --test basic_chart_demos_surface`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `rg -n "fret_ui::retained_bridge|use fret_ui::retained_bridge|pub mod retained_bridge|retained_bridge::" apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/lib.rs apps/fret-cookbook/src/lib.rs ecosystem/fret-node/src/lib.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `apps/fret-cookbook/src/lib.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-23---rbx-m4-017-delete-the-legacy-fret_uiretained_bridge-alias-module`
  - Result:
    - `crates/fret-ui/src/retained_bridge.rs` was deleted and `fret-ui` no longer exports
      `pub mod retained_bridge`.
    - Remaining compat islands continue to compile against `fret_ui::compat_retained_canvas`.
    - Workspace Rust sources no longer contain real `fret_ui::retained_bridge` imports; remaining
      occurrences are policy-test forbidden strings.

- [x] RBX-M4-018 Remove `UiTreeRetainedExt` from the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the public extension-trait shape from the remaining retained compatibility facade.
    - Keep only the narrower explicit free function
      `fret_ui::compat_retained_canvas::create_node_retained(...)` for the remaining retained
      canvas islands.
    - Preserve retained plot/chart/node compatibility while preventing downstream code from
      depending on a broad `UiTree` extension trait.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-chart/src/retained/canvas.rs ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-node --features compat-retained-canvas retained_bridge_source_usage_stays_on_the_migration_ledger`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `rg -n "\\.create_node_retained\\(" apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `rg -n "trait UiTreeRetainedExt|impl<.*UiTreeRetainedExt|impl .*UiTreeRetainedExt" crates/fret-ui/src/compat_retained_canvas.rs apps crates ecosystem --glob '*.rs' || test $? -eq 1`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-chart/src/retained/canvas.rs ecosystem/fret-plot/src/retained/canvas/mod.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-plot/src/retained/canvas/mod.rs`
    - `ecosystem/fret-chart/src/retained/canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-018-remove-uitreeretainedext-from-the-compat-retained-canvas-facade`
  - Result:
    - `compat_retained_canvas` now exposes `create_node_retained(...)` as a free function rather
      than a `UiTreeRetainedExt` public extension trait.
    - `fret-plot` and `fret-chart` retained canvas helpers call the free function explicitly.
    - No workspace Rust source calls `.create_node_retained(...)` or defines/implements
      `UiTreeRetainedExt`; remaining `UiTreeRetainedExt` strings are policy-test forbidden
      markers.

- [x] RBX-M4-019 Lock the compat retained canvas facade shape with a source-policy gate.
  - Scope:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Turn the facade's "do not grow" quarantine comment into executable policy.
    - Require the remaining facade to expose only retained widget/context types plus the explicit
      `create_node_retained(...)` free function.
    - Reject legacy retained bridge surface regrowth, including extension traits, retained subtree
      bridge types, old text/viewport helpers, stable enum re-exports, and legacy module naming.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/compat_retained_canvas.rs`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge compat_retained_canvas_facade`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/compat_retained_canvas.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-019-lock-the-compat-retained-canvas-facade-shape-with-a-source-policy-gate`
  - Result:
    - Added `compat_retained_canvas_facade_exports_only_retained_widget_contexts`.
    - Added `compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface`.
    - The gate slices the facade source before its test module, so forbidden marker strings in the
      test itself do not mask real facade regrowth.

- [x] RBX-M4-035 Remove the unused `create_node_retained` free function from the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Narrow the compat retained canvas facade to the retained widget/context exports only now that
      no workspace Rust source calls `create_node_retained(...)`.
    - Keep the node source-policy gate on the explicit compat facade import path.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/compat_retained_canvas.rs`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-035-remove-the-unused-createnode_retained-free-function-from-the-compat-retained-canvas-facade`
  - Result:
    - `compat_retained_canvas` now exports only the retained widget/context types required by the
      remaining compat islands.
    - No workspace Rust source calls `create_node_retained(...)`; remaining occurrences are
      policy-test forbidden strings.

- [x] RBX-M4-036 Quarantine node-only command/frame retained contexts behind compat_retained_canvas submodules.
  - Scope:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/command.rs`
    - `crates/fret-ui/src/compat_retained_canvas/frame.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Split the node-only retained command and frame contexts out of the top-level compat facade
      so the main retained bridge surface stays limited to the core widget contexts.
    - Keep explicit import paths and separate policy coverage for the quarantined node-only
      submodules.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/compat_retained_canvas.rs crates/fret-ui/src/compat_retained_canvas/command.rs crates/fret-ui/src/compat_retained_canvas/frame.rs crates/fret-ui/src/compat_retained_canvas/layout.rs crates/fret-ui/src/compat_retained_canvas/paint.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface) | test(compat_retained_canvas_command_facade_exports_only_command_contexts) | test(compat_retained_canvas_frame_facade_exports_only_frame_contexts)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_canvas_facade_usage_stays_explicit_not_globbed)'`
    - `cargo check -p fret-plot --features compat-retained-canvas`
    - `cargo check -p fret-chart --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
    - `rg -n "^(<<<<<<<|=======|>>>>>>>)" crates/fret-ui/src/compat_retained_canvas.rs crates/fret-ui/src/compat_retained_canvas ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/ui/canvas/widget ecosystem/fret-node/src/lib.rs docs/workstreams/retained-bridge-exit-v1 || test $? -eq 1`
  - Evidence:
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/command.rs`
    - `crates/fret-ui/src/compat_retained_canvas/frame.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-036-quarantine-node-only-commandframe-retained-contexts-behind-compat_retained_canvas-submodules`
  - Result:
    - The top-level `compat_retained_canvas` facade is now namespace-only; the retained
      contexts live behind explicit `command`, `event`, `frame`, `layout`, `paint`, and
      `widget` submodules.
    - Node-only `CommandCx` / `CommandAvailabilityCx` live under
      `compat_retained_canvas::command`, `EventCx` lives under `compat_retained_canvas::event`,
      `PrepaintCx` / `SemanticsCx` live under `compat_retained_canvas::frame`, and `LayoutCx` /
      `PaintCx` live under `compat_retained_canvas::layout` / `compat_retained_canvas::paint`.
    - `fret-node` runtime code and test helpers now import the quarantined contexts explicitly,
      including the last `wire_drag/retained_commit_cx.rs` seam, and the focused `fret-ui` /
      `fret-node` policy checks still pass.

- [x] RBX-M4-037 Move the `Widget` trait out of the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Re-export the retained `Widget` trait from the stable `fret_ui` root so the remaining
      retained compatibility facade stays context-only.
    - Delete the `compat_retained_canvas::widget` entry point after migrating the remaining
      `fret-node` imports and add a source-policy gate that rejects regressions.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/lib.rs crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-037-move-the-widget-trait-out-of-the-compat-retained-canvas-facade`
  - Result:
    - `fret_ui::Widget` is now a stable top-level export.
    - `compat_retained_canvas` no longer exposes a `widget` submodule; the compat facade is
      context-only again.
    - `fret-node` runtime and test imports now use `fret_ui::Widget`, and a source-policy gate
      rejects future `compat_retained_canvas::widget` regrowth.

- [x] RBX-M4-038 Move command contexts out of the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/command.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/edit_command_availability_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Re-export `CommandCx` and `CommandAvailabilityCx` from the stable `fret_ui` root so the
      remaining retained compatibility facade stays context-only.
    - Delete the `compat_retained_canvas::command` entry point after migrating the remaining
      `fret-node` imports and add a source-policy gate that rejects regressions.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/lib.rs crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_canvas_command_contexts_stay_on_stable_command_export) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/wire_drag/retained_commit_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-038-move-command-contexts-out-of-the-compat-retained-canvas-facade`
  - Result:
    - `fret_ui::CommandCx` and `fret_ui::CommandAvailabilityCx` are now stable top-level exports.
    - `compat_retained_canvas` no longer exposes a `command` submodule; the compat facade is
      command-free again.
    - `fret-node` runtime and test imports now use the stable command context exports, and a
      source-policy gate rejects future `compat_retained_canvas::command` regrowth.

- [x] RBX-M4-039 Move event contexts out of the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/event.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/**`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Re-export `EventCx` from the stable `fret_ui` root so the remaining retained
      compatibility facade stays limited to the still-quarantined frame/layout/paint contexts.
    - Delete the `compat_retained_canvas::event` entry point after migrating the remaining
      `fret-node` imports and add a source-policy gate that rejects regressions.
  - Validation:
    - `git diff --name-only -- '*.rs' | rg '^(crates/fret-ui/src/(lib\\.rs|compat_retained_canvas\\.rs|compat_retained_canvas/)|ecosystem/fret-node/src/)' | while IFS= read -r f; do [ -f "$f" ] && printf '%s\\0' "$f"; done | xargs -0 rustfmt --edition 2024 --check`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface) | test(compat_retained_canvas_frame_facade_exports_only_frame_contexts) | test(compat_retained_canvas_layout_facade_exports_only_layout_contexts) | test(compat_retained_canvas_paint_facade_exports_only_paint_contexts)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_canvas_command_contexts_stay_on_stable_command_export) | test(retained_canvas_event_contexts_stay_on_stable_event_export) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "compat_retained_canvas::event|pub mod event" crates/fret-ui/src/compat_retained_canvas.rs crates/fret-ui/src/compat_retained_canvas ecosystem/fret-node/src/ui -g '*.rs' || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/**/retained*_cx.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/mod.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/harness/contexts.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-039-move-event-contexts-out-of-the-compat-retained-canvas-facade`
  - Result:
    - `fret_ui::EventCx` is now a stable top-level export.
    - `compat_retained_canvas` no longer exposes an `event` submodule; the compat facade is
      reduced to `frame`, `layout`, and `paint`.
    - `fret-node` runtime and test imports now use the stable event context export, and a
      source-policy gate rejects future `compat_retained_canvas::event` regrowth.

- [x] RBX-M4-040 Move layout contexts out of the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/layout.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/fit_view_on_mount_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Re-export `LayoutCx` from the stable `fret_ui` root so the remaining retained
      compatibility facade stays limited to the still-quarantined frame/paint contexts.
    - Delete the `compat_retained_canvas::layout` entry point after migrating the remaining
      `fret-node` imports and add a source-policy gate that rejects regressions.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/lib.rs crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/tests/fit_view_on_mount_conformance.rs ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface) | test(compat_retained_canvas_frame_facade_exports_only_frame_contexts) | test(compat_retained_canvas_paint_facade_exports_only_paint_contexts)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_canvas_command_contexts_stay_on_stable_command_export) | test(retained_canvas_event_contexts_stay_on_stable_event_export) | test(retained_canvas_layout_contexts_stay_on_stable_layout_export) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "compat_retained_canvas::layout::LayoutCx" ecosystem/fret-node/src/ui -g '*.rs' || test $? -eq 1`
    - `rg -n "pub mod layout" crates/fret-ui/src/compat_retained_canvas.rs crates/fret-ui/src/compat_retained_canvas -g '*.rs' || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/fit_view_on_mount_conformance.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/perf_cache.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-040-move-layout-contexts-out-of-the-compat-retained-canvas-facade`
  - Result:
    - `fret_ui::LayoutCx` is now a stable top-level export.
    - `compat_retained_canvas` no longer exposes a `layout` submodule; the compat facade is
      frame/paint-only again.
    - `fret-node` runtime and test imports now use the stable layout context export, and a
      source-policy gate rejects future `compat_retained_canvas::layout` regrowth.

- [x] RBX-M4-041 Move paint contexts out of the compat retained canvas facade.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/paint.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/*.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Re-export `PaintCx` from the stable `fret_ui` root so the remaining retained
      compatibility facade stays limited to the still-quarantined frame context.
    - Delete the `compat_retained_canvas::paint` entry point after migrating the remaining
      `fret-node` imports and add a source-policy gate that rejects regressions.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/lib.rs crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/tests/*.rs`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_only_retained_widget_contexts) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface) | test(compat_retained_canvas_frame_facade_exports_only_frame_contexts)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_canvas_command_contexts_stay_on_stable_command_export) | test(retained_canvas_event_contexts_stay_on_stable_event_export) | test(retained_canvas_layout_contexts_stay_on_stable_layout_export) | test(retained_canvas_paint_contexts_stay_on_stable_paint_export) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n "compat_retained_canvas::paint::PaintCx" ecosystem/fret-node/src/ui -g '*.rs' || test $? -eq 1`
    - `rg -n "pub mod paint" crates/fret-ui/src/compat_retained_canvas.rs crates/fret-ui/src/compat_retained_canvas -g '*.rs' || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/*.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-041-move-paint-contexts-out-of-the-compat-retained-canvas-facade`
  - Result:
    - `fret_ui::PaintCx` is now a stable top-level export.
    - `compat_retained_canvas` no longer exposes a `paint` submodule; the compat facade is
      frame-only again.
    - `fret-node` runtime and test imports now use the stable paint context export, and a
      source-policy gate rejects future `compat_retained_canvas::paint` regrowth.

- [x] RBX-M4-042 Delete the remaining compat retained canvas frame submodule and keep the shell empty.
  - Scope:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `crates/fret-ui/src/compat_retained_canvas/frame.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/*.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Re-export `PrepaintCx` and `SemanticsCx` from the stable `fret_ui` root so the retained
      compatibility facade no longer needs any public submodules.
    - Delete the `compat_retained_canvas::frame` entry point after migrating the remaining
      `fret-node` imports and keep the compat module as an empty quarantine shell with a
      source-policy gate that rejects regressions.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/lib.rs crates/fret-ui/src/compat_retained_canvas.rs ecosystem/fret-node/src/ui/canvas/widget.rs ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/canvas/widget/tests/*.rs`
    - `cargo check -p fret-ui --features unstable-retained-bridge`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `cargo nextest run -p fret-ui --features unstable-retained-bridge -E 'test(compat_retained_canvas_facade_exports_no_public_context_modules) | test(compat_retained_canvas_facade_does_not_regrow_legacy_bridge_surface)'`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_canvas_command_contexts_stay_on_stable_command_export) | test(retained_canvas_event_contexts_stay_on_stable_event_export) | test(retained_canvas_layout_contexts_stay_on_stable_layout_export) | test(retained_canvas_paint_contexts_stay_on_stable_paint_export) | test(retained_canvas_frame_contexts_stay_on_stable_frame_export) | test(retained_bridge_source_usage_stays_on_the_migration_ledger)'`
    - `rg -n \"compat_retained_canvas::frame::PrepaintCx|compat_retained_canvas::frame::SemanticsCx|compat_retained_canvas::frame\" ecosystem/fret-node/src/ui -g '*.rs' || test $? -eq 1`
    - `rg -n \"pub mod frame\" crates/fret-ui/src/compat_retained_canvas.rs crates/fret-ui/src/compat_retained_canvas -g '*.rs' || test $? -eq 1`
    - `python3 tools/check_layering.py`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget.rs`
    - `ecosystem/fret-node/src/ui/canvas/widget/tests/*.rs`
    - `ecosystem/fret-node/src/lib.rs`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-042-delete-the-remaining-compat-retained-canvas-frame-submodule-and-keep-the-shell-empty`
  - Result:
    - `fret_ui::PrepaintCx` and `fret_ui::SemanticsCx` are now stable top-level exports.
    - `compat_retained_canvas` no longer exposes any public submodules; it is now an empty
      quarantine shell.
    - `fret-node` runtime and test imports now use the stable frame context exports, and a
      source-policy gate rejects future `compat_retained_canvas::frame` regrowth.

- [x] RBX-M4-043 Delete the empty retained bridge feature and compat facade.
  - Scope:
    - `crates/fret-ui/Cargo.toml`
    - `crates/fret-ui/src/lib.rs`
    - `crates/fret-ui/src/compat_retained_canvas.rs`
    - `ecosystem/fret-node/Cargo.toml`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `tools/check_layering.py`
    - `tools/test_check_layering.py`
    - `docs/adr/0052-ui-host-runtime-boundary.md`
    - `docs/adr/0077-resizable-panel-groups-and-docking-split-sizing.md`
    - `docs/adr/0096-plot-widgets-and-crate-placement.md`
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
    - `docs/crate-usage-guide.md`
    - `docs/roadmap.md`
    - `docs/shadcn-declarative-progress.md`
    - `docs/workstreams/retained-bridge-exit-v1/*`
  - Goal:
    - Remove the now-empty `fret-ui/unstable-retained-bridge` feature and the empty
      `fret_ui::compat_retained_canvas` module after all retained bridge exports moved to stable
      root exports or out of first-party crates.
    - Keep `fret-node/compat-retained-canvas` as a node-local legacy implementation gate without
      mapping to any `fret-ui` retained bridge feature.
    - Update layering policy so any future package feature or direct dependency that tries to
      enable `fret-ui/unstable-retained-bridge` is rejected instead of allowlisted.
  - Validation:
    - `rustfmt --edition 2024 --check crates/fret-ui/src/lib.rs ecosystem/fret-node/src/lib.rs ecosystem/fret-node/src/ui/mod.rs`
    - `cargo check -p fret-ui`
    - `cargo check -p fret-node --features compat-retained-canvas`
    - `python3 tools/test_check_layering.py`
    - `python3 tools/check_layering.py`
    - `cargo nextest run -p fret-node --features compat-retained-canvas -E 'test(retained_compatibility_surface_stays_declarative_only) | test(retained_bridge_source_usage_stays_on_the_migration_ledger) | test(retained_canvas_facade_usage_stays_explicit_not_globbed) | test(retained_canvas_widget_trait_stays_on_stable_widget_export) | test(retained_canvas_command_contexts_stay_on_stable_command_export) | test(retained_canvas_event_contexts_stay_on_stable_event_export) | test(retained_canvas_layout_contexts_stay_on_stable_layout_export) | test(retained_canvas_paint_contexts_stay_on_stable_paint_export) | test(retained_canvas_frame_contexts_stay_on_stable_frame_export)'`
    - `rg -n "unstable-retained-bridge" Cargo.toml apps crates ecosystem -g 'Cargo.toml' -g '*.rs' || test $? -eq 1`
    - `rg -n "pub mod compat_retained_canvas|compat_retained_canvas.rs" crates/fret-ui ecosystem/fret-node -g '*.rs' -g 'Cargo.toml' || test $? -eq 1`
    - `python3 tools/check_workstream_catalog.py`
    - `git diff --check`
  - Evidence:
    - `crates/fret-ui/Cargo.toml`
    - `crates/fret-ui/src/lib.rs`
    - `ecosystem/fret-node/Cargo.toml`
    - `ecosystem/fret-node/src/lib.rs`
    - `ecosystem/fret-node/src/ui/mod.rs`
    - `tools/check_layering.py`
    - `tools/test_check_layering.py`
    - `docs/adr/0052-ui-host-runtime-boundary.md`
    - `docs/adr/0077-resizable-panel-groups-and-docking-split-sizing.md`
    - `docs/adr/0096-plot-widgets-and-crate-placement.md`
    - `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
    - `docs/crate-usage-guide.md`
    - `docs/roadmap.md`
    - `docs/shadcn-declarative-progress.md`
    - `docs/workstreams/retained-bridge-exit-v1/EVIDENCE_AND_GATES.md#2026-05-24---rbx-m4-043-delete-the-empty-retained-bridge-feature-and-compat-facade`
  - Result:
    - `fret-ui` no longer defines `unstable-retained-bridge` and no longer exports
      `compat_retained_canvas`.
    - `fret-node/compat-retained-canvas` remains only as a node-local legacy implementation gate
      mapped to `fret-ui`, not to a retained bridge feature.
    - Layering policy now treats any `fret-ui/unstable-retained-bridge` feature mapping as a
      regression.

- [x] Delete or quarantine any further retained bridge exports not required by remaining clients.
- [x] If allowlist becomes empty: remove `fret-ui/unstable-retained-bridge` feature and all bridge code.
- [x] Otherwise: quarantine the remaining retained path behind a narrower, clearly named compatibility facade with
  explicit “do not grow” policy and separate tracking. Superseded by RBX-M4-043 because the allowlist became empty.
