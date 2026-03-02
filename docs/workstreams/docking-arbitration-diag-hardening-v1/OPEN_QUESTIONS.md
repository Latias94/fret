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
- Current product-level gap (vs desired “returns to canonical signature”): the chained tear-off + merge-back script now
  runs through both merges but fails the final exact signature assertion. The last observed signature in a failing run:
  - `dock(root=tabs(a=1:[demo.controls,demo.viewport.right]);floatings=[])`
  - fingerprint64: `2526963005150391245` (expected `7509174212363425732`)

## 4) Should overlap-based “peek-behind” be required?

Options:

- Require `FRET_DOCK_FOLLOW_WINDOW_DURING_DRAG=1` for drag-back determinism (ImGui-style).
- Support drag-back without follow-window by requiring the cursor to physically enter the main window.

Recommendation:

- Keep both modes, but ensure the non-follow path is explicitly tested by a script that moves the
  cursor into the destination window (not just overlap).
