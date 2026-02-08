use crate::{HistoryAdapter, NavigationAction, RouteLocation, format_query_pairs};

fn route_location_to_web_parts(location: &RouteLocation) -> (String, String, String) {
    let canonical = location.canonicalized();
    let pathname = canonical.path;
    let search = format_query_pairs(canonical.query.as_slice());
    let hash = canonical
        .fragment
        .as_deref()
        .filter(|fragment| !fragment.is_empty())
        .map(|fragment| format!("#{fragment}"))
        .unwrap_or_default();

    (pathname, search, hash)
}

#[cfg(all(target_arch = "wasm32", feature = "web-history"))]
#[derive(Debug, Clone)]
pub struct WebHistoryAdapter {
    base_path: Option<String>,
    location: RouteLocation,
}

#[cfg(all(target_arch = "wasm32", feature = "web-history"))]
impl WebHistoryAdapter {
    pub fn new() -> Option<Self> {
        let location = crate::web::current_route_location()?;
        Some(Self {
            base_path: None,
            location,
        })
    }

    pub fn in_base_path(base_path: impl Into<String>) -> Option<Self> {
        let base_path = base_path.into();
        let location = crate::web::current_route_location_in_base_path(base_path.as_str())?;
        Some(Self {
            base_path: Some(base_path),
            location,
        })
    }

    fn refresh_from_window(&mut self) -> bool {
        let next = if let Some(base_path) = self.base_path.as_deref() {
            crate::web::current_route_location_in_base_path(base_path)
        } else {
            crate::web::current_route_location()
        };

        let Some(next) = next else {
            return false;
        };

        if next != self.location {
            self.location = next;
            true
        } else {
            false
        }
    }
}

#[cfg(all(target_arch = "wasm32", feature = "web-history"))]
impl HistoryAdapter for WebHistoryAdapter {
    fn current(&self) -> &RouteLocation {
        &self.location
    }

    fn refresh(&mut self) {
        let _ = self.refresh_from_window();
    }

    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool {
        let ok = match action {
            NavigationAction::Back | NavigationAction::Forward => {
                crate::web::navigate_with_history(action, None, None, None)
            }
            NavigationAction::Push | NavigationAction::Replace => {
                let Some(target) = target else {
                    return false;
                };

                let target = if let Some(base_path) = self.base_path.as_deref() {
                    target.with_base_path(base_path)
                } else {
                    target
                };

                let (pathname, search, hash) = route_location_to_web_parts(&target);
                crate::web::navigate_with_history(
                    action,
                    Some(pathname.as_str()),
                    Some(search.as_str()),
                    Some(hash.as_str()),
                )
            }
        };

        self.refresh();
        ok
    }
}

#[cfg(all(target_arch = "wasm32", feature = "hash-routing"))]
#[derive(Debug, Clone)]
pub struct HashHistoryAdapter {
    base_path: Option<String>,
    location: RouteLocation,
}

#[cfg(all(target_arch = "wasm32", feature = "hash-routing"))]
impl HashHistoryAdapter {
    pub fn new() -> Option<Self> {
        let location = crate::web::current_hash_route_location()?;
        Some(Self {
            base_path: None,
            location,
        })
    }

    pub fn in_base_path(base_path: impl Into<String>) -> Option<Self> {
        let base_path = base_path.into();
        let location = crate::web::current_hash_route_location()?;
        Some(Self {
            base_path: Some(base_path),
            location,
        })
    }

    fn refresh_from_window(&mut self) -> bool {
        let Some(next) = crate::web::current_hash_route_location() else {
            return false;
        };

        if next != self.location {
            self.location = next;
            true
        } else {
            false
        }
    }
}

#[cfg(all(target_arch = "wasm32", feature = "hash-routing"))]
impl HistoryAdapter for HashHistoryAdapter {
    fn current(&self) -> &RouteLocation {
        &self.location
    }

    fn refresh(&mut self) {
        let _ = self.refresh_from_window();
    }

    fn navigate(&mut self, action: NavigationAction, target: Option<RouteLocation>) -> bool {
        let ok = match action {
            NavigationAction::Back | NavigationAction::Forward => {
                crate::web::navigate_hash(action, "")
            }
            NavigationAction::Push | NavigationAction::Replace => {
                let Some(target) = target else {
                    return false;
                };

                let canonical = target.canonicalized();
                let hash_route = format!(
                    "{}{}",
                    canonical.path,
                    format_query_pairs(canonical.query.as_slice())
                );

                if let Some(base_path) = self.base_path.as_deref() {
                    crate::web::navigate_hash_in_base_path(action, hash_route.as_str(), base_path)
                } else {
                    crate::web::navigate_hash(action, hash_route.as_str())
                }
            }
        };

        self.refresh();
        ok
    }
}
