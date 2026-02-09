use super::*;

#[test]
fn web_vs_fret_dialog_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_overlay_chrome_matches(
        "dialog-demo",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}
#[test]
fn web_vs_fret_dialog_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_overlay_chrome_matches_by_portal_slot("dialog-demo", "dialog-content", |cx, open| {
        Dialog::new(open.clone()).into_element(
            cx,
            |cx| {
                Button::new("Open Dialog")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| {
                DialogContent::new(vec![cx.text("Edit profile")])
                    .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                    .into_element(cx)
            },
        )
    });
}
#[test]
fn web_vs_fret_dialog_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ButtonVariant, Dialog, DialogContent};

    assert_overlay_chrome_matches_by_portal_slot_theme(
        "dialog-demo",
        "dialog-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_100 + 2,
        |cx, open| {
            Dialog::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open Dialog")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| {
                    DialogContent::new(vec![cx.text("Edit profile")])
                        .refine_layout(fret_ui_kit::LayoutRefinement::default().max_w(Px(425.0)))
                        .into_element(cx)
                },
            )
        },
    );
}
