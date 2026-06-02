use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length, RowProps, SpacingLength};
use fret_ui::{ElementContext, UiHost};

use crate::imui::TabBarOptions;
use crate::primitives::tabs;

pub(super) fn tab_list_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    triggers: Vec<AnyElement>,
    options: &TabBarOptions,
) -> AnyElement {
    let gap = options.gap.clone();
    let list_layout = LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        ..Default::default()
    };
    cx.semantics(
        {
            let mut props =
                tabs::tab_list_semantics_props(list_layout, tabs::TabsOrientation::Horizontal);
            props.test_id = options.test_id.clone();
            props
        },
        move |cx| {
            let mut row = RowProps::default();
            row.layout.size.width = Length::Fill;
            row.layout.size.height = Length::Auto;
            row.gap = SpacingLength::Px(Px(0.0));
            vec![cx.row(row, move |cx| {
                vec![
                    crate::ui::h_flex(move |_cx| triggers)
                        .gap_metric(gap.clone())
                        .justify(crate::Justify::Start)
                        .items(crate::Items::Center)
                        .no_wrap()
                        .into_element(cx),
                ]
            })]
        },
    )
}
