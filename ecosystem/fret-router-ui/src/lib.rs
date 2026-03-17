//! UI adoption helpers for `fret-router` (desktop-first).
//!
//! This crate intentionally provides a thin layer:
//! - a window-scoped `RouterUiStore` that owns a router and a snapshot `Model`
//! - typed navigation helpers that update the snapshot model
//!
//! Policy-heavy behavior remains in apps and higher-level ecosystem crates.

use std::hash::Hash;
use std::sync::Arc;

use fret_app::App;
use fret_core::{AppWindowId, KeyCode, Modifiers, SemanticsRole};
use fret_router::{
    HistoryAdapter, NavigationAction, PathParam, RouteCodec, RouteLocation, RouteMatchSnapshot,
    RoutePrefetchIntent, RouteSearchValidationFailure, Router, RouterBuildLocationError,
    RouterEvent, RouterTransition, RouterUpdate, RouterUpdateWithPrefetchIntents, SearchMap,
};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, Effect, KeyChord,
    Model, PlatformFilter, WeakModel, WhenExpr, WindowCommandAvailabilityService,
};
use fret_ui::action::{ActionCx, OnActivate, OnHoverChange, UiActionHost, UiFocusActionHost};
use fret_ui::element::AnyElement;
use fret_ui::element::{PressableKeyActivation, PressableProps, SemanticsDecoration};
use fret_ui::{ElementContext, Invalidation};
use fret_ui_kit::IntoUiElement;

pub mod app;

#[derive(Debug, Clone)]
pub struct RouterUiSnapshot<R> {
    pub location: RouteLocation,
    pub matches: Vec<RouteMatchSnapshot<R>>,
    pub is_not_found: bool,
    pub last_transition: Option<RouterTransition>,
}

impl<R> RouterUiSnapshot<R>
where
    R: Clone,
{
    pub fn from_state(state: &fret_router::RouterState<R>) -> Self {
        Self {
            location: state.location.clone(),
            matches: state.matches.clone(),
            is_not_found: state.is_not_found,
            last_transition: state.last_transition.clone(),
        }
    }

    pub fn leaf_match(&self) -> Option<&RouteMatchSnapshot<R>> {
        self.matches.last()
    }

    pub fn leaf_route(&self) -> Option<&R> {
        self.leaf_match().map(|m| &m.route)
    }

    pub fn match_depth(&self) -> usize {
        self.matches.len()
    }

    pub fn match_at(&self, index: usize) -> Option<&RouteMatchSnapshot<R>> {
        self.matches.get(index)
    }

    pub fn route_at(&self, index: usize) -> Option<&R> {
        self.match_at(index).map(|m| &m.route)
    }

    pub fn is_at_location(&self, location: &RouteLocation) -> bool {
        self.location.canonicalized() == location.canonicalized()
    }

    pub fn is_at_href(&self, href: &str) -> bool {
        self.location.to_url() == href.trim()
    }

    pub fn is_at_link(&self, link: &RouterLink) -> bool {
        self.is_at_location(&link.to) || self.is_at_href(&link.href)
    }
}

pub struct RouterUiStore<R, H>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    window: AppWindowId,
    router: Model<Router<R, H>>,
    snapshot: Model<RouterUiSnapshot<R>>,
    intents: Model<Vec<RoutePrefetchIntent<R>>>,
}

pub const ROUTER_COMMAND_BACK: &str = "router.back";
pub const ROUTER_COMMAND_FORWARD: &str = "router.forward";

pub fn register_router_commands(registry: &mut CommandRegistry) {
    registry.register(
        CommandId::from(ROUTER_COMMAND_BACK),
        CommandMeta::new("Back")
            .with_category("Router")
            .with_description("Navigate back in router history.")
            .with_default_keybindings([
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::BracketLeft,
                        Modifiers {
                            meta: true,
                            ..Default::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::Windows,
                    KeyChord::new(
                        KeyCode::ArrowLeft,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::Linux,
                    KeyChord::new(
                        KeyCode::ArrowLeft,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    ),
                ),
            ])
            .with_when(WhenExpr::parse("router.can_back").expect("when expr should parse"))
            .with_scope(CommandScope::Window),
    );
    registry.register(
        CommandId::from(ROUTER_COMMAND_FORWARD),
        CommandMeta::new("Forward")
            .with_category("Router")
            .with_description("Navigate forward in router history.")
            .with_default_keybindings([
                DefaultKeybinding::single(
                    PlatformFilter::Macos,
                    KeyChord::new(
                        KeyCode::BracketRight,
                        Modifiers {
                            meta: true,
                            ..Default::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::Windows,
                    KeyChord::new(
                        KeyCode::ArrowRight,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    ),
                ),
                DefaultKeybinding::single(
                    PlatformFilter::Linux,
                    KeyChord::new(
                        KeyCode::ArrowRight,
                        Modifiers {
                            alt: true,
                            ..Default::default()
                        },
                    ),
                ),
            ])
            .with_when(WhenExpr::parse("router.can_forward").expect("when expr should parse"))
            .with_scope(CommandScope::Window),
    );
}

impl<R, H> RouterUiStore<R, H>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    pub fn new(app: &mut App, window: AppWindowId, router: Router<R, H>) -> Self {
        let snapshot = RouterUiSnapshot::from_state(router.state());
        let router = app.models_mut().insert(router);
        let snapshot = app.models_mut().insert(snapshot);
        let intents = app.models_mut().insert(Vec::new());
        let store = Self {
            window,
            router,
            snapshot,
            intents,
        };
        store.sync_command_availability(app);
        store
    }

    pub fn snapshot_model(&self) -> Model<RouterUiSnapshot<R>> {
        self.snapshot.clone()
    }

    pub fn router_model(&self) -> Model<Router<R, H>> {
        self.router.clone()
    }

    pub fn intents_model(&self) -> Model<Vec<RoutePrefetchIntent<R>>> {
        self.intents.clone()
    }

    pub fn build_location(
        &self,
        app: &App,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
    ) -> Result<RouteLocation, RouterBuildLocationError> {
        app.models()
            .read(&self.router, |router| {
                router.build_location(route, params, search, fragment)
            })
            .expect("router model should be readable")
    }

    pub fn href_to(
        &self,
        app: &App,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
    ) -> Result<String, RouterBuildLocationError> {
        app.models()
            .read(&self.router, |router| {
                router.href_to(route, params, search, fragment)
            })
            .expect("router model should be readable")
    }

    pub fn link_to(
        &self,
        app: &App,
        action: NavigationAction,
        route: &R,
        params: &[PathParam],
        search: SearchMap,
        fragment: Option<String>,
    ) -> Result<RouterLink, RouterBuildLocationError> {
        let to = self.build_location(app, route, params, search, fragment)?;
        Ok(self.link_to_location(action, to))
    }

    pub fn link_to_location(&self, action: NavigationAction, mut to: RouteLocation) -> RouterLink {
        to.canonicalize();
        let href: Arc<str> = Arc::from(to.to_url());
        RouterLink { action, href, to }
    }

    pub fn link_to_typed_route<C>(
        &self,
        action: NavigationAction,
        codec: &C,
        route: &C::Route,
    ) -> RouterLink
    where
        C: RouteCodec,
    {
        self.link_to_location(action, codec.encode_canonical(route))
    }

    pub fn take_events(&self, app: &mut App) -> Vec<RouterEvent<R>> {
        app.models_mut()
            .update(&self.router, |router| router.take_events())
            .expect("router model should be updatable")
    }

    pub fn can_navigate(&self, app: &App, action: NavigationAction) -> bool {
        app.models()
            .read(&self.router, |router| router.history().can_navigate(action))
            .expect("router model should be readable")
    }

    fn current_history_availability(&self, app: &App) -> (bool, bool) {
        app.models()
            .read(&self.router, |router| {
                let history = router.history();
                (
                    history.can_navigate(NavigationAction::Back),
                    history.can_navigate(NavigationAction::Forward),
                )
            })
            .expect("router model should be readable")
    }

    /// Publish the current router back/forward availability for this window.
    ///
    /// This is useful when a store is created from pre-populated history or when an app wants to
    /// re-publish availability after external window/service lifecycle changes.
    pub fn sync_command_availability(&self, app: &mut App) {
        let (can_back, can_forward) = self.current_history_availability(app);
        app.with_global_mut(WindowCommandAvailabilityService::default, |svc, _app| {
            svc.set_router_availability(self.window, can_back, can_forward);
        });
    }

    pub fn handle_router_command(
        &self,
        app: &mut App,
        command: &CommandId,
    ) -> Result<bool, RouteSearchValidationFailure> {
        match command.as_str() {
            ROUTER_COMMAND_BACK => {
                if !self.can_navigate(app, NavigationAction::Back) {
                    return Ok(true);
                }
                let _ = self.navigate_with_prefetch_intents(app, NavigationAction::Back, None)?;
                Ok(true)
            }
            ROUTER_COMMAND_FORWARD => {
                if !self.can_navigate(app, NavigationAction::Forward) {
                    return Ok(true);
                }
                let _ =
                    self.navigate_with_prefetch_intents(app, NavigationAction::Forward, None)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn navigate_with_prefetch_intents_via_host(
        host: &mut dyn UiActionHost,
        window: AppWindowId,
        router: &WeakModel<Router<R, H>>,
        snapshot: &WeakModel<RouterUiSnapshot<R>>,
        intents: &WeakModel<Vec<RoutePrefetchIntent<R>>>,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> bool {
        let Some(router) = router.upgrade() else {
            return false;
        };
        let Some(snapshot_model) = snapshot.upgrade() else {
            return false;
        };
        let Some(intents_model) = intents.upgrade() else {
            return false;
        };

        let result = host.models_mut().update(&router, |router| {
            let update = router.navigate_with_prefetch_intents(action, target)?;
            let next_snapshot = RouterUiSnapshot::from_state(router.state());
            let history = router.history();
            let can_back = history.can_navigate(NavigationAction::Back);
            let can_forward = history.can_navigate(NavigationAction::Forward);
            Ok::<_, RouteSearchValidationFailure>((update, next_snapshot, can_back, can_forward))
        });

        let Ok(result) = result else {
            return false;
        };
        let Ok((update, next_snapshot, can_back, can_forward)) = result else {
            return false;
        };

        host.set_router_command_availability(window, can_back, can_forward);

        if !update.update.changed() {
            return false;
        }

        let _ = host
            .models_mut()
            .update(&snapshot_model, |value| *value = next_snapshot);
        let _ = host
            .models_mut()
            .update(&intents_model, |value| *value = update.intents);
        true
    }

    /// Build a history-navigation action handler for typed action surfaces.
    ///
    /// Intended usage:
    ///
    /// ```rust,ignore
    /// use fret::advanced::AppUiRawActionExt as _;
    ///
    /// cx.on_action_notify::<act::RouterBack>(store.back_on_action());
    /// cx.on_action_notify::<act::RouterForward>(store.forward_on_action());
    /// ```
    ///
    /// Only `NavigationAction::Back` and `NavigationAction::Forward` are meaningful here; passing
    /// `Push` or `Replace` returns a handler that reports `false`.
    pub fn navigate_history_on_action(
        &self,
        action: NavigationAction,
    ) -> impl Fn(&mut dyn UiFocusActionHost, ActionCx) -> bool + 'static {
        let window = self.window;
        let router: WeakModel<Router<R, H>> = self.router.downgrade();
        let snapshot: WeakModel<RouterUiSnapshot<R>> = self.snapshot.downgrade();
        let intents: WeakModel<Vec<RoutePrefetchIntent<R>>> = self.intents.downgrade();
        move |host, _action_cx| match action {
            NavigationAction::Back | NavigationAction::Forward => {
                Self::navigate_with_prefetch_intents_via_host(
                    host, window, &router, &snapshot, &intents, action, None,
                )
            }
            NavigationAction::Push | NavigationAction::Replace => false,
        }
    }

    /// Convenience wrapper for `navigate_history_on_action(NavigationAction::Back)`.
    pub fn back_on_action(
        &self,
    ) -> impl Fn(&mut dyn UiFocusActionHost, ActionCx) -> bool + 'static {
        self.navigate_history_on_action(NavigationAction::Back)
    }

    /// Convenience wrapper for `navigate_history_on_action(NavigationAction::Forward)`.
    pub fn forward_on_action(
        &self,
    ) -> impl Fn(&mut dyn UiFocusActionHost, ActionCx) -> bool + 'static {
        self.navigate_history_on_action(NavigationAction::Forward)
    }

    fn apply_update(
        &self,
        app: &mut App,
        update: &RouterUpdate,
        intents: &[RoutePrefetchIntent<R>],
    ) {
        self.sync_command_availability(app);

        if !update.changed() {
            return;
        }

        let next = app
            .models()
            .read(&self.router, |router| {
                RouterUiSnapshot::from_state(router.state())
            })
            .expect("router model should be readable");
        let _ = app.models_mut().update(&self.snapshot, |v| *v = next);
        let intents = intents.to_vec();
        let _ = app.models_mut().update(&self.intents, |v| *v = intents);
        app.request_redraw(self.window);
    }

    pub fn navigate(
        &self,
        app: &mut App,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| router.navigate(action, target))
            .expect("router model should be updatable")?;
        self.apply_update(app, &update, &[]);
        Ok(update)
    }

    pub fn navigate_with_prefetch_intents(
        &self,
        app: &mut App,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| {
                router.navigate_with_prefetch_intents(action, target)
            })
            .expect("router model should be updatable")?;
        self.apply_update(app, &update.update, &update.intents);
        Ok(update)
    }

    pub fn navigate_typed_route<C>(
        &self,
        app: &mut App,
        action: NavigationAction,
        codec: &C,
        route: &C::Route,
    ) -> Result<RouterUpdate, RouteSearchValidationFailure>
    where
        C: RouteCodec,
    {
        self.navigate(app, action, Some(codec.encode_canonical(route)))
    }

    pub fn navigate_typed_route_with_prefetch_intents<C>(
        &self,
        app: &mut App,
        action: NavigationAction,
        codec: &C,
        route: &C::Route,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure>
    where
        C: RouteCodec,
    {
        self.navigate_with_prefetch_intents(app, action, Some(codec.encode_canonical(route)))
    }

    pub fn sync_with_prefetch_intents(
        &self,
        app: &mut App,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| router.sync_with_prefetch_intents())
            .expect("router model should be updatable")?;
        self.apply_update(app, &update.update, &update.intents);
        Ok(update)
    }

    pub fn init_with_prefetch_intents(
        &self,
        app: &mut App,
    ) -> Result<RouterUpdateWithPrefetchIntents<R>, RouteSearchValidationFailure> {
        let update = app
            .models_mut()
            .update(&self.router, |router| router.init_with_prefetch_intents())
            .expect("router model should be updatable")?;
        self.apply_update(app, &update.update, &update.intents);
        Ok(update)
    }

    pub fn navigate_link_on_activate(&self, link: RouterLink) -> OnActivate {
        let window = self.window;
        let router: WeakModel<Router<R, H>> = self.router.downgrade();
        let snapshot: WeakModel<RouterUiSnapshot<R>> = self.snapshot.downgrade();
        let intents: WeakModel<Vec<RoutePrefetchIntent<R>>> = self.intents.downgrade();
        Arc::new(move |host, _cx, _reason| {
            let action = link.action;
            let to = link.to.clone();
            if Self::navigate_with_prefetch_intents_via_host(
                host,
                window,
                &router,
                &snapshot,
                &intents,
                action,
                Some(to),
            ) {
                host.request_redraw(window);
            }
        })
    }

    pub fn prefetch_link_on_hover_change(&self, link: RouterLink) -> OnHoverChange {
        let window = self.window;
        let router: WeakModel<Router<R, H>> = self.router.downgrade();
        let intents: WeakModel<Vec<RoutePrefetchIntent<R>>> = self.intents.downgrade();
        Arc::new(move |host, _cx, hovered| {
            if !hovered {
                return;
            }

            let Some(router) = router.upgrade() else {
                return;
            };
            let Some(intents_model) = intents.upgrade() else {
                return;
            };

            let to = link.to.clone();
            let result = host
                .models_mut()
                .update(&router, |router| router.prefetch_intents_for_location(&to));

            let Ok(result) = result else {
                return;
            };
            let Ok(intents) = result else {
                host.request_redraw(window);
                return;
            };

            let _ = host.models_mut().update(&intents_model, |v| *v = intents);
            host.request_redraw(window);
        })
    }
}

#[derive(Debug, Clone)]
pub struct RouterLink {
    pub action: NavigationAction,
    pub href: Arc<str>,
    pub to: RouteLocation,
}

impl RouterLink {
    pub fn copy_href_on_activate(href: impl Into<Arc<str>>) -> OnActivate {
        let href: Arc<str> = href.into();
        Arc::new(move |host, _cx, _reason| {
            host.push_effect(Effect::ClipboardSetText {
                text: href.to_string(),
            });
        })
    }

    pub fn dispatch_command_on_activate(
        window: Option<AppWindowId>,
        command: impl Into<CommandId>,
    ) -> OnActivate {
        let command = command.into();
        Arc::new(move |host, _cx, _reason| {
            host.dispatch_command(window, command.clone());
        })
    }
}

/// Read a router snapshot model and hand a typed render closure the cloned value.
///
/// This keeps typed child inputs on the public surface and only lands on `AnyElement` at the
/// snapshot-read boundary.
pub fn router_outlet<R, T>(
    cx: &mut ElementContext<'_, App>,
    snapshot: &Model<RouterUiSnapshot<R>>,
    render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> T,
) -> AnyElement
where
    R: Clone + 'static,
    T: IntoUiElement<App>,
{
    let snap = cx
        .get_model_cloned(snapshot, Invalidation::Layout)
        .expect("router snapshot model should be readable");
    render(cx, &snap).into_element(cx)
}

/// Typed router-outlet helper with explicit diagnostics stamping at the final landing seam.
pub fn router_outlet_with_test_id<R, T>(
    cx: &mut ElementContext<'_, App>,
    snapshot: &Model<RouterUiSnapshot<R>>,
    test_id: impl Into<Arc<str>>,
    render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> T,
) -> AnyElement
where
    R: Clone + 'static,
    T: IntoUiElement<App>,
{
    router_outlet(cx, snapshot, render).test_id(test_id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterLeafStatus {
    Ready,
    Pending,
    Error { message: Arc<str> },
}

#[derive(Debug, Clone)]
pub struct RouterOutlet<R>
where
    R: Clone + 'static,
{
    snapshot: Model<RouterUiSnapshot<R>>,
    test_id: Option<Arc<str>>,
}

impl<R> RouterOutlet<R>
where
    R: Clone + 'static,
{
    pub fn new(snapshot: Model<RouterUiSnapshot<R>>) -> Self {
        Self {
            snapshot,
            test_id: None,
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    #[track_caller]
    /// Keep typed render inputs on the public surface and land on `AnyElement` only where the
    /// outlet owns test-id decoration and snapshot selection.
    pub fn into_element<T>(
        self,
        cx: &mut ElementContext<'_, App>,
        render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> T,
    ) -> AnyElement
    where
        T: IntoUiElement<App>,
    {
        let elem = router_outlet(cx, &self.snapshot, render);
        match self.test_id {
            Some(test_id) => elem.test_id(test_id),
            None => elem,
        }
    }

    #[track_caller]
    pub fn into_element_by_leaf<T, N>(
        self,
        cx: &mut ElementContext<'_, App>,
        render: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>) -> T,
        not_found: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> N,
    ) -> AnyElement
    where
        T: IntoUiElement<App>,
        N: IntoUiElement<App>,
    {
        let Self { snapshot, test_id } = self;
        let mut render = Some(render);
        let mut not_found = Some(not_found);
        let elem = router_outlet(cx, &snapshot, move |cx, snap| {
            if snap.is_not_found {
                return (not_found.take().expect("not_found should be callable"))(cx, snap)
                    .into_element(cx);
            }
            match snap.leaf_route() {
                Some(route) => (render.take().expect("render should be callable"))(cx, route, snap)
                    .into_element(cx),
                None => (not_found.take().expect("not_found should be callable"))(cx, snap)
                    .into_element(cx),
            }
        });
        match test_id {
            Some(test_id) => elem.test_id(test_id),
            None => elem,
        }
    }

    #[track_caller]
    pub fn into_element_by_leaf_with_status<Ready, Pending, Error, NotFound>(
        self,
        cx: &mut ElementContext<'_, App>,
        status: impl FnOnce(&App, &RouterUiSnapshot<R>, &R) -> RouterLeafStatus,
        ready: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>) -> Ready,
        pending: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>) -> Pending,
        error: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>, Arc<str>) -> Error,
        not_found: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> NotFound,
    ) -> AnyElement
    where
        Ready: IntoUiElement<App>,
        Pending: IntoUiElement<App>,
        Error: IntoUiElement<App>,
        NotFound: IntoUiElement<App>,
    {
        let Self { snapshot, test_id } = self;
        let mut status = Some(status);
        let mut ready = Some(ready);
        let mut pending = Some(pending);
        let mut error = Some(error);
        let mut not_found = Some(not_found);
        let elem = router_outlet(cx, &snapshot, move |cx, snap| {
            if snap.is_not_found {
                return (not_found.take().expect("not_found should be callable"))(cx, snap)
                    .into_element(cx);
            }

            let Some(route) = snap.leaf_route() else {
                return (not_found.take().expect("not_found should be callable"))(cx, snap)
                    .into_element(cx);
            };

            match (status.take().expect("status should be callable"))(&*cx.app, snap, route) {
                RouterLeafStatus::Ready => {
                    (ready.take().expect("ready should be callable"))(cx, route, snap)
                        .into_element(cx)
                }
                RouterLeafStatus::Pending => {
                    (pending.take().expect("pending should be callable"))(cx, route, snap)
                        .into_element(cx)
                }
                RouterLeafStatus::Error { message } => {
                    (error.take().expect("error should be callable"))(cx, route, snap, message)
                        .into_element(cx)
                }
            }
        });
        match test_id {
            Some(test_id) => elem.test_id(test_id),
            None => elem,
        }
    }
}

pub fn router_link<R, H, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    children: I,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    router_link_with_props(cx, store, link, PressableProps::default(), children)
}

#[allow(clippy::too_many_arguments)]
pub fn router_link_to<R, H, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    action: NavigationAction,
    route: &R,
    params: &[PathParam],
    search: SearchMap,
    fragment: Option<String>,
    children: I,
) -> Result<AnyElement, RouterBuildLocationError>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    let app: &App = &*cx.app;
    let link = store.link_to(app, action, route, params, search, fragment)?;
    Ok(router_link(cx, store, link, children))
}

pub fn router_link_to_typed_route<R, H, C, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    action: NavigationAction,
    codec: &C,
    route: &C::Route,
    children: I,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    C: RouteCodec,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    let link = store.link_to_typed_route(action, codec, route);
    router_link(cx, store, link, children)
}

pub fn router_link_to_typed_route_with_test_id<R, H, C, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    action: NavigationAction,
    codec: &C,
    route: &C::Route,
    test_id: impl Into<Arc<str>>,
    children: I,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    C: RouteCodec,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    let link = store.link_to_typed_route(action, codec, route);
    router_link_with_test_id(cx, store, link, test_id, children)
}

#[allow(clippy::too_many_arguments)]
pub fn router_link_to_with_test_id<R, H, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    action: NavigationAction,
    route: &R,
    params: &[PathParam],
    search: SearchMap,
    fragment: Option<String>,
    test_id: impl Into<Arc<str>>,
    children: I,
) -> Result<AnyElement, RouterBuildLocationError>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    let app: &App = &*cx.app;
    let link = store.link_to(app, action, route, params, search, fragment)?;
    Ok(router_link_with_test_id(cx, store, link, test_id, children))
}

/// Build a low-level router link pressable with explicit `PressableProps`.
///
/// The pressable:
/// - navigates on `pressable_on_activate`
/// - computes prefetch intents on hover and stores them in `RouterUiStore::intents_model()`
///
/// Child inputs stay typed; the explicit `AnyElement` return is the pressable/semantics landing
/// seam.
pub fn router_link_with_props<R, H, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    props: PressableProps,
    children: I,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    let mut props = props;
    props.a11y.role.get_or_insert(SemanticsRole::Link);
    props.key_activation = PressableKeyActivation::EnterOnly;

    let href = link.href.clone();
    let on_activate = store.navigate_link_on_activate(link.clone());
    let on_hover = store.prefetch_link_on_hover_change(link);
    let children: Vec<AnyElement> = children
        .into_iter()
        .map(|child| child.into_element(cx))
        .collect();

    cx.pressable_with_id_props(|cx, _state, _id| {
        cx.pressable_on_activate(on_activate);
        cx.pressable_on_hover_change(on_hover);
        (props, children)
    })
    .attach_semantics(SemanticsDecoration::default().url(href.clone()).value(href))
}

/// Build a router link pressable and stamp a diagnostics `test_id`.
///
/// Prefer this in demos and automated `fretboard diag` scripts.
pub fn router_link_with_test_id<R, H, I, T>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    test_id: impl Into<Arc<str>>,
    children: I,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<App>,
{
    let mut props = PressableProps::default();
    props.a11y.test_id = Some(test_id.into());
    router_link_with_props(cx, store, link, props, children)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouterLinkContextMenuAction {
    CopyLink,
    OpenInNewWindow,
}

#[derive(Debug, Clone)]
pub struct RouterLinkContextMenuItem {
    pub action: RouterLinkContextMenuAction,
    pub label: Arc<str>,
}

impl RouterLink {
    pub fn default_context_menu_items(&self) -> [RouterLinkContextMenuItem; 2] {
        [
            RouterLinkContextMenuItem {
                action: RouterLinkContextMenuAction::CopyLink,
                label: Arc::from("Copy link"),
            },
            RouterLinkContextMenuItem {
                action: RouterLinkContextMenuAction::OpenInNewWindow,
                label: Arc::from("Open in new window"),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::{RouterLink, RouterOutlet, RouterUiSnapshot, RouterUiStore, router_link};
    use fret_app::App;
    use fret_router::{
        MemoryHistory, RouteCodec, RouteHooks, RouteLocation, RouteNode, RoutePrefetchIntent,
        RouteSearchTable, RouteTree, Router,
    };
    use fret_runtime::{Effect, Model, ModelStore, TimerToken, WindowCommandAvailabilityService};
    use fret_ui::action::{ActionCx, UiActionHost, UiFocusActionHost};
    use fret_ui::element::AnyElement;
    use fret_ui::{ElementContext, GlobalElementId};
    use fret_ui_kit::ui;
    use std::sync::Arc;

    const APP_RS: &str = include_str!("app.rs");
    const LIB_RS: &str = include_str!("lib.rs");

    fn action_cx(window: fret_core::AppWindowId) -> ActionCx {
        ActionCx {
            window,
            target: GlobalElementId(0x1),
        }
    }

    struct TestFocusHost<'a> {
        app: &'a mut App,
    }

    impl UiActionHost for TestFocusHost<'_> {
        fn models_mut(&mut self) -> &mut ModelStore {
            self.app.models_mut()
        }

        fn push_effect(&mut self, effect: Effect) {
            self.app.push_effect(effect);
        }

        fn request_redraw(&mut self, window: fret_core::AppWindowId) {
            self.app.request_redraw(window);
        }

        fn next_timer_token(&mut self) -> TimerToken {
            self.app.next_timer_token()
        }

        fn next_clipboard_token(&mut self) -> fret_runtime::ClipboardToken {
            self.app.next_clipboard_token()
        }

        fn next_share_sheet_token(&mut self) -> fret_runtime::ShareSheetToken {
            self.app.next_share_sheet_token()
        }

        fn set_router_command_availability(
            &mut self,
            window: fret_core::AppWindowId,
            can_back: bool,
            can_forward: bool,
        ) {
            self.app
                .with_global_mut(WindowCommandAvailabilityService::default, |svc, _app| {
                    svc.set_router_availability(window, can_back, can_forward);
                });
        }
    }

    impl UiFocusActionHost for TestFocusHost<'_> {
        fn request_focus(&mut self, _target: GlobalElementId) {}
    }

    #[allow(dead_code)]
    fn router_outlet_by_leaf_accepts_builder_children(
        cx: &mut ElementContext<'_, App>,
        snapshot: Model<RouterUiSnapshot<CompileRoute>>,
    ) -> AnyElement {
        RouterOutlet::new(snapshot).into_element_by_leaf(
            cx,
            |_cx, _route, snap| {
                let depth = snap.match_depth();
                ui::container(move |cx| ui::children![cx; ui::text(format!("depth={depth}"))])
                    .w_full()
            },
            |_cx, _snap| ui::text("not found"),
        )
    }

    #[allow(dead_code)]
    fn router_link_accepts_builder_children(
        cx: &mut ElementContext<'_, App>,
        store: &RouterUiStore<CompileRoute, MemoryHistory>,
        link: RouterLink,
    ) -> AnyElement {
        router_link(cx, store, link, [ui::text("go")])
    }

    #[allow(dead_code)]
    #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
    enum CompileRoute {
        Home,
    }

    #[test]
    fn crate_docs_keep_router_ui_positioned_as_thin_adoption_layer() {
        assert!(LIB_RS.contains("This crate intentionally provides a thin layer:"));
        assert!(
            LIB_RS.contains(
                "Policy-heavy behavior remains in apps and higher-level ecosystem crates."
            )
        );
    }

    #[test]
    fn router_ui_surface_does_not_grow_a_second_app_runtime() {
        let public_surface = LIB_RS
            .split("#[cfg(test)]")
            .next()
            .expect("router-ui source should contain a test module split");
        let normalized = public_surface.split_whitespace().collect::<String>();

        assert!(public_surface.contains("pub mod app;"));
        assert!(public_surface.contains("pub fn register_router_commands("));
        assert!(public_surface.contains("pub struct RouterUiStore"));
        assert!(public_surface.contains("pub struct RouterOutlet"));
        assert!(public_surface.contains("pub fn router_link<"));
        assert!(public_surface.contains("pub fn router_outlet<"));
        assert!(public_surface.contains("pub fn into_element<T>("));
        assert!(public_surface.contains("pub fn into_element_by_leaf<T, N>("));
        assert!(
            public_surface.contains(
                "pub fn into_element_by_leaf_with_status<Ready, Pending, Error, NotFound>("
            )
        );

        assert!(!public_surface.contains("pub fn install_app("));
        assert!(!public_surface.contains("pub fn install(app: &mut App)"));
        assert!(!public_surface.contains("pub struct RouterApp"));
        assert!(!public_surface.contains("pub trait RouterApp"));
        assert!(!public_surface.contains("FretApp::"));
        assert!(!public_surface.contains("AppUi<"));
        assert!(!public_surface.contains("ViewCx<"));
        assert!(!public_surface.contains("Plugin"));
        assert!(!public_surface.contains("RouterOutletIntoElement"));
        assert!(!public_surface.contains("pub fn router_outlet_ui("));
        assert!(!public_surface.contains("pub fn router_outlet_with_test_id_ui("));
        assert!(!public_surface.contains("pub fn into_element_ui("));
        assert!(!public_surface.contains("pub fn into_element_by_leaf_ui("));
        assert!(!public_surface.contains("pub fn into_element_by_leaf_with_status_ui("));
        assert!(!normalized.contains("IntoIterator<Item=AnyElement>"));
        assert!(normalized.contains("pubfnrouter_link<R,H,I,T>("));
        assert!(normalized.contains("pubfnrouter_link_with_props<R,H,I,T>("));
        assert!(APP_RS.contains("pub fn install(app: &mut App)"));
    }

    #[test]
    fn router_ui_helpers_keep_typed_inputs_and_explicit_landing_returns() {
        let public_surface = LIB_RS
            .split("#[cfg(test)]")
            .next()
            .expect("router-ui source should contain a test module split");
        let normalized = public_surface.split_whitespace().collect::<String>();

        assert!(LIB_RS.contains("This keeps typed child inputs on the public surface"));
        assert!(LIB_RS.contains("snapshot-read boundary."));
        assert!(LIB_RS.contains(
            "Child inputs stay typed; the explicit `AnyElement` return is the pressable/semantics landing seam."
        ));

        assert!(normalized.contains("pubfnrouter_outlet<R,T>("));
        assert!(
            normalized
                .contains("render:implFnOnce(&mutElementContext<'_,App>,&RouterUiSnapshot<R>)->T,")
        );
        assert!(normalized.contains("T:IntoUiElement<App>,"));
        assert!(!normalized.contains(
            "render:implFnOnce(&mutElementContext<'_,App>,&RouterUiSnapshot<R>)->AnyElement,"
        ));

        assert!(normalized.contains("pubfnrouter_link_with_props<R,H,I,T>("));
        assert!(normalized.contains("I:IntoIterator<Item=T>,"));
        assert!(normalized.contains("T:IntoUiElement<App>,"));
        assert!(!normalized.contains("I:IntoIterator<Item=AnyElement>"));
    }

    #[test]
    fn router_ui_store_updates_snapshot_on_navigation() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let store = RouterUiStore::new(&mut app, window, router);

        let update = store
            .navigate(
                &mut app,
                fret_router::NavigationAction::Push,
                Some(RouteLocation::parse("/settings")),
            )
            .expect("navigate should succeed");
        assert!(update.changed());

        let snapshot = app
            .models()
            .get_cloned(&store.snapshot_model())
            .expect("snapshot should be readable");
        assert_eq!(snapshot.location.to_url(), "/settings");
        assert_eq!(snapshot.matches.len(), 2);
    }

    #[test]
    fn router_ui_store_init_returns_prefetch_intents() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/settings"));
        let mut router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        router.route_hooks_mut().insert(
            RouteId::Settings,
            RouteHooks {
                before_load: None,
                loader: Some(Arc::new(|ctx| {
                    vec![RoutePrefetchIntent {
                        route: ctx.matched.route,
                        namespace: "fret-router-ui.tests.init_prefetch",
                        location: ctx.to.clone(),
                        extra: None,
                    }]
                })),
            },
        );

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let store = RouterUiStore::new(&mut app, window, router);

        let update = store
            .init_with_prefetch_intents(&mut app)
            .expect("init should succeed");
        assert_eq!(update.intents.len(), 1);

        let snapshot = app
            .models()
            .get_cloned(&store.snapshot_model())
            .expect("snapshot should be readable");
        assert_eq!(snapshot.location.to_url(), "/settings");
        assert_eq!(snapshot.matches.len(), 2);
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum TypedRoute {
        Home,
        Settings,
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TypedRouteDecodeError {
        NoMatch,
    }

    struct TypedRouteCodec;

    impl RouteCodec for TypedRouteCodec {
        type Route = TypedRoute;
        type Error = TypedRouteDecodeError;

        fn encode(&self, route: &Self::Route) -> RouteLocation {
            match route {
                TypedRoute::Home => RouteLocation::from_path("/"),
                TypedRoute::Settings => RouteLocation::parse("settings///"),
            }
        }

        fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
            match location.path.as_str() {
                "/" => Ok(TypedRoute::Home),
                "/settings" => Ok(TypedRoute::Settings),
                _ => Err(TypedRouteDecodeError::NoMatch),
            }
        }
    }

    #[test]
    fn router_ui_store_builds_typed_route_links_via_codec() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let store = RouterUiStore::new(&mut app, window, router);

        let link = store.link_to_typed_route(
            fret_router::NavigationAction::Push,
            &TypedRouteCodec,
            &TypedRoute::Settings,
        );
        assert_eq!(link.href.as_ref(), "/settings");
        assert_eq!(link.to.to_url(), "/settings");
    }

    #[test]
    fn router_ui_store_new_syncs_command_availability_from_history() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let mut history = MemoryHistory::new(RouteLocation::parse("/"));
        assert!(history.push(RouteLocation::parse("/settings")));
        let router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let _store = RouterUiStore::new(&mut app, window, router);

        let availability = app
            .global::<WindowCommandAvailabilityService>()
            .and_then(|svc| svc.snapshot(window).copied())
            .expect("router command availability should be published");
        assert!(availability.router_can_back);
        assert!(!availability.router_can_forward);
    }

    #[test]
    fn router_ui_store_history_action_updates_snapshot_and_availability() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Settings,
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Settings, "settings").unwrap()]),
        ));
        let search_table = Arc::new(RouteSearchTable::new());
        let mut history = MemoryHistory::new(RouteLocation::parse("/"));
        assert!(history.push(RouteLocation::parse("/settings")));
        let router = Router::new(
            tree,
            search_table,
            fret_router::SearchValidationMode::Strict,
            history,
        )
        .expect("router should build");

        let mut app = App::new();
        let window = fret_core::AppWindowId::default();
        let store = RouterUiStore::new(&mut app, window, router);
        let back = store.back_on_action();

        {
            let mut host = TestFocusHost { app: &mut app };
            assert!(back(&mut host, action_cx(window)));
        }

        let snapshot = app
            .models()
            .get_cloned(&store.snapshot_model())
            .expect("snapshot should be readable");
        assert_eq!(snapshot.location.to_url(), "/");

        let availability = app
            .global::<WindowCommandAvailabilityService>()
            .and_then(|svc| svc.snapshot(window).copied())
            .expect("router command availability should be published");
        assert!(!availability.router_can_back);
        assert!(availability.router_can_forward);
    }
}
