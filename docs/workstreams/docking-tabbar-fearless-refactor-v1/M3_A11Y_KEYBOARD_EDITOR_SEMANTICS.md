# Docking TabBar Fearless Refactor v1 — Milestone 3 (A11y + Keyboard + Editor Semantics)

## Outcome

Docking TabBar is usable without a mouse and supports editor-grade semantics in the correct policy
layer without polluting `fret-ui`.

## Deliverables

- Keyboard navigation:
  - focus tabs
  - left/right movement
  - activate
  - close (policy-dependent)
- Focus/selection invariants:
  - focus-visible styling is consistent
  - focus restore on close/move is deterministic
- Workspace/editor semantics hooks (policy layer):
  - pinned/preview regions and rules
  - dirty indicator and close policy

## Exit criteria

- At least one keyboard-focused diag script (or unit-level key event gate) exists for tab selection.
- Parity matrix updated with editor semantics status.

