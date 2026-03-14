pub const SOURCE: &str = include_str!("usage.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let content = shadcn::SidebarProvider::new()
        .width(Px(288.0))
        .width_icon(Px(64.0))
        .width_mobile(Px(320.0))
        .with(cx, |cx| {
            let primary = shadcn::SidebarGroup::new([
                shadcn::SidebarGroupLabel::new("Application").into_element(cx),
                shadcn::SidebarMenu::new([
                    shadcn::SidebarMenuItem::new(
                        shadcn::SidebarMenuButton::new("Home")
                            .icon(fret_icons::IconId::new_static("lucide.house"))
                            .active(true)
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(
                        shadcn::SidebarMenuButton::new("Projects")
                            .icon(fret_icons::IconId::new_static("lucide.folder-kanban"))
                            .into_element(cx),
                    )
                    .into_element(cx),
                    shadcn::SidebarMenuItem::new(
                        shadcn::SidebarMenuButton::new("Settings")
                            .icon(fret_icons::IconId::new_static("lucide.settings-2"))
                            .into_element(cx),
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx);

            let sidebar = shadcn::Sidebar::new([
                shadcn::SidebarHeader::new([
                    shadcn::raw::typography::small("Workspace").into_element(cx)
                ])
                .into_element(cx),
                shadcn::SidebarContent::new([primary]).into_element(cx),
                shadcn::SidebarFooter::new([shadcn::raw::typography::muted(
                    "Sidebar width is owned by SidebarProvider.",
                )
                .into_element(cx)])
                .into_element(cx),
            ])
            .collapsible(shadcn::SidebarCollapsible::Icon)
            .refine_layout(LayoutRefinement::default().h_full())
            .into_element(cx);

            let inset = shadcn::SidebarInset::new([shadcn::card(|cx| {
                ui::children![
                    cx;
                    shadcn::card_header(
                        |cx| ui::children![cx; shadcn::card_title("Minimal usage")],
                    ),
                    shadcn::card_content(|cx| {
                        vec![
                            shadcn::SidebarTrigger::new().into_element(cx),
                            cx.text("Use SidebarProvider to own width defaults and state."),
                            cx.text("Sidebar keeps theme-token fallbacks for recipe chrome."),
                        ]
                    }),
                ]
            })
            .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
            .into_element(cx)])
            .into_element(cx);

            let frame = ui::h_flex(|_cx| vec![sidebar, inset])
                .gap(Space::N4)
                .items_start()
                .layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
                .into_element(cx);

            vec![frame]
        });

    content
        .into_iter()
        .next()
        .unwrap_or_else(|| cx.text("Missing sidebar usage content."))
}

// endregion: example
