# Editor Canvas Paint Replay Row Setup v1 TODO

## Tasks

- [x] ECPR-RS-010: Add planned replay setup attribution.
  - Scope:
    `ecosystem/fret-code-editor/src/editor/diagnostics.rs`,
    `ecosystem/fret-code-editor/src/editor/paint/mod.rs`,
    `apps/fret-ui-gallery/src/driver/diag_snapshot.rs`,
    `crates/fret-diag/src/stats.rs`,
    `crates/fret-diag/src/stats/bundle_stats_compute.inc.rs`,
    `crates/fret-diag/src/stats/bundle_stats_report.inc.rs`.
  - Expected result:
    code-editor paint diagnostics, app snapshots, and `fret-diag stats` expose
    `us_row_scene_replay_setup` / `ns_row_scene_replay_setup` for matching planned replay rows.
  - Gates:
    - `cargo nextest run -p fret-diag bundle_stats_extracts_code_editor_paint_perf_from_app_snapshot --no-fail-fast`
    - `cargo check -p fret-code-editor --tests --features syntax-rust`
    - `cargo check -p fret-diag --tests`
  - Result:
    implemented on 2026-05-24. The counter is diagnostics-only and does not alter replay behavior.

- [x] ECPR-RS-020: Run local format/catalog/diff gates and refresh evidence.
  - Commands:
    - `cargo fmt -p fret-code-editor -p fret-diag -p fret-ui-gallery --check`
    - `python -m json.tool docs/workstreams/editor-canvas-paint-replay-row-setup-v1/WORKSTREAM.json`
    - `python -m json.tool docs/workstreams/ui-perf-zed-smoothness-v1/WORKSTREAM.json`
    - `python tools/check_workstream_catalog.py`
    - `git diff --check`
  - Result:
    passed on 2026-05-24.

- [x] ECPR-RS-030: Run target-machine attribution before optimizing.
  - Required shape:
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r64-row-setup-baseline --keep-going`
    - `python tools/perf/diag_editor_paint_contract_validate.py --date-tag 20260524-r64-row-setup-attrib-rebuilt --with-paint-perf --keep-going`
    - artifact verifier and closeout over the two directories.
  - Result:
    passed on 2026-05-24 after rebuilding `target/release/fretboard-dev.exe` and the release
    gallery so the target-machine attribution bundle included the new schema `14` counter.
  - Evidence:
    - Baseline validation:
      `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/summary.json`
    - Rebuilt attribution validation:
      `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-attrib-rebuilt/summary.json`
    - Artifact verifier:
      `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/artifact-verification.summary.json`
    - Closeout:
      `target/fret-diag/editor-paint-contract-validate-20260524-r64-row-setup-baseline/editor-paint-contract-closeout.summary.json`
  - Decision rule:
    this diagnostics slice should not change checked-in baselines. Use the target-machine
    attribution to decide whether the next implementation slice attacks row setup, replay ops,
    Canvas overhead outside the editor, or another owner.

## Current Decision

Closed. The diagnostics slice is implemented, locally gated, and target-machine verified. The
closeout still selects `owner=canvas-paint-replay`; the next implementation work should start in a
new bounded follow-on for row replay setup/ops/touch together rather than keeping this diagnostics
lane open.
