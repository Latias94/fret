pub const SOURCE: &str = include_str!("mic_selector_demo.rs");

// region: example
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::element::SemanticsDecoration;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

#[derive(Default)]
struct DemoModels {
    open: Option<Model<bool>>,
    value: Option<Model<Option<Arc<str>>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.open.is_none() || st.value.is_none()
    });
    if needs_init {
        let open = cx.app.models_mut().insert(false);
        let value = cx.app.models_mut().insert(None::<Arc<str>>);
        cx.with_state(DemoModels::default, move |st| {
            st.open = Some(open.clone());
            st.value = Some(value.clone());
        });
    }

    let (open, value) = cx.with_state(DemoModels::default, |st| {
        (
            st.open.clone().expect("open"),
            st.value.clone().expect("value"),
        )
    });

    let devices: Arc<[ui_ai::MicSelectorDevice]> = Arc::from(vec![
        ui_ai::MicSelectorDevice::new("default", "Default Microphone (1234:abcd)"),
        ui_ai::MicSelectorDevice::new("usb", "USB Audio Device (5678:ef01)"),
        ui_ai::MicSelectorDevice::new("loopback", "Loopback"),
    ]);
    let list_devices = devices.clone();

    let selected = cx.app.models().read(&value, |v| v.clone()).ok().flatten();

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
        .into_element_with_children(cx, move |cx| {
            let trigger =
                ui_ai::MicSelectorTrigger::new([ui_ai::MicSelectorValue::new().into_element(cx)])
                    .test_id("ui-ai-mic-selector-demo-trigger")
                    .into_element(cx);

            let items = list_devices
                .iter()
                .cloned()
                .map(|device| {
                    ui_ai::MicSelectorItem::new(device.label.clone())
                        .value(device.id.clone())
                        .test_id(format!(
                            "ui-ai-mic-selector-demo-item-{}",
                            device.id.as_ref().replace(' ', "-")
                        ))
                        .children([ui_ai::MicSelectorLabel::new(device).into_element(cx)])
                })
                .collect::<Vec<_>>();

            let content = ui_ai::MicSelectorContent::new([
                ui_ai::MicSelectorInput::new()
                    .test_id("ui-ai-mic-selector-demo-input")
                    .into_element(cx),
                ui_ai::MicSelectorList::new_entries(items)
                    .empty(ui_ai::MicSelectorEmpty::new())
                    .into_element(cx),
            ])
            .test_id_root("ui-ai-mic-selector-demo-content")
            .into_element(cx);

            (trigger, content)
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
