# Switch Diagnostics Packet v1

## Truth

- Switch is already covered by the Material3 adapter report as an interaction-heavy seed component.
- Fresh gallery diagnostics confirm the current Switch snippets still expose stable root/chrome,
  track, handle, and icon selectors across default, disabled, icons-both, and selected-icon-only
  states.
- Switch remains split correctly: recipe owns chrome/animation, Material foundation owns
  indication/ripple and minimum target sizing.

## Artifacts

- `ecosystem/fret-ui-material3/src/switch.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/switch.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icons-state-matrix-screenshots.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The diagnostics script opens the Material3 Switch page and captures idle, hover, pressed,
selected, focus-visible, and disabled states for both-icons and selected-icon-only variants.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icons-state-matrix-screenshots.json --dir target/fret-diag/material3-switch-icons-state-matrix-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Result: `PASS`, run id `1779937207775`.

Bounded evidence:

- `target/fret-diag/material3-switch-icons-state-matrix-20260528/sessions/1779936902105-61844/1779937207775/ai.packet`
- `target/fret-diag/material3-switch-icons-state-matrix-20260528/sessions/1779936902105-61844/share/1779937207775.zip`

The adapter report remains valid: 5 parts, 5 `pass_known`, zero mismatches, and zero top findings.

## Residual Risk

Pixel-level comparison against upstream Material Web motion remains future parity hardening if a
product-visible mismatch appears. It is not a current Fret mechanism gap.
