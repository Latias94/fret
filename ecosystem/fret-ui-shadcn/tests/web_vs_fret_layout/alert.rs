use super::*;
use fret_ui_shadcn::facade as shadcn;

#[test]
fn web_vs_fret_layout_alert_demo_alerts_are_w_full_like_web() {
    let web = read_web_golden("alert-demo");
    let theme = web_theme(&web);

    let web_alerts = find_all_in_theme(theme, &|n| {
        n.tag == "div" && n.attrs.get("role").is_some_and(|r| r == "alert")
    });
    assert!(
        !web_alerts.is_empty(),
        "expected at least one role=alert node in alert-demo web golden"
    );

    let expected_w = web_alerts.iter().map(|n| n.rect.w).fold(0.0_f32, f32::max);
    assert!(expected_w > 0.0, "expected web alert width > 0");

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(theme.viewport.w), Px(theme.viewport.h)),
    );

    const ALERT_TEST_ID: &str = "Golden:alert-demo:w-full";
    let snap = run_fret_root(bounds, |cx| {
        let alert = shadcn::Alert::new([
            shadcn::AlertTitle::new("Success!").into_element(cx),
            shadcn::AlertDescription::new("Your changes have been saved.").into_element(cx),
        ])
        .into_element(cx)
        .test_id(ALERT_TEST_ID);

        vec![cx.column(
            ColumnProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Px(Px(expected_w)),
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                // Ensure `w-full` is required for the child to fill the available width.
                align: CrossAlign::Start,
                ..Default::default()
            },
            move |_cx| vec![alert],
        )]
    });

    let fret_alert = find_by_test_id(&snap, ALERT_TEST_ID);
    assert_close_px(
        "alert w-full",
        fret_alert.bounds.size.width,
        expected_w,
        1.0,
    );
}
