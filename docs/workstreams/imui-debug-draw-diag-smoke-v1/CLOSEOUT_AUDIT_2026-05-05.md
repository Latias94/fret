# ImUi Debug Draw Diag Smoke v1 Closeout Audit - 2026-05-05

Status: closed closeout record.

## What Shipped

- Added a schema v2 diagnostics smoke script for `imui_debug_draw_basics`.
- Added the `cookbook-imui-debug-draw-basics` suite manifest.
- Refreshed `tools/diag-scripts/index.json`; the generator also normalized two existing IMUI table
  gate entries that were suite-reachable but missing from the registry.
- Updated the cookbook index to expose the new suite.
- Follow-up response metadata coverage is now tracked by
  `docs/workstreams/imui-debug-draw-response-surface-v1/`.

## Evidence

- `tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json`
- `tools/diag-scripts/suites/cookbook-imui-debug-draw-basics/suite.json`
- `tools/diag-scripts/index.json`
- `apps/fret-cookbook/EXAMPLES.md`
- `apps/fret-cookbook/examples/imui_debug_draw_basics.rs`
- Local launched run artifact (not checked in):
  `target/fret-diag/1777994933876-cookbook-imui-debug-draw-basics-smoke`

## Gates Run

```bash
cargo run -p fretboard-dev -- diag script validate tools/diag-scripts/cookbook/imui-debug-draw-basics/cookbook-imui-debug-draw-basics-smoke.json --json
python tools/check_diag_scripts_registry.py
FRET_DIAG=1 cargo run -p fretboard-dev -- diag suite cookbook-imui-debug-draw-basics --launch -- cargo run -p fret-cookbook --features cookbook-imui,cookbook-diag --example imui_debug_draw_basics
git diff --check
```

## Residual Gaps

- The smoke is first-open evidence only.
- It intentionally avoids pixel-perfect assertions until debug-draw visuals have a stable visual
  contract.
- Layout sidecars, renderer attribution, and per-geometry hit testing remain separate follow-ons.
