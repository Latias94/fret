# ImUi Facade Internal Modularization v1 - M3 Root Facade Hub Slice (2026-04-21)

## Decision

M3 keeps `ecosystem/fret-ui-kit/src/imui.rs` as the stable outward hub while moving two mixed
owners out of the root file:

- facade-local support helpers now live in `ecosystem/fret-ui-kit/src/imui/facade_support.rs`,
- floating window / area public surface types now live in
  `ecosystem/fret-ui-kit/src/imui/floating_options.rs`.

This slice remains structural only:

- outward helper names stay unchanged,
- sibling modules still read the same root `super::...` names,
- and no interaction behavior or runtime contract is intentionally widened here.

## What changed

`ecosystem/fret-ui-kit/src/imui.rs` now re-imports smaller owner files for support helpers and
floating facade types instead of co-owning them directly:

- `facade_support.rs` owns `UiWriterUiKitExt`, transient-event keys, slider math, snap helpers,
  and other facade-local support functions/constants.
- `floating_options.rs` owns `FloatingWindowResizeOptions`, `FloatingWindowOptions`,
  `WindowOptions`, `FloatingAreaOptions`, and `FloatingAreaContext`.
- `imui.rs` keeps the public surface stable by importing those owners back into the root namespace
  instead of forcing sibling modules to adopt new paths.

Current size snapshot after M3:

- `ecosystem/fret-ui-kit/src/imui.rs`: 1927 lines
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`: 155 lines
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`: 169 lines

## Why this closes M3

Before M3, the root file still mixed:

- module hub / re-export wiring,
- public floating facade surface types,
- facade-local support helpers,
- and the large `ImUiFacade` / `UiWriterImUiFacadeExt` writer glue.

After M3:

- the root hub no longer co-owns support utilities and floating-surface types,
- the new owner files make those surfaces reviewable without opening the full facade writer glue,
- and future structural work can focus on `ImUiFacade` / `UiWriterImUiFacadeExt` specifically.

## Deferred to M4

- deeper `ImUiFacade` / `UiWriterImUiFacadeExt` owner decomposition if the remaining root file
  still proves too large to review comfortably,
- or explicit lane closeout if the current owner split is already sufficient for near-term parity
  work.

## Evidence anchors

- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `docs/workstreams/imui-facade-internal-modularization-v1/EVIDENCE_AND_GATES.md`
