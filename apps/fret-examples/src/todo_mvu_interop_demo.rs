use std::collections::HashSet;
use std::sync::Arc;

use fret_core::ViewportFit;
use fret_kit::interop::embedded_viewport as embedded;
use fret_kit::prelude::*;
use fret_render::RenderTargetColorSpace;
use fret_runtime::{FrameId, TickId};

#[derive(Clone)]
struct TodoItem {
    id: u64,
    done: Model<bool>,
    text: Arc<str>,
}

struct TodoMvuInteropState {
    todos: Model<Vec<TodoItem>>,
    draft: Model<String>,
    next_id: u64,

    embedded: embedded::EmbeddedViewportSurface,
}

#[derive(Debug, Clone)]
enum Msg {
    Add,
    Remove(u64),
    ClearDone,
}

impl embedded::EmbeddedViewportRecord for TodoMvuInteropState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("todo-mvu-interop viewport")
    }

    fn record_embedded_viewport(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        _context: &fret_render::WgpuContext,
        _renderer: &mut fret_render::Renderer,
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

        embedded::clear_pass(encoder, view, Some("todo-mvu-interop clear"), clear);
    }
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::mvu::app_with_hooks::<TodoMvuInteropProgram>("todo-mvu-interop-demo", |d| {
        d.drive_embedded_viewport()
    })?
    .with_main_window("todo_mvu_interop_demo", (980.0, 640.0))
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

struct TodoMvuInteropProgram;

impl MvuProgram for TodoMvuInteropProgram {
    type State = TodoMvuInteropState;
    type Message = Msg;

    fn init(app: &mut App, window: AppWindowId) -> Self::State {
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

        Self::State {
            todos,
            draft,
            next_id: 3,
            embedded: embedded::EmbeddedViewportSurface::new(
                wgpu::TextureFormat::Bgra8UnormSrgb,
                RenderTargetColorSpace::Srgb,
                (640, 360),
            ),
        }
    }

    fn update(app: &mut App, st: &mut Self::State, message: Self::Message) {
        match message {
            Msg::Add => {
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
            Msg::Remove(id) => {
                let _ = app
                    .models_mut()
                    .update(&st.todos, |v| v.retain(|t| t.id != id));
            }
            Msg::ClearDone => {
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
        }
    }

    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut Self::State,
        msg: &mut MessageRouter<Self::Message>,
    ) -> Vec<AnyElement> {
        cx.observe_model(&st.todos, Invalidation::Layout);
        cx.observe_model(&st.draft, Invalidation::Layout);

        let Some(models) = embedded::models(&*cx.app, cx.window) else {
            return vec![cx.text("Embedded viewport models are not installed.")];
        };
        cx.observe_model(&models.clicks, Invalidation::Paint);
        cx.observe_model(&models.last_input, Invalidation::Paint);
        cx.observe_model(&models.target, Invalidation::Paint);

        let theme = Theme::global(&*cx.app).clone();

        let todos = cx
            .app
            .models()
            .read(&st.todos, |v| v.clone())
            .unwrap_or_default();
        for t in &todos {
            cx.observe_model(&t.done, Invalidation::Layout);
        }

        let clicks = cx.app.models().read(&models.clicks, |v| *v).unwrap_or(0);
        let last_input = cx
            .app
            .models()
            .read(&models.last_input, |v| v.clone())
            .unwrap_or_else(|_| Arc::<str>::from("<error>"));
        let target = cx
            .app
            .models()
            .read(&models.target, |v| *v)
            .unwrap_or_default();

        let mut root_layout = fret_ui::element::LayoutStyle::default();
        root_layout.size.width = fret_ui::element::Length::Fill;
        root_layout.size.height = fret_ui::element::Length::Fill;

        let root = cx.container(
            fret_ui::element::ContainerProps {
                layout: root_layout,
                background: Some(theme.color_required("background")),
                ..Default::default()
            },
            |cx| {
                let flex = fret_ui::element::FlexProps {
                    layout: root_layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(16.0),
                    padding: fret_core::Edges::all(Px(16.0)),
                    ..Default::default()
                };

                let left = todo_panel(cx, &theme, st, msg, &todos);
                let right = external_panel(cx, &st.embedded, target, clicks, last_input);

                [cx.flex(flex, move |_cx| [left, right])]
            },
        );

        vec![root]
    }
}

fn todo_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    st: &TodoMvuInteropState,
    msg: &mut MessageRouter<Msg>,
    todos: &[TodoItem],
) -> AnyElement {
    let header = shadcn::CardHeader::new([shadcn::CardTitle::new("Todo").into_element(cx)]);

    let draft = shadcn::Input::new(st.draft.clone())
        .a11y_label("Todo")
        .placeholder("New task...")
        .submit_command(msg.cmd(Msg::Add))
        .into_element(cx);

    let add = shadcn::Button::new("Add")
        .on_click(msg.cmd(Msg::Add))
        .into_element(cx);
    let clear_done = shadcn::Button::new("Clear done")
        .variant(shadcn::ButtonVariant::Outline)
        .on_click(msg.cmd(Msg::ClearDone))
        .into_element(cx);

    let list = todos.iter().map(|t| todo_row(cx, theme, msg, t)).elements();

    let body = shadcn::CardContent::new([
        stack::hstack_iter(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| [draft, add, clear_done],
        ),
        cx.column(fret_ui::element::ColumnProps::default(), move |_cx| list),
    ]);

    shadcn::Card::new([header.into_element(cx), body.into_element(cx)])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(360.0)))
                .h_full(),
        )
        .into_element(cx)
}

fn todo_row(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    msg: &mut MessageRouter<Msg>,
    item: &TodoItem,
) -> AnyElement {
    let checkbox = shadcn::Checkbox::new(item.done.clone())
        .a11y_label("Done")
        .into_element(cx);
    let text = cx.text(item.text.clone());
    let remove = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_click(msg.cmd(Msg::Remove(item.id)))
        .into_element(cx);

    let row = stack::hstack_iter(
        cx,
        stack::HStackProps::default()
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full()),
        move |_cx| [checkbox, text, remove],
    );

    let props = shadcn::decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Md)
            .p(Space::N2),
        LayoutRefinement::default().w_full(),
    );
    cx.container(props, move |_cx| [row])
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
