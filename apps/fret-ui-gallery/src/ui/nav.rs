use super::*;

fn matches_query(query: &str, item: &PageSpec) -> bool {
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

pub(crate) fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let title_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            vec![
                cx.text("Fret UI Gallery"),
                shadcn::Badge::new("WIP")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        },
    );

    let query_input = {
        let nav_query = nav_query.clone();
        shadcn::Input::new(nav_query.clone())
            .a11y_label("Search components")
            .placeholder("Search (id / tag)")
            .into_element(cx)
            .test_id("ui-gallery-nav-search")
    };

    let mut nav_sections: Vec<AnyElement> = Vec::new();
    for group in PAGE_GROUPS {
        let group_sections = cx.keyed(group.title, |cx| {
            let mut group_items: Vec<AnyElement> = Vec::new();
            for item in group.items {
                if !matches_query(query, item) {
                    continue;
                }

                let is_selected = selected == item.id;

                group_items.push(cx.keyed(item.id, |cx| {
                    let selected_page_for_activate = selected_page.clone();
                    let workspace_tabs_for_activate = workspace_tabs.clone();
                    let page_id_for_activate: Arc<str> = Arc::from(item.id);

                    let on_activate: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host.models_mut().update(&selected_page_for_activate, |v| {
                                *v = page_id_for_activate.clone();
                            });
                            let _ = host.models_mut().update(&workspace_tabs_for_activate, |t| {
                                if !t
                                    .iter()
                                    .any(|id| id.as_ref() == page_id_for_activate.as_ref())
                                {
                                    t.push(page_id_for_activate.clone());
                                }
                            });
                            host.request_redraw(action_cx.window);
                            // `request_redraw()` may be coalesced or fail to wake the event loop on some
                            // platforms/driver configurations. Ensure we get at least one follow-up turn
                            // so the new page presents promptly after navigation.
                            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                                action_cx.window,
                            ));
                        });
                    shadcn::SidebarMenuButton::new(item.label)
                        .active(is_selected)
                        .collapsed(false)
                        .on_click(item.command)
                        .on_activate(on_activate)
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
                    text: Arc::from(group.title),
                    style: None,
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }),
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N1),
                    |_cx| group_items,
                ),
            ]
        });

        nav_sections.extend(group_sections);
    }

    let nav_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| nav_sections,
    );
    let nav_scroll = {
        let nav_scroll = if (bisect & BISECT_DISABLE_SIDEBAR_SCROLL) != 0 {
            nav_body
        } else {
            shadcn::ScrollArea::new([nav_body])
                .refine_layout(LayoutRefinement::default().w_full().h_full())
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
                        .unwrap_or_else(|| theme.color_required("background")),
                ))
                .p(Space::N4),
            LayoutRefinement::default().w_px(Px(280.0)).h_full(),
        ),
        |cx| {
            [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N4),
                |_cx| [title_row, query_input, nav_scroll],
            )]
        },
    );

    container
}
