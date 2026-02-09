use super::*;

#[test]
fn web_vs_fret_sheet_demo_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_overlay_chrome_matches("sheet-demo", "dialog", SemanticsRole::Dialog, |cx, open| {
        Sheet::new(open.clone()).into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
        )
    });
}
#[test]
fn web_vs_fret_sheet_demo_surface_colors_match_web() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_overlay_chrome_matches_by_portal_slot("sheet-demo", "sheet-content", |cx, open| {
        Sheet::new(open.clone()).into_element(
            cx,
            |cx| {
                Button::new("Open")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
        )
    });
}
#[test]
fn web_vs_fret_sheet_demo_surface_colors_match_web_dark() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent};

    assert_overlay_surface_colors_match(
        "sheet-demo",
        "sheet-content",
        "dark",
        fret_ui_shadcn::shadcn_themes::ShadcnColorScheme::Dark,
        SemanticsRole::Dialog,
        fret_ui_kit::declarative::overlay_motion::SHADCN_MOTION_TICKS_500 + 2,
        |cx, open| {
            Sheet::new(open.clone()).into_element(
                cx,
                |cx| {
                    Button::new("Open")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches("sheet-side", "dialog", SemanticsRole::Dialog, |cx, open| {
        Sheet::new(open.clone()).side(SheetSide::Top).into_element(
            cx,
            |cx| {
                Button::new("top")
                    .variant(ButtonVariant::Outline)
                    .into_element(cx)
            },
            |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
        )
    });
}
#[test]
fn web_vs_fret_sheet_side_right_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches(
        "sheet-side.right",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Right)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("right")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_bottom_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches(
        "sheet-side.bottom",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone())
                .side(SheetSide::Bottom)
                .into_element(
                    cx,
                    |cx| {
                        Button::new("bottom")
                            .variant(ButtonVariant::Outline)
                            .into_element(cx)
                    },
                    |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
                )
        },
    );
}
#[test]
fn web_vs_fret_sheet_side_left_panel_chrome_matches() {
    use fret_ui_shadcn::{Button, ButtonVariant, Sheet, SheetContent, SheetSide};

    assert_overlay_chrome_matches(
        "sheet-side.left",
        "dialog",
        SemanticsRole::Dialog,
        |cx, open| {
            Sheet::new(open.clone()).side(SheetSide::Left).into_element(
                cx,
                |cx| {
                    Button::new("left")
                        .variant(ButtonVariant::Outline)
                        .into_element(cx)
                },
                |cx| SheetContent::new(vec![cx.text("Edit profile")]).into_element(cx),
            )
        },
    );
}
