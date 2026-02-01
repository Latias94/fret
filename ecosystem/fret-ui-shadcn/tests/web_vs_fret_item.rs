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

#[derive(Debug, Clone, Deserialize)]
struct WebNode {
    tag: String,
    #[serde(default)]
    #[serde(rename = "className")]
    class_name: Option<String>,
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

fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.split_whitespace().any(|t| t == token))
}

const ITEM_KEYS: &[&str] = &[
    "item-avatar",
    "item-demo",
    "item-group",
    "item-header",
    "item-icon",
    "item-image",
    "item-link",
    "item-size",
    "item-variant",
];

#[test]
fn shadcn_item_goldens_are_targeted_gates() {
    for &key in ITEM_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| {
            (n.tag == "div" || n.tag == "a") && class_has_token(n, "group/item")
        })
        .expect("missing item row node (group/item)");
    }
}
