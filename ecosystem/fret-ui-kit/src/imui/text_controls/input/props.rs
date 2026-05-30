use std::sync::Arc;

use fret_core::NodeId;
use fret_ui::UiHost;
use fret_ui::element::TextInputProps;

use crate::imui::{InputTextMode, InputTextOptions};

#[derive(Debug, Default, Clone, Copy)]
pub(in crate::imui) struct InputTextAssistiveSemantics {
    pub active_descendant: Option<NodeId>,
    pub active_descendant_element: Option<u64>,
    pub controls_element: Option<u64>,
    pub expanded: Option<bool>,
}

pub(in crate::imui::text_controls) fn input_text_props<H: UiHost>(
    cx: &fret_ui::ElementContext<'_, H>,
    model: fret_runtime::Model<String>,
    enabled: bool,
    options: &InputTextOptions,
    assistive_semantics: InputTextAssistiveSemantics,
) -> TextInputProps {
    let mut props = TextInputProps::new(model);
    props.enabled = enabled;
    props.focusable = enabled && options.focusable;
    props.read_only = options.read_only;
    props.obscure_text = matches!(options.mode, InputTextMode::Password);
    props.layout = super::super::style::input_text_layout();
    props.a11y_label = options.a11y_label.clone();
    props.a11y_role = options.a11y_role;
    props.active_descendant = assistive_semantics.active_descendant;
    props.active_descendant_element = assistive_semantics.active_descendant_element;
    props.controls_element = assistive_semantics.controls_element;
    props.expanded = assistive_semantics.expanded;
    props.test_id = options.test_id.clone();
    props.placeholder = options.placeholder.clone();
    props.submit_command = options.submit_command.clone();
    props.cancel_command = options.cancel_command.clone();
    if !options.filters.is_empty() || options.custom_filter.is_some() {
        let filters = options.filters;
        let custom_filter = options.custom_filter.clone();
        props.insert_filter = Some(Arc::new(move |text| {
            let filtered = filters.filter_text(text);
            match custom_filter.as_ref() {
                Some(filter) => filter.filter_text(&filtered),
                None => filtered,
            }
        }));
    }
    let theme = fret_ui::Theme::global(&*cx.app);
    props.chrome = super::super::style::imui_text_input_style_from_theme(theme);
    props.text_style = super::super::style::default_input_text_style_from_theme(theme);
    props
}
