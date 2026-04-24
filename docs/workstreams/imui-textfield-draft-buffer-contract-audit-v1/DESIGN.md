# ImUi TextField Draft Buffer Contract Audit v1 - Design

Status: closed narrow P1 audit lane
Last updated: 2026-04-24

## Problem

`imui-editor-notes-draft-actions-v1` closed with app-owned draft action markers instead of claiming
access to the preserved `TextField` draft buffer. The next question is whether Fret should expose a
public contract for observing, committing, or discarding that preserved draft buffer.

## Scope

Owned here:

1. Audit the current `TextField` buffered draft implementation.
2. Decide whether there is enough evidence to expose a public draft-buffer API now.
3. Record the smallest future API-proof lane shape if evidence is insufficient.
4. Keep the editor-notes proof and `fret-ui-editor` implementation aligned with the mechanism vs
   policy boundary.

Not owned here:

1. No `TextFieldOptions` or `TextField` API changes.
2. No public model handles for the internal draft buffer.
3. No persistence, dirty-close, save command, command bus, or menu integration.
4. No `fret-ui-kit::imui`, `fret-imui`, `fret-authoring`, or `crates/fret-ui` API widening.

## Target Outcome

Close with an explicit verdict on whether to expose a draft-buffer contract now, and name the
minimum evidence required before any future public API is admitted.
