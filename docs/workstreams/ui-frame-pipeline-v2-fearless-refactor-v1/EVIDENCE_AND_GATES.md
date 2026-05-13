# Evidence and Gates

Status: Active
Last updated: 2026-05-13

## Primary Repro

The first repro remains the code-editor resize/paint pressure path:

```bash
cargo run -p fretboard-dev --release -- diag perf ui-code-editor-resize-probes \
  --repeat 3 \
  --warmup-frames 5 \
  --reuse-launch \
  --sort time \
  --top 15 \
  --json \
  --perf-baseline docs/workstreams/perf-baselines/ui-code-editor-resize-probes.macos-m4.v2.json \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --env FRET_CODE_EDITOR_DIAG_PAINT_PERF=1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Use an explicit `--dir target/<descriptive-dir>` for publishable evidence.

## Required Attribution

For every perf claim, run:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 15
```

The summary must mention:

- total/layout/prepaint/paint p50 and p95,
- `layout.engine_solve`,
- `paint.widget`,
- `paint.text_prepare`,
- renderer prepare/encode/upload counters,
- and `code_editor.paint_perf` when the code-editor surface is involved.

## Current Baseline Evidence

Most recent pre-lane evidence:

- M0 baseline/source audit:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M0_BASELINE_AUDIT_2026-05-13.md`
- M1 boundary diagnostics slice:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`
- Workstream log:
  `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1-log.md`
- macOS contained-layout run:
  `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/check.perf_thresholds.json`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-contained-layout-20260513/1778661520873/bundle.schema2.json`

Observed result from that run:

- gate failures: `[]`,
- p95/max top total: `1361/1361us`,
- p95/max top layout: `295/295us`,
- p95/max top layout solve: `116/116us`,
- p95/max paint: `1134/1134us`,
- `code_editor.paint_perf` p50/p95 total: `241/401us`.

Most recent boundary-diagnostics slice evidence:

- Slice note:
  `docs/workstreams/ui-frame-pipeline-v2-fearless-refactor-v1/M1_BOUNDARY_DIAGNOSTICS_SLICE_2026-05-13.md`
- Perf output directory:
  `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513`
- Worst bundle:
  `target/fret-diag-code-editor-resize-probes-boundary-diag-20260513/1778668519515/bundle.schema2.json`

Observed result from that run:

- time p50/p95: total `1203/1811us`, layout `38/364us`, prepaint `15/34us`,
  paint `949/1737us`,
- hot p50/p95: `layout.engine_solve=0/140us`, `paint.widget=731/1494us`,
  `paint.text_prepare=10/15us`,
- `code_editor.paint_perf` p50/p95 total: `302/743us`,
- renderer prepare/encode/upload counters stayed at zero.

## Correctness Gates

Use focused tests first:

```bash
cargo nextest run -p fret-ui <filter>
cargo test -p fret-ui-gallery --features gallery-full --lib <filter>
cargo test -p fret-ui-shadcn --lib <filter>
```

Required for boundary/invalidation changes:

```bash
python3 tools/check_layering.py
```

## Future Paint Stressor

If `ui-code-editor-resize-probes` stops catching the active paint bottleneck, add a narrower
code-editor paint stressor before continuing:

- route directly to the code-editor torture surface,
- keep `FRET_CODE_EDITOR_DIAG_PAINT_PERF=1`,
- stress row replay/content resolution without unrelated gallery setup noise,
- and seed a baseline/policy only after the script is deterministic.

## Closeout Evidence

Closeout requires:

- final perf run paths,
- final worst-bundle attribution,
- deletion audit path,
- ADR alignment row,
- and exact commands for all promoted gates.
