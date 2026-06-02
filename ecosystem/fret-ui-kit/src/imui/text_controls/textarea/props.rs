use fret_runtime::Model;
use fret_ui::element::{Length, TextAreaProps};
use fret_ui::{ElementContext, UiHost};

use crate::imui::TextAreaOptions;

pub(super) fn textarea_props<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<String>,
    enabled: bool,
    options: &TextAreaOptions,
) -> TextAreaProps {
    let mut props = TextAreaProps::new(model);
    props.enabled = enabled;
    props.focusable = enabled && options.focusable;
    props.read_only = options.read_only;
    props.allow_tab_input = options.allow_tab_input;
    props.layout.size.width = Length::Fill;
    props.a11y_label = options.a11y_label.clone();
    props.test_id = options.test_id.clone();
    props.min_height = options.min_height;

    let theme = fret_ui::Theme::global(&*cx.app);
    props.chrome = super::super::style::imui_text_area_style_from_theme(theme);
    props.text_style = if options.stable_line_boxes {
        crate::typography::text_area_control_text_style(theme)
    } else {
        crate::typography::text_area_content_text_style(theme)
    };

    props
}
