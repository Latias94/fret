use std::collections::HashSet;
use std::sync::Arc;

use fret_core::ViewportFit;
use fret_kit::prelude::*;
use fret_launch::{EngineFrameUpdate, ViewportRenderTarget};
use fret_render::{RenderTargetColorSpace, Renderer, WgpuContext};
use fret_runtime::{FrameId, TickId};
use fret_ui::element::AnyElementIterExt as _;
use fret_ui_kit::declarative as kit_decl;

const CMD_ADD: &str = "todo-interop-kit.add";
const CMD_CLEAR_DONE: &str = "todo-interop-kit.clear_done";
const CMD_REMOVE_PREFIX: &str = "todo-interop-kit.remove.";

#[derive(Debug, Clone)]
struct ExternalInteropModels {
    clicks: Model<u32>,
    last_input: Model<Arc<str>>,
    target: Model<fret_core::RenderTargetId>,
}

fn install_external_interop_models(app: &mut App) {
    let clicks = app.models_mut().insert(0u32);
    let last_input = app.models_mut().insert(Arc::<str>::from(
        "Click inside the embedded viewport surface.",
    ));
    let target = app
        .models_mut()
        .insert(fret_core::RenderTargetId::default());
    app.set_global(ExternalInteropModels {
        clicks,
        last_input,
        target,
    });
}

fn external_models(app: &App) -> ExternalInteropModels {
    app.global::<ExternalInteropModels>()
        .cloned()
        .expect("ExternalInteropModels must be installed in init_app")
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
    next_id: u64,

    external_target: ViewportRenderTarget,
    external_target_px_size: (u32, u32),
}

pub fn run() -> anyhow::Result<()> {
    fret_kit::app_with_hooks("todo-interop-kit-demo", init_window, view, |d| {
        d.on_command(on_command)
            .viewport_input(on_viewport_input)
            .record_engine_frame(record_engine_frame)
    })?
    .with_main_window("todo_interop_kit_demo", (980.0, 640.0))
    .init_app(|app| {
        install_external_interop_models(app);
        shadcn::shadcn_themes::apply_shadcn_new_york_v4(
            app,
            shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            shadcn::shadcn_themes::ShadcnColorScheme::Light,
        );
    })
    .run()?;
    Ok(())
}

fn init_window(app: &mut App, _window: AppWindowId) -> TodoInteropKitState {
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

    TodoInteropKitState {
        todos,
        draft,
        next_id: 3,
        external_target: ViewportRenderTarget::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
        ),
        external_target_px_size: (640, 360),
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut TodoInteropKitState) -> Vec<AnyElement> {
    cx.observe_model(&st.todos, Invalidation::Layout);
    cx.observe_model(&st.draft, Invalidation::Layout);

    let models = external_models(&*cx.app);
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

            let left = todo_panel(cx, &theme, st, &todos);
            let right = external_panel(cx, target, st.external_target_px_size, clicks, last_input);

            vec![cx.flex(flex, move |_cx| [left, right])]
        },
    );

    vec![root]
}

fn todo_panel(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    st: &TodoInteropKitState,
    todos: &[TodoItem],
) -> AnyElement {
    let header = shadcn::CardHeader::new([shadcn::CardTitle::new("Todo").into_element(cx)]);

    let draft = shadcn::Input::new(st.draft.clone())
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

    let list = todos.iter().map(|t| todo_row(cx, theme, t)).elements();

    let body = shadcn::CardContent::new([
        stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full()),
            move |_cx| vec![draft, add, clear_done],
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

fn todo_row(cx: &mut ElementContext<'_, App>, theme: &Theme, item: &TodoItem) -> AnyElement {
    let checkbox = shadcn::Checkbox::new(item.done.clone())
        .a11y_label("Done")
        .into_element(cx);
    let text = cx.text(item.text.clone());
    let remove = shadcn::Button::new("Remove")
        .variant(shadcn::ButtonVariant::Ghost)
        .size(shadcn::ButtonSize::Sm)
        .on_click(CommandId::new(format!("{CMD_REMOVE_PREFIX}{}", item.id)))
        .into_element(cx);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N2)
            .items_center()
            .layout(LayoutRefinement::default().w_full()),
        move |_cx| vec![checkbox, text, remove],
    );

    let props = shadcn::decl_style::container_props(
        theme,
        ChromeRefinement::default()
            .border_1()
            .rounded(Radius::Md)
            .p(Space::N2),
        LayoutRefinement::default().w_full(),
    );
    cx.container(props, move |_cx| vec![row])
}

fn external_panel(
    cx: &mut ElementContext<'_, App>,
    target: fret_core::RenderTargetId,
    target_px_size: (u32, u32),
    clicks: u32,
    last_input: Arc<str>,
) -> AnyElement {
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

    let viewport = {
        let props = kit_decl::viewport_surface::ViewportSurfacePanelProps {
            target,
            target_px_size,
            fit: ViewportFit::Contain,
            opacity: 1.0,
            forward_input: true,
        };
        kit_decl::viewport_surface::viewport_surface_panel(cx, props)
    };

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
            if let Some(rest) = command.as_str().strip_prefix(CMD_REMOVE_PREFIX) {
                if let Ok(id) = rest.parse::<u64>() {
                    let _ = app
                        .models_mut()
                        .update(&st.todos, |v| v.retain(|t| t.id != id));
                }
            }
        }
    }
}

fn on_viewport_input(app: &mut App, event: fret_core::ViewportInputEvent) {
    let Some(models) = app.global::<ExternalInteropModels>().cloned() else {
        return;
    };

    let target = app
        .models()
        .read(&models.target, |v| *v)
        .unwrap_or_default();
    if target == fret_core::RenderTargetId::default() || event.target != target {
        return;
    }

    if matches!(event.kind, fret_core::ViewportInputKind::PointerDown { .. }) {
        let _ = app
            .models_mut()
            .update(&models.clicks, |v| *v = v.saturating_add(1));
    }

    let msg: Arc<str> = Arc::from(format!(
        "kind={:?} uv=({:.3},{:.3}) target_px={:?}",
        event.kind, event.uv.0, event.uv.1, event.target_px
    ));
    let _ = app.models_mut().update(&models.last_input, |v| *v = msg);
    app.request_redraw(event.window);
}

fn record_engine_frame(
    app: &mut App,
    window: AppWindowId,
    _ui: &mut UiTree<App>,
    st: &mut TodoInteropKitState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    _scale_factor: f32,
    _tick_id: TickId,
    frame_id: FrameId,
) -> EngineFrameUpdate {
    let models = external_models(app);

    let (id, view) = st.external_target.ensure_size_owned_view(
        context,
        renderer,
        st.external_target_px_size,
        Some("todo-interop-kit viewport"),
    );
    let _ = app.models_mut().update(&models.target, |v| *v = id);

    let clicks = app.models().read(&models.clicks, |v| *v).unwrap_or(0);

    let t = (frame_id.0 as f32 * 0.02).sin() * 0.5 + 0.5;
    let f = ((clicks % 8) as f32) / 8.0;
    let clear = wgpu::Color {
        r: (0.08 + 0.30 * t) as f64,
        g: (0.08 + 0.25 * (1.0 - t)) as f64,
        b: (0.10 + 0.35 * f) as f64,
        a: 1.0,
    };

    let mut encoder = context
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("todo-interop-kit encoder"),
        });
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("todo-interop-kit clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
    }

    let mut update = EngineFrameUpdate::default();
    update.push_command_buffer(encoder.finish());
    app.request_redraw(window);
    update
}
