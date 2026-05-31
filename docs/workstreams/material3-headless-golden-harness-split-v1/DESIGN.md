# Material3 Headless Golden Harness Split v1 Design

Status: Closed
Last updated: 2026-05-31

## Problem

`ecosystem/fret-ui-material3/tests/radio_alignment.rs` still hosted broad
`material3_headless_*_suite_goldens_v1` suites after the earlier gate-hygiene lane. That kept the
file shaped as a Material3 golden harness instead of a focused Radio and interaction regression
surface.

The stale navigation and overlay broad goldens were already ignored by default, but their suite
ownership still lived in the Radio test binary.

## Decision

Move all broad Material3 headless golden suites into
`ecosystem/fret-ui-material3/tests/material3_headless_goldens.rs`.

Keep `radio_alignment.rs` focused on Radio-owned geometry/ripple/pressed-scene checks and the
existing interaction regressions that still need separate future ownership decisions.

The moved suites are behavior-preserving:

- function bodies were mechanically moved;
- `scale_segment` moved with the suites because it only serves golden case naming;
- navigation and overlay ignored maintenance semantics were preserved;
- `support::goldens` continues to own snapshot writing/assertion mechanics.

## Non-Goals

- Do not refresh stale navigation or overlay expected payloads in this lane.
- Do not split every non-Radio interaction regression out of `radio_alignment.rs`.
- Do not convert the broad suites to JSON fixtures in this lane.

## Follow-On Shape

Future work can split `material3_headless_goldens.rs` by family or convert large repeated rows to
JSON-backed fixture runners. That should be a separate test-maintenance lane with per-family gates.
