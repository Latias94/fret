use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_text_input<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    props: crate::element::TextInputProps,
    event: &Event,
) {
    if this.text_input.is_none() {
        this.text_input = Some(BoundTextInput::new(props.model));
    }
    let input = this.text_input.as_mut().expect("text input");
    if input.model_id() != props.model.id() {
        input.set_model(props.model);
    }
    input.set_chrome_style(props.chrome);
    input.set_text_style(props.text_style);
    input.set_submit_command(props.submit_command);
    input.set_cancel_command(props.cancel_command);
    input.event(cx, event);
}

pub(super) fn handle_text_area<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    props: crate::element::TextAreaProps,
    event: &Event,
) {
    if this.text_area.is_none() {
        this.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
    }
    let area = this.text_area.as_mut().expect("text area");
    if area.model_id() != props.model.id() {
        area.set_model(props.model);
    }
    area.set_style(props.chrome);
    area.set_text_style(props.text_style);
    area.set_min_height(props.min_height);
    area.event(cx, event);
}

pub(super) fn handle_resizable_panel_group<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    props: crate::element::ResizablePanelGroupProps,
    event: &Event,
) {
    if this.resizable_panel_group.is_none() {
        this.resizable_panel_group = Some(
            crate::resizable_panel_group::BoundResizablePanelGroup::new(props.axis, props.model),
        );
    }
    let group = this
        .resizable_panel_group
        .as_mut()
        .expect("resizable panel group");
    if group.model_id() != props.model.id() {
        group.set_model(props.model);
    }
    group.set_axis(props.axis);
    group.set_enabled(props.enabled);
    group.set_min_px(props.min_px.clone());
    group.set_style(props.chrome.clone());
    group.event(cx, event);
}
