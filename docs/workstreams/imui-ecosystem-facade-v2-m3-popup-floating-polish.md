# imui Ecosystem Facade v2 M3 Popup/Select and Floating Coexistence Polish

Status: Accepted (M3 locked)
Last updated: 2026-02-08

This note captures the v2 M3 polish scope for popup/select choreography and floating coexistence.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m2-adapter-seam.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-imui/src/lib.rs`
- `tools/diag-scripts/imui-float-window-select-popup-coexistence.json`

---

## 1) Popup/Select Choreography (adapter-first)

M3 decision:

- `select_model_ex` now uses popup choreography instead of click-to-cycle behavior.
- Trigger activation opens a popup menu anchored to trigger bounds.
- Option activation updates the model, emits `changed` once, and closes popup.
- Option rows are exposed as radio-menu items to reuse canonical menu interaction policy.
- Popups are kept alive by `begin_popup_*` calls; if a popup scope is opened but not rendered on a
  subsequent frame (e.g. the triggering widget disappears), it will auto-close to avoid stale
  "open" state resurfacing later.

Implementation anchors:

- `select_model_ex` popup flow in `ecosystem/fret-ui-kit/src/imui.rs`
- stable popup scope id generation per select trigger
- option test id generation: `{trigger_test_id}.option.{index}`

---

## 2) Focus Restore and Dismiss Consistency

M3 decision:

- Select popup uses existing popup/menu overlay contracts for dismiss/focus handling.
- `Escape` closes popup and focus returns to trigger, matching menu/popup expectations.

Verification anchors:

- `select_popup_escape_closes_and_restores_trigger_focus` in `ecosystem/fret-imui/src/lib.rs`

---

## 3) Floating + Popup Coexistence Regression Script

M3 adds a scripted diagnostics gate that exercises:

- floating window drag,
- select popup open/pick/escape close,
- context-menu open/close in the same floating window.

Script anchor:

- `tools/diag-scripts/imui-float-window-select-popup-coexistence.json`

Demo wiring anchor:

- `apps/fret-examples/src/imui_floating_windows_demo.rs` (`imui-float-demo.select` trigger and option ids)

---

## 4) Fearless Refactor Note

This M3 change intentionally breaks prior select trigger semantics (cycle-on-click) in favor of
popup-based behavior. Pre-release policy allows this when it improves layering and interaction
consistency.

---

## 5) M3 Completion Mapping

- `IMUIECO2-float-030`: section 1 + `select_model_ex` implementation.
- `IMUIECO2-float-031`: section 2 + focus/dismiss test.
- `IMUIECO2-test-032`: section 3 + diag script and demo wiring.
