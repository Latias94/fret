use fret_app::{
    App, BindingV1, Effect, KeySpecV1, Keymap, KeymapFileV1, KeymapService, Menu, MenuItem,
    WindowRequest,
};
use fret_components_icons::IconId;
use fret_components_ui::{
    ContextMenuService, DialogAction, DialogRequest, DialogService, PopoverService,
    Size as ComponentSize, StyleRefinement, ToastAction, TooltipService, WindowOverlays,
    button::{Button, ButtonIntent, ButtonVariant},
    checkbox::Checkbox,
    combobox::Combobox,
    command::{CommandItem, CommandList},
    command_palette::install_command_palette,
    dropdown_menu::DropdownMenuButton,
    frame::Frame,
    icon_button::IconButton,
    list_view::ListView,
    progress::ProgressBar,
    resizable_panel_group::ResizablePanelGroup,
    scroll_area::ScrollArea,
    select::{Select, SelectOption},
    separator::Separator,
    slider::Slider,
    sonner,
    switch::Switch,
    tabs::Tabs,
    text_field::TextField,
    toolbar::Toolbar,
    tooltip::TooltipArea,
};
use fret_core::{
    AppWindowId, Color, NodeId, PlatformCapabilities, Px, Rect, Scene, Size, TextService,
};
use fret_render::{ImageColorSpace, ImageDescriptor, Renderer, WgpuContext};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui_app::{
    ColoredPanel, Column, FixedPanel, Invalidation, PanelThemeBackground, Row, Scroll, Stack, Text,
    Theme, ThemeConfig, UiTree, VirtualList, VirtualListDataSource, VirtualListRow,
    VirtualListRowHeight,
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
    overlays: WindowOverlays,
}

#[derive(Debug, Default, Clone, Copy)]
struct UiKitRichListDataSource {
    len: usize,
}

impl VirtualListDataSource for UiKitRichListDataSource {
    type Key = usize;

    fn len(&self) -> usize {
        self.len
    }

    fn key_at(&self, index: usize) -> Self::Key {
        index
    }

    fn row_at(&self, index: usize) -> VirtualListRow<'_> {
        match index {
            0 => VirtualListRow::new("Recents").header(),
            1 => VirtualListRow::separator(),
            _ => {
                let i = index - 2;
                let leading = if i % 3 == 0 { "●" } else { "○" };
                VirtualListRow::new(format!("Project {i}"))
                    .with_leading_text(leading)
                    .with_secondary_text("Modified 2 hours ago")
                    .with_trailing_text("⌘O")
            }
        }
    }
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
    command_palette_root: NodeId,
) {
    let col = ui.create_node(Column::new().with_padding(Px(16.0)).with_spacing(Px(12.0)));
    ui.add_child(parent, col);

    let title = ui.create_node(Text::new("UI Kit"));
    ui.add_child(col, title);

    let subtitle = ui.create_node(Text::new(
        "Token-driven primitives (shadcn-inspired) + retained UI runtime.",
    ));
    ui.add_child(col, subtitle);

    let size_title = ui.create_node(Text::new("Size matrix (xs/sm/md/lg)"));
    ui.add_child(col, size_title);

    let size_buttons = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let btn_xs = ui.create_node(Button::new("XS").with_size(ComponentSize::XSmall));
    let btn_sm = ui.create_node(Button::new("SM").with_size(ComponentSize::Small));
    let btn_md = ui.create_node(Button::new("MD").with_size(ComponentSize::Medium));
    let btn_lg = ui.create_node(Button::new("LG").with_size(ComponentSize::Large));
    ui.add_child(size_buttons, btn_xs);
    ui.add_child(size_buttons, btn_sm);
    ui.add_child(size_buttons, btn_md);
    ui.add_child(size_buttons, btn_lg);
    ui.add_child(col, size_buttons);

    let text_xs = app.models_mut().insert("TextField xs".to_string());
    let text_sm = app.models_mut().insert("TextField sm".to_string());
    let text_md = app.models_mut().insert("TextField md".to_string());
    let text_lg = app.models_mut().insert("TextField lg".to_string());
    let field_xs = ui.create_node(TextField::new(text_xs).with_size(ComponentSize::XSmall));
    let field_sm = ui.create_node(TextField::new(text_sm).with_size(ComponentSize::Small));
    let field_md = ui.create_node(TextField::new(text_md).with_size(ComponentSize::Medium));
    let field_lg = ui.create_node(TextField::new(text_lg).with_size(ComponentSize::Large));
    ui.add_child(col, field_xs);
    ui.add_child(col, field_sm);
    ui.add_child(col, field_md);
    ui.add_child(col, field_lg);

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
            .with_size(ComponentSize::Medium),
    );
    let default_btn = ui.create_node(
        Button::new("Default")
            .variant(ButtonVariant::Default)
            .with_size(ComponentSize::Medium),
    );
    let ghost = ui.create_node(
        Button::new("Ghost")
            .variant(ButtonVariant::Ghost)
            .with_size(ComponentSize::Medium),
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

    let dialogs_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let open_dialog = ui.create_node(Button::new("Open Dialog").on_click("ui_kit.dialog.open"));
    ui.add_child(dialogs_row, open_dialog);
    ui.add_child(col, dialogs_row);

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

    let command_title = ui.create_node(Text::new("Command (search + virtual list)"));
    ui.add_child(col, command_title);

    let command_query = app.models_mut().insert(String::new());
    let command_selection = app.models_mut().insert(None::<Arc<str>>);
    let command_items = app.models_mut().insert(vec![
        CommandItem::new("ui_kit.action.one", "Run Action One")
            .group("Actions")
            .shortcut("⌘1")
            .keyword("action")
            .keyword("one")
            .detail("Example command handler integration"),
        CommandItem::new("ui_kit.action.two", "Run Action Two")
            .group("Actions")
            .shortcut("⌘2")
            .keyword("action")
            .keyword("two"),
        CommandItem::new("ui_kit.action.three", "Run Action Three")
            .group("Actions")
            .shortcut("⌘3")
            .keyword("action")
            .keyword("three"),
        CommandItem::new("ui_kit.dialog.open", "Open Dialog")
            .group("Overlays")
            .keyword("dialog")
            .detail("Show a modal dialog overlay"),
        CommandItem::new("command_palette.open", "Open Command Palette")
            .group("Overlays")
            .keyword("palette")
            .keyword("command"),
        CommandItem::new("disabled.example", "Disabled item").disabled(),
    ]);

    let command_frame = ui.create_node(Frame::new(
        StyleRefinement::default()
            .rounded_md()
            .border_1()
            .px_3()
            .py_1(),
    ));
    ui.add_child(col, command_frame);

    let command_col = ui.create_node(Column::new().with_spacing(Px(8.0)));
    ui.add_child(command_frame, command_col);

    let command_query_field =
        ui.create_node(TextField::new(command_query).with_size(ComponentSize::Medium));
    ui.add_child(command_col, command_query_field);

    let command_list_panel = ui.create_node(FixedPanel::new(Px(180.0), Color::TRANSPARENT));
    ui.add_child(command_col, command_list_panel);

    let command_list = ui.create_node(
        CommandList::new(command_items, command_query)
            .with_size(ComponentSize::Medium)
            .with_selection_model(command_selection),
    );
    ui.add_child(command_list_panel, command_list);

    let palette_open_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    ui.add_child(col, palette_open_row);
    let open_palette = ui.create_node(
        Button::new("Open Command Palette")
            .on_click(fret_app::CommandId::from("command_palette.open")),
    );
    ui.add_child(palette_open_row, open_palette);

    let _palette = install_command_palette(
        ui,
        app,
        command_palette_root,
        command_items,
        ComponentSize::Medium,
    );

    let scroll_area_label = ui.create_node(Text::new("ScrollArea"));
    ui.add_child(col, scroll_area_label);

    let scroll_area_panel = ui.create_node(FixedPanel::new(Px(160.0), Color::TRANSPARENT));
    ui.add_child(col, scroll_area_panel);

    let scroll_area = ui.create_node(
        ScrollArea::new().refine_style(StyleRefinement::default().rounded_md().border_1()),
    );
    ui.add_child(scroll_area_panel, scroll_area);

    let scroll_area_content =
        ui.create_node(Column::new().with_padding(Px(12.0)).with_spacing(Px(6.0)));
    ui.add_child(scroll_area, scroll_area_content);
    for i in 0..8 {
        let row = ui.create_node(Text::new(format!("ScrollArea row {}", i + 1)));
        ui.add_child(scroll_area_content, row);
    }

    let list_view_label = ui.create_node(Text::new("ListView (virtualized)"));
    ui.add_child(col, list_view_label);

    let items: Vec<String> = (0..20_000).map(|i| format!("Item {}", i + 1)).collect();
    let items_model = app.models_mut().insert(items);
    let selection_model = app.models_mut().insert(None::<usize>);

    let list_panel = ui.create_node(FixedPanel::new(Px(220.0), Color::TRANSPARENT));
    ui.add_child(col, list_panel);

    let list_view =
        ui.create_node(ListView::new(items_model).with_selection_model(selection_model));
    ui.add_child(list_panel, list_view);

    let rich_list_label = ui.create_node(Text::new("VirtualList (rich rows)"));
    ui.add_child(col, rich_list_label);

    let rich_list_panel = ui.create_node(FixedPanel::new(Px(260.0), Color::TRANSPARENT));
    ui.add_child(col, rich_list_panel);

    let rich_list = ui.create_node(
        VirtualList::new(UiKitRichListDataSource { len: 2000 })
            .with_row_height(VirtualListRowHeight::Measured { min: Px(44.0) }),
    );
    ui.add_child(rich_list_panel, rich_list);

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

    // Combobox (typeahead) demo: input + anchored list with cmdk-style keyboard navigation.
    let combobox_items = app.models_mut().insert(vec![
        "Albedo".to_string(),
        "Ambient Occlusion".to_string(),
        "Metallic".to_string(),
        "Normal".to_string(),
        "Roughness".to_string(),
        "Emissive".to_string(),
    ]);
    let combobox_selection = app.models_mut().insert(None::<usize>);
    let combobox_query = app.models_mut().insert(String::new());
    let combobox_title = ui.create_node(Text::new("Combobox (typeahead)"));
    ui.add_child(col, combobox_title);
    let combobox = ui.create_node(
        Combobox::new(combobox_items, combobox_selection, combobox_query)
            .with_size(ComponentSize::Medium),
    );
    ui.add_child(col, combobox);

    let toast_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let toast_success = ui.create_node(
        Button::new("Toast: Success")
            .intent(ButtonIntent::Primary)
            .on_click("ui_kit.toast.success"),
    );
    let toast_error = ui.create_node(
        Button::new("Toast: Error")
            .intent(ButtonIntent::Danger)
            .on_click("ui_kit.toast.error"),
    );
    let toast_action = ui.create_node(
        Button::new("Toast: Action")
            .variant(ButtonVariant::Ghost)
            .on_click("ui_kit.toast.action"),
    );
    ui.add_child(toast_row, toast_success);
    ui.add_child(toast_row, toast_error);
    ui.add_child(toast_row, toast_action);
    ui.add_child(col, toast_row);

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

    // Component-level layout primitive demo: resizable panel group with a thick hit target and
    // hairline divider, using a `Model<f32>` for persistence.
    let split_sep = ui.create_node(Separator::horizontal());
    ui.add_child(col, split_sep);
    let split_title = ui.create_node(Text::new("Resizable split (prototype)"));
    ui.add_child(col, split_title);

    let split_fraction = app.models_mut().insert(0.5f32);
    let split = ui.create_node(
        ResizablePanelGroup::horizontal(split_fraction)
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
        app.with_global_mut(DialogService::default, |_svc, _app| {});
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

        let overlays = WindowOverlays::install(&mut ui);

        build_ui_kit_contents(
            app,
            &mut ui,
            scroll,
            self.ui_kit_image.as_ref().map(|i| i.id),
            overlays.command_palette_node(),
        );

        UiKitWindowState { ui, root, overlays }
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
            "ui_kit.action.one" => {
                tracing::info!("action one");
            }
            "ui_kit.action.two" => {
                tracing::info!("action two");
            }
            "ui_kit.action.three" => {
                tracing::info!("action three");
            }
            "ui_kit.dialog.open" => {
                let request = DialogRequest {
                    owner: state.root,
                    title: Arc::from("Delete 12 files?"),
                    message: Arc::from(
                        "This action cannot be undone.\n\nAre you sure you want to continue?",
                    ),
                    actions: vec![
                        DialogAction::new(
                            "Cancel",
                            fret_app::CommandId::from("ui_kit.dialog.cancelled"),
                        ),
                        DialogAction::new(
                            "Delete",
                            fret_app::CommandId::from("ui_kit.dialog.delete_confirmed"),
                        ),
                    ],
                    default_action: Some(1),
                    cancel_command: Some(fret_app::CommandId::from("ui_kit.dialog.cancelled")),
                };

                app.with_global_mut(DialogService::default, |service, _app| {
                    service.set_request(window, request);
                });
                app.push_effect(Effect::Command {
                    window: Some(window),
                    command: fret_app::CommandId::from("dialog.open"),
                });
            }
            "ui_kit.dialog.delete_confirmed" => {
                tracing::info!("dialog confirmed");
            }
            "ui_kit.dialog.cancelled" => {
                tracing::info!("dialog cancelled");
            }
            "ui_kit.toast.success" => {
                sonner::toast_success(app, window, "Build completed");
            }
            "ui_kit.toast.error" => {
                sonner::toast_error(app, window, "Build failed");
            }
            "ui_kit.toast.action" => {
                sonner::toast_action(
                    app,
                    window,
                    "New update available",
                    "Restart to apply changes.",
                    ToastAction::new("Restart", fret_app::CommandId::from("ui_kit.toast.restart")),
                );
            }
            "ui_kit.toast.restart" => {
                tracing::info!("toast action: restart");
            }
            _ => {}
        }

        if state
            .overlays
            .handle_command(app, &mut state.ui, text, window, &command)
        {
            return;
        }

        tracing::info!(window = ?window, command = ?command, "unhandled command");
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
    app.set_global(KeymapService::default());
    app.with_global_mut(KeymapService::default, |svc, _app| {
        // Cmd/Ctrl+K to toggle the command palette (shadcn/cmdk convention).
        let file = KeymapFileV1 {
            keymap_version: 1,
            bindings: vec![
                BindingV1 {
                    command: Some("command_palette.toggle".to_string()),
                    platform: Some("macos".to_string()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["meta".to_string()],
                        key: "KeyK".to_string(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".to_string()),
                    platform: Some("windows".to_string()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".to_string()],
                        key: "KeyK".to_string(),
                    },
                },
                BindingV1 {
                    command: Some("command_palette.toggle".to_string()),
                    platform: Some("linux".to_string()),
                    when: None,
                    keys: KeySpecV1 {
                        mods: vec!["ctrl".to_string()],
                        key: "KeyK".to_string(),
                    },
                },
            ],
        };

        svc.keymap = Keymap::from_v1(file).unwrap_or_else(|_| Keymap::empty());
    });
    let driver = UiKitDriver::default();
    let mut runner = WinitRunner::new(config, app, driver);
    event_loop.run_app(&mut runner)?;
    Ok(())
}
