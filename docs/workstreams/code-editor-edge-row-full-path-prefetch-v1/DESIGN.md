# Code Editor Edge Row Full Path Prefetch V1

Date: 2026-05-15
Status: Active after M1

## Goal

Reduce live-resize paint tail cost when a newly exposed code-editor viewport edge row falls outside
the prepaint replay plan and has to walk the full row content, rich text, geometry, and row scene
path in paint.

This is a narrow follow-on to `code-editor-row-content-snapshot-cache-v1`. That lane made replay-hit
rows cheap by sharing `Arc<RowContentSnapshot>` through row text cache, row scene cache, replay
plans, and paint. The remaining useful work should stay inside `ecosystem/fret-code-editor` unless
perf evidence points elsewhere.

## Assumptions

- Area: lane ownership
  - Assumption: this lane is a follow-on, not a reopen of the closed row-content snapshot lane.
  - Evidence: `docs/workstreams/code-editor-row-content-snapshot-cache-v1/CLOSEOUT_AUDIT_2026-05-15.md`.
  - Confidence: Confident.
  - Consequence if wrong: new work could blur the closed lane's shipped verdict.
- Area: architecture scope
  - Assumption: no broad `crates/fret-ui` layout, view-cache, `Scroll`, or `VirtualList` refactor is
    justified by the current evidence.
  - Evidence: prior closeouts show code-editor row content/scene fields as the dominant owned
    subfields, while view-cache and layout counters are not the primary tail.
  - Confidence: Confident.
  - Consequence if wrong: this lane would under-address a framework-level bottleneck.
- Area: M1 behavior
  - Assumption: M1 only covers cached plain rows that already have row scene cache entries; it does
    not prebuild scene ops for completely uncached edge rows.
  - Evidence: `replay_row_scene_plan_candidates_for_frame` accepts `RowScenePaintKey::Plain` cache
    entries but still starts from `st.row_scene_cache.get(&row)`.
  - Confidence: Confident.
  - Consequence if wrong: the lane would overclaim the shipped optimization.
- Area: next optimization pressure
  - Assumption: the next useful slice is to make candidate planning more edge-aware and to classify
    the remaining miss rows before adding larger prebuild mechanics.
  - Evidence: M1 improved code-editor p95 and row-content sums, but `us_row_scene_prepaint_plan`
    increased from `70us` to `111us` in the worst bundle.
  - Confidence: Likely.
  - Consequence if wrong: a candidate-planning slice may save less than directly prebuilding full
    edge-row payloads.

## Owning Layer

- In scope:
  - `ecosystem/fret-code-editor` row content snapshots,
  - row rich cache lookup and prefetch,
  - row geometry key/cache validation,
  - row scene cache and prepaint replay plan construction,
  - code-editor paint perf counters and focused tests.
- Out of scope:
  - `crates/fret-ui` layout semantics,
  - view-cache containment contracts,
  - `Scroll` / `VirtualList` behavior,
  - renderer scene encoding or text atlas behavior,
  - public code-editor API changes.

## M1 Shipped Shape

M1 expands prepaint replay planning so cached plain row scene entries can participate even when they
do not have a syntax replay key. Syntax rows still use syntax replay validation. Plain rows validate
against `RowSceneKey::plain(cached_row_geom_key, fg)`, intentionally reusing the cached entry's
`RowGeomKey` to avoid pointer-identity drift from rebuilding `AttributedText`.

The focused regression test uses blank Rust-language rows so the first frame seeds plain row scene
entries. The second frame resizes the editor and asserts that prepaint plans plain cached rows,
paint consumes the plan, row text work stays out of paint, and scene hits cover the planned rows.

## Perf Read

Compared with the previous row-content snapshot worst bundle:

- worst-bundle `code_editor_paint_perf.p95.us_total`: `394us` -> `371us`
- `us_row_content_resolve`: `305us` -> `281us`
- `us_row_text`: `12us` -> `6us`
- `us_row_rich_cache_compare`: `23us` -> `20us`
- `us_row_geom_key`: `55us` -> `53us`
- `us_row_scene_prepaint_plan`: `70us` -> `111us`

Interpretation: M1 moves more cached rows into the replay-hit shape and reduces total code-editor
row-content work, but it also makes replay-plan construction heavier. The next step should reduce
candidate-planning cost and classify remaining miss rows before adding broader prebuild behavior.

## Non-Goals

- Do not claim M1 prebuilds brand-new edge row scene ops.
- Do not move row scene construction into a global `fret-ui` prepaint contract.
- Do not widen `CanvasPainter` or `Scene` APIs only for this lane without a separate design.
- Do not loosen perf baselines silently.
