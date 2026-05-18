# UI Gallery Code Editor Canvas Paint Tail Attribution v1 - Evidence And Gates

Status: Active
Last updated: 2026-05-18

## Starting Evidence

This lane starts from the VCRJ-030 fresh bundle:

- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json`
- `target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/layout.perf.summary.v1.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/view-cache-resize-jitter-attribution-v1-vcrj030/1779091052963/bundle.schema2.json \
  --sort time --top 20
```

Observed local result:

- `time p50/p95 (us): total=693/362814, layout=34/1070, prepaint=14/1353, paint=643/360395`
- Worst frame `tick=386 frame=442`: `total=362814`, `layout=1070`, `prepaint=1349`,
  `paint=360395`
- `paint_widget.hotspots canvas.top_exclusive_us(p50/p95/max)=354464/360009/360009`
- Top paint hotspot:

```text
Canvas paint_time_us=360009 inclusive_us=360009 scene_ops_delta=20009
```

The same stats output reports zero `code_editor.paint_perf` counters. That mismatch is the first
source-audit target.

## CPT-020 Source Attribution

Evidence:

- `CPT_020_SOURCE_ATTRIBUTION_2026-05-18.md`

Verdict:

- The `Canvas` hotspot is owned by the code-editor windowed rows surface callback:
  `TextInputRegion` -> `windowed_rows_surface_with_pointer_region` -> `PointerRegion` -> `Canvas`
  -> `paint_windowed_rows` -> `paint::paint_row`.
- `code_editor.paint_perf` is missing from the bundle because
  `FRET_CODE_EDITOR_DIAG_PAINT_PERF` was not set. UI Gallery serialized
  `paint_perf: null` for every captured snapshot.
- The all-zero `code_editor.paint_perf frames=10` stats output is a reporting artifact: the stats
  code treats a present `null` `paint_perf` field as a default all-zero sample.
- Cumulative cache stats show heavy row text/scene/syntax churn, but the current bundle cannot prove
  that churn owns the 360ms frame because per-frame paint perf was disabled.

CPT-030 decision:

- Rerun the same script with `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`.
- Treat runtime optimization as blocked until the new bundle distinguishes
  `us_windowed_surface_paint_callback`, `us_windowed_surface_row_paint`,
  `us_windowed_surface_non_row`, and row scene/text counters.

## CPT-030 Repro Result

Evidence:

- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json`
- `CPT_030_CPT_040_OWNER_PROOF_2026-05-18.md`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030/1779092655064/bundle.schema2.json \
  --sort time --top 20
```

Observed local result:

- `time p50/p95 (us): total=722/380102, layout=35/1140, prepaint=16/2575, paint=669/376387`
- Worst frame `total=380102us`, `layout=1140us`, `prepaint=2575us`, `paint=376387us`
- `code_editor.paint_perf` worst visible range: `visible(start/end/rows)=0/20003/20004`
- `code_editor.paint_perf.surface` worst row paint: `row_paint=359962us`

Viewport evidence:

```json
{"test_id":"ui-gallery-code-editor-torture-viewport","viewport_h":320064.0,"content_h":320064.0}
```

Verdict:

- The `Canvas` tail repeated with paint perf enabled.
- The row paint was real work, but the root owner was a wrong inner scroll viewport equal to the
  full content height.

## CPT-040 Owner Proof Result

Runtime anchors:

- `crates/fret-ui/src/declarative/host_widget/layout/positioned_container.rs`
- `crates/fret-ui/src/declarative/tests/layout/scroll.rs`

Focused test:

```bash
cargo nextest run -p fret-ui scroll_viewport_for_tall_canvas_child
```

Fresh local result:

```text
2 tests run: 2 passed, 1019 skipped
```

After bundle:

- `target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json`

Stats command:

```bash
target/release/fretboard-dev diag stats \
  target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt040/1779099328829/bundle.schema2.json \
  --sort time --top 20
```

Observed local result:

- `time p50/p95 (us): total=698/1425, layout=39/761, prepaint=124/286, paint=393/418`
- Worst frame `total=1425us`, `layout=753us`, `prepaint=274us`, `paint=398us`
- `paint_widget.hotspots canvas.top_exclusive_us(p50/p95/max)=125/134/134`
- `code_editor.paint_perf` max rows: `painted=289`, `replayed=289`

Viewport evidence:

```json
{"test_id":"ui-gallery-code-editor-torture-viewport","viewport_h":518.0,"content_h":320064.0}
```

Verdict:

- The `fret-ui` positioned-container final child sizing fix bounded the inner windowed scroll
  viewport.
- The same local repro no longer shows the 360ms `Canvas` paint tail.
- No renderer, `ViewCache`, or code-editor row-surface follow-on is split from this lane.

## CPT-050 Closeout

Evidence:

- `CLOSEOUT_AUDIT_2026-05-18.md`

Closeout verdict:

- Closed with a runtime mechanism fix in `fret-ui`.
- Future performance work should start from a new fresh bundle and a new owner boundary.

Fresh verification:

```bash
cargo nextest run -p fret-ui scroll_viewport_for_tall_canvas_child
```

Result: passed, `2 tests run: 2 passed, 1019 skipped`.

```bash
python3 -m json.tool docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
git diff --check
cargo fmt --check
python3 tools/check_layering.py
```

Results:

- `WORKSTREAM.json`: valid JSON.
- `check_workstream_catalog.py`: passed, `416 dedicated directories, 47 standalone markdown files`.
- `git diff --check`: passed.
- `cargo fmt --check`: passed.
- `check_layering.py`: passed.

## Repro Template

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --prewarm-script tools/diag-scripts/_prelude/tooling-suite-prewarm-fonts.json \
  --prelude-script tools/diag-scripts/_prelude/tooling-suite-prelude-reset-diagnostics.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_LAYOUT_NODE_PROFILE=1 \
  --env FRET_LAYOUT_NODE_PROFILE_TOP=20 \
  --env FRET_LAYOUT_NODE_PROFILE_MIN_US=1 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/ui-gallery-code-editor-canvas-paint-tail-attribution-v1-cpt030 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

## Canonical Gates

Workstream state:

```bash
python3 -m json.tool docs/workstreams/ui-gallery-code-editor-canvas-paint-tail-attribution-v1/WORKSTREAM.json
python3 tools/check_workstream_catalog.py
git diff --check
```

Focused source audit:

```bash
rg -n "windowed_rows_surface|Canvas|paint_perf|row_scene|surface_callback|torture" \
  ecosystem/fret-code-editor \
  ecosystem/fret-ui-kit/src/declarative \
  crates/fret-ui \
  crates/fret-diag \
  -S
```

Perf attribution:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20
```

Runtime gates for later slices:

```bash
python3 tools/check_layering.py
cargo fmt --check
```

Add focused `cargo nextest` or diag gates after CPT-020 identifies the owner.

## Evidence Rules

- Do not make renderer/canvas API changes before the source audit proves the owner.
- Do not treat the VCRJ-030 360ms tail as a cross-machine baseline; it is local attribution
  evidence.
- Do not reopen `ViewCache` from this lane unless a fresh bundle again proves `ViewCache` is the top
  owner.
