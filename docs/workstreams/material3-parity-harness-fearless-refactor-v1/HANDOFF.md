# Material 3 Parity Harness Fearless Refactor v1 - Handoff

Status: Closed
Last updated: 2026-05-27

## Current State

The workstream is closed. Material implementation code changed only to expose stable recipe-level
automation part IDs for Select and Switch.

The initial audit found:

- `ecosystem/fret-ui-material3` already has broad component, token, foundation, interaction, gallery,
  diag, and headless golden coverage.
- Existing Material docs mark the original foundation/MVP work as complete; the active gap is the
  harness loop, not basic component presence.
- `component-parity-fact-harness-v1` closed with a Material Button adapter pilot and explicitly says
  broader Material coverage should be a new follow-on.
- `radio_alignment.rs` is the main test-structure risk and should be modularized after inventory,
  not rewritten blindly.

M3PH-020 through M3PH-090 are complete. The current artifacts are:

- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_coverage_inventory_v1.json`
- `tools/parity-discovery/suites/material3_parity_discovery_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_parity_suite_report_v1.json`
- `tools/parity-discovery/fixtures/material3_select_adapter_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_select_adapter_report_v1.json`
- `tools/parity-discovery/fixtures/material3_switch_adapter_v1.json`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_switch_adapter_report_v1.json`
- `ecosystem/fret-ui-material3/tests/support/`
- `ecosystem/fret-ui-material3/tests/select_behavior.rs`
- `ecosystem/fret-ui-material3/tests/automation_surface.rs`
- `docs/workstreams/material3-parity-harness-fearless-refactor-v1/artifacts/material3_test_id_contract_v1.md`

Key inventory findings:

- 39 Material component modules are present.
- 25 components have direct gallery snippets.
- 21 components have direct diag script matches.
- 18 components have direct golden-prefix matches, with additional aggregate coverage through
  `material3-controls`, `material3-navigation`, and `material3-overlays`.
- The only current Material parity packet is Button.
- The Material suite baseline summarizes the Button adapter as 1 report, 2 parts, 2 pass-known rows,
  0 repair rows, and 3 hardening rows.
- The Material Select packet adds 5 pass-known parts from live `gallery-material3` bundle evidence,
  with 0 repair rows, 3 hardening rows, and 5 gate rows.
- The Material Switch packet adds 5 pass-known parts from fixed-delta `gallery-material3` bundle
  evidence, with 0 repair rows, 3 hardening rows, and 5 gate rows.
- The suite now summarizes Button, Select, and Switch as 3 reports, 12 pass-known parts, 0 repair
  rows, and 9 hardening rows.
- Recommended next packet is `Tabs` or a navigation active-indicator surface.
- `radio_alignment.rs` had its reusable host/snapshot/golden/event helpers extracted to
  `ecosystem/fret-ui-material3/tests/support/`.
- The Select interaction behavior family has moved to the focused `select_behavior` test target.
- Stable Select/Switch automation surfaces are now documented and gated:
  - Select derives chrome, active indicator, trigger trailing icon, listbox, option chrome, and
    option icon IDs.
  - Switch derives chrome, track, handle, selected icon, and unselected icon IDs.
- The specified M3PH-060 controls golden gate currently fails in both this worktree and a clean
  detached `HEAD` worktree, so the failure is recorded as pre-existing Material controls golden drift.

## Closeout State

- Task ID: M3PH-100
- Owner: codex
- Files:
  - `ecosystem/fret-ui-material3`
  - `ecosystem/fret-ui-kit`
  - `apps/fret-ui-gallery`
  - `tools/diag-scripts`
- Validation:
  - closeout targeted Rust, JSON, parity-suite, workstream-catalog, and whitespace gates passed
- Status: CLOSED
- Review: no blocking findings; M3PH-080 closed as `DONE_NOT_APPLICABLE` because the initial suite
  has no repair rows.
- Evidence: M3PH-020 inventory, M3PH-030 suite baseline, M3PH-040 Select adapter, M3PH-050 Switch
  adapter, M3PH-060 extracted test support, M3PH-070 focused Select test target, and M3PH-090
  automation-surface contract.

## Decisions Since Last Update

- Start a new workstream instead of reopening `component-parity-fact-harness-v1`.
- Keep the first implementation in `tools/parity-discovery`; do not create a new crate yet.
- Use axis-specific Material source precedence:
  - Material spec for intent and taxonomy,
  - Compose Material3 for toolkit state, motion, semantics, and touch behavior,
  - MUI Material UI for web defaults and DOM-facing composition,
  - Base UI for headless accessibility fallback.
- Treat Material policy as `ecosystem/fret-ui-material3` or `ecosystem/fret-ui-kit` work unless a
  packet proves a real `crates/*` mechanism gap.
- M3PH-020 chose `Select` as the first field-family packet, `Switch` as the first
  interaction-heavy packet, and `Tabs` as the next motion/indicator candidate.
- M3PH-030 keeps the suite in `tools/parity-discovery` and uses `--suite-from-existing-reports` for
  the current baseline, because archived report artifacts exist even when local diag sidecars may
  not.
- M3PH-040 used `cargo run -p fret-ui-gallery --features gallery-material3` for Select diagnostics.
  The broader `gallery-full` launch path timed out and exited before `ready.touch` with code 101, so
  do not use it as the current Material packet gate without a separate diag-launch investigation.
- M3PH-050 used `--env FRET_DIAG_FIXED_FRAME_DELTA_MS=16` for the Switch fixed-timestep gate. The
  current `fretboard diag run` CLI does not accept `--fixed-frame-delta-ms` directly.
- M3PH-060 extracted shared Material test support from `radio_alignment.rs` without editing the
  component test bodies. `cargo test -p fret-ui-material3 --test radio_alignment --no-run` passes.
- M3PH-060 did not refresh Material controls goldens. The specified
  `cargo nextest run -p fret-ui-material3 --test radio_alignment material3_headless_controls_suite_goldens_v1`
  gate fails in this worktree and also fails in clean detached `HEAD` at
  `F:\SourceCodes\Rust\fret-worktrees\m3ph060-head-check`, so treat it as pre-existing golden drift.
- M3PH-070 moved eight Select behavior tests into `ecosystem/fret-ui-material3/tests/select_behavior.rs`.
  `cargo nextest run -p fret-ui-material3 --test select_behavior` passed, the adjacent
  `chip_set_roving_treats_trailing_action_focus_as_active_chip` filter still passed in
  `radio_alignment`, and `cargo test -p fret-ui-material3 --test radio_alignment --no-run` passed.
- M3PH-090 made the Material automation-surface contract executable. `SelectPartTestIds`,
  `SelectItemPartTestIds`, and `SwitchPartTestIds` derive stable part selectors in recipe code.
  `ecosystem/fret-ui-material3/tests/automation_surface.rs` verifies the selectors via
  `fret-ui/diagnostics` live test-id matches. Default test-target compilation remains safe because
  the test body is `#[cfg(feature = "diagnostics")]`.

## Blockers

- None for this closed lane.

## Next Recommended Action

Start a new narrow follow-on for any of these:

1. If protecting headless golden gates is the priority, open a narrow task to classify the
   pre-existing `material3-controls.scale1_0.dark.tonal_spot.json` drift before further splits.
2. If packet coverage is the priority, add the next Material packet, likely Tabs or navigation
   active-indicator/motion, and extend `material3_test_id_contract_v1.md` before new diag predicates.
3. If implementation repair is desired, M3PH-080 should first pick a concrete row from a packet
   report; current Select/Switch repair queues are empty, so this likely means promoting a
   hardening/gate row rather than fixing a mismatch.
4. If broader test modularization is desired, split the next family out of `radio_alignment.rs`
   only after deciding how to handle the controls golden drift.
