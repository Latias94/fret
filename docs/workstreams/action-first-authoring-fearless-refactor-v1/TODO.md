# Action-First Authoring + View Runtime (Fearless Refactor v1) — TODO

Status: Closed (v1), maintenance only unless a new narrower lane is opened
Last updated: 2026-03-16

Related:

- Design: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`
- Closeout audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-03-16.md`
- Evidence/gates: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Post-v1 execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_EXECUTION_CHECKLIST.md`
- Shared-surface evidence matrix: `docs/workstreams/action-first-authoring-fearless-refactor-v1/SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`
- Post-v1 proposal: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`
- Post-v1 shortlist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_SURFACE_SHORTLIST.md`
- Post-v1 endgame summary: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_ENDGAME_SUMMARY.md`
- Post-app-entry retained-seam audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_APP_ENTRY_RETAINED_SEAMS_AUDIT_2026-03-10.md`
- Endgame execution outlook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md`
- Default-path productization: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION.md`
- Default-path productization audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION_AUDIT_2026-03-10.md`
- Invalidation/local-state review: `docs/workstreams/action-first-authoring-fearless-refactor-v1/INVALIDATION_LOCAL_STATE_REVIEW.md`
- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Event surface unification: `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVENT_SURFACE_UNIFICATION_DESIGN.md`
- DataTable audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_AUTHORING_AUDIT.md`
- DataTable golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_GOLDEN_PATH.md`
- Teaching-surface inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- Hard-delete endgame index: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_ENDGAME_INDEX.md`
- Source alignment audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/SOURCE_ALIGNMENT_AUDIT_2026-03-09.md`
- Author surface alignment audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/AUTHOR_SURFACE_ALIGNMENT_AUDIT_2026-03-09.md`
- Hard-delete execution checklist: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md`
- App-entry removal playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_REMOVAL_PLAYBOOK.md`
- Compat-driver inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md`
- Compat-driver policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`
- Compat-driver quarantine playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md`
- `use_state` inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md`
- `use_state` policy: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md`
- `use_state` surface playbook: `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_SURFACE_PLAYBOOK.md`
- Command-first widget audit: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md`
- Command-first retained-seam decision: `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`

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

## Closeout reading rule (2026-03-16)

This workstream is now closed for the v1 action/view migration and default-path hardening goals.

- `fret-ui-shadcn` discovery-lane closure and `fret` root lane budgeting belong first to
  `docs/workstreams/authoring-surface-and-ecosystem-fearless-refactor-v1/`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/` is now also a closeout /
  maintenance lane rather than an adjacent active blocker
- the first density-reduction batch is now already landed on the canonical trio, generated
  todo/simple-todo templates, and default-path docs:
  `state.layout(cx).value_*` / `state.paint(cx).value_*` are the taught tracked-read path, and
  `cx.actions().payload_local_update_if::<A, _>(...)` is the taught keyed-row payload write path
- the next child-collection follow-up has now also started landing:
  `ui::single(cx, child)` is the narrow default helper for late-landing one typed child, and the
  first-party root/wrapper cases (`hello`, `hello_counter_demo`, `todo_demo`, generated
  todo/simple-todo templates) now use it instead of `ui::children![cx; child].into()`
- `AppActivateExt` now stays only as a shrinking bridge-maintenance rule; it is not a growth lane
- multi-frontend convergence, editor-grade proof points, and local-state architecture are future
  separate lanes rather than open items on this workstream
- do not promote new default-path sugar, new macros, or broader ecosystem-facing trait sugar from
  this closed workstream
- if fresh cross-surface evidence appears, open a narrower follow-on lane first and use
  `SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md` to justify it

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
- [x] AFA-actions-016a Expand native action-first slots on default-lane shadcn activation widgets.
  - Goal: app-facing shadcn widgets should prefer widget-native `.action(...)` over
    `AppActivateExt` whenever the action slot is stable and semantically narrow.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/badge.rs` (`Badge::action`, `Badge::action_payload`)
    - `ecosystem/fret-ui-shadcn/src/extras/banner.rs`
      (`BannerAction::action`, `BannerClose::action`, close-first action dispatch helper)
    - `ecosystem/fret-ui-shadcn/src/extras/ticker.rs` (`Ticker::action`, `Ticker::action_payload`)
    - `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
  - 2026-03-16 bridge-shrink follow-up:
    - `fret::app::AppActivateSurface` no longer forwards `Badge`, `BannerAction`,
      `BannerClose`, or `Ticker`,
    - those widgets now stay on their native `.action(...)` / `.action_payload(...)` surface,
      while bridge-only `.listen(...)` remains reserved for the smaller residue that still lacks
      native action slots.
    - first-party UI Gallery link-badge authoring also now stays on the widget-owned
      `Badge::on_activate(...)` hook when it needs to suppress the default URL open effect during
      diagnostics; it does not reintroduce `AppActivateExt` just to attach a no-op override.
- [x] AFA-actions-016b Expand native action-first slots on first-party AI wrapper buttons.
  - Goal: `fret-ui-ai` wrappers that are structurally just button/tooltip/layout composition
    should not require `AppActivateExt` for ordinary action dispatch.
  - Evidence:
    - `ecosystem/fret-ui-ai/src/elements/workflow/controls.rs`
    - `ecosystem/fret-ui-ai/src/elements/message_actions.rs`
    - `ecosystem/fret-ui-ai/src/elements/artifact.rs`
    - `ecosystem/fret-ui-ai/src/elements/confirmation.rs`
    - `ecosystem/fret-ui-ai/src/elements/prompt_input.rs`
    - `ecosystem/fret-ui-ai/src/elements/checkpoint.rs`
    - `ecosystem/fret-ui-ai/src/elements/conversation_download.rs`
    - `ecosystem/fret-ui-ai/src/elements/web_preview.rs`
    - `ecosystem/fret-ui-ai/src/surface_policy_tests.rs`
- [x] AFA-actions-016c Expand native action-first slots on remaining default-lane Material3 wrappers.
  - Goal: Material3 wrappers that are still app-facing click targets should offer the same native
    `.action(...)` contract as the crate's primary buttons/icon-buttons.
  - Evidence:
    - `ecosystem/fret-ui-material3/src/card.rs`
    - `ecosystem/fret-ui-material3/src/dialog.rs`
    - `ecosystem/fret-ui-material3/src/top_app_bar.rs`
    - `ecosystem/fret-ui-material3/src/lib.rs`
  - 2026-03-16 bridge-shrink follow-up:
    - `fret::app::AppActivateSurface` no longer forwards `fret_ui_material3::{Card,
      DialogAction, TopAppBarAction}`,
    - those wrappers now stay entirely on their native `.action(...)` surface unless a caller
      intentionally drops to raw `.on_activate(...)`.
- [x] AFA-actions-017 Add action-first naming parity helpers in `fret-ui-kit`.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/command.rs` (`action_is_enabled`, `dispatch_action_if_enabled`)
    - `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (`pressable_dispatch_action_if_enabled`)
- [x] AFA-actions-018 Ensure action availability/dispatch can reach app handlers from overlay roots.
  - Goal: portal-mounted menus/overlays can invoke app-level actions without duplicating handler tables.
  - Evidence:
    - `crates/fret-ui/src/tree/commands.rs` (dispatch/availability fallback to default root)
    - `crates/fret-ui/src/tree/tests/command_availability.rs` (cross-layer fallback tests)
- [x] AFA-actions-016d Continue shrinking bridge-only residue after the grouped `UiCx` helper pass.
  - Goal: keep `AppActivateExt` as an explicitly shrinking residue list rather than a stable
    ecosystem integration surface.
  - 2026-03-16 shrink batch landed:
    - `fret::app::AppActivateSurface` no longer forwards
      `fret_ui_ai::{WorkflowControlsButton, MessageAction, ArtifactAction, ArtifactClose, CheckpointTrigger}`,
    - first-party UI Gallery snippets now route those widgets through `UiCxActionsExt` plus
      widget-owned `.on_activate(...)` instead of `AppActivateExt`.
  - 2026-03-16 button/sidebar follow-up landed:
    - `fret::app::AppActivateSurface` no longer forwards
      `fret_ui_shadcn::{facade::Button, facade::SidebarMenuButton}`,
    - `SidebarMenuButton` now exposes native `.action_payload(...)`,
    - first-party UI Gallery button/sidebar listener snippets now use `UiCxActionsExt` plus
      widget-owned `.on_activate(...)` instead of `AppActivateExt`.
  - Closure note (2026-03-16):
    - the first-party default widget bridge table in `ecosystem/fret/src/view.rs` is now
      intentionally empty,
    - `AppActivateExt` remains only as an explicit custom/third-party activation-only seam plus
      a maintenance gate against future bridge growth.
  - Revalidation bundle:
    - `cargo nextest run -p fret grouped_authoring_surfaces_replace_flat_app_ui_helpers usage_and_component_docs_keep_app_activate_surface_narrow shadcn_docs_keep_advanced_hooks_off_curated_lane app_prelude_stays_explicit_instead_of_reexporting_legacy_surface advanced_prelude_reexports_app_facing_view_aliases uicx_actions_ext_is_part_of_the_default_and_advanced_preludes app_lane_keeps_explicit_uicx_helper_traits_for_manual_imports crate_usage_guide_keeps_query_guidance_on_grouped_app_surfaces --no-fail-fast`
    - `cargo nextest run -p fret-ui-gallery --test ui_authoring_surface_default_app --no-fail-fast`
  - Rule:
    - if a widget can stay on native `.action(...)`, `.action_payload(...)`, or widget-owned
      `.on_activate(...)`, do not add or keep it on `AppActivateSurface`.

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
    - Entry points: `ecosystem/fret/src/app_entry.rs` (`FretApp::view`, `FretApp::view_with_hooks`)
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
    - `docs/workstreams/genui-json-render-v1/genui-json-render-v1.md` (ActionId/CommandId naming + executor glue note)
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
      - `tools/diag_gate_action_first_authoring_v1.py` (includes workspace shell demo gate)
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
    - `apps/fret-ui-gallery/src/ui/pages/ai_commit_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_chain_of_thought_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_checkpoint_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_test_results_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_persona_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_context_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_mic_selector_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_model_selector_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_shimmer_demo.rs`
    - `apps/fret-ui-gallery/src/ui/pages/ai_voice_selector_demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/table/demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/table/usage.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/table/footer.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/table/rtl.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/table/actions.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/checkbox/table.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/typography/table.rs`
    - `apps/fret-cookbook/examples/drag_basics.rs`
    - `apps/fret-cookbook/examples/effects_layer_basics.rs`
    - `apps/fret-cookbook/examples/text_input_basics.rs`
  - Next likely slice (as of 2026-03-09): keep this pass focused on remaining table-heavy ui-gallery
    reference pages where `Table*::build(...)` removes real density noise without widening the
    helper surface; `ai_artifact_demo.rs` does not currently need this pass, so any next slice
    should be chosen only from remaining reference pages that still contain substantial eager
    `Table*::new(...).into_element(cx)` sections after this selector/documentation cleanup.
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
  - Additional 2026-03-07 follow-up: `router_basics` was re-audited as part of the same sweep; that former outlet-card cliff has since been removed in `fret-router-ui` and the modern surface now lands directly on `RouterOutlet::into_element_by_leaf(...)`, so any remaining churn there is no longer cookbook-local builder noise but the intentional host-side router availability path.

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
    - `tools/gate_no_mvu_in_cookbook.py`

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
- [x] AFA-postv1-002b Decide whether all top-level example links should collapse to one canonical docs
  page while preserving `examples/README.md` as a GitHub portal alias.
  - Decision (2026-03-16 closeout): yes, treat `docs/examples/README.md` as the canonical docs
    index and keep `examples/README.md` only as the GitHub portal alias; this is now documentation
    navigation policy, not an open action-first item.
  - Evidence:
    - `docs/examples/README.md`
    - `examples/README.md`
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

Current sequencing note (as of 2026-03-09):

- first: productize the current default path (onboarding ladder, default/comparison/advanced taxonomy, helper visibility) and keep docs/templates/examples aligned on that story,
- productization note: treat `DataTable` as a separate business-table/reference-surface audit rather than continuing the primitive `Table` builder-first cleanup under that same bucket,
- second: continue local-state / invalidation ergonomics (`AFA-postv1-001` + `AFA-postv1-004`) only where real medium-surface evidence still shows a state-boundary cliff after the doc/product pass,
- builder-first note: `AFA-postv1-002` is now maintenance mode for this pass; reopen only if a new cross-surface host/root seam still forces eager landing across multiple real default-facing surfaces,
- third: keep keyed-list / payload-row handler ergonomics (`AFA-postv1-003`) in maintenance mode unless a new medium surface shows the same row-local pressure beyond the current todo-style evidence,
- fourth: only after the first three stabilize, re-evaluate narrow macros (`AFA-postv1-005`).
- shortlist note: `POST_V1_SURFACE_SHORTLIST.md` now fixes the current priority order explicitly so
  `DataTable` helper churn, broad macros, and compat cleanup do not displace the higher-value
  default-path and invalidation work.
- productization note (as of 2026-03-09): `DEFAULT_PATH_PRODUCTIZATION.md` now defines the
  repo-wide ladder and label contract explicitly, and `README.md`, `docs/first-hour.md`,
  `docs/crate-usage-guide.md`, `docs/ui-ergonomics-and-interop.md`, the examples index, todo
  golden-path note, cookbook README/index, gallery README/page framing, and generated scaffold
  READMEs now use that same default/comparison/advanced framing.
- invalidation policy note (as of 2026-03-09): `INVALIDATION_DEFAULT_RULES.md` now compresses the
  post-v1 default rule into one short card: straightforward single-local writes stay on
  `on_action_notify_local_*`, coordinated writes stay on `on_action_notify_models::<A>(...)`,
  transient App/runtime effects stay on `on_action_notify_transient::<A>(...)`, and explicit
  `notify()` / render-time invalidation remain escape hatches only for imperative/runtime/cache
  boundaries.
  - Classification update (as of 2026-03-09): the remaining default-path
    `on_action_notify_models::<A>(...)` surfaces are now explicitly grouped as
    coordinated-write ownership, command/keymap ownership, and cross-field form ownership so they
    do not keep being mistaken for generic invalidation-helper gaps.
- endgame summary note (as of 2026-03-09): `POST_V1_ENDGAME_SUMMARY.md` now compresses the current
  state into one page: default-path convergence is effectively complete, `AFA-postv1-002` /
  `003` / `004` are maintenance-mode tracks, `AFA-postv1-001` remains open only as an
  architectural local-state question, and the remaining cleanup pressure is primarily in the
  staged hard-delete/quarantine sequence.
- app-entry removal note (as of 2026-03-09): `APP_ENTRY_REMOVAL_PLAYBOOK.md` now records the
  historical patch shape for the `App::ui*` hard delete, so the repo no longer has to reconstruct
  why that pre-release removal was safe.
- compat-runner quarantine note (as of 2026-03-09):
  `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` now records the quarantine-first execution checklist for
  `run_native_with_compat_driver(...)`, so a future facade-reduction pass can move that advanced
  seam behind an explicit compat boundary without reopening the same policy debate.
- `use_state` surface note (as of 2026-03-09): `USE_STATE_SURFACE_PLAYBOOK.md` now records the
  future keep-vs-deprecate execution checklist for the explicit raw-model seam, so a later
  surface-reduction pass can proceed without relitigating whether the default local-state migration
  is already complete.
- hard-delete index note (as of 2026-03-09): `HARD_DELETE_ENDGAME_INDEX.md` now acts as the
  one-page entrypoint for `App::ui*`, compat runner, `use_state`, and command-first retained-seam
  cleanup, so future work can start from a single reviewer-facing summary before opening the
  deeper matrix/checklist/playbooks.
- command-first retained-seam note (as of 2026-03-09):
  `COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md` now records the boundary rule for the remaining
  command-shaped surfaces, so future work only reopens this lane on default-path leak or explicit
  deprecation rather than treating it as generic residue.
- source-alignment audit note (as of 2026-03-09): `SOURCE_ALIGNMENT_AUDIT_2026-03-09.md` now
  records that the remaining hard-delete / retained-seam decisions are source-aligned, and the
  missing compat-runner default-path gate is now closed by
  `tools/gate_compat_runner_default_surface.py`.
- author-surface audit note (as of 2026-03-09): `AUTHOR_SURFACE_ALIGNMENT_AUDIT_2026-03-09.md`
  now records that the remaining author-entry docs already align with the action-first story, and
  `ecosystem/fret-ui-material3/README.md` now closes the last missing crate-entry surface.
- endgame outlook note (as of 2026-03-09): `ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md` now records
  the repo's current execution forecast: `App::ui*` is now closed, while compat runner,
  `use_state`, and command-first retained seams are the remaining lanes expected to stay unless a
  later explicit product decision reopens them.
- app-entry hard-delete update (as of 2026-03-10): `ecosystem/fret/src/app_entry.rs` no longer
  exposes `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}`, and the workstream docs now treat
  that lane as closed pre-release rather than waiting on a published deprecation window.
- retained-seam audit update (as of 2026-03-10):
  `POST_APP_ENTRY_RETAINED_SEAMS_AUDIT_2026-03-10.md` now records the narrower post-app-entry
  verdict: `ViewCx::use_state::<T>()` remains a retained explicit raw-model seam because it still
  underlies `use_local*`, and `run_native_with_compat_driver(...)` remains a retained advanced
  interop seam because its caller families still carry real capability; neither is the next
  delete-ready cleanup lane.
- productization audit update (as of 2026-03-10):
  `DEFAULT_PATH_PRODUCTIZATION_AUDIT_2026-03-10.md` now records that the ingress docs are broadly
  aligned on the same default/comparison/advanced taxonomy, and that the remaining drift was only
  wording-level in `README.md` and the `fret` crate README rather than evidence for more helper/API
  expansion.

- [~] AFA-postv1-001 Investigate direct local-state ergonomics beyond `Model<T>` in `ViewCx::use_state`.
  - Goal: let simple demos keep state in a plain-Rust shape without weakening dirty/notify semantics
    or shared-model interop.
  - Evidence target: rewrite one medium demo as a comparison branch before promoting any new surface.
  - Update (as of 2026-03-06): additive prototype landed as `LocalState<T>` + `ViewCx::use_local*` / `watch_local(...)`; `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now use the prototype instead of storing explicit local model handles in the view struct, with the query demos validating `use_local` alongside `use_query` / `use_query_async` + transient invalidation.
  - Update (as of 2026-03-07): `TrackedStateExt::{layout, paint, hit_test}` plus `LocalState::watch(cx)` and the first `ViewCx::on_action_notify_local_*` helpers landed as the next additive step. `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now read local state from the handle side and use local-state-specific write helpers for the straightforward set/toggle cases without re-exposing raw model handles. `apps/fret-cookbook/examples/hello_counter.rs` and `apps/fret-cookbook/examples/query_basics.rs` now mirror the same direction for the first medium cookbook samples.
  - Update (as of 2026-03-08): `LocalState::read_in` / `revision_in` now cover the remaining ?generic model-store closure? read path too, so cookbook `hello_counter`, `form_basics`, `text_input_basics`, `simple_todo`, `virtual_list_basics`, the `fretboard` simple-todo template, and `hello_counter_demo` no longer need to leak `local.model()` just to read or revision-check local state inside `on_action_notify_models` or derived revision code.
  - Update (as of 2026-03-08, store-side value helpers): `LocalState::value_in*` now mirrors render-time `value_*` reads for the common `ModelStore` transaction path. `simple_todo`, `simple_todo_v2_target`, and the `fretboard` simple-todo template now use `value_in_or*` for their plain local-state reads, which narrows the remaining invalidation/default-path discussion back to tracked writes rather than store-side read boilerplate.
  - Update (as of 2026-03-08, handled-aware local writes): `LocalState::update_in_if` now lets the mutation closure return the `handled` decision directly, while grouped `locals::<A>(...)` and `payload_local_update_if::<A>(...)` own the rerendering path above it. `simple_todo_v2_target` uses that shape for clear/toggle/remove list mutations, so the remaining tracked-write pressure is now more about which action surface should own multi-state transactions than about passing handled flags through external mutable locals.
  - Inventory note (as of 2026-03-08): `docs/workstreams/action-first-authoring-fearless-refactor-v1/TRACKED_WRITE_PATTERN_INVENTORY.md` now records the remaining repo-wide transaction shapes. Current conclusion: do not add another default helper yet; the next evidence target should be explicit-model collection surfaces rather than more local-state sugar.
  - Inventory note (as of 2026-03-08, explicit-model collections): `docs/workstreams/action-first-authoring-fearless-refactor-v1/EXPLICIT_MODEL_COLLECTION_SURFACE_INVENTORY.md` now records that both `apps/fret-examples/src/todo_demo.rs` and the `fretboard` simple-todo scaffold path have joined the v2 local-state keyed-list path. Current conclusion: default-surface collection migration is no longer the blocker; do not widen tracked-write helpers just to chase the remaining explicit comparison/advanced surfaces.
  - Next-phase note (as of 2026-03-08): with cookbook / app-grade / scaffold keyed-list defaults aligned, the immediate next work should focus on onboarding docs, default/comparison/advanced taxonomy, visual productization, and deprecation planning rather than more generic authoring helpers.
  - Update (as of 2026-03-08, query handle follow-up): query handle-side reads now stay handle-first across both `ViewCx` and `ElementContext` authoring surfaces. `TrackedStateExt` covers `QueryHandle<T>` in the `ViewCx` path, while `fret-ui-kit::declarative::QueryHandleWatchExt` covers `ElementContext` surfaces behind `state-query`, so cookbook `query_basics`, `fret-examples` `query_demo` / `query_async_tokio_demo` / `async_playground_demo` / `markdown_demo`, the scaffold query-tip template, `docs/examples/todo-app-golden-path.md`, `docs/integrating-tokio-and-reqwest.md`, `docs/workstreams/standalone/imui-state-integration-v1.md`, and `fret-markdown`'s MathJax/Mermaid helpers can all read query state from the handle side via `handle.layout(cx).value_*` / `handle.layout_query(cx).value_*` instead of reopening `handle.model()`.
  - Update (as of 2026-03-08, todo comparison target): `apps/fret-cookbook/examples/simple_todo_v2_target.rs` now keeps a keyed todo list in `LocalState<Vec<TodoRow>>` and uses payload actions for per-row toggle/remove, proving that the current runtime can already express small view-owned dynamic collections without `Model<Vec<_>>`.
  - Update (as of 2026-03-08, checkbox source alignment): after comparing against `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx` and its checkbox examples, `ecosystem/fret-ui-shadcn/src/checkbox.rs` now supports a shadcn-style checked snapshot path (`Checkbox::from_checked(...)`) plus `action(...)` / `action_payload(...)`. That removes the need for per-row checkbox models in the todo comparison target; the remaining visible noise shifts more clearly to root-level handler registration and keyed-row payload-action placement ergonomics.
  - Review update (as of 2026-03-09): `INVALIDATION_LOCAL_STATE_REVIEW.md` now uses `simple_todo_v2_target`, `query_basics`, `commands_keymap_basics`, and `form_basics` as a focused medium-surface set. Current result: the remaining pressure is no longer local-state storage shape or explicit `notify()` burden; keyed-list pressure has shifted to root handler placement for payload row actions, query/client invalidation remains an intentional render-time escape hatch, and both command/keymap plus cross-field form coordination remain intentional root-level ownership boundaries rather than default sugar targets.
  - Current boundary (as of 2026-03-09): the default local-state teaching path is now stable enough (`use_local*`, `value_*`, `value_in*`, `read_in`, `update_in_if`, direct text bridge, query-handle reads, and the todo-like `LocalState<Vec<_>>` path). The remaining distance to the GPUI-style north-star is no longer “missing default helper surface”; it is the deeper fact that `LocalState<T>` is still model-backed rather than plain-Rust/self-owned state, and some existing widget/runtime seams still intentionally bridge through `clone_model()` or explicit host/model boundaries.
  - Current recommendation: keep `AFA-postv1-001` open as an architectural gap, not as an active helper-expansion track. Reopen additive API work only if a new real default-facing medium surface still feels materially blocked after the current `use_local*` path, and do not chase a fake plain-Rust story that would weaken shared-model interop, diagnostics, or dirty/notify determinism without a stronger runtime-level proposal.
- [x] AFA-postv1-002 Investigate builder-first composition paths that reduce `ui::children!` and nested
  `into_element(cx)` in medium demos.
  - Goal: measure whether a builder-only path materially improves density without helper sprawl.
  - Evidence target: compare `hello_counter_demo` or `query_demo` against the current default path.
  - Update (as of 2026-03-06): `fret-ui-kit::ui::UiElementSinkExt`, `UiChildIntoElement`, and `ui::*_build` sinks now power builder-first `query_demo` and `query_async_tokio_demo` variants while also letting `ui::children!` / `push_ui()` accept nested layout builders plus host-bound `Card::build(...)` / `CardHeader::build(...)` / `CardContent::build(...)` values without an extra `.into_element(cx)` cliff. That same card-builder path now also covers the `fretboard` todo/simple-todo templates plus `commands_keymap_basics`, `form_basics`, and `async_inbox_basics` through the generic `.ui()` patch path; `ecosystem/fret-ui-shadcn/src/layout.rs` now exposes `container_vstack_build(...)` / `container_hstack_build(...)` / `container_hstack_centered_build(...)` so the first older helper family can stay on the same late-landing pipeline; `ecosystem/fret-ui-shadcn/src/table.rs` plus `ecosystem/fret-genui-shadcn/src/resolver/data.rs` now extend that same pattern into the table composite stack (`Table::build(...)` / `TableHeader::build(...)` / `TableBody::build(...)` / `TableFooter::build(...)` / `TableRow::build(...)`) for GenUI-driven data tables; `TableCell::build(child)` now serves as the first single-child late-landing sample (also reflected in the UI Gallery typography table snippet); `DialogTrigger::build(...)` / `SheetTrigger::build(...)` / `DrawerTrigger::build(...)` now bring the first overlay-trigger wrappers onto the same child pipeline for sink-based / direct late-landing paths and the `Dialog` / `Sheet` composition builders accept those `*_Trigger::build(...)` values directly; the wider overlay single-child family now follows the same shape too (`PopoverTrigger::build(...)`, `PopoverAnchor::build(...)`, `HoverCardTrigger::build(...)`, `HoverCardAnchor::build(...)`, `TooltipTrigger::build(...)`, `TooltipAnchor::build(...)`); `Popover::new(cx, trigger, content)` / `Popover::new_controllable(...)` now remove the next popover root landing cliff while `Popover::from_open(...)` stays as the explicit advanced seam for managed-open or anchor-aware closure composition; `DropdownMenuTrigger::build(...)` plus `DropdownMenu::build(...)` / `DropdownMenu::build_parts(...)` now bring the first composite menu root onto that same late-landing path; and `HoverCard::new(cx, ...)` / `HoverCard::new_controllable(...)` / `Tooltip::new(cx, ...)` keep the same root-level direction, with `Tooltip::new(...)` accepting `TooltipContent` directly. The UI Gallery now teaches the intended overlay paths through `apps/fret-ui-gallery/src/ui/snippets/hover_card/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/basic.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/align.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/rtl.rs`, `apps/fret-ui-gallery/src/ui/snippets/popover/with_form.rs`, `apps/fret-ui-gallery/src/ui/snippets/dropdown_menu/*.rs`, and `apps/fret-ui-gallery/src/ui/snippets/tooltip/demo.rs`. Remaining gap: broader composite APIs beyond the first dropdown-menu path and the remaining eager-only wrappers still sit outside the modern child pipeline.
  - Update (as of 2026-03-07): `Dialog::compose().content_with(...)` / `Sheet::compose().content_with(...)` support deferred content authoring so `DialogClose::from_scope()` / `SheetClose::from_scope()` can be used inside composed content without forcing eager `into_element(cx)` landing.
  - Update (as of 2026-03-07): `CardFooter::build(...)` now extends the same late-landing card-section path to footer action rows, which lets `hello_counter` drop its last footer-level landing cliff; `virtual_list_basics` is correspondingly down to a single low-level `cx.container(...)` row seam, so the next meaningful cleanup would be a higher-level row/container authoring surface rather than more cookbook-local churn.
  - Update (as of 2026-03-07): `toast_basics` and `text_input_basics` now validate that the current builder-first surface is sufficient for a host-bound card shell plus semantics-rich status badges, while the re-audit of `overlay_basics` shows the next real gap more clearly: the dialog recipe family still needs a builder-first content/root path before that example can shed its remaining eager trigger/content seams.
  - Update (as of 2026-03-07): `fret-ui-kit::ui::effect_layer(...)` / `effect_layer_build(...)` now move effect-root child collection onto the same late-landing child pipeline as `container` / `stack`, which lets the renderer demos drop their pre-effect landing cliff. At that point `simple_todo` keyed rows and `RouterOutlet` closures were the clearest next host-bound gaps; both have since been addressed by the keyed helper follow-up and the later router outlet surface closeout.
  - Update (as of 2026-03-07): `fret-ui-kit::ui::keyed(...)` now preserves the original keyed callsite across builder-first sink paths by routing through `ElementContext::keyed_at(...)`. `simple_todo`, `payload_actions_basics`, and the generated fretboard todo templates all drop their last keyed-row eager landing cliff, which makes `RouterOutlet` leaf closures the clearer next host-bound seam to tackle.
  - Update (as of 2026-03-07, finalized on 2026-03-14): `fret-router-ui::RouterOutlet::{into_element,into_element_by_leaf,into_element_by_leaf_with_status}` now accept builder-first route content directly via `IntoUiElement<App>`. The temporary router-local `RouterOutletIntoElement` adapter and outlet `*_ui(...)` overloads are now deleted, and `router_basics` correspondingly stays on the single named outlet surface.
  - Update (as of 2026-03-07): `fret-ui-kit::ui::container_props(...)` / `container_props_build(...)` now keep caller-specified `ContainerProps` roots on the same late-landing child pipeline as `container` / `stack` / `effect_layer`. `virtual_list_basics` correspondingly drops both its low-level row-root and list-slot eager landing cliffs, so the remaining authoring gaps are no longer cookbook-local raw list containers but the narrower overlay/dialog host seams and other explicit `AnyElement` escape hatches.
  - Update (as of 2026-03-07): `DialogContent::build(...)` / `DialogHeader::build(...)` / `DialogFooter::build(...)` together with their `Sheet*::build(...)` counterparts now keep nested overlay sections on the same late-landing path while still landing at the existing dialog/sheet root boundary. `overlay_basics` correspondingly drops its remaining inner dialog content/header/footer eager landing cliffs, so the next overlay work is more about the still-older alert-dialog/drawer surfaces and other explicit `AnyElement` escape hatches than cookbook-local dialog glue.
  - Update (as of 2026-03-07): `AlertDialogContent::build(...)` / `AlertDialogHeader::build(...)` / `AlertDialogFooter::build(...)` together with their `Drawer*::build(...)` counterparts now keep the remaining shadcn overlay section seams on the same late-landing path while still landing at the existing alert-dialog/drawer root boundary. The UI Gallery `alert_dialog` / `drawer` demos now exercise that builder-first path directly, so the next overlay work is narrower root-level `AnyElement` escape hatches rather than section-local eager landings.
  - Update (as of 2026-03-07): `AlertDialogTrigger::build(...)` plus `AlertDialog::build(cx, trigger, content)` and the new generic `AlertDialog::compose().trigger(...)` / `Drawer::compose().trigger(...)` trigger-arg support now close the older root trigger seam too. The UI Gallery `alert_dialog/demo.rs`, `alert_dialog/parts.rs`, `alert_dialog/usage.rs`, `drawer/demo.rs`, and `drawer/usage.rs` now teach those narrower root helpers directly, so the remaining overlay gaps are mostly around broader root content/part sugar rather than trigger landing.
  - Follow-up conclusion (as of 2026-03-09): the later `Alert::build(...)`, `ScrollArea::build(...)`, and `FieldSet::build(...)` / `FieldGroup::build(...)` / `Field::build(...)` passes closed the last clearly repeated medium-surface families. The remaining visible density is now mostly adoption of existing builders, advanced/runtime-owned host seams, or intentionally separate product questions such as `DataTable`, not evidence that another generic builder-first helper surface is missing.
  - Historical verification update (as of 2026-03-13, before the later `compose()` teaching-surface follow-up): the `DropdownMenu` root fearlessly-refactored surface was fully closed on first-party call sites. At that checkpoint, the default recipe path still stayed `DropdownMenu::uncontrolled(cx).build(...)` / `build_parts(...)`, the explicit managed-open seams stayed `DropdownMenu::from_open(...)` / `new_controllable(...)`, the UI Gallery dropdown snippets/pages plus shadcn parity tests stayed aligned with that split, and the temporary `DropdownMenu::new(open)` compatibility alias had already been removed. Validation: `cargo check -p fret-ui-shadcn --tests`, `cargo check -p fret-ui-gallery`, and `cargo check -p fret-ui-ai`.
  - Verification follow-up (as of 2026-03-14): the menu-root teaching surface has now moved one step further. `DropdownMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` and `ContextMenu::uncontrolled(cx).compose().trigger(...).content(...).entries(...)` are the default copyable paths, while `build(...)`, `build_parts(...)`, and `into_element_parts(...)` remain the narrower adapter seams. The typed `DropdownMenuComposition<H, _>` / `ContextMenuComposition<H, _>` builders now implement `IntoUiElement<H>` directly, first-party menu snippets plus overlay gallery helpers follow that path, and `dropdown_menu/parts.rs` is the only intentional `build_parts(...)` snippet left on the default-app lane. Validation: `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_default_app menu_snippets_keep_build_parts_only_for_the_intentional_parts_example -- --exact --nocapture`; `CARGO_TARGET_DIR=target/codex-ui-gallery cargo test -p fret-ui-gallery --test ui_authoring_surface_internal_previews gallery_overlay_preview_retains_intentional_raw_boundaries -- --exact --nocapture`; `CARGO_TARGET_DIR=target/codex-shadcn-lib cargo test -p fret-ui-shadcn --lib root_promotes_uncontrolled_builder_path -- --nocapture`.
  - Verification update (as of 2026-03-13): the `Popover` root fearlessly-refactored surface is now fully closed on first-party call sites. The default recipe path stays `Popover::new(cx, trigger, content)`, the managed-open/anchor-aware advanced seams stay `Popover::from_open(...).into_element_with(...)` / `into_element_with_anchor(...)`, the UI Gallery popover snippets/pages plus the GenUI overlay resolver stay aligned with that split, and the remaining shadcn test callers have been migrated off the removed one-arg root constructor path. Validation: `cargo check -p fret-ui-shadcn --tests`, `cargo check -p fret-ui-ai --lib`, and `cargo check -p fret-genui-shadcn --lib`.
  - Current conclusion: treat builder-first seam work as maintenance mode for this pass. Reopen only if a new cross-surface host/root seam appears that still forces eager `AnyElement` landing across multiple real default-facing surfaces.
- [x] AFA-postv1-003 Investigate keyed-list / payload-row handler ergonomics without expanding the default helper surface prematurely.
  - Goal: decide whether keyed dynamic lists need a narrower row-action/handler placement surface than the current root `on_payload_action_notify` path.
  - Guardrail: do not treat command/keymap, query/runtime-trigger, or cross-field form handlers as evidence for this item; only keyed-list/payload-row pressure counts.
  - Prototype update (as of 2026-03-09): `ecosystem/fret/src/view.rs` now exposes `ViewCx::on_payload_action_notify_local_update_if::<A, T>(...)` as a deliberately narrow keyed-list helper. It keeps action identity and root handler registration explicit, but removes the repeated `LocalState` clone + `host.models_mut()` boilerplate for payload-row mutations.
  - Adoption update (as of 2026-03-09): `apps/fret-cookbook/examples/simple_todo_v2_target.rs`, `apps/fret-examples/src/todo_demo.rs`, and the generated `simple_todo` scaffold in `apps/fretboard/src/scaffold/templates.rs` now use that helper for row toggle/remove flows.
  - Review update (as of 2026-03-09): `apps/fret-cookbook/examples/simple_todo.rs` now also uses `LocalState<Vec<TodoRow>>` plus payload row toggles on the default cookbook path, while `INVALIDATION_LOCAL_STATE_REVIEW.md` records that `query_basics`, `commands_keymap_basics`, and `form_basics` do **not** justify broader keyed-list sugar because their root handlers represent explicit ownership boundaries rather than accidental placement noise.
  - Current conclusion: the narrow helper plus current adoption are sufficient for this pass. Keep broader keyed-list / payload-row sugar deferred and reopen only if another medium surface shows the same row-local handler-pressure pattern as the todo-like evidence slice.

- [~] AFA-postv1-004 Evaluate v2 invalidation ergonomics: keep explicit `notify()` as a low-level runtime escape hatch while making local-state writes rerender implicitly by default.
  - Goal: preserve cache/debug determinism without forcing users to call `notify()` after most tracked state writes.
  - Evidence target: prototype one medium demo and confirm diagnostics still explain rebuild reasons.
  - Update (as of 2026-03-06; narrowed again on 2026-03-17): the prototype keeps explicit `notify()` out of the call site by combining `LocalState::update_in` / `set_in` with the existing `on_action_notify_models` path in `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo`. The later direct `LocalState::update_action*` / `set_action` seam did not earn first-party proof and has now moved back to internal runtime substrate, leaving public rerendering writes on grouped action helpers instead.
  - Update (as of 2026-03-07): `ViewCx::on_action_notify_local_update` / `on_action_notify_local_set` / `on_action_notify_toggle_local_bool` now promote the same “tracked local write => redraw + notify” rule into a first-class authoring path. The current medium demos plus cookbook `hello_counter` and `query_basics` use those helpers for the simple local-state mutations, while `commands_keymap_basics` / `text_input_basics` validate command availability and widget interop on `use_local*` / `state.layout(cx).value_*` / `state.paint(cx).value_*`, `form_basics` shows that multi-field validation/reset flows can stay on the generic `on_action_notify_models` path, `simple_todo` demonstrates the first keyed-list hybrid where draft/ID counters move to local state but the dynamic collection itself remains an explicit `Model<Vec<_>>`, `drop_shadow_basics` proves the same local-state bridge on a pure toggle-only renderer demo, `markdown_and_code_basics` extends that bridge to a mixed editor/render-options page built from model-centered `Textarea` / `ToggleGroup` / `Switch` widgets, `assets_reload_epoch_basics` shows the same local-state path on a host/runtime escape-hatch page where the counter is local but the actual asset reload bump plus redraw/RAF scheduling intentionally stay in render-time code, `virtual_list_basics` closes the first virtualization hybrid by moving mode/toggle/jump controls to local state while intentionally keeping the items collection plus scroll/reorder coordination on explicit model/runtime surfaces, `theme_switching_basics` applies the same hybrid rule to theme selection by moving the chosen scheme to local state while keeping theme application plus redraw/RAF sync as render-time host effects, and `icons_and_assets_basics` now does the same for asset demos by moving the reload bump counter to local state while keeping asset reload epoch bump plus redraw/RAF synchronization as render-time host effects, while `customv1_basics` closes the same loop for renderer/effect demos by moving `enabled` / `strength` to local state and intentionally keeping effect registration, capability checks, and effect-layer plumbing render-time/runtime-owned. `notify()` remains a low-level escape hatch rather than a default teaching-surface step, Queue A and Queue B are now cleared, and the teaching-surface inventory treats the remaining explicit-model cookbook cases as intentionally advanced rather than pending default-surface migrations. The new `LocalState::read_in` / `revision_in` helpers keep even those generic `on_action_notify_models` / derived-revision closures on the local-state handle surface, so the remaining local-state pressure is increasingly about write-path policy rather than read-path leakage.
  - Update (as of 2026-03-08, follow-up): cookbook `customv1_basics` now uses `on_action_notify_toggle_local_bool` for its simple `enabled` flag, while `commands_keymap_basics` intentionally stays on the generic `on_action_notify_models` transaction for command availability gating but now reads the gate through `LocalState::read_in(...)` instead of reopening the raw model handle.
  - Update (as of 2026-03-08, tracked-write review; narrowed again on 2026-03-17): no additional invalidation helper is promoted into the default path for now. `LocalState::update_in` / `set_in` are store-only transaction helpers, while grouped action helpers (`cx.actions().local_*`, `cx.actions().locals::<A>(...)`, and `payload_local_update_if::<A>(...)`) remain the first-class `tracked local write => request_redraw + notify` boundary. A focused unit test in `ecosystem/fret/src/view.rs` locks that contract so `notify()` can stay a low-level escape hatch instead of reappearing as a default teaching-surface step.
  - Update (as of 2026-03-09, cookbook keyed-list alignment): `apps/fret-cookbook/examples/simple_todo.rs` now also uses `LocalState<Vec<TodoRow>>` plus payload row toggles, so the default cookbook keyed-list lesson no longer contradicts the scaffold/app-grade baseline. The remaining keyed-list comparison pressure is now concentrated in `apps/fret-cookbook/examples/simple_todo_v2_target.rs`, which stays as the denser payload-row/root-handler evidence slice rather than as a “real default path” preview.
  - Audit update (as of 2026-03-09, richer todo template): `apps/fretboard/src/scaffold/templates.rs` (`todo_template_main_rs`) remains intentionally explicit for now. The retained `Model<T>` graph is carrying the richer teaching goal itself (selector deps across nested row models, filter coordination, and query invalidation keyed by tracked state), so this surface should be treated as the third-rung selector/query baseline rather than as the next local-state migration target.
  - Review update (as of 2026-03-09): `INVALIDATION_LOCAL_STATE_REVIEW.md` now records a focused review of `apps/fret-cookbook/examples/simple_todo_v2_target.rs`, `apps/fret-cookbook/examples/query_basics.rs`, `apps/fret-cookbook/examples/commands_keymap_basics.rs`, and `apps/fret-cookbook/examples/form_basics.rs`. Conclusion: on real medium surfaces, tracked writes already rerender without explicit `notify()`, query-trigger invalidation still belongs to the explicit render-time path, and command/keymap plus cross-field form handlers are often the runtime contract; the next plausible ergonomics move is therefore still **not** another invalidation helper, and `AFA-postv1-003` now looks like a much narrower keyed-list/payload-row question rather than a general medium-surface need.
  - Policy draft update (as of 2026-03-09): `NOTIFY_POLICY_DECISION_DRAFT.md` now makes the recommendation explicit: keep `notify()` as a public low-level escape hatch, keep tracked writes as the boring default rerender path, and do not spend near-term API budget on another generic invalidation helper unless a new medium-surface contradiction appears.
  - Short-rule update (as of 2026-03-09): `INVALIDATION_DEFAULT_RULES.md` now records the execution-facing split directly: use `on_action_notify_local_*` for straightforward single-local tracked writes, keep `on_action_notify_models::<A>(...)` for coordinated/root-owned writes, keep `on_action_notify_transient::<A>(...)` for transient runtime effects, and treat explicit `notify()` / render-time invalidation as escape hatches only when the real effect boundary lives outside the tracked write.
  - Ownership-class update (as of 2026-03-09): the remaining default-path
    `on_action_notify_models::<A>(...)` usage is now classified into three explicit buckets:
    coordinated writes, command/keymap ownership, and cross-field form ownership. This narrows the
    remaining policy question from “do we need another invalidation helper?” to “does a surface
    really fall outside those three ownership classes?”.
  - Gate update (as of 2026-03-09): `tools/gate_no_notify_in_default_teaching_surfaces.py` now locks the default ladder surfaces plus scaffold templates against explicit `cx.notify(...)` / `host.notify(...)`, and `tools/pre_release.py` runs that gate with the other teaching-surface policy checks.
- [x] AFA-postv1-006 Audit model-centered widget contracts that still leak into gallery/reference surfaces.
  - Goal: separate true widget contract pressure from snippet-level authoring choices before designing new helper APIs.
  - Evidence target: audit note + one snippet cleanup that proves an existing uncontrolled path is sufficient.
  - Status (as of 2026-03-08): `docs/workstreams/action-first-authoring-fearless-refactor-v1/MODEL_CENTERED_WIDGET_CONTRACT_AUDIT.md` now classifies the remaining pressure into text-value widgets (`Input` / `Textarea`), overlay/disclosure roots that already have uncontrolled paths (`Collapsible`, `Popover`, `Dialog`, `AlertDialog`), and intentional outward-sync surfaces. `apps/fret-ui-gallery/src/ui/snippets/collapsible/demo.rs` now uses `Collapsible::default_open(false)` directly, proving that this specific gallery case was a snippet choice rather than a missing runtime/local-state capability. `ecosystem/fret-ui-shadcn/src/text_value_model.rs` plus the matching `LocalState<String>` impl in `ecosystem/fret/src/view.rs` now land the narrow text bridge, so post-v1 teaching surfaces can call `Input::new(&local_text)` / `Textarea::new(&local_text)` directly without widening the helper surface.

- [x] AFA-postv1-007 Publish a post-v1 gap analysis against the current Rust-first UI best-practice target.
  - Goal: keep docs honest about what is already solved versus what still blocks GPUI/Zed-level authoring density.
  - Evidence target: align the workstream proposal, v2 golden path, and todo golden-path docs on the same ?current baseline vs north-star? statement.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_AUTHORING_V2_PROPOSAL.md`, `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`, and `docs/examples/todo-app-golden-path.md` now explicitly record that the narrow text bridge and keyed-list default path both landed; the remaining gap is framed as productization/default-path clarity first, then keyed-list / payload-row handler ergonomics, and only then a macro re-evaluation.

- [x] AFA-postv1-009 Publish a hard-delete gap analysis for the remaining compatibility surfaces.
  - Goal: distinguish true legacy cleanup debt from advanced/interop surfaces we may keep on purpose.
  - Evidence target: one written inventory that names the blockers, evidence anchors, and required preconditions before broader hard deletes.
  - Status (as of 2026-03-08): `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md` now records the four main blockers: `App::ui(...)` / `ui_with_hooks(...)`, `run_native_with_compat_driver(...)`, `ViewCx::use_state::<T>()` as a user-visible alias, and public `CommandId`-first widget contracts.

- [x] AFA-postv1-010 Publish an app-entry policy decision draft for `view::<V>()` vs `.ui(...)`.
  - Goal: turn the hard-delete blocker into one explicit policy choice rather than an open-ended cleanup note.
  - Evidence target: a short decision draft with rationale, staged execution, and exit criteria before deprecation starts.
  - Status (as of 2026-03-10): `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md` now records the final pre-release outcome: `view::<V>()` / `view_with_hooks::<V>(...)` are the only app-entry path on `fret`, and `App::ui*` has been hard-deleted from the facade.

- [x] AFA-postv1-011 Inventory the remaining in-tree `App::ui*` callers against the app-entry policy draft.
  - Goal: turn the app-entry policy into a concrete migration table instead of a generic “later cleanup” note.
  - Evidence target: one inventory that classifies each current `ui(...)` / `ui_with_hooks(...)` caller as `migrate-to-view`, `move-lower-level`, or `keep-temporarily`.
  - Status (as of 2026-03-08): `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_CALLER_INVENTORY.md` now classifies the current in-tree callers; the present conclusion is that almost all of them are `migrate-to-view` debt rather than evidence that closure-root app entry should remain a co-equal long-term surface.
  - Progress update (as of 2026-03-08; builder-then-run updated on 2026-03-11): `apps/fret-examples/src/imui_hello_demo.rs`, `apps/fret-examples/src/imui_response_signals_demo.rs`, `apps/fret-examples/src/chart_declarative_demo.rs`, and `apps/fret-examples/src/node_graph_demo.rs` have already moved to `view::<...>()?.run()`; Batch A is therefore complete and no longer depends on closure-root `App::ui(...)`.
  - Hook-path update (as of 2026-03-08): `apps/fret-examples/src/assets_demo.rs` now uses `view_with_hooks::<AssetsDemoView>(...)` with the same `on_event(...)` hook, establishing the first Batch B proof that driver callbacks can stay while the default entry still converges on the view runtime.
  - Viewport-hook update (as of 2026-03-08): `apps/fret-examples/src/embedded_viewport_demo.rs` now uses `view_with_hooks::<EmbeddedViewportDemoView>(...)`, and `ecosystem/fret/src/interop/embedded_viewport.rs` now provides `EmbeddedViewportView` so retained viewport recording can compose directly with `ViewWindowState<V>` instead of forcing a closure-root wrapper state.
  - Frame-hook update (as of 2026-03-08): `apps/fret-examples/src/image_heavy_memory_demo.rs` now uses `view_with_hooks::<ImageHeavyMemoryView>(...)`, proving a demo that only needs `record_engine_frame(...)` also does not require `ui_with_hooks(...)`.
  - Editor-proof update (as of 2026-03-09): `apps/fret-examples/src/imui_editor_proof_demo.rs` now uses `view_with_hooks::<ImUiEditorProofView>(...)`, so Batch B is complete and the remaining `ui_with_hooks(...)` callers are now limited to Batch C interop demos.
  - Batch C update (as of 2026-03-09): `apps/fret-examples/src/external_texture_imports_demo.rs` now uses `view_with_hooks::<ExternalTextureImportsView>(...)`, reducing the remaining closure-root app-entry risk to the two platform video-import demos.
  - Windows-video update (as of 2026-03-09): `apps/fret-examples/src/external_video_imports_mf_demo.rs` now uses `view_with_hooks::<ExternalVideoImportsMfView>(...)`, so the remaining app-entry migration risk is now isolated to the AVF/macOS demo.
  - AVF-video update (as of 2026-03-09): `apps/fret-examples/src/external_video_imports_avf_demo.rs` now uses `view_with_hooks::<ExternalVideoImportsAvfView>(...)`, so Batch C is complete and closure-root app-entry work is now purely deprecation/cleanup rather than remaining demo migration.
  - Final example update (as of 2026-03-09; builder-then-run updated on 2026-03-11): `apps/fret-examples/src/imui_floating_windows_demo.rs`, `apps/fret-examples/src/imui_node_graph_demo.rs`, and `apps/fret-examples/src/imui_shadcn_adapter_demo.rs` now use `view::<...>()?.run()`, so there are no in-tree example/demo callers left on `App::ui*`.

- [x] AFA-postv1-012 Start staged deprecation for closure-root app entry on the `fret` facade.
  - Goal: make the policy decision visible in code and lock the default docs path while leaving a removal window for downstream users.
  - Evidence target: deprecated `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` plus a narrow gate that keeps `view::<V>()` as the only default path in `ecosystem/fret` README/rustdoc.
  - Status (as of 2026-03-10): `ecosystem/fret/src/app_entry.rs` no longer exports the closure-root app-entry methods, `ecosystem/fret/src/lib.rs` and `ecosystem/fret/README.md` describe only `view::<V>()` / `view_with_hooks::<V>(...)` as the `fret` app-entry path, and `tools/gate_fret_builder_only_surface.py` plus `authoring_surface_policy_tests` now lock that hard-delete.

- [x] AFA-postv1-013 Publish a hard-delete execution checklist for the remaining post-v1 compat surfaces.
  - Goal: turn the blocker inventory into an ordered, landable cleanup sequence instead of leaving the next phase as an implicit policy discussion.
  - Evidence target: one workstream note that classifies stage order, per-surface status, and exit criteria for app-entry closure surfaces, compat runner entry points, `use_state`, and command-first widget contracts.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_EXECUTION_CHECKLIST.md` now defines the execution order, exit criteria, and immediate next action, and `HARD_DELETE_GAP_ANALYSIS.md` now points to it as the operational follow-up.

- [x] AFA-postv1-013c Publish a hard-delete status matrix that separates waiting/deferred seams from the next real cleanup track.
  - Goal: make it obvious which remaining blockers are still code migration work versus policy-held advanced/non-default seams.
  - Evidence target: one matrix that classifies `App::ui*`, compat runner, `use_state`, and command-first widget contracts by readiness and next action.
  - Status (as of 2026-03-10): `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_STATUS_MATRIX.md` now records the narrowed endgame after the app-entry closure lane was removed pre-release: compat runner and `use_state` are retained advanced/non-default seams, and the remaining command-first widget family is the main implementation-scoped cleanup track.

- [x] AFA-postv1-013f Hard-delete `App::ui*` from `fret` before the first public release.
  - Goal: stop carrying a split app-entry mental model on the public facade when no published compatibility promise exists yet.
  - Evidence target: remove `App::{ui, ui_with_hooks, run_ui, run_ui_with_hooks}` from `ecosystem/fret/src/app_entry.rs`, update README/rustdoc/tests/gates, and close the workstream docs as a completed lane rather than a waiting deprecation window.
  - Status (as of 2026-03-10): the closure-root app-entry methods are gone from `fret`, `tools/gate_fret_builder_only_surface.py` now forbids their return, and the app-entry workstream docs now treat the lane as a completed pre-release hard delete.

- [x] AFA-postv1-014 Publish a caller inventory for `run_native_with_compat_driver(...)`.
  - Goal: replace vague “plot/interop demos still use it” language with a concrete in-tree family breakdown before deciding whether the compat runner should be kept, quarantined, or removed.
  - Evidence target: one inventory note that classifies current callers by family and states whether they look like migration leftovers or intentional advanced interop surfaces.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_CALLER_INVENTORY.md` now records the current callers (plot/chart retained-driver demos, low-level renderer/asset demos, and advanced shell demos), and `HARD_DELETE_EXECUTION_CHECKLIST.md` now uses that inventory to move Stage 3 from “unscoped” to “partially scoped”.

- [x] AFA-postv1-015 Publish a policy decision draft for `run_native_with_compat_driver(...)`.
  - Goal: decide whether the compat runner is actually a near-term hard-delete candidate or an intentionally retained advanced interop seam.
  - Evidence target: one decision note that states the recommended product stance and the conditions under which future quarantine/removal would become reasonable.
  - Status (as of 2026-03-12): `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMPAT_DRIVER_POLICY_DECISION_DRAFT.md` now records the landed quarantine outcome: keep the capability, but only under `fret::advanced::interop::run_native_with_compat_driver(...)`; `HARD_DELETE_EXECUTION_CHECKLIST.md` now treats Stage 3 as executed quarantine instead of a draft.

- [x] AFA-postv1-015b Finish compat-runner wording alignment in the workstream docs.
  - Goal: close the gap between the policy draft and the execution docs so Stage 3 is no longer blocked on basic wording drift.
  - Evidence target: the policy draft, hard-delete checklist, and gap analysis all describe `run_native_with_compat_driver(...)` as a retained advanced low-level interop seam and no longer leave README/rustdoc alignment as open work.
  - Status (as of 2026-03-09): `COMPAT_DRIVER_POLICY_DECISION_DRAFT.md`, `HARD_DELETE_EXECUTION_CHECKLIST.md`, and `HARD_DELETE_GAP_ANALYSIS.md` now all treat the remaining work as policy/quarantine follow-up rather than docs wording cleanup.

- [x] AFA-postv1-015c Publish a quarantine playbook for future compat-runner surface reduction.
  - Goal: record the exact quarantine-first patch shape for `run_native_with_compat_driver(...)` so
    future facade reduction does not have to improvise its sequencing.
  - Evidence target: one execution note that states preconditions, patch shape, validation, release
    wording, and abort conditions for moving the compat runner behind an explicit advanced boundary.
  - Status (as of 2026-03-12): `COMPAT_DRIVER_QUARANTINE_PLAYBOOK.md` is now the historical
    execution record for the landed move to
    `fret::advanced::interop::run_native_with_compat_driver(...)`.

- [x] AFA-postv1-015d Close the compat-runner default-surface gate gap.
  - Goal: make the source-facing policy enforceable so first-contact docs cannot drift toward
    `run_native_with_compat_driver(...)` while the seam remains intentionally retained.
  - Evidence target: one narrow gate that requires advanced/non-default wording on the `fret`
    facade surface and fails if approved first-contact docs start recommending compat-runner entry.
  - Status (as of 2026-03-09): `tools/gate_compat_runner_default_surface.py` now enforces that
    split, `tools/pre_release.py` runs it in the canonical policy suite, and
    `SOURCE_ALIGNMENT_AUDIT_2026-03-09.md` records the source-vs-docs alignment result.

- [x] AFA-postv1-016 Publish a caller inventory for `use_state::<T>()`.
  - Goal: replace vague “a few starter/reference snippets still use it” language with a concrete in-tree breakdown before deciding whether `use_state` should be kept, deprecated, or repointed later.
  - Evidence target: one inventory note that classifies current `use_state` callers by starter/reference/API-substrate role and distinguishes runtime callers from contract docs.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_CALLER_INVENTORY.md` now records that `hello`, the `hello` scaffold template, the gallery action-first snippet, `overlay_basics`, and `imui_action_basics` have all moved to `use_local*`; direct runtime/teaching-surface callers are now cleared and `use_state` remains only as explicit runtime/API substrate plus migration/contract documentation.

- [x] AFA-postv1-017 Publish a policy decision draft for `use_state::<T>()`.
  - Goal: decide whether `use_state` is a near-term deprecation target or an intentionally retained explicit raw-model seam.
  - Evidence target: one decision note that states the default teaching rule, the explicit low-level rule, and the preconditions for any future deprecation.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/USE_STATE_POLICY_DECISION_DRAFT.md` now recommends keeping `use_state` for now as an explicit raw-model hook while `use_local*` remains the only default local-state teaching path; `HARD_DELETE_EXECUTION_CHECKLIST.md` now reflects Stage 4 with starter/reference cleanup complete, `docs/examples/todo-app-golden-path.md` no longer lists `use_state` as a generic default hook, and all current first-contact/reference code paths now follow that policy in code.

- [x] AFA-postv1-017b Publish a surface playbook for future `use_state` reduction.
  - Goal: record the exact keep-vs-deprecate execution path for `use_state` so the repo can revisit
    the raw-model seam later without reopening the first-contact migration argument.
  - Evidence target: one execution note that states preconditions, option split, patch shape,
    validation, release wording, and abort conditions for either retaining or shrinking the public
    `use_state` surface.
  - Status (as of 2026-03-09): `USE_STATE_SURFACE_PLAYBOOK.md` now records that sequence, and the
    policy/checklist/status summary docs all point to it as the concrete follow-up only if the repo
    later chooses to revisit the public raw-model seam.

- [x] AFA-postv1-018 Add a narrow default-path gate against reintroducing `use_state::<T>()`.
  - Goal: keep first-contact docs/templates/examples on `use_local*` after the current cleanup lands, without banning explicit raw-model usage in advanced/runtime code.
  - Evidence target: one narrow source/docs/template gate or equivalent repo-local assertion that fails if the approved first-contact surfaces drift back to `use_state`.
  - Status (as of 2026-03-09): `tools/gate_no_use_state_in_default_teaching_surfaces.py` now guards the approved first-contact/reference files (`hello`, `overlay_basics`, `imui_action_basics`, the gallery action-first snippet, and `docs/examples/todo-app-golden-path.md`), `apps/fretboard/src/scaffold/templates.rs` keeps template output covered by unit assertions, and the canonical cross-platform runner `tools/pre_release.py` now runs the new gate alongside the other teaching-surface policy checks.

- [x] AFA-postv1-019 Publish a command-first widget contract audit for the remaining post-v1 blocker families.
  - Goal: replace vague “public `CommandId`-first widget contracts still remain” language with a concrete family split and a landable migration order.
  - Evidence target: one workstream note that distinguishes command-catalog surfaces from app-facing builder APIs and recommends which widget families should gain action-first aliases first.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md` now classifies the remaining pressure into already-aligned dual-surface widgets (`Button`, `CommandItem`), menu-family blockers (`DropdownMenu*`, `ContextMenu*`, `Menubar*`), medium-risk app-facing surfaces (`NavigationMenu*`, `BreadcrumbItem`, Material `Snackbar`), and a staged alias-first migration order.
  - Docs alignment update (as of 2026-03-09): `docs/component-author-guide.md` now explicitly teaches the split between action-first public builder naming and command-pipeline/keymap lowering, so this track no longer has a stale top-level authoring doc leaking the old `CommandId`-first mental model.

- [x] AFA-postv1-020 Land the first action-first alias pass for command-shaped widget builders.
  - Goal: prove that public builder naming can converge on the action-first story without rewriting command-centric internals.
  - Evidence target: at least one low-risk family (`BreadcrumbItem`, `NavigationMenu*`, or Material `Snackbar`) gains an action-first alias and docs/examples prefer it.
  - Status (as of 2026-03-09): `ecosystem/fret-ui-shadcn/src/breadcrumb.rs` now exposes `BreadcrumbItem::action(...)`, `ecosystem/fret-ui-shadcn/src/navigation_menu.rs` now exposes `NavigationMenuLink::action(...)` and `NavigationMenuItem::action(...)`, `ecosystem/fret-ui-material3/src/snackbar.rs` now exposes `Snackbar::action_id(...)` / `Snackbar::action_command(...)`, the navigation-menu gallery snippets (`demo.rs`, `docs_demo.rs`, `rtl.rs`) plus the Material3 snackbar gallery snippet now prefer the action-first spelling, and the first navigation-menu/material3 coverage uses the aliases as the default public path.
  - Gate update (as of 2026-03-09): `tools/gate_material3_snackbar_default_surface.py` now keeps `apps/fret-ui-gallery/src/ui/snippets/material3/snackbar.rs` on `action_id(...)`, and the canonical cross-platform runner `tools/pre_release.py` runs that narrow policy check.
  - Extension update (as of 2026-03-15): the same alias pattern now also covers more default-facing app widgets outside the earlier navigation/material slice: `BreadcrumbLink` / `BreadcrumbLinkBuild`, `InputGroupButton`, `Item`, `PaginationLink` / `PaginationPrevious` / `PaginationNext` / `PaginationLinkBuild`, `TableRow` / `TableRowBuild`, and the `Sidebar*` clickable surfaces (`SidebarTrigger`, `SidebarRail`, `SidebarGroupAction`, `SidebarMenuAction`, `SidebarMenuSubButton`, `SidebarMenuButton`) all expose `action(...)` while still lowering through the same command pipeline.
  - Input follow-up (as of 2026-03-15): the default-facing text-input family now also exposes
    action-first submit/cancel aliases instead of teaching only command-shaped setters:
    `ecosystem/fret-ui-shadcn/src/input.rs` (`Input::submit_action(...)` /
    `cancel_action(...)`), `ecosystem/fret-ui-shadcn/src/input_group.rs`
    (`InputGroup`, `InputGroupInput`, `InputGroupTextarea`), and
    `ecosystem/fret-ui-shadcn/src/sidebar.rs` (`SidebarInput`) all keep the same underlying
    command pipeline while letting Enter/Escape bindings match the repo's action-first mental
    model.
  - Material3 follow-up (as of 2026-03-15): the broader default-facing Material3 pressable family
    now also exposes `action(...)` on `Button`, `Fab`, `IconButton`, `IconToggleButton`,
    `Checkbox`, `Switch`, `Radio`, `AssistChip`, `SuggestionChip`, `FilterChip`, and
    `InputChip`; `ecosystem/fret-ui-material3/src/lib.rs` now locks that surface with a focused
    source-policy test.
  - Material3 secondary-slot follow-up (as of 2026-03-15): `FilterChip` and `InputChip` now also
    expose `trailing_action(...)` for their trailing icon pressables, so the remaining Material3
    alias debt is no longer concentrated on those secondary stable slots.
  - Source-policy update (as of 2026-03-15): `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs` now locks those default-facing clickable surfaces on the action-first alias wording so future cleanup does not silently regress them back to command-shaped-only naming.
  - Toast follow-up (as of 2026-03-15): `ecosystem/fret-ui-shadcn/src/sonner.rs` now exposes `ToastMessageOptions::action_id(...)` / `action_command(...)` / `cancel_id(...)` / `cancel_command(...)`, and the primary Sonner gallery demo now prefers `action_id(...)` / `cancel_id(...)` while keeping the same toast dispatch internals.
  - Button teaching-surface follow-up (as of 2026-03-15): the remaining first-party `Button`
    demos/snippets that bind stable action IDs now also prefer `.action(...)` instead of the
    legacy `.on_click(...)` spelling across curated cookbook/examples, `components_gallery`, and
    the ui-gallery driver/snippet surfaces (`chrome`, `settings_sheet`, `view_cache`, code-editor
    previews, input file browse, and the deprecated toast redirect card).
  - Button teaching-surface gate update (as of 2026-03-15):
    `tools/gate_button_action_default_surfaces.py` now locks that curated first-party slice to the
    action-first builder spelling, and `tools/pre_release.py` runs the gate with the rest of the
    default-surface policy suite.
  - Gallery action-alias follow-up (as of 2026-03-15): the remaining ui-gallery snippets/pages for
    widgets that already expose stable action slots now also prefer `.action(...)` across
    `SidebarMenuButton` navigation, `NavigationMenuItem` link-component, `Button` link-render,
    `BreadcrumbLink`, `Item`, and the `Pagination*` family.
  - Gallery action-alias gate update (as of 2026-03-15):
    `tools/gate_gallery_action_alias_default_surfaces.py` now keeps that curated ui-gallery slice
    off legacy `.on_click(...)`, and `tools/pre_release.py` runs the gate alongside the other
    default-surface policy checks.

- [x] AFA-postv1-021 Land the menu-family action-first alias pass for `ContextMenu*` / `Menubar*`.
  - Goal: remove the largest remaining command-shaped builder inconsistency from the default component surface without changing menu routing internals.
  - Evidence target: `ContextMenuItem` / `ContextMenuCheckboxItem` / `ContextMenuRadioItem{Spec,}` and `MenubarItem` / `MenubarCheckboxItem` / `MenubarRadioItem{Spec,}` all gain `action(...)` aliases, and command-audit docs record the phase-2 progress.
  - Status (as of 2026-03-09): those aliases now exist in `ecosystem/fret-ui-shadcn/src/context_menu.rs` and `ecosystem/fret-ui-shadcn/src/menubar.rs`; the broader gallery menu surface now also prefers `action(...)` across the main context-menu and menubar snippets (`basic`, `usage`, `demo`, `checkboxes`/`checkbox`, `radio`, `destructive`, `groups`, `icons`, `shortcuts`, `sides`, `submenu`, `rtl`, `parts`, `with_icons`), and `COMMAND_FIRST_WIDGET_CONTRACT_AUDIT.md` records the pass while keeping command-centric routing/storage unchanged.
  - Follow-up update (as of 2026-03-09): app/internal helper surfaces now also start converging on the same spelling: `ecosystem/fret-ui-shadcn/src/text_edit_context_menu.rs`, `ecosystem/fret-workspace/src/tab_strip/mod.rs`, and the focused keyboard/dismiss tests for context menu / menubar now use `action(...)` as the default builder name while still routing through the same command pipeline.
  - Dropdown follow-up (as of 2026-03-09): `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs` now exposes `DropdownMenuItem::action(...)` / `trailing_action(...)`, `DropdownMenuCheckboxItem::action(...)`, `DropdownMenuRadioItemSpec::action(...)`, and `DropdownMenuRadioItem::action(...)`; the primary dropdown-menu gallery snippets (`basic`, `demo`) plus overlay preview surfaces now also prefer `action(...)`.
  - Gate update (as of 2026-03-09): `tools/gate_menu_action_default_surfaces.py` now keeps the primary ui-gallery dropdown-menu / context-menu / menubar teaching snippets plus the overlay preview menu surfaces on `action(...)`, and `tools/pre_release.py` runs that narrow policy check alongside the other default-surface gates.
  - Curated internal follow-up (as of 2026-03-09): `ecosystem/fret-workspace/src/tab_strip/overflow.rs` and `ecosystem/fret-genui-shadcn/src/resolver/overlay.rs` now also prefer `action(...)` / `trailing_action(...)` for their stable action-bearing menu rows, and `tools/gate_menu_action_curated_internal_surfaces.py` keeps that explicit internal/app-facing residue slice on the same spelling without broadening the policy to every advanced/internal menu surface.
  - Intentional-retention inventory update (as of 2026-03-09): `COMMAND_FIRST_INTENTIONAL_SURFACES.md` now records that the main remaining command-shaped surfaces are command palette/catalog (`ecosystem/fret-ui-shadcn/src/command.rs`), `DataTable` business-table wiring (`ecosystem/fret-ui-shadcn/src/data_table.rs` plus gallery demos), compat/conformance tests, and out-of-scope callback widgets; the practical rule is to stop broad residue chasing unless a new default-facing leak appears.

- [x] AFA-postv1-021b Publish a retained-seam decision draft for the remaining command-first lane.
  - Goal: stop treating the remaining command-shaped surfaces as one generic migration bucket and
    record which ones are permanent mechanism/catalog seams versus intentionally retained
    advanced/internal residue.
  - Evidence target: one decision note that states the split classification, reopen triggers, and
    the rule for keeping this lane in maintenance mode.
  - Status (as of 2026-03-09): `COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md` now records that
    split, and the hard-delete index/status/checklist/endgame summary docs all point to it as the
    current boundary rule for the command-first retained-seam lane.

- [x] AFA-postv1-021c Audit component-author surfaces against the current action-first story.
  - Goal: verify that crate-entry docs and top-level component author guidance do not still leak an
    outdated command-first/default-path mental model.
  - Evidence target: one short audit note plus any minimal README alignment needed to close a real
    author-entry gap.
  - Status (as of 2026-03-09): `AUTHOR_SURFACE_ALIGNMENT_AUDIT_2026-03-09.md` now records the
    audit result, and `ecosystem/fret-ui-material3/README.md` now gives Material3 the same kind of
    author-facing entrypoint that shadcn already had.

- [x] AFA-postv1-021d Publish a blunt execution outlook for the remaining endgame surfaces.
  - Goal: stop treating every retained seam as equally likely to be deleted and record the repo's
    current best forecast for real removal vs long-term retention.
  - Evidence target: one short note that classifies `App::ui*`, compat runner, `use_state`, and
    command-first retained seams by expected outcome and reopen trigger.
  - Status (as of 2026-03-09): `ENDGAME_EXECUTION_OUTLOOK_2026-03-09.md` now records that forecast,
    and the high-level endgame summary/index/milestones docs all point to it as the blunt
    execution-level reading of the current cleanup track.

- [x] AFA-postv1-022 Audit `DataTable` authoring as a separate post-v1 surface instead of treating it as more primitive `Table` builder cleanup.
  - Goal: determine whether the remaining density pressure is really another `build(...)` / `into_element(cx)` problem or a higher-level business-table recipe/productization problem.
  - Evidence target: one short audit note that classifies `DataTable` against primitive `Table`, the current gallery demos, and the v2 golden-path story.
  - Status (as of 2026-03-09): `docs/workstreams/action-first-authoring-fearless-refactor-v1/DATA_TABLE_AUTHORING_AUDIT.md` now concludes that primitive `Table` builder-first cleanup is largely closed for the current pass, while `DataTable` remains a denser business-table integration surface whose pressure comes from state/output/toolbar wiring rather than missing leaf builders.

- [x] AFA-postv1-023 Decide whether the repo needs a curated `DataTable` default recipe or golden-path note.
  - Goal: decide whether business-table authoring should stay purely advanced/reference-only or gain one deliberately scoped “default” recipe without widening the generic helper surface.
  - Guardrail: any curated recipe must keep `TableState`, `output_model`, row-key strategy, and action/command boundaries visible rather than hiding them behind a macro or opaque helper.
  - Status (as of 2026-03-09): decision made. The repo keeps a docs-first `DataTable` default recipe in `DATA_TABLE_GOLDEN_PATH.md`, treating `DataTable` as a medium/advanced business-table surface rather than a first-contact example, and defers helper/macro expansion until that curated recipe still proves insufficient in real demos.
  - Evidence update (as of 2026-03-09): `apps/fret-ui-gallery/src/ui/snippets/data_table/default_demo.rs` and `apps/fret-ui-gallery/src/ui/pages/data_table.rs` now provide the first curated default-recipe gallery slice aligned with that note.
  - Gate update (as of 2026-03-09): `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery-data-table-default-recipe-smoke.json --dir target/fret-diag/ui-gallery-data-table-default-recipe --timeout-ms 240000 --pack --ai-packet --launch -- cargo run -p fret-ui-gallery` passes locally and emits bounded artifacts for the curated recipe slice.

- [x] AFA-postv1-024 Write a “current best practice vs v2 target” note.
  - Goal: make the repo’s current recommended writing style explicit, and separate real remaining ergonomics gaps from already-closed migration tracks.
  - Status (as of 2026-03-09): `V2_BEST_PRACTICE_GAP.md` now states that v1 migration is effectively complete, command-first residue is in maintenance mode, and the next high-value work is productization + tracked-write/invalidation ergonomics rather than more broad API churn.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_BEST_PRACTICE_GAP.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`

- [x] AFA-postv1-025 Tighten default/comparison/advanced ingress wording.
  - Goal: make the repo entry points repeat the same ladder/taxonomy so users do not infer the default path from scattered examples.
  - Status (as of 2026-03-09): `DEFAULT_PATH_PRODUCTIZATION.md` now records the current convergence snapshot, `README.md`, `docs/first-hour.md`, `docs/crate-usage-guide.md`, and `docs/ui-ergonomics-and-interop.md` now repeat the same ladder/taxonomy at the repo-root, first-hour, crate-guidance, and ergonomics-guidance entry points, `docs/examples/README.md` states that any unlabeled surface is a docs bug, `apps/fret-cookbook/README.md` plus `apps/fret-cookbook/EXAMPLES.md` frame cookbook as a follow-up layer after the `hello` / `simple-todo` ladder, `apps/fret-ui-gallery/README.md` repeats the same “use after the ladder” order explicitly, and `ecosystem/fret/README.md` points back to the same ladder instead of presenting the facade README as the canonical example host.
  - North-star clarification (as of 2026-03-10): this productization/document pass does **not**
    change the v2 target. GPUI/Zed-style authoring/runtime remains the primary north-star;
    gpui-component is at most a secondary source of selected builder/productization cues.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/DEFAULT_PATH_PRODUCTIZATION.md`
    - `README.md`
    - `docs/first-hour.md`
    - `docs/crate-usage-guide.md`
    - `docs/ui-ergonomics-and-interop.md`
    - `docs/examples/README.md`
    - `apps/fret-cookbook/README.md`
    - `apps/fret-cookbook/EXAMPLES.md`
    - `apps/fret-ui-gallery/README.md`
    - `ecosystem/fret/README.md`

- [x] AFA-postv1-026 Close one medium-surface builder seam in the `Alert` family.
  - Goal: prove that post-v1 builder-density work can stay narrow and evidence-driven instead of reopening broad helper expansion.
  - Status (as of 2026-03-09): `Alert::build(...)` and `AlertAction::build(...)` now keep alert content/action composition on the builder path, and the first real surfaces (`form_basics`, `assets_reload_epoch_basics`, ui-gallery alert snippets) have migrated without widening the default authoring story beyond one component family.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/alert.rs`
    - `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
    - `apps/fret-cookbook/examples/form_basics.rs`
    - `apps/fret-cookbook/examples/assets_reload_epoch_basics.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/alert/basic.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/alert/action.rs`

- [x] AFA-postv1-027 Close one medium-surface builder seam in the `ScrollArea` family.
  - Goal: keep the post-v1 builder-density pass focused on one runtime-owned root seam instead of reopening general helper expansion.
  - Status (as of 2026-03-09): `ScrollArea::build(...)` now keeps viewport children on the builder path while preserving the existing axis / scrollbar / viewport-test-id configuration seam; the first real surfaces (`markdown_and_code_basics`, `async_playground_demo`, ui-gallery scroll-area demo) have migrated without broadening the default authoring story.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/scroll_area.rs`
    - `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
    - `apps/fret-cookbook/examples/markdown_and_code_basics.rs`
    - `apps/fret-examples/src/async_playground_demo.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/scroll_area/demo.rs`

- [x] AFA-postv1-028 Close one medium-surface builder seam in the `Field` family.
  - Goal: reduce repeated form/field composition landings (`FieldSet` / `FieldGroup` / `Field`) without widening the helper surface beyond one already-dense family.
  - Status (as of 2026-03-09): `FieldSet::build(...)`, `FieldGroup::build(...)`, and `Field::build(...)` now keep field-family children on the builder path, and the first real surfaces (`ui-gallery` field `input`, field `fieldset`, and form `demo`) have migrated without changing runtime ownership boundaries.
  - Evidence:
    - `ecosystem/fret-ui-shadcn/src/field.rs`
    - `ecosystem/fret-ui-shadcn/tests/ui_builder_smoke.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/field/input.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/field/fieldset.rs`
    - `apps/fret-ui-gallery/src/ui/snippets/form/demo.rs`

- [x] AFA-postv1-008 Decide the next additive API move after the local-collection comparison target.
  - Goal: determine whether the next density win should come from **no new API at all yet** (productize the default path first) or from a narrow keyed-list / payload-row ergonomics pass, without re-expanding the helper surface.
  - Evidence target: keep `V2_GOLDEN_PATH.md`, `POST_V1_AUTHORING_V2_PROPOSAL.md`, and onboarding docs aligned on the same next-step order before any new helper is promoted.
  - Status note: `apps/fret-cookbook/examples/simple_todo_v2_target.rs` now proves `LocalState<Vec<_>>` is already viable for a small keyed todo list and no longer needs per-row checkbox models either; label/control parity for snapshot/action discrete controls is now closed, so the remaining question is which write/event ergonomics buys the next density win.
  - Follow-up (as of 2026-03-08, skill alignment): the same decision tree is now captured in `.agents/skills/fret-shadcn-source-alignment/references/public-surface-parity.md` and `.agents/skills/fret-app-ui-builder/references/mind-models/mm-widget-state-surfaces.md`, so future `Switch` / `Toggle`-style audits can reuse one parity rubric before adding more app-side helpers.
  - Update (as of 2026-03-08, switch source alignment): `ecosystem/fret-ui-shadcn/src/switch.rs` now mirrors the checkbox-style narrow snapshot path via `Switch::from_checked(...)` plus `action(...)` / `action_payload(...)`, and `apps/fret-cookbook/examples/commands_keymap_basics.rs` now uses that path for both its local allow-command toggle and the disabled panel-open indicator.
  - Update (as of 2026-03-08, toggle source alignment): `ecosystem/fret-ui-shadcn/src/toggle.rs` now lands the same narrow snapshot/action pattern via `Toggle::from_pressed(...)` plus `action(...)` / `action_payload(...)`, and `apps/fret-cookbook/examples/toggle_basics.rs` now demonstrates the path on a minimal view-local example.
  - Update (as of 2026-03-08, action-only control parity): `crates/fret-ui/src/declarative/host_widget/event/pointer_region.rs` plus `ecosystem/fret-ui-kit/src/primitives/control_registry.rs` now let label-forwarded pointer activation record command payload/source metadata, and `Checkbox` / `Switch` / `Toggle` all register command-backed `control_id` entries for snapshot/action paths. That closes the shared discrete-widget parity gap, so the next density decision can focus on keyed-list / payload-row handler ergonomics vs broader invalidation ergonomics rather than on label forwarding.
  - Update (as of 2026-03-09): the keyed-list/default local-collection target is now considered closed enough for planning purposes. The recommended next step is to **productize the existing path first** (`hello` → `simple-todo` → `todo`, default/comparison/advanced taxonomy, helper visibility), and to defer any new default helper until that narrative is stable.
  - Update (as of 2026-03-09, business-table scope): `DATA_TABLE_AUTHORING_AUDIT.md` now records that `DataTable` should be treated as a separate productization/reference-surface question rather than as evidence that primitive `Table` builder-first cleanup is still incomplete.
  - Status (as of 2026-03-09): decision made. The post-v1 execution order is now fixed in docs:
    productize the current default path first, revisit local-state / invalidation only where real
    medium-surface evidence still remains after that pass, then re-evaluate narrow keyed-list /
    payload-row handler ergonomics, and keep macros last and optional.

- [x] AFA-postv1-005 Evaluate narrow authoring macros that reduce repeated child/list boilerplate without introducing a full `rsx!`-style DSL as the default surface.
  - Decision (2026-03-16 closeout): do not open a macro lane from this workstream.
  - Reason: the current shared-evidence set does not justify macro promotion, and the repo keeps
    macros optional and last rather than letting them become the default authoring answer.
  - Guardrail: if macros are reconsidered later, they must live on a separate future lane and must
    not hide action identity, key context, or cache-boundary semantics.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/SHARED_SURFACE_EVIDENCE_MATRIX_2026-03-16.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/POST_V1_EXECUTION_CHECKLIST.md`
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`

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
        - Gate: `tools/gate_no_stack_in_ui_gallery_shell.py`
      - As needed: shadcn/genui crates (only when they block teaching-surface convergence)
  - Done: hard delete legacy stack helpers once internal implementations are migrated.
    - Gate: `tools/gate_no_public_stack_in_ui_kit.py`
    - Note: a handful of “host type inference” edge cases need an explicit anchor.
      Preferred: annotate the closure argument type (e.g. `ui::v_flex(|cx: &mut ElementContext<'_, App>| ...)`).
      Alternative: turbofish (e.g. `ui::v_flex::<App, _, _>(...)`).
  - Done: cookbook examples no longer use `stack::hstack/vstack` authoring helpers; the repo teaches
    one layout authoring surface for demos (`fret-ui-kit::ui::*` builders).
    - Gate: `tools/gate_no_stack_in_cookbook.py`
  - Done: examples no longer use `stack::hstack/vstack` authoring helpers.
    - Gate: `tools/gate_no_stack_in_examples.py`
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
  - Simple demo status: `hello_template_main_rs` is now close to the intended golden path (typed actions + `ui::children!` + one model-update helper), and `simple_todo_template_main_rs` now keeps its palette/filter helpers on the same builder-first path while also serving as the generated starter default for keyed lists (`LocalState<Vec<_>>` + payload row actions + snapshot checkbox rendering) without extra template-only `.into_element(cx)` cliffs.
  - Medium demo status: `hello_counter_demo`, `query_demo`, and `query_async_tokio_demo` now use the `LocalState<T>` prototype for view-local state and the default `value_*` read suffix; `hello_counter_demo` now also keeps its generic step read on `LocalState::read_in(...)`, while the query demos plus cookbook `query_basics` now read query resources from the `QueryHandle<T>` side via `TrackedStateExt`, and the remaining `ElementContext` query teaching surfaces (`async_playground_demo`, `markdown_demo`, scaffold query tip, and the markdown MathJax/Mermaid helpers) now use the same `handle.layout_query(cx)` shape via `QueryHandleWatchExt`. Cookbook `hello_counter`, `query_basics`, `commands_keymap_basics`, `text_input_basics`, `date_picker_basics`, `form_basics`, `simple_todo`, `drop_shadow_basics`, `markdown_and_code_basics`, `assets_reload_epoch_basics`, `virtual_list_basics`, `theme_switching_basics`, `icons_and_assets_basics`, and `customv1_basics` now extend that path into the cookbook surface. `text_input_basics` is now the first direct `Input::new(&LocalState<String>)` teaching surface, while `date_picker_basics` remains the first explicit `local.clone_model()` bridge for a controlled non-text widget API (`DatePicker::new_controllable(...)`). `drop_shadow_basics` is the first pure toggle-only renderer example on the same bridge path, `markdown_and_code_basics` now shows the split in one page (`Textarea` uses the direct text bridge while `ToggleGroup::single` / `Switch` still keep their model-centered contracts), `assets_reload_epoch_basics` is the first explicit local-state + render-time host/runtime escape-hatch example, `virtual_list_basics` is the first virtualization hybrid where the collection and scroll handle stay explicit but the surrounding controls move to local state, `theme_switching_basics` is the first explicit local theme-selection + render-time theme-application example, `icons_and_assets_basics` is the matching asset-demo version where the reload trigger becomes local state but asset reload synchronization stays render-time, `customv1_basics` is the matching renderer/effect hybrid where `enabled` / `strength` move to local state while effect registration, capability checks, and effect-layer plumbing stay explicit, `form_basics` is the first multi-field validation/reset example that keeps coordination on `on_action_notify_models`, and `simple_todo` remains the intentional keyed dynamic-list comparison/reference case (`use_local*` for draft / `next_id`, explicit `Model<Vec<_>>` for the collection). `apps/fret-examples/src/todo_demo.rs` plus `apps/fretboard/src/scaffold/templates.rs` now show the default keyed-list path instead (`LocalState<Vec<_>>` + payload row actions + `Checkbox::from_checked(...)`), while the remaining scaffold todo templates, cookbook `async_inbox_basics`, and `fret-genui-shadcn` data-table resolver continue to carry the current card/table-focused builder-first experiment. The remaining inventory is now classified in `TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`, with Queue A + Queue B cleared, the cookbook/template closure separated from advanced/reference surfaces in `apps/fret-examples` and `apps/fret-ui-gallery`, and the remaining explicit-model cases documented as advanced/runtime-bound. `apps/fret-examples/src/embedded_viewport_demo.rs` has now moved its view-local `size_preset` knob to `use_local_with(...)` + `on_action_notify_local_set(...)`; the demo still remains intentionally advanced because the embedded viewport models, forwarded input state, and render-time interop effects are runtime-bound and not treated as post-v1 default-path blockers. The remaining recurring noise classes are:
    1. explicit tracked-state escape hatches that still need raw `watch(...)`, `observe()`, or `revision()` after the default `state.layout(cx).value_*` / `state.paint(cx).value_*` path landed,
    2. broader composite helpers plus the wider family of single-child wrappers still remain outside the modern `ui::children!` / `push_ui()` pipeline, even though the current card/table builder paths, the first `TableCell::build(child)` sample, and the first dropdown-menu trigger/root builder path now round-trip through the generic `.ui()` patch surface,
    3. explicit transient scheduling for App-only effects (`take_transient_on_action_root` + `with_query_client`).
  - Recommended next phase:
    - use a todo-like view-owned collection as the next comparison target for `use_local*` / invalidation ergonomics,
    - keep `on_action*` / `on_activate*` as the current closure story until that collection/shared-state boundary is better understood,
    - prefer template/doc guidance first for transient/App-effect patterns,
    - re-evaluate keyed-list / payload-row ergonomics and macros only after one more round of template/demo authoring feedback.
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
    4. Widget-local `dispatch` / `dispatch_payload` / `listen` sugar now exists for activation-only surfaces; the remaining event-density pressure is mostly around shortcut naming and the broader widget event taxonomy.
  - Recommendation:
    - close v1 as successful on architecture + migration + default teaching surface,
    - track density/ergonomics work in a separate post-v1 phase,
    - do not add more tiny helpers until another round of template/demo evidence shows repeated pressure.
- Helper visibility policy snapshot (as of 2026-03-16):
  - Default teaching surface: `cx.actions().locals/models/transient/payload(...)` at the root/view layer, plus widget-local `.action(act::Save)`, `.action_payload(act::Remove, payload)`, and `.listen(|host, acx| ...)` for activation-only surfaces. The explicit `.dispatch::<A>()` / `.dispatch_payload::<A>(payload)` aliases remain available, but they are no longer the shortest recommended wording.
  - Advanced/reference surface: raw `cx.on_action_notify(...)`, `cx.on_payload_action_notify(...)`, and redraw-oriented `on_activate_request_redraw*` helpers.
  - Follow-up shrink (as of 2026-03-17): the zero-proof single-model raw aliases (`on_action_notify_model_update`, `on_action_notify_model_set`, `on_action_notify_toggle_bool`) plus the raw non-notify `cx.on_action(...)` / `cx.on_payload_action(...)` hooks and raw availability hook are deleted from `AppUiRawActionNotifyExt`; advanced code now either stays on raw `on_action_notify(...)` / `on_payload_action_notify(...)` or uses the grouped `cx.actions().models::<A>(...)` / `locals::<A>(...)` / `availability::<A>(...)` surfaces.
  - Promotion rule: do not promote additional helpers into README/templates/first-hour docs unless at least two real demos/templates need the same shape and the generic defaults are clearly noisier.
  - Remaining intentional advanced cookbook cases are now explicitly cookbook-only host-side categories: `toast_basics` (imperative Sonner host integration), `async_inbox_basics::Start` (dispatcher/inbox scheduling), and `undo_basics::Undo`/`Redo` (history traversal + RAF effect).
  - `fret-examples` and ui-gallery teaching pages/snippets are now on the zero-exception path for raw `cx.on_action_notify::<...>` while scaffold templates keep equivalent unit-test assertions; `async_playground_demo::ToggleTheme` and the query demos stay on `on_action_notify_models` / `on_action_notify_transient` with render-time side effects where needed, `embedded_viewport_demo` now uses `use_local_with(...)` + `on_action_notify_local_set(...)` for its view-local size preset while keeping viewport interop/render-time effects explicit, and `hello_counter_demo` plus both query demos remain the intentional `use_local` prototypes that still keep the default `on_action_notify_models` action surface for coordinated writes.
- [x] AFA-postv1-022 Start event-surface unification under `cx.actions()`.
  - Goal: move default widget-side activation glue onto the same grouped action namespace as root/view action registration, without rewriting runtime dispatch.
  - Evidence target: a design note, grouped `action` / `action_payload` / `dispatch` / `dispatch_payload` / `listen` helpers on `AppUiActions`, and at least one docs/source-policy update that treats them as the preferred widget-local glue surface.
  - Evidence:
    - `docs/workstreams/action-first-authoring-fearless-refactor-v1/EVENT_SURFACE_UNIFICATION_DESIGN.md`
    - `ecosystem/fret/src/view.rs` (`AppUiActions::{action, action_payload, dispatch, dispatch_payload, listen, listener}`)
    - `docs/crate-usage-guide.md`
    - `docs/authoring-golden-path-v2.md`
- [x] AFA-postv1-024 Add only thin activatable-widget sugar after the docs/template rewrite proves it is still needed.
  - Goal: if activation-only surfaces still feel materially noisier than `.action(...)`-capable widgets, add a single app-facing extension trait in `ecosystem/fret` rather than another helper family.
  - Landed target: `widget.action(act::Save)`, `widget.action_payload(act::Remove, payload)`, and `widget.listen(|host, acx| ...)` for types that already expose `on_activate(...)`, with `widget.dispatch::<A>()` / `widget.dispatch_payload::<A>(payload)` kept as explicit aliases.
  - Guardrails: do not replace `.action(...)` / `.action_payload(...)` as the default for widgets that already have stable action slots; do not add `click` / `submit` / `listener_notify` style helper taxonomies.
  - Evidence:
    - `ecosystem/fret/src/view.rs` (`AppActivateSurface`, `AppActivateExt`)
    - `ecosystem/fret/src/lib.rs` (`fret::app::{AppActivateSurface, AppActivateExt}` explicit bridge lane; `fret::app::prelude::*` intentionally omits `AppActivateExt as _`)
    - `docs/first-hour.md`
  - Follow-up evidence (as of 2026-03-15):
    - `ecosystem/fret/src/lib.rs` now explicitly exports `fret::app::AppActivateExt` alongside
      `fret::app::AppActivateSurface`, and the facade self-tests lock that shape
    - `ecosystem/fret/src/view.rs` dropped the no-op `cx` marker parameter from
      `AppActivateExt::{action, action_payload, dispatch, dispatch_payload, listen}`, so the
      default activation sugar is now `widget.action(act::Save)`,
      `widget.action_payload(act::Remove, payload)`, and `widget.listen(|host, acx| ...)`, with
      `dispatch` aliases kept as the explicit wording instead of carrying an unused context argument
    - follow-up tightening on 2026-03-16 moved `AppActivateExt` fully off
      `fret::app::prelude::*`; activation-only call sites now import
      `use fret::app::AppActivateExt as _;` explicitly, while ordinary action-capable snippets
      such as `apps/fret-ui-gallery/src/ui/snippets/command/action_first_view.rs` stay on native
      widget `.action(...)` slots without the bridge import
    - follow-up shrink on 2026-03-16 also landed `UiCxActionsExt` for extracted helper functions,
      so first-party helper-heavy snippets can stay on `cx.actions().models::<A>(...)` instead of
      falling back to raw `on_action*` calls or bridge `.listen(...)`
    - `ecosystem/fret/src/view.rs` no longer keeps
      `fret_ui_shadcn::facade::Button` or `fret_ui_shadcn::facade::SidebarMenuButton` on the bridge
      table; those shadcn widgets now stay on their native `.action(...)` / `.action_payload(...)`
      / widget-owned `.on_activate(...)` surface, and the AI widgets that already expose native
      `.action(...)` / widget-owned `.on_activate(...)` also stay off the bridge table
    - `ecosystem/fret/src/view.rs` source-policy also locks the exclusion boundary: no
      `AppActionCxSurface` family, and no `AppActivateSurface` impls for typed payload/context
      callbacks like `fret_ui_ai::{Attachment, FileTreeAction, MessageBranch, QueueItemAction, Suggestion, Test}`.
    - selected activation-only UI Gallery snippets (`ai/{chat_demo,persona_demo,prompt_input_referenced_sources_demo,reasoning_demo,task_demo,transcript_torture}`,
      `drawer/demo`, `data_table/{basic_demo,default_demo,rtl_demo}`,
      `scroll_area/nested_scroll_routing`, `sidebar/{demo,controlled,mobile,rtl}`,
      `sonner/{demo,extras,usage,position}`) now prefer `.listen(...)`
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
      (`selected_activation_snippets_prefer_app_activate_listen`) locks that default teaching lane
    - `apps/fret-ui-gallery/tests/ui_authoring_surface_default_app.rs`
      now also passes the routed-page/copyable-root cleanup that was exposed by the sugar rewrite,
      including `material3_overlay_snippets_prefer_uncontrolled_copyable_roots` and
      `render_doc_page_callers_land_the_typed_doc_page_explicitly`
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
