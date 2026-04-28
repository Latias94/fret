# ImUi TextField Draft Controller API Proof v1 - Design

Status: active narrow P1 lane
Last updated: 2026-04-28

## Problem

`imui-textfield-draft-buffer-contract-audit-v1` correctly kept the internal preserved draft buffer
private because no proof surface needed external commit/discard yet. `editor_notes_demo.rs` now has
a stronger proof surface: a multiline Notes field with `TextFieldBlurBehavior::PreserveDraft` and
app-authored draft actions. Those actions should be real editor affordances instead of status-only
markers.

## Scope

Owned here:

1. Add an opaque `TextFieldDraftController` in `ecosystem/fret-ui-editor`.
2. Allow a buffered `TextField` to bind that controller to its internal draft session.
3. Use `editor_notes_demo.rs` to prove external `Commit draft` and `Discard draft` actions.
4. Gate the public API shape, editor-notes proof surface, and source-policy boundary.

Not owned here:

1. No public `Model<String>` handle for the internal draft buffer.
2. No `crates/fret-ui`, `fret-ui-kit::imui`, `fret-imui`, or `fret-authoring` API widening.
3. No persistence, workspace dirty-close prompts, command-bus integration, clipboard, or menu wiring.
4. No generic document state contract.

## Target Interface

The intended v1 interface is deliberately small:

```rust
let draft_controller = TextFieldDraftController::new();

TextField::new(notes_model)
    .options(TextFieldOptions {
        buffered: true,
        blur_behavior: TextFieldBlurBehavior::PreserveDraft,
        draft_controller: Some(draft_controller.clone()),
        ..Default::default()
    });

draft_controller.commit(host, action_cx);
draft_controller.discard(host, action_cx);
```

The controller is an operation handle, not a draft-data handle. It may commit or discard through the
same internal session semantics used by Enter/Escape and blur behavior, but it does not expose the
draft model.
