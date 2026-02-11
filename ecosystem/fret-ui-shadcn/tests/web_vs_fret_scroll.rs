#[path = "support/web_golden_shadcn.rs"]
mod web_golden_shadcn;
use web_golden_shadcn::*;

fn is_scroll_container(metrics: WebScrollMetrics) -> bool {
    (metrics.scroll_height - metrics.client_height) > 1.0
        || (metrics.scroll_width - metrics.client_width) > 1.0
}

const SCROLL_KEYS: &[&str] = &[
    "scroll-area-demo",
    "scroll-area-demo.hover",
    "scroll-area-demo.hover-out-550ms",
    "scroll-area-demo.hover-out-650ms",
    "scroll-area-demo.scrolled",
    "scroll-area-horizontal-demo",
    "scroll-area-horizontal-demo.hover",
    "scroll-area-horizontal-demo.hover-out-550ms",
    "scroll-area-horizontal-demo.hover-out-650ms",
    "scroll-area-horizontal-demo.scrolled",
];

#[test]
fn shadcn_scroll_goldens_are_targeted_gates() {
    for &key in SCROLL_KEYS {
        let web = read_web_golden(key);
        let theme = web_theme(&web);

        let scroll = find_first(&theme.root, &|n| {
            n.tag == "div" && n.scroll.is_some_and(is_scroll_container)
        })
        .or_else(|| find_first(&theme.root, &|n| n.scroll.is_some_and(is_scroll_container)))
        .expect("missing scroll container (scroll metrics)");

        assert!(scroll.scroll.is_some(), "expected scroll metrics");
    }
}

#[test]
fn shadcn_scroll_area_vertical_scrolled_snapshot_has_scroll_top() {
    let web = read_web_golden("scroll-area-demo.scrolled");
    let theme = web.themes.get("light").expect("missing light theme");

    let scroll = find_first(&theme.root, &|n| {
        n.scroll
            .is_some_and(|m| is_scroll_container(m) && m.scroll_top > 0.0)
    })
    .expect("missing scrolled container with scrollTop>0");

    assert!(
        scroll.scroll.unwrap().scroll_top > 0.0,
        "expected scrollTop>0"
    );
}

#[test]
fn shadcn_scroll_area_horizontal_scrolled_snapshot_has_scroll_left() {
    let web = read_web_golden("scroll-area-horizontal-demo.scrolled");
    let theme = web.themes.get("light").expect("missing light theme");

    let scroll = find_first(&theme.root, &|n| {
        n.scroll
            .is_some_and(|m| is_scroll_container(m) && m.scroll_left > 0.0)
    })
    .expect("missing scrolled container with scrollLeft>0");

    assert!(
        scroll.scroll.unwrap().scroll_left > 0.0,
        "expected scrollLeft>0"
    );
}
