# Material 3 Visual Behavior Layout Parity v2 - Design

Status: Active
Last updated: 2026-05-28

## Problem

The Material 3 v1 sweep closed the component matrix and follow-on queue, but it did not claim
shadcn-level style, behavior, and layout parity. The next refactor lane needs a stronger proof
surface: each component should be evaluated by parity axis instead of by a single packet status.

This lane turns the v1 evidence into a v2, axis-driven refactor plan. The target is not to copy
shadcn visuals into Material. The target is to reach the same engineering maturity that the shadcn
crate has: explicit upstream truth, stable parts, deterministic gates, and clear ownership for
style, layout, behavior, accessibility, and motion.

## Refactor Brief

### Intent

Remove the future complexity caused by broad, ambiguous "Material parity" claims. After this lane,
Material 3 work should be able to say which exact axis is aligned, which gate proves it, and which
layer owns the remaining drift.

### Scope

- `ecosystem/fret-ui-material3/src/`
- `ecosystem/fret-ui-material3/src/foundation/`
- `ecosystem/fret-ui-material3/src/interaction/`
- `ecosystem/fret-ui-material3/src/tokens/`
- `ecosystem/fret-ui-material3/tests/`
- `apps/fret-ui-gallery/src/ui/pages/material3/`
- `apps/fret-ui-gallery/src/ui/snippets/material3/`
- `tools/diag-scripts/ui-gallery/material3/`
- `tools/parity-discovery/`
- `goldens/material3-headless/v1/`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/`

### Deletion Plan

Delete or consolidate code only after a v2 axis packet proves the replacement:

- repeated component-local part-id string construction that belongs in `foundation::test_id`,
- component-local token fallback chains that duplicate typed token accessors,
- stale or ambiguous golden fixtures that encode old harness assumptions,
- gallery snippets that hide component drift behind wrapper-only layout,
- broad tests that should move into one-family targets once their gate is stable.

No compatibility shims are protected by this lane unless a public API consumer is documented.

### Boundary Plan

- Material visual defaults and component chrome stay in `ecosystem/fret-ui-material3`.
- Shared Material behavior moves to `foundation` or `interaction` only when at least two component
  packets prove the same need.
- Design-system-agnostic policy moves to `ecosystem/fret-ui-kit` only when shadcn, Material, or
  another recipe crate needs the same policy.
- `crates/*` remain mechanism/contract layers. This lane should not push Material policy into core.
- Caller-owned layout remains caller-owned when upstream applies it through examples, wrappers, or
  page containers.

### Testing Plan

Each axis packet must leave at least one of:

- deterministic scene or geometry assertions for style/layout,
- headless golden coverage for stable visual states,
- focused behavior tests for state machines and keyboard/pointer outcomes,
- diagnostics scripts for overlay, motion, bounds, and gallery-visible flows,
- semantics assertions for accessibility outcomes.

Use `cargo nextest` for Rust gates and keep JSON/catalog gates current.

### Risk Plan

- Risk: treating screenshots as behavior proof.
  - Mitigation: behavior and accessibility require state/semantics gates.
- Risk: overfitting recipe defaults to gallery wrappers.
  - Mitigation: classify layout as intrinsic recipe default or caller-owned before editing.
- Risk: extracting foundations too early.
  - Mitigation: require two consumers before adding shared foundation or kit policy.
- Risk: making v2 too broad to finish.
  - Mitigation: vertical slices by component family and parity axis.

### Workflow Plan

This is a durable `dev-flow` workstream. `M3PV2-010` opens the lane and creates the v2 parity axis
matrix. Follow-up tasks should be executed one bounded slice at a time with `run-workstream-task`,
reviewed, verified, and committed independently.

## Source Of Truth

- UX intent, taxonomy, token names: Material Design 3 spec.
- Toolkit state machines, semantics, touch, motion: Compose Material3.
- Web composition, popups, portal/focus edge cases: MUI Material UI.
- Headless parts and accessibility fallback patterns: Base UI.
- Fret-side layering and test-id exemplar: mature shadcn components, especially Select.

## Non-Goals

- Do not claim full Material pixel parity in this opening task.
- Do not copy shadcn visual styling into Material recipes.
- Do not rewrite every component before the v2 axis matrix identifies priority.
- Do not add core mechanism APIs unless a packet proves ecosystem layers cannot express the result.
