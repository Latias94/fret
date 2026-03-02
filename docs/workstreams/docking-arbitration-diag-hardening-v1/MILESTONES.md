# Docking arbitration diag hardening (v1) — Milestones

## M1 — Deterministic cross-window drag-back

Goal: the “tear off a tab then drag it back to the main window” scenario runs green reliably in
`--launch` mode.

Deliverables:

- A stable script (schema v2) that:
  - tears off a tab into a new OS window,
  - repositions windows into a known geometry,
  - drags the torn-off tab back into the main window,
  - asserts a stable dock graph outcome.
- A bounded evidence bundle captured near the end of the scenario.

Status (2026-03-02):

- Diagnostics/runtime hang class addressed (no more “script.result stuck running” when a target window is occluded).
- Remaining blocker is docking correctness: chained tear-off + merge-back does not yet return to the original dock graph
  fingerprint, so the “exact signature” gate fails even though windows auto-close and canonicalization passes.

Status update (2026-03-02, later):

- M1.1 delivered: runner-level pointer isolation now masks physical mouse movement when diagnostics cursor overrides are
  active (`crates/fret-launch/src/runner/desktop/runner/mod.rs`).
- M1.2 delivered: cached `test_id` predicate evaluation is bounded by freshness and emits explicit evidence
  (`ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs:297`).
- M1.3 delivered: `known_window_count_*` predicates use a runner-owned source-of-truth (`crates/fret-runtime/src/runner_window_lifecycle_diagnostics.rs`).

## M1.1 — Scripted input isolation (runner cursor override)

Goal: scripted docking drags are deterministic under user input noise.

Deliverables:

- Runner ignores physical mouse cursor updates for docking routing while an active diagnostics cursor override is in
  effect during a script run.
- A small regression script that passes even if the user moves the mouse during playback.

## M1.2 — Cached `test_id` predicate evaluation (freshness + evidence)

Goal: avoid occlusion deadlocks without introducing stale-cache false positives.

Deliverables:

- Allow `exists/not_exists` predicates by `test_id` to be evaluated from per-window cached `test_id_bounds` when the
  snapshot is recent (define and enforce a max age).
- Emit explicit evidence (event log entries) when cached evaluation is used, and when it is rejected as stale.
- Add a minimal repro script that forces an occlusion / non-redraw scenario and demonstrates cache-based evaluation
  is bounded and observable.

## M1.3 — Runner-owned window counting

Goal: diagnostics window-count gates reflect real OS window lifecycle, not “windows that happened to produce input”.

Deliverables:

- Provide a runner-owned source-of-truth for open window count/list to the diagnostics service.
- Switch `known_window_count_*` predicates to use this source-of-truth.
- Add a regression script for multi-window tear-off + auto-close under z-order churn / occlusion.

## M2 — Suite-level stability and isolation

Goal: the full `docking-arbitration` suite runs without cross-script contamination.

Deliverables:

- Launch-mode environment injection is per-script (no suite-level env leakage).
- Quarantined “known-flaky” cases are either fixed or explicitly gated (with a reason + link to evidence).
- Registry/index kept in sync; scripts are discoverable via `script_id` and suites.

## M3 — Contract + tooling closure

Goal: the diagnostics + runner contract for multi-window drags is explicit, testable, and
maintainable.

Deliverables:

- A short contract note documenting:
  - cross-window hover detection expectations,
  - drop routing semantics for scripted drags,
  - required invariants (no stuck drags, consistent window targeting).
- At least one runner-level regression test or diagnostics gate that fails fast on drift.
