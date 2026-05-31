# Material3 Headless Golden Harness Split v1 Handoff

Status: Closed
Last updated: 2026-05-31

## What Changed

- Added `ecosystem/fret-ui-material3/tests/material3_headless_goldens.rs`.
- Moved 21 `material3_headless_*_suite_goldens_v1` tests out of `radio_alignment.rs`.
- Moved `scale_segment` with the headless suites.
- Preserved ignored maintenance semantics for stale navigation and overlay broad goldens.

## What Remains

- `radio_alignment.rs` still contains several non-Radio interaction regressions. They are no
  longer broad headless golden suites, but they may deserve future purpose-owned files.
- `material3_headless_goldens.rs` is intentionally a single owner file for this lane. It can be
  split by family or converted to JSON fixture runners later.
- Stale navigation and overlay broad golden payloads still need dedicated refresh lanes.

## Suggested Follow-Ons

- `material3-headless-navigation-golden-refresh-v1`
- `material3-headless-overlay-golden-refresh-v1`
- `material3-interaction-regression-harness-split-v1`
