# ImUi Facade Internal Modularization v1 - M4 Facade Writer Glue Slice (2026-04-21)

## Decision

M4 moves the remaining `ImUiFacade` / `UiWriterImUiFacadeExt` writer glue out of the root hub into
one dedicated owner file:

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`

The root `ecosystem/fret-ui-kit/src/imui.rs` file now stays as the stable outward hub plus import
bridge while the full immediate facade writer implementation lives in one explicit owner module.

This slice remains structural only:

- outward type names and helper names stay unchanged,
- sibling modules still construct `ImUiFacade` through the same root path,
- and no interaction behavior or runtime contract is intentionally widened here.

## What changed

- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs` now owns:
  - `ImUiFacade`,
  - its inherent helper methods,
  - the `UiWriter` impl,
  - `UiWriterImUiFacadeExt`,
  - and the blanket impl for all `UiWriter`s.
- `ecosystem/fret-ui-kit/src/imui.rs` now re-exports:
  - `ImUiFacade`,
  - `UiWriterImUiFacadeExt`,
  - and the previously split support/type owners.

Current size snapshot after M4:

- `ecosystem/fret-ui-kit/src/imui.rs`: 125 lines
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`: 1809 lines
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`: 155 lines
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`: 169 lines

## Why this closes the remaining structural hotspot

Before M4, the remaining large file pressure still lived inside the root hub itself.

After M4:

- the root file is now a thin outward hub,
- the large writer surface is isolated as one coherent owner rather than mixed with support/types,
- and future work can target a narrower writer-family or policy lane without reopening a generic
  internal modularization bucket.

## What stays out of scope

- splitting `facade_writer.rs` by specific helper families,
- adding new outward helper surface,
- broadening menu/tab policy, collection semantics, or runtime contracts.

Those should land as a different narrow follow-on if they become necessary.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `docs/workstreams/imui-facade-internal-modularization-v1/EVIDENCE_AND_GATES.md`
