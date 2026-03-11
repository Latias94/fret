pub const SOURCE: &str = include_str!("model_selector_demo.rs");

// region: example
use fret_runtime::Model;
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

#[derive(Default)]
struct DemoModels {
    open: Option<Model<bool>>,
    query: Option<Model<String>>,
    selected: Option<Model<Arc<str>>>,
}

pub fn render<H: UiHost + 'static>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let needs_init = cx.with_state(DemoModels::default, |st| {
        st.open.is_none() || st.query.is_none() || st.selected.is_none()
    });
    if needs_init {
        let open = cx.app.models_mut().insert(false);
        let query = cx.app.models_mut().insert(String::new());
        let selected = cx
            .app
            .models_mut()
            .insert(Arc::<str>::from("openai-gpt-4o"));
        cx.with_state(DemoModels::default, move |st| {
            st.open = Some(open.clone());
            st.query = Some(query.clone());
            st.selected = Some(selected.clone());
        });
    }

    let (open, query, selected) = cx.with_state(DemoModels::default, |st| {
        (
            st.open.clone().expect("open"),
            st.query.clone().expect("query"),
            st.selected.clone().expect("selected"),
        )
    });

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

    let model_selector = ui_ai::ModelSelector::new()
        .open_model(open.clone())
        .into_element_with_children(cx, move |cx, open| {
            let logo =
                selected_model.map(|m| ui_ai::ModelSelectorLogo::new(m.chef_slug).into_element(cx));
            let name =
                selected_model.map(|m| ui_ai::ModelSelectorName::new(m.name).into_element(cx));
            let mut trigger_content: Vec<AnyElement> = Vec::new();
            if let Some(logo) = logo {
                trigger_content.push(logo);
            }
            if let Some(name) = name {
                trigger_content.push(name);
            }

            let trigger = ui_ai::ModelSelectorTrigger::new(
                open.clone(),
                shadcn::Button::new("Select model")
                    .variant(shadcn::ButtonVariant::Outline)
                    .content_justify(Justify::Between)
                    .refine_layout(LayoutRefinement::default().w_px(fret_core::Px(200.0)))
                    .children(trigger_content)
                    .into_element(cx),
            )
            .test_id("ui-ai-model-selector-trigger")
            .into_element(cx);

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

            let make_item = |cx: &mut ElementContext<'_, H>, model: DemoModel| {
                let is_selected = model.id == selected_now.as_ref();
                let logos = ui_ai::ModelSelectorLogoGroup::new(
                    model
                        .providers
                        .iter()
                        .map(|p| ui_ai::ModelSelectorLogo::new(*p).into_element(cx)),
                )
                .into_element(cx);

                let name = ui_ai::ModelSelectorName::new(model.name).into_element(cx);

                let check = icon::icon_with(
                    cx,
                    fret_icons::ids::ui::CHECK,
                    Some(fret_core::Px(16.0)),
                    None,
                );
                let check = cx.opacity(if is_selected { 1.0 } else { 0.0 }, move |_cx| vec![check]);

                let mut item = ui_ai::ModelSelectorItem::new(model.name)
                    .value(model.id)
                    .children([logos, name, check])
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
                groups.push(ui_ai::ModelSelectorGroup::new(items).heading(chef).into());
            }

            let content = ui_ai::ModelSelectorContent::new([
                ui_ai::ModelSelectorInput::new(query.clone())
                    .placeholder("Search models...")
                    .input_test_id("ui-ai-model-selector-input")
                    .into_element(cx),
                ui_ai::ModelSelectorList::new_entries(groups)
                    .empty_text("No models found.")
                    .query_model(query.clone())
                    .into_element(cx),
            ])
            .test_id_root("ui-ai-model-selector-content")
            .into_element(cx);

            (trigger, content)
        });

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
