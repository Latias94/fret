# fret-ui Layout Architecture Audit v1 - Evidence And Gates

Status: Closed

## Baseline gates

Use these while auditing only:

```bash
cargo nextest run -p fret-ui layout_engine --no-fail-fast
cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast
python3 tools/check_layering.py
cargo fmt --check
git diff --check
```

## Perf attribution command

Use this command shape for a local orientation sample. Treat results as local evidence, not a
cross-machine formal baseline:

```bash
target/release/fretboard-dev diag perf \
  tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json \
  --repeat 1 \
  --warmup-frames 5 \
  --reuse-launch \
  --env FRET_UI_GALLERY_VIEW_CACHE=1 \
  --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 \
  --env FRET_DIAG_RENDERER_PERF=1 \
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/layout-architecture-audit-v1-baseline \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

If a bundle is captured, inspect it with:

```bash
target/release/fretboard-dev diag stats <bundle.json> --sort time --top 20
```

## Initial evidence read

- `scroll-optimization-v1` says the local clean-geometry resize-jitter phase is closed and future
  work should split into narrower follow-ons.
- Remaining classified blockers from that handoff are wrapped text, small `Canvas`, root `Scroll`,
  other-machine evidence, and optional measured-size data-model work.
- The current clean-geometry classification has already been split into the internal axes
  `layout_effect`, `child_bounds`, and `size_stability`.

## Local FLA-020 attribution sample

Command shape:

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
  --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 \
  --env FRET_DIAG_SEMANTICS=0 \
  --dir target/fret-diag/layout-architecture-audit-v1-baseline-r1 \
  --launch -- cargo run -p fret-ui-gallery --release --features gallery-full
```

Artifacts:

- Bundle:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/1779077560550/bundle.schema2.json`
- Stats:
  `target/fret-diag/layout-architecture-audit-v1-baseline-r1/worst.stats.json`

Result:

- Worst frame: `total=2803us`, `layout=2304us`, `layout_roots=2181us`,
  `layout_engine_solve=202us`, `prepaint=202us`, `paint=297us`.
- Renderer text remains bounded: `renderer_prepare_text_us=65us`; widget text prepare is `0us`.
- ViewCache reuse stays stable: `cache_roots_reused=1/1`.
- Top solves are small `new_frame_key_changed` roots with `measure_time_us=0`: approximately
  `155us`, `43us`, and `3us`.
- Top layout hotspots point to retained-tree/barrier orchestration rather than Taffy solve cost:
  `Semantics` inclusive around `2177us`, `Scroll` around `281us`, and `ViewCache` around `373us`.

Decision from this sample:

- Do not redesign the layout/node classification model now.
- If cleanup continues, prefer a behavior-preserving clean-geometry module extraction.
- If performance work continues, the likely owner is retained layout orchestration around
  `Semantics` / `Scroll` / `ViewCache`, not text or Taffy itself.

## FLA-040 behavior-preserving extraction

Changed code:

- `crates/fret-ui/src/tree/layout/clean_geometry.rs` now owns the clean-geometry proof model,
  skip/rejection attribution, manual child bounds derivation, and clean engine geometry propagation.
- `crates/fret-ui/src/tree/layout/node.rs` now keeps the ordinary per-node layout and measure
  execution path.
- `crates/fret-ui/src/tree/layout/mod.rs` registers the private `clean_geometry` module.

Validation:

- `cargo nextest run -p fret-ui layout_engine --no-fail-fast` - passed, 50 tests.
- `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast` -
  passed, 1 test.
- `python3 tools/check_layering.py` - passed.
- `cargo fmt --check` - passed.
- `python3 -m json.tool docs/workstreams/fret-ui-layout-architecture-audit-v1/WORKSTREAM.json` -
  passed.
- `python3 tools/check_workstream_catalog.py` - passed.
- `git diff --check` - passed.

Decision:

- This was an organization-only refactor. It does not change runtime layout behavior or public
  contracts, so no ADR alignment update is needed.
- FLA-050 closes this audit lane and splits the next runtime performance owner to
  `docs/workstreams/retained-layout-orchestration-v1/`.

## FLA-050 closeout decision

Decision:

- Do not redesign the layout/node classification model from this lane.
- Do not reopen clean-geometry expansion as the default next step.
- Preserve the current stop conditions for text computed-box stability, root `Scroll`,
  tiny `Canvas`, and measured-size data modeling unless fresh evidence makes one of them dominant.
- Open the next performance lane as retained layout orchestration/root `Scroll` side-effect
  boundary work: `docs/workstreams/retained-layout-orchestration-v1/`.

Why:

- The FLA-020 sample showed a layout-heavy worst frame, but Taffy solve time was small
  (`202us`) and renderer/text costs were bounded.
- The remaining top owner shape was retained-tree/barrier orchestration:
  `Semantics` inclusive around `2177us`, `Scroll` around `281us`, and `ViewCache` around `373us`.
- FLA-040 reduced the reviewability risk by moving the proof model out of ordinary
  `layout_node` execution, so the remaining work is not an architecture audit problem.

## Evidence to add

- [x] FLA-010 source inventory note:
  `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_INVENTORY_2026-05-18.md`.
- [x] FLA-020 baseline bundle/stats path.
- [x] FLA-030 decision note:
  `docs/workstreams/fret-ui-layout-architecture-audit-v1/ARCHITECTURE_DECISION_2026-05-18.md`.
- [x] FLA-040 behavior-preserving clean-geometry module extraction:
  `crates/fret-ui/src/tree/layout/clean_geometry.rs`.
- [x] FLA-050 closeout and follow-on split:
  `docs/workstreams/retained-layout-orchestration-v1/`.
