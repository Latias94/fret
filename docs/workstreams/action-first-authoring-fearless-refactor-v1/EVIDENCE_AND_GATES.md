# Action-First Authoring + View Runtime (Fearless Refactor v1) — Evidence and Gates

Last updated: 2026-03-02

This file defines what “done” means beyond subjective UX feel.

The guiding principle: if we change the authoring/routing substrate, we must lock outcomes with
small, deterministic gates (tests and scripted diagnostics), not just manual QA.

---

## 1) Required evidence anchors (update as code lands)

- ADR (actions): `docs/adr/0307-action-registry-and-typed-action-dispatch-v1.md`
- ADR (view runtime): `docs/adr/0308-view-authoring-runtime-and-hooks-v1.md`
- Workstream: `docs/workstreams/action-first-authoring-fearless-refactor-v1/DESIGN.md`

### Implementation anchors (as of 2026-03-02)

Action identity + typed unit actions:

- `crates/fret-runtime/src/action.rs` (`ActionId`, `TypedAction`)
- `ecosystem/fret/src/actions.rs` (`fret::actions!` macro, `ActionHandlerTable` + unit test)

View runtime (v1):

- `ecosystem/fret/src/view.rs` (`View`, `ViewCx`, `use_state`/`use_selector`/`use_query`)
- `ecosystem/fret/src/app_entry.rs` (`App::run_view`)

Pointer-trigger authoring integration (v1 still dispatches through the command pipeline):

- `ecosystem/fret-ui-shadcn/src/button.rs` (`Button::action`)
- `ecosystem/fret-ui-kit/src/command.rs` (`action_is_enabled`, `dispatch_action_if_enabled`)
- `ecosystem/fret-ui-kit/src/declarative/action_hooks.rs` (`pressable_dispatch_action_if_enabled`)
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` (example adoption: view runtime + keymap + action availability gating)
- `apps/fret-cookbook/examples/hello.rs` (example adoption: view runtime + action-first button + handler registration)
- `apps/fret-cookbook/examples/overlay_basics.rs` (example adoption: view runtime + modal barrier gate)

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

Notes:

- Tests must rely on stable selectors (`test_id`/role/name), not pixel coordinates.
- The script output must record the resolved `ActionId` (or command/action identity) for each step.

Current scripts (as of 2026-03-02):

- `tools/diag-scripts/cookbook/commands-keymap-basics/cookbook-commands-keymap-basics-shortcut-and-gating.json`
- `tools/diag-scripts/cookbook/hello/cookbook-hello-click-count.json`
- `tools/diag-scripts/cookbook/overlay-basics/cookbook-overlay-basics-modal-barrier-shortcut-gating.json`

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
