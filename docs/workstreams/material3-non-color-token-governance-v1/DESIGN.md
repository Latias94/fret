# Material3 Non-Color Token Governance v1 Design

Status: Active
Last updated: 2026-05-31

## Problem

The closed `material3-token-resolver-non-field-fallback-v1` lane removed repeated color fallback
chains from non-generated Material3 token modules. A smaller set of direct non-color token reads
remains: typography weight, state-layer opacity, motion easing, and time picker/input numeric
fallbacks.

Those reads are not all the same kind of problem. Some should move behind shared Material
foundation helpers; some are legitimate component-local scalar defaults; some need a narrower
field/time-picker follow-on.

## Target State

- Repeated non-color fallback chains use shared Material foundation vocabulary.
- Typography weight overrides are centralized in `tokens::typography` rather than repeated in
  component token modules.
- State-layer opacity reads use `MaterialTokenResolver` where component-to-system fallback exists.
- Motion/easing and time picker/input numeric reads are either migrated through reusable helpers or
  documented as follow-on scope with a clear owner.
- The workstream leaves a reusable audit command and focused gates.

## Scope

- `ecosystem/fret-ui-material3/src/foundation/token_resolver.rs`
- `ecosystem/fret-ui-material3/src/tokens/typography.rs`
- Component token modules with direct non-color reads:
  - chip family and Slider label typography weight
  - Radio disabled icon opacity
  - Dialog/Snackbar/ModalNavigationDrawer motion easing
  - TimeInput/TimePicker numeric fallback chains
- Workstream docs and catalog state.

## Non-Goals

- Do not migrate generated token data in `v30.rs` or `material_web_v30.rs`.
- Do not move Material-specific policy into `crates/*`.
- Do not replace every `metric_by_key(...).unwrap_or(...)` scalar read. Component-local metric
  defaults remain acceptable unless they form a repeated component-to-system fallback chain.
- Do not reopen the closed color fallback lanes.

## Architecture Direction

Use the same boundary rule as the color fallback lanes:

- `MaterialTokenResolver` owns repeated component/system token fallback vocabulary.
- `tokens::typography` owns typography style and weight normalization.
- Component token modules own local key selection, state/variant branching, and scalar defaults that
  are intrinsic to one component.
- Field/time-picker-specific numeric policy should become a narrow follow-on if the repeated paths
  are too component-specific for this lane.

## Truth / Artifacts / Wiring / Proof

Truth:

- Chip-family and Slider label weight overrides are no longer hand-coded in each component token
  module.
- Radio disabled icon opacity uses the same resolver number vocabulary as other selection controls.
- The residual audit can distinguish intentional scalar metric reads from fallback-chain debt.
- Any unmerged time picker/input numeric policy is explicitly classified and not hidden.

Artifacts:

- Updated token modules and/or foundation helpers.
- `TODO.md` task ledger with residual classifications.
- `EVIDENCE_AND_GATES.md` with audit commands and targeted tests.

Wiring:

- Existing component recipes continue to call their token modules.
- Token visual fixtures and targeted state tests prove behavior preservation.

Proof:

- `cargo nextest run -p fret-ui-material3 --lib material3_token_visual_fixtures_match_expected_token_outcomes`
- Targeted tests for touched families.
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`
- Residual direct-read audits recorded in `EVIDENCE_AND_GATES.md`.

Residual risk:

- Time picker/input may need a separate field-family token governance lane if their numeric
  fallbacks are coupled to picker-specific layout semantics.
