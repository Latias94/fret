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
    attrs: BTreeMap<String, String>,
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

fn class_contains(node: &WebNode, needle: &str) -> bool {
    node.class_name
        .as_deref()
        .is_some_and(|class| class.contains(needle))
}

const LOGIN_KEYS: &[&str] = &["login-01", "login-02", "login-03", "login-04", "login-05"];
const SIGNUP_KEYS: &[&str] = &[
    "signup-01",
    "signup-02",
    "signup-03",
    "signup-04",
    "signup-05",
];
const OTP_KEYS: &[&str] = &["otp-01", "otp-02", "otp-03", "otp-04", "otp-05"];

#[test]
fn shadcn_auth_template_goldens_are_targeted_gates() {
    for &key in LOGIN_KEYS.iter().chain(SIGNUP_KEYS).chain(OTP_KEYS) {
        let web = read_web_golden(key);
        let theme = web.themes.get("light").expect("missing light theme");

        find_first(&theme.root, &|n| class_has_token(n, "min-h-svh"))
            .expect("missing min-h-svh layout wrapper");

        if key.starts_with("otp-") {
            let has_slot_list_recipe =
                find_first(&theme.root, &|n| class_contains(n, "input-otp-slot")).is_some();
            let has_slot_node_recipe = find_first(&theme.root, &|n| {
                class_contains(n, "data-[active=true]:border-ring")
                    && class_contains(n, "first:rounded-l-md")
            })
            .is_some();
            assert!(
                has_slot_list_recipe || has_slot_node_recipe,
                "missing input-otp recipe markers"
            );
        } else {
            let has_max_w = find_first(&theme.root, &|n| {
                class_has_token(n, "max-w-sm") || class_has_token(n, "max-w-xs")
            })
            .is_some();
            assert!(has_max_w, "missing max-w-sm/max-w-xs content container");
        }
    }
}
