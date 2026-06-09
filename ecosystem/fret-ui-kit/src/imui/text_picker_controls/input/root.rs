use fret_ui::element::{AnyElement, ContainerProps, Length};
use fret_ui::{ElementContext, UiHost};

pub(super) fn build_text_picker_input_root_container<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    input_element: AnyElement,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;

    cx.container(props, |_cx| vec![input_element])
}
