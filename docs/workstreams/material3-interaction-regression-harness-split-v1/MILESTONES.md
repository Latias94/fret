# Material3 Interaction Regression Harness Split v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Residual Ownership Classified

The remaining non-Radio tests in `radio_alignment.rs` were classified as historical Material3
interaction regressions.

## M2: Radio Binary Focused

`radio_alignment.rs` now contains only the three Radio-owned tests.

## M3: Interaction Binary Added

`material3_interaction_regressions.rs` owns the moved 48 historical interaction regressions.

## M4: Lane Closed

Focused nextest gates, package check/clippy, catalog, layering, and diff hygiene passed before
commit.
