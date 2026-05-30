use fret_core::Px;
use fret_ui::element::{AnyElement, ContainerProps, InsetStyle, Length, Overflow, PositionStyle};
use fret_ui::{ElementContext, GlobalElementId, UiHost};

pub(super) fn floating_layer_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    layer_id: GlobalElementId,
    windows_sorted: Vec<AnyElement>,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset = InsetStyle {
        left: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        top: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
    };
    props.layout.overflow = Overflow::Visible;
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;

    let mut layer = cx.container(props, move |_cx| windows_sorted);
    layer.id = layer_id;
    layer
}
