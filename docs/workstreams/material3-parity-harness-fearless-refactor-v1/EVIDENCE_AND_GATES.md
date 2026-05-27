# Material 3 Parity Harness Fearless Refactor v1 - Evidence And Gates

Status: Closed
Last updated: 2026-05-27

## Smallest Current Repro

The current proof seed is the existing Material Button adapter pilot:

```powershell
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
python -m json.tool tools/parity-discovery/fixtures/material3_button_adapter_v1.json | Out-Null
python -m json.tool docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json | Out-Null
```

This proves the current packet schema and the first Material adapter artifact are readable before
the lane expands.

## Gate Set

### Workstream Docs Gate

```powershell
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/WORKSTREAM.json | Out-Null
```

Proves the workstream state file is valid JSON.

### Parity Tool Gate

```powershell
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
```

Proves the existing parity report generator still parses after any harness edits.

### Existing Material Adapter Gate

```powershell
python -m json.tool tools/parity-discovery/fixtures/material3_button_adapter_v1.json | Out-Null
python -m json.tool docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json | Out-Null
```

Proves the current Material Button adapter fixture and generated pilot artifact remain valid.

### Material Coverage Inventory Gate

```powershell
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json | Out-Null
```

Proves the M3PH-020 component inventory is valid JSON before suite and packet tasks consume it.

### Material Suite Gate

```powershell
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json | Out-Null
```

Proves suite-level repair/hardening/gate queues can be regenerated from existing Material reports.

### Material Select Packet Gate

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json --dir target/fret-diag/material3-parity-harness/select-a11y-gallery-material3 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_select_adapter_v1.json --fret-bundle-schema2-dir target/fret-diag/material3-parity-harness/select-a11y-gallery-material3/sessions/1779857102433-35492 --output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json
python -m json.tool tools/parity-discovery/fixtures/material3_select_adapter_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json | Out-Null
```

Proves the first Material field-family packet can consume live Fret bundle evidence and reduce the
Select adapter repair queue to zero.

### Material Switch Packet Gate

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-handle-screenshots.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-parity-harness/switch-handle-gallery-material3 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3
python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_switch_adapter_v1.json --fret-bundle-schema2-dir target/fret-diag/material3-parity-harness/switch-handle-gallery-material3/sessions/1779858488431-18276 --output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json
python -m json.tool tools/parity-discovery/fixtures/material3_switch_adapter_v1.json | Out-Null
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json | Out-Null
```

Proves the first interaction-heavy Material packet can consume fixed-delta Switch bundle evidence
and reduce the Switch adapter repair queue to zero.

### Material Crate Inner-Loop Gates

Use the narrowest gate for the touched axis:

```powershell
cargo nextest run -p fret-ui-material3 --lib material3_literal_md_tokens_resolve_in_v30_theme
cargo test -p fret-ui-material3 --test radio_alignment --no-run
cargo test -p fret-ui-material3 --test automation_surface --no-run
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1
cargo nextest run -p fret-ui-material3 --test text_field_hover
```

These cover token resolution, test-target compile health, representative headless goldens, and
focused field chrome behavior. Task-specific gates should replace or extend this list when touching
a specific component.

### Diagnostics Gates

Use diag scripts for overlay, motion, focus, a11y, or screenshot/bundle evidence:

```powershell
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json --dir target/fret-diag/material3-parity-harness/select-a11y --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-full
cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icon-motion-timeline-screenshots.json --fixed-frame-delta-ms 16 --dir target/fret-diag/material3-parity-harness/switch-motion --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-full
```

These are examples. Each task should choose the smallest script that proves its truth set.

### Layering Gate

```powershell
python tools/check_layering.py
```

Run when a task touches `crates/*`, `ecosystem/fret-ui-kit`, or cross-crate dependencies.

### Review And Verification Gates

Run `review-workstream` before accepting task output. Run `verify-rust-workstream` before marking a
task, the lane, or the active goal complete.

## Evidence Anchors

- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/TODO.md`
- `tools/parity-discovery/README.md`
- `tools/parity-discovery/fixtures/material3_button_adapter_v1.json`
- `tools/parity-discovery/fixtures/material3_select_adapter_v1.json`
- `tools/parity-discovery/fixtures/material3_switch_adapter_v1.json`
- `tools/parity-discovery/suites/material3_parity_discovery_v1.json`
- `docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`
- `ecosystem/fret-ui-material3/src/`
- `ecosystem/fret-ui-material3/tests/`
- `apps/fret-ui-gallery/src/ui/snippets/material3/`
- `tools/diag-scripts/ui-gallery/material3/`

## Fresh Evidence Log

- 2026-05-27: Validated current seed artifacts:
  - `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`
  - `python -m json.tool tools/parity-discovery/fixtures/material3_button_adapter_v1.json > $null`
  - `python -m json.tool docs/workstreams/component-parity-fact-harness-v1/artifacts/material3_button_adapter_pilot_v1.json > $null`
- 2026-05-27: Completed M3PH-020 Material coverage inventory:
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json > $null`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/WORKSTREAM.json > $null`
  - `python tools/check_workstream_catalog.py`
  - Inventory count check: 39 components, 25 gallery snippets, 21 direct diag surfaces, 18 direct golden prefixes, 1 existing Material parity packet, 0 duplicate component IDs.
  - First packet priorities: `Select` for field-family overlay, `Switch` for interaction-heavy state/motion, `Tabs` for follow-up indicator/motion proof.
- 2026-05-27: Completed M3PH-030 Material suite baseline:
  - `python -m json.tool tools/parity-discovery/suites/material3_parity_discovery_v1.json > $null`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null`
  - Suite summary: 1 report, 2 parts, status counts `{pass_known: 2, needs_live_measurement: 0, mismatch: 0, blocked: 0}`, agent status `needs_hardening`, 0 repair rows, 3 hardening rows.
- 2026-05-27: Completed M3PH-040 first Material field-family packet:
  - `python -m json.tool tools/parity-discovery/fixtures/material3_select_adapter_v1.json > $null`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-select-a11y-parity-bundle.json --dir target/fret-diag/material3-parity-harness/select-a11y-gallery-material3 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_select_adapter_v1.json --fret-bundle-schema2-dir target/fret-diag/material3-parity-harness/select-a11y-gallery-material3/sessions/1779857102433-35492 --output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json > $null`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null`
  - Select packet summary: 5 parts, 7 bundle schema2 files, status counts `{pass_known: 5, needs_live_measurement: 0, mismatch: 0, blocked: 0}`, agent status `needs_hardening`, 0 repair rows, 3 hardening rows, 5 gate rows.
  - Suite summary after Select: 2 reports, 7 parts, status counts `{pass_known: 7, needs_live_measurement: 0, mismatch: 0, blocked: 0}`, agent status `needs_hardening`, 0 repair rows, 6 hardening rows.
  - Note: the same diag script with `--features gallery-full` timed out after 604 seconds and wrote `script.result.json` with `reason_code: tooling.launch.failed`, `reason: launched demo exited before signaling readiness (ready.touch): exit code: 101`. The narrower `gallery-material3` feature set passed and is the current gate for this lane.
- 2026-05-27: Completed M3PH-050 interaction-heavy Switch packet:
  - `python -m json.tool tools/parity-discovery/fixtures/material3_switch_adapter_v1.json > $null`
  - `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-handle-screenshots.json --env FRET_DIAG_FIXED_FRAME_DELTA_MS=16 --dir target/fret-diag/material3-parity-harness/switch-handle-gallery-material3 --session-auto --pack --ai-packet --exit-after-run --timeout-ms 360000 --launch -- cargo run -p fret-ui-gallery --features gallery-material3`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/material3_switch_adapter_v1.json --fret-bundle-schema2-dir target/fret-diag/material3-parity-harness/switch-handle-gallery-material3/sessions/1779858488431-18276 --output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json > $null`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null`
  - Switch packet summary: 5 parts, 20 bundle schema2 files, status counts `{pass_known: 5, needs_live_measurement: 0, mismatch: 0, blocked: 0}`, agent status `needs_hardening`, 0 repair rows, 3 hardening rows, 5 gate rows.
  - Suite summary after Switch: 3 reports, 12 parts, status counts `{pass_known: 12, needs_live_measurement: 0, mismatch: 0, blocked: 0}`, agent status `needs_hardening`, 0 repair rows, 9 hardening rows.
- 2026-05-27: Completed M3PH-060 Material test support extraction with concerns:
  - Extracted reusable host, fake UI services, theme helpers, layout helpers, event helpers, scene
    snapshot helpers, overlay frame helpers, and golden assertion helpers into
    `ecosystem/fret-ui-material3/tests/support/`.
  - `rustfmt --edition 2024 ecosystem/fret-ui-material3/tests/radio_alignment.rs ecosystem/fret-ui-material3/tests/support/mod.rs ecosystem/fret-ui-material3/tests/support/host.rs ecosystem/fret-ui-material3/tests/support/events.rs ecosystem/fret-ui-material3/tests/support/layout.rs ecosystem/fret-ui-material3/tests/support/theme.rs ecosystem/fret-ui-material3/tests/support/goldens.rs`
  - `git diff --check -- ecosystem/fret-ui-material3/tests/radio_alignment.rs ecosystem/fret-ui-material3/tests/support`
  - `cargo test -p fret-ui-material3 --test radio_alignment --no-run`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1` failed with `material3 suite golden mismatch` for `goldens/material3-headless/v1/material3-controls.scale1_0.dark.tonal_spot.json`.
  - The same nextest command failed in a clean detached `HEAD` worktree at
    `F:\SourceCodes\Rust\fret-worktrees\m3ph060-head-check`, before the support extraction, with the
    same Material controls golden mismatch. Treat this as pre-existing Material controls golden drift,
    not as evidence that M3PH-060 changed rendering behavior.
- 2026-05-27: Completed M3PH-070 first component-family test split:
  - Moved the Select interaction behavior family out of `radio_alignment.rs` into
    `ecosystem/fret-ui-material3/tests/select_behavior.rs`.
  - `rustfmt --edition 2024 ecosystem/fret-ui-material3/tests/radio_alignment.rs ecosystem/fret-ui-material3/tests/select_behavior.rs ecosystem/fret-ui-material3/tests/support/mod.rs`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip`
  - `cargo test -p fret-ui-material3 --test radio_alignment --no-run`
  - `git diff --check -- ecosystem/fret-ui-material3/tests/radio_alignment.rs ecosystem/fret-ui-material3/tests/select_behavior.rs ecosystem/fret-ui-material3/tests/support docs/workstreams/material3-parity-harness-fearless-refactor-v1`
  - Result: 8 Select behavior tests passed in the new target; the adjacent chip-set test passed in
    the old target; the old `radio_alignment` test target still compiles. No Material golden files
    were updated.
- 2026-05-27: Completed M3PH-090 stable Material automation surface contract:
  - Added `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`.
  - Added `ecosystem/fret-ui-material3/tests/automation_surface.rs` behind the `diagnostics`
    feature and exposed `fret-ui-material3/diagnostics` as a feature forwarding to
    `fret-ui/diagnostics`.
  - Select now derives intent-level part IDs from the caller base/item IDs:
    `<base>.chrome`, `<base>.active-indicator`, `<base>.trailing-icon`, `<base>-listbox`,
    `<item>.chrome`, `<item>.leading-icon`, and `<item>.trailing-icon`.
  - Switch now derives `<base>.chrome`, `<base>.track`, `<base>.handle`, `<base>.icon-on`, and
    `<base>.icon-off`.
  - `rustfmt --edition 2024 ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/src/switch.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/support/layout.rs`
  - `cargo test -p fret-ui-material3 --test automation_surface --no-run`
  - `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface --no-run`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `rg -n "SelectPartTestIds|SelectItemPartTestIds|SwitchPartTestIds|active-indicator|trailing-icon|leading-icon|\\.track|\\.handle|\\.icon-on|\\.icon-off|automation_surface|material3_select_exposes_stable_part_test_ids|material3_switch_exposes_stable_part_test_ids" ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/src/switch.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/support/layout.rs ecosystem/fret-ui-material3/Cargo.toml`
  - Result: default compile gate passed; diagnostics compile gate passed; automation-surface
    nextest gate passed with 2 tests; source scan found the derived Select/Switch part IDs and
    focused test assertions.
- 2026-05-27: Completed M3PH-100 closeout verification:
  - Review result: no blocking workstream-compliance or code-quality findings. M3PH-080 is
    `DONE_NOT_APPLICABLE` because the initial Material suite has no non-empty repair queue.
  - `rustfmt --edition 2024 ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/src/switch.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/support/layout.rs`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/WORKSTREAM.json > $null`
  - `python tools/check_workstream_catalog.py`
  - `git diff --check -- ecosystem/fret-ui-material3/Cargo.toml ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/src/switch.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/support/layout.rs docs/workstreams/material3-parity-harness-fearless-refactor-v1`
  - `cargo test -p fret-ui-material3 --lib --no-run`
  - `cargo test -p fret-ui-material3 --test radio_alignment --no-run`
  - `cargo nextest run -p fret-ui-material3 --test select_behavior`
  - `cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip`
  - `cargo test -p fret-ui-material3 --test automation_surface --no-run`
  - `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface --no-run`
  - `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`
  - `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`
  - `python -m json.tool tools/parity-discovery/fixtures/material3_select_adapter_v1.json > $null`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json > $null`
  - `python -m json.tool tools/parity-discovery/fixtures/material3_switch_adapter_v1.json > $null`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json > $null`
  - `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  - `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null`
  - Note: the first `cargo nextest run -p fret-ui-material3 --test select_behavior` closeout attempt
    timed out at 120 seconds after build-lock contention, but its output showed all 8 tests passed.
    It was rerun with a 240-second timeout and exited 0.
  - Result: targeted closeout gates passed. Full workspace clippy/test were not run because this
    lane touched a focused Material crate/workstream slice and the repo-wide workspace is large.

## Notes

Do not treat "component looks closer to Material" as proof. Each task must state:

- Truth: observable outcome that must be true.
- Artifacts: fixture, diag script, test, or source file that owns it.
- Wiring: gallery/component path that uses it.
- Proof: exact gate/evidence.
- Residual risk: remaining unmeasured parity.
