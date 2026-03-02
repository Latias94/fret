# Workspace TabStrip (Fearless Refactor v1) — Milestone 3 (Editor Semantics)

## Outcome

Add editor semantics (pinned/preview/dirty) without leaking policy into `fret-ui` or breaking
mechanism-level regression gates.

## Scope

- Pinned region model and reorder rules.
- Single preview slot (Zed-style) and commit/replace rules.
- Dirty close confirmation policy hooks (workspace-level).

## Exit criteria

- Pinned tabs:
  - reorder within pinned and within normal region works
  - cross-boundary behavior documented and gated
- Preview:
  - opening a new preview replaces previous preview unless committed
  - committing converts preview → normal tab
- Dirty close policy:
  - close action path consults policy, not tab mechanism
