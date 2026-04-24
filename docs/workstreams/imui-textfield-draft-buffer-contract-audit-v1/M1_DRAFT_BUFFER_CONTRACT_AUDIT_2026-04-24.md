# M1 Draft Buffer Contract Audit - 2026-04-24

Status: complete

## Findings

1. The preserved draft buffer is an internal keyed local model.
   Evidence: `TextField::into_element_keyed` derives `draft = buffered.then(|| draft_model(cx))`,
   and `draft_model` uses `cx.local_model(String::new)`.

2. Commit/cancel behavior is tied to internal focus/session state.
   Evidence: `sync_buffered_text_field_session`, `install_buffered_text_field_blur_handler`,
   `commit_buffered_text_field`, and `cancel_buffered_text_field` all coordinate the model, draft,
   pending blur timers, and `BufferedTextFieldState`.

3. `TextFieldBlurBehavior::PreserveDraft` intentionally clears pending blur instead of exposing an
   app-owned draft handle.
   Evidence: `plan_buffered_text_field_focus_transition` maps `PreserveDraft` to
   `BufferedTextFieldPendingBlurPlan::Clear`.

4. The editor-notes proof currently only needs app-owned action/status feedback.
   Evidence: `imui-editor-notes-draft-actions-v1` closes with `Draft actions` markers and explicitly
   avoids hidden-buffer commit/discard APIs.

## Verdict

Do not expose a public `TextField` preserved draft-buffer API now.

The current implementation is a valid internal mechanism, but promoting it would freeze contract
questions that are not yet answered:

- whether apps should receive a draft `Model<String>`, an opaque draft handle, or callback-only
  commands,
- whether commit/discard should be focus-owned, app-owned, or command-owned,
- how pending blur timers interact with external actions,
- and how assistive surfaces, multiline fields, and submit commands should compose with external
  draft control.

## Required Future Evidence

A future API-proof lane must provide all of the following before widening `TextField`:

1. A first-party proof surface that truly needs external preserved-draft commit/discard.
2. A concrete API sketch naming ownership of draft model/handle, commit, discard, and outcome
   emission.
3. Tests for `PreserveDraft`, refocus, pending blur, Enter/Escape, multiline commit shortcut, and
   clear-button behavior.
4. A no-leak proof that policy stays in `fret-ui-editor` or app code, not `crates/fret-ui`.
