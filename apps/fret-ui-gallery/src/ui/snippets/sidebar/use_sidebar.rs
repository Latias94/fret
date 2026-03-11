pub const SOURCE: &str = include_str!("use_sidebar.rs");

// region: example
use fret_core::Px;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let content = shadcn::SidebarProvider::new()
        .width(Px(272.0))
        .width_icon(Px(56.0))
        .width_mobile(Px(336.0))
        .with(cx, |cx| {
            let sidebar_ctx = shadcn::use_sidebar(cx).expect("sidebar context");
            let open_now = cx
                .get_model_cloned(&sidebar_ctx.open, Invalidation::Layout)
                .unwrap_or(true);
            let open_mobile_now = cx
                .get_model_cloned(&sidebar_ctx.open_mobile, Invalidation::Layout)
                .unwrap_or(false);
            let state_label = if sidebar_ctx.collapsed() {
                "collapsed"
            } else {
                "expanded"
            };

            let sidebar =
                shadcn::Sidebar::new([shadcn::SidebarContent::new([shadcn::SidebarGroup::new([
                    shadcn::SidebarGroupLabel::new("Debug").into_element(cx),
                    shadcn::SidebarMenu::new([shadcn::SidebarMenuItem::new(
                        shadcn::SidebarMenuButton::new("Overview")
                            .icon(fret_icons::IconId::new_static("lucide.layout-dashboard"))
                            .active(true)
                            .into_element(cx),
                    )
                    .into_element(cx)])
                    .into_element(cx),
                ])
                .into_element(cx)])
                .into_element(cx)])
                .collapsible(shadcn::SidebarCollapsible::Icon)
                .refine_layout(LayoutRefinement::default().h_full())
                .into_element(cx);

            let summary = shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new("use_sidebar").into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![
                    shadcn::SidebarTrigger::new().into_element(cx),
                    cx.text(format!("state={state_label}")),
                    cx.text(format!("open={open_now}")),
                    cx.text(format!("open_mobile={open_mobile_now}")),
                    cx.text(format!("is_mobile={}", sidebar_ctx.is_mobile)),
                    cx.text(format!("width={:.0}px", sidebar_ctx.width.0)),
                    cx.text(format!("width_icon={:.0}px", sidebar_ctx.width_icon.0)),
                    cx.text(format!("width_mobile={:.0}px", sidebar_ctx.width_mobile.0)),
                ])
                .into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full().h_full().min_w_0())
            .into_element(cx);

            let inset = shadcn::SidebarInset::new([summary]).into_element(cx);
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
        .unwrap_or_else(|| cx.text("Missing use_sidebar example content."))
}

// endregion: example
