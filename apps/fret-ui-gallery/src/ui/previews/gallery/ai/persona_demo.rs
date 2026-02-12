use super::super::super::super::*;

pub(in crate::ui) fn preview_ai_persona_demo(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::Px;
    use fret_ui_kit::declarative::stack;
    use fret_ui_kit::declarative::style as decl_style;
    use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};

    let max_w = LayoutRefinement::default()
        .w_full()
        .max_w(MetricRef::Px(Px(760.0)))
        .min_w_0();

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            vec![
                ui_ai::Persona::new(ui_ai::PersonaState::Idle)
                    .variant(ui_ai::PersonaVariant::Obsidian)
                    .show_label(true)
                    .test_id("ui-ai-persona-demo-idle")
                    .into_element(cx),
                ui_ai::Persona::new(ui_ai::PersonaState::Listening)
                    .variant(ui_ai::PersonaVariant::Halo)
                    .show_label(true)
                    .into_element(cx),
                ui_ai::Persona::new(ui_ai::PersonaState::Thinking)
                    .variant(ui_ai::PersonaVariant::Glint)
                    .show_label(true)
                    .into_element(cx),
                ui_ai::Persona::new(ui_ai::PersonaState::Speaking)
                    .variant(ui_ai::PersonaVariant::Command)
                    .show_label(true)
                    .into_element(cx),
                ui_ai::Persona::new(ui_ai::PersonaState::Asleep)
                    .variant(ui_ai::PersonaVariant::Opal)
                    .show_label(true)
                    .into_element(cx),
            ]
        },
    );

    let shell_props = cx
        .with_theme(|theme| decl_style::container_props(theme, ChromeRefinement::default(), max_w));
    let shell = cx.container(shell_props, move |_cx| [row]);

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N4),
        move |cx| {
            vec![
                cx.text("Persona (AI Elements)"),
                cx.text("UI-only placeholder (upstream uses Rive webgl2 visuals)."),
                shell,
            ]
        },
    )]
}
