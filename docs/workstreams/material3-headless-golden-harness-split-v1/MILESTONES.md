# Material3 Headless Golden Harness Split v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Suites Classified

The `material3_headless_*_suite_goldens_v1` tests were classified as broad Material3 headless
golden suites, not Radio-owned tests.

## M2: Test Binary Split

All broad headless golden suites moved from `radio_alignment.rs` into
`material3_headless_goldens.rs`.

## M3: Default Gates Re-Proved

`radio_alignment` runs as a focused default binary, while `material3_headless_goldens` owns the
broad golden suite run and preserves ignored stale maintenance entries.

## M4: Lane Closed

Catalog, workstream state, formatting, focused nextest gates, check/clippy, layering, and diff
hygiene were verified before commit.
