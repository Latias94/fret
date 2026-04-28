# M3 IMUI Keyed Duplicate Proof - 2026-04-28

Status: landed

## Scope

This slice closes the IMUI authoring proof for duplicate keyed-list identity warnings.

`ImUi::for_each_keyed` now delegates to `ElementContext::for_each_keyed` instead of manually
looping through `ui.id(...)`. That makes the IMUI keyed-list helper use the same runtime list scope,
callsite tracking, duplicate-key detection, and structured diagnostics recorder as declarative UI.

## API Shape

The helper now mirrors the runtime keyed-list shape:

```rust
ui.for_each_keyed(items, |item| key, |ui, index, item| {
    // build row
});
```

Explicit subtree identity remains available through `ui.id(...)` and `ui.push_id(...)`.

## Evidence

- `ecosystem/fret-imui/src/frontend.rs`
- `ecosystem/fret-imui/src/tests/identity_diagnostics.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `ecosystem/fret-ui-kit/src/imui/combo_controls.rs` (drive-by clippy cleanup surfaced by the
  narrowed `fret-imui` gate)
- `ecosystem/fret-ui-kit/src/imui/control_chrome.rs` (drive-by clippy cleanup)
- `ecosystem/fret-ui-kit/src/imui/options/controls.rs` (drive-by clippy cleanup)
- `ecosystem/fret-ui-kit/src/imui/tab_family_controls.rs` (drive-by clippy cleanup)

## Gates

- `cargo nextest run -p fret-imui --features diagnostics identity_diagnostics --no-fail-fast`
- `cargo check -p fret-imui --jobs 1`
- `cargo clippy -p fret-imui --all-targets --features diagnostics -- -D warnings`
- `cargo fmt --package fret-imui --package fret-ui-kit --check`

## Follow-Ons Still Deferred

- full interactive ID-stack browser,
- label-to-`test_id` inference,
- table column identity.
