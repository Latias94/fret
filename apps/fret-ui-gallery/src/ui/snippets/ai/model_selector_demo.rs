pub const SOURCE: &str = include_str!("model_selector_demo.rs");

// region: example
use fret::{UiChild, UiCx};
use fret_ui::Invalidation;
use fret_ui::element::SemanticsProps;
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Clone, Copy)]
struct DemoModel {
    chef: &'static str,
    chef_slug: &'static str,
    id: &'static str,
    name: &'static str,
    providers: &'static [&'static str],
}

const MODELS: &[DemoModel] = &[
    DemoModel {
        chef: "OpenAI",
        chef_slug: "openai",
        id: "openai-gpt-4o",
        name: "GPT-4o",
        providers: &["openai", "azure"],
    },
    DemoModel {
        chef: "OpenAI",
        chef_slug: "openai",
        id: "openai-gpt-4o-mini",
        name: "GPT-4o Mini",
        providers: &["openai", "azure"],
    },
    DemoModel {
        chef: "Anthropic",
        chef_slug: "anthropic",
        id: "anthropic-claude-3-5-sonnet",
        name: "Claude 3.5 Sonnet",
        providers: &["anthropic", "azure", "google-vertex", "amazon-bedrock"],
    },
    DemoModel {
        chef: "Anthropic",
        chef_slug: "anthropic",
        id: "anthropic-claude-opus-4-20250514",
        name: "Claude 4 Opus",
        providers: &["anthropic", "azure", "google-vertex", "amazon-bedrock"],
    },
    DemoModel {
        chef: "Google",
        chef_slug: "google",
        id: "google-gemini-2-0-flash",
        name: "Gemini 2.0 Flash",
        providers: &["google", "google-vertex"],
    },
];

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let open = cx.local_model_keyed("open", || false);
    let query = cx.local_model_keyed("query", String::new);
    let selected = cx.local_model_keyed("selected", || Arc::<str>::from("openai-gpt-4o"));

    let selected_now = cx
        .get_model_cloned(&selected, Invalidation::Paint)
        .unwrap_or_else(|| Arc::<str>::from("openai-gpt-4o"));

    let selected_marker = {
        let test_id = Arc::<str>::from(format!(
            "ui-ai-model-selector-selected-{}",
            selected_now.as_ref()
        ));
        cx.semantics(
            SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                test_id: Some(test_id),
                ..Default::default()
            },
            |cx| vec![cx.text("")],
        )
    };

    let selected_model = MODELS
        .iter()
        .find(|model| model.id == selected_now.as_ref())
        .copied();

    let trigger_content = {
        let mut children: Vec<AnyElement> = Vec::new();
        if let Some(model) = selected_model {
            children.push(ui_ai::ModelSelectorLogo::new(model.chef_slug).into_element(cx));
            children.push(ui_ai::ModelSelectorName::new(model.name).into_element(cx));
        }
        children
    };

    let on_select = |value: &'static str| -> fret_ui::action::OnActivate {
        let selected = selected.clone();
        let query = query.clone();
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host
                .models_mut()
                .update(&selected, |v| *v = Arc::<str>::from(value));
            let _ = host.models_mut().update(&query, |v| v.clear());
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        })
    };

    let make_item = |cx: &mut UiCx<'_>, model: DemoModel| {
        let is_selected = model.id == selected_now.as_ref();

        let check = icon::icon_with(
            cx,
            fret_icons::ids::ui::CHECK,
            Some(fret_core::Px(16.0)),
            None,
        );
        let check = cx.opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| vec![check]);
        let check = ui::h_row(move |_cx| vec![check])
            .layout(
                LayoutRefinement::default()
                    .ml_auto()
                    .w_px(fret_core::Px(16.0))
                    .h_px(fret_core::Px(16.0)),
            )
            .justify_center()
            .items_center()
            .into_element(cx);

        let mut item = ui_ai::ModelSelectorItem::new(model.name)
            .value(model.id)
            .child(ui_ai::ModelSelectorLogo::new(model.chef_slug))
            .child(ui_ai::ModelSelectorName::new(model.name))
            .child(ui_ai::ModelSelectorLogoGroup::new(
                model
                    .providers
                    .iter()
                    .copied()
                    .map(ui_ai::ModelSelectorLogo::new),
            ))
            .child(check)
            .on_select_action(on_select(model.id));

        match model.id {
            "openai-gpt-4o-mini" => {
                item = item.test_id("ui-ai-model-selector-item-openai-gpt-4o-mini")
            }
            "anthropic-claude-3-5-sonnet" => {
                item = item.test_id("ui-ai-model-selector-item-anthropic-claude-3-5-sonnet")
            }
            "google-gemini-2-0-flash" => {
                item = item.test_id("ui-ai-model-selector-item-google-gemini-2-0-flash")
            }
            _ => {}
        }

        item
    };

    let mut chefs: Vec<&'static str> = Vec::new();
    for model in MODELS {
        if !chefs.contains(&model.chef) {
            chefs.push(model.chef);
        }
    }

    let mut groups = Vec::new();
    for chef in chefs {
        let mut items = Vec::new();
        for model in MODELS.iter().copied().filter(|m| m.chef == chef) {
            items.push(make_item(cx, model));
        }
        groups.push(ui_ai::ModelSelectorGroup::new(items).heading(chef));
    }

    let model_selector = ui_ai::ModelSelector::new()
        .open_model(open.clone())
        .children([
            ui_ai::ModelSelectorChild::Trigger(
                ui_ai::ModelSelectorTrigger::new(
                    shadcn::Button::new("Select model")
                        .variant(shadcn::ButtonVariant::Outline)
                        .content_justify(Justify::Between)
                        .refine_layout(LayoutRefinement::default().w_px(fret_core::Px(200.0)))
                        .children(trigger_content)
                        .into_element(cx),
                )
                .test_id("ui-ai-model-selector-trigger"),
            ),
            ui_ai::ModelSelectorChild::Content(
                ui_ai::ModelSelectorContent::new([])
                    .input(
                        ui_ai::ModelSelectorInput::new(query.clone())
                            .placeholder("Search models...")
                            .input_test_id("ui-ai-model-selector-input"),
                    )
                    .list(
                        ui_ai::ModelSelectorList::new_entries(groups)
                            .empty_text("No models found.")
                            .query_model(query.clone()),
                    )
                    .test_id_root("ui-ai-model-selector-content"),
            ),
        ])
        .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            cx.text("ModelSelector (AI Elements)"),
            cx.text("Dialog + Command surfaces; selection is app-owned."),
            ui::h_flex(move |_cx| vec![model_selector])
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .justify_center()
                .items_center()
                .into_element(cx),
            selected_marker,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
