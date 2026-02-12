use super::super::super::*;

pub(in crate::ui) fn preview_material3_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_core::{Corners, Px};
    use fret_ui::element::{AnyElement, ContainerProps, Length};

    let anchor = |cx: &mut ElementContext<'_, App>, size: Px, test_id: &'static str| {
        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Px(size);
        props.layout.size.height = Length::Px(size);
        props.background =
            Some(cx.with_theme(|theme| theme.color_required("md.sys.color.surface-container-low")));
        props.corner_radii = Corners::all(Px(8.0));
        cx.container(props, |_cx| Vec::<AnyElement>::new())
            .test_id(test_id)
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        |cx| {
            let small = Px(24.0);
            vec![
                material3::Badge::dot()
                    .navigation_anchor_size(small)
                    .test_id("ui-gallery-material3-badge-dot-nav")
                    .into_element(cx, |cx| vec![anchor(cx, small, "badge-anchor-dot-nav")]),
                material3::Badge::text("9")
                    .navigation_anchor_size(small)
                    .test_id("ui-gallery-material3-badge-text-nav")
                    .into_element(cx, |cx| vec![anchor(cx, small, "badge-anchor-text-nav")]),
                material3::Badge::dot()
                    .placement(material3::BadgePlacement::TopRight)
                    .test_id("ui-gallery-material3-badge-dot-top-right")
                    .into_element(cx, |cx| {
                        vec![anchor(cx, Px(40.0), "badge-anchor-dot-top-right")]
                    }),
                material3::Badge::text("99+")
                    .placement(material3::BadgePlacement::TopRight)
                    .test_id("ui-gallery-material3-badge-text-top-right")
                    .into_element(cx, |cx| {
                        vec![anchor(cx, Px(40.0), "badge-anchor-text-top-right")]
                    }),
            ]
        },
    );

    vec![
        cx.text("Material 3 Badge: dot + large/value variants via md.comp.badge.*."),
        row,
    ]
}

pub(in crate::ui) fn preview_material3_top_app_bar(
    cx: &mut ElementContext<'_, App>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui_material3::{
        TopAppBar, TopAppBarAction, TopAppBarScrollBehavior, TopAppBarVariant,
    };

    let bar = |cx: &mut ElementContext<'_, App>,
               variant: TopAppBarVariant,
               scrolled: bool,
               title: &'static str,
               test_id: &'static str| {
        TopAppBar::new(title)
            .variant(variant)
            .scrolled(scrolled)
            .navigation_icon(
                TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                    .a11y_label("Navigate")
                    .test_id(format!("{test_id}-nav")),
            )
            .actions(vec![
                TopAppBarAction::new(ids::ui::SEARCH)
                    .a11y_label("Search")
                    .test_id(format!("{test_id}-search")),
                TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                    .a11y_label("More actions")
                    .test_id(format!("{test_id}-more")),
            ])
            .test_id(test_id)
            .into_element(cx)
    };

    let scroll_demo = |cx: &mut ElementContext<'_, App>,
                       key: &'static str,
                       title: &'static str,
                       variant: TopAppBarVariant,
                       behavior: fn(fret_ui::scroll::ScrollHandle) -> TopAppBarScrollBehavior,
                       test_prefix: &'static str| {
        cx.keyed(key, |cx| {
            let scroll_handle =
                cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
            let behavior = cx.with_state(
                || behavior(scroll_handle.clone()),
                |behavior| behavior.clone(),
            );
            let bar = TopAppBar::new(title)
                .variant(variant)
                .scroll_behavior(behavior)
                .navigation_icon(
                    TopAppBarAction::new(ids::ui::CHEVRON_RIGHT)
                        .a11y_label("Navigate")
                        .test_id(format!("{test_prefix}-nav")),
                )
                .actions(vec![
                    TopAppBarAction::new(ids::ui::MORE_HORIZONTAL)
                        .a11y_label("More actions")
                        .test_id(format!("{test_prefix}-more")),
                ])
                .test_id(test_prefix)
                .into_element(cx);

            let mut content_props = stack::VStackProps::default();
            content_props.gap = Space::N2;
            let content = stack::vstack(cx, content_props, |cx| {
                let mut out: Vec<AnyElement> = Vec::new();
                out.push(cx.text("Scroll this area to drive the TopAppBar scroll behavior."));
                for i in 0..80usize {
                    out.push(cx.text(format!("Row {i:02}")));
                }
                out
            });

            let scroll = shadcn::ScrollArea::new([content])
                .scroll_handle(scroll_handle.clone())
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(240.0)))
                .viewport_test_id(format!("{test_prefix}-scroll-viewport"))
                .into_element(cx);

            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4),
                |_cx| [bar, scroll],
            )
        })
    };

    let mut props = stack::VStackProps::default();
    props.gap = Space::N4;
    let content = stack::vstack(cx, props, |cx| {
        vec![
            cx.text("Material 3 Top App Bar: primitives driven by md.comp.top-app-bar.* tokens."),
            cx.text("Scroll behavior demos (policy-only, no fret-ui mechanism changes):"),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-pinned",
                "Pinned scroll behavior (toggle scrolled container treatment)",
                TopAppBarVariant::Small,
                TopAppBarScrollBehavior::pinned,
                "ui-gallery-material3-top-app-bar-pinned",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-enter-always",
                "EnterAlways scroll behavior (collapses fully, shows on reverse scroll)",
                TopAppBarVariant::Small,
                TopAppBarScrollBehavior::enter_always,
                "ui-gallery-material3-top-app-bar-enter-always",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-enter-always-settle-on-idle",
                "EnterAlways + settleOnIdle (policy-only spring settle after idle)",
                TopAppBarVariant::Small,
                |h| TopAppBarScrollBehavior::enter_always(h).settle_on_idle(),
                "ui-gallery-material3-top-app-bar-enter-always-settle-on-idle",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-exit-until-collapsed",
                "ExitUntilCollapsed scroll behavior (Large collapses down to Small height)",
                TopAppBarVariant::Large,
                TopAppBarScrollBehavior::exit_until_collapsed,
                "ui-gallery-material3-top-app-bar-exit-until-collapsed",
            ),
            scroll_demo(
                cx,
                "ui-gallery-material3-top-app-bar-scroll-exit-until-collapsed-settle-on-idle",
                "ExitUntilCollapsed + settleOnIdle (policy-only snap; content moves)",
                TopAppBarVariant::Large,
                |h| TopAppBarScrollBehavior::exit_until_collapsed(h).settle_on_idle(),
                "ui-gallery-material3-top-app-bar-exit-until-collapsed-settle-on-idle",
            ),
            bar(
                cx,
                TopAppBarVariant::Small,
                false,
                "Small (idle)",
                "ui-gallery-material3-top-app-bar-small",
            ),
            bar(
                cx,
                TopAppBarVariant::Small,
                true,
                "Small (scrolled)",
                "ui-gallery-material3-top-app-bar-small-scrolled",
            ),
            bar(
                cx,
                TopAppBarVariant::SmallCentered,
                false,
                "Small Centered (idle)",
                "ui-gallery-material3-top-app-bar-small-centered",
            ),
            bar(
                cx,
                TopAppBarVariant::SmallCentered,
                true,
                "Small Centered (scrolled)",
                "ui-gallery-material3-top-app-bar-small-centered-scrolled",
            ),
            bar(
                cx,
                TopAppBarVariant::Medium,
                false,
                "Medium (idle)",
                "ui-gallery-material3-top-app-bar-medium",
            ),
            bar(
                cx,
                TopAppBarVariant::Medium,
                true,
                "Medium (scrolled)",
                "ui-gallery-material3-top-app-bar-medium-scrolled",
            ),
            bar(
                cx,
                TopAppBarVariant::Large,
                false,
                "Large (idle)",
                "ui-gallery-material3-top-app-bar-large",
            ),
            bar(
                cx,
                TopAppBarVariant::Large,
                true,
                "Large (scrolled)",
                "ui-gallery-material3-top-app-bar-large-scrolled",
            ),
        ]
    });

    vec![content]
}
