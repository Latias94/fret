# Docking arbitration diag hardening (v1) — Open questions

## 1) Who owns “Drop” for scripted cross-window drags?

Options:

- Runner-owned routing (cursor screen pos + window hit test → `InternalDrag::Drop`).
- In-app diagnostics injection (emit `Drop` into a specific window and let docking resolve cross-window).

Recommendation:

- Prefer runner-owned routing for cross-window docking drags, because the runner already owns
  cross-window hover/drop semantics for docking (Enter/Over/Drop).

## 2) What is the source-of-truth coordinate space?

Options:

- Screen physical coordinates (closest to OS semantics).
- Window-client logical coordinates (easier to author, but ambiguous outside the window).
- Hybrid (author in window-client; runner integrates into screen pos).

Recommendation:

- Treat screen position as the source of truth for cross-window hover/drop routing.
- Keep window-client logical coordinates as an authoring convenience, but make the integration
  contract explicit and regression-tested.

## 2.1) Should scripted cursor overrides mask physical mouse movement?

Options:

- Yes: when diagnostics cursor override is active (and a script is running), runner ignores physical mouse movement for
  the purposes of docking hover/drop routing.
- No: physical mouse is always authoritative; scripts must ensure they are the only input source (fragile for local dev).

Recommendation:

- Yes. This is a diagnostics-only determinism requirement; it should not affect normal app input.

## 3) What should a stable “drag-back success” predicate be?

Candidates:

- Dock graph fingerprint matches the pre-tearoff layout.
- A “drop resolved” predicate in the destination window (requires window-scoped predicates or a
  cross-window predicate surface).
- A more structural gate: “tab id present in main window tabs, and tearoff window closed”.

Recommendation:

- Short term: assert dock graph fingerprint + window count.
- Medium term: add a destination-window predicate that can be awaited during a captured-pointer drag.

Status update (2026-03-02):

- `known_window_count_is(n=1)` now reflects the runner-reported open window count (rather than “windows ever seen”),
  so it is a reliable post-condition for “tear-off window auto-closed after re-dock”.
- The current drag-back gate uses:
  - `wait_until known_window_count_is(1)`,
  - `wait_until dock_graph_canonical_is(true)`,
  - and structural `dock_graph_signature_contains(...)` assertions rather than a single exact signature match.
  - Script: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-drag-tab-back-to-main.json`
- `wait_frames` now supports an optional schema v2 `window` target; this fixes overlap/z-order scripts that could stall
  when the drag source window is fully occluded and stops producing redraw callbacks.
  - Script: `tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json`

Status update (2026-03-02, later):

- Fixed a class of “`step_index` stuck (no progress) while `fretboard diag run` waits for `script.result.json`” failures:
  - Root cause: window-targeted `wait_until/assert` steps (e.g. `exists` by `test_id`) could force script migration to
    an occluded window, stalling timeouts and leaving the script permanently `running`.
  - Fix: allow `exists/not_exists` for `test_id` selectors to be evaluated against cached per-window `test_id_bounds`
    (no forced window handoff), and allow migration to follow whichever window is producing frames.
- Made `drag_pointer_until` usable as a “hold the drag, then reposition + release later” primitive even for cross-window
  dock drags by always materializing a pointer session when `release_on_success: false`.
- Improved bundle debuggability: UI debug snapshots now fall back to `WindowInteractionDiagnosticsStore::*_latest_for_window`
  when the frame-scoped snapshot is empty, so bundles reliably include `dock_graph_signature` / `dock_graph_stats`.
- Chained tear-off status: the chained tear-off + merge-back script now returns to the baseline dock graph fingerprint
  (exact fingerprint gate passes) and is included in `diag-hardening-smoke-docking`.
- Repeat-mode note: multi-window docking demos are expected to vary in window bounds + scene fingerprint across runs,
  so `diag repeat` should ignore those fields when validating determinism.

## 3.1) Is cache-based `test_id` predicate evaluation acceptable?

Context:

- Some window-targeted waits/asserts (`exists/not_exists` by `test_id`) can deadlock if they force script migration to an
  occluded window that stops producing redraw callbacks.

Options:

- Allow cached evaluation from per-window `test_id_bounds` (derived from recent semantics snapshots), with a freshness
  policy and explicit evidence.
- Disallow cached evaluation; only evaluate `exists/not_exists` on the current window snapshot (may require new script
  primitives or a runner-assisted snapshot pump).

Recommendation:

- Allow cached evaluation, but only when:
  - the cached snapshot is recent (define max age),
  - the runtime emits bounded evidence of cache hit/miss/stale,
  - and scripts that rely on this are annotated as such.

Notes:

- “Recent” should be an explicit constant (e.g. 30s) until we have a more principled snapshot liveness contract.

## 4) Should overlap-based “peek-behind” be required?

Options:

- Require `FRET_DOCK_FOLLOW_WINDOW_DURING_DRAG=1` for drag-back determinism (ImGui-style).
- Support drag-back without follow-window by requiring the cursor to physically enter the main window.

Recommendation:

- Keep both modes, but ensure the non-follow path is explicitly tested by a script that moves the
  cursor into the destination window (not just overlap).

## 5) Should internal focus snapshot invariants be fatal during diagnostics playback?

Context:

- Some docking demos have hit debug-only assertions in UI dispatch (focus snapshot / bubble chain consistency), which can
  abort the entire repro and prevent a bundle from being captured.

Options:

- Keep assertions fatal in debug: bugs surface early, but scripted repros may be fragile and hard to share.
- Downgrade to warnings during diagnostics playback only (or in harness builds): preserve the repro and capture evidence,
  but risk masking a real bug if it becomes permanent.
- Fix root cause only: no downgrades, invest in making the focus snapshot invariants always hold.

Recommendation:

- Prefer fixing the root cause. If the assertion prevents capturing docking evidence, allow a temporary diagnostics-only
  downgrade (warn + early-exit) with explicit TODOs and a follow-up issue link.

## 6) What does “layout idempotence” mean for fingerprints vs structural assertions?

Context:

- Today we rely on a fingerprint64 / exact signature to assert “returned to the pre-tearoff layout”, but failures show
  multiple plausible “almost equivalent” signatures (e.g. same panels but different split orientation / tab grouping /
  tab ordering).

Options:

- Keep exact fingerprint as the only gate (strict, but can be brittle during refactors).
- Use a tiered gate: structural containment while correctness is being fixed; exact fingerprint once stable.
- Define an equivalence relation for docking graphs (canonicalization + stable sorting) and fingerprint that.

Recommendation:

- Use tiered gates short term to localize failures, and invest in canonicalization/stable ordering so the exact
  fingerprint becomes meaningful and durable.

## 7) What is the deterministic tie-breaker for drop zone resolution?

Context:

- In failing runs, `dock_drop_resolve.resolved.zone` may land on an unexpected side even when a “center” hint is visible,
  suggesting ambiguous hit-testing or non-deterministic ordering in zone selection.

Options:

- Define a strict priority order (e.g. inner-center > inner-left/right > outer zones) when multiple hints are eligible.
- Prefer the “closest to cursor” by distance metric.
- Prefer the most recently entered hint (stateful).

Recommendation:

- Define an explicit priority order (documented + tested), and include `dock_drop_resolve` evidence in bundles so scripts
  can assert the resolved zone deterministically.
