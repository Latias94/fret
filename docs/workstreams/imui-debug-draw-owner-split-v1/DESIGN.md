# ImUi Debug Draw Owner Split v1

Status: active execution lane
Last updated: 2026-05-06

Related:

- `WORKSTREAM.json`
- `M0_BASELINE_AUDIT_2026-05-06.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLOSEOUT_AUDIT_2026-05-06.md`
- `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`

This lane is a narrow follow-on from the current Dear ImGui gap-closure audit. The previous lane
closed P1 cleanup with a no-delete verdict and named `debug_draw_controls.rs` as the highest split
candidate. The problem is now structural: one file owns too many independent debug draw concerns,
which makes the next Dear ImGui-class draw-list work harder to review safely.

## Assumptions-first baseline

### 1) This is a private owner split, not a feature lane.

- Evidence:
  - `docs/workstreams/imui-imgui-gap-closure-v1/P1_CLOSEOUT_AUDIT_2026-05-06.md`
  - `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - feature work could be hidden inside a structural refactor and make public-surface review harder.

### 2) `fret-ui-kit::imui` owns the surface; `crates/fret-ui` stays unchanged.

- Evidence:
  - `docs/adr/0066-fret-ui-runtime-contract-surface.md`
  - `ecosystem/fret-ui-kit/src/imui.rs`
  - `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would drift into runtime contract widening, which needs a separate ADR-backed lane.

### 3) The public debug draw API is already compile-smoke covered.

- Evidence:
  - `ecosystem/fret-ui-kit/tests/imui_debug_draw_smoke.rs`
  - `docs/workstreams/imui-debug-draw-response-surface-v1/CLOSEOUT_AUDIT_2026-05-05.md`
  - `docs/workstreams/imui-debug-draw-command-metadata-v1/CLOSEOUT_AUDIT_2026-05-05.md`
- Confidence:
  - Confident
- Consequence if wrong:
  - the lane would need a public API smoke test before moving internals.

### 4) The safest first split is command metadata/model ownership.

- Evidence:
  - `DebugDrawCommandKind`, `DebugDrawCommandSummary`, `DebugDrawListSummary`, and the private
    `DebugDrawCommand` enum are a coherent command-model cluster.
  - `paint_debug_draw_commands(...)` and path sampling can stay in place for the first slice.
- Confidence:
  - Likely
- Consequence if wrong:
  - a later paint/path split may expose hidden coupling and require a small module boundary adjustment.

### 5) Tests can remain colocated until the implementation owners are smaller.

- Evidence:
  - the current in-file tests inspect private command variants and path samples directly.
- Confidence:
  - Likely
- Consequence if wrong:
  - a follow-up slice should split private test helpers or move smoke coverage into dedicated
    integration tests.

## Goals

1. Split `ecosystem/fret-ui-kit/src/imui/debug_draw_controls.rs` into smaller private owner modules.
2. Keep all public debug draw type names, method names, defaults, re-exports, and behavior stable.
3. Preserve the existing `fret-ui-kit` debug draw smoke/test floor.
4. Leave the next paint/path/test split candidates explicit instead of treating this as a generic
   cleanup bucket.

## Non-goals

- No public API widening or renaming.
- No `crates/fret-ui` or renderer contract changes.
- No Dear ImGui `AddCallback` equivalent.
- No raw mutable draw buffers, vertex/index buffer exposure, or backend draw-call metadata.
- No per-geometry hit-testing or editor picking behavior.

## Target owner split

### `debug_draw_controls.rs`

Owns the public facade-facing helper surface:

- `DebugDrawOptions`
- `DebugDrawInteractionOptions`
- `DebugDrawResponse`
- `ImUiDebugDrawList`
- `ImUiDebugDrawPath`
- public draw-list recording methods
- element construction and writer extension glue

### `debug_draw_controls/commands.rs`

Owns command identity and source-level metadata:

- public command summary vocabulary,
- private recorded command enum,
- command-to-summary conversion,
- aggregate list summary accounting.

### Future private owners

The next slices should be split only after the command owner is stable:

- `paint.rs`: canvas painting, image/SVG/mesh emission, clip emission.
- `paths.rs`: path sampling, arc/bezier/rect/polyline conversion helpers.
- `geometry.rs`: shared finite/empty/rounding/triangle helper math if it becomes useful after the
  paint/path split.

## Execution rules

1. Do not rename public items.
2. Do not add new public methods while moving code.
3. Keep internal visibility as narrow as possible: prefer `pub(super)` only where the parent module
   must record, summarize, or paint commands.
4. Run focused debug draw tests after every implementation slice.
