# ImUi Models Text Mode Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-05-04

This lane records the next small split of the growing `fret-imui` text-model test surface. It
follows the picker and filter splits by isolating single-line text mode policy tests into their own
module.

## Ownership

- `fret-imui` owns the proof tests and test module registration.
- `fret-ui-kit::imui` continues to own `InputTextOptions` and `InputTextMode` policy shape.
- Runtime text editing, public authoring APIs, and editor controls are not in scope.

## Must-Be-True Outcomes

- Read-only, select-all-on-focus, and password-mode tests live in `models_text_modes.rs`.
- `models_text.rs` no longer carries the moved mode-specific test bodies or the `InputTextMode`
  import.
- `tests/mod.rs` registers `models_text_modes`.
- Focused mode tests and the broader `models_text` filter both pass.

## Fixture Decision

These tests remain Rust interaction tests because they depend on focus dispatch, delayed timer
dispatch, command availability, and paint text capture. They are not a simple data matrix.

## Non-Goals

- No change to `InputTextOptions`.
- No change to `InputTextMode`.
- No change to password obscuring behavior.
- No decomposition of command-policy, lifecycle, or textarea tests yet.
