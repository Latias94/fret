pub const SOURCE: &str = include_str!("overlay_demo.rs");

// region: example
use fret_app::App;
use fret_ui_shadcn::{self as shadcn, prelude::*};

pub fn render(cx: &mut ElementContext<'_, App>, dialog_open: Model<bool>) -> AnyElement {
    let shell_layout = LayoutRefinement::default()
        .w_full()
        .max_w(Px(760.0))
        .min_w_0();

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
}
// endregion: example
