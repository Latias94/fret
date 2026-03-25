pub const SOURCE: &str = include_str!("mic_selector_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_core::{Px, SemanticsRole};
use fret_ui::Invalidation;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let value = cx.local_model_keyed("value", || None::<Arc<str>>);

    let devices: Arc<[ui_ai::MicSelectorDevice]> = Arc::from(vec![
        ui_ai::MicSelectorDevice::new("default", "Default Microphone (1234:abcd)"),
        ui_ai::MicSelectorDevice::new("usb", "USB Audio Device (5678:ef01)"),
        ui_ai::MicSelectorDevice::new("loopback", "Loopback"),
    ]);

    let selected = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or(None);

    let marker = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(Arc::from(if selected.is_some() {
                "ui-ai-mic-selector-demo-selected"
            } else {
                "ui-ai-mic-selector-demo-none"
            })),
            ..Default::default()
        },
        |cx| vec![cx.text("")],
    );

    let selector = ui_ai::MicSelector::from_arc(devices.clone())
        .open_model(open.clone())
        .value_model(value.clone())
        .children([
            ui_ai::MicSelectorChild::Trigger(
                ui_ai::MicSelectorTrigger::new([])
                    .value(ui_ai::MicSelectorValue::new())
                    .test_id("ui-ai-mic-selector-demo-trigger")
                    .refine_layout(
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .max_w(Px(384.0)),
                    ),
            ),
            ui_ai::MicSelectorChild::Content(
                ui_ai::MicSelectorContent::new([])
                    .input(ui_ai::MicSelectorInput::new().test_id("ui-ai-mic-selector-demo-input"))
                    .list(
                        ui_ai::MicSelectorList::new()
                            .empty(ui_ai::MicSelectorEmpty::new())
                            .test_id_prefix("ui-ai-mic-selector-demo-item"),
                    )
                    .test_id_root("ui-ai-mic-selector-demo-content"),
            ),
        ])
        .into_element(cx);

    let open_now = cx
        .get_model_copied(&open, Invalidation::Paint)
        .unwrap_or(false);
    let open_marker = cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(Arc::from(if open_now {
                "ui-ai-mic-selector-demo-open-true"
            } else {
                "ui-ai-mic-selector-demo-open-false"
            })),
            ..Default::default()
        },
        |cx| vec![cx.text("")],
    );

    ui::v_flex(move |cx| {
        vec![
            cx.text("MicSelector (AI Elements)"),
            cx.text(
                "Docs-aligned compound example. Device enumeration + permissions stay app-owned.",
            ),
            ui::h_flex(move |_cx| vec![selector])
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .justify_center()
                .into_element(cx),
            marker,
            open_marker,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .test_id("ui-ai-mic-selector-demo-root")
    .into_element(cx)
}
// endregion: example
