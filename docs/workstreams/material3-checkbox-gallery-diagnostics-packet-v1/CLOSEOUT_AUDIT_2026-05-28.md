# Material 3 Checkbox Gallery Diagnostics Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Closed the Checkbox gallery diagnostics residual from the Material3 component matrix.

## Result

- The only defect found was stale diagnostics navigation.
- The centered-chrome script now targets `ui-gallery-nav-material3-checkbox` and waits for
  `ui-gallery-page-material3-checkbox`.
- Repaired centered-chrome diagnostics passed and proved the 48px interaction target plus centered
  visual chrome.
- Existing tri-state screenshots diagnostics passed and proved indeterminate, checked, unchecked,
  and expressive-mode states.

## Gates

- `python -m json.tool tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-centered-chrome.json --dir target/fret-diag/material3-checkbox-centered-chrome-20260528-fixed --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
- `cargo run -p fretboard-dev -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-checkbox-tristate-screenshots.json --dir target/fret-diag/material3-checkbox-tristate-20260528 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 900000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`

## Layering

- `material_recipe`: Checkbox model mapping, semantics, and chrome assembly remain recipe-owned.
- `material_foundation`: shared indication/ripple and minimum target sizing remain foundation-owned.
- `diagnostics`: stale navigation was repaired.
- `kit_policy`: no new shared policy was found.
- `mechanism`: no core mechanism gap was found.
