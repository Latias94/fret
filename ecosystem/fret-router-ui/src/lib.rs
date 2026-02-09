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
use fret_core::{AppWindowId, KeyCode, Modifiers};
use fret_router::{
    HistoryAdapter, NavigationAction, PathParam, RouteLocation, RouteMatchSnapshot,
    RoutePrefetchIntent, RouteSearchValidationFailure, Router, RouterBuildLocationError,
    RouterEvent, RouterTransition, RouterUpdate, RouterUpdateWithPrefetchIntents, SearchMap,
};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, Effect, KeyChord,
    Model, PlatformFilter, WeakModel, WhenExpr, WindowCommandAvailabilityService,
};
use fret_ui::action::{OnActivate, OnHoverChange};
use fret_ui::element::AnyElement;
use fret_ui::element::{PressableProps, SemanticsDecoration};
use fret_ui::{ElementContext, Invalidation};

trait AnyElementTestIdExt {
    fn test_id(self, test_id: impl Into<Arc<str>>) -> AnyElement;
}

impl AnyElementTestIdExt for AnyElement {
    fn test_id(self, test_id: impl Into<Arc<str>>) -> AnyElement {
        self.attach_semantics(SemanticsDecoration::default().test_id(test_id))
    }
}

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
        Self {
            window,
            router,
            snapshot,
            intents,
        }
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

    fn apply_update(
        &self,
        app: &mut App,
        update: &RouterUpdate,
        intents: &[RoutePrefetchIntent<R>],
    ) {
        if !update.changed() {
            return;
        }

        let (next, can_back, can_forward) = app
            .models()
            .read(&self.router, |router| {
                let next = RouterUiSnapshot::from_state(router.state());
                let history = router.history();
                let can_back = history.can_navigate(NavigationAction::Back);
                let can_forward = history.can_navigate(NavigationAction::Forward);
                (next, can_back, can_forward)
            })
            .expect("router model should be readable");
        let _ = app.models_mut().update(&self.snapshot, |v| *v = next);
        let intents = intents.to_vec();
        let _ = app.models_mut().update(&self.intents, |v| *v = intents);
        app.with_global_mut(WindowCommandAvailabilityService::default, |svc, _app| {
            svc.set_router_availability(self.window, can_back, can_forward);
        });
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
            let Some(router) = router.upgrade() else {
                return;
            };
            let Some(snapshot) = snapshot.upgrade() else {
                return;
            };
            let Some(intents_model) = intents.upgrade() else {
                return;
            };

            let action = link.action;
            let to = link.to.clone();

            let result = host.models_mut().update(&router, |router| {
                let update = router.navigate_with_prefetch_intents(action, Some(to))?;
                let snapshot = RouterUiSnapshot::from_state(router.state());
                let history = router.history();
                let can_back = history.can_navigate(NavigationAction::Back);
                let can_forward = history.can_navigate(NavigationAction::Forward);
                Ok::<_, RouteSearchValidationFailure>((update, snapshot, can_back, can_forward))
            });

            let Ok(result) = result else {
                return;
            };
            let Ok((update, next_snapshot, can_back, can_forward)) = result else {
                host.request_redraw(window);
                return;
            };

            if !update.update.changed() {
                return;
            }

            let _ = host.models_mut().update(&snapshot, |v| *v = next_snapshot);
            let _ = host
                .models_mut()
                .update(&intents_model, |v| *v = update.intents.clone());
            host.set_router_command_availability(window, can_back, can_forward);
            host.request_redraw(window);
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

pub fn router_outlet<R>(
    cx: &mut ElementContext<'_, App>,
    snapshot: &Model<RouterUiSnapshot<R>>,
    render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> AnyElement,
) -> AnyElement
where
    R: Clone + 'static,
{
    let snap = cx
        .get_model_cloned(snapshot, Invalidation::Layout)
        .expect("router snapshot model should be readable");
    render(cx, &snap)
}

pub fn router_outlet_with_test_id<R>(
    cx: &mut ElementContext<'_, App>,
    snapshot: &Model<RouterUiSnapshot<R>>,
    test_id: impl Into<Arc<str>>,
    render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> AnyElement,
) -> AnyElement
where
    R: Clone + 'static,
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

    pub fn into_element(
        self,
        cx: &mut ElementContext<'_, App>,
        render: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> AnyElement,
    ) -> AnyElement {
        let elem = router_outlet(cx, &self.snapshot, render);
        match self.test_id {
            Some(test_id) => elem.test_id(test_id),
            None => elem,
        }
    }

    pub fn into_element_by_leaf(
        self,
        cx: &mut ElementContext<'_, App>,
        render: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>) -> AnyElement,
        not_found: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> AnyElement,
    ) -> AnyElement {
        let mut render = Some(render);
        let mut not_found = Some(not_found);
        self.into_element(cx, move |cx, snap| {
            if snap.is_not_found {
                return (not_found.take().expect("not_found should be callable"))(cx, snap);
            }
            match snap.leaf_route() {
                Some(route) => (render.take().expect("render should be callable"))(cx, route, snap),
                None => (not_found.take().expect("not_found should be callable"))(cx, snap),
            }
        })
    }

    pub fn into_element_by_leaf_with_status(
        self,
        cx: &mut ElementContext<'_, App>,
        status: impl FnOnce(&App, &RouterUiSnapshot<R>, &R) -> RouterLeafStatus,
        ready: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>) -> AnyElement,
        pending: impl FnOnce(&mut ElementContext<'_, App>, &R, &RouterUiSnapshot<R>) -> AnyElement,
        error: impl FnOnce(
            &mut ElementContext<'_, App>,
            &R,
            &RouterUiSnapshot<R>,
            Arc<str>,
        ) -> AnyElement,
        not_found: impl FnOnce(&mut ElementContext<'_, App>, &RouterUiSnapshot<R>) -> AnyElement,
    ) -> AnyElement {
        let mut status = Some(status);
        let mut ready = Some(ready);
        let mut pending = Some(pending);
        let mut error = Some(error);
        let mut not_found = Some(not_found);
        self.into_element(cx, move |cx, snap| {
            if snap.is_not_found {
                return (not_found.take().expect("not_found should be callable"))(cx, snap);
            }

            let Some(route) = snap.leaf_route() else {
                return (not_found.take().expect("not_found should be callable"))(cx, snap);
            };

            match (status.take().expect("status should be callable"))(&*cx.app, snap, route) {
                RouterLeafStatus::Ready => {
                    (ready.take().expect("ready should be callable"))(cx, route, snap)
                }
                RouterLeafStatus::Pending => {
                    (pending.take().expect("pending should be callable"))(cx, route, snap)
                }
                RouterLeafStatus::Error { message } => {
                    (error.take().expect("error should be callable"))(cx, route, snap, message)
                }
            }
        })
    }
}

pub fn router_link<R, H>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    router_link_with_props(cx, store, link, PressableProps::default(), children)
}

pub fn router_link_to<R, H>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    action: NavigationAction,
    route: &R,
    params: &[PathParam],
    search: SearchMap,
    fragment: Option<String>,
    children: impl IntoIterator<Item = AnyElement>,
) -> Result<AnyElement, RouterBuildLocationError>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    let app: &App = &*cx.app;
    let link = store.link_to(app, action, route, params, search, fragment)?;
    Ok(router_link(cx, store, link, children))
}

pub fn router_link_to_with_test_id<R, H>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    action: NavigationAction,
    route: &R,
    params: &[PathParam],
    search: SearchMap,
    fragment: Option<String>,
    test_id: impl Into<Arc<str>>,
    children: impl IntoIterator<Item = AnyElement>,
) -> Result<AnyElement, RouterBuildLocationError>
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
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
pub fn router_link_with_props<R, H>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    props: PressableProps,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
{
    let on_activate = store.navigate_link_on_activate(link.clone());
    let on_hover = store.prefetch_link_on_hover_change(link);
    let children: Vec<AnyElement> = children.into_iter().collect();

    cx.pressable_with_id_props(|cx, _state, _id| {
        cx.pressable_on_activate(on_activate);
        cx.pressable_on_hover_change(on_hover);
        (props, children)
    })
}

/// Build a router link pressable and stamp a diagnostics `test_id`.
///
/// Prefer this in demos and automated `fretboard diag` scripts.
pub fn router_link_with_test_id<R, H>(
    cx: &mut ElementContext<'_, App>,
    store: &RouterUiStore<R, H>,
    link: RouterLink,
    test_id: impl Into<Arc<str>>,
    children: impl IntoIterator<Item = AnyElement>,
) -> AnyElement
where
    R: Clone + Eq + Hash + 'static,
    H: HistoryAdapter + 'static,
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
    use super::RouterUiStore;
    use fret_app::App;
    use fret_router::{
        MemoryHistory, RouteHooks, RouteLocation, RouteNode, RoutePrefetchIntent, RouteSearchTable,
        RouteTree, Router,
    };
    use std::sync::Arc;

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
}
