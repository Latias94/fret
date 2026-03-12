//! AI Elements-aligned `WebPreview` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/web-preview.tsx`.

use std::sync::Arc;

use fret_core::{Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiFocusActionHost};
use fret_ui::element::{
    AnyElement, LayoutStyle, PressableProps, SemanticsDecoration, SemanticsProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::chrome::control_chrome_pressable_with_id_props;
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, Items, Justify, LayoutRefinement, Radius, Size,
    Space, ui,
};
use fret_ui_shadcn::{
    Button, ButtonSize, ButtonVariant, Collapsible, CollapsibleContent, Input, OnInputSubmit,
    Tooltip, TooltipContent, TooltipTrigger,
};

#[cfg(feature = "webview")]
use fret_webview::{
    WebViewId, WebViewRequest, WebViewSurfaceRegistration, webview_clear_console,
    webview_console_entries, webview_push_request, webview_register_surface_tracked,
    webview_runtime_state,
};

#[derive(Clone)]
pub struct WebPreviewController {
    pub url: Model<String>,
    pub url_draft: Model<String>,
    pub console_open: Model<bool>,
    pub disabled: bool,
    pub on_url_change: Option<OnWebPreviewUrlChange>,
    #[cfg(feature = "webview")]
    pub nav_intent: Model<Option<WebPreviewBackendAction>>,
    #[cfg(feature = "webview")]
    pub console_clear_intent: Model<bool>,
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
            .field("nav_intent", &{
                #[cfg(feature = "webview")]
                {
                    "<model>"
                }
                #[cfg(not(feature = "webview"))]
                {
                    "<none>"
                }
            })
            .field("console_clear_intent", &{
                #[cfg(feature = "webview")]
                {
                    "<model>"
                }
                #[cfg(not(feature = "webview"))]
                {
                    "<none>"
                }
            })
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebPreviewBackendAction {
    GoBack,
    GoForward,
    Reload,
}

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
    cx.provided::<WebPreviewController>().cloned()
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
        .bg(ColorRef::Color(theme.color_token("card")))
        .border_color(ColorRef::Color(theme.color_token("border")))
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

                #[cfg(feature = "webview")]
                let nav_intent = {
                    #[derive(Default)]
                    struct NavIntentState {
                        model: Option<Model<Option<WebPreviewBackendAction>>>,
                    }

                    let existing = cx.with_state(NavIntentState::default, |st| st.model.clone());
                    if let Some(existing) = existing {
                        existing
                    } else {
                        let model = cx.app.models_mut().insert(None::<WebPreviewBackendAction>);
                        cx.with_state(NavIntentState::default, |st| {
                            st.model = Some(model.clone());
                        });
                        model
                    }
                };

                #[cfg(feature = "webview")]
                let console_clear_intent = {
                    #[derive(Default)]
                    struct ConsoleClearIntentState {
                        model: Option<Model<bool>>,
                    }

                    let existing =
                        cx.with_state(ConsoleClearIntentState::default, |st| st.model.clone());
                    if let Some(existing) = existing {
                        existing
                    } else {
                        let model = cx.app.models_mut().insert(false);
                        cx.with_state(ConsoleClearIntentState::default, |st| {
                            st.model = Some(model.clone());
                        });
                        model
                    }
                };

                let controller = WebPreviewController {
                    url: url_model.clone(),
                    url_draft: url_draft.clone(),
                    console_open,
                    disabled,
                    on_url_change: on_url_change.clone(),
                    #[cfg(feature = "webview")]
                    nav_intent: nav_intent.clone(),
                    #[cfg(feature = "webview")]
                    console_clear_intent: console_clear_intent.clone(),
                    #[cfg(feature = "webview")]
                    backend: backend.clone(),
                };

                #[cfg(feature = "webview")]
                {
                    #[derive(Default)]
                    struct BackendInit {
                        created: bool,
                        last_loaded_url: String,
                        last_backend_id: Option<WebViewId>,
                    }

                    let current_backend = backend.clone();
                    let prev_backend_id =
                        cx.with_state(BackendInit::default, |st| st.last_backend_id);

                    match current_backend {
                        None => {
                            if let Some(prev) = prev_backend_id {
                                webview_push_request(cx.app, WebViewRequest::Destroy { id: prev });
                            }
                            cx.with_state(BackendInit::default, |st| *st = BackendInit::default());
                        }
                        Some(backend) => {
                            if prev_backend_id.is_some_and(|prev| prev != backend.id) {
                                if let Some(prev) = prev_backend_id {
                                    webview_push_request(
                                        cx.app,
                                        WebViewRequest::Destroy { id: prev },
                                    );
                                }
                                cx.with_state(BackendInit::default, |st| {
                                    st.created = false;
                                    st.last_loaded_url.clear();
                                    st.last_backend_id = Some(backend.id);
                                });
                            } else {
                                cx.with_state(BackendInit::default, |st| {
                                    st.last_backend_id = Some(backend.id);
                                });
                            }

                            let needs_create =
                                cx.with_state(BackendInit::default, |st| !st.created);
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
                                let needs_load = cx.with_state(BackendInit::default, |st| {
                                    st.last_loaded_url != url_now
                                });
                                if needs_load {
                                    let next: Arc<str> = Arc::from(url_now.clone());
                                    webview_push_request(
                                        cx.app,
                                        WebViewRequest::LoadUrl {
                                            id: backend.id,
                                            url: next,
                                        },
                                    );
                                    cx.with_state(BackendInit::default, |st| {
                                        st.last_loaded_url = url_now;
                                    });
                                }
                            }

                            let intent = cx
                                .get_model_cloned(&nav_intent, Invalidation::Paint)
                                .unwrap_or(None);
                            if let Some(intent) = intent {
                                let request = match intent {
                                    WebPreviewBackendAction::GoBack => {
                                        WebViewRequest::GoBack { id: backend.id }
                                    }
                                    WebPreviewBackendAction::GoForward => {
                                        WebViewRequest::GoForward { id: backend.id }
                                    }
                                    WebPreviewBackendAction::Reload => {
                                        WebViewRequest::Reload { id: backend.id }
                                    }
                                };
                                webview_push_request(cx.app, request);
                                let _ = cx.app.models_mut().update(&nav_intent, |v| *v = None);
                            }

                            let clear_console = cx
                                .get_model_copied(&console_clear_intent, Invalidation::Paint)
                                .unwrap_or(false);
                            if clear_console {
                                webview_clear_console(cx.app, backend.id);
                                let _ = cx
                                    .app
                                    .models_mut()
                                    .update(&console_clear_intent, |v| *v = false);
                            }

                            // If the backend navigates (e.g. clicking a link), reflect the actual URL into
                            // the address bar *only when the user is not editing the draft*.
                            if let Some(runtime) = webview_runtime_state(cx.app, backend.id)
                                && let Some(runtime_url) = runtime.url.as_deref()
                            {
                                let url_model_now = cx
                                    .get_model_cloned(&url_model, Invalidation::Paint)
                                    .unwrap_or_default();
                                let draft_now = cx
                                    .get_model_cloned(&url_draft, Invalidation::Paint)
                                    .unwrap_or_default();

                                let is_editing = draft_now != url_model_now;
                                if !is_editing && url_model_now != runtime_url {
                                    let next = runtime_url.to_string();
                                    let _ = cx
                                        .app
                                        .models_mut()
                                        .update(&url_model, |v| *v = next.clone());
                                    let _ = cx
                                        .app
                                        .models_mut()
                                        .update(&url_draft, |v| *v = next.clone());
                                    cx.with_state(BackendInit::default, |st| {
                                        st.last_loaded_url = next;
                                    });
                                }
                            }
                        }
                    }
                }

                cx.provide(controller.clone(), |cx| {
                    let body = children(cx, controller);

                    let body = ui::v_stack(move |_cx| body)
                        .layout(
                            LayoutRefinement::default()
                                .w_full()
                                .h_full()
                                .min_w_0()
                                .min_h_0(),
                        )
                        .gap(Space::N0)
                        .items(Items::Stretch)
                        .into_element(cx);

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
                })
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
#[derive(Debug)]
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
        let row = ui::h_row(move |_cx| self.children)
            .layout(self.layout)
            .gap(Space::N1)
            .items_center()
            .into_element(cx);

        let bar = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N2).merge(self.chrome),
                LayoutRefinement::default(),
            ),
            move |_cx| vec![row],
        );

        let separator = fret_ui_shadcn::separator().into_element(cx);
        let el = ui::v_stack(move |_cx| vec![bar, separator])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0)
            .into_element(cx);

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
pub struct WebPreviewNavigationButton {
    tooltip: Option<Arc<str>>,
    disabled: bool,
    on_activate: Option<fret_ui::action::OnActivate>,
    #[cfg(feature = "webview")]
    backend_action: Option<WebPreviewBackendAction>,
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
            #[cfg(feature = "webview")]
            backend_action: None,
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

    #[cfg(feature = "webview")]
    pub fn backend_action(mut self, action: WebPreviewBackendAction) -> Self {
        self.backend_action = Some(action);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        #[cfg(feature = "webview")]
        let derived_disabled = match (self.backend_action, use_web_preview_controller(cx)) {
            (Some(action), Some(controller)) => {
                if let Some(backend) = controller.backend.as_ref() {
                    let runtime = webview_runtime_state(cx.app, backend.id);
                    match action {
                        WebPreviewBackendAction::GoBack => {
                            runtime.map(|st| !st.navigation.can_go_back).unwrap_or(true)
                        }
                        WebPreviewBackendAction::GoForward => runtime
                            .map(|st| !st.navigation.can_go_forward)
                            .unwrap_or(true),
                        WebPreviewBackendAction::Reload => {
                            runtime.map(|st| st.navigation.is_loading).unwrap_or(false)
                        }
                    }
                } else {
                    false
                }
            }
            _ => false,
        };
        #[cfg(not(feature = "webview"))]
        let derived_disabled = false;

        let mut button = Button::new("")
            .children(self.children)
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Sm)
            .disabled(self.disabled || derived_disabled)
            .refine_layout(LayoutRefinement::default().w_px(Px(32.0)).h_px(Px(32.0)))
            .refine_style(
                ChromeRefinement::default()
                    .p(Space::N0)
                    .text_color(ColorRef::Color(theme.color_token("muted-foreground"))),
            );

        #[cfg(feature = "webview")]
        let backend_action = self.backend_action;
        #[cfg(feature = "webview")]
        let user_on_activate = self.on_activate;

        #[cfg(not(feature = "webview"))]
        let user_on_activate = self.on_activate;

        #[cfg(feature = "webview")]
        if let Some(action) = backend_action {
            if let Some(controller) = use_web_preview_controller(cx) {
                let nav_intent = controller.nav_intent.clone();
                let user_on_activate = user_on_activate.clone();
                button = button.on_activate(Arc::new(move |host, action_cx, reason| {
                    let _ = host.models_mut().update(&nav_intent, |v| *v = Some(action));
                    if let Some(user) = user_on_activate.as_ref() {
                        user(host, action_cx, reason);
                    }
                    host.notify(action_cx);
                    host.request_redraw(action_cx.window);
                }));
            } else if let Some(on_activate) = user_on_activate {
                button = button.on_activate(on_activate);
            }
        } else if let Some(on_activate) = user_on_activate {
            button = button.on_activate(on_activate);
        }

        #[cfg(not(feature = "webview"))]
        if let Some(on_activate) = user_on_activate {
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
#[derive(Debug)]
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
                style: Some(
                    typography::TypographyPreset::content_ui(typography::UiTextSize::Sm)
                        .resolve(&theme),
                ),
                color: Some(theme.color_token("muted-foreground")),
                wrap: TextWrap::Word,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
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
            webview_register_surface_tracked(
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
#[derive(Debug)]
pub struct WebPreviewConsole {
    logs: Arc<[WebPreviewConsoleLog]>,
    backend_logs: bool,
    children: Vec<AnyElement>,
    test_id_trigger: Option<Arc<str>>,
    test_id_content: Option<Arc<str>>,
    test_id_marker: Option<Arc<str>>,
    test_id_backend_logs_marker: Option<Arc<str>>,
    test_id_clear: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Default for WebPreviewConsole {
    fn default() -> Self {
        Self {
            logs: Arc::from([]),
            backend_logs: false,
            children: Vec::new(),
            test_id_trigger: None,
            test_id_content: None,
            test_id_marker: None,
            test_id_backend_logs_marker: None,
            test_id_clear: None,
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

    pub fn backend_logs(mut self, enabled: bool) -> Self {
        self.backend_logs = enabled;
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

    pub fn test_id_backend_logs_marker(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_backend_logs_marker = Some(id.into());
        self
    }

    pub fn test_id_clear(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_clear = Some(id.into());
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
        #[cfg(feature = "webview")]
        let test_id_clear = self.test_id_clear.clone();
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
            Some(ColorRef::Color(theme.color_token("muted-foreground"))),
        );

        let label = cx.text("Console");
        let row = ui::h_row(move |_cx| vec![label, chevron])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .justify(Justify::Between)
            .items_center()
            .into_element(cx);

        let trigger_test_id = self.test_id_trigger.clone();
        let console_open = controller.console_open.clone();
        let console_open_for_toggle = console_open.clone();
        let theme_for_toggle = theme.clone();
        let open_now_for_toggle = open_now;
        let toggle_button = control_chrome_pressable_with_id_props(cx, move |cx, st, _id| {
            cx.pressable_toggle_bool(&console_open_for_toggle);

            let mut pressable = PressableProps::default();
            pressable.enabled = true;
            pressable.focusable = true;
            pressable.a11y.role = Some(SemanticsRole::Button);
            pressable.a11y.label = Some(Arc::<str>::from("Toggle console"));
            pressable.a11y.expanded = Some(open_now_for_toggle);
            pressable.a11y.test_id = trigger_test_id.clone();

            let base = theme_for_toggle.color_token("accent");
            let hover_bg = fret_core::Color {
                r: base.r,
                g: base.g,
                b: base.b,
                a: (base.a * 0.18).clamp(0.0, 1.0),
            };
            let pressed_bg = fret_core::Color {
                r: base.r,
                g: base.g,
                b: base.b,
                a: (base.a * 0.28).clamp(0.0, 1.0),
            };

            let bg = if st.pressed {
                Some(pressed_bg)
            } else if st.hovered {
                Some(hover_bg)
            } else {
                None
            };

            let chrome = ChromeRefinement::default().p(Space::N4).rounded(Radius::Md);
            let mut chrome_props = decl_style::container_props(
                &theme_for_toggle,
                chrome,
                LayoutRefinement::default().flex_1(),
            );
            chrome_props.background = bg;

            (pressable, chrome_props, move |_cx| vec![row])
        });

        #[cfg(feature = "webview")]
        let clear_button = if self.backend_logs {
            controller.backend.as_ref().map(|_backend| {
                let clear_intent = controller.console_clear_intent.clone();
                let mut clear = Button::new("Clear")
                    .children([cx.text("Clear")])
                    .variant(ButtonVariant::Ghost)
                    .size(ButtonSize::Sm)
                    .on_activate(Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&clear_intent, |v| *v = true);
                        host.notify(action_cx);
                        host.request_redraw(action_cx.window);
                    }))
                    .into_element(cx);

                if let Some(test_id) = test_id_clear.clone() {
                    clear = clear.attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Button)
                            .test_id(test_id),
                    );
                }
                clear
            })
        } else {
            None
        };

        #[cfg(not(feature = "webview"))]
        let clear_button: Option<AnyElement> = None;

        let header = ui::h_row(move |_cx| {
            let mut items = vec![toggle_button];
            if let Some(clear_button) = clear_button {
                items.push(clear_button);
            }
            items
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_center()
        .gap(Space::N1)
        .into_element(cx);

        #[cfg(feature = "webview")]
        let (logs, backend_logs_present) = {
            let mut out: Vec<WebPreviewConsoleLog> = self.logs.iter().cloned().collect();

            let backend_entries = if self.backend_logs {
                controller
                    .backend
                    .as_ref()
                    .map(|b| webview_console_entries(cx.app, b.id))
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            let backend_logs_present = !backend_entries.is_empty();
            for entry in backend_entries {
                let level = match entry.level {
                    fret_webview::WebViewConsoleLevel::Log => WebPreviewConsoleLogLevel::Log,
                    fret_webview::WebViewConsoleLevel::Warn => WebPreviewConsoleLogLevel::Warn,
                    fret_webview::WebViewConsoleLevel::Error => WebPreviewConsoleLogLevel::Error,
                };
                out.push(WebPreviewConsoleLog {
                    level,
                    timestamp: Arc::<str>::from(""),
                    message: entry.message,
                });
            }

            (
                Arc::<[WebPreviewConsoleLog]>::from(out),
                backend_logs_present,
            )
        };

        #[cfg(not(feature = "webview"))]
        let (logs, backend_logs_present) = (
            Arc::<[WebPreviewConsoleLog]>::from(self.logs.iter().cloned().collect::<Vec<_>>()),
            false,
        );
        let children = self.children;
        let marker_id = self.test_id_marker.clone();
        let backend_marker_id = self.test_id_backend_logs_marker.clone();
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
                        WebPreviewConsoleLogLevel::Error => theme
                            .color_by_key("component.web_preview.console.error_fg")
                            .unwrap_or_else(|| theme.color_token("destructive")),
                        WebPreviewConsoleLogLevel::Warn => theme
                            .color_by_key("component.web_preview.console.warn_fg")
                            // Tailwind: yellow-600 (#ca8a04).
                            .unwrap_or(fret_ui_kit::colors::linear_from_hex_rgb(0xca_8a_04)),
                        WebPreviewConsoleLogLevel::Log => theme
                            .color_by_key("component.web_preview.console.log_fg")
                            .unwrap_or_else(|| theme.color_token("foreground")),
                    };

                    let ts = cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: log.timestamp.clone(),
                        style: Some(
                            typography::TypographyPreset::control_monospace(
                                typography::UiTextSize::Xs,
                            )
                            .resolve(&theme),
                        ),
                        color: Some(theme.color_token("muted-foreground")),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: Default::default(),
                    });

                    let msg = cx.text_props(TextProps {
                        layout: decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default().flex_grow(1.0).min_w_0(),
                        ),
                        text: log.message.clone(),
                        style: Some(
                            typography::TypographyPreset::control_monospace(
                                typography::UiTextSize::Xs,
                            )
                            .resolve(&theme),
                        ),
                        color: Some(fg),
                        wrap: TextWrap::Word,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: Default::default(),
                    });

                    let row = ui::h_row(move |_cx| vec![ts, msg])
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx);
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

            if backend_logs_present && let Some(backend_marker_id) = backend_marker_id {
                rows.push(
                    cx.text("").attach_semantics(
                        SemanticsDecoration::default()
                            .role(SemanticsRole::Generic)
                            .test_id(backend_marker_id),
                    ),
                );
            }

            ui::v_stack(move |_cx| rows)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N1)
                .into_element(cx)
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

        let root = Collapsible::new(console_open)
            .refine_layout(self.layout)
            .refine_style(
                ChromeRefinement::default()
                    .border_color(ColorRef::Color(theme.color_token("border")))
                    .bg(ColorRef::Token {
                        key: "muted",
                        fallback: fret_ui_kit::ColorFallback::ThemeSurfaceBackground,
                    })
                    .merge(self.chrome),
            )
            .into_element(cx, move |_cx, _is_open| header, move |_cx| content);

        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind, Length};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(320.0)),
        )
    }

    fn find_text_by_content<'a>(el: &'a AnyElement, text: &str) -> Option<&'a TextProps> {
        match &el.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(props),
            _ => el
                .children
                .iter()
                .find_map(|child| find_text_by_content(child, text)),
        }
    }

    fn has_test_id(element: &AnyElement, expected: &str) -> bool {
        element
            .semantics_decoration
            .as_ref()
            .and_then(|decoration| decoration.test_id.as_deref())
            .is_some_and(|test_id| test_id == expected)
            || match &element.kind {
                ElementKind::Pressable(props) => props.a11y.test_id.as_deref() == Some(expected),
                ElementKind::Semantics(props) => props.test_id.as_deref() == Some(expected),
                _ => false,
            }
            || element
                .children
                .iter()
                .any(|child| has_test_id(child, expected))
    }

    #[test]
    fn web_preview_root_provides_controller_to_children() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let el =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "web-preview", |cx| {
                WebPreview::new()
                    .default_url("https://example.com")
                    .into_element_with_children(cx, |cx, _controller| {
                        vec![
                            WebPreviewUrl::new().test_id("web-url").into_element(cx),
                            WebPreviewBody::new().test_id("web-body").into_element(cx),
                        ]
                    })
            });

        assert!(has_test_id(&el, "web-url"));
        assert!(has_test_id(&el, "web-body"));
        assert!(find_text_by_content(&el, "Preview: https://example.com").is_some());
    }

    #[test]
    fn web_preview_console_messages_can_shrink_within_log_rows() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let message = "A very long console log message that should wrap instead of overflowing beside the timestamp";

        let el =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "web-preview", |cx| {
                WebPreview::new().into_element_with_children(cx, |cx, controller| {
                    let _ = cx
                        .app
                        .models_mut()
                        .update(&controller.console_open, |open| *open = true);
                    vec![
                        WebPreviewConsole::new()
                            .logs(Arc::from([WebPreviewConsoleLog::new(
                                WebPreviewConsoleLogLevel::Log,
                                message,
                            )
                            .timestamp("12:34:56")]))
                            .into_element(cx),
                    ]
                })
            });

        let msg = find_text_by_content(&el, message).expect("web preview console message text");
        assert_eq!(msg.wrap, TextWrap::Word);
        assert_eq!(msg.layout.flex.grow, 1.0);
        assert_eq!(msg.layout.flex.shrink, 1.0);
        assert_eq!(msg.layout.flex.basis, Length::Auto);
        assert_eq!(msg.layout.size.min_width, Some(Length::Px(Px(0.0))));
    }
}
