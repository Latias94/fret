use super::ElementHostWidget;
use crate::declarative::prelude::*;

pub(super) fn handle_text_input<H: UiHost>(
    this: &mut ElementHostWidget,
    cx: &mut EventCx<'_, H>,
    props: crate::element::TextInputProps,
    event: &Event,
) {
    let model = props.model.clone();
    let model_id = model.id();
    match this.text_input.as_mut() {
        None => {
            this.text_input = Some(BoundTextInput::new(model.clone()));
        }
        Some(input) => {
            if input.model_id() != model_id {
                input.set_model(model);
            }
        }
    }
    let input = this.text_input.as_mut().expect("text input");
    input.set_enabled(props.enabled);
    input.set_focusable(props.focusable);
    input.set_chrome_style(props.chrome);
    input.set_text_style(props.text_style);
    input.set_placeholder(props.placeholder);
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
    let model = props.model.clone();
    let model_id = model.id();
    match this.text_area.as_mut() {
        None => {
            this.text_area = Some(crate::text_area::BoundTextArea::new(model.clone()));
        }
        Some(area) => {
            if area.model_id() != model_id {
                area.set_model(model);
            }
        }
    }
    let area = this.text_area.as_mut().expect("text area");
    area.set_enabled(props.enabled);
    area.set_focusable(props.focusable);
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
    let model = props.model.clone();
    let model_id = model.id();
    match this.resizable_panel_group.as_mut() {
        None => {
            this.resizable_panel_group =
                Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                    props.axis,
                    model.clone(),
                ));
        }
        Some(group) => {
            if group.model_id() != model_id {
                group.set_model(model);
            }
        }
    }
    let group = this
        .resizable_panel_group
        .as_mut()
        .expect("resizable panel group");
    group.set_axis(props.axis);
    group.set_enabled(props.enabled);
    group.set_min_px(props.min_px.clone());
    group.set_style(props.chrome.clone());
    group.event(cx, event);
}
