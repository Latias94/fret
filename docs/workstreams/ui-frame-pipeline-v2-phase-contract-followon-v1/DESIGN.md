# UI Frame Pipeline v2 Phase Contract Follow-On v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

ADR 0327 is accepted and the broad Frame Pipeline v2 lane is closed with known follow-ons. The next
risk is regression by helper accretion: new surfaces can accidentally mix build, layout, prepaint,
paint, and commit responsibilities again. This follow-on chooses one additional surface and proves
the explicit phase contract there.

## Target State

- One non-closed proof surface reports or tests explicit phase ownership.
- Any helper introduced by the slice is named by phase responsibility.
- Old helper-only shortcuts are either deleted or documented as retained mechanisms with reason.

## First Slice

Pick one surface outside the original closeout proof pair, then add a focused test or diagnostics
counter that answers: which phase owns the state, what dirties it, and what reuses it?

## Non-goals

- Reopening `ui-frame-pipeline-v2-fearless-refactor-v1`.
- Rewriting renderer `Scene` recording.
- Running a broad performance campaign before one correctness proof exists.
