//! Editor controls (interactive widgets built from primitives).

pub mod asset_ref_field;
pub mod axis_drag_value;
pub mod checkbox;
pub mod color_edit;
pub mod drag_value;
pub mod editor_theme_preset_picker;
pub mod enum_select;
pub mod field_status;
pub mod icon_button;
pub mod mini_search_box;
pub mod numeric_input;
pub mod slider;
pub mod text_assist_field;
pub mod text_field;
pub mod transform_edit;
pub mod vec_edit;

mod session_shell;

pub use crate::primitives::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, NumericPresentation, NumericTextAffixes,
    NumericValueConstraints, affixed_number_format, affixed_number_parse, degrees_format,
    degrees_parse, fixed_decimals_format, plain_number_parse,
};
pub use asset_ref_field::{
    AssetRefField, AssetRefFieldOptions, AssetRefFieldValue, OnAssetRefFieldAction,
};
pub use axis_drag_value::{
    AxisDragValue, AxisDragValueOptions, AxisDragValueOutcome, AxisDragValueResetAction,
    OnAxisDragValueOutcome,
};
pub use checkbox::{Checkbox, CheckboxOptions};
pub use color_edit::{
    ColorEdit, ColorEditAlphaPreview, ColorEditCopyOptions, ColorEditDragDropComponents,
    ColorEditDragDropOptions, ColorEditDragDropPayload, ColorEditOptions, ColorEditPaletteEntry,
    ColorEditPaletteSlotDrop, ColorEditPopupNumericInputs, ColorEditPopupOptions,
    ColorEditPopupPicker, ColorEditPopupSidePreview, ColorEditTooltipOptions,
    OnColorEditPaletteSlotDrop, default_color_edit_palette,
};
pub use drag_value::{DragValue, DragValueOptions, DragValueOutcome, OnDragValueOutcome};
pub use editor_theme_preset_picker::{EditorThemePresetPicker, EditorThemePresetPickerOptions};
pub use enum_select::{EnumSelect, EnumSelectItem, EnumSelectOptions};
pub use field_status::{FieldStatus, FieldStatusBadge, FieldStatusBadgeOptions};
pub use fret_ui_kit::headless::text_assist::{
    InputOwnedTextAssistKeyOptions, TextAssistItem, TextAssistMatch,
};
pub use icon_button::{IconButton, IconButtonOptions, OnIconButtonActivate};
pub use mini_search_box::{MiniSearchBox, MiniSearchBoxOptions};
pub use numeric_input::{
    NumericFormatFn, NumericInput, NumericInputOptions, NumericInputOutcome,
    NumericInputSelectionBehavior, NumericParseFn, NumericValidateFn, OnNumericInputOutcome,
};
pub use slider::{Slider, SliderOptions};
pub use text_assist_field::{
    OnTextAssistFieldAccept, TextAssistField, TextAssistFieldOptions, TextAssistFieldSurface,
};
pub use text_field::{
    OnTextFieldOutcome, TextField, TextFieldAssistiveSemantics, TextFieldBlurBehavior,
    TextFieldDraftController, TextFieldDraftSnapshot, TextFieldMode, TextFieldOptions,
    TextFieldOutcome,
};
pub use transform_edit::{
    OnTransformEditAxisOutcome, TransformEdit, TransformEditAxisOutcome,
    TransformEditLayoutVariant, TransformEditOptions, TransformEditPresentations,
    TransformEditSection,
};
pub use vec_edit::{
    OnVecEditAxisOutcome, Vec2Edit, Vec3Edit, Vec4Edit, VecEditAxis, VecEditAxisOutcome,
    VecEditLayoutVariant, VecEditOptions,
};
