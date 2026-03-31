# imui Ecosystem Facade v2 M2 Adapter Seam Contract and Template

Status: Accepted (M2 locked)
Last updated: 2026-02-06

This note defines the v2 M2 contract for adapter seams and a reusable external adapter template.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-v2/imui-ecosystem-facade-v2-m1-response-stability.md`
- `ecosystem/fret-ui-kit/src/imui/adapters.rs`

---

## 1) Minimal Adapter Seam Contract

Canonical seam fields:

1) Identity in
- Adapter call sites must provide a stable identity key for dynamic/reorderable scenarios.
- In immediate wrappers, this is implemented through `UiWriterImUiFacadeExt::push_id(...)`.

2) Signal reporter out
- Adapter wrappers may emit a post-render `AdapterSignalRecord` via a lightweight callback.
- The report carries the canonical `ResponseExt` so callers can inspect interaction outcomes.

3) Optional metadata
- Adapter reports may include geometry and focus-restore hints:
  - `rect`
  - `focus_restore_target`

Contract anchors:

- `AdapterSignalMetadata`
- `AdapterSignalRecord`
- `AdapterSeamOptions`
- `report_adapter_signal(...)`

All are defined in `ecosystem/fret-ui-kit/src/imui/adapters.rs`.

---

## 2) External Adapter Template (Author Once, Adapt Many)

Recommended template shape:

- input:
  - `&mut impl UiWriterImUiFacadeExt<H>`
  - stable identity key
  - canonical widget data/model inputs
  - `AdapterSeamOptions`
- body:
  - call `ui.push_id(identity_key, |ui| canonical_wrapper(...))`
  - call `report_adapter_signal(...)`
- output:
  - return canonical `ResponseExt`

Template advantages:

- keeps canonical policy/state machine single-sourced,
- provides explicit seam reporting without extra runtime coupling,
- keeps adapters thin and auditable.

---

## 3) Non-shadcn Example Landed

Historical note:

- Early versions of this seam note pointed at built-in sample wrappers in
  `ecosystem/fret-ui-kit/src/imui/adapters.rs`.
- That public sample pair was later deleted so the module could remain contract-only.
- The current external-style examples now live in test scaffolds instead:
  - `ecosystem/fret-ui-kit/tests/imui_adapter_seam_smoke.rs`
  - `ecosystem/fret-ui-kit/tests/imui_external_adapter_example.rs`

---

## 4) Validation Expectations

M2 adapter seam changes must include:

- at least one compile/test anchor for adapter surface wiring,
- TODO evidence updates,
- no duplicated complex state machines in adapter code.

Reference smoke test:

- `ecosystem/fret-ui-kit/tests/imui_adapter_seam_smoke.rs`

---

## 5) M2 Completion Mapping

- `IMUIECO2-adapter-020`: section 1 + code seam types/functions.
- `IMUIECO2-adapter-021`: section 2.
- `IMUIECO2-adapter-022`: section 3 + adapter example functions.
