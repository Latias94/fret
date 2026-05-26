# Fret Node Paint Root Cached Edge Key Helper v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Scope Freeze

- The lane owns cached edge key-field helper ownership only.
- Cache invalidation, cache lifetime, scope string changes, and route adapters are explicitly out of
  scope.
- `WORKSTREAM.json` validates as JSON.

## M1 - Key Helper

- A shared cached edge key helper exists in `keys.rs`.
- Four existing key functions preserve names, inputs, and scope strings.
- Single-rect rect-origin fields remain outside the shared helper.
- Focused source-policy coverage locks the seam.

## M2 - Verification And Closeout

- `cargo fmt --package fret-node` passes.
- Focused source-policy test passes under `compat-retained-canvas`.
- `cargo check -p fret-node` passes.
- `cargo check -p fret-node --features compat-retained-canvas` passes.
- `python3 tools/check_workstream_catalog.py` passes.
- `python3 tools/check_layering.py` passes.
- `git diff --check` passes.
- A closeout audit records shipped state and residual follow-ons.

Result (2026-05-25): complete.
