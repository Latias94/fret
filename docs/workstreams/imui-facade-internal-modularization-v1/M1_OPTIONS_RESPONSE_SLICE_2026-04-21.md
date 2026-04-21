# ImUi Facade Internal Modularization v1 - M1 Options/Response Slice (2026-04-21)

## Decision

The first landed slice keeps `ecosystem/fret-ui-kit/src/imui/options.rs` and
`ecosystem/fret-ui-kit/src/imui/response.rs` as stable outward hubs, then moves the actual
definitions behind private owner files.

This slice is intentionally structural only:

- no public type names changed,
- no fields changed,
- no default values changed,
- and no response helper method names changed.

## What changed

### `options.rs`

`options.rs` now re-exports smaller private owner files for:

- menus and popup-related options,
- control/disclosure/input options,
- collection/table/virtual-list options,
- container/scroll/child-region options,
- and misc drag/text options.

### `response.rs`

`response.rs` now re-exports smaller private owner files for:

- drag/drop response types,
- hover/query state and `ResponseExt`,
- floating-surface response types,
- and helper-owned widget aggregate responses.

## Why this slice goes first

This was the lowest-risk structural change because `options.rs` and `response.rs` already represent
stable outward vocabularies.
Splitting them first reduces hot-file pressure without touching more coupled runtime bookkeeping.

## Deferred to later milestones

- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui.rs`

Those files still need later owner decomposition, but they remain more coupled and therefore
deserve separate landable slices.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/options/`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/response/`
- `apps/fret-examples/src/lib.rs`
