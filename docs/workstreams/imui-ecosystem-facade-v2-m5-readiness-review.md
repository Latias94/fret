# imui Ecosystem Facade v2 M5 Readiness Review

Status: Accepted (M5 locked; v2 complete)
Last updated: 2026-02-06

This note closes v2 by re-auditing layering ownership and publishing the v2 stable subset with
explicit deferrals.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m1-response-stability.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m2-adapter-seam.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m3-popup-floating-polish.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m4-perf-gates.md`

---

## 1) Layering Re-audit (`IMUIECO2-scope-050`)

### 1.1 `fret-authoring` remains the cross-frontend minimal contract

Decision:

- Keep `fret-authoring` as the small, policy-light contract crate.
- Keep only `Response` + `UiWriter` as shared authoring primitives in v2.

Evidence:

- `ecosystem/fret-authoring/src/lib.rs` (`Response`, `UiWriter`)
- `ecosystem/fret-authoring/Cargo.toml` (minimal dependency surface)

### 1.2 `fret-imui` remains minimal frontend (no policy takeover)

Decision:

- Keep `fret-imui` as immediate authoring frontend and identity/output composition utility.
- Keep richer policy/interaction wrappers out of `fret-imui` runtime dependencies.

Evidence:

- `ecosystem/fret-imui/src/lib.rs` (crate-level note + `ImUi`/`imui` frontend surface)
- `ecosystem/fret-imui/Cargo.toml` (runtime deps: `fret-authoring`, `fret-ui`)

### 1.3 `fret-ui-kit` keeps optional `imui` facade ownership

Decision:

- Keep the immediate ergonomics facade in `fret-ui-kit::imui` behind an optional feature.
- Keep third-party-friendly default by not forcing `imui` dependencies unless requested.

Evidence:

- `ecosystem/fret-ui-kit/Cargo.toml` (`imui = ["dep:fret-authoring"]`, `default = []`)
- `ecosystem/fret-ui-kit/src/lib.rs` (`#[cfg(feature = "imui")] pub mod imui;`)
- `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt`, `ResponseExt`)

### 1.4 Adapter seam remains thin and canonical-policy-first

Decision:

- Keep adapter helpers as delegation/reporting seams; do not duplicate heavy state machines.

Evidence:

- `ecosystem/fret-ui-kit/src/imui/adapters.rs`
- `ecosystem/fret-ui-kit/tests/imui_adapter_seam_smoke.rs`

Conclusion:

- v2 layering remains aligned with the original constraints: one runtime, one canonical policy
  source, multiple authoring/frontdoor surfaces.

---

## 2) Published v2 Stable Subset (`IMUIECO2-docs-051`)

### 2.1 Stable cross-frontend subset (Tier A)

- `fret_authoring::Response`:
  - `hovered`, `pressed`, `focused`, `clicked`, `changed`, `rect`
- `fret_authoring::UiWriter`:
  - `with_cx_mut`, `add`, `extend`, `mount`, `keyed`

Reference:

- `docs/workstreams/imui-ecosystem-facade-v2-m1-response-stability.md`
- `ecosystem/fret-authoring/src/lib.rs`

### 2.2 Facade-stable subset in v2 (Tier B)

- `ResponseExt` stable-at-facade signals:
  - `secondary_clicked`, `double_clicked`, `context_menu_requested`
  - `drag_started`, `dragging`, `drag_stopped`, `drag_delta`, `drag_total`
- Adapter seam contracts:
  - `AdapterSignalMetadata`, `AdapterSignalRecord`, `AdapterSeamOptions`,
    `report_adapter_signal(...)`

Reference:

- `docs/workstreams/imui-ecosystem-facade-v2-m1-response-stability.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m2-adapter-seam.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/adapters.rs`

### 2.3 Interaction-stable subset proven in v2

- Popup/select choreography uses popup menu flow (not click-to-cycle).
- Focus restore + dismiss rules are validated for popup/context menu paths.
- In-window floating window + popup coexistence has scripted diagnostics evidence.

Reference:

- `docs/workstreams/imui-ecosystem-facade-v2-m3-popup-floating-polish.md`
- `tools/diag-scripts/imui-float-window-select-popup-coexistence.json`
- `ecosystem/fret-imui/src/lib.rs`

### 2.4 Perf guard subset adopted

- Hot wrapper regression guard (no select `items.to_vec()` per frame).
- Keyed virtualization path guard for large-list reference demo.

Reference:

- `docs/workstreams/imui-ecosystem-facade-v2-m4-perf-gates.md`
- `ecosystem/fret-ui-kit/tests/imui_perf_guard_smoke.rs`

---

## 3) Explicit Deferrals (Post-v2 Backlog)

1) Tier C response signals remain experimental

- `long_pressed`, `press_holding`, `context_menu_anchor`, raw `id` remain facade-experimental.

2) Non-docking OS-window promotion remains deferred

- Keep docking-owned multi-window promotion path as the only supported route.

3) Rich text/code-editor interaction engines remain external

- Keep text editing depth in code-editor ecosystem workstreams and integrate through adapters.

References:

- `docs/workstreams/imui-ecosystem-facade-v2-m1-response-stability.md`
- `docs/workstreams/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/code-editor-ecosystem-v1.md`

---

## 4) Readiness Gate Commands (v2 close)

- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --test imui_adapter_seam_smoke --test imui_perf_guard_smoke`
- `cargo nextest run -p fret-imui select_model_reports_changed_once_after_option_pick select_popup_escape_closes_and_restores_trigger_focus`

---

## 5) Completion Mapping

- `IMUIECO2-scope-050`: section 1.
- `IMUIECO2-docs-051`: sections 2 and 3.

With this review locked, `imui ecosystem facade v2` milestones M0-M5 are complete.
