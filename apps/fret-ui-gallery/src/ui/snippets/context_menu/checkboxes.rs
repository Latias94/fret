pub const SOURCE: &str = include_str!("checkboxes.rs");

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

#[derive(Default, Clone)]
struct AppearanceState {
    show_status_bar: bool,
    show_activity_bar: bool,
    show_line_numbers: bool,
}

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
    let appearance = cx.local_model(|| AppearanceState {
        show_status_bar: true,
        show_activity_bar: true,
        show_line_numbers: false,
    });
    let appearance_now = cx
        .watch_model(&appearance)
        .layout()
        .cloned()
        .unwrap_or_default();

    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id("ui-gallery-context-menu-checkboxes-content")
        .compose()
        .trigger(trigger_surface(
            "Right click here",
            "Long press here",
            "ui-gallery-context-menu-checkboxes-trigger",
        ))
        .content(shadcn::ContextMenuContent::new())
        .entries([
            shadcn::ContextMenuEntry::CheckboxItem(
                shadcn::ContextMenuCheckboxItem::from_checked(
                    appearance_now.show_status_bar,
                    "Status Bar",
                )
                .on_checked_change({
                    let appearance = appearance.clone();
                    move |host, _action_cx, checked| {
                        let _ = host.models_mut().update(&appearance, |state| {
                            state.show_status_bar = checked;
                        });
                    }
                })
                .action(CommandId::new(
                    "ui_gallery.context_menu.checkboxes.status_bar",
                ))
                .test_id("ui-gallery-context-menu-checkboxes-status-bar"),
            ),
            shadcn::ContextMenuEntry::CheckboxItem(
                shadcn::ContextMenuCheckboxItem::from_checked(
                    appearance_now.show_activity_bar,
                    "Activity Bar",
                )
                .on_checked_change({
                    let appearance = appearance.clone();
                    move |host, _action_cx, checked| {
                        let _ = host.models_mut().update(&appearance, |state| {
                            state.show_activity_bar = checked;
                        });
                    }
                })
                .action(CommandId::new(
                    "ui_gallery.context_menu.checkboxes.activity_bar",
                )),
            ),
            shadcn::ContextMenuEntry::CheckboxItem(
                shadcn::ContextMenuCheckboxItem::from_checked(
                    appearance_now.show_line_numbers,
                    "Show Line Numbers",
                )
                .on_checked_change({
                    let appearance = appearance.clone();
                    move |host, _action_cx, checked| {
                        let _ = host.models_mut().update(&appearance, |state| {
                            state.show_line_numbers = checked;
                        });
                    }
                })
                .action(CommandId::new(
                    "ui_gallery.context_menu.checkboxes.show_line_numbers",
                )),
            ),
        ])
        .test_id("ui-gallery-context-menu-checkboxes")
}
// endregion: example
