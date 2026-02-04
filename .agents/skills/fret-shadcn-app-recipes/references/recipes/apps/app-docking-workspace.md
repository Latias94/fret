# App recipe: Docking workspace (editor shell)

Goal: an editor-like workspace shell (tabs, splits, viewports, tear-off policies) with stable input/drag arbitration and reproducible diagnostics.

## References

- Docking demos:
  - `apps/fret-examples/src/docking_demo.rs`
  - `apps/fret-examples/src/docking_arbitration_demo.rs`
- Diag scripts:
  - `tools/diag-scripts/docking-demo-drag-indicators.json`
  - `tools/diag-scripts/docking-arbitration-demo-split-viewports.json`
  - `tools/diag-scripts/docking-arbitration-demo-modal-dock-drag-viewport-capture.json`

## Mind models to apply

- Overlay/focus rules: `../../mind-models/mm-overlays-and-focus.md`
- Automation and gating: `../../mind-models/mm-a11y-and-testid.md`, `../../mind-models/mm-diagnostics-and-regression-gates.md`

## Checklist

- Drag arbitration is deterministic (dock drag vs viewport input capture).
- Split viewports keep correct focus + input routing.
- Modal overlays (dialogs/menus) don’t break docking drag/capture invariants.
- Provide stable `test_id` anchors for:
  - dock space root
  - tab strip entries
  - drag handles / drop indicators

## Regression gates (recommended)

- Always keep at least one scripted dock-drag repro for edge cases (split viewports + modal overlay).
- Capture screenshots only when visuals are part of the contract (drop indicators, placement cues).

## See also

- `fret-diag-workflow` (scripted repro + packaging)
