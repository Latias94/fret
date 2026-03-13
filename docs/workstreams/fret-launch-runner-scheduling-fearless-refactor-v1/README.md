# Fret Launch Runner Scheduling (Fearless Refactor v1)

Status: Draft

Last updated: 2026-03-13

## Context

Fret's architecture already places the scheduling and presentation responsibility in the correct
layer:

- `crates/fret-app` owns effect queues and redraw intent coalescing.
- `crates/fret-launch` owns runner turn lifecycle, effect draining, rendering, presenting, and
  frame-loop control.
- `crates/fret-platform-web` owns browser APIs such as DOM timers, clipboard, file dialogs, and
  IME bridges.
- `crates/fret-runner-winit` owns event mapping and per-platform input/window normalization.

This is consistent with:

- `docs/architecture.md`
- `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- `crates/fret-platform/src/lib.rs`
- `crates/fret-launch/src/lib.rs`

The current problem is not the macro layering. The problem is that the **desktop and web runner
implementations do not currently realize the same scheduling contract**.

## Why this workstream exists

This workstream is triggered by a small number of high-leverage hazards discovered during the
cross-platform runner audit.

### 1) Web render-loop recovery is currently brittle

`crates/fret-launch/src/runner/web/render_loop.rs` takes ownership of `self.gfx` and
`self.window_state` before attempting to acquire the current surface frame. If surface acquisition
fails, the function currently returns early without restoring that state.

That means a transient `wgpu::SurfaceError::{Lost,Outdated,Timeout,Other}` can leave the web runner
internally inconsistent.

### 2) `TickId` and `FrameId` semantics drift between desktop and web

ADR 0034 locks these semantics:

- `TickId` increments per runner turn, even when no rendering occurs.
- `FrameId` increments only when the runner actually submits/presents a rendered frame.

Desktop is already much closer to that contract.
Web currently increments both counters at render entry, before surface acquisition and before
present.

That creates semantic drift in:

- diagnostics stores,
- same-turn suppression logic,
- renderer-side frame accounting,
- future perf gates.

### 3) Scheduling logic is duplicated instead of shared

Desktop and web both implement:

- bounded fixed-point draining,
- redraw requests,
- animation-frame requests,
- timer wake integration,
- turn-local diagnostics,
- frame lifecycle bookkeeping.

But these concepts are spread across separate large modules, which makes it too easy for one backend
to evolve while the other quietly drifts.

### 4) The current implementation shape will make future backends harder

If mobile or alternate runners are added later, the current model encourages copying one existing
backend and then patching over behavior differences. That is exactly the opposite of what a
fearless refactor should leave behind.

## Goals

- Make desktop and web runners implement the same scheduling contract.
- Preserve existing crate boundaries and keep the mechanism ownership clear.
- Make render-frame state handling failure-safe on web.
- Move shared scheduling semantics into focused internal launch modules.
- Keep redraw, RAF, timer, tick, and frame diagnostics reviewable and testable.
- Leave behind gates that detect semantic drift early.

## Non-goals

- No component-policy changes in `ecosystem/*`.
- No migration of Radix/shadcn interaction policy into `crates/fret-ui`.
- No crate split in v1 (`fret-launch-desktop`, `fret-launch-web`, etc.).
- No redesign of renderer submission contracts or docking behavior.
- No forced timer ownership move out of `fret-platform-web` unless the current boundary proves
  fundamentally unworkable.

## Invariants

These statements must remain true throughout the refactor:

1. `crates/fret-app` remains the source of queued effects and redraw intent coalescing.
2. `crates/fret-launch` remains the owner of runner turn lifecycle and present sequencing.
3. `crates/fret-platform-web` remains the owner of browser API calls such as `setTimeout`.
4. `crates/fret-runner-winit` remains an adapter layer, not the frame-loop contract owner.
5. `crates/fret-ui` remains a mechanism/contract layer and must not absorb runner policy.
6. Desktop and web may differ in wake-up mechanics, but not in public scheduling semantics.

## Current findings summary

### F1 — Web state restoration hole

The web render loop needs an exception-safe ownership boundary around frame resources.

Required outcome:

- no early-return path may permanently drop `gfx` or window state,
- transient surface failures must remain recoverable,
- present success must be the only path that commits a rendered frame.

### F2 — Cross-backend clock drift

`TickId` and `FrameId` are observability contracts, not backend-local implementation details.

Required outcome:

- turn bookkeeping and frame bookkeeping become explicit shared logic,
- desktop and web emit diagnostics under the same meaning of "turn" and "frame".

### F3 — Shared concepts, backend-specific sinks

The following should be shared conceptually, even if the actual wake mechanism differs:

- one-shot redraw requests,
- animation-frame requests,
- bounded fixed-point draining,
- frame commit rules,
- wake-reason diagnostics.

Backend-specific details should stay backend-specific:

- desktop `ControlFlow::{Poll,WaitUntil,Wait}`,
- browser `wake_up`, DOM timers, and canvas redraw wake behavior.

### F4 — Timer ownership is already mostly correct

Desktop currently handles runner-owned timers inside `fret-launch`.
Web currently handles DOM timer implementation inside `fret-platform-web`.

That split is acceptable for v1 as long as both sides feed the same event semantics into the runner:

- timer wakeups must become `Event::Timer`,
- timer-driven work must participate in the same bounded drain model,
- timer wakeups must not imply different `TickId` / `FrameId` semantics by backend.

## Proposed target shape

The target is **shared scheduling semantics with backend-specific wake sinks**.

### A) Introduce a small shared scheduling core inside `fret-launch`

Illustrative module shape:

- `crates/fret-launch/src/runner/common/scheduling/turn_clock.rs`
- `crates/fret-launch/src/runner/common/scheduling/frame_requests.rs`
- `crates/fret-launch/src/runner/common/scheduling/fixed_point.rs`
- `crates/fret-launch/src/runner/common/scheduling/mod.rs`

The shared core should define internal logic such as:

- `begin_turn() -> TickId`
- redraw/RAF request coalescing rules
- bounded fixed-point drain policy
- `commit_presented_frame() -> FrameId`
- wake-reason bookkeeping for diagnostics

This is internal launch infrastructure, not a new public crate contract.

### B) Keep backend-specific wake and timer implementations outside the shared core

Desktop should continue to own:

- `ControlFlow` decisions,
- OS-window redraw calls,
- runner-local timer storage when using native wake paths.

Web should continue to own:

- browser wakeups,
- DOM `setTimeout`,
- canvas redraw wake behavior,
- web-specific async result bridging.

The shared core should not try to erase platform differences. It should only erase **semantic
drift**.

### C) Add an exception-safe frame-resource guard for the web render loop

The web render loop needs a small internal guard that:

- acquires `gfx` and `window_state`,
- automatically restores them on abort/early return,
- only marks the frame "committed" after successful submit/present.

The same pattern may later be generalized for desktop, but web is the must-fix path.

### D) Make "turn" and "frame" explicit commit points

We should stop letting counter updates happen "wherever the backend currently is".

Desired semantics:

- `TickId` commits at runner-turn entry.
- `FrameId` commits after successful render submission/present.
- diagnostics that are keyed by `FrameId` only record a new frame once that commit happened.

### E) Keep the refactor staged and reversible

The target is not a big-bang rewrite.

Preferred order:

1. document the contract,
2. add tests for the shared semantics,
3. extract shared scheduling helpers without behavior changes,
4. adopt desktop,
5. adopt web while fixing the recovery bug,
6. remove duplicated code only after both paths are green.

## Proposed landable phases

### Phase 0 — Documentation + seam selection

Deliverables:

- this workstream folder,
- explicit refactor scope,
- agreement on invariants and target ownership.

### Phase 1 — Shared scheduling seam extraction

Deliverables:

- internal launch scheduling module(s),
- unit tests for turn/frame semantics,
- no intentional behavior change yet.

### Phase 2 — Desktop adoption

Deliverables:

- desktop runner delegates turn/frame bookkeeping to the shared scheduling seam,
- `about_to_wait()` remains the owner of native `ControlFlow`,
- diagnostics semantics stay unchanged except where ADR alignment requires cleanup.

### Phase 3 — Web adoption + recovery hardening

Deliverables:

- web render loop uses the same turn/frame contract,
- web frame-resource ownership becomes failure-safe,
- web no longer increments `FrameId` before present succeeds.

### Phase 4 — Diagnostics + gates closure

Deliverables:

- focused tests around scheduling semantics,
- evidence anchors for the shared logic and both backend integrations,
- ADR alignment update if implementation wording or evidence needs to change.

### Phase 5 — Optional follow-up cleanup

Possible follow-ups, only after Phase 4 is stable:

- further thin `desktop/runner/app_handler.rs`,
- decide whether frame-resource guards should also be standardized on desktop,
- decide whether timer storage should gain a deeper shared abstraction in a future workstream.

## Acceptance criteria

This workstream is considered complete when all of the following are true:

- Web surface acquisition failures do not lose runner-owned state.
- Desktop and web use the same semantic rules for `TickId` and `FrameId`.
- `FrameId` is committed only after a successful rendered frame submission/present.
- `TickId` can advance even on turns without rendering.
- Shared scheduling logic lives in reviewable launch-internal modules rather than being duplicated
  ad hoc.
- Existing crate boundaries remain intact.
- Regression gates exist for both semantic drift and the web recovery path.

## Suggested gates

Minimum intended gates once code work starts:

- `cargo fmt -p fret-launch`
- `cargo nextest run -p fret-launch`
- `python tools/check_layering.py`

Additional targeted gates recommended for this workstream:

- launch-internal unit tests for turn/frame semantics,
- a focused regression test for web frame-resource restoration on surface acquire failure,
- diagnostics assertions for `TickId` / `FrameId` sequencing when feasible.

## Evidence anchors

- Architecture ownership: `docs/architecture.md`
- Scheduling contract: `docs/adr/0034-timers-animation-and-redraw-scheduling.md`
- App redraw/effect coalescing: `crates/fret-app/src/app.rs`
- Launch facade: `crates/fret-launch/src/lib.rs`
- Desktop runner entry: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
- Desktop effect draining: `crates/fret-launch/src/runner/desktop/runner/effects.rs`
- Web runner loop: `crates/fret-launch/src/runner/web/render_loop.rs`
- Web app handler: `crates/fret-launch/src/runner/web/app_handler.rs`
- Web platform timers: `crates/fret-platform-web/src/wasm/{mod.rs,timers.rs}`
- Winit adapter boundary: `crates/fret-runner-winit/src/lib.rs`

## Open questions

These do not block the documentation phase, but they should be answered before large code moves.

1. Should the shared scheduling core also own wake-reason diagnostics, or only the turn/frame
   counters and request coalescing?
2. Should desktop adopt the same frame-resource guard pattern as web, even if its immediate bug
   profile is lower?
3. Should the desktop timer table and web DOM timers remain intentionally separate in v1, with only
   event semantics unified?
4. Should future mobile backends reuse the same shared scheduling seam directly, or should a
   thinner backend-facing trait sit in front of it?

## Recommended first implementation batch

If we start coding after this documentation lands, the safest first batch is:

1. add scheduling-unit-test scaffolding inside `fret-launch`,
2. introduce a launch-internal turn/frame bookkeeping helper,
3. refactor web `render_frame()` to restore state on all abort paths,
4. move web `FrameId` commit to the post-present path,
5. only then start thinning desktop/web duplicated scheduling code.
