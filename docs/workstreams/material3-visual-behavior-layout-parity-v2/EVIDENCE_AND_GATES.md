# Material 3 Visual Behavior Layout Parity v2 - Evidence And Gates

Status: Active
Last updated: 2026-05-28

## Smallest Current Repro

The seed evidence is the closed v1 component sweep and follow-on closure audit:

```powershell
python -m json.tool docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
```

## Gate Set

### Workstream State

```powershell
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null
python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null
python tools/check_workstream_catalog.py
```

### Material Crate Inner Loop

Use narrow gates first:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --test radio_alignment
cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

### Diagnostics

For motion, overlay, or layout-visible work, prefer fixed-timestep diagnostics:

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/<script>.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/<lane-task> --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
```

## Evidence Anchors

- `docs/workstreams/material3-visual-behavior-layout-parity-v2/DESIGN.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/TODO.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/material3_follow_on_closure_audit_v1.md`
- `docs/workstreams/material3-component-alignment-sweep-v1/artifacts/component_alignment_matrix_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/`
- `docs/audits/shadcn-select.md`
- `ecosystem/fret-ui-material3/src/`
- `ecosystem/fret-ui-material3/src/foundation/`
- `ecosystem/fret-ui-material3/src/interaction/`
- `ecosystem/fret-ui-material3/tests/`
- `apps/fret-ui-gallery/src/ui/pages/material3/`
- `apps/fret-ui-gallery/src/ui/snippets/material3/`
- `tools/diag-scripts/ui-gallery/material3/`
- `repo-ref/material-ui`
- `repo-ref/compose-multiplatform-core`
- `repo-ref/base-ui`

## Fresh Evidence Log

- 2026-05-28: M3PV2-010 opened the v2 fearless-refactor lane and created the parity-axis matrix.
  - `python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json | Out-Null`
  - `python -m json.tool docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json | Out-Null`
  - `python tools/check_workstream_catalog.py`
  - Result: the v2 matrix covers all 39 Material3 components from the closed sweep and assigns
    style/layout/behavior/accessibility/motion axis state plus a first v2 gate.

## Proof Note Template

Each v2 packet must record:

- Truth: the observable Material outcomes by axis.
- Sources: Material spec, Compose, MUI, Base UI, or Fret-side exemplar.
- Artifacts: component files, foundation helpers, snippets, diagnostics, tests, or goldens.
- Wiring: which rendered surface actually uses the behavior.
- Proof: exact commands and evidence output.
- Residual risk: what remains unmeasured.
