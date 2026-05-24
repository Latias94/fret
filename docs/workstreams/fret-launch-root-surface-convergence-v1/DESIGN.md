# Fret Launch Root Surface Convergence v1

Status: Active
Last updated: 2026-05-25

## Why This Lane Exists

The previous launch surface lane concluded that `FnDriver` is the preferred advanced example
posture and that direct `WinitAppDriver` usage should be exceptional. This follow-on keeps that
posture from regressing while auditing whether the root exports still teach the right story.

## Target State

- App-facing docs and examples use `fret` or configured `FnDriver` helpers.
- `fret-launch` remains the advanced integration crate.
- Direct `WinitAppDriver` implementations are absent from examples unless explicitly justified.
- Root exports are intentionally curated rather than compatibility-driven.

## First Slice

Run the existing launch posture gates and update the export inventory. If the inventory is already
clean, close this lane as maintenance with the gates as the regression contract.

## Non-goals

- Replacing the desktop runner.
- Adding new launch hooks without an inventory-backed gap.
- Reopening the previous broad launch lane.
