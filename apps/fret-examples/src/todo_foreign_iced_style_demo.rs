use std::sync::Arc;

use fret_core::{ViewportFit, ViewportInputEvent, ViewportInputKind};
use fret_kit::interop::embedded_viewport as embedded;
use fret_kit::prelude::*;
use fret_render::RenderTargetColorSpace;
use fret_runtime::FrameId;

#[derive(Debug, Clone)]
struct ForeignTodoItem {
    id: u64,
    done: bool,
    text: Arc<str>,
}

#[derive(Debug, Clone)]
struct ForeignTodoModels {
    items: Model<Vec<ForeignTodoItem>>,
    hint: Model<Arc<str>>,
}

struct ForeignTodoUi {
    models: ForeignTodoModels,
}

impl ForeignTodoUi {
    fn new(app: &mut App) -> (Self, ForeignTodoModels) {
        let items = app.models_mut().insert(vec![
            ForeignTodoItem {
                id: 1,
                done: false,
                text: Arc::from("Click inside the embedded viewport"),
            },
            ForeignTodoItem {
                id: 2,
                done: true,
                text: Arc::from("Top: toggle, bottom: add"),
            },
        ]);

        let hint: Model<Arc<str>> = app
            .models_mut()
            .insert(Arc::from("This list is owned by a foreign UI runtime."));

        let models = ForeignTodoModels { items, hint };
        (
            Self {
                models: models.clone(),
            },
            models,
        )
    }

    fn toggle_or_add(&mut self, app: &mut App, event: &ViewportInputEvent) {
        let is_click = matches!(event.kind, ViewportInputKind::PointerDown { .. });
        if !is_click {
            return;
        }

        let y = event.uv.1.clamp(0.0, 1.0);
        let add_zone = 0.80_f32;
        if y >= add_zone {
            let _ = app.models_mut().update(&self.models.items, |items| {
                let next_id = items.iter().map(|i| i.id).max().unwrap_or(0) + 1;
                let text: Arc<str> = Arc::from(format!("New item {next_id}"));
                items.push(ForeignTodoItem {
                    id: next_id,
                    done: false,
                    text,
                });
            });
        } else {
            let _ = app.models_mut().update(&self.models.items, |items| {
                if items.is_empty() {
                    return;
                }
                let denom = items.len().max(1) as f32;
                let idx = ((y / add_zone) * denom).floor() as usize;
                let idx = idx.min(items.len().saturating_sub(1));
                items[idx].done = !items[idx].done;
            });
        }

        let _ = app.models_mut().update(&self.models.hint, |v| {
            *v = Arc::from(format!(
                "foreign: clicks are routed by RenderTargetId; uv=({:.3},{:.3})",
                event.uv.0, event.uv.1
            ));
        });
    }
}

impl embedded::EmbeddedViewportForeignUi for ForeignTodoUi {
    fn on_viewport_input(&mut self, app: &mut App, event: &ViewportInputEvent) {
        self.toggle_or_add(app, event);
    }

    fn record_foreign_frame(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        _context: &fret_render::WgpuContext,
        _renderer: &mut fret_render::Renderer,
        _scale_factor: f32,
        _tick_id: fret_runtime::TickId,
        frame_id: FrameId,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let (total, done) = app
            .models()
            .read(&self.models.items, |items| {
                let total = items.len() as u32;
                let done = items.iter().filter(|i| i.done).count() as u32;
                (total, done)
            })
            .unwrap_or((0, 0));

        let done_ratio = if total == 0 {
            0.0
        } else {
            (done as f32) / (total as f32)
        };

        let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
        let clear = wgpu::Color {
            r: (0.06 + 0.28 * t) as f64,
            g: (0.08 + 0.55 * done_ratio) as f64,
            b: (0.10 + 0.22 * (1.0 - t)) as f64,
            a: 1.0,
        };

        embedded::clear_pass(encoder, view, Some("foreign-todo clear"), clear);
    }
}

struct TodoForeignIcedStyleState {
    embedded: embedded::EmbeddedViewportSurface,
    foreign: ForeignTodoModels,
}

impl embedded::EmbeddedViewportSurfaceOwner for TodoForeignIcedStyleState {
    fn embedded_viewport_surface(&mut self) -> &mut embedded::EmbeddedViewportSurface {
        &mut self.embedded
    }

    fn embedded_viewport_label(&self) -> Option<&'static str> {
        Some("todo-foreign-iced-style viewport")
    }
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("todo-foreign-iced-style-demo", init_window, view, |d| {
        d.drive_embedded_viewport_foreign()
    })?
    .with_main_window("todo_foreign_iced_style_demo", (980.0, 640.0))
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

fn init_window(app: &mut App, window: AppWindowId) -> TodoForeignIcedStyleState {
    embedded::ensure_models(app, window);

    let (foreign_ui, foreign) = ForeignTodoUi::new(app);
    embedded::set_foreign_ui(app, window, foreign_ui);

    TodoForeignIcedStyleState {
        embedded: embedded::EmbeddedViewportSurface::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
            (640, 360),
        ),
        foreign,
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoForeignIcedStyleState) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();

    let Some(models) = embedded::models(&*cx.app, cx.window) else {
        return vec![cx.text("Embedded viewport models are not installed.")];
    };
    cx.watch_model(&models.target).paint().observe();

    let items = cx
        .watch_model(&st.foreign.items)
        .layout()
        .cloned()
        .unwrap_or_default();
    let hint = cx
        .watch_model(&st.foreign.hint)
        .paint()
        .cloned()
        .unwrap_or_else(|| Arc::from("<error>"));

    let clicks = cx.watch_model(&models.clicks).paint().copied().unwrap_or(0);
    let last_input = cx
        .watch_model(&models.last_input)
        .paint()
        .cloned()
        .unwrap_or_else(|| Arc::from("<error>"));

    let mut root_layout = fret_ui::element::LayoutStyle::default();
    root_layout.size.width = Length::Fill;
    root_layout.size.height = Length::Fill;

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

            let left = cx.flex(flex, |cx| {
                let title = cx.text("Foreign UI interop (iced-style sketch)");

                let hint = cx.text(hint);
                let last = cx.text(Arc::from(format!(
                    "clicks={} last_input={}",
                    clicks, last_input
                )));

                let lines = items.into_iter().map(|it| {
                    let mark = if it.done { "[x]" } else { "[ ]" };
                    cx.text(Arc::from(format!("{mark} {}", it.text)))
                });

                [title, hint, last]
                    .into_iter()
                    .chain(lines)
                    .collect::<Vec<_>>()
            });

            let right = cx.container(Default::default(), |cx| {
                let panel = st.embedded.panel(
                    cx,
                    embedded::EmbeddedViewportPanelProps {
                        fit: ViewportFit::Contain,
                        opacity: 1.0,
                        forward_input: true,
                    },
                );
                [panel]
            });

            [left, right]
        },
    );

    vec![root]
}
