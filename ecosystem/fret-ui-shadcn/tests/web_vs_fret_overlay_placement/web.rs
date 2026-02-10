use super::*;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGolden {
    pub(crate) themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGoldenTheme {
    #[allow(dead_code)]
    pub(crate) root: WebNode,
    #[serde(default)]
    pub(crate) portals: Vec<WebNode>,
    #[serde(rename = "portalWrappers", default)]
    pub(crate) portal_wrappers: Vec<WebNode>,
    #[serde(default)]
    pub(crate) viewport: Option<WebViewport>,
    #[serde(default)]
    pub(crate) open: Option<WebOpenMeta>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct WebViewport {
    pub(crate) w: f32,
    pub(crate) h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct WebPoint {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebOpenMeta {
    #[allow(dead_code)]
    pub(crate) action: String,
    #[allow(dead_code)]
    pub(crate) selector: String,
    pub(crate) point: WebPoint,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct WebRect {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebNode {
    pub(crate) tag: String,
    #[serde(default)]
    pub(crate) attrs: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) active: bool,
    pub(crate) rect: WebRect,
    #[serde(rename = "computedStyle", default)]
    pub(crate) computed_style: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) text: Option<String>,
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

pub(crate) fn web_golden_open_path(name: &str) -> PathBuf {
    repo_root()
        .join("goldens")
        .join("shadcn-web")
        .join("v4")
        .join("new-york-v4")
        .join(format!("{name}.open.json"))
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

pub(crate) fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
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

pub(crate) fn web_find_by_data_slot_and_state<'a>(
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

pub(crate) fn web_find_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v.as_str() == slot)
    })
}

pub(crate) fn web_portal_node_by_data_slot<'a>(
    theme: &'a WebGoldenTheme,
    slot: &str,
) -> &'a WebNode {
    for portal in &theme.portals {
        if let Some(found) = web_find_by_data_slot(portal, slot) {
            return found;
        }
    }
    for wrapper in &theme.portal_wrappers {
        if let Some(found) = web_find_by_data_slot(wrapper, slot) {
            return found;
        }
    }
    panic!("missing web portal node with data-slot={slot}")
}

pub(crate) fn find_attr_in_subtree<'a>(node: &'a WebNode, key: &str) -> Option<&'a str> {
    node.attrs.get(key).map(String::as_str).or_else(|| {
        for child in &node.children {
            if let Some(found) = find_attr_in_subtree(child, key) {
                return Some(found);
            }
        }
        None
    })
}
