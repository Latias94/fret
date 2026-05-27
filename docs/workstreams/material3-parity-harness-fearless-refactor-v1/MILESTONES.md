# Material 3 Parity Harness Fearless Refactor v1 - Milestones

Status: Closed
Last updated: 2026-05-27

## M0 - Scope And Evidence Freeze

Exit criteria:

- The lane has authoritative workstream docs.
- Existing Material, shadcn, and component parity docs are linked.
- The first executable task is chosen.

Primary evidence:

- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/TODO.md`

## M1 - Material Harness Inventory

Exit criteria:

- Material components are mapped to source refs, gallery snippets, diag scripts, headless tests, and
  current packet coverage.
- High-risk components are classified by family: field, choice control, navigation, overlay, motion,
  or foundation.
- Missing stable automation surfaces are explicit.

Primary gates:

- JSON validation for the inventory artifact.
- Manual review of at least field-family and interaction-heavy priority choices.

Primary evidence:

- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json`

## M2 - Suite Manifest And Packet Baseline

Exit criteria:

- A Material suite manifest exists.
- The existing Button adapter is included as a suite entry.
- One field-family packet and one interaction-heavy packet exist or are explicitly blocked by
  missing evidence.
- Suite-level summary output preserves `repair_queue`, `hardening_queue`, and `gate_queue`.

Primary gates:

- `python -m py_compile tools/parity-discovery/shadcn_parity_discovery.py`
- `python tools/parity-discovery/shadcn_parity_discovery.py --suite <material-suite> --suite-from-existing-reports --suite-output <suite-report>`
- JSON validation for suite, fixtures, and reports.

Primary evidence:

- `tools/parity-discovery/suites/material3_parity_discovery_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
- `tools/parity-discovery/fixtures/material3_select_adapter_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json`
- `tools/parity-discovery/fixtures/material3_switch_adapter_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`

## M3 - Test Harness Refactor

Exit criteria:

- Reusable Material test host/snapshot/golden helpers live outside `radio_alignment.rs`.
- At least one component-family test target is split or a low-churn split plan is recorded.
- Existing targeted Material tests still pass, or pre-existing drift is explicitly classified.

Primary gates:

- `cargo test -p fret-ui-material3 --test radio_alignment --no-run`
- `cargo nextest run -p fret-ui-material3 --test select_behavior`
- `cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip`
- Known concern: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  currently fails in clean `HEAD` with Material controls golden drift.

## M4 - Repair Queues From Evidence

Exit criteria:

- At least one non-empty Material packet queue is acted on, or all initial packets prove no repair
  rows and only hardening/gate rows remain.
- Stable automation surfaces for the covered packets are documented and gated.
- Fixes are made by owner/layer classification.
- Material policy does not move into `crates/*`.

Primary gates:

- Refreshed packet report for the fixed row.
- Relevant Rust or diag gate chosen by row axis.
- `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`

Primary evidence:

- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`

## M5 - Verification And Closeout

Exit criteria:

- Fresh evidence is recorded.
- Remaining risks are listed in `HANDOFF.md`.
- Follow-ons are split by component family or mechanism boundary.
- `WORKSTREAM.json` is updated.

Primary gates:

- `review-workstream`
- `verify-rust-workstream`

Closeout result:

- Closed on 2026-05-27.
- The Material suite regenerates from existing reports with 3 reports, 12 parts, and 0 top
  findings.
- Button, Select, and Switch are represented as the first three Material packets.
- Select/Switch automation surfaces are documented and gated.
- Material test support is extracted from `radio_alignment.rs`, and Select behavior has a focused
  test target.
- Remaining work is split to follow-ons instead of expanding this lane.
