# Fret Launch Runner Scheduling (Fearless Refactor v1) — TODO

Status: Draft

Last updated: 2026-03-13

Companion docs:

- Design: `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/README.md`
- Milestones: `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/MILESTONES.md`

## Documentation + contract alignment

- [x] Create a dedicated workstream folder for launch runner scheduling.
- [x] Record the current cross-platform hazards and desired target shape.
- [x] Confirm the exact v1 ownership statement for:
  - [x] `fret-app` redraw coalescing
  - [x] `fret-launch` turn/frame lifecycle
  - [x] `fret-platform-web` DOM timers
  - [x] `fret-runner-winit` adapter-only responsibilities
- [x] Decide whether ADR 0034 needs wording updates, or only implementation-alignment evidence.

## Shared scheduling seam (`crates/fret-launch/src/runner/common/*`)

- [ ] Add a launch-internal scheduling module for shared semantics:
  - [x] turn bookkeeping
  - [x] frame commit bookkeeping
  - [ ] redraw / RAF coalescing
  - [ ] bounded fixed-point drain policy
- [x] Keep the extracted surface internal-only; do not widen public crate exports.
- [ ] Add unit tests for:
  - [x] `TickId` incrementing once per runner turn
  - [x] `FrameId` incrementing only on committed present
  - [ ] redraw request coalescing
  - [ ] RAF request coalescing

## Desktop runner adoption

- [x] Refactor desktop turn/frame bookkeeping to use the shared seam.
- [x] Keep `about_to_wait()` as the owner of native `ControlFlow` decisions.
- [x] Verify native timer wakeups still participate in the same fixed-point drain semantics.
- [ ] Audit diagnostics writes so their meaning matches the shared turn/frame contract.
- [ ] Thin `app_handler.rs` only after behavior remains unchanged.

## Web runner adoption

- [x] Introduce a frame-resource restoration guard in `runner/web/render_loop.rs`.
- [x] Ensure every early-return path restores `gfx` and window state.
- [x] Move web `FrameId` commit to the successful submit/present path.
- [x] Move web `TickId` updates to runner-turn boundaries instead of render entry.
- [x] Verify DOM timer wakeups from `fret-platform-web` are consumed under the same turn semantics.
- [x] Re-check async wake paths:
  - [x] `proxy_wake_up`
  - [x] pending async events
  - [x] clipboard/file-dialog completions
  - [x] devtools/configured keepalive redraws

## Timer posture

- [x] Document the intentional v1 split:
  - [x] desktop native timers remain runner-owned
  - [x] web timers remain browser-service-owned
- [x] Ensure both paths feed `Event::Timer` before the next bounded drain cycle.
- [x] Decide whether a deeper shared timer abstraction is:
  - [x] out of scope for v1
  - [ ] required immediately
  - [ ] a separate follow-up workstream

## Diagnostics + evidence

- [x] Add focused launch tests for scheduling semantics.
- [x] Add at least one regression path covering web surface acquire failure recovery.
- [x] Add evidence anchors for any touched diagnostics stores or frame-drive reasons.
- [x] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` if the final code path materially improves ADR 0034 alignment.

## Validation gates

- [x] `cargo fmt -p fret-launch`
- [x] `cargo nextest run -p fret-launch`
- [x] `python tools/check_layering.py`
- [x] Any new targeted scheduling tests added for this workstream.

## Closeout

- [ ] Review whether any remaining scheduling duplication is acceptable for v1.
- [ ] Record any intentionally deferred cleanup in the README and milestones docs.
- [ ] Prepare a follow-up worktree only after the documentation commit lands on `main`.
