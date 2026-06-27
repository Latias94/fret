use super::*;
use fret::AppComponentCx;
use fret_ui::scroll::ScrollHandle;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::declarative::{CachedSubtreeProps, style as decl_style};
use fret_ui_kit::theme_tokens;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

fn nav_body_cache_key(selected: &str, query: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    selected.hash(&mut hasher);
    query.trim().to_ascii_lowercase().hash(&mut hasher);
    hasher.finish()
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

fn nav_visibility_summary_from_groups(groups: &[VisibleNavGroup]) -> NavVisibilitySummary {
    let mut summary = NavVisibilitySummary {
        visible_groups_count: groups.len() as u64,
        ..Default::default()
    };

    for group in groups {
        summary.visible_string_bytes_estimate_total = summary
            .visible_string_bytes_estimate_total
            .saturating_add(group.title.len() as u64);
        summary.max_group_items_count = summary.max_group_items_count.max(group.items.len() as u64);

        for item in group.items.iter().copied() {
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

pub(crate) fn nav_visibility_summary(query: &str) -> NavVisibilitySummary {
    nav_visibility_summary_from_groups(&collect_visible_nav_groups(query))
}

fn nav_body_content_height(
    summary: NavVisibilitySummary,
    title_height: Px,
    item_gap: Px,
    button_height: Px,
    group_gap: Px,
) -> Px {
    if summary.visible_groups_count == 0 {
        return Px(0.0);
    }

    let visible_groups = summary.visible_groups_count as f32;
    let visible_items = summary.visible_items_count as f32;
    let group_gaps = summary.visible_groups_count.saturating_sub(1) as f32;
    Px(visible_groups * title_height.0
        + visible_items * (button_height.0 + item_gap.0)
        + group_gaps * group_gap.0)
}

fn nav_body_known_content_size(theme: &Theme, summary: NavVisibilitySummary) -> fret_core::Size {
    let title_height = theme
        .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let button_height = theme
        .metric_by_key("component.sidebar.menu_button.h")
        .unwrap_or(Px(32.0));
    let item_gap = decl_style::space(theme, Space::N1);
    let group_gap = decl_style::space(theme, Space::N4);
    fret_core::Size::new(
        Px(0.0),
        nav_body_content_height(summary, title_height, item_gap, button_height, group_gap),
    )
}

pub(crate) fn sidebar_view(
    cx: &mut AppComponentCx<'_>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();
    let visible_groups = collect_visible_nav_groups(query);
    let nav_summary = nav_visibility_summary_from_groups(&visible_groups);
    let known_content_size = nav_body_known_content_size(theme, nav_summary);

    let nav_scroll_handle = cx.slot_state(ScrollHandle::default, |h| h.clone());
    let nav_query_changed = cx.slot_state(String::new, |last_query| {
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
            decl_text::text_section_chrome_label(cx, "Fret UI Gallery"),
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

    let push_group = |cx: &mut AppComponentCx<'_>,
                      title: &'static str,
                      items: &[&'static PageSpec],
                      nav_sections: &mut Vec<AnyElement>| {
        let group_sections = cx.keyed(title, |cx| {
            let mut group_items: Vec<AnyElement> = Vec::new();
            for item in items.iter().copied() {
                let is_selected = selected == item.id;

                group_items.push(cx.keyed(item.id, |cx| {
                    shadcn::SidebarMenuButton::new(item.label)
                        .active(is_selected)
                        .collapsed(false)
                        .action(item.command)
                        .test_id(format!("ui-gallery-nav-{}", item.id.replace('_', "-")))
                        .into_element(cx)
                }));
            }

            if group_items.is_empty() {
                return Vec::new();
            }

            vec![
                decl_text::text_section_chrome_label(cx, title),
                ui::v_flex(move |_cx| group_items)
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N1)
                    .into_element(cx),
            ]
        });

        nav_sections.extend(group_sections);
    };

    let nav_body = {
        let nav_body_cache_key = nav_body_cache_key(selected, query);
        cx.cached_subtree_with(
            CachedSubtreeProps::default()
                .contain_layout_when_bounds_known(true)
                .cache_key(nav_body_cache_key),
            move |cx| {
                let mut nav_sections: Vec<AnyElement> = Vec::new();
                for group in &visible_groups {
                    push_group(cx, group.title, &group.items, &mut nav_sections);
                }

                [ui::v_flex(move |_cx| nav_sections)
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4)
                    .into_element(cx)]
            },
        )
    };
    let nav_scroll = {
        let nav_scroll = if (bisect & BISECT_DISABLE_SIDEBAR_SCROLL) != 0 {
            nav_body
        } else {
            // This sidebar already lives inside a fixed-width, fixed-height shell. Keep the
            // scroll viewport from recursively measuring the full nav list during intrinsic
            // sizing.
            shadcn::ScrollArea::new([nav_body])
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_full()
                        .flex_1()
                        .min_w_0()
                        .min_h_0(),
                )
                .scroll_handle(nav_scroll_handle.clone())
                .viewport_intrinsic_measure_mode(
                    fret_ui::element::ScrollIntrinsicMeasureMode::Viewport,
                )
                .viewport_probe_unbounded(false)
                .viewport_known_content_size(known_content_size)
                .into_element(cx)
        };
        nav_scroll.test_id("ui-gallery-nav-scroll")
    };

    ui::v_flex(|_cx| [title_row, query_input, nav_scroll])
        .bg(ColorRef::Color(
            theme
                .color_by_key("sidebar")
                .unwrap_or_else(|| theme.color_token("background")),
        ))
        .p(Space::N4)
        .layout(
            LayoutRefinement::default()
                .w_px(Px(280.0))
                .h_full()
                .flex_shrink_0(),
        )
        .gap(Space::N4)
        .into_element(cx)
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
    fn nav_body_cache_key_collapses_query_case_and_whitespace() {
        assert_eq!(
            nav_body_cache_key(PAGE_BUTTON, "  Card  "),
            nav_body_cache_key(PAGE_BUTTON, "card")
        );
        assert_ne!(
            nav_body_cache_key(PAGE_BUTTON, "card"),
            nav_body_cache_key(PAGE_CARD, "card")
        );
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

    #[test]
    fn nav_body_content_height_accounts_for_groups_items_and_gaps() {
        let summary = NavVisibilitySummary {
            visible_groups_count: 2,
            visible_items_count: 5,
            ..Default::default()
        };

        assert_eq!(
            nav_body_content_height(summary, Px(20.0), Px(6.0), Px(32.0), Px(24.0)),
            Px(254.0)
        );
    }
}
