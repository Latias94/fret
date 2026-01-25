use super::super::frame::*;
use super::super::prelude::*;
use super::ElementHostWidget;

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
            ElementInstance::SelectableText(props) => {
                cx.set_role(SemanticsRole::Text);
                cx.set_label(props.rich.text.as_ref().to_string());
                cx.set_text_selection_supported(true);
                if cx.focus == Some(cx.node) {
                    let (anchor, caret) = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::SelectableTextState::default,
                        |state| (state.selection_anchor, state.caret),
                    );
                    cx.set_text_selection(anchor as u32, caret as u32);
                } else {
                    cx.clear_text_selection();
                }
            }
            ElementInstance::Semantics(props) => {
                cx.set_role(props.role);
                if let Some(label) = props.label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if let Some(test_id) = props.test_id.as_ref() {
                    cx.set_test_id(test_id.as_ref().to_string());
                }
                if let Some(value) = props.value.as_ref() {
                    cx.set_value(value.as_ref().to_string());
                }
                if props.focusable && !props.disabled {
                    cx.set_focusable(true);
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
                if let Some(element) = props.labelled_by_element
                    && let Some(node) = cx.resolve_declarative_element(element)
                {
                    cx.push_labelled_by(node);
                }
                if let Some(element) = props.described_by_element
                    && let Some(node) = cx.resolve_declarative_element(element)
                {
                    cx.push_described_by(node);
                }
                if let Some(element) = props.controls_element
                    && let Some(node) = cx.resolve_declarative_element(element)
                {
                    cx.push_controlled(node);
                }
            }
            ElementInstance::SemanticFlex(props) => {
                cx.set_role(props.role);
            }
            ElementInstance::TextInput(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_input.is_none() {
                    self.text_input = Some(BoundTextInput::new(model.clone()));
                }
                let input = self.text_input.as_mut().expect("text input");
                if input.model_id() != model_id {
                    input.set_model(model);
                }
                input.set_enabled(props.enabled);
                input.set_focusable(props.focusable);
                input.set_a11y_role(props.a11y_role.unwrap_or(SemanticsRole::TextField));
                input.set_chrome_style(props.chrome);
                input.set_text_style(props.text_style);
                input.set_placeholder(props.placeholder);
                input.set_submit_command(props.submit_command);
                input.set_cancel_command(props.cancel_command);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if let Some(test_id) = props.test_id.as_ref() {
                    cx.set_test_id(test_id.as_ref().to_string());
                }
                if let Some(expanded) = props.expanded {
                    cx.set_expanded(expanded);
                }
                cx.set_active_descendant(props.active_descendant);
                input.semantics(cx);
            }
            ElementInstance::TextArea(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(model.clone()));
                }
                let area = self.text_area.as_mut().expect("text area");
                if area.model_id() != model_id {
                    area.set_model(model);
                }
                area.set_enabled(props.enabled);
                area.set_focusable(props.focusable);
                area.set_style(props.chrome);
                area.set_text_style(props.text_style);
                area.set_min_height(props.min_height);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if let Some(test_id) = props.test_id.as_ref() {
                    cx.set_test_id(test_id.as_ref().to_string());
                }
                area.semantics(cx);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.resizable_panel_group.is_none() {
                    self.resizable_panel_group =
                        Some(crate::resizable_panel_group::BoundResizablePanelGroup::new(
                            props.axis,
                            model.clone(),
                        ));
                }
                let group = self
                    .resizable_panel_group
                    .as_mut()
                    .expect("resizable panel group");
                if group.model_id() != model_id {
                    group.set_model(model);
                }
                group.set_axis(props.axis);
                group.set_enabled(props.enabled);
                group.set_min_px(props.min_px.clone());
                group.set_style(props.chrome.clone());
                group.semantics(cx);
            }
            ElementInstance::Pressable(props) => {
                if props.a11y.hidden {
                    cx.set_role(SemanticsRole::Generic);
                    if let Some(test_id) = props.a11y.test_id.as_ref() {
                        cx.set_test_id(test_id.as_ref().to_string());
                    }
                    cx.set_disabled(true);
                    cx.set_focusable(false);
                    cx.set_invokable(false);
                } else {
                    cx.set_role(props.a11y.role.unwrap_or(SemanticsRole::Button));
                    if let Some(label) = props.a11y.label.as_ref() {
                        cx.set_label(label.as_ref().to_string());
                    }
                    if let Some(test_id) = props.a11y.test_id.as_ref() {
                        cx.set_test_id(test_id.as_ref().to_string());
                    }
                    cx.set_active_descendant(props.a11y.active_descendant);
                    if props.a11y.selected {
                        cx.set_selected(true);
                    }
                    if let Some(expanded) = props.a11y.expanded {
                        cx.set_expanded(expanded);
                    }
                    if props.a11y.checked.is_some() {
                        cx.set_checked(props.a11y.checked);
                    }
                    if let Some(element) = props.a11y.labelled_by_element
                        && let Some(node) = cx.resolve_declarative_element(element)
                    {
                        cx.push_labelled_by(node);
                    }
                    if let Some(element) = props.a11y.described_by_element
                        && let Some(node) = cx.resolve_declarative_element(element)
                    {
                        cx.push_described_by(node);
                    }
                    if let Some(element) = props.a11y.controls_element
                        && let Some(node) = cx.resolve_declarative_element(element)
                    {
                        cx.push_controlled(node);
                    }
                    cx.set_disabled(!props.enabled);
                    cx.set_focusable(props.enabled && props.focusable);
                    cx.set_invokable(props.enabled);
                    cx.set_collection_position(props.a11y.pos_in_set, props.a11y.set_size);
                }
            }
            ElementInstance::VirtualList(_) => {
                cx.set_role(SemanticsRole::List);
            }
            ElementInstance::Flex(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::FocusScope(_)
            | ElementInstance::InteractivityGate(_)
            | ElementInstance::RovingFlex(_)
            | ElementInstance::Grid(_) => {
                // Flex/Grid are layout containers; they do not imply semantics beyond their children.
            }
            ElementInstance::Image(_)
            | ElementInstance::PointerRegion(_)
            | ElementInstance::InternalDragRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Opacity(_)
            | ElementInstance::EffectLayer(_)
            | ElementInstance::VisualTransform(_)
            | ElementInstance::RenderTransform(_)
            | ElementInstance::FractionalRenderTransform(_)
            | ElementInstance::Anchored(_)
            | ElementInstance::Scroll(_) => {
                cx.set_role(SemanticsRole::Generic);
            }
            ElementInstance::ViewportSurface(_) => {
                cx.set_role(SemanticsRole::Viewport);
            }
            _ => {}
        }
    }
}
