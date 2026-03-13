pub const SOURCE: &str = include_str!("sides.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_core::scene::DashPatternV1;
use fret_runtime::CommandId;
use fret_ui::Theme;
use fret_ui::element::{CrossAlign, GridProps};
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

fn trigger_surface<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let theme = Theme::global(&*cx.app);
    let border = theme.color_token("border");
    let bg = theme.color_token("background");
    let fg = theme.color_token("muted-foreground");

    let label = ui::text(label)
        .text_sm()
        .text_color(ColorRef::Color(fg))
        .into_element(cx);

    let content = ui::v_flex(move |_cx| vec![label])
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_center()
        .justify_center()
        .into_element(cx);

    shadcn::AspectRatio::with_child(content)
        .ratio(16.0 / 9.0)
        .refine_style(
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .border_1()
                .border_dash(DashPatternV1::new(Px(4.0), Px(4.0), Px(0.0)))
                .border_color(ColorRef::Color(border))
                .bg(ColorRef::Color(bg)),
        )
        .refine_layout(LayoutRefinement::default().w_full().max_w(Px(240.0)))
        .test_id(test_id)
}

fn side_menu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    side: shadcn::DropdownMenuSide,
    trigger_test_id: &'static str,
    content_test_id: &'static str,
) -> impl IntoUiElement<H> + use<H> {
    let trigger = shadcn::ContextMenuTrigger::build(trigger_surface(cx, label, trigger_test_id));

    shadcn::ContextMenu::uncontrolled(cx)
        .content_test_id(content_test_id)
        .build_parts(
            cx,
            trigger,
            shadcn::ContextMenuContent::new().side(side),
            |_cx| {
                vec![
                    shadcn::ContextMenuGroup::new(vec![
                        shadcn::ContextMenuItem::new("Back")
                            .action(CommandId::new("ui_gallery.context_menu.sides.back"))
                            .into(),
                        shadcn::ContextMenuItem::new("Forward")
                            .action(CommandId::new("ui_gallery.context_menu.sides.forward"))
                            .into(),
                        shadcn::ContextMenuItem::new("Reload")
                            .action(CommandId::new("ui_gallery.context_menu.sides.reload"))
                            .into(),
                    ])
                    .into(),
                ]
            },
        )
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let theme = Theme::global(&*cx.app).snapshot();
    let gap = MetricRef::space(Space::N4).resolve(&theme);
    let layout = decl_style::layout_style(
        &theme,
        LayoutRefinement::default()
            .w_full()
            .min_w_0()
            .max_w(Px(560.0)),
    );

    cx.grid(
        GridProps {
            layout,
            cols: 2,
            gap: gap.into(),
            align: CrossAlign::Start,
            ..Default::default()
        },
        |cx| {
            vec![
                side_menu(
                    cx,
                    "Right click (top)",
                    shadcn::DropdownMenuSide::Top,
                    "ui-gallery-context-menu-sides-top-trigger",
                    "ui-gallery-context-menu-sides-top-content",
                )
                .into_element(cx),
                side_menu(
                    cx,
                    "Right click (right)",
                    shadcn::DropdownMenuSide::Right,
                    "ui-gallery-context-menu-sides-right-trigger",
                    "ui-gallery-context-menu-sides-right-content",
                )
                .into_element(cx),
                side_menu(
                    cx,
                    "Right click (bottom)",
                    shadcn::DropdownMenuSide::Bottom,
                    "ui-gallery-context-menu-sides-bottom-trigger",
                    "ui-gallery-context-menu-sides-bottom-content",
                )
                .into_element(cx),
                side_menu(
                    cx,
                    "Right click (left)",
                    shadcn::DropdownMenuSide::Left,
                    "ui-gallery-context-menu-sides-left-trigger",
                    "ui-gallery-context-menu-sides-left-content",
                )
                .into_element(cx),
            ]
        },
    )
    .test_id("ui-gallery-context-menu-sides")
}
// endregion: example
