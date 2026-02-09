use super::*;

#[test]
fn web_vs_fret_drawer_demo_surface_colors_match_web_tiny_viewport() {
    use fret_ui_shadcn::{Button, ButtonVariant, Drawer, DrawerContent};

    assert_overlay_surface_colors_match(
        "drawer-demo.vp1440x240",
        "drawer-content",
        "light",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Light,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2,
        |cx, open| {
            Drawer::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Drawer")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| DrawerContent::new(vec![cx.text("Drawer content")]).into_element(cx),
            )
        },
    );
}
