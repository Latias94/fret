# Material 3 Parity Harness Fearless Refactor v1 - Closeout Audit

Status: Closed
Last updated: 2026-05-27

## Verdict

This lane is closed.

The target state is satisfied: Material 3 now has a shadcn-shaped parity harness baseline with a
suite manifest, Button/Select/Switch packets, reusable Material test support, a focused Select
behavior test target, and a gated Select/Switch automation-surface contract.

No mechanism-layer defect was found. The initial Material suite has no non-empty repair queue, so
M3PH-080 is closed as not applicable rather than inventing a repair.

## Shipped State

- `tools/parity-discovery/suites/material3_parity_discovery_v1.json` covers the initial Material
  suite.
- `material3_parity_suite_report_v1.json` regenerates from existing reports with 3 reports, 12
  parts, 12 `pass_known` rows, 0 `mismatch`, and 0 `blocked`.
- Select is the first field-family packet.
- Switch is the first interaction-heavy packet.
- `ecosystem/fret-ui-material3/tests/support/` owns reusable host, event, layout, theme, overlay,
  snapshot, and golden helpers previously trapped in `radio_alignment.rs`.
- `ecosystem/fret-ui-material3/tests/select_behavior.rs` owns the Select behavior family.
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`
  records stable Material selector rules.
- `ecosystem/fret-ui-material3/tests/automation_surface.rs` gates Select and Switch live selector
  surfaces through `fret-ui/diagnostics`.

## Implementation Boundaries

- Material policy remained in `ecosystem/fret-ui-material3`.
- No interaction policy was moved into `crates/fret-ui`.
- The new `fret-ui-material3/diagnostics` feature only forwards to `fret-ui/diagnostics` for the
  focused automation-surface gate.
- No Material golden files were refreshed in this lane.

## Fresh Gates

```powershell
rustfmt --edition 2024 ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/src/switch.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/support/layout.rs
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/WORKSTREAM.json > $null
python tools/check_workstream_catalog.py
git diff --check -- ecosystem/fret-ui-material3/Cargo.toml ecosystem/fret-ui-material3/src/select.rs ecosystem/fret-ui-material3/src/switch.rs ecosystem/fret-ui-material3/tests/automation_surface.rs ecosystem/fret-ui-material3/tests/support/layout.rs docs/workstreams/material3-parity-harness-fearless-refactor-v1
cargo test -p fret-ui-material3 --lib --no-run
cargo test -p fret-ui-material3 --test radio_alignment --no-run
cargo nextest run -p fret-ui-material3 --test select_behavior
cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip
cargo test -p fret-ui-material3 --test automation_surface --no-run
cargo test -p fret-ui-material3 --features diagnostics --test automation_surface --no-run
cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface
python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py
python -m json.tool tools/parity-discovery/fixtures/material3_select_adapter_v1.json > $null
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json > $null
python -m json.tool tools/parity-discovery/fixtures/material3_switch_adapter_v1.json > $null
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json > $null
python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json
python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json > $null
```

All closeout gates passed. The first closeout attempt for `cargo nextest run -p fret-ui-material3
--test select_behavior` hit the 120-second tool timeout after build-lock contention, but its output
already showed all 8 tests passed. The command was rerun with a longer timeout and exited 0.

## Follow-Ons

Start a new narrow workstream instead of reopening this lane for:

- pre-existing `material3-controls.scale1_0.dark.tonal_spot.json` golden drift classification,
- the next Material packet, likely Tabs or navigation active-indicator/motion,
- broader Material component coverage,
- promotion of hardening rows into permanent fixtures or diag scripts,
- any future packet that proves a real `fret-ui-kit` or `crates/*` mechanism defect.

## Residual Risks

- The suite proves Button, Select, and Switch only; it is not a complete Material parity matrix.
- Select/Switch automation surfaces are gated; future components must extend the test-id contract
  before adding new diagnostics predicates.
- The known Material controls golden drift is unchanged and intentionally not refreshed here.
- Full workspace clippy/test were not run because this lane touched a focused Material crate and
  workstream slice; closeout used targeted gates recorded above.
