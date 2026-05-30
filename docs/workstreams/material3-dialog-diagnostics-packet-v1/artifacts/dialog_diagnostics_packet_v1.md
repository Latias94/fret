# Dialog Diagnostics Packet v1

## Truth

- Material3 Dialog exposes stable scrim, panel, panel chrome, and action selectors on the dedicated
  gallery page.
- The panel reports Dialog semantics while `fret-ui-kit` owns the modal barrier and focus barrier.
- Escape dismissal clears the barrier roots and restores focus to the public trigger.
- Existing Rust gates continue to prove focus containment, scrim dismissal without underlay
  activation, and style override wiring.

## Artifacts

- `ecosystem/fret-ui-material3/src/dialog.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/dialog.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json`
- `tools/diag-scripts/suites/ui-gallery-material3-dialog-focus-trap-restore/suite.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

The diagnostics script opens `ui-gallery-page-material3-dialog`, activates
`ui-gallery-material3-dialog-open`, waits for `ui-gallery-material3-dialog.panel`,
`ui-gallery-material3-dialog.scrim`, the Dialog role, and matching modal/focus barrier roots, then
presses Escape and waits for barrier cleanup plus focus restoration to the trigger.

## Proof

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json --dir target/fret-diag/material3-dialog-focus-trap-restore-20260528-final --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Result: `PASS`, run id `1779939874070`.

Bounded evidence:

- `target/fret-diag/material3-dialog-focus-trap-restore-20260528-final/sessions/1779939582503-8856/1779939874070/ai.packet`
- `target/fret-diag/material3-dialog-focus-trap-restore-20260528-final/sessions/1779939582503-8856/share/1779939874070.zip`

## Residual Risk

No current Dialog recipe, foundation, kit-policy, or mechanism residual remains from this packet.
Future changes to generic modal overlay policy should continue in `fret-ui-kit` or a mechanism lane
only if a concrete cross-design-system gap appears.
