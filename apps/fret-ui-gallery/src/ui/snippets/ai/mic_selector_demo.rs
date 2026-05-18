pub const SOURCE: &str = include_str!("mic_selector_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{Px, SemanticsRole};
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: SemanticsRole::Generic,
            test_id: Some(Arc::from(test_id)),
            ..Default::default()
        },
        |cx| {
            vec![cx.spacer(SpacerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(Px(0.0)),
                        height: Length::Px(Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                min: Px(0.0),
            })]
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
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

    let marker = state_marker(
        cx,
        if selected.is_some() {
            "ui-ai-mic-selector-demo-selected"
        } else {
            "ui-ai-mic-selector-demo-none"
        },
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
                            .children(|devices: Arc<[ui_ai::MicSelectorDevice]>| {
                                devices
                                    .iter()
                                    .cloned()
                                    .map(|device| {
                                        ui_ai::MicSelectorItem::new(device.label.clone())
                                            .value(device.id.clone())
                                            .child(ui_ai::MicSelectorLabel::new(device))
                                    })
                                    .collect::<Vec<_>>()
                            })
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
    let open_marker = state_marker(
        cx,
        if open_now {
            "ui-ai-mic-selector-demo-open-true"
        } else {
            "ui-ai-mic-selector-demo-open-false"
        },
    );

    ui::v_flex(move |cx| {
        vec![
            decl_text::text_section_chrome_label(cx, "MicSelector (AI Elements)"),
            decl_text::text_paragraph(
                cx,
                "Docs-shaped compound example with typed item rows. Device inventory + permissions stay app-owned.",
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
