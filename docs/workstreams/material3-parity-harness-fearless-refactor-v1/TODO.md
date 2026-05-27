# Material 3 Parity Harness Fearless Refactor v1 - TODO

Status: Closed
Last updated: 2026-05-27

Task IDs use `M3PH-*`.

## M0 - Scope And Evidence Freeze

- [x] M3PH-010 [owner=planner] [deps=none] [scope=docs/workstreams/material3-parity-harness-fearless-refactor-v1]
  Goal: Open the durable workstream and freeze the refactor direction.
  Validation: `DESIGN.md`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, `WORKSTREAM.json`, and `HANDOFF.md` exist and agree.
  Evidence: `docs/workstreams/material3-parity-harness-fearless-refactor-v1/DESIGN.md`
  Handoff: Planner owns the first executable slice selection.

## M1 - Material Harness Inventory

- [x] M3PH-020 [owner=planner] [deps=M3PH-010] [scope=ecosystem/fret-ui-material3,apps/fret-ui-gallery,tools/diag-scripts,tools/parity-discovery]
  Goal: Produce a Material coverage inventory that maps components to upstream refs, gallery snippets, diag scripts, headless tests, goldens, and required stable `test_id` anchors.
  Validation: `python -m json.tool docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json > $null`; targeted source scan has no unclassified high-risk component.
  Review: Inventory identifies `Select` as the first field-family packet, `Switch` as the first interaction-heavy packet, and `Tabs` as the next motion/indicator candidate. Run `review-workstream` before implementation tasks start.
  Evidence: `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json`
  Handoff: The inventory must identify the first field-family and interaction-heavy proof components.

## M2 - Suite Manifest And Packet Baseline

- [x] M3PH-030 [owner=planner] [deps=M3PH-020] [scope=tools/parity-discovery,docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts]
  Goal: Promote the existing `material3_button_adapter_v1.json` pilot into a Material suite manifest with suite-level summary output.
  Validation: `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json` and JSON validation.
  Review: Report preserves owner/layer summary counts and suite-level agent packet queues: 1 report, 2 parts, 2 `pass_known`, 0 repair rows, 3 hardening rows.
  Evidence: `tools/parity-discovery/suites/material3_parity_discovery_v1.json`, `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  Handoff: Do not add broad component fixtures until the suite command is stable.

- [x] M3PH-040 [owner=planner] [deps=M3PH-030] [scope=tools/parity-discovery/fixtures,tools/diag-scripts/ui-gallery/material3,apps/fret-ui-gallery/src/ui/snippets/material3]
  Goal: Add the first field-family Material packet, preferring `Select` unless M3PH-020 finds a stronger blocker.
  Validation: targeted Select a11y diag capture using `gallery-material3`, packet regeneration with 7 bundle schema2 files, suite regeneration, and JSON validation.
  Review: Material source precedence is axis-specific: Material Web/spec for Select intent and option richness, Compose Material3 for exposed-dropdown state/semantics/motion, MUI for web defaults, Base UI for headless a11y support, shadcn Select only as Fret-side harness exemplar.
  Evidence: `tools/parity-discovery/fixtures/material3_select_adapter_v1.json`, `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json`
  Handoff: Select has no repair rows after live bundle evidence; remaining rows are hardening/gate promotion work.

- [x] M3PH-050 [owner=planner] [deps=M3PH-030] [scope=tools/parity-discovery/fixtures,tools/diag-scripts/ui-gallery/material3,ecosystem/fret-ui-material3]
  Goal: Add one interaction-heavy Material packet, preferring `Switch`, `Tabs`, or navigation item chrome/motion.
  Validation: fixed-timestep Switch handle diag via `FRET_DIAG_FIXED_FRAME_DELTA_MS=16`, packet regeneration with 20 bundle schema2 files, suite regeneration, and JSON validation.
  Review: Findings distinguish recipe-level Switch chrome/motion from shared Material foundations (`interactive_size`, `indication`, `focus_ring`); no foundation edit was made in this slice.
  Evidence: `tools/parity-discovery/fixtures/material3_switch_adapter_v1.json`, `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`
  Handoff: Switch has no repair rows after live bundle evidence; remaining rows are hardening/gate promotion work.

## M3 - Test Harness Refactor

- [x] M3PH-060 [owner=codex] [deps=M3PH-020] [scope=ecosystem/fret-ui-material3/tests]
  Goal: Extract reusable Material test host, scene snapshot, golden assertion, and interaction helpers out of `radio_alignment.rs`.
  Validation: `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  Review: DONE_WITH_CONCERNS. Support extraction is complete and `cargo test -p fret-ui-material3 --test radio_alignment --no-run` passes. The specified controls golden gate still fails, but the same command also fails in a clean detached `HEAD` worktree before this extraction, so it is recorded as pre-existing Material controls golden drift rather than an extraction regression.
  Evidence: `ecosystem/fret-ui-material3/tests/support/`
  Handoff: Component-specific suite splitting should happen after the Material controls golden drift is either refreshed intentionally or replaced by a narrower preserved-behavior gate.

- [x] M3PH-070 [owner=codex] [deps=M3PH-060] [scope=ecosystem/fret-ui-material3/tests]
  Goal: Split one component family out of `radio_alignment.rs` into a focused test target, starting with field-family or switch/tabs based on packet priority.
  Validation: `cargo nextest run -p fret-ui-material3 --test select_behavior`; `cargo nextest run -p fret-ui-material3 --test radio_alignment chip_set_roving_treats_trailing_action_focus_as_active_chip`; `cargo test -p fret-ui-material3 --test radio_alignment --no-run`.
  Review: DONE. The Select behavior family now has a focused test target without golden regeneration; the old target still compiles and the adjacent chip-set test passes.
  Evidence: `ecosystem/fret-ui-material3/tests/select_behavior.rs`
  Handoff: Repeat this split pattern for another family only after deciding how to track the pre-existing Material controls golden drift.

## M4 - Repair Queues From Evidence

- [x] M3PH-080 [owner=codex] [deps=M3PH-040] [scope=ecosystem/fret-ui-material3,ecosystem/fret-ui-kit,apps/fret-ui-gallery,tools/diag-scripts]
  Goal: Fix the first non-empty Material `repair_queue` row by owner/layer.
  Validation: `python tools/parity-discovery/shadcn_parity_discovery.py --suite tools/parity-discovery/suites/material3_parity_discovery_v1.json --suite-from-existing-reports --suite-output docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`; packet scan confirms 0 `mismatch`/`blocked` rows and no non-empty initial repair queue.
  Review: DONE_NOT_APPLICABLE. The initial Button/Select/Switch suite has 12 `pass_known` parts, 0 `mismatch`, and 0 `blocked`; there is no repair row to fix in this lane.
  Evidence: `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
  Handoff: Future repair work should start from a new component packet or hardening row, not from this closed lane.

- [x] M3PH-090 [owner=codex] [deps=M3PH-040,M3PH-050] [scope=ecosystem/fret-ui-material3,apps/fret-ui-gallery]
  Goal: Document and gate stable Material automation surfaces: root, chrome, indicator/handle, popup/listbox, option/item, and secondary affordance IDs.
  Validation: `cargo test -p fret-ui-material3 --test automation_surface --no-run`; `cargo test -p fret-ui-material3 --features diagnostics --test automation_surface --no-run`; `cargo nextest run -p fret-ui-material3 --features diagnostics --test automation_surface`; source scan for derived Select/Switch part IDs.
  Review: DONE. Selector names are intent-level and stable: Select derives chrome, active indicator, trailing icon, listbox, option chrome, and option icon IDs; Switch derives chrome, track, handle, and icon layer IDs.
  Evidence: `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`
  Handoff: Missing selectors for future Material packets become small component tasks before adding new diagnostics predicates.

## M5 - Verification And Closeout

- [x] M3PH-100 [owner=codex] [deps=M3PH-030,M3PH-040,M3PH-050,M3PH-060,M3PH-070,M3PH-080,M3PH-090] [scope=docs/workstreams/material3-parity-harness-fearless-refactor-v1]
  Goal: Verify lane evidence, update milestones/handoff, and decide whether to close or split follow-ons.
  Validation: fresh targeted gates recorded in `EVIDENCE_AND_GATES.md`; `WORKSTREAM.json` validates; workstream catalog validates.
  Review: DONE. The lane target is satisfied; remaining Material work is split into follow-ons.
  Evidence: `docs/workstreams/material3-parity-harness-fearless-refactor-v1/CLOSEOUT_AUDIT_2026-05-27.md`
  Handoff: Remaining Material component coverage, controls golden drift, and Tabs/navigation packet work should be split by component family.
