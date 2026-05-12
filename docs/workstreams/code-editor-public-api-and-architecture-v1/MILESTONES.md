# Milestones

Status: Active milestone plan
Last updated: 2026-05-12

## M0 - Baseline Audit

Status: Complete (2026-05-12)

Exit criteria:

- `WORKSTREAM.json` exists and names the first-open docs.
- Baseline audit records current exports, crate size hazards, direct dependencies, and known gaps.
- Workstream catalog is updated.
- JSON/catalog/diff hygiene gates pass.

## M1 - Public Surface Classification

Status: In progress

Exit criteria:

- Every public item in `fret-code-editor`, `fret-code-editor-buffer`, and
  `fret-code-editor-view` is classified as stable, experimental, or internal-by-accident. (Initial
  pass complete: `PUBLIC_SURFACE_CLASSIFICATION_2026-05-12.md`.)
- `CodeEditorHandle` has a method-by-method owner table. (Initial pass complete:
  `PUBLIC_SURFACE_CLASSIFICATION_2026-05-12.md`.)
- First proposed removals/renames/moves include migration notes. (`Selection` ownership move
  recorded in `M1_SELECTION_OWNERSHIP_CONTRACT_2026-05-12.md`.)
- Command/keymap/undo ownership is recorded before adding more command-facing editor features.
  (`M1_COMMAND_KEYMAP_UNDO_BOUNDARY_2026-05-12.md`.)
- Minimum app-author docs exist for embedding the current surface without reading internal modules.
  (`docs/code-editor.md`.)
- Focused compile/tests cover unchanged current examples. (Initial public-signature re-export test:
  `ecosystem/fret-code-editor/tests/public_surface.rs`.)

## M2 - Target Extension Package

Status: In progress

Exit criteria:

- Diagnostics, decorations, gutter markers, and semantic tokens have target data contracts.
  (Diagnostic range contract started in `M2_DIAGNOSTIC_SPAN_CONTRACT_2026-05-12.md`;
  logical-line summary projection added in
  `M2_DIAGNOSTIC_LINE_SUMMARY_CONTRACT_2026-05-12.md`; gutter marker payload contract added in
  `M2_GUTTER_MARKER_CONTRACT_2026-05-12.md`; range decoration payload contract added in
  `M2_RANGE_DECORATION_CONTRACT_2026-05-12.md`; semantic token input contract added in
  `M2_SEMANTIC_TOKEN_CONTRACT_2026-05-12.md`; hover/completion/code-action overlay ownership
  boundary added in `M2_OVERLAY_FEATURE_BOUNDARY_2026-05-12.md`.)
- Coordinate ownership is explicit and shared with `DisplayMap`. (Diagnostic v1 uses
  `TextBuffer` UTF-8 byte ranges; semantic tokens and range decorations also use `TextBuffer`
  UTF-8 byte ranges; line summaries use logical line indexes. Display-row projection for gutter
  markers validates against `DisplayMap::row_count`; the cross-feature coordinate vocabulary is
  recorded in `M2_COORDINATE_VOCABULARY_2026-05-12.md`.)
- Widget-facing payload storage, public setters/readouts, revision/display-map invalidation, and
  diagnostics snapshot counts exist. (`M2_WIDGET_FEATURE_PAYLOAD_SURFACE_2026-05-12.md`.)
- At least one UI Gallery/example surface can exercise the target package.
- A no-buffer-mutation gate exists for pure decoration toggles.

## M3 - First API Cleanup Slice

Exit criteria:

- At least one public API cleanup lands with a surface-diff note.
- Debug-only APIs are either clearly named as debug/staging or moved away from the default public
  path.
  - Module/test ownership improves without changing behavior. (First test split landed in
    `M3_TEST_MODULE_FIRST_SPLIT_2026-05-12.md`; feature payload store/snapshot split landed in
    `M3_FEATURE_PAYLOAD_STORE_MODULE_SPLIT_2026-05-12.md`; diagnostics/perf snapshot split landed in
    `M3_DIAGNOSTICS_SNAPSHOT_MODULE_SPLIT_2026-05-12.md`; state schema split landed in
    `M3_STATE_SCHEMA_MODULE_SPLIT_2026-05-12.md`; state methods split landed in
  `M3_STATE_METHODS_MODULE_SPLIT_2026-05-12.md`; state initializer split landed in
  `M3_STATE_INITIALIZER_BOUNDARY_2026-05-12.md`; handle module split landed in
  `M3_HANDLE_MODULE_SPLIT_2026-05-12.md`; handle method boundary split landed in
  `M3_HANDLE_METHOD_BOUNDARY_SPLIT_2026-05-12.md`; a11y window/mapping split landed in
  `M3_A11Y_MODULE_SPLIT_2026-05-12.md`; the first input owner slice moved edit transactions,
  IME delete-surrounding, undo/redo, and row-geom cache shifting into
    `M3_INPUT_EDIT_TRANSACTION_SPLIT_2026-05-12.md`; clipboard effects moved into
    `M3_INPUT_CLIPBOARD_SPLIT_2026-05-12.md`; keyboard dispatch moved into
    `M3_INPUT_KEYBOARD_SPLIT_2026-05-12.md`; navigation, pointer selection, and final input boundary
    cleanup landed in `M3_INPUT_NAVIGATION_POINTER_SPLIT_2026-05-12.md`; the first syntax owner
    slice moved prefetch runtime types into
    `M3_SYNTAX_PREFETCH_RUNTIME_SPLIT_2026-05-12.md`; the syntax cache owner split landed in
    `M3_SYNTAX_CACHE_MODULE_SPLIT_2026-05-12.md`; the paint row-scene cache/replay/store owner
    split landed in `M3_PAINT_ROW_SCENE_MODULE_SPLIT_2026-05-12.md`.)
- `cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast` passes.

## M4 - Feature Surface Proof

Status: Complete (2026-05-12)

Exit criteria:

- A realistic editor proof combines syntax, diagnostics/decorations, gutter markers, folds/inlays,
  soft wrap, selection, and at least one overlay-style feature hook. (First fixture proof landed in
  `M4_UI_GALLERY_FEATURE_PAYLOAD_FIXTURE_2026-05-12.md`; the bundle assertion landed in
  `M4_FEATURE_PAYLOAD_BUNDLE_ASSERTION_2026-05-12.md`; the anchored overlay hook landed in
  `M4_OVERLAY_FEATURE_HOOK_PROOF_2026-05-12.md`.)
- Diagnostics snapshots expose enough state to explain regressions.
- The proof has one scripted repro and one public `diag stats` gate. (The overlay hook is covered by
  a render-flow gate; the feature payload bundle remains the public `diag stats` contract.)

## M5 - Performance Contract Closure

Status: Complete for the current non-Linux editor contract surface (2026-05-12)

Exit criteria:

- Feature-heavy editor stressors have p50/p95/max and renderer payload baselines.
  (`M5_PERF_CONTRACT_CLOSURE_2026-05-12.md` maps resize, autoscroll steady, autoscroll typical,
  and complex wheel to checked-in baselines or formal evidence surfaces.)
- Changes to editor paint/layout/rendering can be reviewed against existing complex wheel,
  autoscroll, resize, and payload contracts.
  (`M5_PERF_CONTRACT_CLOSURE_2026-05-12.md` records the gate rule for future hot-path changes.)
- No broad renderer/windowed-surface rewrite is started without failing or near-threshold evidence.
  (Current row-scene replay/store evidence keeps broad renderer, CanvasPainter, and
  WindowedRowsSurface rewrites blocked until a future stressor proves the limiter.)
