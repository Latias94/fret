# Fret Launch Runner Scheduling (Fearless Refactor v1) — TODO

Status: Draft

Last updated: 2026-03-13

Companion docs:

- Design: `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/README.md`
- Milestones: `docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/MILESTONES.md`

## Documentation + contract alignment

- [x] Create a dedicated workstream folder for launch runner scheduling.
- [x] Record the current cross-platform hazards and desired target shape.
- [ ] Confirm the exact v1 ownership statement for:
  - [ ] `fret-app` redraw coalescing
  - [ ] `fret-launch` turn/frame lifecycle
  - [ ] `fret-platform-web` DOM timers
  - [ ] `fret-runner-winit` adapter-only responsibilities
- [ ] Decide whether ADR 0034 needs wording updates, or only implementation-alignment evidence.

## Shared scheduling seam (`crates/fret-launch/src/runner/common/*`)

- [ ] Add a launch-internal scheduling module for shared semantics:
  - [ ] turn bookkeeping
  - [ ] frame commit bookkeeping
  - [ ] redraw / RAF coalescing
  - [ ] bounded fixed-point drain policy
- [ ] Keep the extracted surface internal-only; do not widen public crate exports.
- [ ] Add unit tests for:
  - [ ] `TickId` incrementing once per runner turn
  - [ ] `FrameId` incrementing only on committed present
  - [ ] redraw request coalescing
  - [ ] RAF request coalescing

## Desktop runner adoption

- [ ] Refactor desktop turn/frame bookkeeping to use the shared seam.
- [ ] Keep `about_to_wait()` as the owner of native `ControlFlow` decisions.
- [ ] Verify native timer wakeups still participate in the same fixed-point drain semantics.
- [ ] Audit diagnostics writes so their meaning matches the shared turn/frame contract.
- [ ] Thin `app_handler.rs` only after behavior remains unchanged.

## Web runner adoption

- [ ] Introduce a frame-resource restoration guard in `runner/web/render_loop.rs`.
- [ ] Ensure every early-return path restores `gfx` and window state.
- [ ] Move web `FrameId` commit to the successful submit/present path.
- [ ] Move web `TickId` updates to runner-turn boundaries instead of render entry.
- [ ] Verify DOM timer wakeups from `fret-platform-web` are consumed under the same turn semantics.
- [ ] Re-check async wake paths:
  - [ ] `proxy_wake_up`
  - [ ] pending async events
  - [ ] clipboard/file-dialog completions
  - [ ] devtools/configured keepalive redraws

## Timer posture

- [ ] Document the intentional v1 split:
  - [ ] desktop native timers remain runner-owned
  - [ ] web timers remain browser-service-owned
- [ ] Ensure both paths feed `Event::Timer` before the next bounded drain cycle.
- [ ] Decide whether a deeper shared timer abstraction is:
  - [ ] out of scope for v1
  - [ ] required immediately
  - [ ] a separate follow-up workstream

## Diagnostics + evidence

- [ ] Add focused launch tests for scheduling semantics.
- [ ] Add at least one regression path covering web surface acquire failure recovery.
- [ ] Add evidence anchors for any touched diagnostics stores or frame-drive reasons.
- [ ] Update `docs/adr/IMPLEMENTATION_ALIGNMENT.md` if the final code path materially improves ADR 0034 alignment.

## Validation gates

- [ ] `cargo fmt -p fret-launch`
- [ ] `cargo nextest run -p fret-launch`
- [ ] `python tools/check_layering.py`
- [ ] Any new targeted scheduling tests added for this workstream.

## Closeout

- [ ] Review whether any remaining scheduling duplication is acceptable for v1.
- [ ] Record any intentionally deferred cleanup in the README and milestones docs.
- [ ] Prepare a follow-up worktree only after the documentation commit lands on `main`.
