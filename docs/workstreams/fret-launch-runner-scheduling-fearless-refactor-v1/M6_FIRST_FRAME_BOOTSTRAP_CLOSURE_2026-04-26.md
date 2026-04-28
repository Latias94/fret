# M6 First-Frame Bootstrap Closure

Status: Maintenance slice landed

Date: 2026-04-26

## Problem

The TODO tracker still carried a P0 report for demos that appeared blank until hover or another
input event. Normal desktop window creation already requested a bootstrap redraw after the
`WindowId -> AppWindowId` registry insertion and added a one-shot RAF fallback. The deferred
surface-creation path still used a direct `request_redraw()` call and then recorded
`SurfaceBootstrap` separately, so it did not share the same wake guarantee.

## Decision

`SurfaceBootstrap` now has one desktop bootstrap rule:

1. install the window registry mapping before requesting redraw for newly inserted windows,
2. issue the redraw through `request_window_redraw_with_reason(..., SurfaceBootstrap)`,
3. schedule a one-shot RAF fallback for the same window,
4. hold RAF fallback until the configured frame deadline, then request redraw and poll one turn,
5. keep the tiny native smoke demo as the first local repro for blank-start reports.

This is a runner-owned scheduling fix. It does not widen public runtime APIs or move component
policy into `fret-ui`.

## Evidence

- Normal window creation: `crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs`
- Deferred surface creation: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- RAF deadline state: `crates/fret-launch/src/runner/desktop/runner/{mod.rs,run.rs}`
- Redraw helper and diagnostics write: `crates/fret-launch/src/runner/desktop/runner/scheduling_diagnostics.rs`
- RAF coalescing helper: `crates/fret-launch/src/runner/common/frame_requests.rs`
- Smoke repro: `apps/fret-examples/src/first_frame_smoke_demo.rs`
- Source-policy gate: `apps/fret-examples/src/lib.rs`

## Gates

```bash
cargo fmt -p fret-launch -p fret-examples --check
cargo nextest run -p fret-examples --lib first_frame_bootstrap_smoke_locks_runner_wake_paths --no-fail-fast
cargo nextest run -p fret-launch --lib --no-fail-fast
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json > /dev/null
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

## Follow-up posture

Treat future "blank until hover" reports as new narrow repros only if they bypass both
`SurfaceBootstrap` paths above. Do not reopen broad runner scheduling scope for unrelated layout or
paint invalidation issues.
