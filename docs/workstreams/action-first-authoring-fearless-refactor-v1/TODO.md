# Action-First Authoring + View Runtime (Fearless Refactor v1) — TODO

Status: Landed (v1), hardening follow-ups in progress
Last updated: 2026-03-04

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Evidence/gates: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`

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
      - `apps/fret-cookbook/examples/icons_and_assets_basics.rs`
      - `apps/fret-cookbook/examples/effects_layer_basics.rs`
      - `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
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
  - Status (as of 2026-03-03):
    - `cargo run -p fretboard -- new hello` uses View runtime + typed unit actions:
      `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`)
    - `cargo run -p fretboard -- new todo` uses View runtime + typed unit actions + selector/query hooks:
      `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`)
    - `cargo run -p fretboard -- new simple-todo` uses View runtime + typed unit actions:
      `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`)

- [x] AFA-adopt-044 Migrate `embedded_viewport_demo` to the view runtime (keep a legacy opt-in copy).
  - Goal: prove view-runtime authoring composes cleanly with embedded viewport interop:
    - `viewport_input(...)` forwarding,
    - and a custom `record_engine_frame(...)` hook for offscreen engine passes.
  - Why this matters: `UiAppDriver` only supports a single `record_engine_frame` hook; view runtime
    currently uses it for view-cache enablement (v1), while embedded viewport needs it for engine
    recording. The migrated demo should demonstrate the correct composition pattern.
  - Evidence:
    - `apps/fret-examples/src/embedded_viewport_demo.rs` (composed `record_engine_frame`)
    - `apps/fret-examples/src/embedded_viewport_demo_legacy.rs`
    - `apps/fret-demo/src/main.rs` (demo routing + legacy name)
    - `ecosystem/fret/src/interop/embedded_viewport.rs`
    - `ecosystem/fret/src/app_entry.rs`

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
    - `tools/gate_no_mvu_in_cookbook.ps1`

---

## G. Cleanup and Deletion (Leave it clean)

This phase is intentionally last.

- [x] AFA-clean-060 Deprecate legacy routing glue that is no longer recommended in templates/docs.
  - Note: this is a doc-level deprecation in v1 (no compile-time `#[deprecated]` yet).
  - Evidence:
    - `ecosystem/fret/src/lib.rs` (MVU modules labeled legacy + recommendation pointer)
    - `ecosystem/fret/src/mvu.rs` (legacy note + view-cache footgun callout)
    - `ecosystem/fret/src/mvu_router.rs` (legacy note + action-first recommendation)
- [x] AFA-clean-061 Update docs and templates:
  - `docs/README.md` state management section shows actions + view runtime as the golden path.
  - `fretboard` templates generate action-first demos by default.
  - Evidence:
    - `docs/README.md`
    - `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`, `simple_todo_template_main_rs`, `hello_template_main_rs`)
- [x] AFA-clean-062 Delete or quarantine redundant APIs/modules once adoption is complete.
  - Rule: do not delete until all in-tree demos + ecosystem crates have migrated or have explicit “legacy” labeling.
  - Migration inventory:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`
  - Status (as of 2026-03-03):
    - Quarantined: `fret::prelude::*` no longer re-exports MVU authoring types (use `fret::legacy::prelude::*` for MVU demos).
    - Gated: cookbook does not contain MVU usage (`pwsh tools/gate_no_mvu_in_cookbook.ps1`).
  - Evidence:
    - `ecosystem/fret/src/lib.rs` (prelude quarantine)
    - `ecosystem/fret/src/legacy.rs` (explicit legacy prelude)
    - `apps/fret-examples/src/todo_demo.rs` (example: legacy prelude import)
    - `tools/gate_no_mvu_in_cookbook.ps1`

### Next cleanup steps (post-v1)

- [x] AFA-clean-063 Decide MVU’s long-term status (supported alternative vs legacy-only).
  - Decision inputs:
    - Payload/parameterized actions are not supported in typed actions v1 (ADR 0307 v1 scope),
      which remains a practical reason to keep MVU for some demos/apps.
    - Per-frame message routing must not regress view-cache reuse semantics.
  - Exit criteria:
    - A single, accurate “when to use MVU” policy exists and is reflected in docs/templates.
  - Decision:
    - MVU is legacy-only (compat), not a supported alternative golden path.
  - Evidence:
    - Policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`
    - Guidance: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md` (“When to use MVU vs View”)

- [x] AFA-clean-064 Add compile-time deprecation warnings for legacy MVU surfaces (if feasible).
  - Rule: docs/templates must stop teaching it *before* adding warnings.
  - Candidate surfaces:
    - `ecosystem/fret/src/mvu.rs`
    - `ecosystem/fret/src/mvu_router.rs`
  - Evidence:
    - `ecosystem/fret/src/lib.rs` (deprecated `legacy`/`mvu`/`mvu_router` modules)
    - `ecosystem/fret/src/app_entry.rs` (deprecated MVU entry points when `legacy-mvu` is enabled)
    - `ecosystem/fret/src/interop/embedded_viewport.rs` (deprecated MVU embedding helpers when `legacy-mvu` is enabled)

- [x] AFA-clean-065 Consider feature-gating MVU behind an explicit legacy feature.
  - Goal: keep `fret::prelude::*` boring, and make MVU opt-in in downstream apps.
  - Non-goal: break existing users without a deprecation window.
  - Evidence:
    - `ecosystem/fret/Cargo.toml` (`legacy-mvu` feature)
    - `ecosystem/fret/src/lib.rs` (MVU modules gated behind `legacy-mvu`)
    - `apps/fret-examples/Cargo.toml` (in-tree demo opts in via `legacy-mvu`)
    - `apps/fret-ui-gallery/Cargo.toml` (in-tree demo opts in via `legacy-mvu`)

---

## Post-v1 follow-ups (tracked separately)

These are intentionally *not* part of the v1 milestone closure, but they are likely the next
practical steps:

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
  - Evidence:
    - `ecosystem/fret/src/view.rs` (`on_action_notify`, `on_payload_action_notify`)
    - `apps/fret-cookbook/examples/hello.rs` (uses `on_action_notify`)
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

## H. Hard delete legacy MVU (M9 follow-up)

This workstream landed v1 with MVU quarantined and opt-in. If the repo goal becomes “fully
migrated, then delete”, track the remaining steps here.

Exit target:

- no remaining MVU usage in-tree,
- no `legacy-mvu` or `legacy-mvu-demos` features,
- no `fret::legacy::*` module,
- no MVU references in templates/docs.

Tasks:

- [x] AFA-m9-001 Migrate remaining non-action-first demos in `apps/fret-examples` to View+actions.
- [ ] AFA-m9-002 Delete legacy MVU demo copies once the migrated versions exist (remove `*_legacy.rs` files):
  - `apps/fret-examples/src/todo_demo_legacy.rs`
  - `apps/fret-examples/src/query_demo_legacy.rs`
  - `apps/fret-examples/src/query_async_tokio_demo_legacy.rs`
  - `apps/fret-examples/src/hello_counter_demo_legacy.rs`
  - `apps/fret-examples/src/async_playground_demo_legacy.rs`
  - `apps/fret-examples/src/embedded_viewport_demo_legacy.rs`
  - `apps/fret-examples/src/drop_shadow_demo_legacy.rs`
  - `apps/fret-examples/src/postprocess_theme_demo_legacy.rs`
- [ ] AFA-m9-003 Remove `apps/fret-examples` feature `legacy-mvu-demos` and any routing/printing branches in `apps/fret-demo`.
- [ ] AFA-m9-004 Remove `ecosystem/fret` feature `legacy-mvu` and delete MVU modules:
  - `ecosystem/fret/src/mvu.rs`
  - `ecosystem/fret/src/mvu_router.rs`
  - `ecosystem/fret/src/legacy.rs`
- [ ] AFA-m9-005 Remove any legacy MVU scaffolding sources from `apps/fretboard/src/scaffold/templates.rs`.
- [ ] AFA-m9-006 Update docs to remove MVU guidance:
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MVU_POLICY.md`
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLEANUP_PLAN.md`
- [ ] AFA-m9-007 Add a lightweight gate that fails if MVU identifiers reappear (file list + `git grep` is enough).
