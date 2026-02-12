use fret_app::{App, CommandId};
use fret_core::AppWindowId;
use fret_query::{QueryPolicy, with_query_client};
use fret_router::{
    NamespaceInvalidationRule, NavigationAction, RouteChangePolicy, RouteHooks, RouteLocation,
    RouteNode, RouteSearchTable, RouteTree, Router, RouterUpdate, RouterUpdateWithPrefetchIntents,
    SearchValidationMode, collect_invalidated_namespaces, prefetch_intent_query_key,
};
use fret_runtime::WindowCommandEnabledService;
use std::sync::Arc;

use crate::spec::*;

const UI_GALLERY_PAGE_CONTENT_NS: &str = "fret.ui_gallery.page_content.v1";
const UI_GALLERY_NAV_INDEX_NS: &str = "fret.ui_gallery.nav_index.v1";

#[cfg(target_arch = "wasm32")]
pub(super) type UiGalleryHistory = fret_router::WebHistoryAdapter;

#[cfg(not(target_arch = "wasm32"))]
pub(super) type UiGalleryHistory = fret_router::MemoryHistory;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(super) enum UiGalleryRouteId {
    Root,
    Gallery,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct UiGalleryPagePrefetchSeed {
    selected_page: Arc<str>,
}

fn route_location_for_page(from: &RouteLocation, page: &Arc<str>) -> RouteLocation {
    let location = RouteLocation::from_path("/gallery")
        .with_query_value("page", Some(page.to_string()))
        .with_query_value("source", Some("nav".to_string()));

    if let Some(demo) = from.query_value("demo") {
        location.with_query_value("demo", Some(demo.to_string()))
    } else {
        location
    }
}

pub(super) fn page_from_gallery_location(location: &RouteLocation) -> Option<Arc<str>> {
    let page = location.query_value("page")?;
    page_spec(page).is_some().then_some(Arc::<str>::from(page))
}

pub(super) fn build_ui_gallery_page_router() -> Router<UiGalleryRouteId, UiGalleryHistory> {
    let tree = Arc::new(RouteTree::new(
        RouteNode::new(UiGalleryRouteId::Root, "/")
            .expect("root route should build")
            .with_children(vec![
                RouteNode::new(UiGalleryRouteId::Gallery, "gallery")
                    .expect("gallery route should build"),
            ]),
    ));

    let search_table = Arc::new(RouteSearchTable::new());

    #[cfg(target_arch = "wasm32")]
    let history = UiGalleryHistory::new().expect("web history adapter should resolve a location");

    #[cfg(not(target_arch = "wasm32"))]
    let history = UiGalleryHistory::new(RouteLocation::parse("/"));

    let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
        .expect("ui gallery router should build");

    router.route_hooks_mut().insert(
        UiGalleryRouteId::Gallery,
        RouteHooks {
            before_load: None,
            loader: Some(Arc::new(|ctx| {
                vec![fret_router::RoutePrefetchIntent {
                    route: ctx.matched.route,
                    namespace: UI_GALLERY_PAGE_CONTENT_NS,
                    location: ctx.to.clone(),
                    extra: None,
                }]
            })),
        },
    );

    router
}

pub(super) fn apply_page_route_side_effects_via_router(
    app: &mut App,
    window: AppWindowId,
    action: NavigationAction,
    current_page: Arc<str>,
    router: &mut Router<UiGalleryRouteId, UiGalleryHistory>,
) {
    let current_route = route_location_for_page(&router.state().location, &current_page);
    let update = router.navigate_with_prefetch_intents(action, Some(current_route.canonicalized()));
    apply_page_router_update_side_effects(app, window, current_page, router, update);
}

#[cfg(not(target_arch = "wasm32"))]
fn sync_gallery_page_history_command_enabled(
    app: &mut App,
    window: AppWindowId,
    history: &UiGalleryHistory,
) {
    let can_back = history.can_back();
    let can_forward = history.can_forward();

    let cmd_back = CommandId::new(CMD_GALLERY_PAGE_BACK);
    let cmd_forward = CommandId::new(CMD_GALLERY_PAGE_FORWARD);

    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        if can_back {
            svc.clear_command(window, &cmd_back);
        } else {
            svc.set_enabled(window, cmd_back.clone(), false);
        }

        if can_forward {
            svc.clear_command(window, &cmd_forward);
        } else {
            svc.set_enabled(window, cmd_forward.clone(), false);
        }
    });
}

#[cfg(target_arch = "wasm32")]
fn sync_gallery_page_history_command_enabled(
    app: &mut App,
    window: AppWindowId,
    _history: &UiGalleryHistory,
) {
    let cmd_back = CommandId::new(CMD_GALLERY_PAGE_BACK);
    let cmd_forward = CommandId::new(CMD_GALLERY_PAGE_FORWARD);

    app.with_global_mut(WindowCommandEnabledService::default, |svc, _app| {
        svc.clear_command(window, &cmd_back);
        svc.clear_command(window, &cmd_forward);
    });
}

pub(super) fn apply_page_router_update_side_effects(
    app: &mut App,
    window: AppWindowId,
    current_page: Arc<str>,
    router: &mut Router<UiGalleryRouteId, UiGalleryHistory>,
    update: Result<
        RouterUpdateWithPrefetchIntents<UiGalleryRouteId>,
        fret_router::RouteSearchValidationFailure,
    >,
) {
    sync_gallery_page_history_command_enabled(app, window, router.history());

    let Ok(update) = update else {
        return;
    };

    let RouterUpdateWithPrefetchIntents { update, intents } = update;

    let invalidated = if let RouterUpdate::Changed(transition) = &update {
        collect_invalidated_namespaces(
            &transition.from,
            &transition.to,
            &[
                NamespaceInvalidationRule::new(
                    UI_GALLERY_PAGE_CONTENT_NS,
                    RouteChangePolicy::PathOrQueryChanged,
                ),
                NamespaceInvalidationRule::new(
                    UI_GALLERY_NAV_INDEX_NS,
                    RouteChangePolicy::QueryChanged,
                ),
            ],
        )
    } else {
        Vec::new()
    };

    if invalidated.is_empty() && intents.is_empty() {
        return;
    }

    let _ = with_query_client(app, |client, app| {
        for namespace in invalidated {
            client.invalidate_namespace(namespace);
        }

        for intent in intents {
            if intent.namespace != UI_GALLERY_PAGE_CONTENT_NS {
                continue;
            }

            let seed = UiGalleryPagePrefetchSeed {
                selected_page: current_page.clone(),
            };
            let key = prefetch_intent_query_key::<String, _>(&intent);
            let policy = QueryPolicy::default();
            let _ = client.prefetch(app, window, key, policy, move |_token| {
                Ok::<String, fret_query::QueryError>(format!(
                    "ui_gallery.page_prefetch:{}",
                    seed.selected_page
                ))
            });
        }
    });
}
