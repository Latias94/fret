use std::sync::Arc;

use fret_core::{Axis, Edges, ExternalDragKind, Px};
use fret_icons::IconId;
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActivateReason, OnActivate, OnExternalDrag, OnKeyDown};
use fret_ui::element::{AnyElement, ContainerProps, CrossAlign, FlexProps, MainAlign};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{LayoutRefinement, MetricRef, Space};

use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, InputGroup};

use crate::elements::attachments::{
    Attachment, AttachmentData, AttachmentFileData, AttachmentVariant, Attachments,
};
use crate::model::item_key_from_external_id;

#[derive(Debug, Clone)]
pub struct PromptInputController {
    pub text: Model<String>,
    pub attachments: Option<Model<Vec<AttachmentData>>>,
}

#[derive(Debug, Default, Clone)]
struct PromptInputProviderState {
    controller: Option<PromptInputController>,
}

#[derive(Debug, Default, Clone)]
struct PromptInputLocalState {
    controller: Option<PromptInputController>,
}

/// Returns the nearest prompt input controller in scope.
///
/// This mirrors AI Elements `PromptInputProvider` behavior: prefer a local controller (inside a
/// `PromptInput`), falling back to a provider controller when present.
pub fn use_prompt_input_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<PromptInputController> {
    if let Some(local) = cx.inherited_state::<PromptInputLocalState>() {
        if let Some(controller) = local.controller.clone() {
            return Some(controller);
        }
    }
    cx.inherited_state::<PromptInputProviderState>()
        .and_then(|st| st.controller.clone())
}

#[derive(Debug, Clone)]
pub struct PromptInputProvider {
    text: Option<Model<String>>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    initial_input: Arc<str>,
}

impl PromptInputProvider {
    pub fn new() -> Self {
        Self {
            text: None,
            attachments: None,
            initial_input: Arc::<str>::from(""),
        }
    }

    pub fn initial_input(mut self, input: impl Into<Arc<str>>) -> Self {
        self.initial_input = input.into();
        self
    }

    pub fn text_model(mut self, model: Model<String>) -> Self {
        self.text = Some(model);
        self
    }

    pub fn attachments_model(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
        self
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, PromptInputController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let controlled_text = self.text.clone();
        let controlled_attachments = self.attachments.clone();
        let initial_input = self.initial_input.to_string();

        cx.container(Default::default(), move |cx| {
            let text =
                controllable_state::use_controllable_model(cx, controlled_text, || initial_input)
                    .model();
            let attachments = controllable_state::use_controllable_model(
                cx,
                controlled_attachments,
                Vec::<AttachmentData>::new,
            )
            .model();

            let controller = PromptInputController {
                text,
                attachments: Some(attachments),
            };
            cx.with_state(PromptInputProviderState::default, |st| {
                st.controller = Some(controller.clone());
            });

            children(cx, controller)
        })
    }
}

#[derive(Clone)]
pub struct PromptInput {
    model: Option<Model<String>>,
    textarea_min_height: Px,
    textarea_max_height: Option<Px>,
    disabled: bool,
    loading: bool,
    clear_on_send: bool,
    clear_attachments_on_send: bool,
    on_send: Option<OnActivate>,
    on_stop: Option<OnActivate>,
    on_add_attachments: Option<OnActivate>,
    attachments: Option<Model<Vec<AttachmentData>>>,
    test_id_root: Option<Arc<str>>,
    test_id_textarea: Option<Arc<str>>,
    test_id_send: Option<Arc<str>>,
    test_id_stop: Option<Arc<str>>,
    test_id_attachments: Option<Arc<str>>,
    test_id_add_attachments: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for PromptInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptInput")
            .field("model", &self.model.as_ref().map(|_| "<model>"))
            .field("textarea_min_height", &self.textarea_min_height)
            .field("textarea_max_height", &self.textarea_max_height)
            .field("disabled", &self.disabled)
            .field("loading", &self.loading)
            .field("clear_on_send", &self.clear_on_send)
            .field("clear_attachments_on_send", &self.clear_attachments_on_send)
            .field("on_send", &self.on_send.as_ref().map(|_| "<on_send>"))
            .field("on_stop", &self.on_stop.as_ref().map(|_| "<on_stop>"))
            .field(
                "on_add_attachments",
                &self
                    .on_add_attachments
                    .as_ref()
                    .map(|_| "<on_add_attachments>"),
            )
            .field(
                "attachments",
                &self.attachments.as_ref().map(|_| "<attachments>"),
            )
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("test_id_textarea", &self.test_id_textarea.as_deref())
            .field("test_id_send", &self.test_id_send.as_deref())
            .field("test_id_stop", &self.test_id_stop.as_deref())
            .field("test_id_attachments", &self.test_id_attachments.as_deref())
            .field(
                "test_id_add_attachments",
                &self.test_id_add_attachments.as_deref(),
            )
            .field("layout", &self.layout)
            .finish()
    }
}

impl PromptInput {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model: Some(model),
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            attachments: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            test_id_attachments: None,
            test_id_add_attachments: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn new_uncontrolled() -> Self {
        Self {
            model: None,
            textarea_min_height: Px(96.0),
            textarea_max_height: None,
            disabled: false,
            loading: false,
            clear_on_send: true,
            clear_attachments_on_send: true,
            on_send: None,
            on_stop: None,
            on_add_attachments: None,
            attachments: None,
            test_id_root: None,
            test_id_textarea: None,
            test_id_send: None,
            test_id_stop: None,
            test_id_attachments: None,
            test_id_add_attachments: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn textarea_min_height(mut self, min_height: Px) -> Self {
        self.textarea_min_height = min_height;
        self
    }

    pub fn textarea_max_height(mut self, max_height: Px) -> Self {
        self.textarea_max_height = Some(max_height);
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

    pub fn clear_attachments_on_send(mut self, clear_attachments_on_send: bool) -> Self {
        self.clear_attachments_on_send = clear_attachments_on_send;
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

    /// Add an "add attachments" affordance aligned with AI Elements `PromptInputActionAddAttachments`.
    ///
    /// Effects (file dialog, file IO, clipboard files) remain app-owned; this action hook only
    /// emits an intent.
    pub fn on_add_attachments(mut self, on_add_attachments: OnActivate) -> Self {
        self.on_add_attachments = Some(on_add_attachments);
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

    pub fn attachments(mut self, model: Model<Vec<AttachmentData>>) -> Self {
        self.attachments = Some(model);
        self
    }

    pub fn test_id_attachments(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_attachments = Some(id.into());
        self
    }

    pub fn test_id_add_attachments(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_add_attachments = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let provider = use_prompt_input_controller(cx);
        let text_model = controllable_state::use_controllable_model(
            cx,
            self.model
                .clone()
                .or_else(|| provider.as_ref().map(|c| c.text.clone())),
            String::new,
        )
        .model();

        let attachments_model = self
            .attachments
            .clone()
            .or_else(|| provider.as_ref().and_then(|c| c.attachments.clone()));

        cx.with_state(PromptInputLocalState::default, |st| {
            st.controller = Some(PromptInputController {
                text: text_model.clone(),
                attachments: attachments_model.clone(),
            });
        });

        let current = cx
            .get_model_cloned(&text_model, Invalidation::Layout)
            .unwrap_or_default();
        let is_empty = current.trim().is_empty();

        let attachments = attachments_model.as_ref().and_then(|m| {
            cx.get_model_cloned(m, Invalidation::Layout)
                .or_else(|| Some(Vec::new()))
        });
        let attachments_len = attachments.as_ref().map(|v| v.len()).unwrap_or(0);

        let text_model_for_handlers = text_model.clone();
        let clear_on_send = self.clear_on_send;
        let on_send = self.on_send.clone();
        let attachments_model_for_send = attachments_model.clone();
        let clear_attachments_on_send = self.clear_attachments_on_send;
        let send_activate: OnActivate = Arc::new(move |host, action_cx, reason| {
            let text = host
                .models_mut()
                .read(&text_model_for_handlers, Clone::clone)
                .ok();
            let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);

            let attachments_len = attachments_model_for_send
                .as_ref()
                .and_then(|m| host.models_mut().read(m, |v| v.len()).ok())
                .unwrap_or(0);

            if is_empty && attachments_len == 0 {
                return;
            }

            if let Some(on_send) = on_send.as_ref() {
                on_send(host, action_cx, reason);
            }

            if clear_on_send {
                let _ = host
                    .models_mut()
                    .update(&text_model_for_handlers, |v| v.clear());
            }
            if clear_attachments_on_send {
                if let Some(attachments_model) = attachments_model_for_send.as_ref() {
                    let _ = host.models_mut().update(attachments_model, |v| v.clear());
                }
            }
        });

        let stop_activate = self.on_stop.clone();

        let send_disabled = self.disabled || self.loading || (is_empty && attachments_len == 0);
        let stop_disabled = self.disabled || !self.loading;

        let textarea_min_height = if self.textarea_min_height == Px(96.0) {
            theme
                .metric_by_key("fret.ai.prompt_input.min_height")
                .unwrap_or(self.textarea_min_height)
        } else {
            self.textarea_min_height
        };

        let textarea_max_height = self
            .textarea_max_height
            .or_else(|| theme.metric_by_key("fret.ai.prompt_input.max_height"));

        let send_activate_for_button = send_activate.clone();
        let send_button = (!self.loading).then(|| {
            let mut btn = Button::new("Send")
                .variant(ButtonVariant::Default)
                .size(ButtonSize::Sm)
                .disabled(send_disabled)
                .on_activate(send_activate_for_button);
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

        let add_attachments_button = self.on_add_attachments.clone().map(|on_add| {
            let add_disabled = self.disabled || self.loading;
            let mut btn = Button::new("Add attachments")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconSm)
                .disabled(add_disabled)
                .children([decl_icon::icon(cx, IconId::new("lucide.plus"))])
                .on_activate(on_add);

            let test_id = self.test_id_add_attachments.clone().or_else(|| {
                self.test_id_root
                    .clone()
                    .map(|id| Arc::<str>::from(format!("{id}-add-attachments")))
            });
            if let Some(id) = test_id {
                btn = btn.test_id(id);
            }

            btn.into_element(cx)
        });

        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let actions = cx.flex(
            FlexProps {
                layout: decl_style::layout_style(&theme, LayoutRefinement::default().w_full()),
                direction: Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::End,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |cx| {
                let mut out = Vec::new();
                if let Some(add_button) = add_attachments_button {
                    out.push(cx.container(
                        ContainerProps {
                            layout: decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().mr_auto(),
                            ),
                            ..Default::default()
                        },
                        move |_cx| vec![add_button],
                    ));
                }
                if let Some(stop_button) = stop_button {
                    out.push(stop_button);
                }
                if let Some(send_button) = send_button {
                    out.push(send_button);
                }
                out
            },
        );

        let attachments_el = attachments.and_then(|items| {
            if items.is_empty() {
                return None;
            }

            let attachments_model = attachments_model.clone();
            let on_remove = attachments_model.map(|attachments_model| {
                let model = attachments_model.clone();
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost,
                          _action_cx: fret_ui::action::ActionCx,
                          id: Arc<str>| {
                        let _ = host.models_mut().update(&model, |v| {
                            v.retain(|item| item.id().as_ref() != id.as_ref());
                        });
                    },
                )
            });

            let row_test_id = self.test_id_attachments.clone().or_else(|| {
                self.test_id_root
                    .clone()
                    .map(|id| Arc::<str>::from(format!("{id}-attachments")))
            });

            let mut children = Vec::new();
            for item in items {
                let item_id = item.id().clone();
                let key = item_key_from_external_id(item_id.as_ref());

                let item_test_id = row_test_id
                    .as_deref()
                    .map(|root| Arc::<str>::from(format!("{root}-item-{item_id}")));
                let remove_test_id = item_test_id
                    .as_deref()
                    .map(|root| Arc::<str>::from(format!("{root}-remove")));

                let on_remove = on_remove.clone();
                let el = cx.keyed(key, move |cx| {
                    let mut chip = Attachment::new(item.clone()).variant(AttachmentVariant::Inline);
                    if let Some(on_remove) = on_remove.clone() {
                        chip = chip.on_remove(on_remove);
                    }
                    if let Some(id) = item_test_id.clone() {
                        chip = chip.test_id(id);
                    }
                    if let Some(id) = remove_test_id.clone() {
                        chip = chip.remove_test_id(id);
                    }
                    chip.into_element(cx)
                });
                children.push(el);
            }

            let mut row = Attachments::new(children).variant(AttachmentVariant::Inline);
            if let Some(id) = row_test_id {
                row = row.test_id(id);
            }
            Some(row.into_element(cx))
        });

        let control_key_handler: OnKeyDown = {
            let text_model = text_model.clone();
            let attachments_model = attachments_model.clone();
            let disabled = self.disabled;
            let loading = self.loading;
            let send_activate = send_activate.clone();

            Arc::new(move |host, action_cx, down| {
                if disabled {
                    return false;
                }

                match down.key {
                    fret_core::KeyCode::Enter => {
                        if loading || down.repeat {
                            return false;
                        }
                        if down.modifiers.shift {
                            return false;
                        }

                        let text = host.models_mut().read(&text_model, Clone::clone).ok();
                        let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
                        let attachments_len = attachments_model
                            .as_ref()
                            .and_then(|m| host.models_mut().read(m, |v| v.len()).ok())
                            .unwrap_or(0);
                        if is_empty && attachments_len == 0 {
                            return false;
                        }

                        send_activate(host, action_cx, ActivateReason::Keyboard);
                        host.notify(action_cx);
                        true
                    }
                    fret_core::KeyCode::Backspace => {
                        let Some(attachments_model) = attachments_model.as_ref() else {
                            return false;
                        };
                        let attachments_len = host
                            .models_mut()
                            .read(attachments_model, |v| v.len())
                            .ok()
                            .unwrap_or(0);
                        if attachments_len == 0 {
                            return false;
                        }

                        let text = host.models_mut().read(&text_model, Clone::clone).ok();
                        let is_empty = text.as_deref().map(|v| v.trim().is_empty()).unwrap_or(true);
                        if !is_empty {
                            return false;
                        }

                        let _ = host.models_mut().update(attachments_model, |v| {
                            let _ = v.pop();
                        });
                        host.notify(action_cx);
                        true
                    }
                    _ => false,
                }
            })
        };

        let prompt_empty_state_marker = self.test_id_root.clone().map(|root| {
            let suffix = if is_empty {
                "prompt-empty"
            } else {
                "prompt-nonempty"
            };
            let id = Arc::<str>::from(format!("{root}-{suffix}"));
            cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Text,
                    test_id: Some(id),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.container(
                        fret_ui::element::ContainerProps {
                            layout: fret_ui::element::LayoutStyle {
                                size: fret_ui::element::SizeStyle {
                                    width: fret_ui::element::Length::Px(Px(0.0)),
                                    height: fret_ui::element::Length::Px(Px(0.0)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |_cx| Vec::new(),
                    )]
                },
            )
        });

        let mut group = InputGroup::new(text_model.clone())
            .textarea()
            .textarea_min_height(textarea_min_height)
            .control_on_key_down(control_key_handler)
            .block_end({
                let mut out = vec![actions];
                if let Some(marker) = prompt_empty_state_marker {
                    out.push(marker);
                }
                out
            })
            .block_end_border_top(true)
            .refine_layout(self.layout.w_full());

        if let Some(max_h) = textarea_max_height {
            group = group.textarea_max_height(max_h);
        }

        if let Some(attachments_el) = attachments_el {
            group = group
                .block_start(vec![attachments_el])
                .block_start_border_bottom(true);
        }

        if let Some(id) = self.test_id_root {
            group = group.test_id(id);
        }
        if let Some(id) = self.test_id_textarea {
            group = group.control_test_id(id);
        }

        let content = group.into_element(cx);

        let drop_handler: Option<OnExternalDrag> =
            attachments_model
                .clone()
                .map(|attachments| -> OnExternalDrag {
                    let disabled = self.disabled;
                    let loading = self.loading;
                    Arc::new(
                        move |host: &mut dyn fret_ui::action::UiActionHost,
                              action_cx: fret_ui::action::ActionCx,
                              e: &fret_core::ExternalDragEvent| {
                            if disabled || loading {
                                return false;
                            }

                            let ExternalDragKind::DropFiles(files) = &e.kind else {
                                return false;
                            };
                            if files.files.is_empty() {
                                host.push_effect(Effect::ExternalDropRelease {
                                    token: files.token,
                                });
                                return true;
                            }

                            let token_id = files.token.0;
                            let dropped: Vec<AttachmentData> = files
                                .files
                                .iter()
                                .enumerate()
                                .map(|(i, f)| {
                                    let id = Arc::<str>::from(format!("drop-{token_id}-{i}"));
                                    AttachmentData::File(
                                        AttachmentFileData::new(id).filename(f.name.clone()),
                                    )
                                })
                                .collect();

                            let _ = host.models_mut().update(&attachments, |v| {
                                for item in &dropped {
                                    let id = item.id();
                                    if v.iter()
                                        .any(|existing| existing.id().as_ref() == id.as_ref())
                                    {
                                        continue;
                                    }
                                    v.push(item.clone());
                                }
                            });

                            host.push_effect(Effect::ExternalDropRelease { token: files.token });
                            host.notify(action_cx);
                            true
                        },
                    )
                });

        if let Some(on_drop) = drop_handler {
            cx.external_drag_region(fret_ui::element::ExternalDragRegionProps::default(), |cx| {
                cx.external_drag_region_on_external_drag(on_drop);
                vec![content]
            })
        } else {
            content
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, Event, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
        PathMetrics, PathService, PathStyle, Point, PointerEvent, PointerId, PointerType, Px, Rect,
        Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_ui::UiTree;
    use fret_ui::declarative::render_root;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn prompt_input_provider_text_model_receives_text_input() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let controlled_text = app.models_mut().insert(String::new());
        let controlled_attachments = app.models_mut().insert(Vec::<AttachmentData>::new());

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );
        let mut services = FakeServices::default();

        let root = render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "prompt-input-provider-test",
            |cx| {
                vec![
                    PromptInputProvider::new()
                        .text_model(controlled_text.clone())
                        .attachments_model(controlled_attachments.clone())
                        .into_element_with_children(cx, |cx, _controller| {
                            vec![
                                PromptInput::new_uncontrolled()
                                    .test_id_root("pi-root")
                                    .test_id_textarea("pi-textarea")
                                    .refine_layout(LayoutRefinement::default().w_full())
                                    .into_element(cx),
                            ]
                        }),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let click_pos = Point::new(Px(20.0), Px(20.0));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Down {
                pointer_id: PointerId(0),
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::Pointer(PointerEvent::Up {
                pointer_id: PointerId(0),
                position: click_pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );
        ui.dispatch_event(&mut app, &mut services, &Event::TextInput("a".to_string()));

        let value = app
            .models_mut()
            .read(&controlled_text, Clone::clone)
            .unwrap();
        assert_eq!(value, "a");
    }
}
