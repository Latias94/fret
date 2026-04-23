# ImUi Editor Proof Collection Modularization v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-editor-proof-collection-modularization-v1` as:

- a closeout record for the landed demo-local collection module slice in
  `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`,
- an explicit reminder that the closed inline-rename lane stays closed,
- and a reminder that the next default non-multi-window follow-on is now broader app-owned
  collection command-package depth rather than more host-file accretion.

## What shipped

1. Collection assets, collection-local models, render logic, and unit tests moved into `collection.rs`.
2. `imui_editor_proof_demo.rs` now keeps the collection boundary explicit with `mod collection;`, one render call, and drag-asset delegation.
3. The extracted render entry stays explicitly `KernelApp`-bound because the proof still depends on default app-surface selectors/actions; modularization did not justify a new generic IMUI seam.
4. Collection surface tests now point at the module, and a dedicated modularization surface test freezes the host/module split.
5. Source-policy now records the lane and the updated next-priority order.
6. No public IMUI/runtime surface changed.

## What did not ship

1. No new shared collection helper package in `fret-ui-kit::imui`.
2. No `fret-imui` facade widening.
3. No `crates/fret-ui` runtime contract change.
4. No broader collection command package.
5. No second proof surface.

## Reopen policy

Start a different narrower follow-on only if stronger first-party proof shows either:

- the remaining gap is primarily broader app-owned collection command breadth,
- the same pressure now appears on a second real proof surface,
- or new evidence finally justifies shared helper growth beyond one proof surface.

No reopening of the closed inline-rename lane or premature shared-helper growth from one proof surface.
