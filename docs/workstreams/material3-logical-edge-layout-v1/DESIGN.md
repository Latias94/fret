# Material3 Logical Edge Layout v1

Status: Closed
Last updated: 2026-05-30

## Problem

Material3 direction is now bridged into the core `LayoutDirection` provider, so horizontal Flex rows
can mirror physical order under RTL. Some Material3 recipes still encode logical inline spacing with
physical `left` / `right` padding or insets. Once rows mirror, those physical edges can become
wrong: inline-start padding remains on the left, or an inline-end absolute overlay stays pinned to
the right under RTL.

## Scope

- Add a Material3 foundation helper for converting logical inline edges into physical `Edges`.
- Add a helper for assigning an inline-end absolute inset.
- Migrate FilterChip and InputChip content padding and trailing action overlay insets.
- Add a diagnostics geometry test proving RTL chip content mirrors.

## Non-Goals

- Do not migrate Select/TextField inline field insets in this lane.
- Do not claim a full RTL visual sweep across Material3.
- Do not add logical edges to `fret-core` until multiple design systems prove the generic contract
  shape.

## Assumptions

- Area: ownership
  - Assumption: logical edge conversion is currently Material foundation policy, not a new core
    primitive.
  - Evidence: `docs/adr/0066-fret-ui-runtime-contract-surface.md`,
    `ecosystem/fret-ui-material3/src/foundation/`.
  - Confidence: Likely.
  - Consequence if wrong: the helper may need to move into `fret-ui-kit` or `fret-core` after more
    consumers adopt it.

- Area: first consumer
  - Assumption: FilterChip and InputChip are the right first proof because they already carry
    `LayoutDirection`, have asymmetric leading/trailing padding, and have an absolute trailing
    action target.
  - Evidence: `ecosystem/fret-ui-material3/src/filter_chip.rs`,
    `ecosystem/fret-ui-material3/src/input_chip.rs`.
  - Confidence: Confident.
  - Consequence if wrong: the foundation helper still lands, but the first consumer may not cover
    the highest-risk field-family drift.

## Parity Proof Note

- Truth: Under RTL, chip leading content is physically right of the label and trailing action is
  physically left of the label.
- Artifacts: `foundation::logical_edges`, FilterChip/InputChip adoption, chip diagnostics test.
- Wiring: FilterChip/InputChip content rows provide resolved layout direction and use logical
  padding/inset helpers.
- Proof: A chip diagnostics test compares RTL icon/label/trailing-action visual bounds.
- Residual risk: Field-family text/label inline insets remain a separate follow-on.
