# ImUi Collection Second Proof Surface v1 - Closeout Audit

Status: closed closeout record
Date: 2026-04-23

## Verdict

Treat `imui-collection-second-proof-surface-v1` as:

- a closeout record for the landed shell-mounted `Scene collection` proof surface in
  `apps/fret-examples/src/editor_notes_demo.rs`,
- explicit evidence that the second collection proof surface now exists outside
  `imui_editor_proof_demo`,
- and a no-helper-widening verdict for this cycle because the second surface does not yet show that
  both proof surfaces need the same shared collection helper.

## What shipped

1. `editor_notes_demo.rs` now owns a smaller shell-mounted collection surface in the left
   `WorkspaceFrame` rail.
2. The new `Scene collection` surface keeps stable collection root, summary, and list test ids.
3. Collection rows remain app-owned labels over the existing `SelectMaterial`, `SelectLight`, and
   `SelectCamera` actions.
4. `workspace_shell_demo.rs` remains supporting shell-mounted evidence rather than the only second
   proof answer.
5. Source-policy and surface tests keep the second proof surface visible.
6. No public `fret-imui`, `fret-ui-kit::imui`, or `crates/fret-ui` API changed.

## Why shared helper growth stays closed

The proof-budget rule says future `fret-ui-kit::imui` helper widening must name at least two real
first-party proof surfaces that both need the helper and cannot reasonably stay explicit.

This lane satisfies the “second real proof surface exists” prerequisite, but it does not satisfy
the stronger helper-readiness test:

1. the first proof is an asset-browser grid with multi-select, command-package, context-menu,
   inline rename, and status pressure,
2. the second proof is a compact shell-mounted outline tied to inspector/text-editing state,
3. the two surfaces do not yet demand the same reusable helper shape,
4. and the second proof remains simpler and clearer as explicit app code.

## What did not ship

1. No shared collection helper in `fret-ui-kit::imui`.
2. No `fret-imui` facade widening.
3. No `crates/fret-ui` runtime/mechanism contract change.
4. No new dedicated asset-grid/file-browser demo.
5. No helper-readiness follow-on was started from this folder.

## Reopen policy

Start a different narrow follow-on only if fresh first-party evidence can name:

- the exact shared collection helper being proposed,
- both first-party proof surfaces that need it,
- why explicit app-owned code is no longer the better choice on both surfaces,
- and the gate package that would prove the helper does not pull app policy into generic IMUI.

Do not reopen this folder just because the second proof now exists.
