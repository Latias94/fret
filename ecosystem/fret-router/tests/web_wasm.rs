#![cfg(all(
    target_arch = "wasm32",
    any(feature = "web-history", feature = "hash-routing")
))]

use fret_router::web;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

struct UrlRestoreGuard {
    url: String,
}

impl UrlRestoreGuard {
    fn capture() -> Option<Self> {
        let snapshot = web::current_location()?;
        Some(Self {
            url: web::build_url(
                snapshot.pathname.as_str(),
                snapshot.search.as_str(),
                snapshot.hash.as_str(),
            ),
        })
    }
}

impl Drop for UrlRestoreGuard {
    fn drop(&mut self) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Ok(history) = window.history() else {
            return;
        };

        let _ = history.replace_state_with_url(
            &wasm_bindgen::JsValue::NULL,
            "",
            Some(self.url.as_str()),
        );
    }
}

#[cfg(feature = "web-history")]
#[wasm_bindgen_test]
fn web_history_navigate_replace_updates_location() {
    let _guard = UrlRestoreGuard::capture().expect("window location should be available");

    assert!(web::navigate_with_history(
        fret_router::NavigationAction::Replace,
        Some("/router-wasm-test"),
        Some("case=replace"),
        Some("history"),
    ));

    let location = web::current_location().expect("location snapshot should exist");
    assert_eq!(location.pathname, "/router-wasm-test");
    assert_eq!(location.search, "?case=replace");
    assert_eq!(location.hash, "#history");
}

#[cfg(feature = "web-history")]
#[wasm_bindgen_test]
fn web_history_subscribe_popstate_handles_event_dispatch() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let Some(window) = web_sys::window() else {
        panic!("window should exist in browser test");
    };

    let hits = Rc::new(RefCell::new(0usize));
    let hits_ref = hits.clone();
    let subscription = web::subscribe_popstate(move |_snapshot| {
        *hits_ref.borrow_mut() += 1;
    })
    .expect("subscription should succeed");

    let event = web_sys::Event::new("popstate").expect("event should construct");
    assert!(
        window
            .dispatch_event(&event)
            .expect("dispatch should succeed")
    );
    assert_eq!(*hits.borrow(), 1);

    drop(subscription);
    let event = web_sys::Event::new("popstate").expect("event should construct");
    assert!(
        window
            .dispatch_event(&event)
            .expect("dispatch should succeed")
    );
    assert_eq!(*hits.borrow(), 1);
}

#[cfg(feature = "hash-routing")]
#[wasm_bindgen_test]
fn hash_routing_subscribe_hashchange_handles_event_dispatch() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let Some(window) = web_sys::window() else {
        panic!("window should exist in browser test");
    };

    let hits = Rc::new(RefCell::new(0usize));
    let hits_ref = hits.clone();
    let subscription = web::subscribe_hashchange(move |_snapshot| {
        *hits_ref.borrow_mut() += 1;
    })
    .expect("subscription should succeed");

    let event = web_sys::Event::new("hashchange").expect("event should construct");
    assert!(
        window
            .dispatch_event(&event)
            .expect("dispatch should succeed")
    );
    assert_eq!(*hits.borrow(), 1);

    drop(subscription);
    let event = web_sys::Event::new("hashchange").expect("event should construct");
    assert!(
        window
            .dispatch_event(&event)
            .expect("dispatch should succeed")
    );
    assert_eq!(*hits.borrow(), 1);
}

#[cfg(feature = "web-history")]
#[wasm_bindgen_test]
fn current_location_in_base_path_strips_prefix() {
    let _guard = UrlRestoreGuard::capture().expect("window location should be available");

    assert!(web::navigate_with_history(
        fret_router::NavigationAction::Replace,
        Some("/app/users/42"),
        Some("tab=profile"),
        Some("section-1"),
    ));

    let location = web::current_location_in_base_path("/app")
        .expect("location should resolve under base path");
    assert_eq!(location.pathname, "/users/42");
    assert_eq!(location.search, "?tab=profile");
    assert_eq!(location.hash, "#section-1");
}
