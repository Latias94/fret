# Action-First Authoring + View Runtime (Fearless Refactor v1) — Evidence and Gates

Last updated: 2026-03-08

This file defines what “done” means beyond subjective UX feel.

The guiding principle: if we change the authoring/routing substrate, we must lock outcomes with
small, deterministic gates (tests and scripted diagnostics), not just manual QA.

---

## 1) Required evidence anchors (update as code lands)

- ADR (actions): `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- ADR (view runtime): `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- Workstream: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`

### Implementation anchors (as of 2026-03-05)

Action identity + typed unit actions:

- `crates/fret-runtime/src/action.rs` (`ActionId`, `TypedAction`)
- `ecosystem/fret/src/actions.rs` (`fret::actions!` macro, `ActionHandlerTable` + unit test)

View runtime (v1):

- `ecosystem/fret/src/view.rs` (`View`, `ViewCx`, `use_state`/`use_state_keyed`/`use_selector`/`use_query`, view-cache reuse + handler keepalive)
- `ecosystem/fret/src/app_entry.rs` (`App::run_view`)
- `ecosystem/fret-ui-kit/src/activate.rs` (`on_activate_*` helpers for low-noise pointer activation handlers)
- `ecosystem/fret-ui-kit/src/primitives/menu/checkbox_item.rs` / `ecosystem/fret-ui-kit/src/primitives/menu/radio_group.rs` / `ecosystem/fret-ui-kit/src/primitives/menu/sub_trigger.rs` (internal primitives reuse `on_activate` helpers)
- `ecosystem/fret-ui-kit/src/imui.rs` (imui pressable activation paths reuse `on_activate` / `on_activate_notify` helpers)
- `ecosystem/fret-ui-kit/src/primitives/navigation_menu.rs` / `ecosystem/fret-ui-kit/src/window_overlays/render.rs` (navigation and overlay pressables reuse `on_activate` helpers)

Legacy MVU removal (M9 landed):

- In-tree MVU code surfaces are removed.
- Historical MVU discussion remains only in migration/archive docs.
- Gates: `tools/gate_no_mvu_in_tree.py` and `tools/gate_no_mvu_in_cookbook.py` prevent reintroduction.

UI gallery adoption (v1):

- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` (action-first `.action(...)` + `cx.on_action_notify_models::<...>(...)` via the view runtime)
- `apps/fret-ui-gallery/src/ui/pages/command.rs` (wiring as a `DocSection` + code extraction region)

Fretboard scaffolding templates (teaching surface):

- `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`, `todo_template_main_rs`, `simple_todo_template_main_rs`)
  - Unit tests gate that templates use `ui::children![cx; ...]`, keep explicit `.into_element(cx)` calls low, and keep first-contact templates on `on_action_notify_models` instead of single-model aliases.
- `README.md`, `docs/README.md`, `docs/first-hour.md`, `docs/examples/README.md`, `docs/examples/todo-app-golden-path.md`, `docs/fearless-refactoring.md`, `docs/crate-usage-guide.md`, `docs/ui-ergonomics-and-interop.md`, `apps/fret-ui-gallery/src/ui/pages/command.rs`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/MIGRATION_GUIDE.md`
  - These first-contact, golden-path, and ergonomics narrative surfaces now align on the same default entrypoints: `on_action_notify_models`,
    `on_action_notify_transient`, and local `on_activate*`; advanced helpers are documented as
    cookbook/reference-only host-side categories.
- `apps/fret-examples/src/hello_counter_demo.rs`, `apps/fret-examples/src/embedded_viewport_demo.rs`, `apps/fret-examples/src/query_demo.rs`, `apps/fret-examples/src/query_async_tokio_demo.rs`, `tools/gate_no_single_model_action_helpers_in_default_teaching_surfaces.py`
  - Keeps the default demo surfaces on `on_action_notify_models` / `on_action_notify_transient`, while `embedded_viewport_demo` now uses local-state-specific write helpers only for its view-local size preset and still keeps runtime/interop effects explicit; `hello_counter_demo` now also uses the direct text-value bridge for its step input, and the gate continues preventing single-model helper aliases from drifting back into `fret-examples` or ui-gallery teaching pages/snippets, while scaffold templates keep equivalent unit-test assertions in `apps/fretboard/src/scaffold/templates.rs`.
- `ecosystem/fret-ui-shadcn/src/text_value_model.rs`, `ecosystem/fret/src/view.rs`, `apps/fret-cookbook/examples/text_input_basics.rs`, `apps/fret-cookbook/examples/markdown_and_code_basics.rs`, `apps/fret-cookbook/examples/virtual_list_basics.rs`, `apps/fretboard/src/scaffold/templates.rs`
  - Narrow text-value bridge evidence: `Input` / `Textarea` stay model-backed internally, but post-v1 `LocalState<String>` views now pass local text state directly without reopening `clone_model()` at the teaching surface.

Editor-grade adoption (workspace shell demo):

- `ecosystem/fret-workspace/src/commands.rs` (`act::*` typed unit actions for workspace command IDs)
- `ecosystem/fret-workspace/src/tab_strip/mod.rs` (tab pressable uses `pressable_dispatch_action_if_enabled*` for activation)
- `ecosystem/fret-workspace/src/tab_strip/state.rs` (tab strip state keeps a one-shot “reveal active tab into view” request to stabilize first-interaction hit targets)
- `ecosystem/fret-workspace/src/tab_strip/widgets.rs` (tab close button uses `pressable_dispatch_action_if_enabled*`)
- `ecosystem/fret-workspace/src/tab_strip/interaction.rs` (middle/right click behaviors record pending dispatch source)
- `ecosystem/fret-workspace/src/command_scope.rs` (workspace-level command scope applies model commands even when UI hooks are not idempotent; requests redraw when applied)
- `apps/fret-examples/src/workspace_shell_demo.rs` (applies workspace model commands first, then dispatches UI hooks; records a driver-handled dispatch decision when UI hooks are non-idempotent so scripted gates still observe `handled=true`)
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json` (gates `source_kind=pointer` for `workspace.tab.close.doc-a-0`)
- `tools/diag_gate_action_first_authoring_v1.ps1` (includes the workspace shell demo gate)

View/cache observability (diagnostics):

- `ecosystem/fret-bootstrap/src/ui_diagnostics/invalidation_diagnostics.rs` (`dirty_views`, `notify_requests`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs` (`cache_roots[*].reuse_reason`)

Teaching-surface ergonomics gates:

- `tools/gate_no_models_mut_in_action_handlers.py` (guards cookbook/examples against regressing to verbose
  `move |host, _acx| host.models_mut()...` patterns; prefers `ViewCx` helpers instead).
- `tools/gate_no_on_action_in_teaching_surfaces.py` (guards cookbook/examples plus ui-gallery
  teaching pages/snippets against regressing to bare `cx.on_action` handlers; prefers
  `ViewCx::on_action_notify*` helpers).
- `tools/gate_only_allowed_on_action_notify_in_teaching_surfaces.py` (locks the remaining
  intentional advanced `cx.on_action_notify::<...>` teaching-surface exceptions to a small,
  reasoned cookbook allowlist, while also keeping `fret-examples` and ui-gallery pages/snippets
  at zero advanced `on_action_notify` occurrences: imperative Sonner host integration, router
  availability sync, dispatcher/inbox scheduling, and undo/redo RAF effects).
- `tools/pre_release.ps1` runs the teaching-surface gates as part of the pre-release policy suite.

Examples adoption (authoring-noise reduction):

- `apps/fret-examples/src/custom_effect_v2_web_demo.rs` (reset button uses `on_activate_request_redraw`)
- `apps/fret-examples/src/custom_effect_v2_identity_web_demo.rs` (reset button uses `on_activate_request_redraw`)
- `apps/fret-examples/src/custom_effect_v2_glass_chrome_web_demo.rs` (reset button uses `on_activate_request_redraw`)
- `apps/fret-examples/src/custom_effect_v2_lut_web_demo.rs` (reset button uses `on_activate_request_redraw`)
- `apps/fret-examples/src/imui_floating_windows_demo.rs` (pressable overlap target uses `on_activate_notify`)
- `apps/fret-examples/src/query_demo.rs` (current guidance sample: top-of-render model reads + transient App-effect scheduling)
- `apps/fret-examples/src/hello_counter_demo.rs` (current guidance sample: action helper placement + card/layout subtree boundaries)
- `apps/fret-examples/src/query_async_tokio_demo.rs` (current guidance sample: async query variant using the same transient + subtree-boundary patterns)
- `apps/fret-examples/src/custom_effect_v1_demo.rs` (reset action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/custom_effect_v2_demo.rs` (reset action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/custom_effect_v3_demo.rs` (reset action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/postprocess_theme_demo.rs` (reset action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/liquid_glass_demo.rs` (reset/preset/toggle-inspector actions now use the default `on_action_notify_models` transaction path)
- `apps/fret-examples/src/async_playground_demo.rs` (`ToggleTheme` now uses `on_action_notify_models`; theme application is synchronized in `render()` from the observed model state)
- `apps/fret-cookbook/examples/icons_and_assets_basics.rs` (reload bump action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs` (reload bump action now uses the default `on_action_notify_models` transaction path)
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` (command toggle handler now uses the default `on_action_notify_models` transaction path while availability stays explicit)
- `apps/fret-cookbook/examples/toast_basics.rs` (intentional advanced reference case: imperative Sonner host integration still needs `UiActionHost` + window)
- `apps/fret-cookbook/examples/router_basics.rs` (`ClearIntents` now uses the default `on_action_notify_models` transaction path; back/forward remain advanced because they also sync router command availability)
- `apps/fret-cookbook/examples/async_inbox_basics.rs` (`Cancel` now uses the default `on_action_notify_models` transaction path; `Start` remains advanced because it spawns dispatcher/inbox work)
- `apps/fret-cookbook/examples/undo_basics.rs` (`Inc`/`Dec`/`Reset` now use the default `on_action_notify_models` transaction path; `Undo`/`Redo` remain advanced because they combine history traversal with RAF scheduling)

Pointer-trigger authoring integration (v1 still dispatches through the command pipeline):

- `crates/fret-ui/src/tree/commands.rs` (command dispatch bubbles from focus when available; otherwise uses pending source element metadata to start bubbling without requiring focus-steal; falls back from overlay roots to the window default root)
- `crates/fret-ui/src/tree/tests/command_availability.rs` (cross-layer fallback tests)
- `crates/fret-runtime/src/command_dispatch_diagnostics.rs` (`CommandDispatchSourceV1.test_id` carries stable selector metadata through the pending-source service)
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs` (records pending source `test_id` from `PressableA11y.test_id` when available)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs` (uses direct `decision.source.test_id` when present; falls back to inferring `source_test_id` from the current semantics snapshot; retains script/hit-test fallbacks)
- `ecosystem/fret-bootstrap/src/ui_diagnostics.rs` (unit test: `command_dispatch_trace_infers_pointer_source_test_id_from_semantics_snapshot`)
- `ecosystem/fret-ui-shadcn/src/button.rs` (`Button::action`)
- `ecosystem/fret-ui-kit/src/command.rs` (`action_is_enabled`, `dispatch_action_if_enabled`)
- `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (`pressable_dispatch_action_if_enabled`)
- `ecosystem/fret-ui-kit/src/imui.rs` (`ImUiFacade::action_button_ex`, `ImUiFacade::menu_item_action_ex`, `UiWriterImUiFacadeExt::menu_item_action_ex`)
- `ecosystem/fret-genui-core/src/executor.rs` (`GenUiActionExecutorV1::with_dispatch_command_actions`)
- `ecosystem/fret-genui-core/src/render.rs` (default `test_id` stamping: `genui:{element_key}`)
- `ecosystem/fret-genui-shadcn/src/resolver/basic.rs` (Button action-first mapping for unit `.v1` action ids)
- `ecosystem/fret-genui-shadcn/src/resolver/overlay.rs` (DropdownMenu item action-first mapping for unit `.v1` action ids + `testId` support)
- `ecosystem/fret-ui-kit/src/declarative/cached_subtree.rs` (`CachedSubtreeExt` authoring helper)
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` (example adoption: view runtime + keymap + action availability gating)
- `apps/fret-cookbook/examples/hello.rs` (example adoption: view runtime + action-first button + handler registration)
- `apps/fret-cookbook/examples/text_input_basics.rs` (example adoption: view runtime + action-first submit/clear actions + command-backed Enter/Escape)
- `apps/fret-cookbook/examples/imui_action_basics.rs` (example adoption: shared action handler across declarative + GenUI + imui)
- `apps/fret-cookbook/examples/overlay_basics.rs` (example adoption: view runtime + modal barrier gate)
- Cookbook migration inventory (tracks remaining legacy MVU usage):
  - `docs/workstreams/action-first-authoring-fearless-refactor-v1/LEGACY_MVU_INVENTORY.md`

Command palette integration (must dispatch through the same pipeline):

- `crates/fret-app/src/core_commands.rs` (`app.command_palette` command metadata + default keybinding)
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (command palette overlay + selection dispatch via `UiTree::dispatch_command`)
- `ecosystem/fret-ui-shadcn/src/command.rs` (command palette item selection queues a pending command and dispatches via `Effect::Command` after close-on-select completes)

Keymap/availability explainability (diagnostics traces):

- `crates/fret-runtime/src/shortcut_routing_diagnostics.rs` (`WindowShortcutRoutingDiagnosticsStore`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs` (`record_shortcut_routing_trace_for_window`, `UiShortcutRoutingTraceEntryV1`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/command_gating_trace.rs` (`debug.command_gating_trace[*]`)
- `crates/fret-ui/src/tree/shortcuts.rs` (widget-scoped shortcut gating uses live UI-tree availability to avoid stale `command_disabled` after modal barrier transitions)

Key-context stack + `keyctx.*` evaluation (v1):

- `crates/fret-runtime/src/when_expr/*` (`WhenEvalContext`, `keyctx.*`)
- `crates/fret-ui/src/element.rs` + `crates/fret-ui/src/elements/cx.rs` (`AnyElement::key_context`, `UiBuilder::key_context`)
- `crates/fret-ui/src/tree/shortcuts.rs` (derives the key-context stack from the focused chain / modal barrier root)
- `crates/fret-ui/src/tree/dispatch/window.rs` (publishes window key-context snapshots to `WindowKeyContextStackService`)
- `crates/fret-runtime/src/window_key_context_stack.rs` (`WindowKeyContextStackService` data-only seam)
- Scripted gate: `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json` (shortcut routing trace includes `key_contexts`)

Dispatch path explainability (diagnostics traces):

- `crates/fret-runtime/src/command_dispatch_diagnostics.rs` (`WindowCommandDispatchDiagnosticsStore`, `WindowPendingCommandDispatchSourceService`)
- `crates/fret-ui/src/action.rs` (`UiActionHost::record_pending_command_dispatch_source` default hook + adapter implementation)
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs` (pressable activation host forwards pending command-dispatch source metadata so pointer/keyboard triggers are explainable)
- `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (records pointer activation → pending dispatch source)
- `crates/fret-ui/src/tree/shortcuts.rs` (records shortcut routing → pending dispatch source)
- `crates/fret-ui/src/tree/commands.rs` (records dispatch outcome + handled-by element)
- `crates/fret-ui/src/tree/tests/command_dispatch_source_trace.rs` (unit tests: pending pointer source consumption + programmatic default)
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (records driver-handled dispatch outcomes to the same trace store, including handler scope classification)
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (flushes `Effect::Command` after script-injected input so scripted `wait_command_dispatch_trace` does not depend on runner effect timing)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_types.rs` (`debug.command_dispatch_trace[*]`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs` (`handle_wait_command_dispatch_trace_step` gate runner used by scripted diagnostics)
- `crates/fret-diag-protocol/src/lib.rs` (`UiActionStepV2::WaitCommandDispatchTrace`, `UiCommandDispatchTraceQueryV1`)
- `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json` (`wait_command_dispatch_trace` gate)
- `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json` (gates `source_test_id` for pointer-triggered dispatch trace)

---

## 2) Regression gates (required)

### 2.1 Unit tests (fast)

Requirements:

- At least one unit/integration test covers:
  - keybinding resolution → action dispatch,
  - action availability gating,
  - dispatch path behavior across focus/root changes.

Preferred location:

- mechanism tests near input dispatch / command routing, plus one ecosystem integration test.

### 2.2 Scripted diagnostics (interaction-level)

Requirements:

- At least one scripted diag scenario (ADR 0159) that:
  - clicks a button that dispatches an action,
  - triggers a keybinding that dispatches an action,
  - asserts availability/disabled state under a modal barrier or focus scope.
  - gates command dispatch explainability (a command dispatch trace entry exists and records the source kind and handler classification).

Notes:

- Tests must rely on stable selectors (`test_id`/role/name), not pixel coordinates.
- The script output must record the resolved `ActionId` (or command/action identity) for each step.

Current scripts (as of 2026-03-04):

- `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json` (also gates key-context stack visibility via `wait_shortcut_routing_trace.query.key_context`)
- `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json` (clicks the button via `role_and_name`, but still gates `source_test_id` attribution)
- `tools/diag-scripts/cookbook/hello/cookbook-hello-view-cache-reuse-and-handler-keepalive.json`
- `tools/diag-scripts/cookbook/payload-actions-basics/cookbook-payload-actions-basics-remove.json` (parameterized action dispatch: pointer + payload, asserts row removal)
- `tools/diag-scripts/cookbook/text-input-basics/cookbook-text-input-basics-submit-and-clear.json`
- `tools/diag-scripts/cookbook/simple-todo/cookbook-simple-todo-smoke.json`
- `tools/diag-scripts/cookbook/virtual-list-basics/cookbook-virtual-list-basics-smoke.json`
- `tools/diag-scripts/cookbook/icons-and-assets-basics/cookbook-icons-and-assets-basics-smoke.json`
- `tools/diag-scripts/cookbook/effects-layer-basics/cookbook-effects-layer-basics-screenshots.json`
- `tools/diag-scripts/cookbook/markdown-and-code-basics/cookbook-markdown-and-code-basics-smoke.json`
- `tools/diag-scripts/cookbook/canvas-pan-zoom-basics/cookbook-canvas-pan-zoom-basics-smoke.json`
- `tools/diag-scripts/cookbook/undo-basics/cookbook-undo-basics-smoke.json`
- `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json`
- `tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json`

Notes:

- The cross-frontend script gates both widget-handled and driver-handled dispatch trace entries:
  - `app.command_palette` must record `handled_by_driver=true` when the palette is opened via shortcut,
  - `cookbook.imui_action_basics.inc.v1` must record pointer-triggered widget handling across frontends.

Gate runner:

- `pwsh tools/diag_gate_action_first_authoring_v1.ps1` (default output: `target/dfa-v1/`; runs under fixed frame delta via `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`; keeps output paths short to avoid Windows path-length issues during schema2 bundle dumps)
  - Note: the gate script builds cookbook examples with `--features cookbook-diag` (and enables per-example feature bundles such as `cookbook-imui` when required) so the launched demos expose the diagnostics transport.
  - Cold builds / cold GPU shader caches can occasionally cause timeouts. The gate script retries timed-out runs once with a larger timeout by default.
    - Defaults: `-TimeoutMs 180000 -TimeoutMsRetry 600000 -TimeoutRetryCount 1`
    - Manual override (if needed): `pwsh tools/diag_gate_action_first_authoring_v1.ps1 -TimeoutMs 600000 -TimeoutRetryCount 0`

### 2.3 wasm smoke (build-only)

Requirements:

- The new view runtime surface compiles for wasm:
  - `cargo check` on `wasm32-unknown-unknown` for the relevant crates.

Rationale:

- action/view APIs must remain portable and must not leak desktop-only types.

---

## 3) Suggested commands (maintainer/dev loop)

Prefer `cargo nextest run` when available.

- Format:
  - `cargo fmt`
- Focused tests:
  - `cargo nextest run -p fret-ui -p fret -p fret-ui-kit -p fret-imui -p fret-genui-core`
- Focused clippy (once surfaces land):
  - `cargo clippy -p fret-ui -p fret -p fret-ui-kit --all-targets -- -D warnings`
- wasm smoke:
  - `cargo check -p fret -p fret-ui-kit --target wasm32-unknown-unknown`
  - `tools/gates_wasm_smoke.ps1`
- Run the Action-first authoring diagnostics gate set (commands/keymap + modal barrier + cross-frontend):
  - `pwsh tools/diag_gate_action_first_authoring_v1.ps1`
- Prevent legacy MVU drift in-tree (compile-time grep gates):
  - `python tools/gate_no_mvu_in_tree.py`
  - `python tools/gate_no_mvu_in_cookbook.py` (or `pwsh tools/gate_no_mvu_in_cookbook.ps1`)
- Prevent `stack::*` authoring drift (cookbook/examples stay on `fret-ui-kit::ui::*` builders):
  - `python tools/gate_no_stack_in_cookbook.py` (or `pwsh tools/gate_no_stack_in_cookbook.ps1`)
  - `python tools/gate_no_stack_in_examples.py` (or `pwsh tools/gate_no_stack_in_examples.ps1`)
  - `python tools/gate_no_stack_in_ui_gallery_shell.py` (or `pwsh tools/gate_no_stack_in_ui_gallery_shell.ps1`)
  - `python tools/gate_no_public_stack_in_ui_kit.py` (asserts legacy stack helpers are hard-deleted)
  - Note: the Python gate scripts share helpers in `tools/_gate_lib.py`.

---

## 4) Diagnostics UX expectations (non-optional)

When this workstream lands, the inspector/diag system should make it easy to answer:

- “Which action did this button dispatch?”
- “Which keybinding triggered (or failed), and why?”
- “Why was the action blocked (availability)?”
- “Which cache root/view rebuilt due to notify/model changes?”

These are not “nice to have”. They are the observability surface that keeps fearless refactors safe.

See also:

- Risk matrix: `docs/workstreams/action-first-authoring-fearless-refactor-v1/RISK_MATRIX.md`

---

## 5) Observability checklist (v1)

This is a reviewer-facing checklist. A change is not considered “landed” unless evidence exists for
each applicable item.

Action dispatch:

- [x] A keybinding-triggered dispatch can be explained (matched binding + key context + resolved ActionId).
  - Evidence: shortcut routing trace (`debug.shortcut_routing_trace[*]`) + key-context stack snapshot
    (`debug.shortcut_routing_trace[*].query.key_context`) gated by
    `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json`.
- [x] A pointer-triggered dispatch can be explained (source element/test_id + resolved ActionId).
  - Evidence: command dispatch trace carries `source_test_id` (best-effort, with fallbacks) gated by
    `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json`.
- [x] A blocked dispatch can be explained (availability outcome + blocking reason/scope).
  - Evidence: command gating trace (`debug.command_gating_trace[*]`) gated by
    `tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json`.

View/cache closure:

- [x] A view rebuild can be explained (notify vs observed deps vs inspection/picking).
  - Evidence: invalidation diagnostics (`dirty_views`, `notify_requests`) in
    `ecosystem/fret-bootstrap/src/ui_diagnostics/invalidation_diagnostics.rs`, plus scripted gate
    `tools/diag-scripts/cookbook/hello/cookbook-hello-view-cache-reuse-and-handler-keepalive.json`.
- [x] Cache reuse can be explained (why reuse happened or was skipped at a cache root).
  - Evidence: cache root diagnostics `cache_roots[*].reuse_reason` in
    `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`, plus scripted gate
    `tools/diag-scripts/cookbook/hello/cookbook-hello-view-cache-reuse-and-handler-keepalive.json`.
