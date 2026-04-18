pub const SOURCE: &str = include_str!("destructive.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_core::scene::DashPatternV1;
use fret_icons::IconId;
use fret_runtime::CommandId;
use fret_ui::{Invalidation, Theme};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::primary_pointer_is_coarse;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(
    fine_label: &'static str,
    coarse_label: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    ui::h_flex(move |cx| {
        let theme = Theme::global(&*cx.app);
        let border = theme.color_token("border");
        let bg = theme.color_token("background");
        let fg = theme.color_token("muted-foreground");
        let label = if primary_pointer_is_coarse(cx, Invalidation::Layout, false) {
            coarse_label
        } else {
            fine_label
        };

        let label = ui::text(label)
            .text_sm()
            .text_color(ColorRef::Color(fg))
            .into_element(cx);

        let content = ui::v_flex(move |_cx| vec![label])
            .layout(LayoutRefinement::default().w_full().h_full())
            .items_center()
            .justify_center()
            .into_element(cx);

        [shadcn::AspectRatio::with_child(content)
            .ratio(16.0 / 9.0)
            .refine_style(
                ChromeRefinement::default()
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_dash(DashPatternV1::new(Px(4.0), Px(4.0), Px(0.0)))
                    .border_color(ColorRef::Color(border))
                    .bg(ColorRef::Color(bg)),
            )
            .refine_layout(LayoutRefinement::default().w_full().max_w(Px(320.0)))
            .into_element(cx)
            .test_id(test_id)]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .justify_center()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-destructive-content")
        .compose()
        .trigger(trigger_surface(
            "Right click here",
            "Long press here",
            "ui-gallery-context-menu-destructive-trigger",
        ))
        .content(shadcn::ContextMenuContent::new())
        .entries([
            shadcn::ContextMenuGroup::new([
                shadcn::ContextMenuItem::new("Edit")
                    .leading_icon(IconId::new_static("lucide.pencil"))
                    .action(CommandId::new("ui_gallery.context_menu.destructive.edit"))
                    .into(),
                shadcn::ContextMenuItem::new("Share")
                    .leading_icon(IconId::new_static("lucide.share"))
                    .action(CommandId::new("ui_gallery.context_menu.destructive.share"))
                    .into(),
            ])
            .into(),
            shadcn::ContextMenuEntry::Separator,
            shadcn::ContextMenuGroup::new([shadcn::ContextMenuItem::new("Delete")
                .leading_icon(IconId::new_static("lucide.trash"))
                .action(CommandId::new("ui_gallery.context_menu.destructive.delete"))
                .variant(shadcn::raw::context_menu::ContextMenuItemVariant::Destructive)
                .test_id("ui-gallery-context-menu-destructive-delete")
                .into()])
            .into(),
        ])
        .test_id("ui-gallery-context-menu-destructive")
}
// endregion: example
