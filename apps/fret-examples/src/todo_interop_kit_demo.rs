use std::collections::HashSet;
use std::sync::Arc;

use fret_core::ViewportFit;
use fret_kit::interop::embedded_viewport as embedded;
use fret_kit::prelude::*;
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};

const CMD_ADD: &str = "todo-interop-kit.add";
const CMD_CLEAR_DONE: &str = "todo-interop-kit.clear_done";

#[derive(Debug, Clone)]
enum Msg {
    Remove(u64),
}

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoInteropKitState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    router: MessageRouter<Msg>,
    next_id: u64,

    embedded: embedded::EmbeddedViewportSurface,
}

impl embedded::EmbeddedViewportRecord for TodoInteropKitState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("todo-interop-kit viewport")
    }

    fn record_embedded_viewport(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        _context: &WgpuContext,
        _renderer: &mut Renderer,
        _scale_factor: f32,
        _tick_id: TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let models = embedded::ensure_models(app, window);
        let clicks = app.models().get_copied(&models.clicks).unwrap_or(0);

        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let f = ((clicks % 8) as f32) / 8.0;
        let clear = wgpu::Color {
            r: (0.08 + 0.30 * t) as f64,
            g: (0.08 + 0.25 * (1.0 - t)) as f64,
            b: (0.10 + 0.35 * f) as f64,
            a: 1.0,
        };

        embedded::clear_pass(encoder, view, Some("todo-interop-kit clear"), clear);
    }
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("todo-interop-kit-demo", init_window, view, |d| {
        d.on_command(on_command).drive_embedded_viewport()
    })?
    .with_main_window("todo_interop_kit_demo", (980.0, 640.0))
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

fn init_window(app: &mut App, window: AppWindowId) -> TodoInteropKitState {
    embedded::ensure_models(app, window);

    let done_1 = app.models_mut().insert(false);
    let done_2 = app.models_mut().insert(true);

    let todos = app.models_mut().insert(vec![
        TodoItem {
            id: 1,
            done: done_1,
            text: Arc::from("Add an item"),
        },
        TodoItem {
            id: 2,
            done: done_2,
            text: Arc::from("Click the embedded viewport"),
        },
    ]);

    let draft = app.models_mut().insert(String::new());

    let prefix = format!("todo-interop-kit-demo.{window:?}.");
    TodoInteropKitState {
        todos,
        draft,
        router: MessageRouter::new(prefix),
        next_id: 3,
        embedded: embedded::EmbeddedViewportSurface::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
            (640, 360),
        ),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoInteropKitState) -> fret_kit::ViewElements {
    let Some(models) = embedded::models(&*cx.app, cx.window) else {
        return vec![cx.text("Embedded viewport models are not installed.")].into();
    };
    st.router.clear();

    cx.watch_model(&st.draft).layout().observe();

    let theme = Theme::global(&*cx.app).clone();

    let todos = cx
        .watch_model(&st.todos)
        .layout()
        .cloned()
        .unwrap_or_default();

    let clicks = cx.watch_model(&models.clicks).paint().copied().unwrap_or(0);
    let last_input = cx
        .watch_model(&models.last_input)
        .paint()
        .cloned()
        .unwrap_or_else(|| Arc::<str>::from("<error>"));
    let target = cx
        .watch_model(&models.target)
        .paint()
        .copied()
        .unwrap_or_default();

    let draft = st.draft.clone();
    let root = ui::container(cx, |cx| {
        let left = todo_panel(cx, &theme, draft.clone(), &todos, &mut st.router);
        let right = external_panel(cx, &st.embedded, target, clicks, last_input);

        [ui::h_flex(cx, move |_cx| [left, right])
            .gap(Space::N4)
            .p(Space::N4)
            .w_full()
            .h_full()
            .items_stretch()
            .into_element(cx)]
    })
    .bg(ColorRef::Color(theme.color_required("background")))
    .w_full()
    .h_full()
    .into_element(cx);

    vec![root].into()
}

fn todo_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    draft: Model<String>,
    todos: &[TodoItem],
    router: &mut MessageRouter<Msg>,
) -> AnyElement {
    let header = shadcn::CardHeader::new([shadcn::CardTitle::new("Todo").into_element(cx)]);

    let draft_input = shadcn::Input::new(draft)
        .a11y_label("Todo")
        .placeholder("New task...")
        .submit_command(CommandId::new(CMD_ADD))
        .into_element(cx);

    let add = shadcn::Button::new("Add")
        .on_click(CMD_ADD)
        .into_element(cx);
    let clear_done = shadcn::Button::new("Clear done")
        .variant(shadcn::ButtonVariant::Outline)
        .on_click(CMD_CLEAR_DONE)
        .into_element(cx);

    let rows: Vec<(TodoItem, CommandId)> = todos
        .iter()
        .cloned()
        .map(|t| {
            let remove_cmd = router.cmd(Msg::Remove(t.id));
            (t, remove_cmd)
        })
        .collect();

    let list = cx.column(fret_ui::element::ColumnProps::default(), |cx| {
        rows.iter()
            .map(|(t, remove_cmd)| cx.keyed(t.id, |cx| todo_row(cx, theme, t, remove_cmd.clone())))
            .collect::<Vec<_>>()
    });

    let body = shadcn::CardContent::new([
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| [draft_input, add, clear_done],
        ),
        list,
    ]);

    shadcn::Card::new([header.into_element(cx), body.into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(360.0)).h_full())
        .into_element(cx)
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    item: &TodoItem,
    remove_cmd: CommandId,
) -> AnyElement {
    let checkbox = shadcn::Checkbox::new(item.done.clone())
        .a11y_label("Done")
        .into_element(cx);
    let text = cx.text(item.text.clone());
    let remove = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_click(remove_cmd)
        .into_element(cx);

    let row = ui::h_flex(cx, move |_cx| [checkbox, text, remove])
        .gap(Space::N2)
        .items_center()
        .w_full()
        .into_element(cx);

    ui::container(cx, move |_cx| [row])
        .border_1()
        .border_color(ColorRef::Color(theme.color_required("border")))
        .rounded(Radius::Md)
        .p(Space::N2)
        .w_full()
        .into_element(cx)
}

fn external_panel(
    cx: &mut ElementContext<'_, App>,
    surface: &embedded::EmbeddedViewportSurface,
    target: fret_core::RenderTargetId,
    clicks: u32,
    last_input: Arc<str>,
) -> AnyElement {
    let target_px_size = surface.target_px_size();
    let title = shadcn::CardTitle::new("External viewport (stub)").into_element(cx);
    let desc = shadcn::CardDescription::new(format!(
        "Clicks: {clicks} | Target: {target:?} | {}x{}",
        target_px_size.0, target_px_size.1
    ))
    .into_element(cx);
    let last = shadcn::Badge::new(last_input)
        .variant(shadcn::BadgeVariant::Secondary)
        .into_element(cx);

    let header = shadcn::CardHeader::new([title, desc, last]).into_element(cx);

    let viewport = surface.panel(
        cx,
        embedded::EmbeddedViewportPanelProps {
            fit: ViewportFit::Contain,
            opacity: 1.0,
            forward_input: true,
        },
    );

    let body = shadcn::CardContent::new([viewport]).into_element(cx);

    shadcn::Card::new([header, body])
        .refine_layout(LayoutRefinement::default().w_full().h_full())
        .into_element(cx)
}

fn on_command(
    app: &mut App,
    _services: &mut dyn UiServices,
    _window: AppWindowId,
    _ui: &mut UiTree<App>,
    st: &mut TodoInteropKitState,
    command: &CommandId,
) {
    match command.as_str() {
        CMD_ADD => {
            let draft = app
                .models()
                .read(&st.draft, |s| s.trim().to_string())
                .unwrap_or_default();
            if draft.is_empty() {
                return;
            }

            let id = st.next_id;
            st.next_id += 1;
            let done = app.models_mut().insert(false);
            let item = TodoItem {
                id,
                done,
                text: Arc::from(draft),
            };
            let _ = app.models_mut().update(&st.todos, |v| v.push(item));
            let _ = app.models_mut().update(&st.draft, |s| s.clear());
        }
        CMD_CLEAR_DONE => {
            let done_ids: HashSet<u64> = app
                .models()
                .read(&st.todos, |items| {
                    items
                        .iter()
                        .filter_map(|t| {
                            app.models()
                                .read(&t.done, |b| *b)
                                .ok()
                                .and_then(|done| done.then_some(t.id))
                        })
                        .collect()
                })
                .unwrap_or_default();
            let _ = app
                .models_mut()
                .update(&st.todos, |v| v.retain(|t| !done_ids.contains(&t.id)));
        }
        _ => {
            let Some(msg) = st.router.try_take(command) else {
                return;
            };
            match msg {
                Msg::Remove(id) => {
                    let _ = app
                        .models_mut()
                        .update(&st.todos, |v| v.retain(|t| t.id != id));
                }
            }
        }
    }
}
