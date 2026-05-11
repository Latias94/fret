# M3 Test Module First Split

Status: Landed
Date: 2026-05-12

## Decision

The first low-risk feature-owned test module was extracted from the monolithic
`ecosystem/fret-code-editor/src/editor/tests/mod.rs` file:

- `ecosystem/fret-code-editor/src/editor/tests/feature_payloads.rs`

This keeps the shared editor test harness in `mod.rs` while moving the feature payload API tests
behind a focused module boundary.

## Rationale

The public API and architecture lane identified the large editor test file as a refactor hazard.
The feature payload tests are a good first split because they are contiguous, behavior-focused, and
directly tied to the newly gated editor feature payload surface.

This is not a full test-suite modularization. It establishes the pattern for future slices:

- keep common harness helpers in the parent test module until a second module proves extraction is
  useful,
- move feature-specific tests in small groups,
- run focused nextest filters before broader crate gates.

## Evidence

- Test module:
  `ecosystem/fret-code-editor/src/editor/tests/feature_payloads.rs`
- Parent module registration:
  `ecosystem/fret-code-editor/src/editor/tests/mod.rs`

## Gates

```powershell
cargo fmt -p fret-code-editor --check
cargo nextest run -p fret-code-editor feature_payload --no-fail-fast
```
