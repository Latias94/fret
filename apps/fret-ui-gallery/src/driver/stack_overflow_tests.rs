use super::*;
use fret_core::scene::Scene;
use fret_core::{
    PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle, Point, Px, Rect,
    Size, SvgId, SvgService, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
};
use fret_runtime::FrameId;
use fret_ui::ElementRuntime;
use slotmap::SlotMap;

#[derive(Default)]
struct DummyServices {
    text_blobs: SlotMap<TextBlobId, ()>,
    svgs: SlotMap<SvgId, ()>,
    paths: SlotMap<PathId, ()>,
}

impl TextService for DummyServices {
    fn prepare(
        &mut self,
        input: &TextInput,
        constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        let text = input.text();
        let char_w = 7.5;
        let line_h = 16.0;
        let mut w = Px(text.chars().count() as f32 * char_w);
        if let Some(max) = constraints.max_width {
            w = Px(w.0.min(max.0));
        }
        let metrics = TextMetrics {
            size: Size::new(w, Px(line_h)),
            baseline: Px(line_h * 0.8),
        };
        let id = self.text_blobs.insert(());
        (id, metrics)
    }

    fn release(&mut self, blob: TextBlobId) {
        let _ = self.text_blobs.remove(blob);
    }
}

impl SvgService for DummyServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        self.svgs.insert(())
    }

    fn unregister_svg(&mut self, svg: SvgId) -> bool {
        self.svgs.remove(svg).is_some()
    }
}

impl PathService for DummyServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        let id = self.paths.insert(());
        (id, PathMetrics::default())
    }

    fn release(&mut self, path: PathId) {
        let _ = self.paths.remove(path);
    }
}

fn drive_frame(
    app: &mut App,
    services: &mut DummyServices,
    window: AppWindowId,
    state: &mut UiGalleryWindowState,
    bounds: Rect,
) {
    UiGalleryDriver::render_ui(app, services, window, state, bounds);
    state.ui.layout_all(app, services, bounds, 1.0);
    let mut scene = Scene::default();
    state.ui.paint_all(app, services, bounds, &mut scene, 1.0);
}

fn ui_tree_stats(ui: &UiTree<App>, root: fret_core::NodeId) -> (usize, usize) {
    let mut max_depth: usize = 0;
    let mut node_count: usize = 0;

    let mut stack: Vec<(fret_core::NodeId, usize)> = vec![(root, 0)];
    while let Some((node, depth)) = stack.pop() {
        node_count = node_count.saturating_add(1);
        max_depth = max_depth.max(depth);
        for child in ui.children(node) {
            stack.push((child, depth + 1));
        }
    }

    (node_count, max_depth)
}

#[test]
fn nav_to_datatable_does_not_stack_overflow_without_gc_sweep() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1024.0), Px(768.0)),
    );

    let mut app = build_app();
    app.set_frame_id(FrameId(1));
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.set_gc_lag_frames(10_000);
    });

    let mut state = UiGalleryDriver::build_ui(&mut app, window);
    let mut services = DummyServices::default();

    eprintln!("frame1: intro");
    drive_frame(&mut app, &mut services, window, &mut state, bounds);
    if let Some(root) = state.root {
        let (nodes, depth) = ui_tree_stats(&state.ui, root);
        eprintln!("frame1: ui_tree nodes={nodes} max_depth={depth}");
    }

    let cmd = CommandId::new(CMD_NAV_DATA_TABLE);
    eprintln!("nav: data_table");
    assert!(UiGalleryDriver::handle_nav_command(
        &mut app, &mut state, &cmd
    ));

    app.set_frame_id(FrameId(2));
    eprintln!("frame2: data_table");
    drive_frame(&mut app, &mut services, window, &mut state, bounds);
    if let Some(root) = state.root {
        let (nodes, depth) = ui_tree_stats(&state.ui, root);
        eprintln!("frame2: ui_tree nodes={nodes} max_depth={depth}");
    }
}

#[test]
fn nav_to_datatable_does_not_stack_overflow_with_immediate_gc_sweep() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1024.0), Px(768.0)),
    );

    let mut app = build_app();
    app.set_frame_id(FrameId(1));
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.set_gc_lag_frames(0);
    });

    let mut state = UiGalleryDriver::build_ui(&mut app, window);
    let mut services = DummyServices::default();

    eprintln!("frame1: intro");
    drive_frame(&mut app, &mut services, window, &mut state, bounds);

    let cmd = CommandId::new(CMD_NAV_DATA_TABLE);
    eprintln!("nav: data_table");
    assert!(UiGalleryDriver::handle_nav_command(
        &mut app, &mut state, &cmd
    ));

    app.set_frame_id(FrameId(2));
    eprintln!("frame2: data_table (gc sweep)");
    drive_frame(&mut app, &mut services, window, &mut state, bounds);
}

#[test]
#[ignore]
fn nav_to_datatable_does_not_stack_overflow_with_wgpu_renderer_services() {
    let window = AppWindowId::default();
    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1024.0), Px(768.0)),
    );

    let wgpu = pollster::block_on(fret_render::WgpuContext::new()).unwrap();
    let mut renderer = fret_render::Renderer::new(&wgpu.adapter, &wgpu.device);

    let format = wgpu::TextureFormat::Bgra8UnormSrgb;
    let viewport_size = (1024u32, 768u32);
    let target = wgpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("ui-gallery test target"),
        size: wgpu::Extent3d {
            width: viewport_size.0,
            height: viewport_size.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut app = build_app();
    app.set_frame_id(FrameId(1));
    app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
        runtime.set_gc_lag_frames(0);
    });

    let mut state = UiGalleryDriver::build_ui(&mut app, window);

    eprintln!("frame1: intro (render/layout/paint/render_scene)");
    UiGalleryDriver::render_ui(&mut app, &mut renderer, window, &mut state, bounds);
    state.ui.layout_all(&mut app, &mut renderer, bounds, 1.0);
    let mut scene = Scene::default();
    state
        .ui
        .paint_all(&mut app, &mut renderer, bounds, &mut scene, 1.0);
    let cmd = renderer.render_scene(
        &wgpu.device,
        &wgpu.queue,
        fret_render::RenderSceneParams {
            format,
            target_view: &view,
            scene: &scene,
            clear: fret_render::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );
    wgpu.queue.submit([cmd]);

    let cmd_nav = CommandId::new(CMD_NAV_DATA_TABLE);
    eprintln!("nav: data_table");
    assert!(UiGalleryDriver::handle_nav_command(
        &mut app, &mut state, &cmd_nav,
    ));

    app.set_frame_id(FrameId(2));
    eprintln!("frame2: data_table (render/layout/paint/render_scene)");
    UiGalleryDriver::render_ui(&mut app, &mut renderer, window, &mut state, bounds);
    state.ui.layout_all(&mut app, &mut renderer, bounds, 1.0);
    scene.clear();
    state
        .ui
        .paint_all(&mut app, &mut renderer, bounds, &mut scene, 1.0);
    let cmd = renderer.render_scene(
        &wgpu.device,
        &wgpu.queue,
        fret_render::RenderSceneParams {
            format,
            target_view: &view,
            scene: &scene,
            clear: fret_render::ClearColor::default(),
            scale_factor: 1.0,
            viewport_size,
        },
    );
    wgpu.queue.submit([cmd]);
}

#[test]
#[ignore]
fn nav_to_datatable_repro_on_small_stack() {
    #[stacksafe::stacksafe]
    fn render_ui_stacksafe(
        app: &mut App,
        services: &mut fret_render::Renderer,
        window: AppWindowId,
        state: &mut UiGalleryWindowState,
        bounds: Rect,
    ) {
        UiGalleryDriver::render_ui(app, services, window, state, bounds);
    }

    let join = std::thread::Builder::new()
        .name("ui-gallery-small-stack".to_string())
        .stack_size(1024 * 1024)
        .spawn(|| {
            stacksafe::set_minimum_stack_size(2 * 1024 * 1024);
            stacksafe::set_stack_allocation_size(8 * 1024 * 1024);

            let window = AppWindowId::default();
            let bounds = Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(Px(1024.0), Px(768.0)),
            );

            let wgpu = pollster::block_on(fret_render::WgpuContext::new()).unwrap();
            let mut renderer = fret_render::Renderer::new(&wgpu.adapter, &wgpu.device);

            let format = wgpu::TextureFormat::Bgra8UnormSrgb;
            let viewport_size = (1024u32, 768u32);
            let target = wgpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("ui-gallery small-stack test target"),
                size: wgpu::Extent3d {
                    width: viewport_size.0,
                    height: viewport_size.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let view = target.create_view(&wgpu::TextureViewDescriptor::default());

            let mut app = build_app();
            app.set_frame_id(FrameId(1));
            app.with_global_mut_untracked(ElementRuntime::new, |runtime, _app| {
                let lag = std::env::var("FRET_UI_GALLERY_SMALL_STACK_GC_LAG")
                    .ok()
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(2);
                runtime.set_gc_lag_frames(lag);
            });

            let mut state = UiGalleryDriver::build_ui(&mut app, window);

            eprintln!("frame1: intro (small stack)");
            UiGalleryDriver::render_ui(&mut app, &mut renderer, window, &mut state, bounds);
            eprintln!("frame1: after render_ui (small stack)");
            eprintln!("frame1: before layout_all (small stack)");
            state.ui.layout_all(&mut app, &mut renderer, bounds, 1.0);
            eprintln!("frame1: after layout_all (small stack)");
            let mut scene = Scene::default();
            state
                .ui
                .paint_all(&mut app, &mut renderer, bounds, &mut scene, 1.0);
            eprintln!("frame1: after paint_all (small stack)");
            let cmd = renderer.render_scene(
                &wgpu.device,
                &wgpu.queue,
                fret_render::RenderSceneParams {
                    format,
                    target_view: &view,
                    scene: &scene,
                    clear: fret_render::ClearColor::default(),
                    scale_factor: 1.0,
                    viewport_size,
                },
            );
            eprintln!("frame1: after render_scene (small stack)");
            wgpu.queue.submit([cmd]);
            eprintln!("frame1: after submit (small stack)");

            let cmd_nav = CommandId::new(CMD_NAV_DATA_TABLE);
            eprintln!("nav: data_table (small stack)");
            assert!(UiGalleryDriver::handle_nav_command(
                &mut app, &mut state, &cmd_nav,
            ));

            app.set_frame_id(FrameId(2));
            if let Ok(flags) = std::env::var("FRET_UI_GALLERY_SMALL_STACK_BISECT") {
                eprintln!("apply bisect for frame2: {flags}");
                unsafe {
                    std::env::set_var(ENV_UI_GALLERY_BISECT, flags);
                }
            }
            eprintln!("frame2: data_table (small stack)");
            render_ui_stacksafe(&mut app, &mut renderer, window, &mut state, bounds);
            eprintln!("frame2: after render_ui (small stack)");
            eprintln!("frame2: before layout_all (small stack)");
            state.ui.layout_all(&mut app, &mut renderer, bounds, 1.0);
            eprintln!("frame2: after layout_all (small stack)");
            scene.clear();
            state
                .ui
                .paint_all(&mut app, &mut renderer, bounds, &mut scene, 1.0);
            eprintln!("frame2: after paint_all (small stack)");
            let cmd = renderer.render_scene(
                &wgpu.device,
                &wgpu.queue,
                fret_render::RenderSceneParams {
                    format,
                    target_view: &view,
                    scene: &scene,
                    clear: fret_render::ClearColor::default(),
                    scale_factor: 1.0,
                    viewport_size,
                },
            );
            eprintln!("frame2: after render_scene (small stack)");
            wgpu.queue.submit([cmd]);
            eprintln!("frame2: after submit (small stack)");
        })
        .unwrap();
    join.join().unwrap();
}
