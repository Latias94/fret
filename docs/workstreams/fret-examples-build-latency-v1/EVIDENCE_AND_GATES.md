# Fret Examples Build Latency v1 - Evidence And Gates

Status: active

## Smallest Repro

```text
python tools/gate_imui_shadcn_adapter_sortable_table_source.py
```

## Gate Set

```text
python tools/gate_imui_shadcn_adapter_sortable_table_source.py
python tools/gate_imui_shadcn_adapter_control_discoverability_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/gate_table_source_policy.py
python tools/gate_examples_source_tree_policy.py
python tools/gate_fret_examples_imui_split_source.py
cargo check -p fret-examples-imui --bin imui_shadcn_adapter_demo --jobs 1
cargo check -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1
cargo check -p fret-examples --lib --jobs 1
python tools/check_workstream_catalog.py
git diff --check
```

## Baseline Evidence

- Prior cold `cargo nextest run -p fret-examples imui_shadcn_adapter_demo_keeps_sortable_table_diag_gate --no-fail-fast` took about 5m42s locally because it compiled the monolithic examples crate for a source-marker check.
- Prior cold `cargo build -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1` took about 13m20s locally because the demo bin links through the full examples library.

## Current Evidence

- Passed: `python tools/gate_imui_shadcn_adapter_sortable_table_source.py` (now covers the adapter
  facade/entrypoint markers previously guarded by
  `imui_shadcn_adapter_demo_prefers_root_fret_imui_facade_lane`).
- Passed: `python tools/gate_imui_shadcn_adapter_control_discoverability_source.py`.
- Passed: `python tools/gate_imui_facade_teaching_source.py`.
- Passed: `python tools/gate_table_source_policy.py`.
- Passed: `python tools/gate_examples_source_tree_policy.py`.
- Passed: `python tools/gate_fret_examples_imui_split_source.py`.
- Passed: `python -m py_compile tools/gate_imui_shadcn_adapter_sortable_table_source.py`.
- Passed: `python -m py_compile tools/gate_imui_shadcn_adapter_control_discoverability_source.py`.
- Passed: `python -m py_compile tools/gate_imui_facade_teaching_source.py`.
- Passed: `python -m py_compile tools/gate_table_source_policy.py`.
- Passed: `python -m py_compile tools/gate_examples_source_tree_policy.py`.
- Passed: `python -m py_compile tools/gate_fret_examples_imui_split_source.py`.
- Passed: `cargo check -p fret-examples-imui --bin imui_shadcn_adapter_demo --jobs 1`.
- Passed: `cargo check -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1`.
- Checked: `cargo tree -p fret-examples-imui -e normal` has no `fret-examples v...` dependency
  entry; the direct IMUI proof crate does not depend on the monolithic examples crate.
- Passed: `cargo check -p fret-examples --lib --jobs 1`.
- Passed: `python tools/check_workstream_catalog.py`.
- Passed: `git diff --check`.
- Recorded: `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
  with the remaining source-policy test count and migration candidates.
- Current count after the source-tree policy migration: 281 `include_str!` occurrences and 122
  Rust `#[test]` functions remain in `apps/fret-examples/src/lib.rs`.
- Noted: `python tools/check_workstream_state.py` is not usable as a lane-local gate yet because
  existing historical workstream state files fail the global strict validator before this lane is
  evaluated.

## Evidence Anchors

- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`
- `apps/fret-examples-imui/src/bin/imui_shadcn_adapter_demo.rs`
- `tools/gate_examples_source_tree_policy.py`
- `tools/gate_fret_examples_imui_split_source.py`
- `apps/fret-demo/Cargo.toml`
- `apps/fret-demo/src/bin/imui_shadcn_adapter_demo.rs`
- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json`
- `tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json`
- `docs/workstreams/fret-examples-build-latency-v1/M1_SOURCE_POLICY_AUDIT_2026-04-29.md`
- `docs/workstreams/fret-examples-build-latency-v1/M2_DEMO_BUILD_SPLIT_DECISION_2026-04-29.md`
- `Cargo.toml`
