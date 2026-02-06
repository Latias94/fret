# imui Ecosystem Facade v2 M1 Response Stability Contract

Status: Accepted (M1 locked)
Last updated: 2026-02-06

This note defines the v2 M1 contract for `ResponseExt` stabilization and graduation policy.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v2-m0-contracts.md`
- `ecosystem/fret-authoring/src/lib.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`

---

## 1) Response Surface Partition (M1)

### 1.1 Tier A - Shared stable core (authoring contract)

These fields remain the canonical shared contract in `fret_authoring::Response`:

- `hovered`
- `pressed`
- `focused`
- `clicked`
- `changed`
- `rect`

M1 decision:

- Tier A remains the only immediate graduation target for `fret-authoring` in v2.
- Existing API and semantics are treated as stable.

### 1.2 Tier B - Facade-stable (`fret_ui_kit::imui::ResponseExt`)

These signals are considered stable at the facade layer for v2 (not yet moved to
`fret-authoring`):

- `secondary_clicked`
- `double_clicked`
- `context_menu_requested`
- `drag_started`
- `dragging`
- `drag_stopped`
- `drag_delta`
- `drag_total`

M1 decision:

- Tier B semantics should be preserved across v2 except through explicit versioned migration steps.

### 1.3 Tier C - Experimental facade signals

These signals remain experimental due to higher coupling to timing/session internals:

- `long_pressed`
- `press_holding`
- `context_menu_anchor`
- `id` (as a raw identity exposure field)

M1 decision:

- Tier C may evolve in v2, but changes must be tracked in the v2 TODO with rationale.

---

## 2) Graduation Path (v2)

1) Keep Tier A in `fret-authoring` as the stable cross-frontend minimal contract.
2) Keep Tier B in facade scope in v2 while gathering behavior evidence from demos/tests/diag.
3) Re-evaluate Tier B graduation only in M5 readiness review (or later) when:
   - interaction semantics are proven stable,
   - no layering regressions are introduced,
   - compile smoke and regression gates remain green.

---

## 3) Compatibility and Deprecation Policy

### 3.1 Stable core policy (Tier A)

- Additive changes only by default.
- Renames/removals require a deprecation window and migration notes.
- Semantic changes require explicit workstream entry and compatibility rationale.

### 3.2 Facade-stable policy (Tier B)

- Avoid silent semantic drift.
- If behavior must change, prefer new opt-in options or new methods over in-place mutation.
- Keep tests covering clear-on-read/event timing contracts for edge signals.

### 3.3 Experimental policy (Tier C)

- Evolution is allowed, but must include:
  - TODO update,
  - test or diag evidence where applicable,
  - migration note when call-site behavior changes.

---

## 4) Compile Boundary Smoke Expectations

M1 requires compile smoke checks that validate the boundary between:

- shared minimal contract (`fret_authoring::Response`), and
- facade extension surface (`fret_ui_kit::imui::ResponseExt`).

Reference test anchor:

- `ecosystem/fret-ui-kit/tests/imui_response_contract_smoke.rs`

---

## 5) M1 Completion Mapping

- `IMUIECO2-resp-010`: section 1.
- `IMUIECO2-resp-011`: sections 2 and 3.
- `IMUIECO2-test-012`: section 4 + compile smoke test file.
