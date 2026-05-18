pub const SOURCE: &str = include_str!("tool_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

fn state_marker(cx: &mut AppComponentCx<'_>, test_id: &'static str) -> AnyElement {
    cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Generic,
            test_id: Some(Arc::<str>::from(test_id)),
            ..Default::default()
        },
        |cx| {
            vec![cx.spacer(SpacerProps {
                layout: fret_ui::element::LayoutStyle {
                    size: fret_ui::element::SizeStyle {
                        width: Length::Px(fret_core::Px(0.0)),
                        height: Length::Px(fret_core::Px(0.0)),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                min: fret_core::Px(0.0),
            })]
        },
    )
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let input_payload = ui_ai::model::ToolCallPayload::Json(serde_json::json!({
        "location": "San Francisco",
        "units": "fahrenheit",
    }));

    let weather_markdown = Arc::<str>::from(
        "**Weather for San Francisco**\n\n**Temperature:** 68°F  \n**Conditions:** Sunny  \n**Humidity:** 12%  \n**Wind Speed:** 35 mph  \n\n*Last updated: 2026-03-05 14:13*",
    );

    let pending_input = ui_ai::ToolInput::new(input_payload.clone()).into_element(cx);
    let running_input = ui_ai::ToolInput::new(input_payload.clone()).into_element(cx);

    let completed_input = ui_ai::ToolInput::new(input_payload.clone()).into_element(cx);
    let completed_output =
        ui_ai::ToolOutput::custom([ui_ai::MessageResponse::new(weather_markdown).into_element(cx)])
            .into_element(cx)
            .expect("tool output available");

    let error_input = ui_ai::ToolInput::new(input_payload).into_element(cx);
    let error_output = ui_ai::ToolOutput::new(
        None,
        Some(Arc::<str>::from("API error: weather provider timed out")),
    )
    .into_element(cx)
    .expect("tool output error");

    let pending = ui_ai::Tool::root()
        .children([
            ui_ai::ToolHeader::new("tool-fetch_weather_data", ui_ai::ToolStatus::InputStreaming)
                .into(),
            ui_ai::ToolContent::new([pending_input]).into(),
        ])
        .into_element(cx);

    let running = ui_ai::Tool::root()
        .children([
            ui_ai::ToolHeader::new("tool-fetch_weather_data", ui_ai::ToolStatus::InputAvailable)
                .into(),
            ui_ai::ToolContent::new([running_input]).into(),
        ])
        .into_element(cx);

    let completed = ui_ai::Tool::root()
        .default_open(true)
        .children([
            ui_ai::ToolHeader::new(
                "tool-fetch_weather_data",
                ui_ai::ToolStatus::OutputAvailable,
            )
            .test_id("ui-ai-tool-demo-trigger")
            .into(),
            ui_ai::ToolContent::new([
                state_marker(cx, "ui-ai-tool-demo-content-marker"),
                completed_input,
                completed_output,
            ])
            .into(),
        ])
        .into_element(cx);

    let error = ui_ai::Tool::root()
        .default_open(true)
        .children([
            ui_ai::ToolHeader::new("tool-fetch_weather_data", ui_ai::ToolStatus::OutputError)
                .into(),
            ui_ai::ToolContent::new([error_input, error_output]).into(),
        ])
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            decl_text::text_section_chrome_label(cx, "Tool (AI Elements)"),
            decl_text::text_paragraph(
                cx,
                "Docs-shaped compound composition with the four official Tool states.",
            ),
            decl_text::text_section_chrome_label(cx, "Input Streaming (Pending)"),
            pending,
            decl_text::text_section_chrome_label(cx, "Input Available (Running)"),
            running,
            decl_text::text_section_chrome_label(cx, "Output Available (Completed)"),
            completed,
            decl_text::text_section_chrome_label(cx, "Output Error"),
            error,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
