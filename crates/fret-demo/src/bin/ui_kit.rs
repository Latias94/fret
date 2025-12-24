use fret_app::{App, Effect, Menu, MenuItem, WindowRequest};
use fret_components_icons::IconId;
use fret_components_ui::{
    StyleRefinement,
    button::{Button, ButtonIntent, ButtonSize, ButtonVariant},
    checkbox::Checkbox,
    dropdown_menu::DropdownMenuButton,
    frame::Frame,
    icon_button::IconButton,
    progress::ProgressBar,
    select::{Select, SelectOption},
    separator::Separator,
    slider::Slider,
    switch::Switch,
    tabs::Tabs,
    text_field::TextField,
    toolbar::Toolbar,
    tooltip::TooltipArea,
};
use fret_core::{AppWindowId, NodeId, PlatformCapabilities, Px, Rect, Scene, Size, TextService};
use fret_render::{ImageColorSpace, ImageDescriptor, Renderer, WgpuContext};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui_app::{
    ColoredPanel, Column, ContextMenu, ContextMenuService, FixedPanel, Invalidation,
    PanelThemeBackground, Popover, PopoverService, ResizableSplit, Row, Scroll, Stack, Text, Theme,
    ThemeConfig, TooltipOverlay, TooltipService, UiLayerId, UiTree,
};
use std::sync::Arc;
use winit::event_loop::EventLoop;

#[derive(Debug)]
struct UiKitImage {
    id: fret_core::ImageId,
    #[allow(dead_code)]
    texture: wgpu::Texture,
    #[allow(dead_code)]
    view: wgpu::TextureView,
}

#[derive(Default)]
struct UiKitDriver {
    ui_kit_image: Option<UiKitImage>,
}

struct UiKitWindowState {
    ui: UiTree,
    root: NodeId,
    popover_layer: UiLayerId,
    popover_node: NodeId,
    popover_previous_focus: Option<NodeId>,
    context_menu_layer: UiLayerId,
    context_menu_node: NodeId,
    context_menu_previous_focus: Option<NodeId>,
}

fn load_theme(app: &mut App) {
    let candidates = [
        "./.fret/theme.json",
        "./themes/fret-default-dark.json",
        "./themes/godot-default-dark.json",
        "./themes/hardhacker-dark.json",
    ];
    for path in candidates {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        match ThemeConfig::from_slice(&bytes) {
            Ok(cfg) => {
                Theme::global_mut(app).apply_config(&cfg);
                tracing::info!(theme = %cfg.name, path = %path, "loaded theme");
                return;
            }
            Err(err) => {
                tracing::error!(error = %err, path = %path, "failed to parse theme file");
            }
        }
    }
}

fn build_ui_kit_contents(
    app: &mut App,
    ui: &mut UiTree,
    parent: NodeId,
    image: Option<fret_core::ImageId>,
) {
    let col = ui.create_node(Column::new().with_padding(Px(16.0)).with_spacing(Px(12.0)));
    ui.add_child(parent, col);

    let title = ui.create_node(Text::new("UI Kit"));
    ui.add_child(col, title);

    let subtitle = ui.create_node(Text::new(
        "Token-driven primitives (shadcn-inspired) + retained UI runtime.",
    ));
    ui.add_child(col, subtitle);

    let buttons_frame = ui.create_node(Frame::new(
        StyleRefinement::default()
            .rounded_md()
            .border_1()
            .px_3()
            .py_1(),
    ));
    let buttons = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let primary = ui.create_node(
        Button::new("Primary")
            .intent(ButtonIntent::Primary)
            .size(ButtonSize::Md),
    );
    let default_btn = ui.create_node(
        Button::new("Default")
            .variant(ButtonVariant::Default)
            .size(ButtonSize::Md),
    );
    let ghost = ui.create_node(
        Button::new("Ghost")
            .variant(ButtonVariant::Ghost)
            .size(ButtonSize::Md),
    );
    let disabled = ui.create_node(
        Button::new("Disabled")
            .disabled(true)
            .variant(ButtonVariant::Default),
    );
    ui.add_child(buttons, primary);
    ui.add_child(buttons, default_btn);
    ui.add_child(buttons, ghost);
    ui.add_child(buttons, disabled);
    ui.add_child(buttons_frame, buttons);
    ui.add_child(col, buttons_frame);

    let text_model = app.models_mut().insert("Hello, components.".to_string());
    let text_field = ui.create_node(
        TextField::new(text_model).refine_style(
            StyleRefinement::default()
                .rounded_md()
                .border_1()
                .px_3()
                .py_1(),
        ),
    );
    ui.add_child(col, text_field);

    let checkbox_model = app.models_mut().insert(false);
    let checkbox = ui.create_node(Checkbox::new(checkbox_model, "Enable option"));
    ui.add_child(col, checkbox);

    let switch_model = app.models_mut().insert(false);
    let switch_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let switch_node = ui.create_node(Switch::new(switch_model));
    let switch_label = ui.create_node(Text::new("Switch"));
    ui.add_child(switch_row, switch_node);
    ui.add_child(switch_row, switch_label);
    ui.add_child(col, switch_row);

    let slider_model = app.models_mut().insert(0.35f32);
    let slider_label = ui.create_node(Text::new("Slider / Progress"));
    ui.add_child(col, slider_label);
    let slider = ui.create_node(Slider::new(slider_model).range(0.0, 1.0));
    ui.add_child(col, slider);
    let progress = ui.create_node(ProgressBar::new(slider_model));
    ui.add_child(col, progress);

    let select_model = app.models_mut().insert(0usize);
    let select = ui.create_node(Select::new(
        select_model,
        vec![
            SelectOption::new("First"),
            SelectOption::new("Second"),
            SelectOption::new("Third"),
            SelectOption::new("Disabled").disabled(),
        ],
    ));
    ui.add_child(col, select);

    let separator = ui.create_node(Separator::horizontal());
    ui.add_child(col, separator);

    let icons_frame = ui.create_node(Frame::new(
        StyleRefinement::default()
            .rounded_md()
            .border_1()
            .px_3()
            .py_1(),
    ));
    let icons = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let icon_play_tip = ui.create_node(TooltipArea::new("Play"));
    let icon_play = ui.create_node(IconButton::new(IconId::new("play")));
    ui.add_child(icon_play_tip, icon_play);
    ui.add_child(icons, icon_play_tip);

    let icon_settings_tip = ui.create_node(TooltipArea::new("Settings"));
    let icon_settings = ui.create_node(IconButton::new(IconId::new("settings")));
    ui.add_child(icon_settings_tip, icon_settings);
    ui.add_child(icons, icon_settings_tip);

    let icon_close_tip = ui.create_node(TooltipArea::new("Close"));
    let icon_close = ui.create_node(IconButton::new(IconId::new("close")));
    ui.add_child(icon_close_tip, icon_close);
    ui.add_child(icons, icon_close_tip);
    ui.add_child(icons_frame, icons);
    ui.add_child(col, icons_frame);

    let dropdown_menu = Menu {
        title: Arc::from("Actions"),
        items: vec![
            MenuItem::Command {
                command: fret_app::CommandId::from("ui_kit.action.one"),
                when: None,
            },
            MenuItem::Command {
                command: fret_app::CommandId::from("ui_kit.action.two"),
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Submenu {
                title: Arc::from("More"),
                when: None,
                items: vec![MenuItem::Command {
                    command: fret_app::CommandId::from("ui_kit.action.three"),
                    when: None,
                }],
            },
        ],
    };
    let dropdown = ui.create_node(
        DropdownMenuButton::new("DropdownMenu", dropdown_menu).refine_style(
            StyleRefinement::default()
                .rounded_md()
                .border_1()
                .px_3()
                .py_1(),
        ),
    );
    ui.add_child(col, dropdown);

    if let Some(img) = image {
        let image_frame = ui.create_node(Frame::new(
            StyleRefinement::default()
                .rounded_md()
                .border_1()
                .px_3()
                .py_1(),
        ));
        let row = ui.create_node(Row::new().with_spacing(Px(10.0)));
        let image_node =
            ui.create_node(fret_ui_app::Image::new(img).with_size(Size::new(Px(160.0), Px(120.0))));
        let image_label = ui.create_node(Text::new(
            "Renderer-registered image (checkerboard) drawn via ImageId.",
        ));
        ui.add_child(row, image_node);
        ui.add_child(row, image_label);
        ui.add_child(image_frame, row);
        ui.add_child(col, image_frame);
    }

    let tabs_model = app.models_mut().insert(0usize);
    let tabs = ui.create_node(Tabs::new(
        tabs_model,
        vec!["Scene", "Game", "Inspector", "Console"],
    ));
    ui.add_child(col, tabs);

    let toolbar = ui.create_node(
        Toolbar::new().refine_style(
            StyleRefinement::default()
                .rounded_md()
                .border_1()
                .px_3()
                .py_1(),
        ),
    );
    let toolbar_play = ui.create_node(Button::new("Play").intent(ButtonIntent::Primary));
    let toolbar_settings = ui.create_node(Button::new("Settings").variant(ButtonVariant::Ghost));
    ui.add_child(toolbar, toolbar_play);
    ui.add_child(toolbar, toolbar_settings);
    ui.add_child(col, toolbar);

    // Framework-level layout primitive demo: resizable split with a thick hit target and
    // hairline divider, using a `Model<f32>` for persistence.
    let split_sep = ui.create_node(Separator::horizontal());
    ui.add_child(col, split_sep);
    let split_title = ui.create_node(Text::new("Resizable split (prototype)"));
    ui.add_child(col, split_title);

    let split_fraction = app.models_mut().insert(0.5f32);
    let split = ui.create_node(
        ResizableSplit::new(fret_core::Axis::Horizontal, split_fraction)
            .with_min_px(Px(140.0))
            .with_hit_thickness(Px(8.0))
            .with_paint_device_px(1.0),
    );

    // `Column` measures children with a very large available height. Constrain the demo split's
    // height so it stays visually reasonable and makes dragging obvious.
    let split_frame = ui.create_node(FixedPanel::new(Px(220.0), fret_core::Color::TRANSPARENT));
    ui.add_child(col, split_frame);
    ui.add_child(split_frame, split);

    let left = ui.create_node(Stack::new());
    let right = ui.create_node(Stack::new());

    let left_bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Panel, 1.0));
    let right_bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Surface, 1.0));

    let left_body = ui.create_node(Column::new().with_padding(Px(10.0)).with_spacing(Px(6.0)));
    let right_body = ui.create_node(Column::new().with_padding(Px(10.0)).with_spacing(Px(6.0)));

    let left_label = ui.create_node(Text::new("Left pane"));
    let left_hint = ui.create_node(Text::new("Drag the divider to resize."));
    ui.add_child(left_body, left_label);
    ui.add_child(left_body, left_hint);

    let right_label = ui.create_node(Text::new("Right pane"));
    let right_hint = ui.create_node(Text::new("Hit target is thicker than the hairline."));
    ui.add_child(right_body, right_label);
    ui.add_child(right_body, right_hint);

    ui.add_child(left, left_bg);
    ui.add_child(left, left_body);
    ui.add_child(right, right_bg);
    ui.add_child(right, right_body);

    ui.add_child(split, left);
    ui.add_child(split, right);
}

impl WinitDriver for UiKitDriver {
    type WindowState = UiKitWindowState;

    fn init(&mut self, app: &mut App, _main_window: AppWindowId) {
        app.with_global_mut(PopoverService::default, |_svc, _app| {});
        app.with_global_mut(ContextMenuService::default, |_svc, _app| {});
        app.with_global_mut(TooltipService::default, |_svc, _app| {});
        load_theme(app);
    }

    fn gpu_ready(&mut self, _app: &mut App, context: &WgpuContext, renderer: &mut Renderer) {
        if self.ui_kit_image.is_some() {
            return;
        }

        let tex_w = 128u32;
        let tex_h = 96u32;
        let format = wgpu::TextureFormat::Rgba8UnormSrgb;

        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("fret-ui-kit checkerboard"),
            size: wgpu::Extent3d {
                width: tex_w,
                height: tex_h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut pixels: Vec<u8> = vec![0; (tex_w * tex_h * 4) as usize];
        for y in 0..tex_h {
            for x in 0..tex_w {
                let i = ((y * tex_w + x) * 4) as usize;
                let checker = ((x / 8) ^ (y / 8)) & 1;
                let (mut r, mut g, mut b) = if checker == 0 {
                    (230u8, 230u8, 235u8)
                } else {
                    (150u8, 150u8, 160u8)
                };
                if x % 16 == 0 || y % 16 == 0 {
                    r = 255;
                    g = 120;
                    b = 90;
                }
                pixels[i] = r;
                pixels[i + 1] = g;
                pixels[i + 2] = b;
                pixels[i + 3] = 255;
            }
        }

        context.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(tex_w * 4),
                rows_per_image: Some(tex_h),
            },
            wgpu::Extent3d {
                width: tex_w,
                height: tex_h,
                depth_or_array_layers: 1,
            },
        );

        let id = renderer.register_image(ImageDescriptor {
            view: view.clone(),
            size: (tex_w, tex_h),
            format,
            color_space: ImageColorSpace::Srgb,
        });

        self.ui_kit_image = Some(UiKitImage { id, texture, view });
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        let mut ui = UiTree::new();
        ui.set_window(window);

        let root = ui.create_node(Stack::new());
        ui.set_root(root);

        let bg = ui.create_node(ColoredPanel::themed(PanelThemeBackground::Surface, 1.0));
        ui.add_child(root, bg);

        let scroll = ui.create_node(Scroll::new());
        ui.add_child(root, scroll);

        build_ui_kit_contents(
            app,
            &mut ui,
            scroll,
            self.ui_kit_image.as_ref().map(|i| i.id),
        );

        let tooltip_node = ui.create_node(TooltipOverlay::new());
        let _tooltip_layer = ui.push_overlay_root_ex(tooltip_node, false, false);

        let popover_node = ui.create_node(Popover::new());
        let popover_layer = ui.push_overlay_root(popover_node, true);
        ui.set_layer_visible(popover_layer, false);

        let context_menu_node = ui.create_node(ContextMenu::new());
        let context_menu_layer = ui.push_overlay_root(context_menu_node, true);
        ui.set_layer_visible(context_menu_layer, false);

        UiKitWindowState {
            ui,
            root,
            popover_layer,
            popover_node,
            popover_previous_focus: None,
            context_menu_layer,
            context_menu_node,
            context_menu_previous_focus: None,
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        text: &mut dyn TextService,
        window: AppWindowId,
        state: &mut Self::WindowState,
        event: &fret_core::Event,
    ) {
        if matches!(event, fret_core::Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        state.ui.dispatch_event(app, text, event);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        text: &mut dyn TextService,
        window: AppWindowId,
        state: &mut Self::WindowState,
        command: fret_app::CommandId,
    ) {
        match command.as_str() {
            "popover.open" => {
                let has_request = app
                    .global::<PopoverService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return;
                }
                state.popover_previous_focus = state.ui.focus();
                state.ui.set_layer_visible(state.popover_layer, true);
                state.ui.set_focus(Some(state.popover_node));
                app.request_redraw(window);
            }
            "popover.close" => {
                if state.ui.is_layer_visible(state.popover_layer) {
                    state.ui.cleanup_subtree(text, state.popover_node);
                    state.ui.set_layer_visible(state.popover_layer, false);
                }
                app.with_global_mut(PopoverService::default, |service, _app| {
                    service.clear_request(window);
                });
                if let Some(prev) = state.popover_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }
                app.request_redraw(window);
            }
            "context_menu.open" => {
                let has_request = app
                    .global::<ContextMenuService>()
                    .and_then(|s| s.request(window))
                    .is_some();
                if !has_request {
                    return;
                }
                state.context_menu_previous_focus = state.ui.focus();
                state.ui.set_layer_visible(state.context_menu_layer, true);
                state.ui.set_focus(Some(state.context_menu_node));
                app.request_redraw(window);
            }
            "context_menu.close" => {
                if state.ui.is_layer_visible(state.context_menu_layer) {
                    state.ui.cleanup_subtree(text, state.context_menu_node);
                    state.ui.set_layer_visible(state.context_menu_layer, false);
                }
                app.with_global_mut(ContextMenuService::default, |service, app| {
                    let action = service.take_pending_action(window);
                    service.clear(window);
                    if let Some(command) = action {
                        app.push_effect(Effect::Command {
                            window: Some(window),
                            command,
                        });
                    }
                });
                if let Some(prev) = state.context_menu_previous_focus.take() {
                    state.ui.set_focus(Some(prev));
                }
                app.request_redraw(window);
            }
            other => {
                tracing::info!(window = ?window, command = %other, "unhandled command");
            }
        }
    }

    fn invalidate_ui_layout(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        state.ui.invalidate(state.root, Invalidation::Layout);
    }

    fn handle_model_changes(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        state.ui.propagate_model_changes(app, changed);
    }

    fn render(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        text: &mut dyn fret_core::TextService,
        scene: &mut Scene,
    ) {
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        state.ui.layout_all(app, text, bounds, scale_factor);
        state.ui.paint_all(app, text, bounds, scene, scale_factor);
    }

    fn window_create_spec(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
    ) -> Option<WindowCreateSpec> {
        None
    }

    fn window_created(
        &mut self,
        _app: &mut App,
        _request: &fret_app::CreateWindowRequest,
        _new_window: AppWindowId,
    ) {
    }
}

fn main() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_runner_winit_wgpu=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::new()?;
    let config = WinitRunnerConfig {
        main_window_title: "fret-ui-kit".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    let driver = UiKitDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
