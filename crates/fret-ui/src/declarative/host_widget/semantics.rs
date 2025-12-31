use super::*;

impl ElementHostWidget {
    pub(super) fn semantics_impl<H: UiHost>(&mut self, cx: &mut SemanticsCx<'_, H>) {
        let _element_id = self.element;
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };
        match instance {
            ElementInstance::Text(props) => {
                cx.set_role(SemanticsRole::Text);
                cx.set_label(props.text.as_ref().to_string());
            }
            ElementInstance::Semantics(props) => {
                cx.set_role(props.role);
                if let Some(label) = props.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if let Some(value) = props.value.as_ref() {
                    cx.set_value(value.as_ref().to_string());
                }
                if props.disabled {
                    cx.set_disabled(true);
                }
                if props.selected {
                    cx.set_selected(true);
                }
                if let Some(expanded) = props.expanded {
                    cx.set_expanded(expanded);
                }
                if props.checked.is_some() {
                    cx.set_checked(props.checked);
                }
                if props.active_descendant.is_some() {
                    cx.set_active_descendant(props.active_descendant);
                }
            }
            ElementInstance::TextInput(props) => {
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(props.model));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != props.model.id() {
                    input.set_model(props.model);
                }
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                cx.set_active_descendant(props.active_descendant);
                input.semantics(cx);
            }
            ElementInstance::TextArea(props) => {
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(props.model));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != props.model.id() {
                    area.set_model(props.model);
                }
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                area.semantics(cx);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            props.model,
                        ));
                }
                let group = self
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
                group.semantics(cx);
            }
            ElementInstance::Pressable(props) => {
                cx.set_role(props.a11y.role.unwrap_or(SemanticsRole::Button));
                if let Some(label) = props.a11y.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if props.a11y.selected {
                    cx.set_selected(true);
                }
                if let Some(expanded) = props.a11y.expanded {
                    cx.set_expanded(expanded);
                }
                if props.a11y.checked.is_some() {
                    cx.set_checked(props.a11y.checked);
                }
                cx.set_disabled(!props.enabled);
                cx.set_focusable(props.enabled);
                cx.set_invokable(props.enabled);
                cx.set_collection_position(props.a11y.pos_in_set, props.a11y.set_size);
            }
            ElementInstance::VirtualList(_) => {
                cx.set_role(SemanticsRole::List);
            }
            ElementInstance::Flex(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::RovingFlex(_)
            | ElementInstance::Grid(_) => {
                // Flex/Grid are layout containers; they do not imply semantics beyond their children.
            }
            ElementInstance::Image(_)
            | ElementInstance::PointerRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Opacity(_)
            | ElementInstance::VisualTransform(_)
            | ElementInstance::Scroll(_) => {
                cx.set_role(SemanticsRole::Generic);
            }
            _ => {}
        }
    }
}
