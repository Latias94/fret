pub const SOURCE: &str = include_str!("tool_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use std::sync::Arc;

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
                cx.text("").test_id("ui-ai-tool-demo-content-marker"),
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
            cx.text("Tool (AI Elements)"),
            cx.text("Docs-shaped compound composition with the four official Tool states."),
            cx.text("Input Streaming (Pending)"),
            pending,
            cx.text("Input Available (Running)"),
            running,
            cx.text("Output Available (Completed)"),
            completed,
            cx.text("Output Error"),
            error,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
