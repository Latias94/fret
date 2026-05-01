# Fret Examples Build Latency v1 - Final Status - 2026-05-01

Status: maintenance

## Verdict

The active source-policy migration phase is complete.

`apps/fret-examples/src/lib.rs` no longer owns source-marker tests or `include_str!` snapshots for
demo/source-policy checks. It keeps only the two real parser behavior tests for
`parse_editor_theme_preset_key`.

## Shipped State

- Pure source-marker checks moved to Python gates.
- Owner-heavy source policies split behind `tools/examples_source_tree_policy/` modules.
- IMUI heavy demo sources have a separate fast-path crate: `apps/fret-examples-imui`.
- `fret-examples` compatibility checks remain as build gates.
- Parser behavior remains in Rust and is covered by targeted nextest.

## Current Counts

- `apps/fret-examples/src/lib.rs` Rust tests: 2.
- `apps/fret-examples/src/lib.rs` `include_str!` source snapshots: 0.

## Maintenance Gates

```text
python tools/gate_examples_source_tree_policy.py
python tools/gate_imui_workstream_source.py
python tools/gate_fret_launch_runner_scheduling_source.py
python tools/gate_fret_examples_imui_split_source.py
cargo check -p fret-examples --lib --jobs 1
cargo nextest run -p fret-examples --lib parse_editor_theme_preset_key --no-fail-fast
python tools/check_workstream_catalog.py
git diff --check
```

## Reopen Rule

Continue this lane only for narrow maintenance:

- a source-policy check drifts back into `fret-examples` Rust tests,
- a source-only gate starts requiring a monolithic examples compile again,
- or a measured single-demo iteration path regresses against the documented fast-path split.

Start a narrower follow-on for broad demo-family crate splits, new diagnostics campaigns, or
unrelated component/runtime behavior.
