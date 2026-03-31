//! Immediate-mode section-label helper.

use std::sync::Arc;

use fret_core::Px;
use fret_ui::UiHost;
use fret_ui::element::{ContainerProps, Length, SemanticsDecoration, TextProps};
use fret_ui::{ElementContext, Theme};

use super::{SeparatorTextOptions, UiWriterImUiFacadeExt};

pub(super) fn separator_text_with_options<H: UiHost, W: UiWriterImUiFacadeExt<H> + ?Sized>(
    ui: &mut W,
    label: Arc<str>,
    options: SeparatorTextOptions,
) {
    let element = ui.with_cx_mut(|cx| separator_text_element(cx, label, options));
    ui.add(element);
}

fn separator_text_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    options: SeparatorTextOptions,
) -> fret_ui::element::AnyElement {
    let label_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.label")));
    let line_test_id = options
        .test_id
        .as_ref()
        .map(|base| Arc::from(format!("{base}.line")));

    let mut label_props = TextProps::new(label);
    label_props.layout.flex.shrink = 0.0;
    let mut label = cx.text_props(label_props);
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
