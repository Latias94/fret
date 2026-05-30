# Flex RTL Physical Placement v1 - Handoff

Status: Closed
Last updated: 2026-05-30

## Current State

The lane is closed. The mechanism-layer follow-on from `direction-infrastructure-v1` shipped.

## Target Outcome

- Element construction captures the nearest `LayoutDirection` provider.
- Mount records expose the captured direction to layout code.
- Horizontal Flex rows place children from the right edge under RTL.
- Clean-geometry fast paths do not reuse LTR-only proofs for RTL rows.

## Residuals To Keep Explicit

- Logical margins/insets are not part of this slice.
- Column RTL cross-axis mirroring is not part of this slice.
- Component keyboard/indicator policies remain in ecosystem crates.
