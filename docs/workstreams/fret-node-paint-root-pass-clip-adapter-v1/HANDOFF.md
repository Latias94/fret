# Fret Node Paint Root Pass Clip Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is closed.

The authoritative design narrows the candidate from broad cached/immediate pass clip emission to
immediate pass static scene routing because `paint_root/cached_pass.rs` has no direct pass-router
`cx.scene` access.

## Shipped Action

Implemented `pass_scene_adapter.rs` and `pass_scene_retained_cx.rs`, updated
`paint_root/immediate_pass.rs`, and added source-policy coverage in `ecosystem/fret-node/src/lib.rs`.

## Validation

Run:

```bash
cargo test -p fret-node --features compat-retained-canvas paint_root_pass_scene_adapter
```

The full gate set in `EVIDENCE_AND_GATES.md` passed before closeout.

## Residual Follow-ons

- cached static group/node layer scene replay adapter,
- cached edge scene replay adapter,
- immediate edge/overlay pass routing adapter,
- grid plan or chrome hint routing adapter.
