#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[path = "repo_root.rs"]
mod repo_root;

pub(crate) use repo_root::repo_root;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGolden {
    pub(crate) themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGoldenTheme {
    #[serde(default)]
    pub(crate) viewport: WebViewport,
    pub(crate) root: WebNode,
    #[serde(default)]
    pub(crate) portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    pub(crate) portal_wrappers: Vec<WebNode>,
    #[serde(default)]
    pub(crate) open: Option<WebOpenMeta>,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub(crate) struct WebViewport {
    pub(crate) w: f32,
    pub(crate) h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
pub(crate) struct WebRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub(crate) struct WebPoint {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub(crate) struct WebOpenMeta {
    pub(crate) action: String,
    pub(crate) selector: String,
    pub(crate) point: WebPoint,
}

#[derive(Debug, Clone, Copy, Deserialize, Default)]
pub(crate) struct WebScrollMetrics {
    #[serde(rename = "scrollWidth")]
    pub(crate) scroll_width: f32,
    #[serde(rename = "scrollHeight")]
    pub(crate) scroll_height: f32,
    #[serde(rename = "clientWidth")]
    pub(crate) client_width: f32,
    #[serde(rename = "clientHeight")]
    pub(crate) client_height: f32,
    #[serde(rename = "offsetWidth", default)]
    pub(crate) offset_width: f32,
    #[serde(rename = "offsetHeight", default)]
    pub(crate) offset_height: f32,
    #[serde(rename = "scrollLeft")]
    pub(crate) scroll_left: f32,
    #[serde(rename = "scrollTop")]
    pub(crate) scroll_top: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebNode {
    pub(crate) tag: String,
    #[serde(default)]
    pub(crate) id: Option<String>,
    #[serde(default)]
    #[serde(rename = "className")]
    pub(crate) class_name: Option<String>,
    #[serde(default)]
    pub(crate) attrs: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) active: bool,
    #[serde(rename = "activeDescendant", default)]
    pub(crate) active_descendant: bool,
    #[serde(default)]
    pub(crate) text: Option<String>,
    #[serde(default)]
    pub(crate) rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    pub(crate) computed_style: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) scroll: Option<WebScrollMetrics>,
    #[serde(default)]
    pub(crate) children: Vec<WebNode>,
}

pub(crate) fn web_golden_path_file(file_name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(file_name)
}

pub(crate) fn web_golden_path(name: &str) -> PathBuf {
    web_golden_path_file(&format!("{name}.json"))
}

pub(crate) fn web_golden_open_path(name: &str) -> PathBuf {
    web_golden_path_file(&format!("{name}.open.json"))
}

pub(crate) fn web_golden_path_open_fallback(name: &str) -> PathBuf {
    let base = web_golden_path(name);
    if base.exists() {
        return base;
    }

    // Open-mode goldens use an `.open` suffix, but some coverage keys normalize it away.
    base.with_file_name(format!("{name}.open.json"))
}

pub(crate) fn read_web_golden(name: &str) -> WebGolden {
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

pub(crate) fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_open_path(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via (in-process server):\n  node goldens/shadcn-web/scripts/extract-golden.mts --startServer --baseUrl=http://localhost:4020 {name} --modes=open --update\n\nOr (external server):\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
            path.display()
        )
    });
    serde_json::from_str(&text).unwrap_or_else(|err| {
        panic!(
            "failed to parse web open golden: {}\nerror: {err}",
            path.display()
        )
    })
}

pub(crate) fn read_web_golden_open_fallback(name: &str) -> WebGolden {
    let path = web_golden_path_open_fallback(name);
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

pub(crate) fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
}

pub(crate) fn web_theme_named<'a>(golden: &'a WebGolden, name: &str) -> &'a WebGoldenTheme {
    golden
        .themes
        .get(name)
        .unwrap_or_else(|| panic!("missing {name} theme in web golden"))
}

pub(crate) fn find_first<'a>(
    node: &'a WebNode,
    pred: &(impl Fn(&'a WebNode) -> bool + ?Sized),
) -> Option<&'a WebNode> {
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

pub(crate) fn find_all<'a>(
    node: &'a WebNode,
    pred: &(impl Fn(&'a WebNode) -> bool + ?Sized),
) -> Vec<&'a WebNode> {
    let mut out = Vec::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if pred(n) {
            out.push(n);
        }
        for child in &n.children {
            stack.push(child);
        }
    }
    out
}

pub(crate) fn find_first_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &(impl Fn(&'a WebNode) -> bool + ?Sized),
) -> Option<&'a WebNode> {
    find_first(&theme.root, pred)
        .or_else(|| theme.portals.iter().find_map(|p| find_first(p, pred)))
        .or_else(|| {
            theme
                .portal_wrappers
                .iter()
                .find_map(|p| find_first(p, pred))
        })
}

pub(crate) fn find_all_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &(impl Fn(&'a WebNode) -> bool + ?Sized),
) -> Vec<&'a WebNode> {
    let mut out = find_all(&theme.root, pred);
    for portal in &theme.portals {
        out.extend(find_all(portal, pred));
    }
    for wrapper in &theme.portal_wrappers {
        out.extend(find_all(wrapper, pred));
    }
    out
}

pub(crate) fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|t| t == token))
}

pub(crate) fn class_contains(node: &WebNode, needle: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.contains(needle))
}

pub(crate) fn class_has_all_tokens(node: &WebNode, tokens: &[&str]) -> bool {
    tokens.iter().all(|t| class_has_token(node, t))
}
