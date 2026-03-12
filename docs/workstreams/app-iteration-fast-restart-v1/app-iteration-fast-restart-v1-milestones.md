---
title: App Iteration Fast Restart + State Restore (v1) — Milestones
status: in_progress
date: 2026-02-15
scope: native-first devloop; docking + multi-window follow-up
---

# App Iteration Fast Restart + State Restore (v1) — Milestones

Workstream entry:

- `docs/workstreams/app-iteration-fast-restart-v1/app-iteration-fast-restart-v1.md`

This milestone plan defines “done” in terms of observable outcomes (demos / diagnostics evidence),
not internal implementation details.

## Status (2026-02-15)

Implemented on branch `ws/app-iteration-fast-restart-v1` (worktree); not merged to `main` yet.

- M0–M4 are functionally complete for native devloop iteration.
- Remaining follow-ups are mostly polish / diagnostics surfacing (see TODO tracker).

## M0 — Scope locked (contract + UX)

Definition of done:

- Workstream docs exist (design + TODO + milestones).
- A single recommended default command is documented:
  - `fretboard dev native --bin <app> --watch`
- The dev-state file location and versioning rules are explicit.

## M1 — Window geometry restore (baseline)

Definition of done:

- On restart, the primary window restores:
  - size
  - position (best-effort; may clamp to visible work area)
- Restore is clearly reported in logs/output (ok/partial/ignored).

Evidence:

- Manual repro recipe in a demo.
- At least one unit test for schema parsing + version handling.

Suggested carrier:

- `apps/fret-demo/src/bin/todo_demo.rs`

## M2 — Docking layout restore (editor leverage)

Definition of done:

- A docking-heavy demo restarts into the same docking layout:
  - split geometry (approximate is ok)
  - active tab selection
- Layout restore is best-effort and is skipped when incompatible.

Evidence:

- Demo repro recipe plus a short “what was restored” log line.

Suggested carrier:

- `apps/fret-demo/src/bin/docking_demo.rs`

## M3 — Opt-in app-owned model restore

Definition of done:

- The golden path exposes opt-in export/import hooks for app-owned state.
- One demo opts in and proves model state is preserved across restarts.

Evidence:

- Demo repro recipe.
- A test that import failure does not prevent startup.

Suggested carrier:

- `apps/fret-demo/src/bin/todo_demo.rs`

## M4 — Multi-window restore (best-effort)

Definition of done:

- A multi-window demo restores:
  - the set of windows
  - their role keys
  - their geometry (best-effort)

Evidence:

- Demo repro recipe.
