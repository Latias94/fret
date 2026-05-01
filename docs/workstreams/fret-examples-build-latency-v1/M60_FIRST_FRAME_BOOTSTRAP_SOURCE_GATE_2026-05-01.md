# Fret Examples Build Latency v1 - M60 First Frame Bootstrap Source Gate - 2026-05-01

Status: complete

## Decision

Move the first-frame bootstrap runner scheduling source-policy check out of the monolithic
`fret-examples` Rust unit-test module and into
`tools/gate_fret_launch_runner_scheduling_source.py`.

## Migrated Check

- `first_frame_bootstrap_smoke_locks_runner_wake_paths`

## Behavior

The new Python gate freezes the same source-only contract:

- `first_frame_smoke_demo` paints a full-window quad, requests follow-up RAFs, and self-closes.
- Normal window insertion installs the winit registry mapping before requesting the
  `SurfaceBootstrap` redraw helper and one-shot RAF.
- Deferred surface creation uses the same redraw helper and one-shot RAF fallback.
- RAF deadline wake logic flushes redraw requests and polls one turn when needed.
- The runner scheduling workstream remains the owner for the blank-start invariant.

This slice only moves source markers. It does not change runner behavior or the native smoke demo.

## Evidence

- `tools/gate_fret_launch_runner_scheduling_source.py`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/src/first_frame_smoke_demo.rs`
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/M6_FIRST_FRAME_BOOTSTRAP_CLOSURE_2026-04-26.md`

## Result

After this migration, the Rust `#[test]` count in `apps/fret-examples/src/lib.rs` dropped from 6
to 5, and the `include_str!` count dropped from 74 to 68.

## Gates

```text
python tools/gate_fret_launch_runner_scheduling_source.py
python -m py_compile tools/gate_fret_launch_runner_scheduling_source.py tools/check_workstream_catalog.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json
python -m json.tool docs/workstreams/fret-examples-build-latency-v1/WORKSTREAM.json
cargo fmt --package fret-examples --check
cargo check -p fret-examples --lib --jobs 1
```
