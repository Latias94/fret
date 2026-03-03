---
title: UI Diagnostics Timebase Decoupling v1
status: draft
date: 2026-03-03
scope: diagnostics, runner, automation, multi-window, docking
---

# UI diagnostics timebase decoupling (v1)

## Problem statement

Diagnostics scripted runs historically advanced primarily when a window produced redraw callbacks. In multi-window
scenarios (especially docking tear-off + overlap + z-order churn), it is possible for the “relevant” window to become
fully occluded or otherwise idle/throttled and stop producing redraw callbacks.

When that happens, the following failure class appears:

- the script stays `stage=running` forever because `wait_frames`/timeouts do not elapse,
- the tooling times out waiting for `script.result.json`,
- the run is hard to triage because it ends as a tooling timeout instead of a stable, evidence-rich failure.

This is a determinism and debuggability gap: **scripts must either progress or fail** with a bounded, machine-readable
reason code, even when no frames are being presented.

## Goals

- **Liveness:** a tool-launched run must not hang indefinitely due to “no redraw callbacks”.
- **Bounded failure:** if progress is impossible, fail fast with a stable `reason_code` and capture bounded evidence.
- **Multi-window correctness support:** docking scripts should remain usable even when windows overlap/occlude.
- **Small-by-default:** do not require raw `bundle.json` inspection to understand what happened.

## Non-goals

- Guarantee that every interaction step (pointer-driven UI dispatch, semantics-dependent selector resolution) can run
  without rendering. Some steps fundamentally require real UI frames.
- Redesign script schema in this workstream (schema changes can be proposed as follow-ups once we have evidence).

## Approach (staged)

### M0 (shipped): keepalive timer + conservative “no-frame drive”

While a diagnostics script is active, arm a runner-backed repeating timer (via `Effect::SetTimer`). On each timer tick,
the diagnostics runtime can:

- keep writing “running heartbeat” updates,
- advance a conservative subset of script steps that do not require a fresh `UiTree` / semantics snapshot (e.g.
  `wait_frames`, off-window docking predicates, window ops, `capture_bundle`),
- and if the next step cannot progress without a frame for too long, fail with `reason_code=timeout.no_frames`.

This avoids the worst failure mode (“tooling timeout waiting for `script.result`”) and keeps evidence bounded and
portable.

Timeout semantics (v1 contract):

- `timeout_frames` and `wait_frames` are *frame-based when frames exist*.
- When a window stops producing frames, a runner-backed keepalive timer may advance a conservative subset of steps
  (including decrementing `timeout_frames` / `wait_frames`) so runs do not hang.
- If progress is impossible without frames, fail with `reason_code=timeout.no_frames` after a bounded wall time.

Evidence anchors:

- Timer arming/cancel: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Timer event handling: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Failure mapping: `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`

### M1 (shipped): start scripts without requiring a render loop

Tool-launched filesystem triggers must be observed even when a launch-mode app goes idle and stops producing redraw
callbacks. The keepalive tick polls triggers and can start pending scripts without requiring a steady render loop.

Deliverables:

- Pending scripts start deterministically even when no redraw callbacks are arriving.
- The liveness contract is “keepalive owns polling and bounded progress while scripts are pending/active”.

Evidence anchors:

- Keepalive tick polls triggers + starts pending scripts: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Deterministic test hook: `FRET_DIAG_SIMULATE_NO_FRAMES=1` (see `ecosystem/fret-bootstrap/src/ui_diagnostics/config.rs`)

### M2 (shipped): make “timeout” semantics explicit (frames vs ticks vs ms)

The current schema uses `timeout_frames`. In no-frame scenarios, “frames” are not observable. We need an explicit
contract for what happens when frames are not advancing:

- Option A (strict): timeouts remain frame-based; if no frames, fail with `timeout.no_frames` after a bounded wall time.
- Option B (hybrid): define “script ticks” and allow timeouts to decrement on either a frame or a keepalive tick.
- Option C (schema evolution): introduce `timeout_ms` / `wait_ms` and de-emphasize `wait_frames` for “settle”.

This workstream locks a v1 hybrid contract (bounded keepalive-driven progress + stable `reason_code=timeout.no_frames`)
with a deterministic regression script:

- `tools/diag-scripts/diag/no-frame/diag-no-frame-timeout-no-frames.json`

### M3: expand no-frame coverage safely (optional)

Once M1/M2 are in place, we can decide whether to expand the “no-frame drive” subset (e.g. more predicates from cached
snapshots), or keep it minimal and rely on bounded failure + evidence.

## Related workstreams

- Docking suite hardening: `docs/workstreams/docking-arbitration-diag-hardening-v1/`
- Diag v2 hardening + switches: `docs/workstreams/diag-v2-hardening-and-switches-v1/`
