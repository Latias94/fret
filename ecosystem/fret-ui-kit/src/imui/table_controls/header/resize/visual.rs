use fret_core::Px;
use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, Theme, UiHost};

pub(super) const TABLE_RESIZE_HANDLE_MIN_HEIGHT: Px = Px(24.0);
const TABLE_RESIZE_HANDLE_VISUAL_WIDTH: Px = Px(1.0);

pub(super) fn table_resize_handle_visual<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let mut color = theme
        .color_by_key("table.border")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_token("border"));
    if !enabled {
        color.a *= 0.45;
    }

    let mut grip = ContainerProps::default();
    grip.background = Some(color);
    grip.layout.size.width = Length::Px(TABLE_RESIZE_HANDLE_VISUAL_WIDTH);
    grip.layout.size.height = Length::Px(TABLE_RESIZE_HANDLE_MIN_HEIGHT);
    grip.layout.flex.shrink = 0.0;

    crate::ui::h_flex(move |cx| vec![cx.container(grip, |_cx| Vec::new())])
        .gap_metric(crate::MetricRef::space(crate::Space::N0))
        .justify(crate::Justify::Center)
        .items(crate::Items::Stretch)
        .no_wrap()
        .into_element(cx)
}
