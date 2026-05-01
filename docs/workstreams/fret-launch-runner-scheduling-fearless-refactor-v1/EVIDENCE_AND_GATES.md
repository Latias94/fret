# Fret Launch Runner Scheduling (Fearless Refactor v1) — Evidence and Gates

Status: Maintenance

Last updated: 2026-05-01

## Smallest current repro

Use the source-policy gate first. It locks the deterministic first-frame bootstrap contract without
depending on a specific desktop compositor:

```bash
python tools/gate_fret_launch_runner_scheduling_source.py
```

Use the native smoke demo for a local visual/manual check:

```bash
cargo run -p fret-demo --bin first_frame_smoke_demo
```

What this proves:

- the smoke demo submits a real full-window `SceneOp::Quad`,
- the demo requests follow-up animation frames and self-closes after several rendered frames,
- normal window creation records `SurfaceBootstrap` through the redraw helper after the winit id
  registry is installed,
- deferred surface creation uses the same redraw helper and one-shot RAF fallback,
- desktop RAF fallback waits until the configured frame deadline, requests redraw, and polls one
  turn so redraw delivery is not dependent on pointer input,
- first-frame bootstrap no longer relies on later pointer movement or hover to wake rendering.

## Gate set

### Source and runner gates

```bash
cargo fmt -p fret-launch -p fret-examples --check
python tools/gate_fret_launch_runner_scheduling_source.py
cargo nextest run -p fret-launch --lib --no-fail-fast
```

### Lane hygiene

```bash
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json > /dev/null
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Evidence anchors

- `apps/fret-examples/src/first_frame_smoke_demo.rs`
- `apps/fret-demo/src/bin/first_frame_smoke_demo.rs`
- `tools/gate_fret_launch_runner_scheduling_source.py`
- `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
- `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- `crates/fret-launch/src/runner/desktop/runner/mod.rs`
- `crates/fret-launch/src/runner/desktop/runner/run.rs`
- `crates/fret-launch/src/runner/desktop/runner/scheduling_diagnostics.rs`
- `crates/fret-launch/src/runner/common/frame_requests.rs`
- `crates/fret-launch/src/runner/common/scheduling.rs`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/README.md`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/TODO.md`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/M6_FIRST_FRAME_BOOTSTRAP_CLOSURE_2026-04-26.md`
- `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json`
- `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
