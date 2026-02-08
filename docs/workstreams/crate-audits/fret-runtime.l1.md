# Crate Audit (L1) — `fret-runtime`

Status: L1 complete (targeted deep dive + one new regression gate)

Supersedes: `docs/workstreams/crate-audits/fret-runtime.l0.md` (keep for the initial snapshot)

## Purpose

Portable runtime contracts: models, effects, commands/keymap/menubar, window services and capability signals.

## Audit focus (L1)

- Wire format stability for user-authored config surfaces:
  - `keymap.json` (ADR 0043 related: sequences/continuations),
  - menubar config (v1/v2, replace vs patch).
- Ensure failures are structured and consistent (avoid “stringly” regressions).

## What changed (evidence-backed)

- Added fixture-driven decoding gates for keymap + menubar.
  - Evidence:
    - `crates/fret-runtime/tests/wire_fixtures.rs`
    - `crates/fret-runtime/tests/fixtures/keymap/v1-basic.json`
    - `crates/fret-runtime/tests/fixtures/keymap/v2-sequence.json`
    - `crates/fret-runtime/tests/fixtures/keymap/v2-empty-keys.json`
    - `crates/fret-runtime/tests/fixtures/menubar/v2-replace.json`
    - `crates/fret-runtime/tests/fixtures/menubar/v2-patch.json`
    - `crates/fret-runtime/tests/fixtures/menubar/v2-invalid-both.json`

- Added regression gates for `ModelStore` re-entrancy/borrow discipline during `update` and `update_any`.
  - Evidence:
    - `crates/fret-runtime/src/model/store.rs` (`update_does_not_hold_store_lock_while_running_user_code`)
    - `crates/fret-runtime/src/model/store.rs` (`update_any_does_not_hold_store_lock_while_running_user_code`)

## Hazards (top)

- Accidental wire format drift (field rename/default behavior changes) breaking user configs.
  - Existing gates:
    - `crates/fret-runtime/tests/wire_fixtures.rs` (fixture decoding)
    - `crates/fret-runtime/src/keymap/tests.rs` (when validation + resolution behavior)
    - `crates/fret-runtime/src/menu/tests.rs` (patch semantics)
- `WhenExpr` identifier vocabulary drift breaking gating.
  - Existing gates:
    - `crates/fret-runtime/src/when_expr/tests.rs` (`when_expr_identifier_contract_matches_capability_tables`)
    - `crates/fret-runtime/src/when_expr/tests.rs` (`when_expr_identifier_contract_covers_builtin_identifiers`)

## Recommended next steps

1. Extend fixture set with version-mismatch cases (explicit failure modes) for keymap + menubar.
2. Add a dedicated “identifier contract” test for `WhenExpr` to prevent silent additions/removals.
3. Continue L1 on `model::store` hot path: lease invariants + re-entrancy hazards.
