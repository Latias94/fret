pub const SOURCE: &str = include_str!("task_demo.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_core::Px;
use fret_ui::Theme;
use fret_ui_ai as ui_ai;
use fret_ui_kit::declarative::{icon, style as decl_style};
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space};
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let preset = cx.local_model_keyed("preset", || 0_u8);
    let preset_value = cx.app.models().read(&preset, |v| *v).unwrap_or(0);

    let react_preset = preset.clone();
    let react_dev = shadcn::Button::new("React Development")
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host.models_mut().update(&react_preset, |v| *v = 0);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }))
        .into_element(cx);

    let api_preset = preset.clone();
    let api_integration = shadcn::Button::new("API Integration")
        .variant(shadcn::ButtonVariant::Outline)
        .on_activate(cx.actions().listen(move |host, action_cx| {
            let _ = host.models_mut().update(&api_preset, |v| *v = 1);
            host.notify(action_cx);
            host.request_redraw(action_cx.window);
        }))
        .into_element(cx);

    let tasks: Vec<(
        &'static str,
        Vec<(&'static str, Option<(&'static str, &'static str)>)>,
    )> = match preset_value {
        1 => vec![
            (
                "Integrate streaming API",
                vec![
                    (
                        "Define request schema",
                        Some(("schema.ts", "lucide.file-json")),
                    ),
                    ("Implement streaming transport", None),
                    ("Add retry + backoff policy", None),
                    ("Verify cancellation behavior", None),
                ],
            ),
            (
                "Ship UI conformance gate",
                vec![
                    ("Add stable test_id anchors", None),
                    ("Author a diag script (toggle + bundle)", None),
                    ("Review focus + dismiss outcomes", None),
                    ("Document the expected invariant", None),
                ],
            ),
        ],
        _ => vec![
            (
                "Indexing workspace",
                vec![
                    ("Scan project files", Some(("Cargo.toml", "lucide.file"))),
                    ("Parse Rust modules", None),
                    ("Build symbol index", None),
                    ("Emit diagnostics snapshot", None),
                ],
            ),
            (
                "Generate UI recipes",
                vec![
                    ("Extract layout constraints", None),
                    ("Map tokens to theme refs", None),
                    ("Add copyable code snippet", None),
                ],
            ),
            (
                "Verify interaction parity",
                vec![
                    ("Hover intent + outside press", None),
                    ("Keyboard traversal", None),
                    ("A11y role/flags sanity check", None),
                ],
            ),
        ],
    };

    let task_list = ui::v_flex(move |cx| {
        tasks
            .into_iter()
            .enumerate()
            .map(|(task_index, (title, items))| {
                let trigger = if task_index == 0 {
                    ui_ai::TaskTrigger::new(title).test_id("ui-ai-task-demo-trigger")
                } else {
                    ui_ai::TaskTrigger::new(title)
                };

                let content_children = items.into_iter().map(|(text, file)| {
                    if let Some((file_name, icon_name)) = file {
                        ui_ai::TaskItem::new([
                            cx.text(text),
                            ui_ai::TaskItemFile::new([
                                icon::icon_with(
                                    cx,
                                    fret_icons::IconId::new_static(icon_name),
                                    Some(Px(16.0)),
                                    None,
                                ),
                                cx.text(file_name),
                            ])
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    } else {
                        ui_ai::TaskItem::new([cx.text(text)]).into_element(cx)
                    }
                });

                let content = if task_index == 0 {
                    ui_ai::TaskContent::new(content_children).test_id("ui-ai-task-demo-content")
                } else {
                    ui_ai::TaskContent::new(content_children)
                };

                ui_ai::Task::root()
                    .default_open(task_index == 0)
                    .children([
                        ui_ai::TaskChild::Trigger(trigger),
                        ui_ai::TaskChild::Content(content),
                    ])
                    .into_element(cx)
            })
            .collect::<Vec<_>>()
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx);

    let theme = Theme::global(&*cx.app);
    let chrome = ChromeRefinement::default()
        .p(Space::N6)
        .rounded(Radius::Lg)
        .border_1()
        .bg(ColorRef::Token {
            key: "background",
            fallback: ColorFallback::ThemeSurfaceBackground,
        });
    let layout = LayoutRefinement::default()
        .w_full()
        .min_w_0()
        .max_w(Px(896.0))
        .h_px(Px(600.0));
    let mut props = decl_style::container_props(theme, chrome, layout);
    props.border_color = Some(theme.color_token("border"));
    props.background = Some(theme.color_token("background"));

    let framed = cx.container(props, move |_cx| vec![task_list]);

    ui::v_flex(move |cx| {
        vec![
            cx.text("Task (AI Elements)"),
            cx.text(
                "Collapsible task list demo aligned with the official AI Elements Task structure.",
            ),
            ui::h_row(move |_cx| vec![react_dev, api_integration])
                .gap(Space::N2)
                .items_center()
                .into_element(cx),
            framed,
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N4)
    .into_element(cx)
}
// endregion: example
