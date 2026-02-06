use super::super::frame::ElementInstance;
use super::super::prelude::*;
use super::ElementHostWidget;

mod dismissible;
mod hooks;
mod internal_drag_region;
mod pointer_region;
mod pressable;
mod roving_flex;
mod scroll;
mod scrollbar;
mod selectable_text;
mod text;
mod text_input_region;
mod wheel_region;

pub(super) fn invalidate_scroll_handle_bindings<H: UiHost>(
    cx: &mut EventCx<'_, H>,
    window: AppWindowId,
    handle_key: usize,
    inv: Invalidation,
) {
    let bound = crate::declarative::frame::bound_elements_for_scroll_handle(
        &mut *cx.app,
        window,
        handle_key,
    );
    if bound.is_empty() {
        return;
    }

    let mut unique = std::collections::HashSet::with_capacity(bound.len());
    for element in bound {
        if !unique.insert(element) {
            continue;
        }
        let Some(node) = crate::declarative::mount::node_for_element_in_window_frame(
            &mut *cx.app,
            window,
            element,
        ) else {
            continue;
        };
        cx.invalidate(node, inv);
    }
}

impl ElementHostWidget {
    pub(super) fn event_impl<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        let is_text_input = matches!(
            &instance,
            ElementInstance::TextInput(_)
                | ElementInstance::TextArea(_)
                | ElementInstance::TextInputRegion(_)
        );

        if hooks::handle_timer_event(self, cx, window, event) {
            return;
        }

        if let Event::KeyDown {
            key,
            modifiers,
            repeat,
        } = event
            && cx.focus == Some(cx.node)
            && !is_text_input
            && hooks::try_key_hook(self, cx, window, *key, *modifiers, *repeat)
        {
            return;
        }

        match instance {
            ElementInstance::SelectableText(props) => {
                selectable_text::handle_selectable_text(self, cx, window, props, event);
            }
            ElementInstance::TextInput(props) => {
                text::handle_text_input(self, cx, props, event);
            }
            ElementInstance::TextArea(props) => {
                text::handle_text_area(self, cx, props, event);
            }
            ElementInstance::TextInputRegion(props) => {
                text_input_region::handle_text_input_region(self, cx, window, props, event);
            }
            ElementInstance::ResizablePanelGroup(props) => {
                text::handle_resizable_panel_group(self, cx, props, event);
            }
            ElementInstance::VirtualList(props) => {
                if scroll::handle_virtual_list(self, cx, window, props, event) {
                    return;
                }
            }
            ElementInstance::Scroll(props) => {
                if scroll::handle_scroll(self, cx, window, props, event) {
                    return;
                }
            }
            ElementInstance::Scrollbar(props) => {
                if scrollbar::handle_scrollbar(self, cx, window, props, event) {
                    return;
                }
            }
            ElementInstance::WheelRegion(props) => {
                if wheel_region::handle_wheel_region(self, cx, window, props, event) {
                    return;
                }
            }
            ElementInstance::DismissibleLayer(props) => {
                dismissible::handle_dismissible_layer(self, cx, window, props, event);
            }
            ElementInstance::Pressable(props) => {
                pressable::handle_pressable(self, cx, window, props, event);
            }
            ElementInstance::PointerRegion(props) => {
                pointer_region::handle_pointer_region(self, cx, window, props, event);
            }
            ElementInstance::InternalDragRegion(props) => {
                internal_drag_region::handle_internal_drag_region(self, cx, window, props, event);
            }
            ElementInstance::RovingFlex(props) => {
                roving_flex::handle_roving_flex(self, cx, window, props, event);
            }
            _ => {}
        }

        if is_text_input
            && !cx.stop_propagation
            && let Event::KeyDown {
                key,
                modifiers,
                repeat,
            } = event
            && cx.focus == Some(cx.node)
            && hooks::try_key_hook(self, cx, window, *key, *modifiers, *repeat)
        {}
    }

    pub(super) fn event_observer_impl<H: UiHost>(
        &mut self,
        cx: &mut crate::widget::ObserverCx<'_, H>,
        event: &Event,
    ) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        if let ElementInstance::DismissibleLayer(props) = instance {
            dismissible::handle_dismissible_layer_observer(self, cx, window, props, event);
        }
    }
}
