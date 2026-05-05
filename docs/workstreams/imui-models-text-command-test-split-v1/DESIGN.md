# ImUi Models Text Command Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-05-04

This lane records the next small split of the growing `fret-imui` text-model test surface. It
follows the picker, filter, and mode splits by isolating single-line command-policy tests into
their own module.

## Ownership

- `fret-imui` owns the proof tests and test module registration.
- `fret-ui-kit::imui` continues to own command-oriented `InputTextOptions` policy shape.
- Runtime command dispatch, public authoring APIs, and editor command stacks are not in scope.

## Must-Be-True Outcomes

- Completion, history, undo/redo, and repeat opt-in command-policy tests live in
  `models_text_commands.rs`.
- `models_text.rs` no longer carries the moved command-specific test bodies.
- `tests/mod.rs` registers `models_text_commands`.
- Focused command tests and the broader `models_text` filter both pass.

## Fixture Decision

These tests remain Rust interaction tests because they drive focus, key events, repeat filtering,
and command-effect collection through the real IMUI host. They are not a pure data fixture matrix.

## Non-Goals

- No change to `InputTextOptions`.
- No change to command IDs or app-owned command routing policy.
- No runtime undo-stack ownership.
- No decomposition of textarea or lifecycle tests yet.
