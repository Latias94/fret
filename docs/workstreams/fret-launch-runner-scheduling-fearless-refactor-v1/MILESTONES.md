# Fret Launch Runner Scheduling (Fearless Refactor v1) — Milestones

Status: Draft

Last updated: 2026-03-13

## Current progress (2026-03-13)

- M1: Partial
  - shared turn/frame scheduling helper landed,
  - pure counter semantics tests landed,
  - slot-restoration seam and its regression tests landed,
  - shared RAF coalescing helper landed,
  - shared bounded fixed-point helper landed,
  - app-owned redraw coalescing ownership is now explicit.
- M2: Partial
  - desktop `TickId` turn advancement and `FrameId` present commitment now use the shared helper,
  - native timer participation in the bounded drain has now been audited,
  - present diagnostics now observe the committed frame id through the shared seam,
  - broader diagnostics auditing is still pending.
- M3: Partial
  - web `TickId` turn advancement moved out of render entry,
  - web `FrameId` commitment moved to the post-present path,
  - web frame-state restoration on surface acquire failure is now in place,
  - wake-path audit is now documented,
  - web RAF scheduling now flushes from `about_to_wait()` through the shared helper.
- M4: Partial
  - ADR 0034 wording was confirmed stable for v1,
  - implementation-alignment evidence now reflects the shared turn/frame seam, shared RAF queue,
    shared bounded drain helper, and web recovery path,
  - diagnostics audit now explicitly records that `SurfaceBootstrap` startup writes are
    mutually exclusive while redraw-request diagnostics remain app-owned current-frame snapshots.

## M0 — Workstream agreed

Outcome:

- A dedicated workstream folder exists with design, TODO, and milestones documents.
- The current hazards are recorded before code changes begin.
- The target ownership and non-goals are explicit.

Exit criteria:

- `README.md`, `TODO.md`, and `MILESTONES.md` are reviewable.
- The intended code-owner layer is clear: this is primarily a `fret-launch` refactor.

## M1 — Shared scheduling seam extracted

Outcome:

- Launch-internal shared scheduling helpers exist under `runner/common/*`.
- Turn/frame semantics are defined once instead of duplicated ad hoc.

Exit criteria:

- Shared helpers cover turn bookkeeping, frame commit rules, and request coalescing.
- Unit tests exist for `TickId`, `FrameId`, redraw, and RAF semantics.
- No public crate-root export creep is introduced.

## M2 — Desktop runner aligned

Outcome:

- Desktop uses the shared scheduling seam for turn/frame logic.
- Native `ControlFlow` remains backend-specific.

Exit criteria:

- Desktop turn/frame bookkeeping is no longer backend-local ad hoc logic.
- Native timer and redraw wake behavior still pass existing launch tests.
- No layering violations are introduced.

## M3 — Web runner hardened and aligned

Outcome:

- Web render-frame ownership is failure-safe.
- Web turn/frame semantics match desktop and ADR 0034.

Exit criteria:

- Surface acquire failures restore `gfx` and window state.
- `FrameId` is committed only after successful submit/present.
- `TickId` is not tied to render-entry anymore.

## M4 — Diagnostics and evidence closure

Outcome:

- Scheduling semantics are visible and reviewable in diagnostics and tests.
- Evidence anchors point to both shared logic and backend integrations.

Exit criteria:

- Focused tests cover the intended semantics.
- Any ADR alignment updates are recorded with code/test anchors.
- The workstream docs describe what was intentionally deferred.

## M5 — Optional follow-up cleanup

Outcome:

- We decide whether additional cleanup belongs in this line of work or a new workstream.

Possible follow-ups:

- thinner desktop/web runner modules,
- generalized frame-resource guard usage,
- deeper timer abstraction,
- future mobile/backend reuse strategy.

Exit criteria:

- Deferred work is recorded explicitly instead of silently left behind.

## Remaining closeout blockers (from the current checkpoint)

- diagnostics writes still need a turn/frame semantic audit,
- module thinning is intentionally deferred until semantics and evidence are fully locked,
- remaining backend-local redraw logic still needs an explicit “acceptable sink wiring vs drift”
  decision.

## Recommended continuation order

1. Land a dedicated slice for diagnostics meaning and regression evidence.
2. Decide whether remaining redraw sink wiring is acceptable as-is.
3. Only then decide whether structural thinning belongs here or in a separate workstream.
