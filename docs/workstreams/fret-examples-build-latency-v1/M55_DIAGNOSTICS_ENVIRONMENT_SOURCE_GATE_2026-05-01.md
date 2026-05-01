# Fret Examples Build Latency v1 - M55 Diagnostics Environment Source Gate - 2026-05-01

Status: complete

## Decision

Move the diagnostics environment source-policy checks out of the monolithic `fret-examples` Rust
unit-test module and into `tools/gate_imui_workstream_source.py`.

## Migrated Checks

- `immediate_mode_workstream_freezes_the_p3_mixed_dpi_monitor_topology_follow_on`
- `immediate_mode_workstream_freezes_the_diag_environment_predicate_taxonomy`

## Behavior

The IMUI workstream source gate now covers the closed monitor-topology environment lane, the
environment-predicate taxonomy lane, and the owner source markers those Rust tests previously
included through `include_str!`.

The real behavior gates remain in their owner crates: `fret-runtime`, `fret-bootstrap`,
`fret-diag-protocol`, and `fret-diag`. This slice only moves document/source-policy freeze markers
out of Rust.

## Evidence

- `tools/gate_imui_workstream_source.py`
- `apps/fret-examples/src/lib.rs`
- `docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json`
- `docs/workstreams/diag-monitor-topology-environment-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json`
- `docs/workstreams/diag-environment-predicate-contract-v1/EVIDENCE_AND_GATES.md`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 13
to 11, and the `include_str!` count dropped from 111 to 79.

## Gates

```text
python tools/gate_imui_workstream_source.py
python -m py_compile tools/gate_imui_workstream_source.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/diag-monitor-topology-environment-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/diag-environment-predicate-contract-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
