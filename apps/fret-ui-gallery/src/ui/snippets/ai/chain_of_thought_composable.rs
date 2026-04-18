pub const SOURCE: &str = include_str!("chain_of_thought_composable.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_core::FontWeight;
use fret_ui_ai as ui_ai;
use fret_ui_kit::{Items, LayoutRefinement, Space, ui};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    ui_ai::ChainOfThought::new()
        .default_open(true)
        .refine_layout(LayoutRefinement::default().w_full().min_w_0())
        .header(ui_ai::ChainOfThoughtHeader::new().children([
                    ui::h_row(|cx| {
                        [
                            cx.text("Reasoning trace"),
                            shadcn::Badge::new("Docs-style children")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .label_weight(FontWeight::NORMAL)
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .items(Items::Center)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                ]))
        .content(ui_ai::ChainOfThoughtContent::new([
                ui_ai::ChainOfThoughtStep::new("Collect evidence")
                    .status(ui_ai::ChainOfThoughtStepStatus::Complete)
                    .label_children([
                        ui::h_row(|cx| {
                            [
                                cx.text("Collect evidence"),
                                shadcn::Badge::new("complete")
                                    .variant(shadcn::BadgeVariant::Secondary)
                                    .label_weight(FontWeight::NORMAL)
                                    .into_element(cx),
                            ]
                        })
                        .gap(Space::N2)
                        .items(Items::Center)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                    ])
                    .description_children([
                        cx.text(
                            "Header text, step labels, and descriptions can all be composed from full child elements.",
                        ),
                    ])
                    .children([ui_ai::ChainOfThoughtSearchResults::new([
                        ui_ai::ChainOfThoughtSearchResult::new("docs.ai-elements.dev").into_element(cx),
                        ui_ai::ChainOfThoughtSearchResult::new("shadcn-registry").into_element(cx),
                    ])
                    .into_element(cx)])
                    .into_element(cx),
                ui_ai::ChainOfThoughtStep::new("Summarize the result")
                    .status(ui_ai::ChainOfThoughtStepStatus::Active)
                    .description("Keep the public example copyable, then layer Fret-specific notes underneath.")
                    .into_element(cx),
            ]))
        .into_element(cx)
}
// endregion: example
