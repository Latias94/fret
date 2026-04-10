# M2 Panel-Resize Gate Promotion — 2026-04-10

Status: accepted execution note
Last updated: 2026-04-10

This note records the promotion of the fixed-window panel-resize proof into the active adaptive
lane gate set.

## Goal

Prove that this lane now owns one reviewable container-first resize gate in addition to the
narrow-window UI Gallery proof.

The target behavior is:

- the window size stays fixed,
- the dock panel width changes via the split handle,
- and the adaptive story is still evaluated through container width rather than viewport width.

## Commands used

```bash
cargo check -p fret-demo --bin container_queries_docking_demo --message-format short
cargo build -p fret-demo --bin container_queries_docking_demo --release
cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/adaptive-panel-resize-promote --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo
```

## Result

Promotion succeeded.

Successful run:

- session dir:
  `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694`
- packed share artifact:
  `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694/share/1775822919993.zip`

Produced evidence includes:

- before/after layout sidecars
  - `1775822920135-container-queries-docking-before.layout/layout.taffy.v1.json`
  - `1775822920293-container-queries-docking-after.layout/layout.taffy.v1.json`
- before/after screenshots
  - `screenshots/1775822920149-container-queries-docking-before/window-4294967297-tick-17-frame-17.png`
  - `screenshots/1775822920302-container-queries-docking-after/window-4294967297-tick-34-frame-34.png`
- bounded bundles
  - `1775822920138-container-queries-docking-before/`
  - `1775822920297-container-queries-docking-after/`
  - `1775822920336-container-queries-docking-panel-resize/`

The top-level run verdict is recorded in:

- `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694/script.result.json`

## Script migration note

The promotion surfaced one CLI compatibility detail that should stay documented:

- the real script payload for `--launch` must be `schema_version = 2`,
- but the root alias file that uses `kind = "script_redirect"` still needs
  `schema_version = 1` on the current CLI.

Applied fix:

- keep `tools/diag-scripts/container-queries-docking-panel-resize.json` as the stable redirect,
- upgrade the target script at
  `tools/diag-scripts/docking/container-queries/container-queries-docking-panel-resize.json` to v2,
- and add layout-sidecar capture alongside screenshots and bundles so container-width ownership is
  reviewable.

## Consequence for this lane

`ALC-032` is now considered complete.

From this point forward, the adaptive lane owns two distinct proof classes:

1. narrow-window docs/gallery proof,
2. fixed-window panel-resize proof.

The remaining M2 gap is no longer panel-resize promotion.
It is the explicit Gallery teaching surface that compares container-driven and viewport-driven
behavior without mixing them.
