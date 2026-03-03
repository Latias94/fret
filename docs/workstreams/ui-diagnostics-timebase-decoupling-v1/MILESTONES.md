# UI diagnostics timebase decoupling (v1) — Milestones

## M0 — No-frame hang prevention (keepalive timer)

Goal: tool-launched runs must not hang indefinitely when redraw callbacks stop (occlusion/idle/throttling).

Deliverables:

- A runner-backed keepalive timer armed while scripts are active.
- A conservative “no-frame drive” loop that can:
  - keep writing `stage=running` heartbeats,
  - advance a safe subset of steps,
  - and fail with `reason_code=timeout.no_frames` if the next step cannot progress without frames for too long.

Status (2026-03-03): shipped in-tree.

Evidence:

- Runtime keepalive + no-frame drive: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Timer event hook: `ecosystem/fret-bootstrap/src/ui_app_driver.rs`
- Reason code mapping: `ecosystem/fret-bootstrap/src/ui_diagnostics/labels.rs`
- Tool launch config default keepalive: `crates/fret-diag/src/compare.rs`

## M1 — Pending-script liveness (start without frames)

Goal: “script injected → script starts” should not depend on a steady render loop.

Deliverables:

- A deterministic path to observe script triggers and start pending scripts even if no redraw callbacks are arriving.
- A minimal contract note describing which subsystem owns liveness while a script is pending.

## M2 — Timeout semantics contract

Goal: make timeout behavior explicit and testable when frames are not advancing.

Deliverables:

- Choose and document one semantics model (strict frame-based + bounded failure, hybrid ticks, or schema evolution).
- At least one regression script that forces an occlusion/idle no-frame scenario and validates the chosen behavior.

## M3 — Safe coverage expansion (optional)

Goal: reduce false “no-frame” failures without turning the no-frame path into a second full script runtime.

Deliverables:

- A reviewed list of which steps/predicates are allowed to advance off the no-frame path.
- Evidence that the expanded set remains deterministic and bounded (no output explosion).
