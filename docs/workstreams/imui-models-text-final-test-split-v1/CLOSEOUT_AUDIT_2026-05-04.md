# ImUi Models Text Final Test Split v1 Closeout Audit - 2026-05-04

Status: Closed.

This lane closes the mechanical `models_text.rs` decomposition by retiring the legacy aggregate
file after moving its remaining coverage into capability modules.

## What Shipped

- Added `ecosystem/fret-imui/src/tests/models_text_basic.rs`.
- Added `ecosystem/fret-imui/src/tests/models_text_lifecycle.rs`.
- Added `ecosystem/fret-imui/src/tests/models_text_identity.rs`.
- Registered the new modules from `ecosystem/fret-imui/src/tests/mod.rs`.
- Deleted `ecosystem/fret-imui/src/tests/models_text.rs`.

## Proof

- `cargo nextest run -p fret-imui models_text --no-fail-fast` still runs 26 text-model tests and
  passes across picker, filter, mode, command, textarea, basic, lifecycle, and identity modules.

## Remaining Work

- Future text-model coverage should be added to the specific capability module.
- Reintroducing a generic `models_text.rs` aggregate should require a new reason, not inertia.
