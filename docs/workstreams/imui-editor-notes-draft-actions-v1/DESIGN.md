# ImUi Editor Notes Draft Actions v1 - Design

Status: closed narrow P1 lane
Last updated: 2026-04-24

## Problem

`imui-next-gap-audit-v1` recommends the next non-multi-window IMUI implementation slice should stay
on `editor_notes_demo.rs` and deepen app-owned editor-note draft actions. The existing proof already
shows notes editing, last action, draft status, and one inspector command/status loop, but it lacks
explicit local draft action affordances.

## Scope

Owned here:

1. Add app-owned draft action affordances to the existing editor-notes inspector surface.
2. Keep the action result visible through existing local outcome/status models.
3. Add stable test IDs and source-policy markers.
4. Keep behavior locally testable without macOS or multi-window acceptance.

Not owned here:

1. No persistence or filesystem save command.
2. No workspace dirty-close prompt.
3. No command bus, clipboard, or menu integration.
4. No `TextField` draft-buffer API widening.
5. No `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API changes.

## Target Slice

Add two inspector-local draft action buttons that update app-owned feedback state:

- `Mark draft ready`
- `Clear draft marker`

These are action/status affordances, not hidden-buffer commit/discard APIs. They intentionally avoid
claiming access to the `TextField` preserved draft buffer until a later lane proves that exact
contract is needed.
