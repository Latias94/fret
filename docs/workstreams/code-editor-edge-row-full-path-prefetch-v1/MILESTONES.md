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

Status: Pending

Exit criteria:

- Worst frames can identify why any newly exposed row still misses the replay plan.
- Candidate planning is biased toward edge rows or otherwise capped so `us_row_scene_prepaint_plan`
  does not erase row-content savings.
- The same `ui-code-editor-resize-probes` perf surface shows whether code-editor p95 moves in the
  right direction.

## M3 - True Edge Payload Prebuild

Status: Conditional

Start only if M2 shows that edge rows still need a prebuilt payload and the missing work cannot be
covered by cheaper replay-plan candidate selection.

Exit criteria:

- The implementation prebuilds only the smallest needed edge-row payload.
- Any new `CanvasPainter`, `Scene`, or framework-level prepaint contract is handled in a separate
  ADR/workstream if needed.
- Code-editor paint p95 improves without a larger prepaint/layout regression.
