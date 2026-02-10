use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InsetQuad {
    pub(crate) left: f32,
    pub(crate) top_to_first_option: f32,
    pub(crate) right: f32,
    pub(crate) bottom_from_last_option: f32,
}

pub(crate) fn web_listbox_option_inset(theme: &WebGoldenTheme, listbox: &WebNode) -> InsetQuad {
    let mut all = Vec::new();
    web_collect_all(&theme.root, &mut all);

    let options: Vec<_> = all
        .into_iter()
        .filter(|n| n.attrs.get("role").is_some_and(|v| v == "option"))
        .filter(|n| rect_contains(listbox.rect, n.rect))
        .collect();

    if options.is_empty() {
        panic!("missing web listbox options");
    }

    let mut min_x = options[0].rect.x;
    let mut min_y = options[0].rect.y;
    let mut max_right = options[0].rect.x + options[0].rect.w;
    let mut max_bottom = options[0].rect.y + options[0].rect.h;
    for option in options.iter().skip(1) {
        min_x = min_x.min(option.rect.x);
        min_y = min_y.min(option.rect.y);
        max_right = max_right.max(option.rect.x + option.rect.w);
        max_bottom = max_bottom.max(option.rect.y + option.rect.h);
    }

    let panel_right = listbox.rect.x + listbox.rect.w;
    let panel_bottom = listbox.rect.y + listbox.rect.h;

    InsetQuad {
        left: min_x - listbox.rect.x,
        top_to_first_option: min_y - listbox.rect.y,
        right: panel_right - max_right,
        bottom_from_last_option: panel_bottom - max_bottom,
    }
}

pub(crate) fn fret_listbox_option_inset(snap: &fret_core::SemanticsSnapshot) -> InsetQuad {
    let listbox = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::ListBox)
        .unwrap_or_else(|| panic!("missing fret listbox"));

    let options: Vec<_> = snap
        .nodes
        .iter()
        .filter(|n| n.role == SemanticsRole::ListBoxOption)
        .filter(|n| fret_rect_contains(listbox.bounds, n.bounds))
        .collect();

    if options.is_empty() {
        panic!("missing fret listbox options");
    }

    let mut min_x = options[0].bounds.origin.x.0;
    let mut min_y = options[0].bounds.origin.y.0;
    let mut max_right = options[0].bounds.origin.x.0 + options[0].bounds.size.width.0;
    let mut max_bottom = options[0].bounds.origin.y.0 + options[0].bounds.size.height.0;
    for option in options.iter().skip(1) {
        min_x = min_x.min(option.bounds.origin.x.0);
        min_y = min_y.min(option.bounds.origin.y.0);
        max_right = max_right.max(option.bounds.origin.x.0 + option.bounds.size.width.0);
        max_bottom = max_bottom.max(option.bounds.origin.y.0 + option.bounds.size.height.0);
    }

    let panel_right = listbox.bounds.origin.x.0 + listbox.bounds.size.width.0;
    let panel_bottom = listbox.bounds.origin.y.0 + listbox.bounds.size.height.0;

    InsetQuad {
        left: min_x - listbox.bounds.origin.x.0,
        top_to_first_option: min_y - listbox.bounds.origin.y.0,
        right: panel_right - max_right,
        bottom_from_last_option: panel_bottom - max_bottom,
    }
}

pub(crate) fn assert_inset_quad_close(
    label: &str,
    actual: InsetQuad,
    expected: InsetQuad,
    tol: f32,
) {
    assert_close_px(
        &format!("{label} listbox left_inset"),
        Px(actual.left),
        expected.left,
        tol,
    );
    assert_close_px(
        &format!("{label} listbox top_to_first_option"),
        Px(actual.top_to_first_option),
        expected.top_to_first_option,
        tol,
    );
    assert_close_px(
        &format!("{label} listbox right_inset"),
        Px(actual.right),
        expected.right,
        tol,
    );
    assert_close_px(
        &format!("{label} listbox bottom_from_last_option"),
        Px(actual.bottom_from_last_option),
        expected.bottom_from_last_option,
        tol,
    );
}

pub(crate) fn web_find_smallest_container<'a>(
    root: &'a WebNode,
    nodes: &[&WebNode],
) -> Option<&'a WebNode> {
    if nodes.is_empty() {
        return None;
    }

    let mut all = Vec::new();
    web_collect_all(root, &mut all);

    let mut best: Option<&WebNode> = None;
    let mut best_area = f32::INFINITY;
    for candidate in all {
        if nodes.iter().all(|n| rect_contains(candidate.rect, n.rect)) {
            let area = candidate.rect.w * candidate.rect.h;
            if area < best_area {
                best_area = area;
                best = Some(candidate);
            }
        }
    }
    best
}
