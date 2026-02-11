use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[path = "support/repo_root.rs"]
mod repo_root;

use repo_root::repo_root;

#[derive(Debug, Clone, Deserialize)]
struct WebGolden {
    version: u32,
    style: String,
    name: String,
    themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
struct WebGoldenTheme {
    url: String,
    #[serde(rename = "devicePixelRatio")]
    device_pixel_ratio: f32,
    viewport: WebViewport,
    root: WebNode,
}

#[derive(Debug, Clone, Deserialize)]
struct WebViewport {
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebRect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    path: String,
    tag: String,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    #[serde(rename = "className")]
    class_name: Option<String>,
    #[serde(default)]
    active: bool,
    #[serde(default)]
    attrs: BTreeMap<String, String>,
    rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    computed_style: BTreeMap<String, String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    children: Vec<WebNode>,
}

fn web_golden_dir() -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
}

fn read_web_golden(name: &str) -> WebGolden {
    let path = web_golden_dir().join(format!("{name}.json"));
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

#[test]
fn web_golden_button_default_smoke() {
    let golden = read_web_golden("button-default");
    assert!(golden.version >= 1);
    assert_eq!(golden.style, "new-york-v4");
    assert_eq!(golden.name, "button-default");

    let theme = golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden");

    assert!(!theme.url.is_empty());
    assert!(theme.viewport.w > 0.0);
    assert!(theme.viewport.h > 0.0);
    assert!(theme.device_pixel_ratio > 0.0);

    assert!(!theme.root.tag.is_empty());
    assert_eq!(theme.root.path, "");
    let _ = &theme.root.id;
    let _ = &theme.root.class_name;
    let _ = &theme.root.attrs;
    let _ = &theme.root.text;

    assert!(theme.root.rect.x.is_finite());
    assert!(theme.root.rect.y.is_finite());
    assert!(theme.root.rect.w >= 0.0);
    assert!(theme.root.rect.h >= 0.0);

    let button = find_first(&theme.root, &|n| n.tag == "button")
        .expect("expected at least one <button> node");
    assert!(button.rect.x.is_finite());
    assert!(button.rect.y.is_finite());
    assert!(button.rect.w > 0.0);
    assert!(button.rect.h > 0.0);
    assert!(
        button.computed_style.contains_key("display"),
        "expected computedStyle.display on <button>"
    );
}
