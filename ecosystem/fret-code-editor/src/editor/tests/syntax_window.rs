use super::*;

#[test]
fn syntax_prefetch_visible_window_uses_display_map_lines_under_soft_wrap() {
    let handle = CodeEditorHandle::new("abcdef\nuvwxyz\n123456");
    handle.set_soft_wrap_cols(Some(2));

    let visible_lines = {
        let st = handle.state.borrow();
        crate::editor::syntax::syntax_prefetch_visible_line_window(
            &st,
            WindowedRowsPaintFrame {
                viewport_height: Px(64.0),
                offset_y: Px(0.0),
                visible_start: 2,
                visible_end: 4,
            },
        )
    };

    assert_eq!(
        visible_lines,
        Some((0, 1)),
        "syntax prefetch must translate display rows to physical buffer lines before chunking"
    );
}
