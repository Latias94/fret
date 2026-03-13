# Fret Launch Runner Scheduling (Fearless Refactor v1) — Milestones

Status: Draft

Last updated: 2026-03-13

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
