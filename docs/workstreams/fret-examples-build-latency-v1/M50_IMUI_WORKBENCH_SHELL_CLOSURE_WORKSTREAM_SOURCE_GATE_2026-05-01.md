# Fret Examples Build Latency v1 - M50 IMUI Workbench Shell Closure Workstream Source Gate - 2026-05-01

Status: complete

## Decision

Move the source-only IMUI workbench shell closure source-policy package out of the monolithic
`fret-examples` Rust unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p1_workbench_shell_proof_matrix`
- `immediate_mode_workstream_freezes_the_p1_shell_diag_smoke_minimum`
- `immediate_mode_workstream_freezes_the_p1_default_workbench_assembly_decision`

## Behavior

The IMUI workstream source gate now covers:

- the umbrella P1 workbench proof matrix,
- the promoted P1 shell diagnostics smoke floor and suite roster,
- the closed workbench-shell lane state, evidence, closeout validation, and no-new-helper verdict,
- and the Python source-policy gate marker that replaces the deleted Rust source-marker tests.

The real workspace shell and editor notes rail surface integration tests remain in Rust. The
launched `diag-hardening-smoke-workspace` suite remains the promoted behavior smoke floor. Only the
document/source-policy freeze moved to Python.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json`
- `docs/workstreams/imui-workbench-shell-closure-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-workbench-shell-closure-v1/CLOSEOUT_AUDIT_2026-04-13.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_WORKBENCH_PROOF_MATRIX_2026-04-12.md`
- `docs/workstreams/imui-editor-grade-product-closure-v1/P1_SHELL_DIAG_SMOKE_DECISION_2026-04-12.md`
- `tools/diag-scripts/suites/diag-hardening-smoke-workspace/suite.json`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 32
to 29, and the `include_str!` count dropped from 144 to 140.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-workbench-shell-closure-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
