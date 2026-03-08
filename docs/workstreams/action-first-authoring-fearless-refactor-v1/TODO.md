# Action-First Authoring + View Runtime (Fearless Refactor v1) — TODO

Status: Landed (v1), hardening follow-ups in progress
Last updated: 2026-03-08

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Evidence/gates: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Teaching-surface inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`

ADRs (decision gates for this workstream):

- `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`

Tracking legend:

- `[ ]` open
- `[~]` in progress
- `[x]` done
- `[!]` blocked

ID format:

- `AFA-{area}-{nnn}`

---

## A. Decision + Contract Locking

- [x] AFA-adr-001 Review ADR 0307 (actions) for scope/ownership boundaries.
- [x] AFA-adr-002 Review ADR 0308 (view runtime) for hook order/keying rules and cache boundary semantics.
- [x] AFA-adr-003 Update `docs/adr/README.md` jump table with new action/view ADR anchors.
- [x] AFA-adr-004 Decide keymap strategy (v1):
  - Decision: `ActionId == CommandId` (alias/wrapper; no keymap schema churn in v1).
  - Evidence: ADR 0307 “v1 decision (locked)”.
- [x] AFA-adr-005 Add a short action naming convention note (namespace + `.v1` suffix).
  - Goal: keep IDs predictable for GenUI and future frontends.
- [x] AFA-adr-006 Add an observability checklist for action dispatch + view dirty/reuse.
  - Evidence: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

---

## B. Action System (Additive v1)

- [x] AFA-actions-010 Define the `ActionId` type and metadata surface.
  - Evidence: `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
  - Status (as of 2026-03-03):
    - Implemented: `ActionId` portable identity (`crates/fret-runtime/src/action.rs`)
    - Implemented: action metadata aliases (`ActionMeta` / `ActionRegistry`) reuse the command registry surface (`crates/fret-runtime/src/action.rs`)
    - Implemented: command palette uses host command registry (`ecosystem/fret-ui-shadcn/src/command.rs`)
- [x] AFA-actions-011 Provide an ecosystem macro for defining typed unit actions with stable IDs.
  - Goal: avoid stringly `"my.action.id"` constants in app code.
  - Evidence:
    - Macro: `ecosystem/fret/src/actions.rs`
    - Compile/test: `cargo test -p fret --lib actions::tests::typed_actions_convert_to_command_id`
- [x] AFA-actions-012 Add a minimal action handler table API for views/frontends.
  - Goal: IR binds `ActionId`; handlers live in view/app layer.
  - Evidence:
    - `ecosystem/fret/src/actions.rs` (`ActionHandlerTable`, `build()` adapters)
- [x] AFA-actions-013 Integrate action availability queries with input dispatch v2 semantics.
  - Evidence:
    - `docs/adr/0218-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
    - `crates/fret-ui/src/tree/commands.rs` (`publish_window_command_action_availability_snapshot`)
    - `crates/fret-ui/src/tree/tests/window_command_action_availability_snapshot.rs`
- [x] AFA-actions-014 Add diagnostics traces for:
  - keymap resolution → action id,
  - availability gating outcome,
  - dispatch path resolution.
  - Status (as of 2026-03-03):
    - Implemented (keymap → action id): `crates/fret-runtime/src/shortcut_routing_diagnostics.rs` +
      `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs` (`UiShortcutRoutingTraceEntryV1.command`)
    - Implemented (availability gating outcome): `ecosystem/fret-bootstrap/src/ui_diagnostics/command_gating_trace.rs`
      (`debug.command_gating_trace[*]`)
    - Implemented (dispatch path resolution, best-effort): `crates/fret-runtime/src/command_dispatch_diagnostics.rs` +
      `crates/fret-ui/src/tree/commands.rs` + `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs` +
      `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
      (`debug.command_dispatch_trace[*]` / script evidence, including handled-by element, handled-by scope, driver-handled classification, and default-root fallback)
    - Gated (scripted): `crates/fret-diag-protocol/src/lib.rs` (`UiActionStepV2::WaitCommandDispatchTrace`) +
      `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs` (`handle_wait_command_dispatch_trace_step`) +
      `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
    - Implemented (pointer → stable selector): command dispatch trace entries can include `source_test_id`
      for pointer-triggered dispatch (best-effort).
      - Scripted pointer injection: stamps the selector `test_id` as the `source_test_id` and records
        it alongside the injected step.
      - Fallback: derives `source_test_id` from the hit-test trace when available.
    - Gated (scripted): `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json` asserts
      `source_test_id == cookbook.hello.button` for `cookbook.hello.click.v1`.
    - Implemented (script determinism): the golden-path driver flushes `Effect::Command` after
      script-injected input so `wait_command_dispatch_trace` can observe dispatch decisions without
      depending on runner-level effect timing.
    - Implemented (shortcut correctness): widget-scoped shortcut gating prefers live UI-tree
      availability to avoid stale `command_disabled` decisions after modal barriers close.
- [x] AFA-actions-015 Converge command palette/menu invocation with action dispatch.
  - Goal: palette/menu triggers and pointer triggers share the same action pipeline.
  - Evidence:
    - `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (command palette overlay builds command entries and dispatches via the window command pipeline)
    - `ecosystem/fret-ui-shadcn/src/command.rs` (command palette selection queues a pending command and dispatches via `Effect::Command` after close-on-select completes)
    - `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json` (command palette → action handler gate)
- [x] AFA-actions-019 Make `keyctx.*` gating observable and consistent across surfaces.
  - Goal: the same `when` expression (ADR 0022) drives:
    - keymap matching,
    - command enablement/visibility (menus + palette),
    - shortcut display (best-effort reverse lookup),
    - diagnostics traces.
  - Evidence:
    - `crates/fret-runtime/src/when_expr/*` (`WhenEvalContext`, `keyctx.*`)
    - `crates/fret-runtime/src/window_key_context_stack.rs` (`WindowKeyContextStackService`)
    - `crates/fret-ui/src/tree/dispatch/window.rs` (publishes window key-context snapshots)
    - `crates/fret-runtime/src/window_command_gating/snapshot.rs` (`eval_with_key_contexts`)
    - `crates/fret-runtime/src/keymap/display.rs` (`display_shortcut_for_command_sequence_with_key_contexts`)
    - `ecosystem/fret-ui-shadcn/src/command.rs` (palette shortcut display uses key contexts)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/command_gating_trace.rs` (gating trace uses key contexts)
    - `ecosystem/fret/src/workspace_menu.rs` + `crates/fret-launch/src/runner/desktop/runner/windows_menu.rs` (menu `when` uses key contexts)
  - Gates:
    - `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json` (shortcut routing trace includes key contexts)

### B.1 Authoring integration (pointer triggers)

- [x] AFA-actions-016 Add action-first binding convenience for shadcn `Button`.
  - Evidence: `ecosystem/fret-ui-shadcn/src/button.rs` (`Button::action`)
- [x] AFA-actions-017 Add action-first naming parity helpers in `fret-ui-kit`.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/command.rs` (`action_is_enabled`, `dispatch_action_if_enabled`)
    - `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (`pressable_dispatch_action_if_enabled`)
- [x] AFA-actions-018 Ensure action availability/dispatch can reach app handlers from overlay roots.
  - Goal: portal-mounted menus/overlays can invoke app-level actions without duplicating handler tables.
  - Evidence:
    - `crates/fret-ui/src/tree/commands.rs` (dispatch/availability fallback to default root)
    - `crates/fret-ui/src/tree/tests/command_availability.rs` (cross-layer fallback tests)

---

## C. View Runtime + Hooks (Ecosystem)

- [x] AFA-view-020 Decide crate placement for the view runtime:
  - Decision: land in `ecosystem/fret` for v1; defer split crate until after adoption.
  - Evidence: ADR 0308 “v1 decision (locked)”.
- [x] AFA-view-021 Implement a minimal `View` trait + `ViewCx` with:
  - action handler registration,
  - `notify()` dirty marking,
  - `use_state` (element/view state slots),
  - `use_selector` (re-export from `fret-selector`),
  - `use_query` (re-export from `fret-query`).
  - Status (as of 2026-03-02):
    - Implemented (v1): `ecosystem/fret/src/view.rs`
    - Entry points: `ecosystem/fret/src/app_entry.rs` (`App::run_view`)
    - First adoption: `apps/fret-cookbook/examples/hello.rs`
- [x] AFA-view-022 Define and document hook keying rules:
  - stable callsite key for non-loop hooks,
  - required keyed variants for loops (`use_*_keyed`),
  - diagnostics for misuse (debug-only).
  - Evidence:
    - ADR update: `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
    - `use_state_keyed` + debug rail: `ecosystem/fret/src/view.rs`
- [x] AFA-view-023 Add a view-cache boundary helper aligned with ADR 0213:
  - “cached unless dirty” semantics,
  - inspection/picking disables reuse.
  - Evidence:
    - Helper: `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs`
    - Reuse gating: `crates/fret-ui/src/tree/ui_tree_view_cache.rs` (`UiTree::view_cache_active`)
- [x] AFA-view-024 Provide an adapter path for MVU:
  - keep MVU available while views are adopted,
  - document “when to use MVU vs View” in cookbook guidance.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md` (“When to use MVU vs View”)
- [x] AFA-view-025 Add view-level observability:
  - “why did this view rebuild?”
  - “why was reuse skipped?”
  - “which models/globals were observed?”
  - Evidence:
    - `debug.dirty_views` + `debug.notify_requests`: `ecosystem/fret-bootstrap/src/ui_diagnostics/invalidation_diagnostics.rs`
    - `debug.cache_roots[*].reuse_reason`: `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
    - view-cache reason source: `crates/fret-ui/src/declarative/mount.rs`
- [x] AFA-view-026 Provide a safe “app effects” scheduling helper for views:
  - Goal: allow `cx.on_action*` handlers to request `App`-scoped effects (e.g. `fret-query`
    invalidation) with a boring, reusable pattern that avoids allocating a dedicated model for
    simple “one-shot” effects.
  - Implemented (v1): transient event scheduling at the view action root.
  - Evidence:
    - Helpers: `ecosystem/fret/src/view.rs` (`ViewCx::on_action_notify_transient`, `ViewCx::take_transient_on_action_root`)
    - Adoption: `apps/fret-examples/src/query_demo.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs`

---

## D. Frontend Convergence (Declarative + imui + GenUI)

- [x] AFA-frontends-030 Add an imui seam to dispatch `ActionId` without string commands.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/imui.rs` (`action_button_ex`, `menu_item_action_ex`)
    - `apps/fret-cookbook/examples/imui_action_basics.rs`
    - `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
- [x] AFA-frontends-031 Ensure imui outputs stable semantics/test IDs for diag scripts.
  - Evidence: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
- [x] AFA-frontends-032 Align GenUI action bindings with `ActionId` conventions (namespace/versioning).
  - Evidence:
    - `docs/workstreams/genui-json-render-v1.md` (ActionId/CommandId naming + executor glue note)
    - `ecosystem/fret-genui-core/src/executor.rs` (`GenUiActionExecutorV1::with_dispatch_command_actions`)
- [x] AFA-frontends-033 Add at least one cross-frontend demo:
  - a Rust view triggers an action,
  - an imui panel triggers the same action,
  - a GenUI spec triggers a catalog-approved action ID (strict catalog validation).
  - Evidence:
    - `apps/fret-cookbook/examples/imui_action_basics.rs`
    - `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
- [x] AFA-frontends-034 Add facade-level wrappers for imui menu items that dispatch `ActionId`.
  - Goal: keep focusability tracking (initial focus selection) consistent with action availability gating.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/imui.rs` (`ImUiFacade::menu_item_action_ex`)

---

## E. Adoption (Cookbook + Gallery + Editor-grade shells)

- [x] AFA-adopt-040 Migrate 2–3 cookbook demos to the new View + actions path.
  - Suggested: `apps/fret-cookbook/examples/hello.rs`, `overlay_basics.rs`, `commands_keymap_basics.rs`.
  - Status (as of 2026-03-03):
    - View runtime + action-first adoption landed for `commands_keymap_basics`:
      `apps/fret-cookbook/examples/commands_keymap_basics.rs`
    - View runtime + action-first adoption landed for `hello`:
      `apps/fret-cookbook/examples/hello.rs`
    - View runtime + action-first adoption landed for `overlay_basics`:
      `apps/fret-cookbook/examples/overlay_basics.rs`
    - View runtime + action-first adoption landed for `hello_counter`:
      `apps/fret-cookbook/examples/hello_counter.rs`
    - View runtime + action-first adoption landed for `text_input_basics`:
      `apps/fret-cookbook/examples/text_input_basics.rs`
    - Additional cookbook migrations landed (now fully converged on view runtime + typed actions):
      - `apps/fret-cookbook/examples/simple_todo.rs`
      - `apps/fret-cookbook/examples/theme_switching_basics.rs`
      - `apps/fret-cookbook/examples/undo_basics.rs`
      - `apps/fret-cookbook/examples/async_inbox_basics.rs`
      - `apps/fret-cookbook/examples/virtual_list_basics.rs`
      - `apps/fret-cookbook/examples/customv1_basics.rs`
      - `apps/fret-cookbook/examples/embedded_viewport_basics.rs`
      - `apps/fret-cookbook/examples/external_texture_import_basics.rs`
      - `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
      - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
      - `apps/fret-cookbook/examples/commands_keymap_basics.rs`
      - `apps/fret-cookbook/examples/router_basics.rs`
      - `apps/fret-cookbook/examples/effects_layer_basics.rs`
      - `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
      - `apps/fret-cookbook/examples/utility_window_materials_windows.rs`
      - `apps/fret-cookbook/examples/drop_shadow_basics.rs`
      - `apps/fret-cookbook/examples/overlay_basics.rs`
      - `apps/fret-cookbook/examples/toast_basics.rs`
      - `apps/fret-cookbook/examples/query_basics.rs`
      - `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`
    - Inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
- [x] AFA-adopt-041 Add at least one ui-gallery page/snippet using actions + view runtime.
  - Evidence:
    - `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`
    - `apps/fret-ui-gallery/src/ui/pages/command.rs`
- [x] AFA-adopt-042 Add one editor-grade harness adoption:
  - docking/workspace shell uses actions for tab/command semantics (where appropriate).
  - Status (as of 2026-03-03):
    - Workspace tab strip pointer-triggered dispatches record a command dispatch trace source:
      - `ecosystem/fret-workspace/src/tab_strip/mod.rs` (tab activate)
      - `ecosystem/fret-workspace/src/tab_strip/state.rs` (one-shot reveal of the active tab on first layout, to stabilize hit targets for scripts and users)
      - `ecosystem/fret-workspace/src/tab_strip/widgets.rs` (tab close button)
      - `ecosystem/fret-workspace/src/tab_strip/interaction.rs` (right/middle click behaviors)
      - `ecosystem/fret-workspace/src/command_scope.rs` (workspace-level command scope fallback for `workspace.*` commands)
    - Scripted diagnostics gate:
      - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json` (asserts `source_kind=pointer` for the close command)
      - `tools/diag_gate_action_first_authoring_v1.ps1` (includes workspace shell demo gate)
- [x] AFA-adopt-043 Update `fretboard` scaffold templates to prefer action-first patterns (once v1 is stable).
  - Rule: do not ship two different default paradigms in templates.
  - Status (as of 2026-03-05):
    - `cargo run -p fretboard -- new hello` uses View runtime + typed unit actions:
      `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`)
    - `cargo run -p fretboard -- new todo` uses View runtime + typed unit actions + selector/query hooks:
      `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`)
    - `cargo run -p fretboard -- new simple-todo` uses View runtime + typed unit actions:
      `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`)
    - All templates demonstrate “late `into_element(cx)` + `ui::children![cx; ...]`” (low adapter noise),
      with unit tests guarding against `into_element(cx)` regression:
      `apps/fretboard/src/scaffold/templates.rs` (template tests).
    - Templates prefer `cx.on_action_notify::<A>(...)` to avoid repeating `request_redraw(...)` + `notify(...)`
      boilerplate in action handlers (keeps view-cache closure participation consistent).

- [x] AFA-adopt-044 Migrate `embedded_viewport_demo` to the view runtime.
  - Goal: prove view-runtime authoring composes cleanly with embedded viewport interop:
    - `viewport_input(...)` forwarding,
    - and a custom `record_engine_frame(...)` hook for offscreen engine passes.
  - Why this matters: `UiAppDriver` only supports a single `record_engine_frame` hook; view runtime
    currently uses it for view-cache enablement (v1), while embedded viewport needs it for engine
    recording. The migrated demo should demonstrate the correct composition pattern.
  - Evidence:
    - `apps/fret-examples/src/embedded_viewport_demo.rs` (composed `record_engine_frame`)
    - `apps/fret-demo/src/main.rs` (demo routing)
    - `tools/diag-scripts/viewport/embedded-demo/embedded-viewport-demo-input-forwarding.json` (input forwarding smoke)
    - `ecosystem/fret/src/interop/embedded_viewport.rs`
    - `ecosystem/fret/src/app_entry.rs`

- [~] AFA-adopt-045 Reduce “early element landing” noise in cookbook demos (polish pass).
  - Goal: prefer late-landing child collection (`ui::children![cx; ...]`, `*_::build(...)`) and keep
    `test_id` / key-context / semantics patches on the builder path whenever possible.
  - Non-goal (for this pass): introducing a new UI macro/DSL or replacing `ui::children!` with a new
    mandatory composition language (that is a post-v1/v2 ergonomics decision).
  - Evidence (recent slice):
    - `apps/fret-cookbook/examples/commands_keymap_basics.rs`
    - `apps/fret-cookbook/examples/form_basics.rs`
    - `apps/fret-cookbook/examples/async_inbox_basics.rs`
    - `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
    - `apps/fret-cookbook/examples/router_basics.rs`
    - `apps/fret-cookbook/examples/undo_basics.rs`
    - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
    - `apps/fret-cookbook/examples/virtual_list_basics.rs`
    - `apps/fret-cookbook/examples/customv1_basics.rs`
    - `apps/fret-cookbook/examples/embedded_viewport_basics.rs`
    - `apps/fret-cookbook/examples/external_texture_import_basics.rs`
    - `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
    - `apps/fret-cookbook/examples/utility_window_materials_windows.rs`
    - `apps/fret-cookbook/examples/drop_shadow_basics.rs`
    - `apps/fret-cookbook/examples/overlay_basics.rs`
    - `apps/fret-cookbook/examples/toast_basics.rs`
    - `apps/fret-cookbook/examples/query_basics.rs`
    - `apps/fret-cookbook/examples/chart_interactions_basics.rs`
    - `apps/fret-cookbook/examples/docking_basics.rs`
    - `apps/fret-cookbook/examples/hello_counter.rs`
    - `apps/fret-cookbook/examples/hello.rs`
    - `apps/fret-cookbook/examples/date_picker_basics.rs`
    - `apps/fret-cookbook/examples/drag_basics.rs`
    - `apps/fret-cookbook/examples/undo_basics.rs`
    - `apps/fret-cookbook/examples/gizmo_basics.rs`
    - `apps/fretboard/src/scaffold/templates.rs`
    - `apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs`
    - `apps/fret-cookbook/examples/gizmo_basics.rs`
    - `apps/fret-cookbook/examples/theme_switching_basics.rs`
    - `apps/fret-cookbook/examples/simple_todo.rs`
    - `apps/fret-cookbook/examples/drag_basics.rs`
    - `apps/fret-cookbook/examples/effects_layer_basics.rs`
    - `apps/fret-cookbook/examples/text_input_basics.rs`
  - Follow-up polish (same slice, 2026-03-07): late-landing cleanup continued on already-listed `assets_reload_epoch_basics`, `undo_basics`, and `embedded_viewport_basics` to keep builder-first composition consistent after the broader cookbook sweep.
  - Additional 2026-03-07 polish: trimmed remaining builder-path noise in those same pages (button/test_id ordering, alert/header late-landing, and small row composition cleanup) without changing their runtime contracts.
  - Additional 2026-03-07 follow-up: `icons_and_assets_basics` now keeps the page header, card shells, image/svg status rows, and the final content stack on the late-landing builder path, while `customv1_basics` folds `panel_shell(...)`, the preview/body stacks, and the top-level card body into `push_ui()` so only semantics-driven controls and the final scaffold boundary still land eagerly.
  - Additional 2026-03-07 follow-up: `simple_todo`, `assets_reload_epoch_basics`, and `embedded_viewport_basics` now keep their toolbar/header rows, card content stacks, and most panel shells on the builder path too; the remaining eager landings are largely the final scaffold/card boundaries plus semantics- or host-bound nodes (progress/meter badges, viewport surfaces, and inline asset error text).
  - Additional 2026-03-07 follow-up: `undo_basics`, `hello_counter`, and `drop_shadow_basics` now keep most shortcut/action rows, header/content stacks, and staged grid composition on the late-landing path as well; their remaining eager boundaries are mostly semantics-heavy value badges, effect-layer/card footer boundaries, and the final scaffold surface.
  - Additional 2026-03-07 follow-up: `external_texture_import_basics` and `utility_window_materials_windows` now keep their card headers, control/content stacks, and most inline status text/buttons on the builder path too; their remaining eager boundaries are mainly semantics meters, viewport/material root surfaces, and final scaffold/card shells.
  - Additional 2026-03-07 follow-up: `drag_basics`, `query_basics`, and `commands_keymap_basics` now keep their headers, action rows, and card-content stacks mostly on the builder path as well; remaining eager landings are mainly semantics counters, pointer-region/container boundaries, and a few fixed-array panel-body rows that still need concrete `AnyElement` values.
  - Additional 2026-03-07 follow-up: `date_picker_basics` and `router_basics` now keep their card headers, selected/location rows, and card-content stacks on the late-landing path too; `router_basics` no longer has an eager router-outlet leaf cliff after the `fret-router-ui` builder-first outlet path landed, so its remaining eager boundaries are basically the final scaffold/card surface plus the intentional host-side router availability wiring, while `date_picker_basics` still stops at the date-picker/widget host boundary. A 2026-03-08 re-audit confirmed that the picker `test_id` must still land there because the current `CardContent` sink path pushes concrete `AnyElement`s rather than generic `UiIntoElement` builders.
  - Additional 2026-03-07 follow-up: `markdown_and_code_basics` and `effects_layer_basics` now keep their card headers, control rows, preview layout stacks, and card-content composition on the builder path as well; the remaining eager landings are mainly the final scaffold/card surfaces plus the few effect/semantics boundaries that still need concrete `AnyElement` values or post-build role decoration. `customv1_basics` was re-audited in the same pass and its remaining eager nodes are still mostly semantics wrappers, host/effect boundaries, and the final scaffold shell, so it stays as an intentional follow-up rather than churn for churn's sake.
  - Additional 2026-03-07 follow-up: `data_table_basics` and `theme_switching_basics` now keep their card headers, toolbar/content stacks, toggle/sample rows, and inner sample/card composition on the builder path too; both pages are effectively down to the final scaffold/card surface as the only eager landing, while the table widget host boundary remains intentionally concrete inside the slot container.
  - Additional 2026-03-07 follow-up: `gizmo_basics` and `canvas_pan_zoom_basics` now keep their headers, toolbar/hint rows, viewport/canvas content stacks, and card-content composition on the builder path too; the remaining eager landings are now mostly semantics meter badges plus the final card/scaffold surface, which matches the intended host-boundary cutoff for these editor-style demos.
  - Additional 2026-03-07 follow-up: `virtual_list_basics` and `chart_interactions_basics` now keep their mode/toolbar rows, canvas-or-list body stacks, and `CardContent` composition on the builder path too; their remaining eager landings are mostly the few semantics-rich badges, the conditional destructive alert/list row host boundary, and the final card/scaffold surface.
  - Additional 2026-03-07 follow-up: `text_input_basics` and `drag_basics` now keep their button/stats rows, draggable content stack, and `CardContent` composition on the builder path too; the remaining eager landings are mainly semantics progress badges, the drag-region host surface, and the final card/scaffold boundary. `embedded_viewport_basics` was re-audited in the same pass and is intentionally left with only semantics meters plus the final card/scaffold surface as its remaining eager boundaries.
  - Additional 2026-03-08 follow-up: adjacent `apps/fret-examples` surfaces (`todo_demo`, `window_hit_test_probe_demo`, `launcher_utility_window_demo`, and `launcher_utility_window_materials_demo`) now also keep decorate-only progress/style/root semantics or `test_id` patches on the builder path where the surrounding sink already accepts builders; remaining eager landings there are still the intentional pointer-region and raw container host boundaries.
  - Additional 2026-03-08 follow-up: `async_inbox_basics`, `imui_action_basics`, and `utility_window_materials_windows` now keep their progress/root `test_id` patches on the builder path too. `apps/fret-cookbook/src/scaffold.rs` now does the same for the shared page-shell root, so after that sweep the only remaining cookbook-crate `into_element(cx) -> .test_id(...)` holdout is `date_picker_basics`, and that one is intentionally pinned to the current widget-host / `Vec<AnyElement>` sink boundary.
  - Additional 2026-03-07 follow-up: `payload_actions_basics` and `docking_basics` now keep their row/content stacks, toolbar rows, and `CardContent` composition on the builder path too; `payload_actions_basics` now only keeps the final card/scaffold shell eager after the keyed-row helper landed, while docking still intentionally stops at semantics-rich tab badges, cached dock panel roots, and the final card/scaffold surfaces.
  - Additional 2026-03-07 follow-up: `commands_keymap_basics` and `assets_reload_epoch_basics` now keep their shortcut/panel rows, card-content stacks, and inline alert/error composition on the builder path too; the remaining eager landings are now basically just the final outer card in commands and the root/panel return boundaries in assets reload, which are intentional because those helpers currently return concrete `AnyElement`s.
  - Additional 2026-03-07 follow-up: `toast_basics` and `overlay_basics` now keep their action/content rows and card-content composition on the builder path too; the remaining eager landings are primarily the outer card/body return boundaries plus the dialog/toast host surfaces that still need concrete `AnyElement` values at the current API seam.
  - Additional 2026-03-07 follow-up: `external_texture_import_basics` now keeps its meter badges on the builder path via `.a11y(...)`; its remaining eager boundaries are now mainly the viewport host surface and the final scaffold/card shell, which matches the current cookbook helper seam.
  - Additional 2026-03-07 follow-up: `apps/fret-cookbook/src/scaffold.rs` now exposes `centered_page_*_ui(...)` helpers for host-bound builders, so migrated cookbook pages can pass `shadcn::Card::build(...).ui()` surfaces straight into the scaffold without an extra `.into_element(cx)` hop; this removes the final card landing from the current builder-first pages while preserving the scaffold as the intentional last landing seam.
  - Additional 2026-03-07 follow-up: the new `centered_page_*_ui(...)` seam has now been applied across the remaining cookbook card-shell pages too (`async_inbox_basics`, `date_picker_basics`, `drag_basics`, `form_basics`, `hello_counter`, `icons_and_assets_basics`, `query_basics`, `router_basics`, `simple_todo`, `undo_basics`, plus `drop_shadow_basics`' outer shell), so those demos no longer pay an extra card-level `.into_element(cx)` before entering the scaffold helper.
  - Additional 2026-03-07 follow-up: `embedded_viewport_basics` now keeps all six meter badges on the builder path via `.a11y(...)`, while `customv1_basics` now does the same for its status badges and strength controls, and its `panel_shell(...)` helper accepts any `UiChildIntoElement` body so the inline fallback alert no longer lands eagerly. `customv1_basics` also keeps the color swatches on the builder path now; its remaining concrete boundaries are essentially the `preview_content(...)` return seam and the panel shell root, which still land as `AnyElement` at the current host/effect seam.
  - Additional 2026-03-07 follow-up: `canvas_pan_zoom_basics` now keeps all four viewport state badges (`zoom`, `pan.x`, `pan.y`, `node drags`) on the builder path via `.a11y(...)`, so the remaining eager boundaries in that page are no longer the diagnostics meters and stay focused on the canvas/host surfaces themselves.
  - Additional 2026-03-07 follow-up: `icons_and_assets_basics` now keeps its reusable icon/image rows on the builder path too (only the current SVG host branch remains intentionally concrete), and `chart_interactions_basics` now keeps its `x span`, `hover index`, and `selected index` badges on the builder path via `.a11y(...)`, leaving the remaining eager boundaries concentrated in the chart/canvas host seam rather than diagnostics badges.
  - Additional 2026-03-07 follow-up: `imui_action_basics` now keeps its title label and declarative action button on the builder path too by switching the mixed root children back to `ui::children![...]`; the remaining eager landing there is just the final root container boundary.
  - Additional 2026-03-07 follow-up: `docking_basics` now keeps its active-left / active-right diagnostics badges on the builder path via `.a11y(...)`; the remaining eager landing there is primarily the cached dock panel root seam inside `render_cached_panel_root(...)`, which still expects concrete `AnyElement` children today.
  - Additional 2026-03-07 follow-up: `hello_counter` now uses `shadcn::CardFooter::build(...)` together with `centered_page_muted_ui(...)`, so the example is down to zero local `into_element(cx)` calls; the new footer builder also gives other action-row cards a builder-first escape hatch instead of forcing footer children to materialize early.
  - Additional 2026-03-07 follow-up: `virtual_list_basics` now keeps its mode row, control stack, conditional destructive alert, body split, and `CardContent` composition on the builder path too; the final remaining eager landing is the low-level virtual-row `cx.container(...)` seam where the row host still expects concrete `AnyElement` children for per-row border/padding control.
  - Additional 2026-03-07 follow-up: `toast_basics` now keeps its card shell on the builder path and uses `centered_page_muted_ui(...)` before a single final stack landing that installs `Toaster`; `text_input_basics` likewise dropped its remaining badge landings by moving the numeric semantics wiring to `.a11y(...)`. `overlay_basics` was re-audited in the same pass and still stops at the existing dialog trigger/content seams, so it remains a follow-up for future overlay-root builder work rather than cookbook-local churn.
  - Additional 2026-03-07 follow-up: `simple_todo` now keeps its progress badge on the builder path via `.a11y(...)`, and the new `fret-ui-kit::ui::effect_layer(...)` / `effect_layer_build(...)` helper now lets `drop_shadow_basics` and `effects_layer_basics` defer their effect children until the effect root lands. The remaining eager nodes across those pages are now mostly the keyed todo-row root plus the final effect/no-effect branch roots, not pre-effect child materialization.
  - Additional 2026-03-07 follow-up: `router_basics` was re-audited as part of the same sweep; that former outlet-card cliff has since been removed in `fret-router-ui` via `RouterOutlet::into_element_by_leaf_ui(...)`, so any remaining churn there is no longer cookbook-local builder noise but the intentional host-side router availability path.

---

## F. Evidence + Regression Gates

- [x] AFA-gates-050 Add at least one scripted diag repro that exercises:
  - a keybinding → action dispatch,
  - a button click → action dispatch,
  - action availability gating (disabled state) under a modal barrier.
  - Status (as of 2026-03-03):
    - Implemented (non-modal gating): `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json`
    - Implemented (button click + state update): `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json`
    - Implemented (text input submit/clear): `tools/diag-scripts/cookbook/text-input-basics/cookbook-text-input-basics-submit-and-clear.json`
    - Implemented (modal barrier shortcut gating): `tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json`
- [x] AFA-gates-051 Add compile-only wasm smoke gates for the new view runtime surface.
  - Evidence:
    - `tools/gates_wasm_smoke.ps1`
- [x] AFA-gates-052 Add a small set of unit tests for action routing / handler table behavior.
  - Evidence:
    - `crates/fret-ui/src/tree/tests/command_dispatch_source_trace.rs`
- [x] AFA-gates-053 Add a “risk matrix” review pass for M0/M1 (see `RISK_MATRIX.md`).
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/RISK_MATRIX.md` (review pass section)
- [x] AFA-gates-054 Add a small repo-local gate that prevents legacy MVU from drifting back into the cookbook.
  - Evidence:
    - `tools/gate_no_mvu_in_cookbook.py` (or `tools/gate_no_mvu_in_cookbook.ps1`)

---

## G. Cleanup and Deletion (Leave it clean)

This phase is intentionally last.

- [x] AFA-clean-060 Remove legacy MVU routing glue once it is no longer recommended in templates/docs.
  - Status (as of 2026-03-06): completed in-tree; only historical/external migration guidance remains.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
    - `tools/gate_no_mvu_in_tree.py`
    - `tools/gate_no_mvu_in_cookbook.py`

---

## I. Post-v1 boundary + example-surface alignment

These are documentation/surface-area follow-ups, not blockers for the v1 closure.

- [x] AFA-postv1-001 Clarify `ecosystem/fret` crate ownership in docs.
  - Decision: keep `fret` as the golden-path authoring facade; do not turn it into the repo?s
    canonical example host.
  - Evidence:
    - `ecosystem/fret/README.md`
    - `ecosystem/fret/src/lib.rs`
- [x] AFA-postv1-002 Clarify the Bevy comparison for examples.
  - Decision: borrow Bevy-style discoverability via `examples/README.md`, but keep runnable teaching
    surfaces in cookbook/gallery/app-owned crates because the repo root is a workspace, not a package.
  - Evidence:
    - `examples/README.md`
    - `docs/examples/README.md`
    - `docs/workstreams/example-suite-fearless-refactor-v1/design.md`
- [ ] AFA-postv1-003 Decide whether all top-level example links should collapse to one canonical docs
  page while preserving `examples/README.md` as a GitHub portal alias.
- [x] AFA-clean-061 Update docs and templates:
  - `docs/README.md` state management section shows actions + view runtime as the golden path.
  - `fretboard` templates generate action-first demos by default.
  - Status (as of 2026-03-06): `README.md`, `docs/README.md`, `docs/first-hour.md`, `docs/examples/README.md`, `docs/examples/todo-app-golden-path.md`, `docs/fearless-refactoring.md`, `docs/crate-usage-guide.md`, `docs/ui-ergonomics-and-interop.md`, the migration guide, scaffold templates, and the ui-gallery command teaching page align on the narrowed default entrypoints; keep future narrative pages in sync as examples migrate.
  - Evidence:
    - `README.md`
    - `docs/README.md`
    - `docs/first-hour.md`
    - `docs/examples/README.md`
    - `docs/examples/todo-app-golden-path.md`
    - `docs/fearless-refactoring.md`
    - `docs/crate-usage-guide.md`
    - `docs/ui-ergonomics-and-interop.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
    - `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`, `simple_todo_template_main_rs`, `hello_template_main_rs`)
    - `apps/fret-ui-gallery/src/ui/pages/command.rs`
- [x] AFA-clean-062 Delete or quarantine redundant APIs/modules once adoption is complete.
  - Rule: do not delete until all in-tree demos + ecosystem crates have migrated or have explicit ?legacy? labeling.
  - Migration inventory:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
  - Status (as of 2026-03-06): completed in-tree; `ecosystem/fret` MVU modules/feature gate are gone, legacy MVU demo copies are absent, and templates no longer scaffold MVU.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
    - `ecosystem/fret/src/lib.rs`
    - `tools/gate_no_mvu_in_tree.py`
    - `tools/gate_no_mvu_in_cookbook.py`

### Next cleanup steps (post-v1)
- [x] AFA-clean-063 Decide MVU's long-term status (supported alternative vs legacy-only).
  - Decision:
    - Adopted: MVU is not a supported alternative golden path; it has been removed in-tree and only historical/external migration notes remain.
  - Historical note:
    - During v1, the lack of structured payload actions (and view-cache parity risk) was a practical
      reason to keep MVU during the deprecation window. Payload actions v2 (ADR 0312) landed later.
  - Evidence:
    - Policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`
    - Milestone: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md` (M9)
    - Gate: `tools/gate_no_mvu_in_tree.py`

- [x] AFA-clean-064 Add compile-time deprecation warnings for legacy MVU surfaces (if feasible).
  - Status: not needed; the repo reached the M9 hard delete before a separate warning window was implemented in-tree.

- [x] AFA-clean-065 Consider feature-gating MVU behind an explicit legacy feature.
  - Status: not needed; the repo removed MVU in-tree instead of preserving it behind a legacy feature.

---

## Post-v1 follow-ups (tracked separately)

These are intentionally *not* part of the v1 milestone closure, but they are likely the next
practical steps:

Current sequencing note (as of 2026-03-08):

- first: local-state / invalidation ergonomics (`AFA-postv1-001` + `AFA-postv1-004`), with the next concrete target being todo-like view-owned collections,
- second: narrow widget-local action sugar (`AFA-postv1-003`) once the collection/shared-state boundary is clearer,
- third: continue builder-first convergence where it removes real noise but avoid helper sprawl,
- fourth: only after the first three stabilize, re-evaluate narrow macros (`AFA-postv1-005`).

- [~] AFA-postv1-001 Investigate direct local-state ergonomics beyond `Model<T>` in `ViewCx::use_state`.
  - Goal: let simple demos keep state in a plain-Rust shape without weakening dirty/notify semantics
    or shared-model interop.
  - Evidence target: rewrite one medium demo as a comparison branch before promoting any new surface.
  - Update (as of 2026-03-06): additive prototype landed as `LocalState<T>` + `ViewCx::use_local*` / `watch_local(...)`; `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now use the prototype instead of storing explicit local model handles in the view struct, with the query demos validating `use_local` alongside `use_query` / `use_query_async` + transient invalidation.
  - Update (as of 2026-03-07): `TrackedStateExt::{layout, paint, hit_test}` plus `LocalState::watch(cx)` and the first `ViewCx::on_action_notify_local_*` helpers landed as the next additive step. `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now read local state from the handle side and use local-state-specific write helpers for the straightforward set/toggle cases without re-exposing raw model handles. `apps/fret-cookbook/examples/hello_counter.rs` and `apps/fret-cookbook/examples/query_basics.rs` now mirror the same direction for the first medium cookbook samples.
  - Update (as of 2026-03-08): `LocalState::read_in` / `revision_in` now cover the remaining ?generic model-store closure? read path too, so cookbook `hello_counter`, `form_basics`, `text_input_basics`, `simple_todo`, `virtual_list_basics`, the `fretboard` simple-todo template, and `hello_counter_demo` no longer need to leak `local.model()` just to read or revision-check local state inside `on_action_notify_models` or derived revision code.
  - Update (as of 2026-03-08, query handle follow-up): query handle-side reads now stay handle-first across both `ViewCx` and `ElementContext` authoring surfaces. `TrackedStateExt` covers `QueryHandle<T>` in the `ViewCx` path, while `fret-ui-kit::declarative::QueryHandleWatchExt` covers `ElementContext` surfaces behind `state-query`, so cookbook `query_basics`, `fret-examples` `query_demo` / `query_async_tokio_demo` / `async_playground_demo` / `markdown_demo`, the scaffold query-tip template, `docs/examples/todo-app-golden-path.md`, `docs/integrating-tokio-and-reqwest.md`, `docs/workstreams/imui-state-integration-v1.md`, and `fret-markdown`'s MathJax/Mermaid helpers can all read query state from the handle side via `handle.layout(cx).value_*` / `handle.layout_query(cx).value_*` instead of reopening `handle.model()`.
  - Update (as of 2026-03-08, todo comparison target): `apps/fret-cookbook/examples/simple_todo_v2_target.rs` now keeps a keyed todo list in `LocalState<Vec<TodoRow>>` and uses payload actions for per-row toggle/remove, proving that the current runtime can already express small view-owned dynamic collections without `Model<Vec<_>>`.
  - Update (as of 2026-03-08, checkbox source alignment): after comparing against `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx` and its checkbox examples, `ecosystem/fret-ui-shadcn/src/checkbox.rs` now supports a shadcn-style checked snapshot path (`Checkbox::from_checked(...)`) plus `action(...)` / `action_payload(...)`. That removes the need for per-row checkbox models in the todo comparison target; the remaining visible noise shifts more clearly to root-level handler registration and widget-local listener/dispatch ergonomics.
- [~] AFA-postv1-002 Investigate builder-first composition paths that reduce `ui::children!` and nested
  `into_element(cx)` in medium demos.
  - Goal: measure whether a builder-only path materially improves density without helper sprawl.
  - Evidence target: compare `hello_counter_demo` or `query_demo` against the current default path.
  - Update (as of 2026-03-06): `fret-ui-kit::ui::UiElementSinkExt`, `UiChildIntoElement`, and `ui::*_build` sinks now power builder-first `query_demo` and `query_async_tokio_demo` variants while also letting `ui::children!` / `push_ui()` accept nested layout builders plus host-bound `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)` values without an extra `.into_element(cx)` cliff. That same card-builder path now also covers the `fretboard` todo/simple-todo templates plus `commands_keymap_basics`, `form_basics`, and `async_inbox_basics` through the generic `.ui()` patch path; `ecosystem/fret-ui-shadcn/src/layout.rs` now exposes `container_vstack_build(...)` / `container_hstack_build(...)` / `container_hstack_centered_build(...)` so the first older helper family can stay on the same late-landing pipeline; `ecosystem/fret-ui-shadcn/src/table.rs` plus `ecosystem/fret-genui-shadcn/src/resolver/data.rs` now extend that same pattern into the table composite stack (`Table::build(...)` / `TableHeader::build(...)` / `TableBody::build(...)` / `TableFooter::build(...)` / `TableRow::build(...)`) for GenUI-driven data tables; `TableCell::build(child)` now serves as the first single-child late-landing sample (also reflected in the UI Gallery typography table snippet); `DialogTrigger::build(...)` / `SheetTrigger::build(...)` / `DrawerTrigger::build(...)` now bring the first overlay-trigger wrappers onto the same child pipeline for sink-based / direct late-landing paths and the `Dialog` / `Sheet` composition builders accept those `*_Trigger::build(...)` values directly; the wider overlay single-child family now follows the same shape too (`PopoverTrigger::build(...)`, `PopoverAnchor::build(...)`, `HoverCardTrigger::build(...)`, `HoverCardAnchor::build(...)`, `TooltipTrigger::build(...)`, `TooltipAnchor::build(...)`); `Popover::build(...)` now removes the next popover root landing cliff while letting `PopoverContent::test_id(...)` stay on the late-landing path; `DropdownMenuTrigger::build(...)` plus `DropdownMenu::build(...)` / `DropdownMenu::build_parts(...)` now bring the first composite menu root onto that same late-landing path; and `HoverCard::build(...)` / `HoverCard::build_controllable(...)` / `Tooltip::build(...)` keep the same root-level direction, with `Tooltip::new(...)` accepting `TooltipContent` directly. The UI Gallery now teaches the intended overlay paths through `apps/fret-ui-gallery/src/ui/snippets/hover_card/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/align.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/with_form.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/*.rs`, and `apps/fret-ui-gallery/src/ui/snippets/tooltip/demo.rs`. Remaining gap: broader composite APIs beyond the first dropdown-menu path and the remaining eager-only wrappers still sit outside the modern child pipeline.
  - Update (as of 2026-03-07): `Dialog::compose().content_with(...)` / `Sheet::compose().content_with(...)` support deferred content authoring so `DialogClose::from_scope()` / `SheetClose::from_scope()` can be used inside composed content without forcing eager `into_element(cx)` landing.
  - Update (as of 2026-03-07): `CardFooter::build(...)` now extends the same late-landing card-section path to footer action rows, which lets `hello_counter` drop its last footer-level landing cliff; `virtual_list_basics` is correspondingly down to a single low-level `cx.container(...)` row seam, so the next meaningful cleanup would be a higher-level row/container authoring surface rather than more cookbook-local churn.
  - Update (as of 2026-03-07): `toast_basics` and `text_input_basics` now validate that the current builder-first surface is sufficient for a host-bound card shell plus semantics-rich status badges, while the re-audit of `overlay_basics` shows the next real gap more clearly: the dialog recipe family still needs a builder-first content/root path before that example can shed its remaining eager trigger/content seams.
  - Update (as of 2026-03-07): `fret-ui-kit::ui::effect_layer(...)` / `effect_layer_build(...)` now move effect-root child collection onto the same `UiChildIntoElement` pipeline as `container` / `stack`, which lets the renderer demos drop their pre-effect landing cliff. With that seam closed, `simple_todo` and `router_basics` show the next meaningful host-bound gaps more clearly: keyed row helpers and `RouterOutlet` closures still insist on concrete `AnyElement` values, so those are better next targets than more cookbook-local effect churn.
  - Update (as of 2026-03-07): `fret-ui-kit::ui::keyed(...)` now preserves the original keyed callsite across builder-first sink paths by routing through `ElementContext::keyed_at(...)`. `simple_todo`, `payload_actions_basics`, and the generated fretboard todo templates all drop their last keyed-row eager landing cliff, which makes `RouterOutlet` leaf closures the clearer next host-bound seam to tackle.
  - Update (as of 2026-03-07): `fret-router-ui::RouterOutlet::into_element_ui(...)` / `into_element_by_leaf_ui(...)` / `into_element_by_leaf_with_status_ui(...)` now accept builder-first route content via a router-local `RouterOutletIntoElement` adapter backed by `UiChildIntoElement`. `router_basics` correspondingly drops its last outlet-card eager landing cliff, which makes lower-level row/container seams like `virtual_list_basics` and a few remaining overlay host surfaces the clearer next authoring targets.
  - Update (as of 2026-03-07): `fret-ui-kit::ui::container_props(...)` / `container_props_build(...)` now keep caller-specified `ContainerProps` roots on the same late-landing child pipeline as `container` / `stack` / `effect_layer`. `virtual_list_basics` correspondingly drops both its low-level row-root and list-slot eager landing cliffs, so the remaining authoring gaps are no longer cookbook-local raw list containers but the narrower overlay/dialog host seams and other explicit `AnyElement` escape hatches.
  - Update (as of 2026-03-07): `DialogContent::build(...)` / `DialogHeader::build(...)` / `DialogFooter::build(...)` together with their `Sheet*::build(...)` counterparts now keep nested overlay sections on the same late-landing path while still landing at the existing dialog/sheet root boundary. `overlay_basics` correspondingly drops its remaining inner dialog content/header/footer eager landing cliffs, so the next overlay work is more about the still-older alert-dialog/drawer surfaces and other explicit `AnyElement` escape hatches than cookbook-local dialog glue.
  - Update (as of 2026-03-07): `AlertDialogContent::build(...)` / `AlertDialogHeader::build(...)` / `AlertDialogFooter::build(...)` together with their `Drawer*::build(...)` counterparts now keep the remaining shadcn overlay section seams on the same late-landing path while still landing at the existing alert-dialog/drawer root boundary. The UI Gallery `alert_dialog` / `drawer` demos now exercise that builder-first path directly, so the next overlay work is narrower root-level `AnyElement` escape hatches rather than section-local eager landings.
  - Update (as of 2026-03-07): `AlertDialogTrigger::build(...)` plus `AlertDialog::build(cx, trigger, content)` and the new generic `AlertDialog::compose().trigger(...)` / `Drawer::compose().trigger(...)` trigger-arg support now close the older root trigger seam too. The UI Gallery `alert_dialog/demo.rs`, `alert_dialog/parts.rs`, `alert_dialog/usage.rs`, `drawer/demo.rs`, and `drawer/usage.rs` now teach those narrower root helpers directly, so the remaining overlay gaps are mostly around broader root content/part sugar rather than trigger landing.
- [ ] AFA-postv1-003 Investigate widget-local action sugar (`listener` / `dispatch` / `shortcut`)
  without expanding the default helper surface prematurely.
  - Goal: keep action-first semantics while lowering local event-wiring noise.
  - Guardrail: only promote if at least two real demos/templates need the same shape.

- [~] AFA-postv1-004 Evaluate v2 invalidation ergonomics: keep explicit `notify()` as a low-level runtime escape hatch while making local-state writes rerender implicitly by default.
  - Goal: preserve cache/debug determinism without forcing users to call `notify()` after most tracked state writes.
  - Evidence target: prototype one medium demo and confirm diagnostics still explain rebuild reasons.
  - Update (as of 2026-03-06): the prototype keeps explicit `notify()` out of the call site by combining `LocalState::update_in` / `set_in` with the existing `on_action_notify_models` path in `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo`; `LocalState::update_action` / `set_action` remain available for future widget-local experiments once widget-local dispatch ergonomics are revisited.
  - Update (as of 2026-03-07): `ViewCx::on_action_notify_local_update` / `on_action_notify_local_set` / `on_action_notify_toggle_local_bool` now promote the same “tracked local write => redraw + notify” rule into a first-class authoring path. The current medium demos plus cookbook `hello_counter` and `query_basics` use those helpers for the simple local-state mutations, while `commands_keymap_basics` / `text_input_basics` validate command availability and widget interop on `use_local*` / `state.layout(cx).value_*` / `state.paint(cx).value_*`, `form_basics` shows that multi-field validation/reset flows can stay on the generic `on_action_notify_models` path, `simple_todo` demonstrates the first keyed-list hybrid where draft/ID counters move to local state but the dynamic collection itself remains an explicit `Model<Vec<_>>`, `drop_shadow_basics` proves the same local-state bridge on a pure toggle-only renderer demo, `markdown_and_code_basics` extends that bridge to a mixed editor/render-options page built from model-centered `Textarea` / `ToggleGroup` / `Switch` widgets, `assets_reload_epoch_basics` shows the same local-state path on a host/runtime escape-hatch page where the counter is local but the actual asset reload bump plus redraw/RAF scheduling intentionally stay in render-time code, `virtual_list_basics` closes the first virtualization hybrid by moving mode/toggle/jump controls to local state while intentionally keeping the items collection plus scroll/reorder coordination on explicit model/runtime surfaces, `theme_switching_basics` applies the same hybrid rule to theme selection by moving the chosen scheme to local state while keeping theme application plus redraw/RAF sync as render-time host effects, and `icons_and_assets_basics` now does the same for asset demos by moving the reload bump counter to local state while keeping asset reload epoch bump plus redraw/RAF synchronization as render-time host effects, while `customv1_basics` closes the same loop for renderer/effect demos by moving `enabled` / `strength` to local state and intentionally keeping effect registration, capability checks, and effect-layer plumbing render-time/runtime-owned. `notify()` remains a low-level escape hatch rather than a default teaching-surface step, Queue A and Queue B are now cleared, and the teaching-surface inventory treats the remaining explicit-model cookbook cases as intentionally advanced rather than pending default-surface migrations. The new `LocalState::read_in` / `revision_in` helpers keep even those generic `on_action_notify_models` / derived-revision closures on the local-state handle surface, so the remaining local-state pressure is increasingly about write-path policy rather than read-path leakage.
  - Update (as of 2026-03-08, follow-up): cookbook `customv1_basics` now uses `on_action_notify_toggle_local_bool` for its simple `enabled` flag, while `commands_keymap_basics` intentionally stays on the generic `on_action_notify_models` transaction for command availability gating but now reads the gate through `LocalState::read_in(...)` instead of reopening the raw model handle.
  - Update (as of 2026-03-08, tracked-write review): no additional invalidation helper is promoted into the default path for now. `LocalState::update_in` / `set_in` are now explicitly documented as store-only transaction helpers, while `LocalState::update_action` plus `ViewCx::on_action_notify_local_*` remain the first-class `tracked local write => request_redraw + notify` boundary. A focused unit test in `ecosystem/fret/src/view.rs` locks that contract so `notify()` can stay a low-level escape hatch instead of reappearing as a default teaching-surface step.
- [x] AFA-postv1-006 Audit model-centered widget contracts that still leak into gallery/reference surfaces.
  - Goal: separate true widget contract pressure from snippet-level authoring choices before designing new helper APIs.
  - Evidence target: audit note + one snippet cleanup that proves an existing uncontrolled path is sufficient.
  - Status (as of 2026-03-08): `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md` now classifies the remaining pressure into text-value widgets (`Input` / `Textarea`), overlay/disclosure roots that already have uncontrolled paths (`Collapsible`, `Popover`, `Dialog`, `AlertDialog`), and intentional outward-sync surfaces. `apps/fret-ui-gallery/src/ui/snippets/collapsible/demo.rs` now uses `Collapsible::default_open(false)` directly, proving that this specific gallery case was a snippet choice rather than a missing runtime/local-state capability. `ecosystem/fret-ui-shadcn/src/text_value_model.rs` plus the matching `LocalState<String>` impl in `ecosystem/fret/src/view.rs` now land the narrow text bridge, so post-v1 teaching surfaces can call `Input::new(&local_text)` / `Textarea::new(&local_text)` directly without widening the helper surface.

- [x] AFA-postv1-007 Publish a post-v1 gap analysis against the current Rust-first UI best-practice target.
  - Goal: keep docs honest about what is already solved versus what still blocks GPUI/Zed-level authoring density.
  - Evidence target: align the workstream proposal, v2 golden path, and todo golden-path docs on the same ?current baseline vs north-star? statement.
  - Status (as of 2026-03-08): `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`, and `docs/examples/todo-app-golden-path.md` now explicitly record that the narrow text bridge landed and that the clearest remaining gap is small view-owned collection ergonomics (`Model<Vec<_>>` pressure in simple todo-style views), followed by widget-local action sugar and only then a macro re-evaluation.

- [ ] AFA-postv1-008 Decide the next additive API move after the local-collection comparison target.
  - Goal: determine whether the next density win should now come from narrower widget-local action sugar (`listener` / `dispatch`) or from invalidation/write-path ergonomics, without re-expanding the helper surface.
  - Evidence target: remove one of the remaining `simple_todo_v2_target` noise points in a migrated example or short design note.
  - Status note: `apps/fret-cookbook/examples/simple_todo_v2_target.rs` now proves `LocalState<Vec<_>>` is already viable for a small keyed todo list and no longer needs per-row checkbox models either; label/control parity for snapshot/action discrete controls is now closed, so the remaining question is which write/event ergonomics buys the next density win.
  - Follow-up (as of 2026-03-08, skill alignment): the same decision tree is now captured in `.agents/skills/fret-shadcn-source-alignment/references/public-surface-parity.md` and `.agents/skills/fret-app-ui-builder/references/mind-models/mm-widget-state-surfaces.md`, so future `Switch` / `Toggle`-style audits can reuse one parity rubric before adding more app-side helpers.
  - Update (as of 2026-03-08, switch source alignment): `ecosystem/fret-ui-shadcn/src/switch.rs` now mirrors the checkbox-style narrow snapshot path via `Switch::from_checked(...)` plus `action(...)` / `action_payload(...)`, and `apps/fret-cookbook/examples/commands_keymap_basics.rs` now uses that path for both its local allow-command toggle and the disabled panel-open indicator.
  - Update (as of 2026-03-08, toggle source alignment): `ecosystem/fret-ui-shadcn/src/toggle.rs` now lands the same narrow snapshot/action pattern via `Toggle::from_pressed(...)` plus `action(...)` / `action_payload(...)`, and `apps/fret-cookbook/examples/toggle_basics.rs` now demonstrates the path on a minimal view-local example.
  - Update (as of 2026-03-08, action-only control parity): `crates/fret-ui/src/declarative/host_widget/event/pointer_region.rs` plus `ecosystem/fret-ui-kit/src/primitives/control_registry.rs` now let label-forwarded pointer activation record command payload/source metadata, and `Checkbox` / `Switch` / `Toggle` all register command-backed `control_id` entries for snapshot/action paths. That closes the shared discrete-widget parity gap, so the next density decision can focus on widget-local action sugar vs broader invalidation ergonomics rather than on label forwarding.

- [ ] AFA-postv1-005 Evaluate narrow authoring macros that reduce repeated child/list boilerplate without introducing a full `rsx!`-style DSL as the default surface.
  - Goal: decide whether keyed child-list macros or optional layout collection sugar materially improve density after builder-first improvements.
  - Guardrail: no macro should hide action identity, key context, or cache-boundary semantics.
  - Note: this is optional polish, not a prerequisite for declaring v2 successful.

- Done: key context stack + diagnostics-visible context naming/stacking rules.
  - Evidence:
    - ADR: `docs/adr/0022-when-expressions.md` (`keyctx.*`)
    - Runtime: `crates/fret-runtime/src/when_expr/*` (`keyctx.*` evaluation + validation)
    - UI: `crates/fret-ui/src/tree/shortcuts.rs` (collects `key_contexts[*]` from the focused chain / barrier root)
    - Diag protocol: `crates/fret-diag-protocol/src/lib.rs` (`UiShortcutRoutingTraceEntryV1.key_contexts`)
    - Gate: `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json` (`wait_shortcut_routing_trace.query.key_context`)
- Reduce authoring noise (status):
  - Done: attach `SemanticsDecoration`/`test_id`/`key_context` before `into_element(cx)`:
    - Mechanism helpers: `crates/fret-ui/src/element.rs` (`AnyElement::a11y_*`)
    - Ecosystem authoring ext: `ecosystem/fret-ui-kit/src/declarative/semantics.rs`
    - Prelude import fix: `ecosystem/fret-ui-kit/src/lib.rs` (`UiIntoElement` in `prelude::*`)
  - Done: cookbook demos updated to avoid decorate-only early landing:
    - `apps/fret-cookbook/examples/hello.rs`
    - `apps/fret-cookbook/examples/overlay_basics.rs`
    - `apps/fret-cookbook/examples/commands_keymap_basics.rs`
    - `apps/fret-cookbook/examples/hello_counter.rs`
    - `apps/fret-cookbook/examples/date_picker_basics.rs`
    - `apps/fret-cookbook/examples/drag_basics.rs`
    - `apps/fret-cookbook/examples/undo_basics.rs`
    - `apps/fret-cookbook/examples/gizmo_basics.rs`
  - Done: remove redundant outer `cx` arguments from ecosystem authoring constructors (`fret-ui-kit::ui::*`):
    - Implementation: `ecosystem/fret-ui-kit/src/ui.rs` (`h_flex`, `v_flex`, `h_row`, `v_stack`, `container`, `scroll_area`, `text`, `label`, `raw_text`, …)
    - Call-site migration (status):
      - Done: `apps/fret-cookbook`, `apps/fret-examples`
      - In progress: `apps/fret-ui-gallery` (large surface; migrate in batches)
        - Started: `apps/fret-ui-gallery/src/ui/doc_layout.rs`, `apps/fret-ui-gallery/src/ui/content.rs`
        - Default-helper alignment landed for the command docs surface: `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`, `apps/fret-ui-gallery/src/ui/pages/command.rs`
        - Teaching-surface gate now covers ui-gallery pages/snippets for bare `cx.on_action*` regressions: `tools/gate_no_on_action_in_teaching_surfaces.py`
        - Advanced helper exceptions are now locked by allowlist: `tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py`
        - Gate (shell-only): `tools/gate_no_stack_in_ui_gallery_shell.py` (or `tools/gate_no_stack_in_ui_gallery_shell.ps1`)
      - As needed: shadcn/genui crates (only when they block teaching-surface convergence)
  - Done: hard delete legacy stack helpers once internal implementations are migrated.
    - Gate: `tools/gate_no_public_stack_in_ui_kit.py` (or `tools/gate_no_public_stack_in_ui_kit.ps1`)
    - Note: a handful of “host type inference” edge cases need an explicit anchor.
      Preferred: annotate the closure argument type (e.g. `ui::v_flex(|cx: &mut ElementContext<'_, App>| ...)`).
      Alternative: turbofish (e.g. `ui::v_flex::<App, _, _>(...)`).
  - Done: cookbook examples no longer use `stack::hstack/vstack` authoring helpers; the repo teaches
    one layout authoring surface for demos (`fret-ui-kit::ui::*` builders).
    - Gate: `tools/gate_no_stack_in_cookbook.py` (or `tools/gate_no_stack_in_cookbook.ps1`)
  - Done: examples no longer use `stack::hstack/vstack` authoring helpers.
    - Gate: `tools/gate_no_stack_in_examples.py` (or `tools/gate_no_stack_in_examples.ps1`)
- Pointer-triggered explainability: stable selector → action mapping without relying on script stamping.
  - Status (as of 2026-03-03): `debug.command_dispatch_trace[*].source_test_id` is inferred from the
    current semantics snapshot when `source_element` is available (fallbacks remain for cases where
    semantics/test IDs are unavailable).
  - Update (as of 2026-03-04): pointer-triggered `source_test_id` is now recorded directly into the
    pending dispatch source when available (pressable `PressableA11y.test_id`), and diagnostics
    fall back to semantics snapshot correlation only when the direct test ID is unavailable.
  - Evidence:
    - `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs` (`infer_pointer_source_test_id_from_semantics`)
    - `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (`command_dispatch_trace_infers_pointer_source_test_id_from_semantics_snapshot`)
    - `crates/fret-runtime/src/command_dispatch_diagnostics.rs` (`CommandDispatchSourceV1.test_id`)
    - `crates/fret-ui/src/declarative/host_widget/event/pressable.rs` (records pending source with `test_id`)
- View runtime ergonomics: reduce `on_action` handler boilerplate (`request_redraw` + `notify`) without weakening
  determinism or layering (ecosystem-only).
  - Status (as of 2026-03-04): implemented `ViewCx::on_action_notify` + `ViewCx::on_payload_action_notify` sugar.
  - Update (as of 2026-03-06): added `fret-ui-kit` `on_activate*` helpers so pointer/pressable authoring can
    converge on the same “small closure + built-in redraw/notify policy” shape.
  - Evidence:
    - `ecosystem/fret/src/view.rs` (`on_action_notify`, `on_payload_action_notify`)
    - `ecosystem/fret-ui-kit/src/activate.rs` (`on_activate`, `on_activate_request_redraw`, `on_activate_notify`)
    - `apps/fret-cookbook/examples/hello.rs` (uses `on_action_notify`)
    - `apps/fret-examples/src/custom_effect_v2_web_demo.rs` (uses `on_activate_request_redraw`)
- Demo authoring review snapshot (as of 2026-03-08):
  - Simple demo status: `hello_template_main_rs` is now close to the intended golden path (typed actions + `ui::children!` + one model-update helper), and `simple_todo_template_main_rs` now keeps its palette/filter helpers on the same builder-first path without extra template-only `.into_element(cx)` cliffs.
  - Medium demo status: `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now use the `LocalState<T>` prototype for view-local state and the default `value_*` read suffix; `hello_counter_demo` now also keeps its generic step read on `LocalState::read_in(...)`, while the query demos plus cookbook `query_basics` now read query resources from the `QueryHandle<T>` side via `TrackedStateExt`, and the remaining `ElementContext` query teaching surfaces (`async_playground_demo`, `markdown_demo`, scaffold query tip, and the markdown MathJax/Mermaid helpers) now use the same `handle.layout_query(cx)` shape via `QueryHandleWatchExt`. Cookbook `hello_counter`, `query_basics`, `commands_keymap_basics`, `text_input_basics`, `date_picker_basics`, `form_basics`, `simple_todo`, `drop_shadow_basics`, `markdown_and_code_basics`, `assets_reload_epoch_basics`, `virtual_list_basics`, `theme_switching_basics`, `icons_and_assets_basics`, and `customv1_basics` now extend that path into the cookbook surface. `text_input_basics` is now the first direct `Input::new(&LocalState<String>)` teaching surface, while `date_picker_basics` remains the first explicit `local.clone_model()` bridge for a controlled non-text widget API (`DatePicker::new_controllable(...)`). `drop_shadow_basics` is the first pure toggle-only renderer example on the same bridge path, `markdown_and_code_basics` now shows the split in one page (`Textarea` uses the direct text bridge while `ToggleGroup::single` / `Switch` still keep their model-centered contracts), `assets_reload_epoch_basics` is the first explicit local-state + render-time host/runtime escape-hatch example, `virtual_list_basics` is the first virtualization hybrid where the collection and scroll handle stay explicit but the surrounding controls move to local state, `theme_switching_basics` is the first explicit local theme-selection + render-time theme-application example, `icons_and_assets_basics` is the matching asset-demo version where the reload trigger becomes local state but asset reload synchronization stays render-time, `customv1_basics` is the matching renderer/effect hybrid where `enabled` / `strength` move to local state while effect registration, capability checks, and effect-layer plumbing stay explicit, `form_basics` is the first multi-field validation/reset example that keeps coordination on `on_action_notify_models`, and `simple_todo` is the first keyed dynamic-list example to use a hybrid split (`use_local*` for draft / `next_id`, explicit `Model<Vec<_>>` for the collection). `apps/fretboard/src/scaffold/templates.rs` now mirrors that same split in `simple_todo_template_main_rs`, its draft input uses the same direct text bridge, and the query-tip template now reads its query result from the handle side too, while the remaining scaffold todo templates, cookbook `async_inbox_basics`, and `fret-genui-shadcn` data-table resolver continue to carry the current card/table-focused builder-first experiment. The remaining inventory is now classified in `TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`, with Queue A + Queue B cleared, the cookbook/template closure separated from advanced/reference surfaces in `apps/fret-examples` and `apps/fret-ui-gallery`, and the remaining explicit-model cases documented as advanced/runtime-bound. `apps/fret-examples/src/embedded_viewport_demo.rs` has now moved its view-local `size_preset` knob to `use_local_with(...)` + `on_action_notify_local_set(...)`; the demo still remains intentionally advanced because the embedded viewport models, forwarded input state, and render-time interop effects are runtime-bound and not treated as post-v1 default-path blockers. The remaining recurring noise classes are:
    1. explicit tracked-state escape hatches that still need raw `watch(...)`, `observe()`, or `revision()` after the default `state.layout(cx).value_*` / `state.paint(cx).value_*` path landed,
    2. broader composite helpers plus the wider family of single-child wrappers still remain outside the modern `ui::children!` / `push_ui()` pipeline, even though the current card/table builder paths, the first `TableCell::build(child)` sample, and the first dropdown-menu trigger/root builder path now round-trip through the generic `.ui()` patch surface,
    3. explicit transient scheduling for App-only effects (`take_transient_on_action_root` + `with_query_client`).
  - Recommended next phase:
    - use a todo-like view-owned collection as the next comparison target for `use_local*` / invalidation ergonomics,
    - keep `on_action*` / `on_activate*` as the current closure story until that collection/shared-state boundary is better understood,
    - prefer template/doc guidance first for transient/App-effect patterns,
    - re-evaluate widget-local sugar and macros only after one more round of template/demo authoring feedback.
- Post-v1 design review (as of 2026-03-06):
  - v1 is successful at architecture + teaching-surface convergence: action-first dispatch landed,
    `View` / `ViewCx` plus hooks are in tree, the default helper story narrowed, and MVU is hard-deleted
    behind reintroduction gates.
  - The repo has not yet reached the full GPUI/Zed-style authoring density end-state. The remaining
    gaps are intentionally treated as post-v1 ergonomics work, not as unfinished migration closure.
  - Remaining pressure points:
    1. `use_state` still returns `Model<T>` instead of a plain-Rust local-state authoring story.
    2. Default demos now converge on `value_*` for common tracked reads; the remaining pressure is mostly write-path/state-placement ergonomics in explicit-model or host-bound cases.
    3. the query demos, scaffold templates, a first cookbook slice, the GenUI data-table resolver, and the UI Gallery typography table snippet now demonstrate builder-first card/table paths plus the first single-child late-landing sample (`TableCell::build(child)`) on the generic `.ui()` patch path, and those values now flow through `ui::children!` / `push_ui()` as well; the remaining visible `into_element(cx)` boundaries are mostly tied to the rest of the single-child wrapper family and older helper wrappers that still insist on eager `AnyElement` values.
    4. Widget-local `listener` / `dispatch` / `shortcut` sugar is not the default event story yet.
  - Recommendation:
    - close v1 as successful on architecture + migration + default teaching surface,
    - track density/ergonomics work in a separate post-v1 phase,
    - do not add more tiny helpers until another round of template/demo evidence shows repeated pressure.
- Helper visibility policy snapshot (as of 2026-03-06):
  - Default teaching surface: `cx.on_action_notify_models::<A>(|models| ...)`, `cx.on_action_notify_transient::<A>(...)`, and local `on_activate(...)` / `on_activate_notify(...)` only.
  - Advanced/reference surface: raw `cx.on_action(...)` / `cx.on_action_notify(...)`, single-model aliases (`on_action_notify_model_update`, `on_action_notify_model_set`, `on_action_notify_toggle_bool`), payload hooks, and redraw-oriented `on_activate_request_redraw*` helpers.
  - Promotion rule: do not promote additional helpers into README/templates/first-hour docs unless at least two real demos/templates need the same shape and the generic defaults are clearly noisier.
  - Remaining intentional advanced cookbook cases are now explicitly cookbook-only host-side categories: `toast_basics` (imperative Sonner host integration), `router_basics` back/forward (router availability sync), `async_inbox_basics::Start` (dispatcher/inbox scheduling), and `undo_basics::Undo`/`Redo` (history traversal + RAF effect).
  - `fret-examples` and ui-gallery teaching pages/snippets are now on the zero-exception path for raw `cx.on_action_notify::<...>` and single-model helper aliases, while scaffold templates keep equivalent unit-test assertions; `async_playground_demo::ToggleTheme` and the query demos stay on `on_action_notify_models` / `on_action_notify_transient` with render-time side effects where needed, `embedded_viewport_demo` now uses `use_local_with(...)` + `on_action_notify_local_set(...)` for its view-local size preset while keeping viewport interop/render-time effects explicit, and `hello_counter_demo` plus both query demos remain the intentional `use_local` prototypes that still keep the default `on_action_notify_models` action surface for coordinated writes.
- Payload actions (v2+), behind strict determinism + validation rules.
  - See: `docs/adr/0312-payload-actions-v2.md`

### Payload actions v2 (prototype, post-v1)

- [x] AFA-actions-070 Lock the payload actions v2 contract (ADR 0312) and scope constraints.
  - Constraints (prototype):
    - payload is pointer/programmatic-only (no keymap schema changes),
    - payload is transient (window-scoped pending store + TTL),
    - missing payload is safe (recommended: treat as not handled).
  - Evidence:
    - ADR: `docs/adr/0312-payload-actions-v2.md`

- [x] AFA-actions-071 Implement a window-scoped pending payload service (TTL) in `crates/fret-runtime`.
  - Reference: `crates/fret-runtime/src/command_dispatch_diagnostics.rs` (`WindowPendingCommandDispatchSourceService`).
  - Evidence:
    - `crates/fret-runtime/src/action_payload.rs` (pending payload store + TTL)

- [x] AFA-actions-072 Expose an object-safe host API for recording/consuming payloads during action dispatch.
  - Surface: `crates/fret-ui/src/action.rs` (`UiActionHost`).
  - Evidence:
    - `crates/fret-ui/src/action.rs` (`record_pending_action_payload`, `consume_pending_action_payload`)

- [x] AFA-actions-073 Add ecosystem authoring sugar:
  - typed payload action macro (additive; do not break `actions!`),
  - handler table support for payload actions (consume + downcast),
  - pressable helper to dispatch action + payload while preserving `*_if_enabled` gating.
  - Evidence:
    - `ecosystem/fret/src/actions.rs` (`payload_actions!`, payload handler hooks)
    - `ecosystem/fret/src/view.rs` (`ViewCx::on_payload_action`)
    - `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (pressable helper)
    - `ecosystem/fret-ui-shadcn/src/button.rs` (`action_payload*` helpers)

- [x] AFA-actions-074 Migrate at least one in-tree demo from MVU payload routing to payload actions.
  - Evidence:
    - demo compiles and behaves correctly,
    - diagnostics gate can still explain the dispatch decision (and best-effort payload presence).
  - Evidence:
    - `apps/fret-cookbook/examples/payload_actions_basics.rs`
    - `tools/diag-scripts/cookbook/payload-actions-basics/cookbook-payload-actions-basics-remove.json`

- Macro ergonomics (non-breaking, v1.x):
  - Keep `actions!` explicit-ID requirement (stable IDs must not drift with refactors).
  - Consider additive helpers that reduce repetition (e.g. prefix/namespace helpers), but do not
    infer IDs from type paths/module names.

---

## H. Hard delete legacy MVU (M9 closure)

Completed: the repo teaching surfaces (templates + cookbook + examples) have converged on View
runtime + typed actions, and in-tree MVU has been removed. Historical MVU discussion remains only
for external migration guidance and archival context.

Exit target:

- no remaining MVU usage in-tree,
- no MVU-related feature gates or demo-level opt-ins,
- no `fret::legacy::*` module,
- no MVU references in default templates/docs as an available authoring path.

Tasks:

- [x] AFA-m9-001 Migrate remaining non-action-first demos in `apps/fret-examples` to View+actions.
  - Status: completed; `apps/fret-examples` now stays on the view runtime + typed actions surface.
- [x] AFA-m9-002 Delete legacy MVU demo copies once the migrated versions exist (remove `*_legacy.rs` files).
  - Status: completed; the former MVU legacy demo copies are absent from `apps/fret-examples/src`.
- [x] AFA-m9-003 Remove the demo-level MVU opt-in and any routing/printing branches in `apps/fret-demo`.
  - Status: completed; the remaining `node-graph-demos-legacy` feature is unrelated to MVU and stays out of scope for this checklist.
- [x] AFA-m9-004 Remove the `ecosystem/fret` MVU feature gate and delete MVU modules.
  - Status: completed; `ecosystem/fret/src/mvu.rs`, `ecosystem/fret/src/mvu_router.rs`, and `ecosystem/fret/src/legacy.rs` are absent.
- [x] AFA-m9-005 Remove any legacy MVU scaffolding sources from `apps/fretboard/src/scaffold/templates.rs`.
  - Status: completed; only regression assertions remain to keep the golden path honest.
- [x] AFA-m9-006 Update docs to remove MVU as an in-tree authoring path while keeping historical migration notes.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLEANUP_PLAN.md`
- [x] AFA-m9-007 Add a lightweight gate that fails if MVU identifiers reappear (file list + `git grep` is enough).
  - Evidence:
    - `tools/gate_no_mvu_in_tree.py`
    - `tools/gate_no_mvu_in_cookbook.py`
