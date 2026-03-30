use std::any::Any;
use std::sync::Arc;

use fret_core::{Color, Px};
use fret_icons::IconId;
use fret_runtime::{ActionId, Model};
use fret_ui::action::{ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, SpacerProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::command::ElementCommandGatingExt as _;
use fret_ui_kit::declarative::controllable_state::use_controllable_model;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, WidgetStateProperty, WidgetStates,
    ui,
};

use crate::button::{Button, ButtonSize, ButtonStyle, ButtonVariant};
use crate::test_id::attach_test_id;

type ActionPayloadFactory = Arc<dyn Fn() -> Box<dyn Any + Send + Sync> + 'static>;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn hidden_element<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Px(Px(0.0)),
            height: Length::Px(Px(0.0)),
            ..Default::default()
        },
        flex: fret_ui::element::FlexItemStyle {
            grow: 0.0,
            shrink: 0.0,
            basis: Length::Px(Px(0.0)),
            ..Default::default()
        },
        ..Default::default()
    };
    cx.spacer(SpacerProps {
        layout,
        min: Px(0.0),
    })
}

fn dispatch_action_after_close(
    host: &mut dyn UiActionHost,
    action_cx: fret_ui::action::ActionCx,
    reason: ActivateReason,
    action: &ActionId,
    payload: Option<&ActionPayloadFactory>,
) {
    host.record_pending_command_dispatch_source(action_cx, action, reason);
    if let Some(payload) = payload {
        host.record_pending_action_payload(action_cx, action, payload());
    }
    host.dispatch_command(Some(action_cx.window), action.clone());
}

fn banner_close_on_activate(
    visible: Option<Model<bool>>,
    on_close: Option<OnActivate>,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate_user: Option<OnActivate>,
) -> OnActivate {
    Arc::new(move |host, action_cx, reason| {
        if let Some(visible) = visible.as_ref() {
            let _ = host.models_mut().update(visible, |v| *v = false);
            host.notify(action_cx);
        }

        if let Some(on_close) = on_close.as_ref() {
            on_close(host, action_cx, reason);
        }
        if let Some(action) = action.as_ref() {
            dispatch_action_after_close(host, action_cx, reason, action, action_payload.as_ref());
        }
        if let Some(user) = on_activate_user.as_ref() {
            user(host, action_cx, reason);
        }
    })
}

#[derive(Clone)]
struct BannerScope {
    visible: Model<bool>,
    on_close: Option<OnActivate>,
}

fn banner_scope_inherited<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<BannerScope> {
    cx.provided::<BannerScope>().cloned()
}

#[track_caller]
fn with_banner_scope_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    scope: BannerScope,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.provide(scope, f)
}

/// A dismissible banner block inspired by Kibo's shadcn blocks.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/banner`
pub struct Banner {
    children: Vec<AnyElement>,
    visible: Option<Model<bool>>,
    default_visible: bool,
    on_close: Option<OnActivate>,
    inset: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Banner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Banner")
            .field("children_len", &self.children.len())
            .field("visible", &self.visible.is_some())
            .field("default_visible", &self.default_visible)
            .field("on_close", &self.on_close.is_some())
            .field("inset", &self.inset)
            .field("test_id", &self.test_id)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Banner {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            visible: None,
            default_visible: true,
            on_close: None,
            inset: false,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Controlled visibility model (Radix-style `visible` / `defaultVisible`).
    pub fn visible_model(mut self, visible: Model<bool>) -> Self {
        self.visible = Some(visible);
        self
    }

    pub fn default_visible(mut self, visible: bool) -> Self {
        self.default_visible = visible;
        self
    }

    pub fn on_close(mut self, on_close: OnActivate) -> Self {
        self.on_close = Some(on_close);
        self
    }

    pub fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).snapshot();

            let visible = use_controllable_model(cx, self.visible, || self.default_visible);
            let is_visible = cx
                .watch_model(&visible.model())
                .layout()
                .copied()
                .unwrap_or(true);
            if !is_visible {
                return hidden_element(cx);
            }

            let bg = theme.color_token("primary");
            let fg = theme.color_token("primary-foreground");

            let mut chrome = ChromeRefinement::default()
                .px(Space::N4)
                .py(Space::N2)
                .bg(ColorRef::Color(bg))
                .text_color(ColorRef::Color(fg));
            if self.inset {
                chrome = chrome.rounded(Radius::Lg);
            }
            chrome = chrome.merge(self.chrome);

            let layout = LayoutRefinement::default().w_full().merge(self.layout);
            let props = decl_style::container_props(&theme, chrome, layout);
            let children = self.children;

            let scope = BannerScope {
                visible: visible.model(),
                on_close: self.on_close.clone(),
            };

            let test_id = self
                .test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.banner"));

            let el = with_banner_scope_provider(cx, scope, |cx| {
                cx.container(props, move |cx| {
                    vec![
                        ui::h_row(|_cx| children)
                            .justify_between()
                            .items_center()
                            .gap(Space::N2)
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .into_element(cx),
                    ]
                })
            });

            attach_test_id(el, test_id)
        })
    }
}

#[derive(Debug)]
pub struct BannerIcon {
    child: AnyElement,
    test_id: Option<Arc<str>>,
}

impl BannerIcon {
    pub fn new(child: AnyElement) -> Self {
        Self {
            child,
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let base = theme.color_token("background");
        let border = alpha_mul(base, 0.20);
        let bg = alpha_mul(base, 0.10);

        let chrome = ChromeRefinement::default()
            .p(Space::N1)
            .rounded(Radius::Full)
            .border_1()
            .border_color(ColorRef::Color(border))
            .bg(ColorRef::Color(bg))
            .shadow_sm();

        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let child = self.child;
        let el = cx.container(props, move |_cx| vec![child]);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.banner-icon")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct BannerTitle {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl BannerTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = ui::text(self.text)
            .text_sm()
            .flex_1()
            .min_w_0()
            .into_element(cx);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.banner-title")),
        )
    }
}

#[derive(Clone)]
pub struct BannerAction {
    label: Arc<str>,
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for BannerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BannerAction")
            .field("label", &self.label)
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl BannerAction {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            action: None,
            action_payload: None,
            on_activate: None,
            disabled: false,
            test_id: None,
        }
    }

    /// Bind a stable action ID to this banner action (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized banner actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`BannerAction::action_payload`], but computes the payload lazily on activation.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme.color_token("primary-foreground");
        let mut hover_bg = theme.color_token("background");
        hover_bg = alpha_mul(hover_bg, 0.10);

        let style = ButtonStyle::default()
            .background(
                WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                    .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg))),
            )
            .foreground(WidgetStateProperty::new(Some(ColorRef::Color(fg))));

        let mut button = Button::new(self.label)
            .variant(ButtonVariant::Outline)
            .size(ButtonSize::Sm)
            .disabled(self.disabled)
            .style(style);
        if let Some(action) = self.action {
            button = button.action(action);
        }
        if let Some(payload) = self.action_payload {
            button = button.action_payload_factory(payload);
        }
        if let Some(id) = self.test_id {
            button = button.test_id(id);
        }
        if let Some(on_activate) = self.on_activate {
            button = button.on_activate(on_activate);
        }
        button.into_element(cx)
    }
}

#[derive(Clone)]
pub struct BannerClose {
    action: Option<ActionId>,
    action_payload: Option<ActionPayloadFactory>,
    on_activate: Option<OnActivate>,
    disabled: bool,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for BannerClose {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BannerClose")
            .field("action", &self.action)
            .field("action_payload", &self.action_payload.is_some())
            .field("on_activate", &self.on_activate.is_some())
            .field("disabled", &self.disabled)
            .field("test_id", &self.test_id)
            .finish()
    }
}

impl BannerClose {
    pub fn new() -> Self {
        Self {
            action: None,
            action_payload: None,
            on_activate: None,
            disabled: false,
            test_id: None,
        }
    }

    /// Bind a stable action ID to this banner close affordance (action-first authoring).
    pub fn action(mut self, action: impl Into<fret_runtime::ActionId>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// Attach a payload for parameterized banner close actions (ADR 0312).
    pub fn action_payload<T>(mut self, payload: T) -> Self
    where
        T: Any + Send + Sync + Clone + 'static,
    {
        let payload = Arc::new(payload);
        self.action_payload = Some(Arc::new(move || Box::new(payload.as_ref().clone())));
        self
    }

    /// Like [`BannerClose::action_payload`], but computes the payload lazily on activation.
    pub fn action_payload_factory(mut self, payload: ActionPayloadFactory) -> Self {
        self.action_payload = Some(payload);
        self
    }

    pub fn on_activate(mut self, on_activate: OnActivate) -> Self {
        self.on_activate = Some(on_activate);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme.color_token("primary-foreground");
        let mut hover_bg = theme.color_token("background");
        hover_bg = alpha_mul(hover_bg, 0.10);

        let scope = banner_scope_inherited(cx);
        let visible = scope.as_ref().map(|s| s.visible.clone());
        let on_close = scope.and_then(|s| s.on_close.clone());
        let action = self.action.clone();
        let action_payload = self.action_payload.clone();
        let on_activate_user = self.on_activate.clone();
        let disabled = self.disabled
            || action
                .as_ref()
                .is_some_and(|action| !cx.command_is_enabled(action));

        let style = ButtonStyle::default()
            .background(
                WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                    .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg))),
            )
            .foreground(WidgetStateProperty::new(Some(ColorRef::Color(fg))));

        let icon = decl_icon::icon_with(
            cx,
            IconId::new_static("lucide.x"),
            Some(Px(18.0)),
            Some(ColorRef::Color(fg)),
        );

        let on_activate =
            banner_close_on_activate(visible, on_close, action, action_payload, on_activate_user);

        let mut button = Button::new("Close")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Icon)
            .disabled(disabled)
            .style(style)
            .children([icon])
            .on_activate(on_activate);

        if let Some(id) = self.test_id {
            button = button.test_id(id);
        }

        button.into_element(cx)
    }
}

impl Default for BannerClose {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{banner_close_on_activate, dispatch_action_after_close};

    use fret_core::AppWindowId;
    use fret_runtime::{ActionId, CommandId, Effect, ModelStore, TimerToken};
    use fret_ui::GlobalElementId;
    use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
    use std::any::Any;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::Arc;

    struct FakeHost {
        models: ModelStore,
        notifies: Vec<ActionCx>,
        effects: Vec<Effect>,
        dispatch_sources: Vec<(ActionCx, CommandId, ActivateReason)>,
        payloads: Vec<(ActionCx, ActionId, Box<dyn Any + Send + Sync>)>,
        trace: Rc<RefCell<Vec<&'static str>>>,
    }

    impl Default for FakeHost {
        fn default() -> Self {
            Self {
                models: ModelStore::default(),
                notifies: Vec::new(),
                effects: Vec::new(),
                dispatch_sources: Vec::new(),
                payloads: Vec::new(),
                trace: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl UiActionHost for FakeHost {
        fn models_mut(&mut self) -> &mut ModelStore {
            &mut self.models
        }

        fn push_effect(&mut self, effect: Effect) {
            self.effects.push(effect);
        }

        fn request_redraw(&mut self, _window: AppWindowId) {}

        fn next_timer_token(&mut self) -> TimerToken {
            TimerToken(0)
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            fret_runtime::ClipboardToken::default()
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            fret_runtime::ShareSheetToken::default()
        }

        fn notify(&mut self, cx: ActionCx) {
            self.notifies.push(cx);
        }

        fn record_pending_command_dispatch_source(
            &mut self,
            cx: ActionCx,
            command: &CommandId,
            reason: ActivateReason,
        ) {
            self.trace.borrow_mut().push("action");
            self.dispatch_sources.push((cx, command.clone(), reason));
        }

        fn record_pending_action_payload(
            &mut self,
            cx: ActionCx,
            action: &ActionId,
            payload: Box<dyn Any + Send + Sync>,
        ) {
            self.payloads.push((cx, action.clone(), payload));
        }
    }

    #[test]
    fn dispatch_action_after_close_records_dispatch_source_and_payload() {
        let mut host = FakeHost::default();
        let cx = ActionCx {
            window: AppWindowId::default(),
            target: GlobalElementId(0),
        };
        let action = ActionId::from("test.banner.close.v1");
        let payload: super::ActionPayloadFactory =
            Arc::new(|| Box::new(41_u32) as Box<dyn Any + Send + Sync>);

        dispatch_action_after_close(
            &mut host,
            cx,
            ActivateReason::Pointer,
            &action,
            Some(&payload),
        );

        assert_eq!(host.dispatch_sources.len(), 1);
        assert_eq!(host.dispatch_sources[0].0, cx);
        assert_eq!(host.dispatch_sources[0].1, action.clone());
        assert_eq!(host.dispatch_sources[0].2, ActivateReason::Pointer);
        assert_eq!(host.effects.len(), 1);
        assert_eq!(
            host.effects[0],
            Effect::Command {
                window: Some(cx.window),
                command: action.clone(),
            }
        );
        assert_eq!(host.payloads.len(), 1);
        let payload_value = host.payloads.remove(0).2;
        let payload_value = payload_value
            .downcast::<u32>()
            .expect("payload should keep the typed value");
        assert_eq!(*payload_value, 41);
    }

    #[test]
    fn banner_close_runs_close_then_action_then_user_hook() {
        let trace = Rc::new(RefCell::new(Vec::new()));
        let mut host = FakeHost {
            trace: trace.clone(),
            ..Default::default()
        };
        let visible = host.models.insert(true);
        let close_trace = trace.clone();
        let on_close: OnActivate = Arc::new(move |_host, _cx, _reason| {
            close_trace.borrow_mut().push("close");
        });
        let user_trace = trace.clone();
        let user: OnActivate = Arc::new(move |_host, _cx, _reason| {
            user_trace.borrow_mut().push("user");
        });
        let handler = banner_close_on_activate(
            Some(visible.clone()),
            Some(on_close),
            Some(ActionId::from("test.banner.close.v1")),
            None,
            Some(user),
        );
        let cx = ActionCx {
            window: AppWindowId::default(),
            target: GlobalElementId(0),
        };

        handler(&mut host, cx, ActivateReason::Pointer);

        assert_eq!(host.models.get_copied(&visible), Some(false));
        assert_eq!(host.notifies, vec![cx]);
        assert_eq!(*trace.borrow(), vec!["close", "action", "user"]);
        assert_eq!(
            host.effects,
            vec![Effect::Command {
                window: Some(cx.window),
                command: CommandId::from("test.banner.close.v1"),
            }]
        );
    }
}
