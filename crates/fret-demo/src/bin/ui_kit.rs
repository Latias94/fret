use fret_app::{
    App, BindingV1, Effect, KeySpecV1, Keymap, KeymapFileV1, KeymapService, Menu, MenuItem, Model,
    WindowRequest,
};
use fret_components_icons::IconId;
use fret_components_shadcn::{
    Accordion as ShadcnAccordion, AccordionContent as ShadcnAccordionContent,
    AccordionItem as ShadcnAccordionItem, AccordionTrigger as ShadcnAccordionTrigger,
    AspectRatio as ShadcnAspectRatio, Avatar as ShadcnAvatar,
    AvatarFallback as ShadcnAvatarFallback, AvatarImage as ShadcnAvatarImage,
    Breadcrumb as ShadcnBreadcrumb, BreadcrumbItem as ShadcnBreadcrumbItem, Button as ShadcnButton,
    ButtonGroup as ShadcnButtonGroup, ButtonGroupItem as ShadcnButtonGroupItem,
    ButtonGroupOrientation as ShadcnButtonGroupOrientation, ButtonSize as ShadcnButtonSize,
    ButtonVariant as ShadcnButtonVariant, Collapsible as ShadcnCollapsible,
    CollapsibleContent as ShadcnCollapsibleContent, CollapsibleTrigger as ShadcnCollapsibleTrigger,
    HoverCard as ShadcnHoverCard, HoverCardContent as ShadcnHoverCardContent,
    HoverCardTrigger as ShadcnHoverCardTrigger, InputGroup as ShadcnInputGroup,
    RadioGroup as ShadcnRadioGroup, RadioGroupItem as ShadcnRadioGroupItem,
    Skeleton as ShadcnSkeleton, Spinner as ShadcnSpinner, ToggleGroup as ShadcnToggleGroup,
    ToggleGroupItem as ShadcnToggleGroupItem, ToggleSize as ShadcnToggleSize,
    ToggleVariant as ShadcnToggleVariant,
};
use fret_components_ui::{
    ChromeRefinement, ContextMenuService, DialogAction, DialogRequest, DialogService,
    PopoverService, Size as ComponentSize, StyledExt as _, ToastAction, TooltipService,
    WindowOverlays,
    button::{Button, ButtonIntent, ButtonVariant},
    checkbox::Checkbox,
    combobox::Combobox,
    command::CommandItem,
    command_palette::{
        CommandPaletteHandles, install_command_palette, render_command_palette_list,
    },
    dropdown_menu::DropdownMenuButton,
    frame::Frame,
    icon_button::IconButton,
    progress::ProgressBar,
    resizable_panel_group::ResizablePanelGroup,
    scroll_area::ScrollArea,
    select::{Select, SelectOption},
    separator::Separator,
    slider::Slider,
    sonner,
    switch::Switch,
    tabs::Tabs,
    text_area_field::TextAreaField,
    text_field::TextField,
    toolbar::Toolbar,
    tooltip::TooltipArea,
};
use fret_core::{
    AppWindowId, Color, NodeId, PlatformCapabilities, Point, Px, Rect, Scene, Size, TextService,
};
use fret_render::{ImageColorSpace, ImageDescriptor, Renderer, WgpuContext};
use fret_runner_winit_wgpu::{WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig};
use fret_ui_app::{
    ColoredPanel, Column, FixedPanel, Invalidation, PanelThemeBackground, Row, Scroll, Stack, Text,
    Theme, ThemeConfig, UiTree,
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
    command_palette: CommandPaletteHandles,
    theme_candidates: Vec<ThemeCandidate>,
    theme_selected: Model<usize>,
    theme_last_selected: usize,
    declarative_mount: NodeId,
    declarative_root: Option<NodeId>,
    declarative_bounds: Rect,
    declarative_text: Model<String>,
    declarative_selection: Model<Option<usize>>,
    declarative_items: Model<Vec<String>>,
}

#[derive(Debug, Clone)]
struct ThemeCandidate {
    path: String,
    name: String,
}

fn load_theme_candidate(path: &str) -> Option<ThemeCandidate> {
    let bytes = std::fs::read(path).ok()?;
    let cfg = ThemeConfig::from_slice(&bytes).ok()?;
    Some(ThemeCandidate {
        path: path.to_string(),
        name: cfg.name,
    })
}

fn discover_theme_candidates() -> Vec<ThemeCandidate> {
    let mut out = Vec::new();

    if let Some(candidate) = load_theme_candidate("./.fret/theme.json") {
        out.push(candidate);
    }

    let Ok(dir) = std::fs::read_dir("./themes") else {
        return out;
    };

    let mut theme_paths: Vec<String> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    theme_paths.sort();

    for path in theme_paths {
        if let Some(candidate) = load_theme_candidate(&path) {
            out.push(candidate);
        }
    }

    out
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
    theme_selected: Model<usize>,
    theme_options: Vec<SelectOption>,
) -> (NodeId, Model<Vec<CommandItem>>) {
    let col = ui.create_node(Column::new().with_padding(Px(16.0)).with_spacing(Px(12.0)));
    ui.add_child(parent, col);

    let title = ui.create_node(Text::new("UI Kit"));
    ui.add_child(col, title);

    let subtitle = ui.create_node(Text::new(
        "Token-driven primitives (shadcn-inspired) + retained UI runtime.",
    ));
    ui.add_child(col, subtitle);

    let theme_title = ui.create_node(Text::new("Theme (hot reload)"));
    ui.add_child(col, theme_title);
    let theme_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let theme_label = ui.create_node(Text::new("Theme"));
    let theme_select = ui.create_node(
        Select::new(theme_selected, theme_options)
            .with_size(ComponentSize::Small)
            .placeholder("Select theme..."),
    );
    ui.add_child(theme_row, theme_label);
    ui.add_child(theme_row, theme_select);
    ui.add_child(col, theme_row);

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

    let textarea_model = app.models_mut().insert(
        "Multiline text area (shadcn-inspired)\n\n- Uses BoundTextArea + component chrome\n- Token/size-driven padding and text\n"
            .to_string(),
    );
    let textarea = ui.create_node(
        TextAreaField::new(textarea_model)
            .with_size(ComponentSize::Medium)
            .with_min_height(Px(140.0))
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
    );
    ui.add_child(col, textarea);

    let buttons_frame = ui.create_node(
        Frame::default()
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
    );
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

    let shadcn_buttons_title = ui.create_node(Text::new("shadcn/ui v4 Button (prototype)"));
    ui.add_child(col, shadcn_buttons_title);
    let shadcn_buttons_frame = ui.create_node(
        Frame::default()
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
    );
    let shadcn_buttons = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let shadcn_default = ui.create_node(
        ShadcnButton::new("Default")
            .variant(ShadcnButtonVariant::Default)
            .size(ShadcnButtonSize::Default),
    );
    let shadcn_destructive = ui.create_node(
        ShadcnButton::new("Destructive")
            .variant(ShadcnButtonVariant::Destructive)
            .size(ShadcnButtonSize::Default),
    );
    let shadcn_outline = ui.create_node(
        ShadcnButton::new("Outline")
            .variant(ShadcnButtonVariant::Outline)
            .size(ShadcnButtonSize::Default),
    );
    let shadcn_secondary = ui.create_node(
        ShadcnButton::new("Secondary")
            .variant(ShadcnButtonVariant::Secondary)
            .size(ShadcnButtonSize::Default),
    );
    let shadcn_ghost = ui.create_node(
        ShadcnButton::new("Ghost")
            .variant(ShadcnButtonVariant::Ghost)
            .size(ShadcnButtonSize::Default),
    );
    let shadcn_link = ui.create_node(
        ShadcnButton::new("Link")
            .variant(ShadcnButtonVariant::Link)
            .size(ShadcnButtonSize::Default),
    );
    ui.add_child(shadcn_buttons, shadcn_default);
    ui.add_child(shadcn_buttons, shadcn_destructive);
    ui.add_child(shadcn_buttons, shadcn_outline);
    ui.add_child(shadcn_buttons, shadcn_secondary);
    ui.add_child(shadcn_buttons, shadcn_ghost);
    ui.add_child(shadcn_buttons, shadcn_link);
    ui.add_child(shadcn_buttons_frame, shadcn_buttons);
    ui.add_child(col, shadcn_buttons_frame);

    let shadcn_button_group_label =
        ui.create_node(Text::new("shadcn/ui v4 ButtonGroup (prototype)"));
    ui.add_child(col, shadcn_button_group_label);
    let shadcn_button_group_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let shadcn_button_group = ui.create_node(
        ShadcnButtonGroup::new()
            .orientation(ShadcnButtonGroupOrientation::Horizontal)
            .item(
                ShadcnButtonGroupItem::new("Left")
                    .variant(ShadcnButtonVariant::Outline)
                    .on_click("ui_kit.button_group.left"),
            )
            .item(
                ShadcnButtonGroupItem::new("Middle")
                    .variant(ShadcnButtonVariant::Outline)
                    .on_click("ui_kit.button_group.middle"),
            )
            .item(
                ShadcnButtonGroupItem::new("Right")
                    .variant(ShadcnButtonVariant::Outline)
                    .on_click("ui_kit.button_group.right"),
            ),
    );
    let shadcn_button_group_vertical = ui.create_node(
        ShadcnButtonGroup::new()
            .orientation(ShadcnButtonGroupOrientation::Vertical)
            .item(
                ShadcnButtonGroupItem::new("Top")
                    .variant(ShadcnButtonVariant::Outline)
                    .on_click("ui_kit.button_group.top"),
            )
            .item(
                ShadcnButtonGroupItem::new("Center")
                    .variant(ShadcnButtonVariant::Outline)
                    .disabled(true),
            )
            .item(
                ShadcnButtonGroupItem::new("Bottom")
                    .variant(ShadcnButtonVariant::Outline)
                    .on_click("ui_kit.button_group.bottom"),
            ),
    );
    ui.add_child(shadcn_button_group_row, shadcn_button_group);
    ui.add_child(shadcn_button_group_row, shadcn_button_group_vertical);
    ui.add_child(col, shadcn_button_group_row);

    let dialogs_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let open_dialog = ui.create_node(Button::new("Open Dialog").on_click("ui_kit.dialog.open"));
    let open_alert_dialog =
        ui.create_node(Button::new("Open AlertDialog").on_click("ui_kit.alert_dialog.open"));
    ui.add_child(dialogs_row, open_dialog);
    ui.add_child(dialogs_row, open_alert_dialog);
    ui.add_child(col, dialogs_row);

    let text_model = app.models_mut().insert("Hello, components.".to_string());
    let text_field = ui.create_node(
        TextField::new(text_model)
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
    );
    ui.add_child(col, text_field);

    let input_group_label = ui.create_node(Text::new("shadcn/ui v4 InputGroup (prototype)"));
    ui.add_child(col, input_group_label);
    let input_group_model = app.models_mut().insert("Search...".to_string());
    let input_group = ui.create_node(
        ShadcnInputGroup::new(input_group_model)
            .leading_icon(IconId::new("search"))
            .trailing_icon(IconId::new("close")),
    );
    ui.add_child(col, input_group);

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

    let toggle_group_label = ui.create_node(Text::new("shadcn/ui v4 ToggleGroup (prototype)"));
    ui.add_child(col, toggle_group_label);
    let toggle_group_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    let toggle_single = app.models_mut().insert(None::<Arc<str>>);
    let toggle_group_single = ui.create_node(
        ShadcnToggleGroup::single(toggle_single)
            .variant(ShadcnToggleVariant::Outline)
            .size(ShadcnToggleSize::Default)
            .item(ShadcnToggleGroupItem::new("bold", "Bold"))
            .item(ShadcnToggleGroupItem::new("italic", "Italic"))
            .item(ShadcnToggleGroupItem::new("underline", "Underline")),
    );
    ui.add_child(toggle_group_row, toggle_group_single);
    ui.add_child(col, toggle_group_row);

    let radio_group_label = ui.create_node(Text::new("shadcn/ui v4 RadioGroup (prototype)"));
    ui.add_child(col, radio_group_label);
    let radio_model = app.models_mut().insert(Some(Arc::<str>::from("default")));
    let radio_group = ui.create_node(
        ShadcnRadioGroup::new(radio_model)
            .item(ShadcnRadioGroupItem::new("default", "Default"))
            .item(ShadcnRadioGroupItem::new("comfortable", "Comfortable"))
            .item(ShadcnRadioGroupItem::new("compact", "Compact")),
    );
    ui.add_child(col, radio_group);

    let collapsible_label = ui.create_node(Text::new("shadcn/ui v4 Collapsible (prototype)"));
    ui.add_child(col, collapsible_label);
    let collapsible_open = app.models_mut().insert(false);
    let collapsible = ui.create_node(ShadcnCollapsible::new().with_spacing(Px(6.0)));
    ui.add_child(col, collapsible);
    let collapsible_trigger = ui.create_node(ShadcnCollapsibleTrigger::new(collapsible_open));
    let collapsible_trigger_row = ui.create_node(Row::new().with_spacing(Px(8.0)));
    let collapsible_trigger_text = ui.create_node(Text::new("Toggle details"));
    let collapsible_trigger_chevron = ui.create_node(Text::new("▾"));
    ui.add_child(collapsible_trigger_row, collapsible_trigger_text);
    ui.add_child(collapsible_trigger_row, collapsible_trigger_chevron);
    ui.add_child(collapsible_trigger, collapsible_trigger_row);
    ui.add_child(collapsible, collapsible_trigger);
    let collapsible_content = ui.create_node(ShadcnCollapsibleContent::new(collapsible_open));
    let collapsible_content_frame = ui.create_node(
        Frame::default()
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
    );
    let collapsible_content_text = ui.create_node(Text::new("Hidden content (placeholder)."));
    ui.add_child(collapsible_content_frame, collapsible_content_text);
    ui.add_child(collapsible_content, collapsible_content_frame);
    ui.add_child(collapsible, collapsible_content);

    let accordion_label = ui.create_node(Text::new("shadcn/ui v4 Accordion (prototype)"));
    ui.add_child(col, accordion_label);
    let accordion_value = app.models_mut().insert(None::<Arc<str>>);
    let accordion = ui.create_node(ShadcnAccordion::new().with_spacing(Px(6.0)));
    ui.add_child(col, accordion);

    for (value, title) in [("a", "Section A"), ("b", "Section B"), ("c", "Section C")] {
        let item = ui.create_node(ShadcnAccordionItem::new().with_spacing(Px(4.0)));
        ui.add_child(accordion, item);

        let trigger = ui.create_node(ShadcnAccordionTrigger::single(accordion_value, value));
        let trigger_row = ui.create_node(Row::new().with_spacing(Px(8.0)));
        let trigger_title = ui.create_node(Text::new(title));
        let trigger_chevron = ui.create_node(Text::new("▾"));
        ui.add_child(trigger_row, trigger_title);
        ui.add_child(trigger_row, trigger_chevron);
        ui.add_child(trigger, trigger_row);
        ui.add_child(item, trigger);

        let content = ui.create_node(ShadcnAccordionContent::single(accordion_value, value));
        let content_frame = ui.create_node(
            Frame::default()
                .styled()
                .rounded_md()
                .border_1()
                .px_3()
                .py_2()
                .finish(),
        );
        let content_text = ui.create_node(Text::new(format!("Content for {title}")));
        ui.add_child(content_frame, content_text);
        ui.add_child(content, content_frame);
        ui.add_child(item, content);
    }

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
        CommandItem::new("ui_kit.alert_dialog.open", "Open AlertDialog")
            .group("Overlays")
            .keyword("alert-dialog")
            .keyword("dialog")
            .detail("Show an alert-style dialog (cancel closes silently)"),
        CommandItem::new("command_palette.open", "Open Command Palette")
            .group("Overlays")
            .keyword("palette")
            .keyword("command"),
        CommandItem::new("disabled.example", "Disabled item").disabled(),
    ]);

    let palette_open_row = ui.create_node(Row::new().with_spacing(Px(10.0)));
    ui.add_child(col, palette_open_row);
    let open_palette = ui.create_node(
        Button::new("Open Command Palette")
            .on_click(fret_app::CommandId::from("command_palette.open")),
    );
    ui.add_child(palette_open_row, open_palette);

    let scroll_area_label = ui.create_node(Text::new("ScrollArea"));
    ui.add_child(col, scroll_area_label);

    let scroll_area_panel = ui.create_node(FixedPanel::new(Px(160.0), Color::TRANSPARENT));
    ui.add_child(col, scroll_area_panel);

    let scroll_area = ui.create_node(ScrollArea::new().styled().rounded_md().border_1().finish());
    ui.add_child(scroll_area_panel, scroll_area);

    let scroll_area_content =
        ui.create_node(Column::new().with_padding(Px(12.0)).with_spacing(Px(6.0)));
    ui.add_child(scroll_area, scroll_area_content);
    for i in 0..8 {
        let row = ui.create_node(Text::new(format!("ScrollArea row {}", i + 1)));
        ui.add_child(scroll_area_content, row);
    }

    let separator = ui.create_node(Separator::horizontal());
    ui.add_child(col, separator);

    let icons_frame = ui.create_node(Frame::new(
        ChromeRefinement::default()
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
        DropdownMenuButton::new("DropdownMenu", dropdown_menu)
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
    );
    ui.add_child(col, dropdown);

    if let Some(img) = image {
        let image_frame = ui.create_node(Frame::new(
            ChromeRefinement::default()
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
        Toolbar::new()
            .styled()
            .rounded_md()
            .border_1()
            .px_3()
            .py_2()
            .finish(),
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

    let decl_title = ui.create_node(Text::new("Declarative (experimental)"));
    ui.add_child(col, decl_title);

    let decl_panel = ui.create_node(FixedPanel::new(Px(220.0), Color::TRANSPARENT));
    ui.add_child(col, decl_panel);

    let decl_mount = ui.create_node(Stack::new());
    ui.add_child(decl_panel, decl_mount);

    (decl_mount, command_items)
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

        let theme_candidates = discover_theme_candidates();
        let current_theme_name = Theme::global(app).name.clone();
        let initial_theme_index = theme_candidates
            .iter()
            .position(|c| c.name == current_theme_name)
            .unwrap_or(0);
        let theme_selected = app.models_mut().insert(initial_theme_index);
        let theme_last_selected = initial_theme_index;
        let theme_options = if theme_candidates.is_empty() {
            vec![SelectOption::new("No themes found").disabled()]
        } else {
            theme_candidates
                .iter()
                .map(|c| SelectOption::new(c.name.clone()))
                .collect()
        };

        let (declarative_mount, command_items) = build_ui_kit_contents(
            app,
            &mut ui,
            scroll,
            self.ui_kit_image.as_ref().map(|i| i.id),
            theme_selected,
            theme_options,
        );

        let command_palette = install_command_palette(
            &mut ui,
            app,
            overlays.command_palette_node(),
            command_items,
            ComponentSize::Medium,
        );

        let declarative_selection = app.models_mut().insert(None::<usize>);
        let declarative_items = app.models_mut().insert(
            (0..200usize)
                .map(|i| format!("Project {i}"))
                .collect::<Vec<_>>(),
        );
        let declarative_text = app.models_mut().insert("".to_string());

        UiKitWindowState {
            ui,
            root,
            overlays,
            command_palette,
            theme_candidates,
            theme_selected,
            theme_last_selected,
            declarative_mount,
            declarative_root: None,
            declarative_bounds: Rect::default(),
            declarative_text,
            declarative_selection,
            declarative_items,
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
        if state
            .overlays
            .handle_command(app, &mut state.ui, text, window, &command)
        {
            return;
        }

        if state.ui.dispatch_command(app, text, &command) {
            return;
        }

        if let Some(index) = command
            .as_str()
            .strip_prefix("ui_kit.declarative_list.select.")
            .and_then(|s| s.parse::<usize>().ok())
        {
            let _ = app
                .models_mut()
                .update(state.declarative_selection, |v| *v = Some(index));
            app.request_redraw(window);
            return;
        }

        match command.as_str() {
            "ui_kit.declarative_text.clear" => {
                let _ = app
                    .models_mut()
                    .update(state.declarative_text, |v| v.clear());
                app.request_redraw(window);
            }
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
            "ui_kit.alert_dialog.open" => {
                let request = DialogRequest {
                    owner: state.root,
                    title: Arc::from("Delete project?"),
                    message: Arc::from(
                        "This action cannot be undone.\n\nAre you sure you want to continue?",
                    ),
                    actions: vec![
                        DialogAction::cancel("Cancel"),
                        DialogAction::new(
                            "Continue",
                            fret_app::CommandId::from("ui_kit.alert_dialog.confirmed"),
                        ),
                    ],
                    default_action: Some(1),
                    cancel_command: None,
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
            "ui_kit.alert_dialog.confirmed" => {
                tracing::info!("alert dialog confirmed");
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
        window: AppWindowId,
        state: &mut Self::WindowState,
        changed: &[fret_app::ModelId],
    ) {
        state.ui.propagate_model_changes(app, changed);

        if !changed.contains(&state.theme_selected.id()) {
            return;
        }

        let selected = state
            .theme_selected
            .get(&*app)
            .copied()
            .unwrap_or(state.theme_last_selected);
        if selected == state.theme_last_selected {
            return;
        }

        state.theme_last_selected = selected;
        let Some(candidate) = state.theme_candidates.get(selected) else {
            return;
        };

        let bytes = match std::fs::read(&candidate.path) {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::error!(error = %err, path = %candidate.path, "failed to read theme file");
                sonner::toast_error(app, window, "Failed to read theme file");
                return;
            }
        };

        match ThemeConfig::from_slice(&bytes) {
            Ok(cfg) => {
                Theme::global_mut(app).apply_config(&cfg);
                tracing::info!(theme = %cfg.name, path = %candidate.path, "loaded theme");
                state.ui.invalidate(state.root, Invalidation::Layout);
                state.ui.invalidate(state.root, Invalidation::Paint);
                app.request_redraw(window);
            }
            Err(err) => {
                tracing::error!(error = %err, path = %candidate.path, "failed to parse theme file");
                sonner::toast_error(app, window, "Failed to parse theme file");
            }
        }
    }

    fn render(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        text: &mut dyn fret_core::TextService,
        scene: &mut Scene,
    ) {
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();

        let desired_bounds = if state.declarative_bounds.size.width.0 > 0.0
            && state.declarative_bounds.size.height.0 > 0.0
        {
            state.declarative_bounds
        } else {
            Rect::new(
                Point::new(Px(0.0), Px(0.0)),
                Size::new(bounds.size.width, Px(220.0)),
            )
        };

        let declarative_image = self.ui_kit_image.as_ref().map(|i| i.id);

        let root = fret_ui_app::declarative::render_root(
            &mut state.ui,
            app,
            text,
            window,
            desired_bounds,
            "ui-kit-declarative-list",
            |cx| {
                cx.observe_model(state.declarative_items, Invalidation::Layout);
                let values = cx
                    .app
                    .models()
                    .get(state.declarative_items)
                    .cloned()
                    .unwrap_or_default();

                let size = fret_components_ui::Size::Medium;
                let (
                    theme_snapshot,
                    base_row_h,
                    outer_gap,
                    secondary_gap,
                    focus_ring,
                    shadow,
                    radius,
                ) = {
                    let theme = Theme::global(&*cx.app);
                    let theme_snapshot = theme.snapshot();
                    let base_row_h = size.list_row_h(theme);
                    let outer_gap = fret_components_ui::declarative::style::space(
                        theme,
                        fret_components_ui::Space::N2,
                    );
                    let secondary_gap = fret_components_ui::declarative::style::space(
                        theme,
                        fret_components_ui::Space::N0p5,
                    );
                    let radius = fret_components_ui::declarative::style::radius(
                        theme,
                        fret_components_ui::Radius::Md,
                    );
                    let focus_ring =
                        fret_components_ui::declarative::style::focus_ring(theme, radius);
                    let shadow = fret_components_ui::declarative::style::shadow_md(theme, radius);
                    (
                        theme_snapshot,
                        base_row_h,
                        outer_gap,
                        secondary_gap,
                        focus_ring,
                        shadow,
                        radius,
                    )
                };

                vec![cx.column(
                    fret_ui_app::element::ColumnProps {
                        gap: outer_gap,
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            cx.text("Recents (declarative virtualized list)"),
                            cx.pressable(
                                fret_ui_app::element::PressableProps {
                                    focus_ring: Some(focus_ring),
                                    ..Default::default()
                                },
                                |cx, st| {
                                    let bg = if st.pressed {
                                        Some(theme_snapshot.colors.selection_background)
                                    } else if st.hovered {
                                        Some(theme_snapshot.colors.hover_background)
                                    } else {
                                        Some(theme_snapshot.colors.panel_background)
                                    };

                                    vec![cx.container(
                                        fret_ui_app::element::ContainerProps {
                                            padding: fret_core::Edges::all(outer_gap),
                                            background: bg,
                                            shadow: Some(shadow),
                                            border: fret_core::Edges::all(fret_core::Px(1.0)),
                                            border_color: Some(theme_snapshot.colors.panel_border),
                                            corner_radii: fret_core::Corners::all(radius),
                                            ..Default::default()
                                        },
                                        |cx| vec![cx.text("Focus ring demo (click to focus)")],
                                    )]
                                },
                            ),
                            cx.text("TextField (declarative, absolute icon+clear)"),
                            fret_components_ui::declarative::text_field::text_field_with_leading_icon_and_clear(
                                cx,
                                state.declarative_text,
                                size,
                                IconId::new("search"),
                                fret_app::CommandId::from("ui_kit.declarative_text.clear"),
                            ),
                            cx.text("Truncate (ellipsis)"),
                            cx.container(
                                fret_ui_app::element::ContainerProps {
                                    layout: fret_ui_app::element::LayoutStyle {
                                        size: fret_ui_app::element::SizeStyle {
                                            width: fret_ui_app::element::Length::Px(Px(260.0)),
                                            ..Default::default()
                                        },
			                                        ..Default::default()
			                                    },
			                                    padding: fret_core::Edges::all(outer_gap),
			                                    background: Some(theme_snapshot.colors.panel_background),
			                                    shadow: None,
			                                    border: fret_core::Edges::all(fret_core::Px(1.0)),
			                                    border_color: Some(theme_snapshot.colors.panel_border),
		                                    corner_radii: fret_core::Corners::all(radius),
		                                },
		                                |cx| {
		                                    let mut p = fret_ui_app::element::TextProps::new(
		                                        "This is a very long line that should truncate with an ellipsis.",
		                                    );
		                                    p.layout.size.width = fret_ui_app::element::Length::Fill;
		                                    p.wrap = fret_core::TextWrap::None;
		                                    p.overflow = fret_core::TextOverflow::Ellipsis;
		                                    vec![cx.text_props(p)]
		                                },
		                            ),
                            cx.text("shadcn/ui v4 Breadcrumb (prototype)"),
                            ShadcnBreadcrumb::new()
                                .item(
                                    ShadcnBreadcrumbItem::new("Home")
                                        .on_click("ui_kit.breadcrumb.home"),
                                )
                                .item(
                                    ShadcnBreadcrumbItem::new("Components")
                                        .on_click("ui_kit.breadcrumb.components"),
                                )
                                .item(ShadcnBreadcrumbItem::ellipsis())
                                .item(ShadcnBreadcrumbItem::new("Breadcrumb"))
                                .into_element(cx),
                            cx.text("shadcn/ui v4 Skeleton (prototype)"),
                            cx.container(
                                fret_ui_app::element::ContainerProps {
                                    layout: fret_ui_app::element::LayoutStyle {
                                        size: fret_ui_app::element::SizeStyle {
                                            width: fret_ui_app::element::Length::Px(Px(260.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        ShadcnSkeleton::new()
                                            .refine_layout(
                                                fret_components_ui::LayoutRefinement::default()
                                                    .w_full(),
                                            )
                                            .into_element(cx),
                                        ShadcnSkeleton::new()
                                            .secondary()
                                            .refine_layout(
                                                fret_components_ui::LayoutRefinement::default()
                                                    .w_full()
                                                    .h_px(fret_components_ui::MetricRef::space(
                                                        fret_components_ui::Space::N2,
                                                    )),
                                            )
                                            .into_element(cx),
                                    ]
                                },
                            ),
                            cx.text("shadcn/ui v4 HoverCard (prototype)"),
                            {
                                let trigger = cx.container(
                                    fret_ui_app::element::ContainerProps {
                                        padding: fret_core::Edges::symmetric(Px(12.0), Px(8.0)),
                                        background: Some(theme_snapshot.colors.panel_background),
                                        border: fret_core::Edges::all(Px(1.0)),
                                        border_color: Some(theme_snapshot.colors.panel_border),
                                        corner_radii: fret_core::Corners::all(Px(8.0)),
                                        ..Default::default()
                                    },
                                    |cx| vec![cx.text("Hover me")],
                                );

                                let content = ShadcnHoverCardContent::new(vec![
                                    cx.text("This is hover card content."),
                                    cx.text("It is anchored to the trigger."),
                                    ShadcnSkeleton::new()
                                        .secondary()
                                        .refine_layout(
                                            fret_components_ui::LayoutRefinement::default()
                                                .w_full()
                                                .h_px(fret_components_ui::MetricRef::Px(Px(10.0))),
                                        )
                                        .into_element(cx),
                                ])
                                .into_element(cx);

                                ShadcnHoverCard::new(
                                    ShadcnHoverCardTrigger::new(trigger).into_element(cx),
                                    content,
                                )
                                .open_delay_frames(8)
                                .close_delay_frames(8)
                                .into_element(cx)
                            },
                            cx.text("shadcn/ui v4 Spinner (prototype)"),
                            cx.row(
                                fret_ui_app::element::RowProps {
                                    gap: Px(10.0),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        ShadcnSpinner::new().into_element(cx),
                                        cx.text("Loading..."),
                                        ShadcnSpinner::new()
                                            .refine_layout(
                                                fret_components_ui::LayoutRefinement::default()
                                                    .w_px(fret_components_ui::MetricRef::Px(
                                                        Px(24.0),
                                                    ))
                                                    .h_px(fret_components_ui::MetricRef::Px(
                                                        Px(24.0),
                                                    )),
                                            )
                                            .into_element(cx),
                                    ]
                                },
                            ),
		                            cx.text("Absolute badge (position/inset)"),
			                            cx.container(
			                                fret_ui_app::element::ContainerProps {
		                                    layout: fret_ui_app::element::LayoutStyle {
		                                        size: fret_ui_app::element::SizeStyle {
		                                            width: fret_ui_app::element::Length::Px(Px(260.0)),
		                                            ..Default::default()
		                                        },
		                                        ..Default::default()
		                                    },
		                                    padding: fret_core::Edges::all(outer_gap),
		                                    background: Some(theme_snapshot.colors.panel_background),
		                                    shadow: None,
		                                    border: fret_core::Edges::all(fret_core::Px(1.0)),
		                                    border_color: Some(theme_snapshot.colors.panel_border),
	                                    corner_radii: fret_core::Corners::all(radius),
	                                },
	                                |cx| {
	                                    let mut badge = fret_ui_app::element::ContainerProps::default();
		                                    badge.background =
		                                        Some(theme_snapshot.colors.accent);
		                                    badge.corner_radii =
		                                        fret_core::Corners::all(Px(999.0));
		                                    badge.padding = fret_core::Edges::symmetric(secondary_gap, fret_core::Px(0.0));
		                                    badge.layout.position =
		                                        fret_ui_app::element::PositionStyle::Absolute;
		                                    badge.layout.inset.top = Some(Px(0.0));
		                                    badge.layout.inset.right = Some(Px(0.0));

	                                    vec![
	                                        cx.text(
	                                            "A container can host absolute children (badge/icon overlays).",
	                                        ),
	                                        cx.container(badge, |cx| vec![cx.text("NEW")]),
	                                    ]
	                                },
	                            ),
	                            cx.text("Image"),
		                            cx.row(
		                                fret_ui_app::element::RowProps {
		                                    gap: outer_gap,
	                                    align: fret_ui_app::element::CrossAlign::Center,
	                                    ..Default::default()
	                                },
		                                |cx| {
		                                    let mut out = Vec::new();

                                    if let Some(img) = declarative_image {
                                        let mut p = fret_ui_app::element::ImageProps::new(img);
                                        p.layout.size.width =
                                            fret_ui_app::element::Length::Px(Px(160.0));
                                        p.layout.size.height =
                                            fret_ui_app::element::Length::Px(Px(120.0));
                                        out.push(cx.image_props(p));
                                    } else {
                                        out.push(cx.text("Image not ready yet"));
                                    }

                                    out.push(cx.text(
                                        "Declarative Image (SceneOp::Image/ImageRegion) with explicit size.",
                                    ));
		                                    out
		                                },
		                            ),
		                            cx.text("shadcn/ui v4 Avatar (prototype)"),
		                            cx.row(
		                                fret_ui_app::element::RowProps {
		                                    gap: outer_gap,
		                                    align: fret_ui_app::element::CrossAlign::Center,
		                                    ..Default::default()
		                                },
		                                |cx| {
		                                    let mut out = Vec::new();

		                                    let avatar = {
		                                        let mut layers =
		                                            vec![ShadcnAvatarFallback::new("FO").into_element(
		                                                cx,
		                                            )];
		                                        if let Some(img) = declarative_image {
		                                            layers.push(
		                                                ShadcnAvatarImage::new(img).into_element(cx),
		                                            );
		                                        }
		                                        ShadcnAvatar::new(layers).into_element(cx)
		                                    };
		                                    out.push(avatar);

		                                    let avatar_lg = {
		                                        let layers =
		                                            vec![ShadcnAvatarFallback::new("LG").into_element(
		                                                cx,
		                                            )];
		                                        ShadcnAvatar::new(layers)
		                                            .refine_layout(
		                                                fret_components_ui::LayoutRefinement::default()
		                                                    .w_px(
		                                                        fret_components_ui::MetricRef::space(
		                                                            fret_components_ui::Space::N10,
		                                                        ),
		                                                    )
		                                                    .h_px(
		                                                        fret_components_ui::MetricRef::space(
		                                                            fret_components_ui::Space::N10,
		                                                        ),
		                                                    ),
		                                            )
		                                            .into_element(cx)
		                                    };
		                                    out.push(avatar_lg);

		                                    out.push(cx.text(
		                                        "Declarative Avatar layers: fallback under image (if present).",
		                                    ));
		                                    out
		                                },
		                            ),
		                            cx.text("shadcn/ui v4 Aspect Ratio (prototype)"),
		                            cx.row(
		                                fret_ui_app::element::RowProps {
		                                    gap: outer_gap,
		                                    align: fret_ui_app::element::CrossAlign::Center,
		                                    ..Default::default()
		                                },
		                                |cx| {
		                                    let mut out = Vec::new();

		                                    if let Some(img) = declarative_image {
		                                        let mut p = fret_ui_app::element::ImageProps::new(img);
		                                        p.layout.position =
		                                            fret_ui_app::element::PositionStyle::Absolute;
		                                        p.layout.inset.top = Some(Px(0.0));
		                                        p.layout.inset.right = Some(Px(0.0));
		                                        p.layout.inset.bottom = Some(Px(0.0));
		                                        p.layout.inset.left = Some(Px(0.0));
		                                        p.layout.size.width =
		                                            fret_ui_app::element::Length::Fill;
		                                        p.layout.size.height =
		                                            fret_ui_app::element::Length::Fill;

		                                        let image = cx.image_props(p);
		                                        out.push(
		                                            ShadcnAspectRatio::new(16.0 / 9.0, image)
		                                                .refine_layout(
		                                                    fret_components_ui::LayoutRefinement::default()
		                                                        .w_px(
		                                                            fret_components_ui::MetricRef::Px(Px(220.0)),
		                                                        ),
		                                                )
		                                                .into_element(cx),
		                                        );
		                                    } else {
		                                        out.push(cx.text("Image not ready yet"));
		                                    }

		                                    out.push(cx.text(
		                                        "AspectRatio sets width=Fill + height=Auto by default (ADR 0057).",
		                                    ));
		                                    out
		                                },
		                            ),
		                            cx.text("Scroll"),
		                            cx.scroll(
		                                fret_ui_app::element::ScrollProps {
		                                    layout: fret_ui_app::element::LayoutStyle {
                                        size: fret_ui_app::element::SizeStyle {
                                            height: fret_ui_app::element::Length::Px(Px(72.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![cx.column(
                                        fret_ui_app::element::ColumnProps {
                                            gap: secondary_gap,
                                            align: fret_ui_app::element::CrossAlign::Start,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            (0..12)
                                                .map(|i| cx.text(format!("Scrollable line {i}")))
                                                .collect()
                                        },
                                    )]
                                },
                            ),
                            fret_components_ui::declarative::list::list_virtualized(
                                cx,
                                Some(state.declarative_selection),
                                size,
                                Some(Px(base_row_h.0 * 1.9)),
                                values.len(),
                                2,
                                None,
                                |i| values.get(i).map(String::as_str).unwrap_or(""),
                                |i| {
                                    Some(fret_app::CommandId::new(format!(
                                        "ui_kit.declarative_list.select.{i}"
                                    )))
                                },
                                |cx, i| {
                                    let label = values.get(i).map(String::as_str).unwrap_or("");
                                    let leading_icon = if i % 3 == 0 { "play" } else { "settings" };
                                    let trailing_icon =
                                        if i % 5 == 0 { Some("chevron_down") } else { None };

                                    let mut out = Vec::new();
                                    out.push(fret_components_ui::declarative::icon::icon(
                                        cx,
                                        IconId::new(leading_icon),
                                    ));
                                    out.push(cx.column(
                                        fret_ui_app::element::ColumnProps {
                                            gap: secondary_gap,
                                            align: fret_ui_app::element::CrossAlign::Start,
                                            ..Default::default()
                                        },
                                        |cx| vec![cx.text(label), cx.text("Modified 2 hours ago")],
                                    ));
                                    out.push(cx.spacer(fret_ui_app::element::SpacerProps {
                                        min: Px(0.0),
                                        ..Default::default()
                                    }));
                                    if let Some(icon) = trailing_icon {
                                        out.push(fret_components_ui::declarative::icon::icon(
                                            cx,
                                            IconId::new(icon),
                                        ));
                                    }
                                    out
                                },
                            ),
                        ]
                    },
                )]
            },
        );

        state.ui.set_children(state.declarative_mount, vec![root]);
        state.declarative_root = Some(root);

        // Render the command palette list via declarative composition (composable rows).
        render_command_palette_list(
            &mut state.ui,
            app,
            text,
            window,
            &state.command_palette,
            ComponentSize::Medium,
        );

        state.ui.layout_all(app, text, bounds, scale_factor);
        if let Some(root) = state.declarative_root
            && let Some(b) = state.ui.debug_node_bounds(root)
        {
            state.declarative_bounds = b;
        }
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
