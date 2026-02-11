use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGolden {
    pub(crate) themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGoldenTheme {
    pub(crate) root: WebNode,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebNode {
    #[allow(dead_code)]
    pub(crate) tag: String,
    #[serde(default)]
    #[serde(rename = "className")]
    pub(crate) class_name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) attrs: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) children: Vec<WebNode>,
}

pub(crate) fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .map(Path::to_path_buf)
        .expect("repo root")
}

pub(crate) fn web_golden_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.json"))
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

pub(crate) fn find_first<'a>(
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
