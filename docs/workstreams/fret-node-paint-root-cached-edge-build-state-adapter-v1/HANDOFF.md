# Fret Node Paint Root Cached Edge Build State Adapter v1 - Handoff

Status: Closed
Last updated: 2026-05-25

## Current State

This lane is closed. Cached edge build-state host/services/scale route inputs now go through
`build_state_adapter.rs` and `build_state_retained_cx.rs`.

## Scope Guard

Do not include edge-label build-state in this lane. `labels/single.rs` and `labels/tiled.rs` have
similar retained route-input reads, but they remain a dedicated follow-on.

## Closeout Evidence

See `CLOSEOUT_AUDIT_2026-05-25.md` and `EVIDENCE_AND_GATES.md`.

## Follow-Ons

- cached edge-label build-state host/services/scale route-input adapter,
- cache-local temporary scene construction,
- cache-local clip-op construction,
- overlay routing.
