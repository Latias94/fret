pub const SOURCE: &str = include_str!("test_results_large_demo.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui::element::{AnyElement, Length, SemanticsProps, SpacerProps};
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_kit::ui;
use fret_ui_kit::{LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
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
    let activated = cx.local_model_keyed("activated", || false);

    let activated_now = cx
        .get_model_copied(&activated, Invalidation::Layout)
        .unwrap_or(false);

    let on_activate: ui_ai::OnTestActivate = Arc::new({
        let activated = activated.clone();
        move |host, action_cx, _name| {
            let _ = host.models_mut().update(&activated, |v| *v = true);
            host.notify(action_cx);
        }
    });

    let mut tests: Vec<AnyElement> = Vec::new();
    for i in 0..500u32 {
        let id = format!("ui-ai-test-results-large-test-{i:04}");
        let name = Arc::<str>::from(format!("test_{i:04}"));
        let mut row = ui_ai::Test::new(name.clone(), ui_ai::TestStatusKind::Passed)
            .duration_ms(3)
            .test_id(id.clone());
        if i == 350 {
            row = row.on_activate(on_activate.clone());
        }
        tests.push(row.into_element(cx));
    }

    let marker =
        activated_now.then(|| state_marker(cx, "ui-ai-test-results-large-activated-marker"));

    let suite = ui_ai::TestSuite::new(
        ui_ai::TestSuiteName::new("Large suite", ui_ai::TestStatusKind::Passed)
            .stats(500, 0, 0)
            .test_id("ui-ai-test-results-large-suite-trigger"),
        ui_ai::TestSuiteContent::new(tests).test_id("ui-ai-test-results-large-suite-content"),
    )
    .default_open(true)
    .into_element(cx);

    let mut test_results_children = vec![suite];
    if let Some(marker) = marker {
        test_results_children.push(marker);
    }

    let root = ui_ai::TestResults::new()
        .summary(ui_ai::TestResultsSummaryData::new(500, 0, 0, 500).duration_ms(888))
        .children(test_results_children)
        .test_id_root("ui-ai-test-results-large-root")
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            decl_text::text_section_chrome_label(cx, "Test Results Large (AI Elements)"),
            decl_text::text_paragraph(cx, "Scroll the page and click a deep row to set a marker."),
            root,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
