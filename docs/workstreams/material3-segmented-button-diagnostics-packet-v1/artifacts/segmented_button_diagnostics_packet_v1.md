# Segmented Button Diagnostics Packet v1

## Truth

- Material3 `SegmentedButtonSet` exposes stable group and item selectors.
- The single-select variant presents radio-group / radio-button semantics with a checked active
  segment.
- The multi-select variant presents checkbox semantics and keeps selection state mirrored in the
  items.
- Roving focus across segments works in the gallery and the expressive variant remains covered by
  the same diagnostics surface.
- This remains a Material recipe and diagnostics concern; no new shared kit abstraction is proven.

## Artifacts

- `ecosystem/fret-ui-material3/src/segmented_button.rs`
- `ecosystem/fret-ui-material3/src/tokens/segmented_button.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/segmented_button.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json`
- `tools/diag-scripts/suites/ui-gallery-material3-segmented-button-roving-semantics-screenshots/suite.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The roving-semantics script opens the Material3 segmented-button page, waits for the single
segmented group and its item ids, captures idle and selected states, drives ArrowRight / Home / End
roving, checks the multi-select group role, flips expressive mode, and captures a final bundle.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json --dir target/fret-diag/material3-segmented-button-roving-semantics-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo nextest run -p fret-ui-material3 --test radio_alignment segmented_button_semantics_roles_match_compose_baseline
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_segmented_button_suite_goldens_v1
```

Results:

- roving-semantics diagnostics: `PASS`, run id `1779946252252`
- segmented-button semantics role gate: `PASS`
- segmented-button headless goldens gate: `PASS`

Bounded evidence:

- `target/fret-diag/material3-segmented-button-roving-semantics-20260528/sessions/1779945893709-62064/1779946252252/ai.packet`
- `target/fret-diag/material3-segmented-button-roving-semantics-20260528/sessions/1779945893709-62064/share/1779946252252.zip`

## Residual Risk

No current SegmentedButtonSet recipe, foundation, kit-policy, or mechanism residual remains from
this packet. Broader chip/checkbox/radio follow-ons remain owned by their own rows.
