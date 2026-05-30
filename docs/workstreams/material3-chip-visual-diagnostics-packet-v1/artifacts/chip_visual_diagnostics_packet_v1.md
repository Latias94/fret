# Chip Visual Diagnostics Packet v1

## Truth

- Chip visual follow-ons were conditional on gallery diagnostics showing spacing/elevation drift.
- A new gallery diagnostics script now proves representative root/chrome geometry without
  requiring component changes.
- Exact pixel/material elevation extraction remains out of scope until a consumer needs that
  mechanism; current root/chrome bounds and captured bundles are sufficient for this packet.

## Artifacts

- `ecosystem/fret-ui-material3/src/chip.rs`
- `ecosystem/fret-ui-material3/src/suggestion_chip.rs`
- `ecosystem/fret-ui-material3/src/filter_chip.rs`
- `ecosystem/fret-ui-material3/src/input_chip.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/state_matrix.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json`
- `tools/diag-scripts/ui-gallery-material3-chip-visual-chrome.json`
- `tools/diag-scripts/suites/ui-gallery-material3-chip-visual-chrome/suite.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The diagnostics script starts on `material3_state_matrix`, scrolls representative chip rows into
view, then asserts:

- AssistChip flat and elevated root/chrome centers align.
- SuggestionChip flat and elevated root/chrome centers align.
- FilterChip selected and override root/chrome centers align.
- InputChip selected and unselected root/chrome centers align.
- FilterChip and InputChip trailing-icon selectors exist.
- A representative chip keeps the minimum 48px interactive target.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json --dir target/fret-diag/material3-chip-visual-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Result: `PASS`, run id `1779936147211`.

Bounded evidence:

- `target/fret-diag/material3-chip-visual-chrome-20260528/sessions/1779935853792-64132/1779936147211/ai.packet`
- `target/fret-diag/material3-chip-visual-chrome-20260528/sessions/1779935853792-64132/share/1779936147211.zip`

## Residual Risk

Per-draw-operation elevation or non-rectangular state-layer inspection remains a future diagnostics
mechanism only if a concrete product or parity case needs it.
