use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsDecoration};
use fret_ui::{ElementContext, Theme, UiHost};

use super::SeparatorTextOptions;

pub(super) fn separator_text_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    options: SeparatorTextOptions,
) -> AnyElement {
    let label_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.label")));
    let line_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.line")));

    let mut label = crate::declarative::text::text_section_chrome_label(cx, label);
    if let Some(test_id) = label_test_id {
        label = label.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let theme = Theme::global(&*cx.app);
    let mut line_props = ContainerProps::default();
    line_props.background = Some(
        theme
            .color_by_key("border")
            .unwrap_or_else(|| theme.color_token("border")),
    );
    line_props.layout.size.width = Length::Px(Px(0.0));
    line_props.layout.size.height = Length::Px(Px(1.0));
    line_props.layout.flex.grow = 1.0;
    line_props.layout.flex.shrink = 1.0;
    line_props.layout.flex.basis = Length::Px(Px(0.0));

    let mut line = cx.container(line_props, |_cx| Vec::new());
    if let Some(test_id) = line_test_id {
        line = line.attach_semantics(SemanticsDecoration::default().test_id(test_id));
    }

    let row = crate::ui::h_flex(move |_cx| vec![label, line])
        .gap_metric(Px(8.0).into())
        .items(crate::Items::Center)
        .no_wrap();

    if let Some(test_id) = options.test_id {
        row.test_id(test_id).into_element(cx)
    } else {
        row.into_element(cx)
    }
}
