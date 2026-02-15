use std::sync::Arc;

use fret::prelude::*;
use fret_genui_core::catalog::CatalogV1;
use fret_genui_core::render::{GenUiActionQueue, GenUiRuntime, render_spec};
use fret_genui_core::spec::SpecV1;
use fret_genui_core::validate::ValidationMode;
use fret_genui_shadcn::catalog::shadcn_catalog_v1;
use fret_genui_shadcn::resolver::ShadcnResolver;
use serde_json::Value;

pub fn run() -> anyhow::Result<()> {
    fret::mvu::app::<GenUiProgram>("genui-demo")?
        .with_main_window("genui_demo", (980.0, 720.0))
        .init_app(|app| {
            shadcn::shadcn_themes::apply_shadcn_new_york_v4(
                app,
                shadcn::shadcn_themes::ShadcnBaseColor::Slate,
                shadcn::shadcn_themes::ShadcnColorScheme::Light,
            );
        })
        .run()?;
    Ok(())
}

const SPEC_JSON: &str = r#"
{
  "schema_version": 1,
  "root": "root",
  "elements": {
    "root": {
      "type": "VStack",
      "props": { "gap": "N3" },
      "children": ["header", "card"]
    },
    "header": { "type": "Text", "props": { "text": "GenUI Demo (json-render-inspired)" }, "children": [] },
    "card": {
      "type": "Card",
      "props": { "wrapContent": false },
      "children": ["card_header", "card_content"]
    },
    "card_header": {
      "type": "CardHeader",
      "props": {},
      "children": ["card_title", "card_desc"]
    },
    "card_title": { "type": "CardTitle", "props": { "text": "Spec-driven UI" }, "children": [] },
    "card_desc": { "type": "CardDescription", "props": { "text": "Bindings, repeat, and standard actions." }, "children": [] },
    "card_content": {
      "type": "CardContent",
      "props": {},
      "children": ["card_stack"]
    },
    "card_stack": {
      "type": "VStack",
      "props": { "gap": "N2" },
      "children": [
        "bind_title",
        "enabled_row",
        "name_row",
        "name_buttons",
        "sep_1",
        "counter_title",
        "counter_row",
        "sep_2",
        "todos_title",
        "todo_add_row",
        "todos_list",
        "sep_3",
        "responsive_title",
        "responsive_grid",
        "responsive_stack_title",
        "responsive_stack"
      ]
    },
    "bind_title": { "type": "Text", "props": { "text": "Bindings ($bindState demo)" }, "children": [] },
    "enabled_row": {
      "type": "HStack",
      "props": { "gap": "N2" },
      "children": ["enabled_label", "enabled_switch", "enabled_badge", "disabled_badge"]
    },
    "enabled_label": { "type": "Text", "props": { "text": "Enabled:" }, "children": [] },
    "enabled_switch": {
      "type": "Switch",
      "props": { "checked": { "$bindState": "/enabled" } },
      "children": []
    },
    "enabled_badge": {
      "type": "Badge",
      "props": { "label": "Enabled", "variant": "secondary" },
      "visible": { "$state": "/enabled" },
      "children": []
    },
    "disabled_badge": {
      "type": "Badge",
      "props": { "label": "Disabled", "variant": "outline" },
      "visible": { "$state": "/enabled", "not": true },
      "children": []
    },
    "name_row": {
      "type": "HStack",
      "props": { "gap": "N2" },
      "children": ["name_label", "name_input", "name_value"]
    },
    "name_label": { "type": "Text", "props": { "text": "Name:" }, "children": [] },
    "name_input": {
      "type": "Input",
      "props": { "placeholder": "Type your name…", "value": { "$bindState": "/name" } },
      "children": []
    },
    "name_value": { "type": "Text", "props": { "text": { "$state": "/name" } }, "children": [] },
    "name_buttons": {
      "type": "HStack",
      "props": { "gap": "N2" },
      "children": ["set_name_grace", "clear_name"]
    },
    "set_name_grace": {
      "type": "Button",
      "props": { "label": "Set name = Grace" },
      "on": { "press": { "action": "setState", "params": { "statePath": "/name", "value": "Grace" } } },
      "children": []
    },
    "clear_name": {
      "type": "Button",
      "props": { "label": "Clear name" },
      "on": { "press": { "action": "setState", "params": { "statePath": "/name", "value": "" } } },
      "children": []
    },
    "sep_1": { "type": "Separator", "props": {}, "children": [] },
    "counter_title": { "type": "Text", "props": { "text": "Counter (standard actions)" }, "children": [] },
    "counter_row": {
      "type": "HStack",
      "props": { "gap": "N2" },
      "children": ["counter_label", "counter_value", "counter_dec", "counter_inc"]
    },
    "counter_label": { "type": "Text", "props": { "text": "Count:" }, "children": [] },
    "counter_value": {
      "type": "Badge",
      "props": { "label": { "$state": "/count" }, "variant": "secondary" },
      "children": []
    },
    "counter_dec": {
      "type": "Button",
      "props": { "label": "Decrement" },
      "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": -1 } } },
      "children": []
    },
    "counter_inc": {
      "type": "Button",
      "props": { "label": "Increment" },
      "on": { "press": { "action": "incrementState", "params": { "statePath": "/count", "delta": 1 } } },
      "children": []
    },
    "sep_2": { "type": "Separator", "props": {}, "children": [] },
    "todos_title": { "type": "Text", "props": { "text": "Todos (repeat demo)" }, "children": [] },
    "todo_add_row": {
      "type": "HStack",
      "props": { "gap": "N2" },
      "children": ["todo_input", "todo_add_btn"]
    },
    "todo_input": {
      "type": "Input",
      "props": { "placeholder": "New todo…", "value": { "$bindState": "/newTodoText" } },
      "children": []
    },
    "todo_add_btn": {
      "type": "Button",
      "props": { "label": "Add" },
      "on": { "press": { "action": "pushState", "params": { "statePath": "/todos", "value": { "id": "$id", "label": { "$state": "/newTodoText" } }, "clearStatePath": "/newTodoText" } } },
      "children": []
    },
    "todos_list": {
      "type": "VStack",
      "props": { "gap": "N1" },
      "repeat": { "statePath": "/todos", "key": "id" },
      "children": ["todo_item"]
    },
    "todo_item": { "type": "HStack", "props": { "gap": "N2" }, "children": ["todo_label", "todo_remove"] },
    "todo_label": { "type": "Text", "props": { "text": { "$item": "label" } }, "children": [] },
    "todo_remove": {
      "type": "Button",
      "props": { "label": "Remove" },
      "on": { "press": { "action": "removeState", "params": { "statePath": "/todos", "index": { "$index": true } } } },
      "children": []
    },
    "sep_3": { "type": "Separator", "props": {}, "children": [] },
    "responsive_title": { "type": "Text", "props": { "text": "ResponsiveGrid (container query demo)" }, "children": [] },
    "responsive_grid": {
      "type": "ResponsiveGrid",
      "props": {
        "gap": "N2",
        "query": "container",
        "fillLastRow": true,
        "columns": { "base": 1, "md": 2, "lg": 3 }
      },
      "children": [
        "rg_card_1",
        "rg_card_2",
        "rg_card_3",
        "rg_card_4",
        "rg_card_5",
        "rg_card_6"
      ]
    },
    "rg_card_1": { "type": "Card", "props": {}, "children": ["rg_card_1_text"] },
    "rg_card_1_text": { "type": "Text", "props": { "text": "Card 1" }, "children": [] },
    "rg_card_2": { "type": "Card", "props": {}, "children": ["rg_card_2_text"] },
    "rg_card_2_text": { "type": "Text", "props": { "text": "Card 2" }, "children": [] },
    "rg_card_3": { "type": "Card", "props": {}, "children": ["rg_card_3_text"] },
    "rg_card_3_text": { "type": "Text", "props": { "text": "Card 3" }, "children": [] },
    "rg_card_4": { "type": "Card", "props": {}, "children": ["rg_card_4_text"] },
    "rg_card_4_text": { "type": "Text", "props": { "text": "Card 4" }, "children": [] },
    "rg_card_5": { "type": "Card", "props": {}, "children": ["rg_card_5_text"] },
    "rg_card_5_text": { "type": "Text", "props": { "text": "Card 5" }, "children": [] },
    "rg_card_6": { "type": "Card", "props": {}, "children": ["rg_card_6_text"] },
    "rg_card_6_text": { "type": "Text", "props": { "text": "Card 6" }, "children": [] },

    "responsive_stack_title": { "type": "Text", "props": { "text": "ResponsiveStack (container query demo)" }, "children": [] },
    "responsive_stack": {
      "type": "ResponsiveStack",
      "props": {
        "gap": "N2",
        "query": "container",
        "direction": { "base": "vertical", "lg": "horizontal" }
      },
      "children": ["rs_card_1", "rs_card_2", "rs_card_3"]
    },
    "rs_card_1": { "type": "Card", "props": {}, "children": ["rs_card_1_text"] },
    "rs_card_1_text": { "type": "Text", "props": { "text": "Stack card A" }, "children": [] },
    "rs_card_2": { "type": "Card", "props": {}, "children": ["rs_card_2_text"] },
    "rs_card_2_text": { "type": "Text", "props": { "text": "Stack card B" }, "children": [] },
    "rs_card_3": { "type": "Card", "props": {}, "children": ["rs_card_3_text"] },
    "rs_card_3_text": { "type": "Text", "props": { "text": "Stack card C" }, "children": [] }
  },
  "state": {
    "name": "Ada",
    "enabled": true,
    "count": 0,
    "newTodoText": "",
    "todos": [
      { "id": "a", "label": "Keep runtime mechanism-only (ADR 0066)" },
      { "id": "b", "label": "Render from a flat spec + catalog" },
      { "id": "c", "label": "Repeat scopes with stable keys" }
    ]
  }
}
"#;

struct GenUiState {
    spec: SpecV1,
    catalog: Arc<CatalogV1>,
    genui_state: Model<Value>,
    action_queue: Model<GenUiActionQueue>,
}

#[derive(Debug, Clone)]
enum Msg {
    ClearActions,
    ResetState,
}

struct GenUiProgram;

impl MvuProgram for GenUiProgram {
    type State = GenUiState;
    type Message = Msg;

    fn init(app: &mut App, _window: AppWindowId) -> Self::State {
        let spec: SpecV1 = serde_json::from_str(SPEC_JSON).expect("SPEC_JSON must parse");
        let seed = spec.state.clone().unwrap_or(Value::Null);
        GenUiState {
            spec,
            catalog: Arc::new(shadcn_catalog_v1()),
            genui_state: app.models_mut().insert(seed),
            action_queue: app.models_mut().insert(GenUiActionQueue::default()),
        }
    }

    fn update(app: &mut App, state: &mut Self::State, message: Self::Message) {
        match message {
            Msg::ClearActions => {
                let _ = app
                    .models_mut()
                    .update(&state.action_queue, |q| q.invocations.clear());
            }
            Msg::ResetState => {
                let seed = state.spec.state.clone().unwrap_or(Value::Null);
                let _ = app.models_mut().update(&state.genui_state, |v| *v = seed);
                let _ = app
                    .models_mut()
                    .update(&state.action_queue, |q| q.invocations.clear());
            }
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        state: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Elements {
        view(cx, state, msg)
    }
}

fn view(
    cx: &mut ElementContext<'_, App>,
    st: &mut GenUiState,
    msg: &mut MessageRouter<Msg>,
) -> Elements {
    let theme = Theme::global(&*cx.app).snapshot();

    let clear_cmd = msg.cmd(Msg::ClearActions);
    let reset_cmd = msg.cmd(Msg::ResetState);

    let queue_snapshot: Vec<Arc<str>> = cx
        .watch_model(&st.action_queue)
        .layout()
        .read(|_host, q| {
            q.invocations
                .iter()
                .enumerate()
                .map(|(i, inv)| {
                    Arc::<str>::from(format!("{:02}  {}  params={}", i, inv.action, inv.params))
                })
                .collect::<Vec<_>>()
        })
        .ok()
        .unwrap_or_default();
    let queue_len = queue_snapshot.len();
    let queue_lines: Vec<AnyElement> = queue_snapshot
        .into_iter()
        .map(|line| ui::text(cx, line).text_sm().into_element(cx))
        .collect();

    let toolbar = ui::h_flex(cx, move |cx| {
        vec![
            shadcn::Badge::new("Actions: auto-apply standard actions only")
                .variant(shadcn::BadgeVariant::Secondary)
                .into_element(cx),
            shadcn::Button::new("Clear queue")
                .variant(shadcn::ButtonVariant::Secondary)
                .on_click(clear_cmd)
                .into_element(cx),
            shadcn::Button::new("Reset state")
                .variant(shadcn::ButtonVariant::Outline)
                .on_click(reset_cmd)
                .into_element(cx),
        ]
    })
    .gap(Space::N2)
    .items_center()
    .into_element(cx);

    let runtime = GenUiRuntime {
        state: st.genui_state.clone(),
        action_queue: Some(st.action_queue.clone()),
        auto_apply_standard_actions: true,
        limits: Default::default(),
        catalog: Some(st.catalog.clone()),
        catalog_validation: ValidationMode::Strict,
    };

    let mut resolver = ShadcnResolver::new();
    let rendered = render_spec(cx, &st.spec, &runtime, &mut resolver);

    let (spec_root, spec_issues) = match rendered {
        Ok(out) => (
            out.roots.into_iter().next(),
            out.issues
                .into_iter()
                .map(|i| {
                    let s = Arc::<str>::from(format!("{:?}: {}", i.code, i.message));
                    ui::text(cx, s).text_sm().into_element(cx)
                })
                .collect::<Vec<_>>(),
        ),
        Err(err) => {
            let err_el = shadcn::Alert::new([
                shadcn::AlertTitle::new("Render error").into_element(cx),
                shadcn::AlertDescription::new(Arc::<str>::from(err.to_string())).into_element(cx),
            ])
            .into_element(cx);
            (Some(err_el), Vec::new())
        }
    };

    let left = {
        let mut body: Vec<AnyElement> = Vec::new();
        body.push(toolbar);
        if !spec_issues.is_empty() {
            body.push(
                shadcn::Alert::new([
                    shadcn::AlertTitle::new("Spec issues").into_element(cx),
                    shadcn::AlertDescription::new("Fix the spec before rendering.")
                        .into_element(cx),
                ])
                .into_element(cx),
            );
            body.extend(spec_issues);
        }
        if let Some(root) = spec_root {
            body.push(root);
        }
        shadcn::Card::new([shadcn::CardContent::new(body).into_element(cx)])
            .ui()
            .w_full()
            .h_full()
            .min_w(Px(420.0))
            .into_element(cx)
    };

    let right = {
        let title = ui::text(cx, Arc::<str>::from(format!("Action queue ({queue_len})")))
            .text_sm()
            .font_medium()
            .into_element(cx);
        let scroll = shadcn::ScrollArea::new([ui::v_flex(cx, |_cx| queue_lines)
            .gap(Space::N1)
            .w_full()
            .items_start()
            .into_element(cx)])
        .ui()
        .w_full()
        .h_full()
        .into_element(cx);

        shadcn::Card::new([
            shadcn::CardHeader::new([title]).into_element(cx),
            shadcn::CardContent::new([scroll]).into_element(cx),
        ])
        .ui()
        .w_full()
        .h_full()
        .min_w(Px(360.0))
        .into_element(cx)
    };

    let page = ui::container(cx, move |cx| {
        [ui::h_flex(cx, move |_cx| vec![left, right])
            .gap(Space::N3)
            .w_full()
            .h_full()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_token("muted")))
    .p(Space::N4)
    .w_full()
    .h_full()
    .into_element(cx);

    vec![page].into()
}
