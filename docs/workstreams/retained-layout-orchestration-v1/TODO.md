# Retained Layout Orchestration v1 - TODO

Status: Closed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

- [x] RLO-010 [owner=planner] [deps=none] [scope=docs/workstreams/retained-layout-orchestration-v1]
  Goal: Split the retained layout orchestration follow-on from the closed layout architecture audit.
  Validation: `DESIGN.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and
  `HANDOFF.md` agree on scope.
  Evidence: `docs/workstreams/fret-ui-layout-architecture-audit-v1/CLOSEOUT_AUDIT_2026-05-18.md`.
  Handoff: Complete; implementation should start at RLO-020.

## M1 - Fresh Attribution

- [x] RLO-020 [owner=codex] [deps=RLO-010] [scope=diag,tree/layout]
  Goal: Capture a fresh retained layout orchestration baseline and classify whether the dominant
  owner is root collection, root solve scheduling, contained `ViewCache` relayout, root `Scroll`
  side effects, semantics refresh, or debug/hotspot accounting.
  Validation:
  `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json --repeat 1 --warmup-frames 5 --reuse-launch --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_RENDERER_PERF=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --dir target/fret-diag/retained-layout-orchestration-v1-baseline --launch -- cargo run -p fret-ui-gallery --release --features gallery-full`
  Evidence: `EVIDENCE_AND_GATES.md`.
  Result: Fresh baseline recorded at `target/fret-diag/retained-layout-orchestration-v1-baseline/1779080825844/bundle.schema2.json` and classified with `layout_perf_summary.json`. The dominant owner is retained root solve scheduling / root orchestration around the `Semantics` root, with `Scroll` and `ViewCache` secondary. No runtime code changed in this lane.
  Handoff: Any RLO-030 follow-on should stay narrow and start from the fresh attribution above.

## M2 - Smallest Safe Slice

- [x] RLO-030 [owner=codex] [deps=RLO-020] [scope=crates/fret-ui/src/tree/layout]
  Goal: Land the smallest behavior-preserving orchestration improvement if RLO-020 identifies one.
  Validation:
  `cargo nextest run -p fret-ui layout_engine scroll view_cache --no-fail-fast`,
  `python3 tools/check_layering.py`,
  `cargo fmt --check`.
  Evidence: code/test anchors plus before/after diag bundle paths in `EVIDENCE_AND_GATES.md`.
  Result: Added `ElementInstance::Semantics(_)` to the clean-geometry propagation fast path and
  locked the behavior with `clean_geometry_small_resize_propagates_through_semantics_wrapper`.
  The shared resize-jitter perf diff moved from `p95.total_time_us=3050` and
  `p95.layout_time_us=2479` in the baseline to `p95.total_time_us=1442` and
  `p95.layout_time_us=885` in the after bundle, while `layout_engine_solve_time_us` stayed
  roughly flat (`220` to `214`). Remaining after-sample hot owners are `Pressable`, `Scroll`, and
  `ViewCache`; split any further work from those owners into a separate follow-on.
  Handoff: Preserve `Scroll` side effects and `ViewCache` contained relayout semantics; treat the
  Semantics wrapper fast path as landed.

## M3 - Closeout Or Split

- [x] RLO-040 [owner=codex] [deps=RLO-020,RLO-030] [scope=docs/workstreams/retained-layout-orchestration-v1]
  Goal: Close this lane or split a narrower follow-on after the first attribution/proof cycle.
  Validation: closeout note and final gate set recorded.
  Evidence: `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`.
  Result: Closed the lane in `CLOSEOUT_AUDIT_2026-05-18.md`. The shipped fix is the Semantics
  wrapper fast path; remaining after-sample owners are follow-on candidates, not unfinished scope.
  Handoff: Start a new narrower lane for `Pressable`, `Scroll`, or `ViewCache` only with fresh
  attribution and side-effect proof.
