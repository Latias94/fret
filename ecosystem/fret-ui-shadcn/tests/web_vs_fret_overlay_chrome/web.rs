use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebGolden {
    pub(super) themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebGoldenTheme {
    #[allow(dead_code)]
    pub(super) root: WebNode,
    #[serde(default)]
    pub(super) portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    pub(super) portal_wrappers: Vec<WebNode>,
    #[serde(default)]
    pub(super) viewport: Option<WebViewport>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(super) struct WebViewport {
    pub(super) w: f32,
    pub(super) h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(super) struct WebRect {
    #[allow(dead_code)]
    pub(super) x: f32,
    #[allow(dead_code)]
    pub(super) y: f32,
    pub(super) w: f32,
    pub(super) h: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct WebNode {
    #[allow(dead_code)]
    pub(super) tag: String,
    #[serde(default)]
    pub(super) attrs: BTreeMap<String, String>,
    #[serde(default)]
    pub(super) active: bool,
    #[serde(rename = "activeDescendant", default)]
    pub(super) active_descendant: bool,
    #[serde(default)]
    pub(super) text: Option<String>,
    pub(super) rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    pub(super) computed_style: BTreeMap<String, String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub(super) children: Vec<WebNode>,
}

pub(super) fn web_theme_named<'a>(golden: &'a WebGolden, name: &str) -> &'a WebGoldenTheme {
    golden
        .themes
        .get(name)
        .unwrap_or_else(|| panic!("missing {name} theme in web golden"))
}

pub(super) fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

pub(super) fn web_golden_path(file_name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(file_name)
}

pub(super) fn read_web_golden_open(name: &str) -> WebGolden {
    let path = web_golden_path(&format!("{name}.open.json"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "missing web open golden: {}\nerror: {err}\n\nGenerate it via:\n  pnpm -C repo-ref/ui/apps/v4 exec tsx --tsconfig ./tsconfig.scripts.json ../../../../goldens/shadcn-web/scripts/extract-golden.mts {name} --modes=open --update --baseUrl=http://localhost:4020\n\nDocs:\n  docs/shadcn-web-goldens.md",
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

pub(super) fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
}

pub(super) fn find_portal_by_role<'a>(
    theme: &'a WebGoldenTheme,
    role: &str,
) -> Option<&'a WebNode> {
    theme
        .portals
        .iter()
        .find(|n| n.attrs.get("role").is_some_and(|v| v == role))
}

pub(super) fn find_portal_by_slot<'a>(
    theme: &'a WebGoldenTheme,
    slot: &str,
) -> Option<&'a WebNode> {
    theme
        .portals
        .iter()
        .find(|n| n.attrs.get("data-slot").is_some_and(|v| v == slot))
}

pub(super) fn find_first<'a>(
    node: &'a WebNode,
    pred: &impl Fn(&'a WebNode) -> bool,
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

pub(super) fn find_by_data_slot_and_state<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
    })
}

pub(super) fn find_by_data_slot_and_state_and_text<'a>(
    root: &'a WebNode,
    slot: &str,
    state: &str,
    text: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
            && n.attrs
                .get("data-state")
                .is_some_and(|v| v.as_str() == state)
            && n.text.as_deref() == Some(text)
    })
}

pub(super) fn parse_px(s: &str) -> Option<f32> {
    let s = s.trim();
    let v = s.strip_suffix("px").unwrap_or(s);
    v.parse::<f32>().ok()
}

pub(super) fn web_border_widths_px(node: &WebNode) -> Option<[f32; 4]> {
    Some([
        node.computed_style
            .get("borderTopWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderRightWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderBottomWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
        node.computed_style
            .get("borderLeftWidth")
            .map(String::as_str)
            .and_then(parse_px)?,
    ])
}

pub(super) fn web_corner_radii_effective_px(node: &WebNode) -> Option<[f32; 4]> {
    let max = node.rect.w.min(node.rect.h) * 0.5;
    let radius = |key: &str| {
        node.computed_style
            .get(key)
            .map(String::as_str)
            .and_then(parse_px)
            .map(|v| v.min(max))
    };

    Some([
        radius("borderTopLeftRadius")?,
        radius("borderTopRightRadius")?,
        radius("borderBottomRightRadius")?,
        radius("borderBottomLeftRadius")?,
    ])
}
