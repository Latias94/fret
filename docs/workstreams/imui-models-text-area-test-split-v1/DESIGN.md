# ImUi Models Text Area Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-05-04

This lane records the next small split of the growing `fret-imui` text-model test surface. It
follows the picker, filter, mode, and command splits by isolating multiline `textarea` tests into
their own module.

## Ownership

- `fret-imui` owns the proof tests and test module registration.
- `fret-ui-kit::imui` continues to own `TextAreaOptions` policy shape.
- Runtime text editing, public authoring APIs, and single-line input policy are not in scope.

## Must-Be-True Outcomes

- Textarea read-only, Tab policy, changed-signal, and lifecycle tests live in `models_text_area.rs`.
- `models_text.rs` no longer carries the moved multiline test bodies or the `TextAreaOptions`
  import.
- `tests/mod.rs` registers `models_text_area`.
- Focused textarea tests and the broader `models_text` filter both pass.

## Fixture Decision

These tests remain Rust interaction tests because they drive focus, keyboard input, multiline text
events, model state, and lifecycle flags through the real IMUI host.

## Non-Goals

- No change to `TextAreaOptions`.
- No change to multiline input behavior.
- No new fixture schema.
- No decomposition of remaining single-line lifecycle, bounds, or push-id tests yet.
