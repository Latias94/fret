use super::*;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGolden {
    pub(crate) themes: BTreeMap<String, WebGoldenTheme>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct WebGoldenTheme {
    pub(crate) viewport: WebViewport,
    pub(crate) root: WebNode,
    #[serde(default)]
    pub(crate) portals: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct WebViewport {
    pub(crate) w: f32,
    pub(crate) h: f32,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct WebRect {
    #[allow(dead_code)]
    pub(crate) x: f32,
    #[allow(dead_code)]
    pub(crate) y: f32,
    pub(crate) w: f32,
    pub(crate) h: f32,
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
    pub(crate) active: bool,
    #[serde(default)]
    #[serde(rename = "computedStyle")]
    pub(crate) computed_style: BTreeMap<String, String>,
    #[allow(dead_code)]
    #[serde(default)]
    pub(crate) attrs: BTreeMap<String, String>,
    pub(crate) rect: WebRect,
    #[serde(default)]
    pub(crate) scroll: Option<WebScrollMetrics>,
    #[serde(default)]
    pub(crate) text: Option<String>,
    #[serde(default)]
    pub(crate) children: Vec<WebNode>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub(crate) struct WebScrollMetrics {
    #[serde(rename = "scrollWidth")]
    pub(crate) scroll_width: f32,
    #[serde(rename = "scrollHeight")]
    pub(crate) scroll_height: f32,
    #[serde(rename = "clientWidth")]
    pub(crate) client_width: f32,
    #[serde(rename = "clientHeight")]
    pub(crate) client_height: f32,
    #[serde(rename = "offsetWidth")]
    #[allow(dead_code)]
    pub(crate) offset_width: f32,
    #[serde(rename = "offsetHeight")]
    #[allow(dead_code)]
    pub(crate) offset_height: f32,
    #[serde(rename = "scrollLeft")]
    pub(crate) scroll_left: f32,
    #[serde(rename = "scrollTop")]
    pub(crate) scroll_top: f32,
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

pub(crate) fn web_theme<'a>(golden: &'a WebGolden) -> &'a WebGoldenTheme {
    golden
        .themes
        .get("light")
        .or_else(|| golden.themes.get("dark"))
        .expect("missing theme in web golden")
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

pub(crate) fn find_all<'a>(
    node: &'a WebNode,
    pred: &impl Fn(&'a WebNode) -> bool,
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
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Option<&'a WebNode> {
    find_first(&theme.root, pred).or_else(|| {
        theme
            .portals
            .iter()
            .find_map(|portal| find_first(portal, pred))
    })
}

pub(crate) fn find_all_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    pred: &impl Fn(&'a WebNode) -> bool,
) -> Vec<&'a WebNode> {
    let mut out = find_all(&theme.root, pred);
    for portal in &theme.portals {
        out.extend(find_all(portal, pred));
    }
    out
}

pub(crate) fn contains_text(node: &WebNode, needle: &str) -> bool {
    if node.text.as_deref().is_some_and(|t| t.contains(needle)) {
        return true;
    }
    node.children.iter().any(|c| contains_text(c, needle))
}

pub(crate) fn contains_id(node: &WebNode, needle: &str) -> bool {
    if node
        .id
        .as_deref()
        .or_else(|| node.attrs.get("id").map(String::as_str))
        .is_some_and(|id| id == needle)
    {
        return true;
    }
    node.children.iter().any(|c| contains_id(c, needle))
}

pub(crate) fn web_find_by_tag_and_text<'a>(
    root: &'a WebNode,
    tag: &str,
    text: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| n.tag == tag && contains_text(n, text))
}

pub(crate) fn web_find_badge_spans_with_spinner<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let tokens = &[
        "inline-flex",
        "items-center",
        "justify-center",
        "rounded-full",
        "px-2",
        "py-0.5",
        "text-xs",
        "gap-1",
        "overflow-hidden",
    ];

    let mut spans = find_all(root, &|n| {
        n.tag == "span" && class_has_all_tokens(n, tokens)
    });
    spans.retain(|span| {
        find_first(span, &|n| {
            n.tag == "svg" && class_has_token(n, "animate-spin")
        })
        .is_some()
    });
    spans.sort_by(|a, b| {
        a.rect
            .x
            .partial_cmp(&b.rect.x)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                a.rect
                    .y
                    .partial_cmp(&b.rect.y)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    spans
}

pub(crate) fn web_find_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v == slot)
    })
}

pub(crate) fn web_find_all_by_data_slot<'a>(root: &'a WebNode, slot: &str) -> Vec<&'a WebNode> {
    find_all(root, &|n| {
        n.attrs.get("data-slot").is_some_and(|v| v == slot)
    })
}

pub(crate) fn web_find_scroll_area_scrollbar<'a>(
    root: &'a WebNode,
    orientation: &str,
) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "scroll-area-scrollbar")
            && n.attrs
                .get("data-orientation")
                .is_some_and(|v| v == orientation)
    })
}

pub(crate) fn web_find_scroll_area_thumb<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.attrs
            .get("data-slot")
            .is_some_and(|v| v == "scroll-area-thumb")
    })
}

pub(crate) fn web_find_scroll_area_thumb_in_scrollbar<'a>(
    scrollbar: &'a WebNode,
) -> Option<&'a WebNode> {
    web_find_scroll_area_thumb(scrollbar)
}

pub(crate) fn web_find_by_class_token<'a>(root: &'a WebNode, token: &str) -> Option<&'a WebNode> {
    find_first(root, &|n| class_has_token(n, token))
}

pub(crate) fn web_find_by_class_token_in_theme<'a>(
    theme: &'a WebGoldenTheme,
    token: &str,
) -> Option<&'a WebNode> {
    find_first_in_theme(theme, &|n| class_has_token(n, token))
}

pub(crate) fn class_has_token(node: &WebNode, token: &str) -> bool {
    node.class_name
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .any(|t| t == token)
}

pub(crate) fn class_has_all_tokens(node: &WebNode, tokens: &[&str]) -> bool {
    tokens.iter().all(|t| class_has_token(node, t))
}

pub(crate) fn web_find_by_class_tokens<'a>(
    root: &'a WebNode,
    tokens: &[&str],
) -> Option<&'a WebNode> {
    find_first(root, &|n| class_has_all_tokens(n, tokens))
}

pub(crate) fn web_css_px(node: &WebNode, key: &str) -> Px {
    let raw = node
        .computed_style
        .get(key)
        .unwrap_or_else(|| panic!("missing computedStyle[{key:?}] for <{}>", node.tag));
    let s = raw.strip_suffix("px").unwrap_or(raw);
    Px(s.parse::<f32>().unwrap_or_else(|_| {
        panic!(
            "invalid computedStyle[{key:?}] value {raw:?} for <{}>",
            node.tag
        )
    }))
}

pub(crate) fn web_css_u16(node: &WebNode, key: &str) -> u16 {
    let raw = node
        .computed_style
        .get(key)
        .unwrap_or_else(|| panic!("missing computedStyle[{key:?}] for <{}>", node.tag));
    raw.parse::<u16>().unwrap_or_else(|_| {
        panic!(
            "invalid computedStyle[{key:?}] value {raw:?} for <{}>",
            node.tag
        )
    })
}

pub(crate) fn web_collect_all<'a>(node: &'a WebNode, out: &mut Vec<&'a WebNode>) {
    out.push(node);
    for child in &node.children {
        web_collect_all(child, out);
    }
}

pub(crate) fn web_collect_tag<'a>(node: &'a WebNode, tag: &str, out: &mut Vec<&'a WebNode>) {
    if node.tag == tag {
        out.push(node);
    }
    for child in &node.children {
        web_collect_tag(child, tag, out);
    }
}

pub(crate) fn web_collect_item_rows<'a>(root: &'a WebNode) -> Vec<&'a WebNode> {
    let mut items = find_all(root, &|n| {
        (n.tag == "div" || n.tag == "a") && class_has_token(n, "group/item")
    });
    items.sort_by(|a, b| {
        a.rect
            .y
            .total_cmp(&b.rect.y)
            .then_with(|| a.rect.x.total_cmp(&b.rect.x))
    });
    items
}

pub(crate) fn web_find_item_group<'a>(root: &'a WebNode) -> Option<&'a WebNode> {
    find_first(root, &|n| {
        n.tag == "div" && class_has_token(n, "group/item-group")
    })
}

pub(crate) fn web_find_best_by<'a>(
    root: &'a WebNode,
    pred: &impl Fn(&'a WebNode) -> bool,
    score: &impl Fn(&'a WebNode) -> f32,
) -> Option<&'a WebNode> {
    let mut all = Vec::new();
    web_collect_all(root, &mut all);

    let mut best: Option<&WebNode> = None;
    let mut best_score = f32::INFINITY;
    let mut best_area = f32::INFINITY;
    for node in all.into_iter().filter(|n| pred(n)) {
        let s = score(node);
        if !s.is_finite() {
            continue;
        }
        let area = node.rect.w * node.rect.h;
        if s < best_score || (s == best_score && area < best_area) {
            best = Some(node);
            best_score = s;
            best_area = area;
        }
    }
    best
}

pub(crate) fn rect_contains(outer: WebRect, inner: WebRect) -> bool {
    let eps = 0.01;
    inner.x + eps >= outer.x
        && inner.y + eps >= outer.y
        && inner.x + inner.w <= outer.x + outer.w + eps
        && inner.y + inner.h <= outer.y + outer.h + eps
}

pub(crate) fn fret_rect_contains(outer: Rect, inner: Rect) -> bool {
    let eps = 0.01;
    inner.origin.x.0 + eps >= outer.origin.x.0
        && inner.origin.y.0 + eps >= outer.origin.y.0
        && inner.origin.x.0 + inner.size.width.0 <= outer.origin.x.0 + outer.size.width.0 + eps
        && inner.origin.y.0 + inner.size.height.0 <= outer.origin.y.0 + outer.size.height.0 + eps
}
