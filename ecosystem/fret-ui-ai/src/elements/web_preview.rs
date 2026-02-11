//! AI Elements-aligned `WebPreview` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/web-preview.tsx`.

use std::sync::Arc;

use fret_core::{FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsDecoration, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Radius, Size, Space,
};
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, Collapsible, CollapsibleContent, CollapsibleTrigger, Input,
    OnInputSubmit, Tooltip, TooltipContent, TooltipTrigger,
};

#[cfg(feature = "webview")]
use fret_webview::{
    WebViewId, WebViewRequest, WebViewSurfaceRegistration, webview_push_request,
    webview_register_surface,
};

#[derive(Debug, Default, Clone)]
struct WebPreviewProviderState {
    controller: Option<WebPreviewController>,
}

#[derive(Clone)]
pub struct WebPreviewController {
    pub url: Model<String>,
    pub url_draft: Model<String>,
    pub console_open: Model<bool>,
    pub disabled: bool,
    pub on_url_change: Option<OnWebPreviewUrlChange>,
    #[cfg(feature = "webview")]
    pub backend: Option<WebPreviewBackendController>,
}

impl std::fmt::Debug for WebPreviewController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebPreviewController")
            .field("url", &"<model>")
            .field("url_draft", &"<model>")
            .field("console_open", &"<model>")
            .field("disabled", &self.disabled)
            .field("has_on_url_change", &self.on_url_change.is_some())
            .field("has_backend", &{
                #[cfg(feature = "webview")]
                {
                    self.backend.is_some()
                }
                #[cfg(not(feature = "webview"))]
                {
                    false
                }
            })
            .finish()
    }
}

pub type OnWebPreviewUrlChange =
    Arc<dyn Fn(&mut dyn UiFocusActionHost, ActionCx, Arc<str>) + 'static>;

#[cfg(feature = "webview")]
#[derive(Clone)]
pub struct WebPreviewBackendController {
    pub id: WebViewId,
    pub surface_test_id: Arc<str>,
}

#[cfg(feature = "webview")]
impl std::fmt::Debug for WebPreviewBackendController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebPreviewBackendController")
            .field("id", &self.id)
            .field("surface_test_id", &self.surface_test_id.as_ref())
            .finish()
    }
}

pub fn use_web_preview_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<WebPreviewController> {
    cx.inherited_state::<WebPreviewProviderState>()
        .and_then(|st| st.controller.clone())
}

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        fret_ui::element::InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

fn base_chrome(theme: &Theme) -> ChromeRefinement {
    ChromeRefinement::default()
        .rounded(Radius::Lg)
        .border_1()
        .bg(ColorRef::Color(theme.color_required("card")))
        .border_color(ColorRef::Color(theme.color_required("border")))
}

/// Root provider aligned with AI Elements `WebPreview`.
#[derive(Clone)]
pub struct WebPreview {
    url: Option<Model<String>>,
    default_url: Arc<str>,
    disabled: bool,
    on_url_change: Option<OnWebPreviewUrlChange>,
    #[cfg(feature = "webview")]
    backend: Option<WebPreviewBackendController>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl WebPreview {
    pub fn new() -> Self {
        Self {
            url: None,
            default_url: Arc::<str>::from(""),
            disabled: false,
            on_url_change: None,
            #[cfg(feature = "webview")]
            backend: None,
            test_id_root: None,
            layout: LayoutRefinement::default()
                .w_full()
                .h_full()
                .min_w_0()
                .min_h_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn url_model(mut self, model: Model<String>) -> Self {
        self.url = Some(model);
        self
    }

    pub fn default_url(mut self, default_url: impl Into<Arc<str>>) -> Self {
        self.default_url = default_url.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_url_change(mut self, cb: OnWebPreviewUrlChange) -> Self {
        self.on_url_change = Some(cb);
        self
    }

    #[cfg(feature = "webview")]
    pub fn backend(mut self, backend: WebPreviewBackendController) -> Self {
        self.backend = Some(backend);
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element_with_children<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>, WebPreviewController) -> Vec<AnyElement>,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let layout = self.layout;
        let chrome = base_chrome(&theme).merge(self.chrome);
        let controlled_url = self.url.clone();
        let default_url = self.default_url.clone();
        let disabled = self.disabled;
        let on_url_change = self.on_url_change.clone();
        #[cfg(feature = "webview")]
        let backend = self.backend.clone();
        let test_id_root = self.test_id_root.clone();

        let root = cx.container(
            decl_style::container_props(&theme, chrome, layout),
            move |cx| {
                let url_model =
                    controllable_state::use_controllable_model(cx, controlled_url.clone(), || {
                        default_url.to_string()
                    })
                    .model();

                let url_now = cx
                    .get_model_cloned(&url_model, Invalidation::Layout)
                    .unwrap_or_default();

                let url_draft = cx.with_state(|| None::<Model<String>>, |st| st.clone());
                let url_draft = if let Some(model) = url_draft {
                    model
                } else {
                    let model = cx.app.models_mut().insert(url_now.clone());
                    cx.with_state(|| None::<Model<String>>, |st| *st = Some(model.clone()));
                    model
                };

                let console_open = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
                let console_open = if let Some(model) = console_open {
                    model
                } else {
                    let model = cx.app.models_mut().insert(false);
                    cx.with_state(|| None::<Model<bool>>, |st| *st = Some(model.clone()));
                    model
                };

                let controller = WebPreviewController {
                    url: url_model,
                    url_draft,
                    console_open,
                    disabled,
                    on_url_change: on_url_change.clone(),
                    #[cfg(feature = "webview")]
                    backend: backend.clone(),
                };

                #[cfg(feature = "webview")]
                if let Some(backend) = backend.clone() {
                    #[derive(Default)]
                    struct BackendInit {
                        created: bool,
                        last_loaded_url: String,
                    }

                    let needs_create = cx.with_state(BackendInit::default, |st| !st.created);
                    if needs_create {
                        let url_now_string = url_now.clone();
                        let url_now: Arc<str> = Arc::from(url_now_string.clone());
                        webview_push_request(
                            cx.app,
                            WebViewRequest::Create {
                                id: backend.id,
                                window: cx.window,
                                initial_url: url_now,
                            },
                        );
                        cx.with_state(BackendInit::default, |st| {
                            st.created = true;
                            st.last_loaded_url = url_now_string;
                        });
                    } else {
                        let needs_load =
                            cx.with_state(BackendInit::default, |st| st.last_loaded_url != url_now);
                        if needs_load {
                            let next: Arc<str> = Arc::from(url_now.clone());
                            webview_push_request(
                                cx.app,
                                WebViewRequest::LoadUrl {
                                    id: backend.id,
                                    url: next,
                                },
                            );
                            cx.with_state(BackendInit::default, |st| st.last_loaded_url = url_now);
                        }
                    }
                }

                cx.with_state(WebPreviewProviderState::default, |st| {
                    st.controller = Some(controller.clone());
                });

                let body = children(cx, controller);

                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_full()
                                .min_w_0()
                                .min_h_0(),
                        )
                        .gap(Space::N0)
                        .items(Items::Stretch),
                    move |_cx| body,
                );

                let body = if let Some(test_id) = test_id_root.clone() {
                    cx.semantics(
                        SemanticsProps {
                            role: SemanticsRole::Group,
                            test_id: Some(test_id),
                            ..Default::default()
                        },
                        move |_cx| vec![body],
                    )
                } else {
                    body
                };

                vec![body]
            },
        );

        root
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.into_element_with_children(cx, |cx, _controller| vec![hidden(cx)])
    }
}

impl Default for WebPreview {
    fn default() -> Self {
        Self::new()
    }
}

/// Navigation bar aligned with AI Elements `WebPreviewNavigation`.
#[derive(Debug, Clone)]
pub struct WebPreviewNavigation {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl WebPreviewNavigation {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(self.layout)
                .gap(Space::N1)
                .items_center(),
            move |_cx| self.children,
        );

        let bar = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N2).merge(self.chrome),
                LayoutRefinement::default(),
            ),
            move |_cx| vec![row],
        );

        let separator = fret_ui_shadcn::separator(cx);
        let el = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0),
            move |_cx| vec![bar, separator],
        );

        let Some(test_id) = self.test_id else {
            return el;
        };
        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Navigation button with tooltip aligned with AI Elements `WebPreviewNavigationButton`.
#[derive(Clone)]
pub struct WebPreviewNavigationButton {
    tooltip: Option<Arc<str>>,
    disabled: bool,
    on_activate: Option<fret_ui::action::OnActivate>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for WebPreviewNavigationButton {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebPreviewNavigationButton")
            .field("tooltip", &self.tooltip.as_deref())
            .field("disabled", &self.disabled)
            .field("has_on_activate", &self.on_activate.is_some())
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl WebPreviewNavigationButton {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            tooltip: None,
            disabled: false,
            on_activate: None,
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn tooltip(mut self, tooltip: impl Into<Arc<str>>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_activate(mut self, on_activate: fret_ui::action::OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut button = Button::new("")
            .children(self.children)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .disabled(self.disabled)
            .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
            .refine_style(
                ChromeRefinement::default()
                    .p(Space::N0)
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground"))),
            );

        if let Some(on_activate) = self.on_activate {
            button = button.on_activate(on_activate);
        }
        if let Some(test_id) = self.test_id.clone() {
            button = button.test_id(test_id);
        }

        let mut button = button.into_element(cx);

        let Some(tooltip) = self.tooltip else {
            return button;
        };

        button = button.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Button)
                .label(tooltip.clone()),
        );

        let trigger = TooltipTrigger::new(button).into_element(cx);
        let content = TooltipContent::new([TooltipContent::text(cx, tooltip)]).into_element(cx);
        Tooltip::new(trigger, content).into_element(cx)
    }
}

/// URL input aligned with AI Elements `WebPreviewUrl`.
#[derive(Debug, Clone)]
pub struct WebPreviewUrl {
    placeholder: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Default for WebPreviewUrl {
    fn default() -> Self {
        Self {
            placeholder: Arc::<str>::from("Enter URL..."),
            test_id: None,
            layout: LayoutRefinement::default().flex_1().min_w_0(),
        }
    }
}

impl WebPreviewUrl {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn placeholder(mut self, placeholder: impl Into<Arc<str>>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_web_preview_controller(cx) else {
            return hidden(cx);
        };

        let url_now = cx
            .get_model_cloned(&controller.url, Invalidation::Layout)
            .unwrap_or_default();

        #[derive(Default)]
        struct SyncState {
            prev_url: String,
        }

        let needs_sync = cx.with_state(SyncState::default, |st| st.prev_url != url_now);
        if needs_sync {
            let _ = cx
                .app
                .models_mut()
                .update(&controller.url_draft, |v| *v = url_now.clone());
            cx.with_state(SyncState::default, |st| st.prev_url = url_now.clone());
        }

        let draft = controller.url_draft.clone();
        let url = controller.url.clone();
        let on_url_change = controller.on_url_change.clone();
        let disabled = controller.disabled;

        let on_submit: OnInputSubmit = Arc::new(
            move |host: &mut dyn UiFocusActionHost, action_cx: ActionCx| {
                let next = host.models_mut().get_cloned(&draft).unwrap_or_default();
                let _ = host.models_mut().update(&url, |v| *v = next.clone());
                if let Some(cb) = on_url_change.clone() {
                    cb(host, action_cx, Arc::<str>::from(next));
                }
                host.request_redraw(action_cx.window);
            },
        );

        let input = Input::new(controller.url_draft)
            .placeholder(self.placeholder)
            .size(Size::Small)
            .disabled(disabled)
            .on_submit(on_submit)
            .refine_layout(self.layout)
            .into_element(cx);

        let Some(test_id) = self.test_id else {
            return input;
        };
        input.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::TextField)
                .test_id(test_id),
        )
    }
}

/// Body container aligned with AI Elements `WebPreviewBody`.
#[derive(Debug, Clone)]
pub struct WebPreviewBody {
    loading: Option<AnyElement>,
    child: Option<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl Default for WebPreviewBody {
    fn default() -> Self {
        Self {
            loading: None,
            child: None,
            test_id: None,
            layout: LayoutRefinement::default()
                .w_full()
                .min_w_0()
                .flex_1()
                .min_h_0(),
        }
    }
}

impl WebPreviewBody {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn loading(mut self, loading: AnyElement) -> Self {
        self.loading = Some(loading);
        self
    }

    pub fn child(mut self, child: AnyElement) -> Self {
        self.child = Some(child);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_web_preview_controller(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let url_now = cx
            .get_model_cloned(&controller.url, Invalidation::Layout)
            .unwrap_or_default();

        let placeholder = if let Some(child) = self.child {
            child
        } else {
            let text = Arc::<str>::from(format!("Preview: {}", url_now.trim()));
            cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text,
                style: Some(TextStyle {
                    font: FontId::default(),
                    size: theme.metric_required("component.text.sm_px"),
                    weight: FontWeight::NORMAL,
                    slant: Default::default(),
                    line_height: Some(theme.metric_required("component.text.sm_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(theme.color_required("muted-foreground")),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
            })
        };

        let mut children = vec![placeholder];
        if let Some(loading) = self.loading {
            children.push(loading);
        }

        let el = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N4),
                self.layout,
            ),
            move |_cx| children,
        );

        #[cfg(feature = "webview")]
        let (test_id, role) = {
            let test_id = self.test_id.clone().or_else(|| {
                controller
                    .backend
                    .as_ref()
                    .map(|b| b.surface_test_id.clone())
            });
            let role = if controller.backend.is_some() {
                SemanticsRole::Viewport
            } else {
                SemanticsRole::Group
            };
            (test_id, role)
        };
        #[cfg(not(feature = "webview"))]
        let (test_id, role) = (self.test_id.clone(), SemanticsRole::Group);

        let Some(test_id) = test_id else {
            return el;
        };
        #[cfg(feature = "webview")]
        if let Some(backend) = controller.backend.as_ref() {
            webview_register_surface(
                cx.app,
                WebViewSurfaceRegistration::new(backend.id, cx.window, test_id.clone())
                    .visible(true),
            );
        }
        el.attach_semantics(SemanticsDecoration::default().role(role).test_id(test_id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebPreviewConsoleLogLevel {
    Log,
    Warn,
    Error,
}

#[derive(Debug, Clone)]
pub struct WebPreviewConsoleLog {
    pub level: WebPreviewConsoleLogLevel,
    pub timestamp: Arc<str>,
    pub message: Arc<str>,
}

impl WebPreviewConsoleLog {
    pub fn new(level: WebPreviewConsoleLogLevel, message: impl Into<Arc<str>>) -> Self {
        Self {
            level,
            timestamp: Arc::<str>::from(""),
            message: message.into(),
        }
    }

    pub fn timestamp(mut self, timestamp: impl Into<Arc<str>>) -> Self {
        self.timestamp = timestamp.into();
        self
    }
}

/// Console disclosure aligned with AI Elements `WebPreviewConsole`.
#[derive(Debug, Clone)]
pub struct WebPreviewConsole {
    logs: Arc<[WebPreviewConsoleLog]>,
    children: Vec<AnyElement>,
    test_id_trigger: Option<Arc<str>>,
    test_id_content: Option<Arc<str>>,
    test_id_marker: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Default for WebPreviewConsole {
    fn default() -> Self {
        Self {
            logs: Arc::from([]),
            children: Vec::new(),
            test_id_trigger: None,
            test_id_content: None,
            test_id_marker: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }
}

impl WebPreviewConsole {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn logs(mut self, logs: Arc<[WebPreviewConsoleLog]>) -> Self {
        self.logs = logs;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
        self
    }

    pub fn test_id_trigger(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(id.into());
        self
    }

    pub fn test_id_content(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_content = Some(id.into());
        self
    }

    pub fn test_id_marker(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_marker = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_web_preview_controller(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let open_now = cx
            .get_model_copied(&controller.console_open, Invalidation::Layout)
            .unwrap_or(false);

        let chevron = decl_icon::icon_with(
            cx,
            if open_now {
                fret_icons::ids::ui::CHEVRON_UP
            } else {
                fret_icons::ids::ui::CHEVRON_DOWN
            },
            Some(Px(16.0)),
            Some(ColorRef::Color(theme.color_required("muted-foreground"))),
        );

        let label = cx.text("Console");
        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .justify(Justify::Between)
                .items_center(),
            move |_cx| vec![label, chevron],
        );

        let button = Button::new("Console")
            .children([row])
            .variant(ButtonVariant::Ghost)
            .refine_layout(LayoutRefinement::default().w_full().min_w_0())
            .refine_style(ChromeRefinement::default().p(Space::N4))
            .into_element(cx);

        let trigger = {
            let mut trigger =
                CollapsibleTrigger::new(controller.console_open.clone(), vec![button])
                    .a11y_label("Toggle console");
            let trigger = trigger.into_element(cx, open_now);
            if let Some(test_id) = self.test_id_trigger {
                trigger.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Button)
                        .test_id(test_id),
                )
            } else {
                trigger
            }
        };

        let logs = self.logs.clone();
        let children = self.children;
        let marker_id = self.test_id_marker.clone();
        let content = CollapsibleContent::new([{
            let mut rows: Vec<AnyElement> = Vec::new();
            if logs.is_empty() {
                rows.push(
                    cx.text("No console output")
                        .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Text)),
                );
            } else {
                for log in logs.iter() {
                    let fg = match log.level {
                        WebPreviewConsoleLogLevel::Error => theme.color_required("destructive"),
                        WebPreviewConsoleLogLevel::Warn => fret_core::Color {
                            r: 0.792,
                            g: 0.541,
                            b: 0.016,
                            a: 1.0,
                        },
                        WebPreviewConsoleLogLevel::Log => theme.color_required("foreground"),
                    };

                    let ts = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: log.timestamp.clone(),
                        style: Some(TextStyle {
                            font: FontId::monospace(),
                            size: theme.metric_required("component.text.xs_px"),
                            weight: FontWeight::NORMAL,
                            slant: Default::default(),
                            line_height: Some(
                                theme.metric_required("component.text.xs_line_height"),
                            ),
                            letter_spacing_em: None,
                        }),
                        color: Some(theme.color_required("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                    });

                    let msg = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: log.message.clone(),
                        style: Some(TextStyle {
                            font: FontId::monospace(),
                            size: theme.metric_required("component.text.xs_px"),
                            weight: FontWeight::NORMAL,
                            slant: Default::default(),
                            line_height: Some(
                                theme.metric_required("component.text.xs_line_height"),
                            ),
                            letter_spacing_em: None,
                        }),
                        color: Some(fg),
                        wrap: TextWrap::Word,
                        overflow: TextOverflow::Clip,
                    });

                    let row = stack::hstack(
                        cx,
                        stack::HStackProps::default()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N2)
                            .items_center(),
                        move |_cx| vec![ts, msg],
                    );
                    rows.push(row);
                }
            }

            rows.extend(children);

            if let Some(marker_id) = marker_id {
                rows.push(
                    cx.text("").attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Generic)
                            .test_id(marker_id),
                    ),
                );
            }

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N1),
                move |_cx| rows,
            )
        }])
        .refine_style(ChromeRefinement::default().px(Space::N4).pb(Space::N4))
        .into_element(cx);

        let content = if let Some(test_id) = self.test_id_content {
            content.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            )
        } else {
            content
        };

        let root = Collapsible::new(controller.console_open)
            .refine_layout(self.layout)
            .refine_style(
                ChromeRefinement::default()
                    .border_color(ColorRef::Color(theme.color_required("border")))
                    .bg(ColorRef::Token {
                        key: "muted",
                        fallback: fret_ui_kit::ColorFallback::ThemeSurfaceBackground,
                    })
                    .merge(self.chrome),
            )
            .into_element(cx, move |_cx, _is_open| trigger, move |_cx| content);

        root
    }
}
