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

fn class_has_all_tokens(node: &WebNode, tokens: &[&str]) -> bool {
    tokens.iter().all(|t| class_has_token(node, t))
}

const CAROUSEL_KEYS: &[&str] = &[
    "carousel-api",
    "carousel-demo",
    "carousel-orientation",
    "carousel-plugin",
    "carousel-size",
    "carousel-spacing",
];

#[test]
fn shadcn_carousel_goldens_are_targeted_gates() {
    for &key in CAROUSEL_KEYS {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| {
            n.tag == "div" && class_has_token(n, "overflow-hidden")
        })
        .expect("missing carousel viewport (overflow-hidden)");

        find_first(&theme.root, &|n| {
            n.tag == "div" && class_has_token(n, "min-w-0") && class_has_token(n, "shrink-0")
        })
        .expect("missing carousel item basis (min-w-0 shrink-0)");

        let has_horizontal_controls = find_first(&theme.root, &|n| {
            n.tag == "button"
                && n.class_name
                    .as_deref()
                    .is_some_and(|c| c.contains("-left-12") || c.contains("-right-12"))
        })
        .is_some();

        let has_vertical_controls = find_first(&theme.root, &|n| {
            n.tag == "button"
                && n.class_name
                    .as_deref()
                    .is_some_and(|c| c.contains("-top-12") || c.contains("-bottom-12"))
        })
        .is_some();

        assert!(
            has_horizontal_controls || has_vertical_controls,
            "missing carousel nav controls (expected -left/-right or -top/-bottom buttons) in {key}"
        );
    }
}
