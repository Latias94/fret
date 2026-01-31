use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    root: WebNode,
}

#[derive(Debug, Clone, Copy, Deserialize)]
struct WebScrollMetrics {
    #[serde(rename = "scrollWidth")]
    scroll_width: f32,
    #[serde(rename = "scrollHeight")]
    scroll_height: f32,
    #[serde(rename = "clientWidth")]
    client_width: f32,
    #[serde(rename = "clientHeight")]
    client_height: f32,
    #[serde(rename = "scrollLeft")]
    scroll_left: f32,
    #[serde(rename = "scrollTop")]
    scroll_top: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    scroll: Option<WebScrollMetrics>,
    #[serde(default)]
    children: Vec<WebNode>,
}

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

fn web_golden_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.json"))
}

fn read_web_golden(name: &str) -> WebGolden {
    let path = web_golden_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 golden:extract {name} --update\n\nDocs:\n  goldens/README.md\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web golden: {}\nerror: {err}",
            path.display()
        )
    })
}

fn find_first<'a>(node: &'a WebNode, pred: &impl Fn(&'a WebNode) -> bool) -> Option<&'a WebNode> {
    if pred(node) {
        return Some(node);
    }
    for child in &node.children {
        if let Some(found) = find_first(child, pred) {
            return Some(found);
        }
    }
    None
}

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
        let theme = web.themes.get("light").expect("missing light theme");

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
