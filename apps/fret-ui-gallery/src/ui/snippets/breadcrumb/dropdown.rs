pub const SOURCE: &str = include_str!("dropdown.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_kit::IntoUiElement;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use shadcn::raw::breadcrumb::primitives as bc;
use std::sync::Arc;

fn slash_separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> impl IntoUiElement<H> + use<H> {
    bc::BreadcrumbSeparator::new()
        .children(|cx| {
            [shadcn::raw::icon::icon_with(
                cx,
                fret_icons::ids::ui::SLASH,
                Some(Px(14.0)),
                None,
            )]
        })
        .into_element(cx)
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model(|| false);

    let crumb = bc::Breadcrumb::new().into_element(cx, |cx| {
        let list = bc::BreadcrumbList::new().into_element(cx, |cx| {
            let home = bc::BreadcrumbItem::new().into_element(cx, |cx| {
                vec![
                    bc::BreadcrumbLink::new("Home")
                        .href("/")
                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                        .into_element(cx)
                        .test_id("ui-gallery-breadcrumb-dropdown-home-link"),
                ]
            });

            let components_dropdown = bc::BreadcrumbItem::new().into_element(cx, |cx| {
                let menu = shadcn::DropdownMenu::from_open(open.clone())
                    .align(shadcn::DropdownMenuAlign::Start)
                    .into_element(
                        cx,
                        move |cx| {
                            let theme = Theme::global(&*cx.app);
                            let fg = theme.color_token("foreground");
                            let muted = theme.color_token("muted-foreground");
                            let mut props = fret_ui::element::PressableProps::default();
                            props.a11y.label = Some(Arc::<str>::from("Components"));
                            props.a11y.test_id =
                                Some(Arc::<str>::from("ui-gallery-breadcrumb-dropdown-trigger"));

                            cx.pressable(props, move |cx, st| {
                                let color = if st.hovered { fg } else { muted };
                                let label = fret_ui_kit::ui::text("Components")
                                    .text_color(fret_ui_kit::ColorRef::Color(color))
                                    .nowrap()
                                    .into_element(cx);
                                let chevron = shadcn::raw::icon::icon_with(
                                    cx,
                                    fret_icons::IconId::new_static("lucide.chevron-down"),
                                    Some(Px(14.0)),
                                    Some(fret_ui_kit::ColorRef::Color(color)),
                                );

                                vec![
                                    ui::h_row(move |_cx| vec![label, chevron])
                                        .gap(Space::N1)
                                        .items_center()
                                        .into_element(cx),
                                ]
                            })
                        },
                        |_cx| {
                            vec![
                                shadcn::DropdownMenuEntry::Item(
                                    shadcn::DropdownMenuItem::new("Documentation")
                                        .on_activate(Arc::new(|_host, _acx, _reason| {}))
                                        .test_id("ui-gallery-breadcrumb-dropdown-docs"),
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

            let page = bc::BreadcrumbItem::new().into_element(cx, |cx| {
                vec![bc::BreadcrumbPage::new("Breadcrumb").into_element(cx)]
            });

            vec![
                home,
                slash_separator(cx).into_element(cx),
                components_dropdown,
                slash_separator(cx).into_element(cx),
                page,
            ]
        });

        vec![list]
    });

    crumb.test_id("ui-gallery-breadcrumb-dropdown")
}
// endregion: example
