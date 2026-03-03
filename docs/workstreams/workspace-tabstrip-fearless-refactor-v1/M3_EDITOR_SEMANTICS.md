# Workspace TabStrip (Fearless Refactor v1) — Milestone 3 (Editor Semantics)

## Outcome

Add editor semantics (pinned/preview/dirty) without leaking policy into `fret-ui` or breaking
mechanism-level regression gates.

## Scope

- Pinned region model and reorder rules.
- Single preview slot (Zed-style) and commit/replace rules.
- Bulk-close commands (close left/right/others) with pinned protection.
- Dirty close confirmation policy hooks (workspace-level).

## Exit criteria

- Pinned tabs:
  - reorder within pinned and within normal region works
  - cross-boundary behavior documented and gated
- Preview:
  - opening a new preview replaces previous preview unless committed
  - committing converts preview → normal tab
- Bulk close:
  - close-left/close-right/close-others do not close pinned tabs
- Dirty close policy:
  - close action path consults policy, not tab mechanism
  - Evidence: `ecosystem/fret-workspace/src/close_policy.rs` and `ecosystem/fret-workspace/src/tabs.rs`
  - Demo-only prompt (optional UX proof):
    - `apps/fret-examples/src/workspace_shell_demo.rs`
    - `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-dirty-shows-prompt-and-discard-closes-smoke.json`
