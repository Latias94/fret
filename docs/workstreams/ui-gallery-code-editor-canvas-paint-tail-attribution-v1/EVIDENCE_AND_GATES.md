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
