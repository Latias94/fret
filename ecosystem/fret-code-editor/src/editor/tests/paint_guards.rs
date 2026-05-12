#[test]
fn paint_source_does_not_materialize_whole_buffer_string() {
    // Regression guard: the editor paint path should never call `TextBuffer::text_string()`.
    // Materializing the entire rope would scale with document size and defeat row virtualization.
    const SRC: &str = include_str!("../paint/mod.rs");
    assert!(
        !SRC.contains(".text_string("),
        "paint/mod.rs must not call TextBuffer::text_string()"
    );
}
