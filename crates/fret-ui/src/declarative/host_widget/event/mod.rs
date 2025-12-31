use super::super::frame::ElementInstance;
use super::super::prelude::*;
use super::ElementHostWidget;

mod dismissible;
mod hooks;
mod pointer_region;
mod pressable;
mod roving_flex;
mod scroll;
mod scrollbar;
mod text;

impl ElementHostWidget {
    pub(super) fn event_impl<H: UiHost>(&mut self, cx: &mut EventCx<'_, H>, event: &Event) {
        let Some(window) = cx.window else {
            return;
        };
        let Some(instance) = self.instance(cx.app, window, cx.node) else {
            return;
        };

        let is_text_input = matches!(
            instance,
            ElementInstance::TextInput(_) | ElementInstance::TextArea(_)
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
            ElementInstance::TextInput(props) => {
                text::handle_text_input(self, cx, props, event);
            }
            ElementInstance::TextArea(props) => {
                text::handle_text_area(self, cx, props, event);
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
            ElementInstance::DismissibleLayer(props) => {
                dismissible::handle_dismissible_layer(self, cx, window, props, event);
            }
            ElementInstance::Pressable(props) => {
                pressable::handle_pressable(self, cx, window, props, event);
            }
            ElementInstance::PointerRegion(props) => {
                pointer_region::handle_pointer_region(self, cx, window, props, event);
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
}
