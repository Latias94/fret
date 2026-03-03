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

Evidence anchors:

- Timer arming/cancel: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Timer event handling: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Failure mapping: `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`

### M1: start scripts without requiring a render loop

Today, “script trigger → script start” is still primarily observed from per-window post-paint hooks. If a launch-mode
app goes idle at an unlucky time, the script trigger may not be observed promptly.

Deliverables:

- Ensure pending scripts can be started deterministically even when no redraw callbacks are arriving.
- Define the minimal liveness contract for “tool-launched filesystem-trigger” mode:
  - what wakes the loop,
  - which component owns “keep ticking while scripts are pending”.

### M2: make “timeout” semantics explicit (frames vs ticks vs ms)

The current schema uses `timeout_frames`. In no-frame scenarios, “frames” are not observable. We need an explicit
contract for what happens when frames are not advancing:

- Option A (strict): timeouts remain frame-based; if no frames, fail with `timeout.no_frames` after a bounded wall time.
- Option B (hybrid): define “script ticks” and allow timeouts to decrement on either a frame or a keepalive tick.
- Option C (schema evolution): introduce `timeout_ms` / `wait_ms` and de-emphasize `wait_frames` for “settle”.

This workstream should choose one path and lock it with at least one regression script.

### M3: expand no-frame coverage safely (optional)

Once M1/M2 are in place, we can decide whether to expand the “no-frame drive” subset (e.g. more predicates from cached
snapshots), or keep it minimal and rely on bounded failure + evidence.

## Related workstreams

- Docking suite hardening: `docs/workstreams/docking-arbitration-diag-hardening-v1/`
- Diag v2 hardening + switches: `docs/workstreams/diag-v2-hardening-and-switches-v1/`

