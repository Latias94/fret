# Milestones

## M0 - Baseline From Prior Lane

Status: Done by inherited closeout

Baseline source:

- `docs/workstreams/code-editor-row-content-snapshot-cache-v1/CLOSEOUT_AUDIT_2026-05-15.md`
- worst bundle:
  `target/fret-diag/code-editor-row-content-snapshot-cache-v1-after-m2-20260515/1778827921081/bundle.schema2.json`

Exit criteria:

- The prior lane is treated as closed.
- This lane owns only the remaining edge-row full-path tail.

## M1 - Cached Plain Row Replay Planning

Status: Done on 2026-05-15

Shipped:

- Plain row scene cache entries can enter the prepaint replay plan even without syntax replay keys.
- Focused resize test proves planned plain rows are consumed in paint without row text work.

Evidence:

- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/M1_PLAIN_CACHED_REPLAY_2026-05-15.md`
- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m1-20260515/1778830062977/bundle.schema2.json`

## M2 - Edge Miss Taxonomy And Candidate Cost

Status: Diagnostics and syntax-key mismatch reduction shipped on 2026-05-15; edge-aware candidate
selection remains open

Exit criteria:

- [x] Worst frames can identify why any newly exposed row still misses the replay plan.
- [ ] Candidate planning is biased toward edge rows or otherwise capped so `us_row_scene_prepaint_plan`
  does not erase row-content savings.
- [x] The same `ui-code-editor-resize-probes` perf surface shows whether code-editor p95 moves in the
  right direction.

Evidence:

- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/M2_DIAGNOSTICS_2026-05-15.md`
- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/M2_FOLLOWUP_SYNTAX_REPLAY_KEY_2026-05-15.md`
- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m2-diagnostics-20260515/1778832028679/bundle.schema2.json`
- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-syntax-key-content-eq-20260515/1778835965902/bundle.schema2.json`

## M3 - True Edge Payload Prebuild

Status: Done on 2026-05-15

Shipped:

- `CanvasPrepaintCx` can prepare hosted text resources and replayable scene fragments through a
  scratch `CanvasPrepaintPainter`.
- The code editor prebuilds the missing visible-end row scene payload before replay planning.
- Paint diagnostics distinguish paint row-scene stores from prepaint edge-row stores.

Exit criteria:

- [x] The implementation seeds or prebuilds only the smallest needed visible-edge row payload.
- [x] Any framework-level prepaint contract change is narrow and covered by ADR alignment evidence.
- [x] Code-editor paint no-entry/full-miss/store counters disappear in the resize perf surface.

Evidence:

- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/M3_EDGE_ROW_PREBUILD_2026-05-15.md`
- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/CLOSEOUT_AUDIT_2026-05-15.md`
- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m3-edge-prebuild-diagnostics-split-20260515/1778841130928/bundle.schema2.json`

## Closeout

Status: Closed on 2026-05-15

The original edge-row paint full-path objective is complete. Further planner-cost, renderer, or
generic virtual-surface work should start as a separate follow-on lane.
