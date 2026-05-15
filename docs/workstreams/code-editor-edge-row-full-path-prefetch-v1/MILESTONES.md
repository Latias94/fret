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

Status: Diagnostics shipped on 2026-05-15; candidate-cost reduction remains open

Exit criteria:

- [x] Worst frames can identify why any newly exposed row still misses the replay plan.
- [ ] Candidate planning is biased toward edge rows or otherwise capped so `us_row_scene_prepaint_plan`
  does not erase row-content savings.
- [x] The same `ui-code-editor-resize-probes` perf surface shows whether code-editor p95 moves in the
  right direction.

Evidence:

- `docs/workstreams/code-editor-edge-row-full-path-prefetch-v1/M2_DIAGNOSTICS_2026-05-15.md`
- `target/fret-diag/code-editor-edge-row-full-path-prefetch-v1-after-m2-diagnostics-20260515/1778832028679/bundle.schema2.json`

## M3 - True Edge Payload Prebuild

Status: Recommended next code slice, but keep it code-editor-local first

M2 shows that the remaining full miss is a no-cache row at `visible_end`. Start with the smallest
code-editor-owned edge seeding path. If it requires new `CanvasPainter`, `Scene`, or framework-level
prepaint contracts, split that contract work into a separate lane before implementing it.

Exit criteria:

- The implementation seeds or prebuilds only the smallest needed visible-edge row payload.
- Any new `CanvasPainter`, `Scene`, or framework-level prepaint contract is handled in a separate
  ADR/workstream if needed.
- Code-editor paint p95 improves without a larger prepaint/layout regression.
