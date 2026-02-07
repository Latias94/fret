use std::hash::Hash;
use std::sync::Arc;

use crate::{
    MemoryHistory, NavigationAction, RouteLocation, RouteMatchResultWithSearch, RouteSearchTable,
    RouteSearchValidationFailure, RouteTree, SearchValidationMode,
};

pub trait HistoryAdapter {
    fn current(&self) -> &RouteLocation;
    fn refresh(&mut self) {}
    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool;
}

impl HistoryAdapter for MemoryHistory {
    fn current(&self) -> &RouteLocation {
        MemoryHistory::current(self)
    }

    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool {
        MemoryHistory::navigate(self, action, target)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RouterTransitionCause {
    Navigate { action: NavigationAction },
    Redirect { action: NavigationAction },
    Sync,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct RouterBlockReason {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouterTransition {
    pub cause: RouterTransitionCause,
    pub from: RouteLocation,
    pub to: RouteLocation,
    pub redirect_chain: Vec<RouteLocation>,
    pub blocked_by: Option<RouterBlockReason>,
}

impl RouterTransition {
    pub fn navigate(action: NavigationAction, from: RouteLocation, to: RouteLocation) -> Self {
        Self {
            cause: RouterTransitionCause::Navigate { action },
            from,
            to,
            redirect_chain: Vec::new(),
            blocked_by: None,
        }
    }

    pub fn sync(from: RouteLocation, to: RouteLocation) -> Self {
        Self {
            cause: RouterTransitionCause::Sync,
            from,
            to,
            redirect_chain: Vec::new(),
            blocked_by: None,
        }
    }

    pub fn action(&self) -> Option<NavigationAction> {
        match self.cause {
            RouterTransitionCause::Navigate { action } => Some(action),
            RouterTransitionCause::Redirect { action } => Some(action),
            RouterTransitionCause::Sync => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterUpdate {
    NoChange,
    Changed(RouterTransition),
}

impl RouterUpdate {
    pub fn changed(&self) -> bool {
        match self {
            Self::NoChange => false,
            Self::Changed(_) => true,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RouterEvent<R> {
    Transitioned {
        transition: RouterTransition,
        state: RouterState<R>,
    },
}

#[derive(Debug, Clone)]
pub struct RouterState<R> {
    pub location: RouteLocation,
    pub matches: Vec<RouteMatchSnapshot<R>>,
    pub is_not_found: bool,
    pub last_transition: Option<RouterTransition>,
}

#[derive(Debug, Clone)]
pub struct RouteMatchSnapshot<R> {
    pub route: R,
    pub matched_path: String,
    pub params: Vec<crate::PathParam>,
    pub search: crate::SearchMap,
    pub search_error: Option<crate::SearchValidationError>,
}

impl<R> RouterState<R> {
    fn from_match_result(result: RouteMatchResultWithSearch<'_, R>) -> Self
    where
        R: Clone,
    {
        Self {
            location: result.location,
            matches: result
                .matches
                .into_iter()
                .map(|entry| RouteMatchSnapshot {
                    route: entry.route.clone(),
                    matched_path: entry.matched_path,
                    params: entry.params,
                    search: entry.search,
                    search_error: entry.search_error,
                })
                .collect(),
            is_not_found: result.is_not_found,
            last_transition: None,
        }
    }
}

pub struct Router<R, H> {
    tree: Arc<RouteTree<R>>,
    search_table: Arc<RouteSearchTable<R>>,
    search_mode: SearchValidationMode,
    history: H,
    state: RouterState<R>,
    events: Vec<RouterEvent<R>>,
}

impl<R, H> Router<R, H>
where
    R: Clone + Hash + Eq,
    H: HistoryAdapter,
{
    pub fn new(
        tree: Arc<RouteTree<R>>,
        search_table: Arc<RouteSearchTable<R>>,
        search_mode: SearchValidationMode,
        mut history: H,
    ) -> Result<Self, RouteSearchValidationFailure> {
        history.refresh();
        let initial_location = history.current().clone();
        let state = RouterState::from_match_result(tree.match_routes_with_search(
            &initial_location,
            search_table.as_ref(),
            search_mode,
        )?);

        Ok(Self {
            tree,
            search_table,
            search_mode,
            history,
            state,
            events: Vec::new(),
        })
    }

    pub fn state(&self) -> &RouterState<R> {
        &self.state
    }

    pub fn history(&self) -> &H {
        &self.history
    }

    pub fn history_mut(&mut self) -> &mut H {
        &mut self.history
    }

    pub fn take_events(&mut self) -> Vec<RouterEvent<R>> {
        std::mem::take(&mut self.events)
    }

    pub fn sync(&mut self) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let next_location = self.history.current().clone();
        if next_location == self.state.location {
            return Ok(RouterUpdate::NoChange);
        }

        let next_state = RouterState::from_match_result(self.tree.match_routes_with_search(
            &next_location,
            self.search_table.as_ref(),
            self.search_mode,
        )?);

        let transition =
            RouterTransition::sync(self.state.location.clone(), next_state.location.clone());
        self.state = next_state;
        self.state.last_transition = Some(transition.clone());
        self.events.push(RouterEvent::Transitioned {
            transition: transition.clone(),
            state: self.state.clone(),
        });

        Ok(RouterUpdate::Changed(transition))
    }

    pub fn navigate(
        &mut self,
        action: NavigationAction,
        target: Option<RouteLocation>,
    ) -> Result<RouterUpdate, RouteSearchValidationFailure> {
        self.history.refresh();
        let from = self.history.current().clone();

        let changed = self.history.navigate(action, target);
        if !changed {
            return Ok(RouterUpdate::NoChange);
        }

        self.history.refresh();
        let to = self.history.current().clone();
        let mut next_state = RouterState::from_match_result(self.tree.match_routes_with_search(
            &to,
            self.search_table.as_ref(),
            self.search_mode,
        )?);
        let transition = RouterTransition::navigate(action, from, to);
        next_state.last_transition = Some(transition.clone());

        self.state = next_state;
        self.events.push(RouterEvent::Transitioned {
            transition: transition.clone(),
            state: self.state.clone(),
        });

        Ok(RouterUpdate::Changed(transition))
    }
}

#[cfg(test)]
mod tests {
    use super::Router;
    use crate::{
        MemoryHistory, NavigationAction, RouteLocation, RouteNode, RouteSearchTable, RouteTree,
        SearchMap, SearchValidationError, SearchValidationMode,
    };
    use std::sync::Arc;

    #[test]
    fn router_navigate_updates_state_and_transition() {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        enum RouteId {
            Root,
            Gallery,
        }

        fn validate_root(
            _location: &RouteLocation,
            search: &SearchMap,
        ) -> Result<SearchMap, SearchValidationError> {
            Ok(search.clone().with("lang", Some("en".to_string())))
        }

        let tree = Arc::new(RouteTree::new(
            RouteNode::new(RouteId::Root, "/")
                .unwrap()
                .with_children(vec![RouteNode::new(RouteId::Gallery, "gallery").unwrap()]),
        ));

        let mut search_table = RouteSearchTable::new();
        search_table.insert(RouteId::Root, validate_root);
        let search_table = Arc::new(search_table);

        let history = MemoryHistory::new(RouteLocation::parse("/"));
        let mut router = Router::new(tree, search_table, SearchValidationMode::Lenient, history)
            .expect("router should build");

        assert_eq!(router.state().location.to_url(), "/");
        assert!(
            router
                .navigate(
                    NavigationAction::Push,
                    Some(RouteLocation::parse("/gallery?lang=zh"))
                )
                .expect("navigate should succeed")
                .changed()
        );

        assert_eq!(router.state().location.to_url(), "/gallery?lang=zh");
        assert_eq!(router.state().matches.len(), 2);
        assert_eq!(router.state().matches[0].search.first("lang"), Some("en"));

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert_eq!(transition.action(), Some(NavigationAction::Push));
        assert_eq!(transition.from.to_url(), "/");
        assert_eq!(transition.to.to_url(), "/gallery?lang=zh");

        let events = router.take_events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn router_sync_records_transition_and_event_when_location_changes() {
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
        let mut router = Router::new(tree, search_table, SearchValidationMode::Strict, history)
            .expect("router should build");

        assert!(router.history_mut().push(RouteLocation::parse("/settings")));
        let update = router.sync().expect("sync should succeed");
        assert!(update.changed());

        let transition = router
            .state()
            .last_transition
            .as_ref()
            .expect("transition should exist");
        assert!(matches!(
            transition.cause,
            super::RouterTransitionCause::Sync
        ));

        let events = router.take_events();
        assert_eq!(events.len(), 1);
    }

    #[test]
    fn memory_history_adapter_is_usable_via_trait() {
        let mut history = MemoryHistory::new(RouteLocation::parse("/"));
        assert!(history.navigate(NavigationAction::Push, Some(RouteLocation::parse("/next"))));
        assert_eq!(history.current().to_url(), "/next");
    }
}
