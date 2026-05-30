# IconButton Diagnostics Packet v1

## Truth

- IconButton centered chrome is a recipe/foundation outcome: the root keeps the minimum target and
  the visual chrome remains centered.
- The existing diagnostics script was stale because it opened the aggregate Material3 gallery page.
- After navigation repair, no component drift was found.

## Artifacts

- `ecosystem/fret-ui-material3/src/icon_button.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/icon_button.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The diagnostics script opens the dedicated Material3 Icon Button page, waits for
`ui-gallery-material3-icon-button-centered`, then asserts minimum target size, bounded chrome size,
and root/chrome center alignment.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-icon-button-centered-chrome.json --dir target/fret-diag/material3-icon-button-centered-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Result: `PASS`, run id `1779937783108`.

Bounded evidence:

- `target/fret-diag/material3-icon-button-centered-chrome-20260528/sessions/1779937486360-34444/1779937783108/ai.packet`
- `target/fret-diag/material3-icon-button-centered-chrome-20260528/sessions/1779937486360-34444/share/1779937783108.zip`

## Residual Risk

No current IconButton component, kit-policy, or mechanism residual remains from this packet.
