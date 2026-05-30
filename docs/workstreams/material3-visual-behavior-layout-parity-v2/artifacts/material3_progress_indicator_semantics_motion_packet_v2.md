# Material3 Progress Indicator Semantics And Motion Packet v2

Date: 2026-05-29
Task: M3PV2-064

## Truth

- Linear and circular progress indicators expose `ProgressBar` semantics.
- Determinate indicators expose numeric range metadata: current value, min `0`, and max `1`.
- Indeterminate indicators expose busy state and no determinate numeric value.
- Progress indicators accept an explicit accessible label on the recipe surface.
- Linear and circular determinate indicators expose stable track and active-track part ids.
- Indeterminate linear and circular draw regions move across fixed frames and continue requesting
  animation frames.

## Sources

- Compose Material3 `ProgressIndicator.kt`: determinate linear and circular indicators set
  `ProgressBarRangeInfo(current, 0f..1f)` in semantics; indeterminate circular uses
  `progressSemantics()`.
- Compose Material3 `ProgressIndicatorDefaults`: progress animation uses a low-bounce spring for
  determinate value changes, linear indicators have track/active/stop regions, and circular
  indeterminate track color defaults to transparent.
- Compose Material3 `LinearProgressIndicatorTokens.kt` and `CircularProgressIndicatorTokens.kt`:
  linear height/thickness, circular size/thickness, track-active gap, wave metrics, and active
  thickness tokens.
- Existing Fret implementation already used generated Material Web v30 token colors/shapes and a
  deterministic frame-based indeterminate renderer; this packet added the missing semantics and
  explicit motion gate.

MUI Material UI is not available in this worktree's `repo-ref/`; this packet used local Compose
and generated Material Web token snapshots.

## Layer Finding

This packet found a Material recipe/diagnostics wiring gap, not a core or kit mechanism gap:

- `fret-core` already has `SemanticsRole::ProgressBar`, numeric range metadata, and busy flags.
- The Material recipe wrapped progress indicators as generic test-id semantics nodes and did not
  publish progress range metadata.
- Linear determinate indicators already exposed `.track` and `.active-track` anchors; circular
  determinate indicators did not expose equivalent stable parts.
- The indeterminate renderer already advanced by frame id, but no focused gate proved draw-region
  movement across frames.

No cross-crate mechanism change was required.

## Artifacts

- `ecosystem/fret-ui-material3/src/progress_indicator.rs`
- `ecosystem/fret-ui-material3/tests/progress_indicator_state.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/artifacts/material3_parity_axis_matrix_v2.json`

## Proof

Red gate before fix:

```powershell
cargo nextest run -p fret-ui-material3 --features diagnostics --test progress_indicator_state
```

The new test initially failed because `LinearProgressIndicator` and `CircularProgressIndicator`
had no local `a11y_label` builder and the current semantics node did not expose progressbar
semantics.

Green gates:

```powershell
cargo fmt --package fret-ui-material3
cargo nextest run -p fret-ui-material3 --features diagnostics --test progress_indicator_state
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface material3_surface_data_display_expose_stable_part_test_ids
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_progress_indicator_suite_goldens_v1
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\WORKSTREAM.json | Out-Null
python -m json.tool docs\workstreams\material3-visual-behavior-layout-parity-v2\artifacts\material3_parity_axis_matrix_v2.json | Out-Null
python tools\check_workstream_catalog.py
git diff --check
cargo check -p fret-ui-material3 --features diagnostics --tests
cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings
```

The first headless progress golden run timed out while waiting on Cargo build locks; the immediate
long-timeout rerun passed without a golden refresh.

## Residual Risk

- Determinate value interpolation uses caller-driven model updates; this packet did not add an
  internal spring for changes between determinate values.
- Linear default width remains on the existing Fret fill-width recipe path; this packet focused on
  semantics and animated draw-region proof.
- Wavy progress indicators are not implemented in the current Fret Material3 surface.
