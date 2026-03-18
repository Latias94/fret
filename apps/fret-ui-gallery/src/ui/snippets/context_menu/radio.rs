pub const SOURCE: &str = include_str!("radio.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_core::scene::DashPatternV1;
use fret_runtime::CommandId;
use fret_ui::{Invalidation, Theme};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::ModelWatchExt as _;
use fret_ui_kit::declarative::primary_pointer_is_coarse;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

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

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme_mode = cx.local_model(|| Some(Arc::<str>::from("system")));
    let theme_mode_now = cx.watch_model(&theme_mode).layout().cloned().flatten();

    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-radio-content")
        .compose()
        .trigger(trigger_surface(
            "Right click here",
            "Long press here",
            "ui-gallery-context-menu-radio-trigger",
        ))
        .content(shadcn::ContextMenuContent::new())
        .entries([shadcn::ContextMenuEntry::RadioGroup(
            shadcn::ContextMenuRadioGroup::from_value(theme_mode_now)
                .on_value_change({
                    let theme_mode = theme_mode.clone();
                    move |host, _action_cx, value| {
                        let _ = host
                            .models_mut()
                            .update(&theme_mode, |selected| *selected = Some(value));
                    }
                })
                .item(
                    shadcn::ContextMenuRadioItemSpec::new("system", "System")
                        .action(CommandId::new("ui_gallery.context_menu.radio.theme.system")),
                )
                .item(
                    shadcn::ContextMenuRadioItemSpec::new("light", "Light")
                        .action(CommandId::new("ui_gallery.context_menu.radio.theme.light")),
                )
                .item(
                    shadcn::ContextMenuRadioItemSpec::new("dark", "Dark")
                        .action(CommandId::new("ui_gallery.context_menu.radio.theme.dark")),
                ),
        )])
        .test_id("ui-gallery-context-menu-radio")
}
// endregion: example
