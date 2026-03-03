# Editor TabStrip Unification Fearless Refactor v1 (Rolling Log)

This file is a short, append-only log of landings and decisions for this workstream.

## 2026-03-03

- Landed docking arbitration diagnostics hardening (multi-window tear-off robustness + better failure reasons).
  - Commits: `a0116acbb`, `4bf3ad09d`
  - Evidence: `docs/workstreams/docking-tabbar-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Made schema v2 `wait_until` tolerate missing `timeout_frames` by defaulting to `default_action_timeout_frames()`.
  - Commit: `6f9d2df4b`
  - Rationale: reduces script authoring footguns; aligns with other v2 steps that already default.
- Introduced a shared headless “ensure visible” helper for tab strips and wired it in both adapters.
  - Code: `ecosystem/fret-ui-headless/src/tab_strip_scroll.rs`, `ecosystem/fret-workspace/src/tab_strip/utils.rs`,
    `ecosystem/fret-docking/src/dock/tab_bar_geometry.rs`
  - Rationale: keep workspace and docking aligned on the same scroll-to-visible math, so refactors remain fearless.
- Introduced a shared clamped edge auto-scroll helper and started converging adapters onto it.
  - Code: `ecosystem/fret-dnd/src/scroll.rs`, `ecosystem/fret-workspace/src/tab_strip/kernel.rs`,
    `ecosystem/fret-docking/src/dock/space.rs`
  - Rationale: keep drag-to-scroll behavior consistent (and easier to gate) across workspace and docking.
- Extracted an overflow dropdown item selection helper (policy remains adapter-owned).
  - Code: `ecosystem/fret-ui-headless/src/tab_strip_overflow_menu.rs`, wired in
    `ecosystem/fret-workspace/src/tab_strip/overflow.rs` and `ecosystem/fret-docking/src/dock/tab_overflow.rs`
  - Rationale: reduce drift in “which indices appear in the overflow dropdown” while keeping per-adapter defaults.
- Hardened workspace tab close arbitration so clicking the close affordance does not arm the parent tab pressable.
  - Code: `ecosystem/fret-workspace/src/tab_strip/interaction.rs`, `ecosystem/fret-workspace/src/tab_strip/mod.rs`
  - Gate: `tools/diag-scripts/workspace/shell-demo/workspace-shell-demo-tab-close-button-does-not-activate.json`
  - Rationale: editor-grade behavior expects "close without activation" to be reliable, even when close is nested.
- Made pressable hover edges mark view-cache roots as needing rerender.
  - Code: `crates/fret-ui/src/tree/dispatch/hover.rs`, `crates/fret-ui/src/tree/debug/invalidation.rs`
  - Test: `crates/fret-ui/src/declarative/tests/layout/interactivity.rs` (`pressable_hover_marks_view_cache_root_dirty_on_hover_edges`)
  - Rationale: components that mount/unmount children based on `PressableState::hovered` must remain deterministic under view caching.

## Next (proposed)

- Extract a shared `TabStripController` into `ecosystem/fret-ui-kit/` (policy toolbox):
  - shared close-vs-activate arbitration hooks
  - shared “active stays reachable/visible” scroll policy helpers
  - adapter-specific policy remains in `fret-workspace` vs `fret-docking`
- Wire workspace tab strip to use the controller first (lower multi-window complexity), then docking.
- Add/refresh diag gates that assert outcomes rather than ordering (avoid tab-order being treated as a contract).
