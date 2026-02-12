use fret_app::{App, Model};
use fret_core::Px;
use fret_ui::element::{AnyElement, LayoutStyle, Length, ViewCacheProps};
use fret_ui::{ElementContext, Invalidation};
use fret_ui_shadcn::{decl_style, prelude::*};
use std::sync::Arc;

use crate::spec::{BISECT_SIMPLE_CONTENT, BISECT_SIMPLE_SIDEBAR, PAGE_INTRO};
use crate::ui;

pub(super) fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    bisect: u32,
    cache_shell: bool,
    nav_query: &Model<String>,
    selected_page: &Model<Arc<str>>,
    workspace_tabs: &Model<Vec<Arc<str>>>,
) -> AnyElement {
    if cache_shell {
        cx.view_cache(
            {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Px(Px(280.0));
                layout.size.height = Length::Fill;
                ViewCacheProps {
                    layout,
                    ..Default::default()
                }
            },
            |cx| {
                let selected = cx
                    .get_model_cloned(selected_page, Invalidation::Layout)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
                let query = cx
                    .get_model_cloned(nav_query, Invalidation::Layout)
                    .unwrap_or_default();

                vec![if (bisect & BISECT_SIMPLE_SIDEBAR) != 0 {
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(theme.color_required("muted")))
                                .p(Space::N4),
                            LayoutRefinement::default().w_px(Px(280.0)).h_full(),
                        ),
                        |cx| vec![cx.text("Sidebar (disabled)")],
                    )
                } else {
                    ui::sidebar_view(
                        cx,
                        theme,
                        selected.as_ref(),
                        query.as_str(),
                        nav_query.clone(),
                        selected_page.clone(),
                        workspace_tabs.clone(),
                    )
                }]
            },
        )
    } else {
        cx.keyed("ui_gallery.sidebar", |cx| {
            let selected = cx
                .get_model_cloned(selected_page, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));
            let query = cx
                .get_model_cloned(nav_query, Invalidation::Layout)
                .unwrap_or_default();

            if (bisect & BISECT_SIMPLE_SIDEBAR) != 0 {
                cx.container(
                    decl_style::container_props(
                        theme,
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(theme.color_required("muted")))
                            .p(Space::N4),
                        LayoutRefinement::default().w_px(Px(280.0)).h_full(),
                    ),
                    |cx| vec![cx.text("Sidebar (disabled)")],
                )
            } else {
                ui::sidebar_view(
                    cx,
                    theme,
                    selected.as_ref(),
                    query.as_str(),
                    nav_query.clone(),
                    selected_page.clone(),
                    workspace_tabs.clone(),
                )
            }
        })
    }
}

pub(super) fn content_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    bisect: u32,
    cache_shell: bool,
    selected_page: &Model<Arc<str>>,
    models: &ui::UiGalleryModels,
) -> AnyElement {
    if cache_shell {
        cx.view_cache(
            {
                let mut layout = LayoutStyle::default();
                layout.size.width = Length::Fill;
                layout.size.height = Length::Fill;
                layout.flex.grow = 1.0;
                ViewCacheProps {
                    layout,
                    ..Default::default()
                }
            },
            |cx| {
                let selected = cx
                    .get_model_cloned(selected_page, Invalidation::Layout)
                    .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

                vec![cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                    if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                        cx.container(
                            decl_style::container_props(
                                theme,
                                ChromeRefinement::default()
                                    .bg(ColorRef::Color(theme.color_required("background")))
                                    .p(Space::N6),
                                LayoutRefinement::default().w_full().h_full(),
                            ),
                            |cx| vec![cx.text("Content (disabled)")],
                        )
                    } else {
                        ui::content_view(cx, theme, selected.as_ref(), models)
                    }
                })]
            },
        )
    } else {
        cx.keyed("ui_gallery.content_root", |cx| {
            let selected = cx
                .get_model_cloned(selected_page, Invalidation::Layout)
                .unwrap_or_else(|| Arc::<str>::from(PAGE_INTRO));

            cx.keyed(("ui_gallery.content", selected.as_ref()), |cx| {
                if (bisect & BISECT_SIMPLE_CONTENT) != 0 {
                    cx.container(
                        decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(theme.color_required("background")))
                                .p(Space::N6),
                            LayoutRefinement::default().w_full().h_full(),
                        ),
                        |cx| vec![cx.text("Content (disabled)")],
                    )
                } else {
                    ui::content_view(cx, theme, selected.as_ref(), models)
                }
            })
        })
    }
}
