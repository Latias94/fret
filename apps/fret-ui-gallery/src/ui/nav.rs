use super::*;
use fret_ui::scroll::ScrollHandle;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct NavVisibilitySummary {
    pub visible_groups_count: u64,
    pub visible_items_count: u64,
    pub visible_ai_items_count: u64,
    pub visible_tags_count: u64,
    pub max_group_items_count: u64,
    pub visible_string_bytes_estimate_total: u64,
}

struct VisibleNavGroup {
    title: &'static str,
    items: Vec<&'static PageSpec>,
}

pub(crate) fn matches_query(query: &str, item: &PageSpec) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return true;
    }

    let q_lower = q.to_ascii_lowercase();
    let q_norm: String = q_lower
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect();

    let matches_norm = |haystack: &str| {
        if q_norm.is_empty() {
            return false;
        }
        let norm: String = haystack
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| c.to_ascii_lowercase())
            .collect();
        norm.contains(&q_norm)
    };

    if item.id.to_ascii_lowercase().contains(&q_lower) || matches_norm(item.id) {
        return true;
    }
    if item.label.to_ascii_lowercase().contains(&q_lower) || matches_norm(item.label) {
        return true;
    }
    if item.origin.to_ascii_lowercase().contains(&q_lower) || matches_norm(item.origin) {
        return true;
    }
    item.tags
        .iter()
        .any(|t| t.to_ascii_lowercase().contains(&q_lower) || matches_norm(t))
}

fn collect_visible_nav_groups(query: &str) -> Vec<VisibleNavGroup> {
    let mut groups: Vec<VisibleNavGroup> = Vec::new();
    let mut deferred_ai_items: Vec<&'static PageSpec> = Vec::new();
    let mut inserted_ai_group = false;

    for group in PAGE_GROUPS {
        let mut group_items: Vec<&'static PageSpec> = Vec::new();
        for item in group.items {
            if !matches_query(query, item) {
                continue;
            }
            if item.id.starts_with("ai_") {
                deferred_ai_items.push(item);
            } else {
                group_items.push(item);
            }
        }

        if !group_items.is_empty() {
            groups.push(VisibleNavGroup {
                title: group.title,
                items: group_items,
            });
        }

        if group.title == "Shadcn" && !inserted_ai_group {
            if !deferred_ai_items.is_empty() {
                groups.push(VisibleNavGroup {
                    title: "AI Elements",
                    items: std::mem::take(&mut deferred_ai_items),
                });
            }
            inserted_ai_group = true;
        }
    }

    if !inserted_ai_group && !deferred_ai_items.is_empty() {
        groups.push(VisibleNavGroup {
            title: "AI Elements",
            items: deferred_ai_items,
        });
    }

    groups
}

pub(crate) fn nav_visibility_summary(query: &str) -> NavVisibilitySummary {
    let groups = collect_visible_nav_groups(query);
    let mut summary = NavVisibilitySummary {
        visible_groups_count: groups.len() as u64,
        ..Default::default()
    };

    for group in groups {
        summary.visible_string_bytes_estimate_total = summary
            .visible_string_bytes_estimate_total
            .saturating_add(group.title.len() as u64);
        summary.max_group_items_count = summary.max_group_items_count.max(group.items.len() as u64);

        for item in group.items {
            summary.visible_items_count = summary.visible_items_count.saturating_add(1);
            summary.visible_tags_count = summary
                .visible_tags_count
                .saturating_add(item.tags.len() as u64);
            if item.id.starts_with("ai_") {
                summary.visible_ai_items_count = summary.visible_ai_items_count.saturating_add(1);
            }
            summary.visible_string_bytes_estimate_total = summary
                .visible_string_bytes_estimate_total
                .saturating_add(item.id.len() as u64)
                .saturating_add(item.label.len() as u64)
                .saturating_add(item.title.len() as u64)
                .saturating_add(item.origin.len() as u64)
                .saturating_add(item.command.len() as u64);
            for tag in item.tags {
                summary.visible_string_bytes_estimate_total = summary
                    .visible_string_bytes_estimate_total
                    .saturating_add(tag.len() as u64);
            }
        }
    }

    summary
}

pub(crate) fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let nav_scroll_handle = cx.with_state(ScrollHandle::default, |h| h.clone());
    let nav_query_changed = cx.with_state(String::new, |last_query| {
        if last_query.as_str() == query {
            false
        } else {
            *last_query = query.to_owned();
            true
        }
    });
    if nav_query_changed {
        // Keep search results discoverable: when the filter changes, reset the nav scroll position
        // so matches near the top of the list are visible immediately.
        nav_scroll_handle.scroll_to_offset(Point::new(Px(0.0), Px(0.0)));
    }

    let title_row = ui::h_row(|cx| {
        [
            cx.text("Fret UI Gallery"),
            shadcn::Badge::new("WIP")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().w_full())
    .justify_between()
    .items_center()
    .into_element(cx);

    let query_input = {
        let nav_query = nav_query.clone();
        shadcn::Input::new(nav_query.clone())
            .a11y_label("Search components")
            .placeholder("Search (id / tag)")
            .test_id("ui-gallery-nav-search")
            .into_element(cx)
    };

    let push_group = |cx: &mut ElementContext<'_, App>,
                      title: &'static str,
                      items: &[&'static PageSpec],
                      nav_sections: &mut Vec<AnyElement>| {
        let group_sections = cx.keyed(title, |cx| {
            let mut group_items: Vec<AnyElement> = Vec::new();
            for item in items {
                if !matches_query(query, item) {
                    continue;
                }

                let is_selected = selected == item.id;

                group_items.push(cx.keyed(item.id, |cx| {
                    shadcn::SidebarMenuButton::new(item.label)
                        .active(is_selected)
                        .collapsed(false)
                        .on_click(item.command)
                        .test_id(format!("ui-gallery-nav-{}", item.id.replace('_', "-")))
                        .into_element(cx)
                }));
            }

            if group_items.is_empty() {
                return Vec::new();
            }

            vec![
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::from(title),
                    style: None,
                    color: Some(theme.color_token("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: fret_ui::element::TextInkOverflow::None,
                }),
                ui::v_flex(move |_cx| group_items)
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N1)
                    .into_element(cx),
            ]
        });

        nav_sections.extend(group_sections);
    };

    let mut nav_sections: Vec<AnyElement> = Vec::new();
    for group in collect_visible_nav_groups(query) {
        push_group(cx, group.title, &group.items, &mut nav_sections);
    }

    let nav_body = ui::v_flex(move |_cx| nav_sections)
        .layout(LayoutRefinement::default().w_full())
        .gap(Space::N4)
        .into_element(cx);
    let nav_scroll = {
        let nav_scroll = if (bisect & BISECT_DISABLE_SIDEBAR_SCROLL) != 0 {
            nav_body
        } else {
            shadcn::ScrollArea::new([nav_body])
                .refine_layout(LayoutRefinement::default().w_full().h_full())
                .scroll_handle(nav_scroll_handle.clone())
                .into_element(cx)
        };
        nav_scroll.test_id("ui-gallery-nav-scroll")
    };

    let container = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(
                    theme
                        .color_by_key("sidebar")
                        .unwrap_or_else(|| theme.color_token("background")),
                ))
                .p(Space::N4),
            LayoutRefinement::default().w_px(Px(280.0)).h_full(),
        ),
        |cx| {
            [ui::v_flex(|_cx| [title_row, query_input, nav_scroll])
                .layout(LayoutRefinement::default().w_full().h_full())
                .gap(Space::N4)
                .into_element(cx)]
        },
    );

    container
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: PageSpec = PageSpec::new(
        "hit_test_torture",
        "Hit Test (Torture)",
        "Hit Test / Spatial Index Harness",
        "fret-ui (hit testing)",
        "ui_gallery.nav.select.hit_test_torture",
        &[
            "hit_test",
            "pointer",
            "dispatch",
            "performance",
            "gpui-parity",
        ],
    );

    #[test]
    fn nav_search_matches_empty_query() {
        assert!(matches_query("", &ITEM));
        assert!(matches_query("   ", &ITEM));
    }

    #[test]
    fn nav_search_matches_case_insensitive_substrings() {
        assert!(matches_query("HIT", &ITEM));
        assert!(matches_query("torture", &ITEM));
        assert!(matches_query("FRET-UI", &ITEM));
        assert!(matches_query("gpui", &ITEM));
    }

    #[test]
    fn nav_search_matches_normalized_tokens_across_separators() {
        assert!(matches_query("hit test", &ITEM));
        assert!(matches_query("hit-test", &ITEM));
        assert!(matches_query("hit_test", &ITEM));
        assert!(matches_query("gpuiparity", &ITEM));
        assert!(matches_query("gpui parity", &ITEM));
    }

    #[test]
    fn nav_search_rejects_non_matching_terms() {
        assert!(!matches_query("accordion", &ITEM));
        assert!(!matches_query("chart", &ITEM));
    }

    #[test]
    fn nav_visibility_summary_counts_items_for_empty_query() {
        let summary = nav_visibility_summary("");
        let expected_items = PAGE_GROUPS
            .iter()
            .flat_map(|group| group.items.iter())
            .count() as u64;
        let expected_ai_items = PAGE_GROUPS
            .iter()
            .flat_map(|group| group.items.iter())
            .filter(|item| item.id.starts_with("ai_"))
            .count() as u64;

        assert_eq!(summary.visible_items_count, expected_items);
        assert_eq!(summary.visible_ai_items_count, expected_ai_items);
        assert!(summary.visible_groups_count > 0);
        assert!(summary.max_group_items_count > 0);
        assert!(summary.visible_string_bytes_estimate_total > 0);
    }

    #[test]
    fn nav_visibility_summary_shrinks_for_filtered_query() {
        let full = nav_visibility_summary("");
        let filtered = nav_visibility_summary("card");

        assert!(filtered.visible_items_count > 0);
        assert!(filtered.visible_items_count < full.visible_items_count);
        assert!(filtered.visible_groups_count <= full.visible_groups_count);
        assert!(
            filtered.visible_string_bytes_estimate_total < full.visible_string_bytes_estimate_total
        );
    }
}
