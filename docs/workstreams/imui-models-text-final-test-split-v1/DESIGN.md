# ImUi Models Text Final Test Split v1

Status: Closed narrow test-architecture follow-on
Last updated: 2026-05-04

This lane closes the mechanical decomposition of the legacy `fret-imui` `models_text.rs` aggregate
test file. The remaining tests were small but still mixed basic changed-signal, lifecycle/bounds,
and identity stability concerns in one file.

## Ownership

- `fret-imui` owns the proof tests and test module registration.
- `fret-ui-kit::imui` continues to own text control policy behavior.
- Runtime text editing, public authoring APIs, and identity contracts are not changed.

## Must-Be-True Outcomes

- Basic single-line changed-signal coverage lives in `models_text_basic.rs`.
- Single-line focus bounds and lifecycle coverage lives in `models_text_lifecycle.rs`.
- Push-id reorder stability coverage lives in `models_text_identity.rs`.
- `models_text.rs` is deleted instead of kept as an empty aggregate.
- `tests/mod.rs` registers the new modules.
- The broad `models_text` nextest filter still runs all text-model tests through module-name
  matching.

## Fixture Decision

These tests remain Rust interaction tests because they drive model state, focus, bounds inspection,
text input, and identity stack behavior through the real IMUI host.

## Non-Goals

- No behavior or public API changes.
- No fixture schema.
- No changes to runtime text editing internals.
- No changes to identity hashing or push-id semantics.
