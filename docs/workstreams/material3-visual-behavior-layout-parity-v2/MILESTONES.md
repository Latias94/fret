# Material 3 Visual Behavior Layout Parity v2 - Milestones

Status: Active
Last updated: 2026-05-29

## M0 - Lane Opened

Exit criteria:

- Workstream docs exist.
- `material3_parity_axis_matrix_v2.json` covers all 39 v1 components.
- Workstream catalog and JSON gates pass.

## M1 - High-Risk Axes Prioritized

Exit criteria:

- Field, navigation, choice-control, overlay, and low-interaction families each have selected v2
  packet candidates.
- Each selected row states which source axis leads: Material spec, Compose, MUI, Base UI, or
  Fret-side shadcn exemplar.
- Layout candidates are classified as recipe default or caller-owned before code edits.

## M2 - Component Families Gated

Exit criteria:

- Each high-priority family has at least one v2 gate that proves a style, layout, behavior,
  accessibility, or motion truth.
- Shared foundation refactors have at least two consumer proofs.
- No Material policy leaks into `crates/*`.

## M3 - Harness Simplified

Exit criteria:

- Redundant broad tests or stale goldens are deleted or documented as historical artifacts.
- New tests are grouped by stable component family.
- Diagnostics scripts use stable intent-level selectors.

## M4 - Closeout

Exit criteria:

- The v2 matrix has no missing axis states or first gates.
- Remaining work is split into narrow follow-ons with source-backed truth and gates.
- Closeout audit records commands, residual risk, and layer decisions.
