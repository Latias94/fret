use crate::{
    NavigationAction, apply_base_path, first_query_value_from_search_or_hash,
    strip_base_path as strip_base_path_from_path,
};
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebLocationSnapshot {
    pub pathname: String,
    pub search: String,
    pub hash: String,
}

impl Default for WebLocationSnapshot {
    fn default() -> Self {
        Self {
            pathname: "/".to_string(),
            search: String::new(),
            hash: String::new(),
        }
    }
}

impl WebLocationSnapshot {
    pub fn query_value(&self, key: &str) -> Option<String> {
        first_query_value_from_search_or_hash(&self.search, &self.hash, key)
    }
}

pub struct WebLocationSubscription {
    event_name: &'static str,
    callback: Closure<dyn FnMut(web_sys::Event)>,
}

impl Drop for WebLocationSubscription {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            let _ = window.remove_event_listener_with_callback(
                self.event_name,
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
}

pub fn current_location() -> Option<WebLocationSnapshot> {
    let window = web_sys::window()?;
    let location = window.location();

    let pathname = location.pathname().ok().unwrap_or_default();
    let search = location.search().ok().unwrap_or_default();
    let hash = location.hash().ok().unwrap_or_default();

    Some(WebLocationSnapshot {
        pathname: if pathname.is_empty() {
            "/".to_string()
        } else {
            pathname
        },
        search,
        hash,
    })
}

pub fn current_location_in_base_path(base_path: &str) -> Option<WebLocationSnapshot> {
    let mut location = current_location()?;
    location.pathname = strip_base_path_from_path(location.pathname.as_str(), base_path)?;
    Some(location)
}

pub fn build_url(pathname: &str, search: &str, hash: &str) -> String {
    let path = if pathname.trim().is_empty() {
        "/".to_string()
    } else {
        pathname.to_string()
    };
    format!(
        "{}{}{}",
        path,
        normalize_search(search),
        normalize_hash(hash)
    )
}

pub fn build_url_in_base_path(pathname: &str, search: &str, hash: &str, base_path: &str) -> String {
    let path = apply_base_path(pathname, base_path);
    build_url(path.as_str(), search, hash)
}

#[cfg(feature = "web-history")]
pub fn navigate_with_history(
    action: NavigationAction,
    pathname: Option<&str>,
    search: Option<&str>,
    hash: Option<&str>,
) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let Ok(history) = window.history() else {
        return false;
    };

    match action {
        NavigationAction::Back => history.back().is_ok(),
        NavigationAction::Forward => history.forward().is_ok(),
        NavigationAction::Push | NavigationAction::Replace => {
            let current = current_location().unwrap_or_default();
            let url = build_url(
                pathname.unwrap_or(current.pathname.as_str()),
                search.unwrap_or(current.search.as_str()),
                hash.unwrap_or(current.hash.as_str()),
            );

            if action == NavigationAction::Push {
                history
                    .push_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            } else {
                history
                    .replace_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            }
        }
    }
}

#[cfg(feature = "web-history")]
pub fn navigate_with_history_in_base_path(
    action: NavigationAction,
    pathname: Option<&str>,
    search: Option<&str>,
    hash: Option<&str>,
    base_path: &str,
) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let Ok(history) = window.history() else {
        return false;
    };

    match action {
        NavigationAction::Back => history.back().is_ok(),
        NavigationAction::Forward => history.forward().is_ok(),
        NavigationAction::Push | NavigationAction::Replace => {
            let current = current_location_in_base_path(base_path).unwrap_or_default();
            let url = build_url_in_base_path(
                pathname.unwrap_or(current.pathname.as_str()),
                search.unwrap_or(current.search.as_str()),
                hash.unwrap_or(current.hash.as_str()),
                base_path,
            );

            if action == NavigationAction::Push {
                history
                    .push_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            } else {
                history
                    .replace_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            }
        }
    }
}

#[cfg(feature = "hash-routing")]
pub fn navigate_hash(action: NavigationAction, hash: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let Ok(history) = window.history() else {
        return false;
    };

    match action {
        NavigationAction::Back => history.back().is_ok(),
        NavigationAction::Forward => history.forward().is_ok(),
        NavigationAction::Push | NavigationAction::Replace => {
            let current = current_location().unwrap_or_default();
            let url = build_url(current.pathname.as_str(), current.search.as_str(), hash);

            if action == NavigationAction::Push {
                history
                    .push_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            } else {
                history
                    .replace_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            }
        }
    }
}

#[cfg(feature = "hash-routing")]
pub fn navigate_hash_in_base_path(action: NavigationAction, hash: &str, base_path: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };

    let Ok(history) = window.history() else {
        return false;
    };

    match action {
        NavigationAction::Back => history.back().is_ok(),
        NavigationAction::Forward => history.forward().is_ok(),
        NavigationAction::Push | NavigationAction::Replace => {
            let current = current_location_in_base_path(base_path).unwrap_or_default();
            let url = build_url_in_base_path(
                current.pathname.as_str(),
                current.search.as_str(),
                hash,
                base_path,
            );

            if action == NavigationAction::Push {
                history
                    .push_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            } else {
                history
                    .replace_state_with_url(&JsValue::NULL, "", Some(url.as_str()))
                    .is_ok()
            }
        }
    }
}

#[cfg(feature = "web-history")]
pub fn subscribe_popstate(
    on_change: impl FnMut(WebLocationSnapshot) + 'static,
) -> Option<WebLocationSubscription> {
    subscribe_location_event("popstate", on_change)
}

#[cfg(feature = "hash-routing")]
pub fn subscribe_hashchange(
    on_change: impl FnMut(WebLocationSnapshot) + 'static,
) -> Option<WebLocationSubscription> {
    subscribe_location_event("hashchange", on_change)
}

fn subscribe_location_event(
    event_name: &'static str,
    mut on_change: impl FnMut(WebLocationSnapshot) + 'static,
) -> Option<WebLocationSubscription> {
    let window = web_sys::window()?;
    let callback = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if let Some(location) = current_location() {
            on_change(location);
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    window
        .add_event_listener_with_callback(event_name, callback.as_ref().unchecked_ref())
        .ok()?;

    Some(WebLocationSubscription {
        event_name,
        callback,
    })
}

fn normalize_search(search: &str) -> String {
    let search = search.trim();
    if search.is_empty() {
        String::new()
    } else if search.starts_with('?') {
        search.to_string()
    } else {
        format!("?{search}")
    }
}

fn normalize_hash(hash: &str) -> String {
    let hash = hash.trim();
    if hash.is_empty() {
        String::new()
    } else if hash.starts_with('#') {
        hash.to_string()
    } else {
        format!("#{hash}")
    }
}

#[cfg(test)]
mod tests {
    use super::{build_url, build_url_in_base_path, current_location_in_base_path};

    #[test]
    fn build_url_normalizes_components() {
        assert_eq!(build_url("/", "a=1", "demo"), "/?a=1#demo");
        assert_eq!(build_url("/x", "?a=1", "#demo"), "/x?a=1#demo");
        assert_eq!(build_url("", "", ""), "/");
    }

    #[test]
    fn build_url_in_base_path_joins_path_prefix() {
        assert_eq!(
            build_url_in_base_path("/users/42", "a=1", "demo", "/app"),
            "/app/users/42?a=1#demo"
        );
    }

    #[test]
    fn current_location_in_base_path_is_none_without_window() {
        // Native test environment does not provide `window`.
        assert!(current_location_in_base_path("/app").is_none());
    }
}
