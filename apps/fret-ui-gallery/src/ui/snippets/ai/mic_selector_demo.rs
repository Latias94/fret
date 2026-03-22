pub const SOURCE: &str = include_str!("mic_selector_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
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

    let marker = cx
        .text(format!(
            "selected={}",
            selected.as_deref().unwrap_or("<none>")
        ))
        .attach_semantics(
            SemanticsDecoration::default()
                .role(fret_core::SemanticsRole::Generic)
                .test_id(if selected.is_some() {
                    "ui-ai-mic-selector-demo-selected"
                } else {
                    "ui-ai-mic-selector-demo-none"
                }),
        );

    let selector = ui_ai::MicSelector::from_arc(devices)
        .open_model(open.clone())
        .value_model(value.clone())
        .into_element_with_children(cx, move |cx, slot| match slot {
            ui_ai::MicSelectorChildSlot::Trigger => {
                ui_ai::MicSelectorTrigger::new([ui_ai::MicSelectorValue::new().into_element(cx)])
                    .test_id("ui-ai-mic-selector-demo-trigger")
                    .into_element(cx)
            }
            ui_ai::MicSelectorChildSlot::Content => ui_ai::MicSelectorContent::new([
                ui_ai::MicSelectorInput::new()
                    .test_id("ui-ai-mic-selector-demo-input")
                    .into_element(cx),
                ui_ai::MicSelectorList::new()
                    .empty(ui_ai::MicSelectorEmpty::new())
                    .into_element_with_children(cx, |cx, devices| {
                        devices
                            .iter()
                            .cloned()
                            .map(|device| {
                                ui_ai::MicSelectorItem::new(device.label.clone())
                                    .value(device.id.clone())
                                    .test_id(format!(
                                        "ui-ai-mic-selector-demo-item-{}",
                                        device.id.as_ref().replace(' ', "-")
                                    ))
                                    .children([
                                        ui_ai::MicSelectorLabel::new(device).into_element(cx)
                                    ])
                            })
                            .collect::<Vec<_>>()
                    }),
            ])
            .test_id_root("ui-ai-mic-selector-demo-content")
            .into_element(cx),
        });

    let open_now = cx
        .get_model_copied(&open, Invalidation::Paint)
        .unwrap_or(false);
    let open_marker = cx.text(format!("open={open_now}")).attach_semantics(
        SemanticsDecoration::default()
            .role(fret_core::SemanticsRole::Generic)
            .test_id(if open_now {
                "ui-ai-mic-selector-demo-open-true"
            } else {
                "ui-ai-mic-selector-demo-open-false"
            }),
    );

    ui::v_flex(move |cx| {
        vec![
            cx.text("MicSelector (AI Elements)"),
            cx.text(
                "Docs-aligned compound example. Device enumeration + permissions stay app-owned.",
            ),
            marker,
            open_marker,
            selector,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .test_id("ui-ai-mic-selector-demo-root")
    .into_element(cx)
}
// endregion: example
