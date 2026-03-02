pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default, Clone)]
struct Models {
    open: Option<Model<bool>>,
}

fn open_model<H: UiHost>(cx: &mut ElementContext<'_, H>) -> Model<bool> {
    let state = cx.with_state(Models::default, |st| st.clone());
    match state.open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(model.clone()));
            model
        }
    }
}

fn dot_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    shadcn::breadcrumb::primitives::BreadcrumbSeparator::new()
        .kind(
            shadcn::breadcrumb::primitives::BreadcrumbSeparatorKind::Icon {
                icon: fret_icons::IconId::new_static("lucide.dot"),
                size: Px(14.0),
            },
        )
        .into_element(cx)
}

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let open = open_model(cx);

    fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        move |cx| {
            let crumb =
                shadcn::breadcrumb::primitives::Breadcrumb::new().into_element(cx, |cx| {
                    let list = shadcn::breadcrumb::primitives::BreadcrumbList::new()
                        .into_element(cx, |cx| {
                            let home = shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                                .into_element(cx, |cx| {
                                    vec![shadcn::breadcrumb::primitives::BreadcrumbLink::new("Home")
                                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                        .into_element(cx)]
                                });

                            let components_dropdown =
                                shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                                    .into_element(cx, |cx| {
                                        let menu = shadcn::DropdownMenu::new(open.clone())
                                            .align(shadcn::DropdownMenuAlign::End)
                                            .into_element(
                                                cx,
                                                move |cx| {
                                                    let theme = Theme::global(&*cx.app);
                                                    let fg = theme.color_token("foreground");
                                                    let muted = theme.color_token("muted-foreground");
                                                    let mut props =
                                                        fret_ui::element::PressableProps::default();
                                                    props.a11y.label =
                                                        Some(Arc::<str>::from("Components"));
                                                    props.a11y.test_id = Some(Arc::<str>::from(
                                                        "ui-gallery-breadcrumb-rtl-dropdown-trigger",
                                                    ));

                                                    cx.pressable(props, move |cx, st| {
                                                        let color = if st.hovered { fg } else { muted };
                                                        let label = fret_ui_kit::ui::text(cx, "Components")
                                                            .text_color(fret_ui_kit::ColorRef::Color(color))
                                                            .nowrap()
                                                            .into_element(cx);
                                                        let chevron = shadcn::icon::icon_with(
                                                            cx,
                                                            fret_icons::IconId::new_static(
                                                                "lucide.chevron-down",
                                                            ),
                                                            Some(Px(14.0)),
                                                            Some(fret_ui_kit::ColorRef::Color(color)),
                                                        );

                                                        vec![stack::hstack(
                                                            cx,
                                                            stack::HStackProps::default()
                                                                .gap_x(Space::N1)
                                                                .items_center(),
                                                            move |_cx| vec![label, chevron],
                                                        )]
                                                    })
                                                },
                                                |_cx| {
                                                    vec![
                                                        shadcn::DropdownMenuEntry::Item(
                                                            shadcn::DropdownMenuItem::new(
                                                                "Documentation",
                                                            )
                                                            .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                                            .test_id(
                                                                "ui-gallery-breadcrumb-rtl-dropdown-docs",
                                                            ),
                                                        ),
                                                        shadcn::DropdownMenuEntry::Item(
                                                            shadcn::DropdownMenuItem::new("Themes")
                                                                .on_activate(Arc::new(|_host, _acx, _reason| {})),
                                                        ),
                                                        shadcn::DropdownMenuEntry::Item(
                                                            shadcn::DropdownMenuItem::new("GitHub")
                                                                .on_activate(Arc::new(|_host, _acx, _reason| {})),
                                                        ),
                                                    ]
                                                },
                                            );
                                        vec![menu]
                                    });

                            let page = shadcn::breadcrumb::primitives::BreadcrumbItem::new()
                                .into_element(cx, |cx| {
                                    vec![shadcn::breadcrumb::primitives::BreadcrumbPage::new(
                                        "Breadcrumb",
                                    )
                                    .into_element(cx)]
                                });

                            vec![home, dot_separator(cx), components_dropdown, dot_separator(cx), page]
                        });

                    vec![list]
                });

            crumb.test_id("ui-gallery-breadcrumb-rtl")
        },
    )
}
// endregion: example
