# Action-First Authoring + View Runtime (Fearless Refactor v1) — Evidence and Gates

Last updated: 2026-03-03

This file defines what “done” means beyond subjective UX feel.

The guiding principle: if we change the authoring/routing substrate, we must lock outcomes with
small, deterministic gates (tests and scripted diagnostics), not just manual QA.

---

## 1) Required evidence anchors (update as code lands)

- ADR (actions): `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- ADR (view runtime): `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- Workstream: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`

### Implementation anchors (as of 2026-03-03)

Action identity + typed unit actions:

- `crates/fret-runtime/src/action.rs` (`ActionId`, `TypedAction`)
- `ecosystem/fret/src/actions.rs` (`fret::actions!` macro, `ActionHandlerTable` + unit test)

View runtime (v1):

- `ecosystem/fret/src/view.rs` (`View`, `ViewCx`, `use_state`/`use_state_keyed`/`use_selector`/`use_query`, view-cache reuse + handler keepalive)
- `ecosystem/fret/src/app_entry.rs` (`App::run_view`)

UI gallery adoption (v1):

- `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` (action-first `.action(...)` + `cx.on_action::<...>(...)` via the view runtime)
- `apps/fret-ui-gallery/src/ui/pages/command.rs` (wiring as a `DocSection` + code extraction region)

Editor-grade adoption (workspace shell demo):

- `ecosystem/fret-workspace/src/commands.rs` (`act::*` typed unit actions for workspace command IDs)
- `ecosystem/fret-workspace/src/tab_strip/mod.rs` (tab pressable uses `pressable_dispatch_action_if_enabled*` for activation)
- `ecosystem/fret-workspace/src/tab_strip/widgets.rs` (tab close button uses `pressable_dispatch_action_if_enabled*`)
- `ecosystem/fret-workspace/src/tab_strip/interaction.rs` (middle/right click behaviors record pending dispatch source)
- `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json` (gates `source_kind=pointer` for `workspace.tab.close.doc-a-0`)
- `tools/diag_gate_action_first_authoring_v1.ps1` (includes the workspace shell demo gate)

View/cache observability (diagnostics):

- `ecosystem/fret-bootstrap/src/ui_diagnostics/invalidation_diagnostics.rs` (`dirty_views`, `notify_requests`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs` (`cache_roots[*].reuse_reason`)

Pointer-trigger authoring integration (v1 still dispatches through the command pipeline):

- `crates/fret-ui/src/tree/commands.rs` (command availability/dispatch fallback from overlay roots to the window default root)
- `crates/fret-ui/src/tree/tests/command_availability.rs` (cross-layer fallback tests)
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

Dispatch path explainability (diagnostics traces):

- `crates/fret-runtime/src/command_dispatch_diagnostics.rs` (`WindowCommandDispatchDiagnosticsStore`, `WindowPendingCommandDispatchSourceService`)
- `crates/fret-ui/src/action.rs` (`UiActionHost::record_pending_command_dispatch_source` default hook + adapter implementation)
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs` (pressable activation host forwards pending command-dispatch source metadata so pointer/keyboard triggers are explainable)
- `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (records pointer activation → pending dispatch source)
- `crates/fret-ui/src/tree/shortcuts.rs` (records shortcut routing → pending dispatch source)
- `crates/fret-ui/src/tree/commands.rs` (records dispatch outcome + handled-by element)
- `crates/fret-ui/src/tree/tests/command_dispatch_source_trace.rs` (unit tests: pending pointer source consumption + programmatic default)
- `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (records driver-handled dispatch outcomes to the same trace store, including handler scope classification)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/debug_snapshot_types.rs` (`debug.command_dispatch_trace[*]`)
- `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps_wait.rs` (`handle_wait_command_dispatch_trace_step` gate runner used by scripted diagnostics)
- `crates/fret-diag-protocol/src/lib.rs` (`UiActionStepV2::WaitCommandDispatchTrace`, `UiCommandDispatchTraceQueryV1`)
- `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json` (`wait_command_dispatch_trace` gate)

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

Current scripts (as of 2026-03-03):

- `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json`
- `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json`
- `tools/diag-scripts/cookbook/hello/cookbook-hello-view-cache-reuse-and-handler-keepalive.json`
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

- [ ] A keybinding-triggered dispatch can be explained (matched binding + key context + resolved ActionId).
- [ ] A pointer-triggered dispatch can be explained (source element/test_id + resolved ActionId).
- [ ] A blocked dispatch can be explained (availability outcome + blocking reason/scope).

View/cache closure:

- [ ] A view rebuild can be explained (notify vs observed deps vs inspection/picking).
- [ ] Cache reuse can be explained (why reuse happened or was skipped at a cache root).
