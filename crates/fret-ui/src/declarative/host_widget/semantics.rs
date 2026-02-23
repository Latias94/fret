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
                let text = props.rich.text.as_ref();
                cx.set_label(text.to_string());
                cx.set_value(text.to_string());
                cx.set_text_selection_supported(true);
                if cx.focus == Some(cx.node) {
                    let (anchor, caret) = crate::elements::with_element_state(
                        &mut *cx.app,
                        window,
                        self.element,
                        crate::element::SelectableTextState::default,
                        |state| (state.selection_anchor, state.caret),
                    );
                    let mut anchor = anchor.min(text.len());
                    let mut caret = caret.min(text.len());
                    crate::text_edit::utf8::clamp_selection_to_grapheme_boundaries(
                        text,
                        &mut anchor,
                        &mut caret,
                    );
                    cx.set_text_selection(anchor as u32, caret as u32);
                } else {
                    cx.clear_text_selection();
                }

                for span in props.interactive_spans.iter() {
                    let start = span.range.start.min(text.len());
                    let end = span.range.end.min(text.len());
                    if start > end {
                        debug_assert!(false, "interactive span range start > end");
                        continue;
                    }
                    if !text.is_char_boundary(start) || !text.is_char_boundary(end) {
                        debug_assert!(false, "interactive span range not on utf-8 boundary");
                        continue;
                    }
                    if let (Ok(start), Ok(end)) = (u32::try_from(start), u32::try_from(end)) {
                        cx.push_inline_link_span(start, end, Some(span.tag.as_ref().to_string()));
                    }
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
                if let Some(placeholder) = props.placeholder.as_ref() {
                    cx.set_placeholder(Some(placeholder.as_ref().to_string()));
                }
                if let Some(url) = props.url.as_ref() {
                    cx.set_url(Some(url.as_ref().to_string()));
                }
                cx.set_level(props.level);
                cx.set_orientation(props.orientation);
                cx.set_numeric_value(props.numeric_value);
                cx.set_numeric_range(props.min_numeric_value, props.max_numeric_value);
                cx.set_numeric_step(props.numeric_value_step);
                cx.set_numeric_jump(props.numeric_value_jump);
                cx.set_scroll_x(props.scroll_x, props.scroll_x_min, props.scroll_x_max);
                cx.set_scroll_y(props.scroll_y, props.scroll_y_min, props.scroll_y_max);
                if props.focusable && !props.disabled {
                    cx.set_focusable(true);
                }
                if let Some(editable) = props.value_editable {
                    cx.set_value_editable(editable);
                }
                if props.disabled {
                    cx.set_disabled(true);
                }
                if props.read_only {
                    cx.set_read_only(true);
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
                let Some(input) = self.text_input.as_mut() else {
                    debug_assert!(false, "text input must be initialized");
                    return;
                };
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
                if let Some(element) = props.controls_element
                    && let Some(node) = cx.resolve_declarative_element(element)
                {
                    cx.push_controlled(node);
                }
                input.semantics(cx);
            }
            ElementInstance::TextArea(props) => {
                let model = props.model.clone();
                let model_id = model.id();
                if self.text_area.is_none() {
                    self.text_area = Some(crate::text_area::BoundTextArea::new(model.clone()));
                }
                let Some(area) = self.text_area.as_mut() else {
                    debug_assert!(false, "text area must be initialized");
                    return;
                };
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
                let Some(group) = self.resizable_panel_group.as_mut() else {
                    debug_assert!(false, "resizable panel group must be initialized");
                    return;
                };
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
                    cx.set_level(props.a11y.level);
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
                    if props.a11y.checked_state.is_some() {
                        cx.set_checked_state(props.a11y.checked_state);
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
            ElementInstance::TextInputRegion(props) => {
                cx.set_role(SemanticsRole::TextField);
                if let Some(label) = props.a11y_label.as_ref() {
                    cx.set_label(label.as_ref().to_string());
                }
                if let Some(value) = props.a11y_value.as_ref() {
                    cx.set_value(value.as_ref().to_string());
                }
                cx.set_disabled(!props.enabled);
                cx.set_focusable(props.enabled);

                // This is a mechanism-only surface: it can accept selection updates via hooks, but
                // does not provide an internal buffer for `SetValue` edits.
                cx.set_value_editable(false);
                cx.set_read_only(true);
                cx.set_text_selection_supported(props.enabled);

                // Only publish ranges when focused, matching TextInput/TextArea behavior.
                if cx.focus == Some(cx.node) && props.a11y_value.is_some() {
                    if let Some((anchor, focus)) = props.a11y_text_selection {
                        cx.set_text_selection(anchor, focus);
                    } else {
                        cx.clear_text_selection();
                    }
                    if let Some((start, end)) = props.a11y_text_composition {
                        cx.set_text_composition(start, end);
                    } else {
                        cx.clear_text_composition();
                    }
                } else {
                    cx.clear_text_selection();
                    cx.clear_text_composition();
                }
            }
            ElementInstance::Flex(_)
            | ElementInstance::DismissibleLayer(_)
            | ElementInstance::FocusScope(_)
            | ElementInstance::InteractivityGate(_)
            | ElementInstance::FocusTraversalGate(_)
            | ElementInstance::RovingFlex(_)
            | ElementInstance::Grid(_) => {
                // Flex/Grid are layout containers; they do not imply semantics beyond their children.
            }
            ElementInstance::Image(_) => {
                cx.set_role(SemanticsRole::Image);
            }
            ElementInstance::PointerRegion(_)
            | ElementInstance::InternalDragRegion(_)
            | ElementInstance::ExternalDragRegion(_)
            | ElementInstance::HoverRegion(_)
            | ElementInstance::Spinner(_)
            | ElementInstance::Opacity(_)
            | ElementInstance::EffectLayer(_)
            | ElementInstance::MaskLayer(_)
            | ElementInstance::CompositeGroup(_)
            | ElementInstance::VisualTransform(_)
            | ElementInstance::RenderTransform(_)
            | ElementInstance::FractionalRenderTransform(_)
            | ElementInstance::Anchored(_) => {
                cx.set_role(SemanticsRole::Generic);
            }
            ElementInstance::Scroll(props) => {
                cx.set_role(SemanticsRole::Viewport);
                cx.set_scroll_by_supported(true);

                let scroll_x = props.axis.scroll_x();
                let scroll_y = props.axis.scroll_y();
                cx.set_orientation(match (scroll_x, scroll_y) {
                    (true, false) => Some(fret_core::SemanticsOrientation::Horizontal),
                    (false, true) => Some(fret_core::SemanticsOrientation::Vertical),
                    _ => None,
                });

                let external_handle = props.scroll_handle.clone();
                let handle = crate::elements::with_element_state(
                    &mut *cx.app,
                    window,
                    self.element,
                    crate::element::ScrollState::default,
                    |state| {
                        external_handle
                            .as_ref()
                            .unwrap_or(&state.scroll_handle)
                            .clone()
                    },
                );
                let offset = handle.offset();
                let max = handle.max_offset();
                if scroll_x {
                    cx.set_scroll_x(Some(offset.x.0 as f64), Some(0.0), Some(max.x.0 as f64));
                }
                if scroll_y {
                    cx.set_scroll_y(Some(offset.y.0 as f64), Some(0.0), Some(max.y.0 as f64));
                }
            }
            ElementInstance::ViewportSurface(_) => {
                cx.set_role(SemanticsRole::Viewport);
            }
            _ => {}
        }

        if let Some(Some(decoration)) = with_element_record_for_node(cx.app, window, cx.node, |r| {
            r.semantics_decoration.clone()
        }) {
            if let Some(role) = decoration.role {
                cx.set_role(role);
            }
            if let Some(label) = decoration.label.as_ref() {
                cx.set_label(label.as_ref().to_string());
            }
            if let Some(test_id) = decoration.test_id.as_ref() {
                cx.set_test_id(test_id.as_ref().to_string());
            }
            if let Some(value) = decoration.value.as_ref() {
                cx.set_value(value.as_ref().to_string());
            }
            if let Some(disabled) = decoration.disabled {
                cx.set_disabled(disabled);
            }
            if let Some(read_only) = decoration.read_only {
                cx.set_read_only(read_only);
            }
            if let Some(selected) = decoration.selected {
                cx.set_selected(selected);
            }
            if let Some(expanded) = decoration.expanded {
                cx.set_expanded(expanded);
            }
            if let Some(checked) = decoration.checked {
                cx.set_checked(checked);
            }
            if let Some(placeholder) = decoration.placeholder.as_ref() {
                cx.set_placeholder(Some(placeholder.as_ref().to_string()));
            }
            if let Some(url) = decoration.url.as_ref() {
                cx.set_url(Some(url.as_ref().to_string()));
            }
            if let Some(level) = decoration.level {
                cx.set_level(Some(level));
            }
            if let Some(orientation) = decoration.orientation {
                cx.set_orientation(Some(orientation));
            }
            if let Some(value) = decoration.numeric_value {
                cx.set_numeric_value(Some(value));
            }
            if decoration.min_numeric_value.is_some() || decoration.max_numeric_value.is_some() {
                cx.set_numeric_range(decoration.min_numeric_value, decoration.max_numeric_value);
            }
            if let Some(step) = decoration.numeric_value_step {
                cx.set_numeric_step(Some(step));
            }
            if let Some(jump) = decoration.numeric_value_jump {
                cx.set_numeric_jump(Some(jump));
            }
            if decoration.scroll_x.is_some()
                || decoration.scroll_x_min.is_some()
                || decoration.scroll_x_max.is_some()
            {
                cx.set_scroll_x(
                    decoration.scroll_x,
                    decoration.scroll_x_min,
                    decoration.scroll_x_max,
                );
            }
            if decoration.scroll_y.is_some()
                || decoration.scroll_y_min.is_some()
                || decoration.scroll_y_max.is_some()
            {
                cx.set_scroll_y(
                    decoration.scroll_y,
                    decoration.scroll_y_min,
                    decoration.scroll_y_max,
                );
            }
            if let Some(element) = decoration.active_descendant_element
                && let Some(node) = cx.resolve_declarative_element(element)
            {
                cx.set_active_descendant(Some(node));
            }
            if let Some(element) = decoration.labelled_by_element
                && let Some(node) = cx.resolve_declarative_element(element)
            {
                cx.push_labelled_by(node);
            }
            if let Some(element) = decoration.described_by_element
                && let Some(node) = cx.resolve_declarative_element(element)
            {
                cx.push_described_by(node);
            }
            if let Some(element) = decoration.controls_element
                && let Some(node) = cx.resolve_declarative_element(element)
            {
                cx.push_controlled(node);
            }
        }
    }
}
