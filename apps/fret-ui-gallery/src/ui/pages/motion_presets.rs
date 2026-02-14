use super::super::*;

fn fmt_bezier(b: fret_ui::theme::CubicBezier) -> String {
    format!("{:.2}, {:.2}, {:.2}, {:.2}", b.x1, b.y1, b.x2, b.y2)
}

pub(super) fn preview_motion_presets(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    let preset = cx
        .watch_model(&motion_preset)
        .paint()
        .cloned_or_default()
        .unwrap_or_else(|| Arc::from("theme"));

    let shell_layout = LayoutRefinement::default()
        .w_full()
        .max_w(Px(760.0))
        .min_w_0();

    let preset_select = {
        let select = shadcn::Select::new(motion_preset, motion_preset_open)
            .placeholder("Motion preset")
            .trigger_test_id("ui-gallery-motion-preset-trigger")
            .items([
                shadcn::SelectItem::new("theme", "Theme (baseline)")
                    .test_id("ui-gallery-motion-preset-item-theme"),
                shadcn::SelectItem::new("snappy", "Snappy")
                    .test_id("ui-gallery-motion-preset-item-snappy"),
                shadcn::SelectItem::new("bouncy", "Bouncy")
                    .test_id("ui-gallery-motion-preset-item-bouncy"),
                shadcn::SelectItem::new("gentle", "Gentle")
                    .test_id("ui-gallery-motion-preset-item-gentle"),
            ])
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
            .into_element(cx);

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Preset selector").into_element(cx),
                shadcn::CardDescription::new(
                    "Applies a ThemeConfig patch (durations/easings/spring params) on top of the current theme preset.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N4)
                        .items_center(),
                    move |cx| {
                        vec![
                            select,
                            shadcn::Badge::new(format!("active: {}", preset.as_ref()))
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                        ]
                    },
                ),
            ])
            .into_element(cx),
        ])
        .refine_layout(shell_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-selector-card")
    };

    let token_snapshot = {
        let rows = [
            (
                "duration.shadcn.motion.100",
                theme
                    .duration_ms_token("duration.shadcn.motion.100")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.200",
                theme
                    .duration_ms_token("duration.shadcn.motion.200")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.300",
                theme
                    .duration_ms_token("duration.shadcn.motion.300")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.500",
                theme
                    .duration_ms_token("duration.shadcn.motion.500")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.overlay.open",
                theme
                    .duration_ms_token("duration.shadcn.motion.overlay.open")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.overlay.close",
                theme
                    .duration_ms_token("duration.shadcn.motion.overlay.close")
                    .to_string(),
            ),
            (
                "easing.shadcn.motion.overlay",
                fmt_bezier(theme.easing_token("easing.shadcn.motion.overlay")),
            ),
            (
                "duration.shadcn.motion.toast.enter",
                theme
                    .duration_ms_token("duration.shadcn.motion.toast.enter")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.toast.exit",
                theme
                    .duration_ms_token("duration.shadcn.motion.toast.exit")
                    .to_string(),
            ),
            (
                "duration.shadcn.motion.spring.drawer.settle",
                theme
                    .duration_ms_token("duration.shadcn.motion.spring.drawer.settle")
                    .to_string(),
            ),
            (
                "number.shadcn.motion.spring.drawer.settle.bounce",
                format!(
                    "{:.2}",
                    theme.number_token("number.shadcn.motion.spring.drawer.settle.bounce")
                ),
            ),
            (
                "duration.motion.presence.enter",
                theme
                    .duration_ms_token("duration.motion.presence.enter")
                    .to_string(),
            ),
            (
                "easing.motion.standard",
                fmt_bezier(theme.easing_token("easing.motion.standard")),
            ),
        ];

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3)
                .items_start(),
            move |cx| {
                rows.into_iter()
                    .map(|(key, value)| {
                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .justify_between()
                                .items_center()
                                .gap(Space::N4),
                            move |cx| {
                                vec![
                                    cx.text(key),
                                    shadcn::Badge::new(value)
                                        .variant(shadcn::BadgeVariant::Outline)
                                        .into_element(cx),
                                ]
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            },
        );

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Token snapshot").into_element(cx),
                shadcn::CardDescription::new(
                    "Current effective values for a small, shared set of motion tokens.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([content]).into_element(cx),
        ])
        .refine_layout(shell_layout.clone())
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-token-snapshot")
    };

    let overlay_demo = {
        let open_for_trigger = dialog_open.clone();
        let open_for_close = dialog_open.clone();

        let dialog = shadcn::Dialog::new(dialog_open)
            .into_element(
                cx,
                move |cx| {
                    shadcn::Button::new("Open dialog (presence)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .toggle_model(open_for_trigger.clone())
                        .test_id("ui-gallery-motion-presets-dialog-trigger")
                        .into_element(cx)
                },
                move |cx| {
                    shadcn::DialogContent::new([
                        shadcn::DialogHeader::new([
                            shadcn::DialogTitle::new("Motion preset demo").into_element(cx),
                            shadcn::DialogDescription::new(
                                "Switch motion presets to compare presence timing + easing under fixed frame delta gates.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::DialogFooter::new([shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Outline)
                            .toggle_model(open_for_close.clone())
                            .test_id("ui-gallery-motion-presets-dialog-close")
                            .into_element(cx)])
                        .into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id("ui-gallery-motion-presets-dialog-content")
                },
            )
            .test_id("ui-gallery-motion-presets-dialog");

        shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new("Overlay demo").into_element(cx),
                shadcn::CardDescription::new(
                    "Presence motion is token-driven and should feel consistent across refresh rates.",
                )
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([dialog]).into_element(cx),
        ])
        .refine_layout(shell_layout)
        .into_element(cx)
        .test_id("ui-gallery-motion-presets-overlay-demo")
    };

    vec![preset_select, token_snapshot, overlay_demo]
}
