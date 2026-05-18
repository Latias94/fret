# fret-ui Layout Architecture Audit v1 - TODO

Status: Closed

## Task ledger

- [x] FLA-010 Build the baseline architecture inventory.
  - Scope: `tree/layout/node.rs`, `tree/layout/solve.rs`, `tree/layout/entrypoints.rs`,
    `layout/engine.rs`.
  - Output: `ARCHITECTURE_INVENTORY_2026-05-18.md`.
  - Validation: source anchors cite the current classification axes, execution entry points, and
    diagnostics fields.
  - Result: current conceptual axes are sound; the main risk is reviewability from keeping the
    proof model inside `tree/layout/node.rs`.

- [x] FLA-020 Run a small local perf and attribution baseline.
  - Scope: one resize-jitter script and one focused Rust test gate.
  - Suggested command:
    `target/release/fretboard-dev diag perf tools/diag-scripts/ui-gallery/code-editor/ui-gallery-code-editor-window-resize-drag-jitter-steady.json --repeat 1 --warmup-frames 5 --reuse-launch --env FRET_UI_GALLERY_VIEW_CACHE=1 --env FRET_UI_GALLERY_VIEW_CACHE_SHELL=1 --env FRET_DIAG_RENDERER_PERF=1 --env FRET_DIAG_SCRIPT_AUTO_DUMP=0 --env FRET_DIAG_SEMANTICS=0 --dir target/fret-diag/layout-architecture-audit-v1-baseline --launch -- cargo run -p fret-ui-gallery --release --features gallery-full`
  - Output: a short note in `EVIDENCE_AND_GATES.md` with bundle path and top solve/rejection
    summary.
  - Result: local sample captured at
    `target/fret-diag/layout-architecture-audit-v1-baseline-r1/1779077560550/bundle.schema2.json`;
    stats at `target/fret-diag/layout-architecture-audit-v1-baseline-r1/worst.stats.json`.
    Worst frame was layout-heavy (`2803us` total, `2304us` layout) but Taffy solve was small
    (`202us`), so it does not justify a model rewrite.

- [x] FLA-030 Decide whether clean-geometry needs extraction.
  - Scope: behavior-preserving organization only.
  - Decision options: no extraction, split to `tree/layout/clean_geometry.rs`, or introduce a
    stronger internal proof model first.
  - Validation: decision references FLA-010 and FLA-020.
  - Result: do not redesign the model. Prefer behavior-preserving extraction only if we continue
    adding clean-geometry proof cases.

- [x] FLA-040 If extraction is justified, land the smallest behavior-preserving module split.
  - Scope: no runtime behavior change; move proof model/helpers only.
  - Validation:
    `cargo nextest run -p fret-ui layout_engine --no-fail-fast`,
    `cargo nextest run -p fret-ui clean_geometry_small_resize_rejects_auto_height_text_reflow --no-fail-fast`,
    `python3 tools/check_layering.py`,
    `cargo fmt --check`.
  - Dependency: FLA-030.
  - Result: clean-geometry proof types, skip decision logic, manual child-bounds derivation, and
    clean engine geometry propagation now live in
    `crates/fret-ui/src/tree/layout/clean_geometry.rs`. `tree/layout/node.rs` keeps ordinary
    per-node layout/measure execution and calls the private propagation entry point unchanged.

- [x] FLA-050 Decide next performance owner.
  - Options: text computed-box stability, root `Scroll` side-effect-boundary redesign, tiny `Canvas`
    proof, measured-size data model, or no further local perf work.
  - Validation: must cite fresh perf evidence or explicitly preserve the existing stop condition.
  - Result: close this audit lane. The next performance owner is a separate retained layout
    orchestration/root `Scroll` side-effect-boundary lane at
    `docs/workstreams/retained-layout-orchestration-v1/`. Do not reopen clean-geometry expansion or
    redesign the layout model without fresh evidence.

## Closeout decision

This audit lane is complete. It answered the architecture question, landed the smallest
behavior-preserving organization refactor, and split the next runtime performance owner into a
separate workstream.
