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

## Next (proposed)

- Extract a shared `TabStripController` into `ecosystem/fret-ui-kit/` (policy toolbox):
  - shared close-vs-activate arbitration hooks
  - shared “active stays reachable/visible” scroll policy helpers
  - adapter-specific policy remains in `fret-workspace` vs `fret-docking`
- Wire workspace tab strip to use the controller first (lower multi-window complexity), then docking.
- Add/refresh diag gates that assert outcomes rather than ordering (avoid tab-order being treated as a contract).
