# Closeout Audit — 2026-04-09

Lane: `bootstrap-known-startup-failure-taxonomy-v1`
Status: Closed

## Shipped verdict

This lane is closed.

The shipped outcome is:

- `fret-bootstrap` now owns one broader known startup failure taxonomy,
- returned bootstrap startup failures and panic-only explicit icon install failures now map into
  one `BootstrapKnownFailureReport`,
- `fret::Error` now exposes one bridge method for that taxonomy,
- bootstrap diagnostics now log one bootstrap-level field family,
- and the `fret` root direct re-export budget remains unchanged.

## What landed

### 1) Shared bootstrap taxonomy

Landed in `ecosystem/fret-bootstrap/src/lib.rs`:

- `BootstrapKnownFailureStage`
- `BootstrapKnownFailureKind`
- `BootstrapKnownFailureReport`
- `BootstrapError::known_failure_report()`

### 2) Returned-error coverage

The shipped mappings cover:

- settings read/parse
- keymap read/parse
- menu bar read/parse
- asset manifest read/parse/serialize/write/bundle-root/invalid/duplicate-key
- asset startup missing-lane failures

### 3) Panic-only icon coverage

The existing icon panic-time report now maps into the same bootstrap taxonomy, so the diagnostics
surface no longer treats icon failures as a special one-off schema.

### 4) App-facing facade bridge

`ecosystem/fret/src/lib.rs` now exposes:

- `Error::known_bootstrap_failure_report()`

This closes the previous gap where `fret::Error` had split asset failures away from
`BootstrapError`.

### 5) Diagnostics field unification

`init_panic_hook()` now logs bootstrap-level fields rather than icon-only field names.

## What intentionally did not land

- no startup recovery UI,
- no persistent diagnostics bundle for startup failures,
- no broader `Result` plumbing for setup/bootstrap/install seams,
- no new root-level `fret` direct re-exports.

## Follow-on rule

Keep this lane closed unless fresh evidence names a new narrower problem such as:

- startup recovery UI,
- richer persistent startup diagnostics artifacts,
- or a new facade helper that still respects the root-surface policy budget.

Do not reopen this lane for broad lifecycle redesign.

## Gate summary

Passed on 2026-04-09:

```bash
cargo nextest run -p fret-bootstrap
cargo nextest run -p fret --lib
cargo check -p fret-bootstrap --features diagnostics
python3 tools/check_layering.py
git diff --check
```
