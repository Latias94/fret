# ImUi Collection Command Package v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-collection-command-package-v1` as:

- a closeout record for the landed app-owned collection command package in
  `apps/fret-examples/src/imui_editor_proof_demo/collection.rs`,
- explicit evidence that duplicate-selected plus explicit rename-trigger breadth is coherent enough
  for this folder,
- and a reminder that the default next non-multi-window follow-on is now
  `imui-collection-second-proof-surface-v1`, not a third command verb in this lane.

## What shipped

1. `Primary+D` duplicate-selected now routes through one proof-local command path on the existing
   collection owner surface.
2. The same duplicate command now stays aligned across keyboard, explicit button, and the
   collection context menu.
3. The explicit `Rename active asset` button now reuses the same demo-local rename activation flow
   as `F2` plus the existing context-menu entry.
4. Command status feedback stays app-owned inside `collection.rs` instead of widening a generic
   command bus.
5. Source-policy and workstream docs now freeze the bounded duplicate-plus-rename package and the
   updated next-follow-on order.
6. No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.

## What did not ship

1. No third command verb was added in this folder.
2. No shared `collection_commands(...)` or equivalent helper landed in `fret-ui-kit::imui`.
3. No `fret-imui` facade widening landed.
4. No `crates/fret-ui` runtime/mechanism contract changed.
5. No second proof surface landed inside this lane.

## Reopen policy

Start `imui-collection-second-proof-surface-v1` or a different narrow follow-on only if stronger
first-party evidence shows either:

- a second shell-mounted collection proof now needs to carry the next pressure,
- a later second proof surface finally justifies shared helper growth,
- or broader collection command depth reappears on a materially different owner surface.

Do not reopen this folder just to add one more verb.
Do not treat one proof surface's duplicate-plus-rename package as evidence for shared helper or
runtime widening.
