# Material 3 Test Modularization v1

Status: closed with known follow-ons
Date: 2026-05-27
Task: M3CAS-110

## Scope

Split one low-coupling TopAppBar semantics test out of `radio_alignment.rs` into a dedicated
integration target.

## Decision

This case is not fixture-driven because it is a single procedural semantics smoke test rather than a
repeated matrix. The split keeps the test in Rust and leaves broader golden/matrix extraction for a
later family-by-family pass.

## Changes

- Added `ecosystem/fret-ui-material3/tests/top_app_bar_alignment.rs`.
- Moved `top_app_bar_exposes_toolbar_semantics_role` out of `radio_alignment.rs`.
- Kept `radio_alignment.rs` compiling so future splits can proceed incrementally.

## Proof

- `cargo nextest run -p fret-ui-material3 --test top_app_bar_alignment`
- `cargo test -p fret-ui-material3 --test radio_alignment --no-run`
- `cargo check -p fret-ui-material3 --features diagnostics --tests`
- `cargo clippy -p fret-ui-material3 --features diagnostics --tests --no-deps -- -D warnings`

## Residual Risk

- `radio_alignment.rs` remains large; the next extraction should pick one stable golden family only
  after its stale-golden status is known.
- Broad navigation headless goldens still have unrelated stale geometry drift and should not be
  refreshed as part of test-target plumbing.
