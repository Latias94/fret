# Material 3 SearchView State Packet v1 - Handoff

Status: Closed
Last updated: 2026-05-28

## Current State

This lane is closed.

Fret `SearchView` now keeps Docked as the default and adds `SearchViewPresentation::FullScreen` for
modal expanded search. The full-screen presentation renders an overlay-local search header, focuses
that header input, exposes stable overlay/header part ids, and collapses on Escape through existing
overlay dismissal policy.

## Completed Tasks

- M3SV-010: source packet and layer split.
- M3SV-020: full-screen presentation API and modal overlay.
- M3SV-030: overlay-local focus and stable ids.
- M3SV-040: SearchView golden case for `full_screen_open`.
- M3SV-050: closeout gates and matrix update.

## Closeout Evidence

- `artifacts/search_view_source_packet_v1.md`
- `CLOSEOUT_AUDIT_2026-05-28.md`
- `ecosystem/fret-ui-material3/tests/search_view_behavior.rs`

## Guardrails

- Keep docked behavior as the default.
- Do not add a `crates/*` platform back primitive unless Escape/back-equivalent evidence proves it
  is insufficient.
- Do not implement predictive back gesture progress in this lane.
- Do not duplicate root `test_id`s in full-screen overlay content; use `*.overlay.header*` ids.
