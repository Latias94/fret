# Action-First Authoring + View Runtime (Fearless Refactor v1) — Migration Guide

Last updated: 2026-03-01

This guide is intentionally practical: it describes how to migrate in-tree demos and ecosystem code
in small, reviewable slices.

---

## 1) Migration sequence (recommended)

1) Migrate **event identities** first (commands → actions).
2) Migrate authoring loop (MVU → view runtime) only after action IDs are stable.
3) Add/upgrade gates (tests + diag scripts) while migrating, not after.

Rationale:

- action-first is the cross-frontend convergence seam (declarative + imui + GenUI),
- view runtime is the “authoring density” win, but it builds on stable action semantics.

---

## 2) Commands → Actions (authoring-level refactor)

Target outcome:

- UI triggers and keybindings reference an `ActionId` (string-visible, stable).

Migration steps:

1) Introduce action IDs for the existing command IDs (prefer keeping the same string).
2) Update UI widgets to bind `.action(...)` rather than `.on_click(cmd_id)` where appropriate.
3) Update handler registration:
   - `on_command` hooks become `on_action` hooks (or a unified handler path).

---

## 3) MVU → View runtime (minimal refactor)

Target outcome:

- the “one file demo” does not need:
  - `MessageRouter`,
  - `enum Msg`,
  - the `update(msg)` boilerplate.

Migration steps:

1) Move state into:
   - app-owned models (recommended for shared state), or
   - view-local state slots for simple demos.
2) Replace:
   - `msg.cmd(Msg::X)` with `act::X` action references.
3) Replace `update(...)` with `cx.on_action(...)` handlers.
4) Replace manual “force refresh” hacks with:
   - `cx.notify()` and/or
   - selector/query hooks that carry proper dependency observation.

---

## 4) imui alignment (imui widgets dispatch actions)

Target outcome:

- imui widgets can trigger the same action IDs as declarative UI.

Migration steps:

1) Add a `UiWriter` helper to emit an action trigger (no string glue).
2) Ensure imui outputs stable `test_id`/semantics for diag scripts.
3) Keep policy in ecosystem components, not in `fret-imui`.

---

## 5) GenUI alignment (spec bindings reuse action IDs)

Target outcome:

- GenUI specs and Rust UI can share action IDs and metadata where appropriate.

Migration steps:

1) Standardize action ID naming conventions (namespace + `.v1` suffix).
2) Expose action metadata to the GenUI inspector surfaces (optional v1).
3) Keep GenUI guardrails: do not allow specs to dispatch arbitrary actions without catalog approval.

