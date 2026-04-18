pub const SOURCE: &str = include_str!("shimmer_elements_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::{FontId, FontWeight, Px, SemanticsRole, TextStyle};
use fret_ui_ai as ui_ai;
use fret_ui_kit::IntoUiElement;
use fret_ui_kit::ui;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};

fn item<B>(label: &'static str, el: B) -> impl UiChild + use<B>
where
    B: IntoUiElement<fret_app::App>,
{
    ui::v_stack(move |cx| {
        let theme = fret_ui::Theme::global(&*cx.app).snapshot();
        let muted = ColorRef::Color(fret_ui_kit::typography::muted_foreground_color(&theme));
        vec![
            ui::text(label).text_sm().text_color(muted).into_element(cx),
            el.into_element(cx),
        ]
    })
    .gap(Space::N3)
    .items_center()
}

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let heading_style = TextStyle {
        font: FontId::ui(),
        size: Px(24.0),
        weight: FontWeight::BOLD,
        line_height: Some(Px(28.0)),
        ..Default::default()
    };
    let custom_style = TextStyle {
        font: FontId::ui(),
        size: Px(18.0),
        weight: FontWeight::SEMIBOLD,
        line_height: Some(Px(22.0)),
        ..Default::default()
    };

    let paragraph = ui_ai::Shimmer::new("This is rendered as a paragraph").into_element(cx);
    let heading = ui_ai::Shimmer::new("Large Heading with Shimmer")
        .role(SemanticsRole::Heading)
        .text_style(heading_style)
        .into_element(cx);
    let inline = ui::h_row(move |cx| {
        vec![
            ui::text("Processing your request").into_element(cx),
            ui_ai::Shimmer::new("with AI magic").into_element(cx),
            ui::text("...").into_element(cx),
        ]
    })
    .gap(Space::N1)
    .items_center()
    .into_element(cx);
    let custom = ui_ai::Shimmer::new("Custom styled shimmer text")
        .text_style(custom_style)
        .into_element(cx);

    ui::v_flex(move |cx| {
        ui::children![
            cx;
            item("As paragraph (default)", paragraph),
            item("As heading", heading),
            item("As span (inline)", inline),
            item("As div with custom styling", custom),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N6)
    .into_element(cx)
    .test_id("ui-ai-shimmer-elements-root")
}
// endregion: example
