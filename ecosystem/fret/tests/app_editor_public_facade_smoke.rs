#![cfg(feature = "editor")]

use fret::app::editor::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, EditorThemePreset,
    EditorThemePresetPicker, EditorThemePresetPickerLocalStateExt as _, InspectorTextFieldBinding,
    TextField, TextFieldAssistiveSemantics, TextFieldBlurBehavior, TextFieldMode, TextFieldOptions,
    TextFieldOutcome,
};
use fret::app::{App, AppLocalStateExt as _};

#[test]
fn app_editor_facade_builds_inspector_controls_without_raw_model_imports() {
    let mut app = App::new();
    let binding = InspectorTextFieldBinding::new(&mut app, "Draft", "Ready");
    let field: TextField = binding.text_field(TextFieldOptions {
        a11y_label: Some("Inspector notes".into()),
        mode: TextFieldMode::PlainText,
        blur_behavior: TextFieldBlurBehavior::PreserveDraft,
        selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
        cancel_behavior: EditorTextCancelBehavior::Clear,
        assistive_semantics: TextFieldAssistiveSemantics::default(),
        ..Default::default()
    });
    let preset = app.local_state(EditorThemePreset::Default);
    let picker: EditorThemePresetPicker = preset.editor_theme_preset_picker();

    let _ = (field, picker);
    let _ = std::mem::size_of::<Option<TextFieldOutcome>>();
}
