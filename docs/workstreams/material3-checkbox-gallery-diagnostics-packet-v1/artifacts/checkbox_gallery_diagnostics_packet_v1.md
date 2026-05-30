# Checkbox Gallery Diagnostics Packet v1

## Truth

- Checkbox's remaining residual was gallery diagnostics evidence, not a component architecture gap.
- The pre-existing centered-chrome script targeted the wrong page and therefore failed before it
  reached the component.
- Once the script opened the dedicated Checkbox page, centered chrome and tri-state behavior passed.

## Artifacts

- `ecosystem/fret-ui-material3/src/checkbox.rs`
- `apps/fret-ui-gallery/src/ui/snippets/material3/checkbox.rs`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json`
- `tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-tristate-screenshots.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`

## Wiring

- Centered chrome diagnostics navigate to `material3_checkbox`, wait for
  `ui-gallery-page-material3-checkbox`, then assert:
  - root `ui-gallery-material3-checkbox` is at least 48px by 48px,
  - chrome `ui-gallery-material3-checkbox.chrome` stays bounded,
  - root and chrome centers align.
- Tri-state diagnostics navigate to the same page and capture standard/expression states while
  asserting checkbox role and checked/mixed state transitions.

## Proof

Pre-fix stale diagnostics run:

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json --dir target/fret-diag/material3-checkbox-centered-chrome-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Result: failed at step 13 waiting for `ui-gallery-material3-checkbox` on the aggregate Material3
gallery page.

Fixed runs:

```powershell
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json --dir target/fret-diag/material3-checkbox-centered-chrome-20260528-fixed --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-tristate-screenshots.json --dir target/fret-diag/material3-checkbox-tristate-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

Results:

- centered chrome: `PASS`, run id `1779935007417`
- tri-state screenshots: `PASS`, run id `1779935349931`

Bounded evidence:

- `target/fret-diag/material3-checkbox-centered-chrome-20260528-fixed/sessions/1779934722696-58708/1779935007417/ai.packet`
- `target/fret-diag/material3-checkbox-centered-chrome-20260528-fixed/sessions/1779934722696-58708/share/1779935007417.zip`
- `target/fret-diag/material3-checkbox-tristate-20260528/sessions/1779935052939-58744/1779935349931/ai.packet`
- `target/fret-diag/material3-checkbox-tristate-20260528/sessions/1779935052939-58744/share/1779935349931.zip`

## Residual Risk

Checkbox has no open component, kit-policy, or mechanism residual from this packet. Future visual
drift should start from the two promoted gallery scripts plus focused headless tests.
