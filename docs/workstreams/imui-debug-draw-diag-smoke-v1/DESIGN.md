# ImUi Debug Draw Diag Smoke v1

Status: Closed narrow diagnostics follow-on
Last updated: 2026-05-05

`imui_debug_draw_basics` made the debug-draw API publicly teachable, but the page still had no
promoted diagnostics script. This lane adds the smallest useful evidence artifact: launch the
cookbook example, wait for stable selectors, capture a screenshot, and capture a bundle.

## Ownership

- `tools/diag-scripts/cookbook/imui-debug-draw-basics/` owns the canonical smoke script.
- `tools/diag-scripts/suites/cookbook-imui-debug-draw-basics/` owns suite membership.
- `tools/diag-scripts/index.json` is generated registry state.
- `apps/fret-cookbook/EXAMPLES.md` owns user-facing suite discoverability.

## Must-Be-True Outcomes

- The script uses schema v2 and stable `test_id` selectors.
- The suite is discoverable through `fretboard-dev diag suite cookbook-imui-debug-draw-basics`.
- The cookbook index lists the suite beside `imui_debug_draw_basics`.
- The script captures both screenshot and bundle evidence without adding fragile pixel assertions.

## Non-Goals

- No pixel-perfect comparison gate.
- No layout sidecar or renderer draw-call attribution.
- No interaction scenario beyond first-open visual evidence.
