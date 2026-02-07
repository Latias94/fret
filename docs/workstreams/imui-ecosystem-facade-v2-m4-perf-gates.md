# imui Ecosystem Facade v2 M4 Performance Gates

Status: Accepted (M4 locked)
Last updated: 2026-02-06

This note locks the v2 M4 scope that turns perf guidance into concrete review and test gates.

Related:

- `docs/workstreams/imui-ecosystem-facade-v2.md`
- `docs/workstreams/imui-ecosystem-facade-v2-todo.md`
- `docs/workstreams/imui-ecosystem-facade-perf-v1.md`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/tests/imui_perf_guard_smoke.rs`
- `apps/fret-examples/src/virtual_list_stress_demo.rs`

---

## 1) Review Checklist (from v1 perf guide)

Every wrapper or adapter change in `fret_ui_kit::imui` must pass this checklist:

1) Stable identity for dynamic collections

- Use keyed identity (`push_id`, `for_each_keyed`, model-keyed rows/items).
- Reject index-only identity for reorderable/insertable/removable collections.

2) Allocation-light hot paths

- Avoid per-frame materialization of owned collections for wrapper internals.
- Avoid per-frame formatting/allocation in loops unless unavoidable and justified.
- Prefer borrowing (`&[T]`, `&str`) over cloning into temporary buffers.

3) Single-sourced interaction state

- Keep complex session state in canonical component/local element state.
- Keep facade response signals edge-style and transient.

4) Intentional geometry stabilization

- Use last-frame bounds for popup/floating choreography.
- Do not add retry loops or compensating per-frame allocations for first-frame misses.

5) Bounded work for large lists

- Large list/table/tree examples must keep virtualization + keyed identity on by default.

---

## 2) Automated Gates Added in M4

M4 introduces a lightweight regression gate file:

- `ecosystem/fret-ui-kit/tests/imui_perf_guard_smoke.rs`

Current guards:

1) `select_wrapper_does_not_materialize_items_vec_each_frame`

- Verifies `select_model_ex` does not regress to `items.to_vec()` materialization.
- Anchor: `ecosystem/fret-ui-kit/src/imui.rs` (`select_model_ex`).

2) `virtual_list_stress_demo_keeps_keyed_virtualization_path`

- Verifies `virtual_list_stress_demo` still uses `VirtualListOptions::new` and
  `virtual_list_keyed_with_layout` as default path.
- Anchor: `apps/fret-examples/src/virtual_list_stress_demo.rs`.

These are fast source-level guards meant to fail early in CI for known high-risk regressions.

---

## 3) M4 Gate Commands

Recommended local/CI commands:

1) Perf guard smoke

- `cargo nextest run -p fret-ui-kit --features imui --test imui_perf_guard_smoke`

2) Existing response + adapter contracts

- `cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --test imui_adapter_seam_smoke`

---

## 4) Completion Mapping

- `IMUIECO2-perf-040`: section 1 (review checklist locked from perf guide rules).
- `IMUIECO2-perf-041`: section 2 (targeted hot-path regression gate).
- `IMUIECO2-perf-042`: section 2 (virtualized + keyed large-list default evidence).
