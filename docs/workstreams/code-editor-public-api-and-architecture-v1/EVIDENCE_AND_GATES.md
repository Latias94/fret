# Evidence and Gates

Status: Active gate list
Last updated: 2026-05-12

## First-Open Repro

```powershell
python tools/audit_crate.py --crate fret-code-editor
rg -n "pub (struct|enum|trait|type|fn)|pub use|pub mod" ecosystem/fret-code-editor/src ecosystem/fret-code-editor-buffer/src ecosystem/fret-code-editor-view/src
rg -n "0185|code editor ecosystem v1|fret-code-editor" docs/adr docs/workstreams -g "*.md"
```

Use this before public API changes to confirm the current surface and evidence anchors.

## Documentation Gates

```powershell
python -m json.tool docs/workstreams/code-editor-public-api-and-architecture-v1/WORKSTREAM.json
python tools/check_workstream_catalog.py
git diff --check
```

## Code Editor Gates

Run these when a slice changes editor source or public API:

```powershell
cargo fmt -p fret-code-editor-view --check
cargo check -p fret-code-editor-view
cargo nextest run -p fret-code-editor-view --lib --no-fail-fast
cargo fmt -p fret-code-editor-buffer --check
cargo check -p fret-code-editor-buffer
cargo nextest run -p fret-code-editor-buffer --lib --no-fail-fast
cargo fmt -p fret-code-editor --check
cargo check -p fret-code-editor --features syntax-rust
cargo nextest run -p fret-code-editor --lib --features syntax --no-fail-fast
cargo nextest run -p fret-code-editor --test public_surface --no-fail-fast
cargo check -p fret-ui-gallery
python tools/check_layering.py
```

## Feature Payload Bundle Gate

Run this after capturing a UI Gallery code-editor torture bundle with feature payloads:

```powershell
cargo run -p fretboard -- diag stats <bundle.schema2.json> --warmup-frames 5 --check-ui-gallery-code-editor-torture-feature-payloads-stable
```

The command writes
`check.ui_gallery_code_editor_torture_feature_payloads_stable.json` next to the bundle. It requires
stable non-zero diagnostics, diagnostic line summaries, range decorations, gutter markers, semantic
tokens, schema fields, buffer revision, and display-map epoch after warmup.

## Overlay Hook Render-Flow Gate

Run this after changing the `code_editor_torture` overlay proof or the editor assist recipe wiring:

```powershell
cargo nextest run -p fret-ui-gallery --features gallery-dev code_editor_torture_assist_overlay_hook_stays_within_window --no-fail-fast
```

The gate opens the code-editor torture page, clicks the assist trigger, and asserts that the
recipe-owned anchored listbox and its first feature-hook row enter the semantics tree with in-window
geometry.

## Perf Gates

Use the existing editor perf contract workstream as the first source of truth:

- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-audit.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-contract-matrix.md`
- `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`

Hot-path editor changes should include:

- p50/p95/max top-frame or frame-p95 evidence,
- renderer payload fields where relevant,
- paint/detail attribution when the change touches text, row scenes, Canvas replay, or syntax
  materialization.

The lane-level M5 closure is recorded in
`docs/workstreams/code-editor-public-api-and-architecture-v1/M5_PERF_CONTRACT_CLOSURE_2026-05-12.md`.
It maps the current resize, autoscroll steady, autoscroll typical, complex wheel, and row-scene
replay/store surfaces to the perf workstream evidence and blocks broad renderer/windowed-surface
rewrites without failing or near-threshold evidence.

Do not reseed thresholds solely because a new implementation is faster or slower. Reseed only after
the target behavior and stressor scope are explicit.

For paint attribution probes, set `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1` so
`app_snapshot.code_editor.torture.paint_perf` is emitted by
`apps/fret-ui-gallery/src/driver/diag_snapshot.rs`.

Latest resize probe with paint attribution enabled:

- `target/fret-diag-resize-probes-gate-1778534548/summary.json`
- `target/fret-diag-resize-probes-gate-1778534548/attempt-1/1778534561410/bundle.json`
- `code_editor.paint_perf` p95 total `655us`, content `458us`, text `33us`, fast_path `282us`

## Evidence Anchors

- ADR split: `docs/adr/0185-code-editor-ecosystem-v1.md`
- Display fragment composition: `docs/adr/0188-code-editor-display-fragments-and-displaymap-composition-v1.md`
- Text navigation: `docs/adr/0179-text-navigation-and-word-boundaries-v1.md`
- Platform text input: `docs/adr/0261-platform-text-input-client-interop-v1.md`
- Replay/resource semantics: `docs/adr/0055-frame-recording-and-subtree-replay-caching.md`
- Current alignment: `docs/adr/IMPLEMENTATION_ALIGNMENT.md`
- Public app-author guide: `docs/code-editor.md`
- Public surface classification:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/PUBLIC_SURFACE_CLASSIFICATION_2026-05-12.md`
- Selection ownership move:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M1_SELECTION_OWNERSHIP_CONTRACT_2026-05-12.md`
- Command/keymap/undo boundary:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M1_COMMAND_KEYMAP_UNDO_BOUNDARY_2026-05-12.md`
- Diagnostic span contract:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_DIAGNOSTIC_SPAN_CONTRACT_2026-05-12.md`
- Diagnostic line summary contract:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_DIAGNOSTIC_LINE_SUMMARY_CONTRACT_2026-05-12.md`
- Gutter marker contract:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_GUTTER_MARKER_CONTRACT_2026-05-12.md`
- Range decoration contract:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_RANGE_DECORATION_CONTRACT_2026-05-12.md`
- Semantic token contract:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_SEMANTIC_TOKEN_CONTRACT_2026-05-12.md`
- Overlay feature boundary:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_OVERLAY_FEATURE_BOUNDARY_2026-05-12.md`
- Coordinate vocabulary:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_COORDINATE_VOCABULARY_2026-05-12.md`
- Widget-facing feature payload surface:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M2_WIDGET_FEATURE_PAYLOAD_SURFACE_2026-05-12.md`
- UI Gallery feature payload fixture:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M4_UI_GALLERY_FEATURE_PAYLOAD_FIXTURE_2026-05-12.md`
- Feature payload bundle assertion:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M4_FEATURE_PAYLOAD_BUNDLE_ASSERTION_2026-05-12.md`
- Overlay feature hook proof:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M4_OVERLAY_FEATURE_HOOK_PROOF_2026-05-12.md`
- Performance contract closure:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M5_PERF_CONTRACT_CLOSURE_2026-05-12.md`
- First editor test module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_TEST_MODULE_FIRST_SPLIT_2026-05-12.md`
- Syntax test module split:
  `ecosystem/fret-code-editor/src/editor/tests/syntax.rs`
- Feature payload store module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_FEATURE_PAYLOAD_STORE_MODULE_SPLIT_2026-05-12.md`
- Diagnostics snapshot module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_DIAGNOSTICS_SNAPSHOT_MODULE_SPLIT_2026-05-12.md`
- State schema module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_STATE_SCHEMA_MODULE_SPLIT_2026-05-12.md`
- State methods module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_STATE_METHODS_MODULE_SPLIT_2026-05-12.md`
- State initializer boundary:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_STATE_INITIALIZER_BOUNDARY_2026-05-12.md`
- Handle module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_HANDLE_MODULE_SPLIT_2026-05-12.md`
- Handle method boundary split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_HANDLE_METHOD_BOUNDARY_SPLIT_2026-05-12.md`
- A11y window/mapping module split:
  `docs/workstreams/code-editor-public-api-and-architecture-v1/M3_A11Y_MODULE_SPLIT_2026-05-12.md`
- Buffer source: `ecosystem/fret-code-editor-buffer/src/lib.rs`
- View source: `ecosystem/fret-code-editor-view/src/lib.rs`
- Diagnostic projection source: `ecosystem/fret-code-editor-view/src/diagnostics.rs`
- Gutter marker source: `ecosystem/fret-code-editor-view/src/gutter.rs`
- Range decoration source: `ecosystem/fret-code-editor-view/src/decorations.rs`
- Semantic token source: `ecosystem/fret-code-editor-view/src/semantic_tokens.rs`
- Surface root: `ecosystem/fret-code-editor/src/lib.rs`
- Surface integration: `ecosystem/fret-code-editor/src/editor/mod.rs`
- Feature payload store: `ecosystem/fret-code-editor/src/editor/feature_payloads.rs`
- State schema: `ecosystem/fret-code-editor/src/editor/state.rs`
- Paint hot path: `ecosystem/fret-code-editor/src/editor/paint/mod.rs`
- Input hot path: `ecosystem/fret-code-editor/src/editor/input/mod.rs`
- A11y projection boundary: `ecosystem/fret-code-editor/src/editor/a11y/mod.rs`
- A11y text-window owner: `ecosystem/fret-code-editor/src/editor/a11y/window.rs`
- A11y offset-mapping owner: `ecosystem/fret-code-editor/src/editor/a11y/mapping.rs`
- Overlay infrastructure: `ecosystem/fret-ui-kit/src/overlay_controller.rs`
- Existing editor anchored overlay recipe: `ecosystem/fret-ui-editor/src/controls/text_assist_field.rs`
- UI Gallery overlay proof:
  `apps/fret-ui-gallery/src/ui/previews/pages/editors/code_editor/torture.rs`
- UI Gallery overlay render-flow gate: `apps/fret-ui-gallery/src/driver/render_flow.rs`

## Known Caveats

- Linux performance is not currently validated by this lane. Keep Windows and wasm evidence labeled
  by environment.
- Existing code editor tests are large and concentrated. Passing the current crate tests is useful
  but does not replace a public API surface-diff note.
- `code-editor-ecosystem-v1` remains the broad historical rollout lane; this follow-on owns public
  API and extension architecture closure.
