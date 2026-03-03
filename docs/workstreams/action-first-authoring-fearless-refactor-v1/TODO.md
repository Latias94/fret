# Action-First Authoring + View Runtime (Fearless Refactor v1) — TODO

Status: Active
Last updated: 2026-03-02

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
  - Status (as of 2026-03-02):
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
- [ ] AFA-actions-013 Integrate action availability queries with input dispatch v2 semantics.
  - Evidence: `docs/adr/0218-input-dispatch-phases-prevent-default-and-action-availability-v2.md`
- [~] AFA-actions-014 Add diagnostics traces for:
  - keymap resolution → action id,
  - availability gating outcome,
  - dispatch path resolution.
  - Status (as of 2026-03-02):
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
    - Pending: a first-class pointer-triggered mapping from stable selectors (`test_id`) → dispatched `ActionId`
      (today the dispatch trace records `GlobalElementId.0`, which can be correlated via element runtime/semantics snapshots).
- [x] AFA-actions-015 Converge command palette/menu invocation with action dispatch.
  - Goal: palette/menu triggers and pointer triggers share the same action pipeline.
  - Evidence:
    - `ecosystem/fret-bootstrap/src/ui_app_driver.rs` (command palette overlay builds command entries and dispatches via the window command pipeline)
    - `ecosystem/fret-ui-shadcn/src/command.rs` (command palette selection queues a pending command and dispatches via `Effect::Command` after close-on-select completes)
    - `tools/diag-scripts/cookbook/imui-action-basics/cookbook-imui-action-basics-cross-frontend.json` (command palette → action handler gate)

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
- [ ] AFA-view-024 Provide an adapter path for MVU:
  - keep MVU available while views are adopted,
  - document “when to use MVU vs View” in cookbook guidance.
- [x] AFA-view-025 Add view-level observability:
  - “why did this view rebuild?”
  - “why was reuse skipped?”
  - “which models/globals were observed?”
  - Evidence:
    - `debug.dirty_views` + `debug.notify_requests`: `ecosystem/fret-bootstrap/src/ui_diagnostics/invalidation_diagnostics.rs`
    - `debug.cache_roots[*].reuse_reason`: `ecosystem/fret-bootstrap/src/ui_diagnostics/cache_root_diagnostics.rs`
    - view-cache reason source: `crates/fret-ui/src/declarative/mount.rs`

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
  - Status (as of 2026-03-02):
    - View runtime + action-first adoption landed for `commands_keymap_basics`:
      `apps/fret-cookbook/examples/commands_keymap_basics.rs`
    - View runtime + action-first adoption landed for `hello`:
      `apps/fret-cookbook/examples/hello.rs`
    - View runtime + action-first adoption landed for `overlay_basics`:
      `apps/fret-cookbook/examples/overlay_basics.rs`
- [x] AFA-adopt-041 Add at least one ui-gallery page/snippet using actions + view runtime.
  - Evidence:
    - `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs`
    - `apps/fret-ui-gallery/src/ui/pages/command.rs`
- [x] AFA-adopt-042 Add one editor-grade harness adoption:
  - docking/workspace shell uses actions for tab/command semantics (where appropriate).
  - Status (as of 2026-03-02):
    - Workspace tab strip pointer-triggered dispatches record a command dispatch trace source:
      - `ecosystem/fret-workspace/src/tab_strip/mod.rs` (tab activate)
      - `ecosystem/fret-workspace/src/tab_strip/widgets.rs` (tab close button)
      - `ecosystem/fret-workspace/src/tab_strip/interaction.rs` (right/middle click behaviors)
    - Scripted diagnostics gate:
      - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-closes-tab-smoke.json` (asserts `source_kind=pointer` for the close command)
      - `tools/diag_gate_action_first_authoring_v1.ps1` (includes workspace shell demo gate)
- [x] AFA-adopt-043 Update `fretboard` scaffold templates to prefer action-first patterns (once v1 is stable).
  - Rule: do not ship two different default paradigms in templates.
  - Status (as of 2026-03-03):
    - `fretboard new hello` uses View runtime + typed unit actions:
      `apps/fretboard/src/scaffold/templates.rs` (`hello_template_main_rs`)
    - `fretboard new todo` uses View runtime + typed unit actions + selector/query hooks:
      `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`)
    - `fretboard new simple-todo` uses View runtime + typed unit actions:
      `apps/fretboard/src/scaffold/templates.rs` (`simple_todo_template_main_rs`)

---

## F. Evidence + Regression Gates

- [~] AFA-gates-050 Add at least one scripted diag repro that exercises:
  - a keybinding → action dispatch,
  - a button click → action dispatch,
  - action availability gating (disabled state) under a modal barrier.
  - Status (as of 2026-03-02):
    - Implemented (non-modal gating): `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json`
    - Implemented (button click + state update): `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json`
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
- [ ] AFA-clean-062 Delete or quarantine redundant APIs/modules once adoption is complete.
  - Rule: do not delete until all in-tree demos + ecosystem crates have migrated or have explicit “legacy” labeling.
