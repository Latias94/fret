# Editor Canvas Paint Replay Slice v1

## Problem

The Windows RTX4090 editor-paint formal closeout for `20260523-r58` passed and selected
`owner=canvas-paint-replay`, `action=open-canvas-paint-replay-slice`. The owner reason is that
`paint.widget` / `Canvas` remains the dominant verified attribution owner across the three editor probes:

| probe | paint_widget_p95_us | canvas_exclusive_p95_us | renderer_prepare_text_p95_us | renderer_encode_scene_p95_us | renderer_upload_p95_us | code_editor_total_p95_us |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| resize-jitter | 912 | 494 | 153 | 778 | 155 | 417 |
| typical-autoscroll | 697 | 458 | 140 | 560 | 218 | 377 |
| complex-wheel | 631 | 419 | 168 | 486 | 104 | 327 |

This lane owns the next narrow implementation slice after that decision.

## Target State

- Canvas / paint-widget replay work is reduced with a bounded, reversible change.
- The change is proven by the same editor-paint three-probe validation/attribution shape that selected the owner.
- Row replay/cache correctness remains intact.
- Renderer text/encode/upload thresholds are not weakened to hide Canvas work.
- Existing closed Canvas attribution workstreams remain historical references unless a fresh bundle proves the same scope.

## Scope

- Code-editor Canvas paint and row-surface paint callback boundaries.
- Canvas-hosted resource touch/replay costs when they appear inside the selected owner.
- Paint-cache / paint-widget attribution fields needed to prove the owner boundary.
- `fret-diag` summary fields only when they are required to separate implementation choices.

## Non-Goals

- Broad row display-list rewrites without a fresh owner proof.
- Renderer text/glyph residency work unless closeout selects renderer text as owner.
- Checked-in baseline updates before target-machine post-change evidence justifies them.
- Reopening `ui-gallery-code-editor-canvas-paint-tail-attribution-v1` by default.

## Architecture Direction

Start from the exact owner boundary in the closeout:

1. Reconcile `paint_widget_hotspot_summary` with code-editor paint perf counters.
2. Identify whether the cost is Canvas-hosted replay/touch, row-surface callback assembly, paint-cache bookkeeping, or
   generic paint traversal.
3. Land one reversible change against the proven owner.
4. Rerun the three editor probes and closeout before any baseline decision.

Any mechanism-level Canvas cache change must reference ADR 0161. Any text residency change must reference ADR 0143.
Frame/view-boundary changes must reference ADR 0327.

## Closeout

This lane is closed after the `20260523-r59` baseline validation, `20260523-r59-attrib` attribution validation, and
the `20260523-r59` artifact verifier / closeout all passed. The baseline policy was left unchanged.
Any future Canvas replay work should open a fresh lane only if a new bounded bundle proves a different owner.
