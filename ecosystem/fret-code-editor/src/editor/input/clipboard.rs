use super::*;

pub(in crate::editor) fn copy_selection(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &CodeEditorState,
) {
    let range = st.selection.normalized();
    if range.is_empty() {
        return;
    }
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let Some(text) = st.buffer.slice_to_string(start..end) else {
        return;
    };
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardWriteText {
        window: action_cx.window,
        token,
        text,
    });
}

pub(in crate::editor) fn request_paste(host: &mut dyn UiActionHost, action_cx: ActionCx) {
    let token = host.next_clipboard_token();
    host.push_effect(Effect::ClipboardReadText {
        window: action_cx.window,
        token,
    });
}

pub(in crate::editor) fn cut_selection(
    host: &mut dyn UiActionHost,
    action_cx: ActionCx,
    st: &mut CodeEditorState,
) -> bool {
    let range = st.selection.normalized();
    if range.is_empty() {
        return false;
    }
    copy_selection(host, action_cx, st);
    let start = range.start.min(st.buffer.len_bytes());
    let end = range.end.min(st.buffer.len_bytes());
    let out = apply_and_record_edit(
        st,
        UndoGroupKind::Cut,
        Edit::Delete { range: start..end },
        Selection {
            anchor: start,
            focus: start,
        },
    )
    .is_some();
    if out {
        st.caret_preferred_x = None;
    }
    out
}
