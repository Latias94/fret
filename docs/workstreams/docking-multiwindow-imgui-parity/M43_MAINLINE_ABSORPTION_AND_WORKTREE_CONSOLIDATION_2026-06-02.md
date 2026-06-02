# M43 Mainline Absorption and Worktree Consolidation (2026-06-02)

## Summary

After the latest `main` updates and pull reconciliation, the historical worktrees used for the IMUI
editor-grade refactor and the workstream lane registry no longer carry unique code or documentation
history relative to `main`.

This milestone records the consolidation decision so the lane can continue from the canonical
`F:\\SourceCodes\\Rust\\fret` main worktree instead of splitting attention across stale worktrees.

## Findings

1. `imui-imgui-editor-grade-refactor` is fully absorbed by `main`.
   - `git rev-list --left-right --count main...imui-imgui-editor-grade-refactor` returned `906 0`.
   - `git log --left-right main...imui-imgui-editor-grade-refactor` showed only `main`-side commits.
   - The worktree at `F:\\SourceCodes\\Rust\\fret-worktrees\\imui-imgui-editor-grade-refactor` was
     clean and had no uncommitted changes.
2. `docs/workstream-lane-registry` is fully absorbed by `main`.
   - `git rev-list --left-right --count main...docs/workstream-lane-registry` returned `359 0`.
   - The worktree at `F:\\SourceCodes\\Rust\\fret-worktrees\\fret-workstream-lanes` was also clean
     and had no uncommitted changes.
3. The primary repository at `F:\\SourceCodes\\Rust\\fret` had no unresolved merge conflicts after
   the latest pull.

## Decision

- Treat `main` as the sole active baseline for the current IMUI docking/editor-grade continuation.
- Do not attempt a content merge from either historical worktree branch, because there is no unique
  content left to merge.
- It is now safe to prune the absorbed worktrees after recording this milestone.

## Consequences

- Future progress for this lane should be recorded and implemented from the main worktree unless a
  new isolated worktree is intentionally opened for a fresh slice.
- If a new worktree is created later, it should start from current `main`, not from the absorbed
  `imui-imgui-editor-grade-refactor` or `docs/workstream-lane-registry` branches.

## Evidence

- `git worktree list --porcelain`
- `git status --short --branch`
- `git rev-list --left-right --count main...imui-imgui-editor-grade-refactor`
- `git rev-list --left-right --count main...docs/workstream-lane-registry`
- `git log --oneline --decorate --graph --left-right main...imui-imgui-editor-grade-refactor -n 80`
- `git log --oneline --decorate --graph --left-right main...docs/workstream-lane-registry -n 40`
