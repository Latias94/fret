use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::slider as snippets;

pub(super) fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        #[derive(Default)]
        struct SliderPageState {
            last_commit: Option<Model<Vec<f32>>>,
            controlled_values: Option<Model<Vec<f32>>>,
        }

        let last_commit = cx.with_state(SliderPageState::default, |st| st.last_commit.clone());
        let last_commit = match last_commit {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(Vec::<f32>::new());
                cx.with_state(SliderPageState::default, |st| {
                    st.last_commit = Some(model.clone());
                });
                model
            }
        };

        let controlled_values =
            cx.with_state(SliderPageState::default, |st| st.controlled_values.clone());
        let controlled_values = match controlled_values {
            Some(model) => model,
            None => {
                let model = cx.app.models_mut().insert(vec![0.3, 0.7]);
                cx.with_state(SliderPageState::default, |st| {
                    st.controlled_values = Some(model.clone());
                });
                model
            }
        };

        let demo = cx.keyed("ui_gallery.slider.demo", |cx| {
            snippets::demo::render(cx, controlled_values.clone())
        });
        let usage = cx.keyed("ui_gallery.slider.usage", |cx| snippets::usage::render(cx));
        let label = cx.keyed("ui_gallery.slider.label", |cx| snippets::label::render(cx));

        let notes = doc_layout::notes(
            cx,
            [
                "API reference: `ecosystem/fret-ui-shadcn/src/slider.rs` (Slider).",
                "Slider already exposes the important authoring surface (`new`, `new_controllable`, range/step/orientation/on_value_commit), so the main parity gap here is usage clarity rather than missing composition APIs.",
                "Uncontrolled sliders store their values in element state; controlled sliders store values in a shared model.",
                "Prefer `on_value_commit` for expensive reactions (e.g. save, fetch) and use live updates for lightweight UI.",
                "Vertical sliders should have an explicit height to avoid zero-size layouts.",
            ],
        );

        let extras = cx.keyed("ui_gallery.slider.extras", |cx| {
            snippets::extras::render(cx, last_commit.clone())
        });

        let body = doc_layout::render_doc_page(
            cx,
            Some("Preview follows shadcn Slider docs flow: Demo -> Usage. Extras cover label association and Fret-specific interaction variants."),
            vec![
                DocSection::new("Demo", demo)
                    .description("shadcn demo: single, range, multiple, vertical, and controlled.")
                    .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
                DocSection::new("Usage", usage)
                    .description("Copyable minimal usage for `Slider`.")
                    .test_id_prefix("ui-gallery-slider-usage")
                    .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
                DocSection::new("Label Association", label)
                    .description("Use `FieldLabel::for_control`, `Slider::control_id`, and `Slider::test_id_prefix` to focus the active thumb and keep derived automation anchors stable.")
                    .test_id_prefix("ui-gallery-slider-label")
                    .code_rust_from_file_region(snippets::label::SOURCE, "example"),
                DocSection::new("Extras", extras)
                    .description("Fret extras: disabled, RTL, inverted, and onValueCommit.")
                    .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
                DocSection::new("Notes", notes).description("Behavior notes."),
            ],
        );

        vec![body.test_id("ui-gallery-slider")]
    })
}
