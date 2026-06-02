# Material3 FAB Token Defaults v1 - Handoff

Status: closed

## Closed Outcome

Reduced FAB token-module magic constants by extracting FAB and extended-FAB default matrices into a
private Material3 token helper.

## Closed Scope

- `ecosystem/fret-ui-material3/src/tokens/fab_common.rs`
- `ecosystem/fret-ui-material3/src/tokens/fab.rs`
- `tools/parity-discovery/material3_token_inventory.py`
- `docs/workstreams/material3-fab-token-defaults-v1/`

## Boundaries

- Do not touch `crates/fret-ui` for this lane.
- Do not change FAB public APIs.
- Do not change token values or rendered geometry.
- Preserve the existing token function names consumed by `Fab`.

## Closeout Note

The FAB default helper landed, `fab.rs` now delegates visual default matrices to it, and the lane
inventory artifact was regenerated. See `CLOSEOUT_AUDIT_2026-05-31.md` and
`EVIDENCE_AND_GATES.md` for final evidence.
