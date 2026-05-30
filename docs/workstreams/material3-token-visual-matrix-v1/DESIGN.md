# Material3 Token Visual Matrix v1 - Design

Status: Active
Last updated: 2026-05-30

## Problem

The Material3 v2 component-axis lane closed current component style/layout/behavior/a11y/motion
gaps, but it intentionally did not claim exhaustive token visual parity. Many components now have
good focused gates, yet token ownership is still uneven: some values are typed accessors, some are
fallback chains, some are recipe-local constants, and some visual states are proven only by one
representative scene.

This lane turns Material3 styling into an explicit token visual matrix:

`component x variant x state x scheme x part x token role`

The target is not pixel-perfect screenshot sprawl. The target is a maintainable source map and
fixture-driven proof system that can say which Material token drives each visible outcome and which
gate proves the rendered result.

## Target State

- A machine-readable matrix records every supported Material3 component, visual variant, state,
  token role, owner layer, and gate state.
- Component token access is localized behind typed token outcome helpers where practical.
- Recipe-local fallback chains and magic visual constants are either deleted or explicitly recorded
  as intentional local policy.
- A generated inventory report keeps token-module, v30 alias, fallback-chain, and visual-constant
  counts reproducible before each family packet changes code.
- Scene assertions prove high-value token outcomes directly: color, alpha, shape radius, elevation,
  outline width, active indicator geometry, typography role, and motion/easing channel.
- Golden coverage remains representative; exhaustive token correctness is handled by fixtures and
  deterministic scene/token assertions.

## Scope

- `ecosystem/fret-ui-material3/src/tokens/`
- `ecosystem/fret-ui-material3/src/foundation/`
- `ecosystem/fret-ui-material3/src/interaction/`
- `ecosystem/fret-ui-material3/src/*.rs` component recipes
- `ecosystem/fret-ui-material3/tests/`
- `goldens/material3-headless/v1/`
- `tools/parity-discovery/`
- `docs/workstreams/material3-token-visual-matrix-v1/`

## Non-Goals

- Do not reopen the closed M3PV2 component-axis lane.
- Do not copy shadcn visuals into Material recipes.
- Do not require a screenshot/golden for every matrix row.
- Do not push Material-specific token policy into `crates/*`.
- Do not chase unsupported future API breadth such as adaptive NavigationSuite, rich tooltip
  actions, or full carousel containers in this lane.

## Source Precedence

- Token inventory and generated values: Material Web v30 snapshot currently injected by
  `ecosystem/fret-ui-material3/src/tokens/material_web_v30.rs`.
- Dynamic color and scheme axes: `tokens::v30::{SchemeMode, DynamicVariant}`.
- Toolkit behavior and state naming: Compose Material3 local reference under
  `repo-ref/compose-multiplatform-core`.
- Accessibility and headless part sanity checks: Base UI local reference under `repo-ref/base-ui`.
- Fret boundary and evidence discipline: M3PV2 closeout audit and existing Material3 tests.

## Architecture Direction

- `tokens/*` should expose typed token outcomes by component/variant/state.
- `foundation/*` should own reusable visual mechanisms: state layers, ripple, elevation, shape,
  active indicators, field geometry, modal/overlay motion, and interactive sizing.
- Component recipes should consume token outcomes and compose parts; they should not duplicate token
  fallback trees.
- Tests should move token truth toward fixture-driven runners that can be expanded without adding
  large bespoke Rust tests per row.

## Deletion Plan

Delete only after replacement gates exist:

- duplicate recipe-local token fallback chains,
- magic color/size/elevation constants that have token equivalents,
- stale matrix/golden rows that encode old harness assumptions,
- broad helper copies once fixture-driven runners cover the same invariant.

## Risk Plan

- Risk: exhaustive matrix scope becomes too large.
  - Mitigation: close one family packet at a time and keep unsupported API breadth out of scope.
- Risk: Material Web and Compose token names disagree.
  - Mitigation: record axis-specific source precedence per row and keep conflicts in the matrix.
- Risk: golden count grows too fast.
  - Mitigation: use scene/token assertions for exact values and reserve goldens for representative
    visual signatures.
- Risk: token refactors break existing component gates.
  - Mitigation: run the existing M3PV2 gate set after each family packet.
