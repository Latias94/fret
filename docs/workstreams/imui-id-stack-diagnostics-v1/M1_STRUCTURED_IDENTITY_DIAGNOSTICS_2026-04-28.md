# M1 Structured Identity Diagnostics - 2026-04-28

Status: landed

## Summary

The first slice converts the existing debug-only identity warnings into structured diagnostics
records:

- duplicate key-hash detection in `ElementContext::for_each_keyed`,
- unkeyed repeated-subtree reorder detection in `ElementContext::for_each_unkeyed`,
- diagnostics snapshot export through `fret-bootstrap` under `debug.element_runtime.identity_warnings`,
- IMUI callsite preservation through `#[track_caller]` on identity-bearing facade helpers.

The lane stays active because a future interactive ID-stack browser, explicit diagnostics query
command, and `ui.for_each_keyed` duplicate-key authoring proof still deserve separate slices.

## Evidence

- Runtime recorder: `crates/fret-ui/src/elements/cx.rs`
- Snapshot storage: `crates/fret-ui/src/elements/runtime.rs`
- Runtime gates: `crates/fret-ui/src/declarative/tests/identity.rs`
- IMUI callsite proof: `ecosystem/fret-imui/src/tests/identity_diagnostics.rs`
- Diagnostics JSON bridge: `ecosystem/fret-bootstrap/src/ui_diagnostics.rs`

## Gates

- `cargo nextest run -p fret-ui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo nextest run -p fret-imui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo check -p fret-imui --jobs 1`
- `cargo check -p fret-bootstrap --features ui-app-driver --jobs 1`
