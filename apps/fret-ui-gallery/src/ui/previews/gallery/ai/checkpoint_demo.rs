use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_checkpoint_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use std::sync::Arc;

    use fret_runtime::Model;
    use fret_ui::Invalidation;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::{LayoutRefinement, Space};

    #[derive(Default)]
    struct DemoModels {
        clicked: Option<Model<bool>>,
    }

    let clicked = cx.with_state(DemoModels::default, |st| st.clicked.clone());
    let clicked = match clicked {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DemoModels::default, |st| st.clicked = Some(model.clone()));
            model
        }
    };

    let clicked_now = cx
        .get_model_copied(&clicked, Invalidation::Layout)
        .unwrap_or(false);

    let on_activate: fret_ui::action::OnActivate = Arc::new({
        let clicked = clicked.clone();
        move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&clicked, |v| *v = true);
            host.notify(action_cx);
        }
    });

    let row = ui_ai::Checkpoint::new([
        ui_ai::CheckpointIcon::default().into_element(cx),
        ui_ai::CheckpointTrigger::new([cx.text("Checkpoint")])
            .tooltip("Bookmark this moment")
            .tooltip_panel_test_id("ui-ai-checkpoint-tooltip-panel")
            .test_id("ui-ai-checkpoint-trigger")
            .on_activate(on_activate)
            .into_element(cx),
    ])
    .into_element(cx);

    let marker = clicked_now
        .then(|| cx.text("").test_id("ui-ai-checkpoint-clicked-marker"))
        .unwrap_or_else(|| cx.text(""));

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Checkpoint (AI Elements)"),
                cx.text("Hover to see tooltip; click to set a demo marker."),
                row,
                marker,
            ]
        },
    )]
}
