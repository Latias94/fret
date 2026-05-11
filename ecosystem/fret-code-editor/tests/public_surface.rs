use fret_code_editor::{
    CodeEditor, CodeEditorCacheSizeSnapshotV1, CodeEditorHandle, CodeEditorMemorySnapshotV1,
    CodeFontFeaturePolicy, CodeFontFeaturePreset, Selection,
};
use fret_code_editor_buffer::Selection as BufferSelection;

#[test]
fn crate_root_exports_public_signature_types() {
    let handle = CodeEditorHandle::new("fn main() {}\n");
    let policy = CodeFontFeaturePolicy {
        preset: CodeFontFeaturePreset::NoLigatures,
        overrides: Vec::new(),
    };

    handle.set_code_font_feature_policy(policy.clone());
    let _editor = CodeEditor::new(handle.clone()).code_font_features(policy);

    let _cache_sizes: CodeEditorCacheSizeSnapshotV1 = handle.cache_size_snapshot();
    let _memory: CodeEditorMemorySnapshotV1 = handle.memory_snapshot();
    let selection = Selection {
        anchor: 0,
        focus: 2,
    };
    let _buffer_selection: BufferSelection = selection;
    handle.set_selection(selection);
}
