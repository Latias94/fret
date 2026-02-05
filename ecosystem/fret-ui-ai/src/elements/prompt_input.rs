use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::OnActivate;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{Justify, LayoutRefinement, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, InputGroup};

#[derive(Clone)]
pub struct PromptInput {
    model: Model<String>,
    textarea_min_height: Px,
    disabled: bool,
    loading: bool,
    clear_on_send: bool,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    test_id_root: Option<Arc<str>>,
    test_id_textarea: Option<Arc<str>>,
    test_id_send: Option<Arc<str>>,
    test_id_stop: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for PromptInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptInput")
            .field("model", &"<model>")
            .field("textarea_min_height", &self.textarea_min_height)
            .field("disabled", &self.disabled)
            .field("loading", &self.loading)
            .field("clear_on_send", &self.clear_on_send)
            .field("on_send", &self.on_send.as_ref().map(|_| "<on_send>"))
            .field("on_stop", &self.on_stop.as_ref().map(|_| "<on_stop>"))
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_textarea", &self.test_id_textarea.as_deref())
            .field("test_id_send", &self.test_id_send.as_deref())
            .field("test_id_stop", &self.test_id_stop.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl PromptInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            textarea_min_height: Px(96.0),
            disabled: false,
            loading: false,
            clear_on_send: true,
            on_send: None,
            on_stop: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn textarea_min_height(mut self, min_height: Px) -> Self {
        self.textarea_min_height = min_height;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    pub fn clear_on_send(mut self, clear_on_send: bool) -> Self {
        self.clear_on_send = clear_on_send;
        self
    }

    pub fn on_send(mut self, on_send: OnActivate) -> Self {
        self.on_send = Some(on_send);
        self
    }

    pub fn on_stop(mut self, on_stop: OnActivate) -> Self {
        self.on_stop = Some(on_stop);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn test_id_textarea(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_textarea = Some(id.into());
        self
    }

    pub fn test_id_send(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_send = Some(id.into());
        self
    }

    pub fn test_id_stop(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_stop = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let current = cx
            .get_model_cloned(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let is_empty = current.trim().is_empty();

        let model = self.model.clone();
        let clear_on_send = self.clear_on_send;
        let on_send = self.on_send.clone();
        let send_activate: OnActivate = Arc::new(move |host, action_cx, reason| {
            let text = host.models_mut().read(&model, Clone::clone).ok();
            let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
            if is_empty {
                return;
            }

            if let Some(on_send) = on_send.as_ref() {
                on_send(host, action_cx, reason);
            }

            if clear_on_send {
                let _ = host.models_mut().update(&model, |v| v.clear());
            }
        });

        let stop_activate = self.on_stop.clone();

        let send_disabled = self.disabled || self.loading || is_empty;
        let stop_disabled = self.disabled || !self.loading;

        let send_button = (!self.loading).then(|| {
            let mut btn = Button::new("Send")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .disabled(send_disabled)
                .on_activate(send_activate);
            if let Some(id) = self.test_id_send.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        });

        let stop_button = (self.loading).then(|| {
            let mut btn = Button::new("Stop")
                .variant(ButtonVariant::Secondary)
                .size(ButtonSize::Sm)
                .disabled(stop_disabled);

            if let Some(on_stop) = stop_activate {
                btn = btn.on_activate(on_stop);
            }
            if let Some(id) = self.test_id_stop.clone() {
                btn = btn.test_id(id);
            }
            btn.into_element(cx)
        });

        let actions = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .justify(Justify::End),
            |_cx| {
                let mut out = Vec::new();
                if let Some(stop_button) = stop_button {
                    out.push(stop_button);
                }
                if let Some(send_button) = send_button {
                    out.push(send_button);
                }
                out
            },
        );

        let mut group = InputGroup::new(self.model)
            .textarea()
            .textarea_min_height(self.textarea_min_height)
            .block_end(vec![actions])
            .block_end_border_top(true)
            .refine_layout(self.layout.w_full());

        if let Some(id) = self.test_id_root {
            group = group.test_id(id);
        }
        if let Some(id) = self.test_id_textarea {
            group = group.control_test_id(id);
        }

        group.into_element(cx)
    }
}
