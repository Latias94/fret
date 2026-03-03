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
- M1 is now green again after hardening scripted input + window predicates, and fixing chained tear-off merge targeting
  so the final dock graph fingerprint returns to baseline.

Status update (2026-03-02, later):

- M1.1 delivered: runner-level pointer isolation now masks physical mouse movement when diagnostics cursor overrides are
  active (`crates/fret-launch/src/runner/desktop/runner/mod.rs`).
- M1.2 delivered: cached `test_id` predicate evaluation is bounded by freshness and emits explicit evidence
  (`ecosystem/fret-bootstrap/src/ui_diagnostics/service.rs:297`).
- M1.3 delivered: `known_window_count_*` predicates use a runner-owned source-of-truth (`crates/fret-runtime/src/runner_window_lifecycle_diagnostics.rs`).
- M1.4 delivered: Windows/MSVC `docking_arbitration_demo` rebuilds no longer fail with `taffy`-related LNK2019 unresolved
  externals by forcing `taffy` to compile with a single codegen unit in dev profiles (`Cargo.toml`).
- M1.5 delivered: the chained tear-off script is now deterministic enough to be a regression gate (stable hover
  retargeting + explicit zone selection).
- M1.6 delivered: chained tear-off + merge-back returns to the pre-tearoff dock graph fingerprint (idempotence).
- M1.7 delivered: multi-window overlap z-order switching is stable again under `--timeout-ms 60000` by:
  - avoiding global dock-drag cancellation on `PointerCancel` (`ecosystem/fret-docking/src/dock/space.rs`),
  - allowing global predicate steps to run off-window (avoid occlusion stalls) (`ecosystem/fret-bootstrap/src/ui_diagnostics/script_runner.rs`),
  - canceling runner-routed cross-window drags using the internal routing pointer id on mouse-up (`crates/fret-launch/src/runner/desktop/runner/app_handler.rs`).

Stability check (2026-03-02):

- `fretboard diag repeat` passes 7x with:
  - `--env FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF=1`,
  - `--reuse-launch`,
  - `--compare-ignore-bounds --compare-ignore-scene-fingerprint` (expected to drift for multi-window demos).

Post-merge verification (2026-03-02):

- Note: when iterating on in-app diagnostics logic, rebuild the launched demo binary (`docking_arbitration_demo.exe`);
  `fretboard` itself does not rebuild the demo when using `--launch -- target/debug/...exe`.
- `fretboard diag run` is green with `--timeout-ms 60000`:
  - out dir: `target/fret-diag-step35-fix3b`
  - run id: `1772462696715`
- `fretboard diag repeat` is green 3x with:
  - `--timeout-ms 60000 --reuse-launch --compare-ignore-bounds --compare-ignore-scene-fingerprint`
  - out dir: `target/fret-diag-step35-fix3b-repeat2`

Post-merge verification (2026-03-02, after syncing `origin/main` into local `main`):

- Merge commit: `d3d97c321`.
- Base out dir: `target/fret-diag-merge-smoke`.
- `fretboard diag run` PASS with `--timeout-ms 60000`:
  - overlap z-order switch: run id `1772468892427` (session `1772468392070-85720`)
  - chained tear-off (two tabs): run id `1772468949607` (session `1772468946994-57504`)

Post-merge verification (2026-03-03):

- Prefer running the already-built binaries (instead of `cargo run`) when the workspace may be compiling in another
  terminal (avoids Cargo build-lock contention during diagnostics authoring):
  - `target/debug/fretboard.exe ... --launch -- target/debug/docking_arbitration_demo.exe`
- Chained tear-off (two tabs) is stable again:
  - `fretboard diag run` PASS with `--timeout-ms 60000`:
    - run id `1772522076604` (base out dir `target/fret-diag-chained4`, session `1772522070686-66016`)
  - `fretboard diag repeat` is green 7x with:
    - `--timeout-ms 60000 --reuse-launch --compare-ignore-bounds --compare-ignore-scene-fingerprint`
    - summary: `target/fret-diag-chained-repeat1/repeat.summary.json`
- Dock tab titles no longer disappear after a short idle delay (~2s) during diagnostics runs:
  - Symptom: tab labels vanish while SVG close icons remain (retained docking tabs caching text blobs across frames).
  - Root cause: `DockSpace` cached prepared tab titles but did not rebuild when `TextFontStackKey` changed (system font
    rescan / font stack stabilization), so cached `TextBlobId`s became stale.
  - Fix: include `TextFontStackKey` in the tab-title rebuild cache key.
    - implementation: `ecosystem/fret-docking/src/dock/space.rs`
    - evidence repro: `tools/diag-scripts/docking/arbitration/local-debug/docking-arbitration-demo-tab-text-disappears-after-2s-single-window.json`
      - before: run id `1772532891331` (idle t2s screenshot missing titles)
      - after: run id `1772534011243` (idle t2s screenshot retains titles)

Hover peek-behind hardening (2026-03-02):

- Runner hover routing consumes `window_under_moving_window` when transparent payload is requested (or follow is active),
  so docking previews/resolve can target the overlapped window under a moving payload window.
  - implementation: `crates/fret-launch/src/runner/desktop/runner/event_routing.rs`
- Updated gates (all PASS with `--timeout-ms 60000`):
  - transparent payload z-order switch: run id `1772470944468` (`target/fret-diag-transparent-payload-smoke4`)
  - under-moving-window peek-behind: run id `1772471076428` (`target/fret-diag-under-moving-hover4`)
  - overlap z-order switch (non-transparent): run id `1772471119364` (`target/fret-diag-post-peek`)

Stage gates for merge-back correctness (2026-03-03):

- Chained tear-off: added per-drop `dock_drop_resolved_*` gates + bounded bundles to pinpoint where a panel is lost
  (drop vs auto-close/cleanup).
  - PASS: run id `1772493305362` (`target/fret-diag-stage-gates2`)
- Chained tear-off: added additional bundles right after each auto-close window-count gate (explicit “after close” stage).
  - PASS: run id `1772494218337` (`target/fret-diag-stage-gates5`)
- Transparent payload drag-back: switched the merge-back targeting to `dock-arb-hint-inner-right` to avoid `wrap_binary`
  outcomes from outer-hint drops, and added a drop-stage bundle.
  - PASS: run id `1772493899790` (`target/fret-diag-stage-gates4`)
- Chained tear-off: removed trailing `wait_frames` after the final `capture_bundle` to avoid “script.result timeout”
  when the last remaining window is occluded/idle and stops producing redraw callbacks.
  - PASS: run id `1772495444909` (`target/fret-diag-chained-check2`)
- Diagnostics runtime: arm a keepalive timer while scripts are active so `wait_frames` / `wait_until` can progress (or
  fail with `timeout.no_frames`) even when redraw callbacks stop (occlusion/idle).
  - PASS: run id `1772497918062` (`target/fret-diag-chained-postfix`)
- Diagnostics runtime: allow a small “burst” of frame-independent tail steps per drive call so scripts do not require an
  additional rendered frame to execute a final `capture_bundle` after the last semantic assertion (avoids launch-mode
  timeouts under occlusion/idle throttling and tight tooling budgets).
  - implementation: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_engine.rs`
- Tear-off + drag-back loop script hardening:
  - `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-tearoff-merge-loop-no-leak.json` now
    captures stage bundles (`loop-cycle*-drop` and `loop-cycle*-after-merge` after `known_window_count_is n=1`) and uses
    open-window semantics consistently across cycles.
  - Repeat check is green 20x with `--reuse-launch --compare-ignore-bounds --compare-ignore-scene-fingerprint`:
    `target/fret-diag-docking-loop-repeat2`.

## M1.4 — Rebuild reliability for docking demos (Windows/MSVC)

Goal: docking demo binaries used by `--launch` diagnostics can be rebuilt reliably in local dev.

Deliverables:

- Document the observed failure mode (toolchain + target + error signature).
- Unblock building `docking_arbitration_demo` (and related docking demos) from a clean-ish state on Windows/MSVC.
- Add a short “if this regresses” note (what to try, what evidence to attach to an issue).

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

## M1.5 — Deterministic chained tear-off repro (two tabs, two merges)

Goal: the chained tear-off repro is deterministic enough to be used as a stable regression gate (even if it currently
fails on correctness).

Deliverables:

- A schema v2 script that:
  - tears off a first tab, drags it back, waits for auto-close,
  - tears off a second tab, drags it back, waits for auto-close,
  - emits clear evidence for each merge (drop resolve zone, dock graph signature/stats).
- Intermediate structural gates that pinpoint where a panel is lost or a split/tab shape changes.

## M1.6 — Chained tear-off correctness (layout idempotence)

Goal: after the chained tear-off + merge-back sequence, the dock graph returns to the pre-tearoff fingerprint.

Deliverables:

- Docking model/ops + interaction arbitration yield an idempotent outcome for tear-off → merge-back cycles (no panel
  loss; stable canonicalization; stable ordering where applicable).
- The chained repro script passes with the exact fingerprint gate (not just `contains`).

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
