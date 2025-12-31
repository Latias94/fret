use anyhow::Context as _;
use fret_app::{App, CommandId, Effect, Model, WindowRequest};
use fret_components_app::tree::AppTreeRowRenderer;
use fret_components_icons::IconRegistry;
use fret_components_shadcn as shadcn;
use fret_components_ui::tree::{TreeItem, TreeItemId, TreeState};
use fret_core::{
    AppWindowId, Edges, Event, KeyCode, PlatformCapabilities, Px, Rect, Scene, SemanticsRole,
    UiServices,
};
use fret_runner_winit_wgpu::{
    RunnerUserEvent, WindowCreateSpec, WinitDriver, WinitRunner, WinitRunnerConfig,
};
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
};
use fret_ui::{Invalidation, Theme, UiTree};
use std::sync::Arc;
use winit::event_loop::EventLoop;

struct ComponentsGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    items: Model<Vec<TreeItem>>,
    tree_state: Model<TreeState>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    radio: Model<Option<Arc<str>>>,
    select: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
}

#[derive(Default)]
struct ComponentsGalleryDriver;

impl ComponentsGalleryDriver {
    fn sample_tree_items() -> Vec<TreeItem> {
        vec![
            TreeItem::new(1, "src")
                .child(TreeItem::new(2, "components").child(TreeItem::new(3, "tree.rs")))
                .child(TreeItem::new(4, "theme.rs"))
                .child(TreeItem::new(5, "virtual_list.rs")),
            TreeItem::new(10, "crates")
                .child(TreeItem::new(11, "fret-ui"))
                .child(TreeItem::new(12, "fret-components-ui"))
                .child(TreeItem::new(13, "fret-demo").disabled(true)),
            TreeItem::new(20, "docs").child(TreeItem::new(21, "adr")),
        ]
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> ComponentsGalleryWindowState {
        let items = app.models_mut().insert(Self::sample_tree_items());

        let initial_state = TreeState {
            selected: Some(1),
            expanded: [1, 10, 20].into_iter().collect(),
        };
        let tree_state = app.models_mut().insert(initial_state);
        let progress = app.models_mut().insert(35.0f32);
        let checkbox = app.models_mut().insert(false);
        let switch = app.models_mut().insert(true);
        let radio = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("a")));
        let select = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("apple")));
        let select_open = app.models_mut().insert(false);
        let dropdown_open = app.models_mut().insert(false);
        let context_menu_open = app.models_mut().insert(false);
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        let alert_dialog_open = app.models_mut().insert(false);
        let sheet_open = app.models_mut().insert(false);
        let cmdk_open = app.models_mut().insert(false);
        let cmdk_query = app.models_mut().insert(String::new());
        let last_action = app.models_mut().insert(Arc::<str>::from("<none>"));

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        ComponentsGalleryWindowState {
            ui,
            root: None,
            items,
            tree_state,
            progress,
            checkbox,
            switch,
            radio,
            select,
            select_open,
            dropdown_open,
            context_menu_open,
            popover_open,
            dialog_open,
            alert_dialog_open,
            sheet_open,
            cmdk_open,
            cmdk_query,
            last_action,
        }
    }

    fn render_gallery(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut ComponentsGalleryWindowState,
        bounds: Rect,
    ) {
        fret_components_ui::window_overlays::begin_frame(app, window);

        let items = state.items.clone();
        let tree_state = state.tree_state.clone();
        let progress = state.progress.clone();
        let checkbox = state.checkbox.clone();
        let switch = state.switch.clone();
        let radio = state.radio.clone();
        let select = state.select.clone();
        let select_open = state.select_open.clone();
        let dropdown_open = state.dropdown_open.clone();
        let context_menu_open = state.context_menu_open.clone();
        let popover_open = state.popover_open.clone();
        let dialog_open = state.dialog_open.clone();
        let alert_dialog_open = state.alert_dialog_open.clone();
        let sheet_open = state.sheet_open.clone();
        let cmdk_open = state.cmdk_open.clone();
        let cmdk_query = state.cmdk_query.clone();
        let last_action = state.last_action.clone();

        let root = declarative::render_root(
            &mut state.ui,
            app,
            services,
            window,
            bounds,
            "components-gallery",
            |cx| {
                cx.observe_model(&tree_state, Invalidation::Layout);
                let theme = Theme::global(&*cx.app).clone();
                let selected = cx
                    .app
                    .models()
                    .read(&tree_state, |s| s.selected)
                    .ok()
                    .flatten();

                let title: Arc<str> = Arc::from("components_gallery");
                let subtitle: Arc<str> = Arc::from(format!(
                    "Tree MVP (driver-owned): arrows navigate, left/right collapses, click selects. Selected: {}",
                    selected
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| "<none>".to_string())
                ));

                let mut root_layout = LayoutStyle::default();
                root_layout.size.width = Length::Fill;
                root_layout.size.height = Length::Fill;

                let mut tree_slot_layout = LayoutStyle::default();
                tree_slot_layout.size.width = Length::Fill;
                tree_slot_layout.size.height = Length::Fill;
                tree_slot_layout.flex.grow = 1.0;
                tree_slot_layout.flex.basis = Length::Px(Px(0.0));

                let padding = theme.metrics.padding_md;
                let bg = theme.colors.surface_background;

                vec![cx.container(
                    ContainerProps {
                        layout: root_layout,
                        background: Some(bg),
                        ..Default::default()
                    },
                    |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: root_layout,
                                direction: fret_core::Axis::Vertical,
                                gap: Px(12.0),
                                padding: Edges::all(padding),
                                justify: MainAlign::Start,
                                align: CrossAlign::Stretch,
                                wrap: false,
                            },
                            |cx| {
                                let mut renderer = AppTreeRowRenderer;
                                vec![
                                    cx.text(title),
                                    cx.text(subtitle),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(8.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            vec![
                                                shadcn::Button::new("Primary")
                                                    .on_click(CommandId::from("gallery.progress.inc"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Destructive")
                                                    .variant(shadcn::ButtonVariant::Destructive)
                                                    .on_click(CommandId::from("gallery.progress.dec"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Outline")
                                                    .variant(shadcn::ButtonVariant::Outline)
                                                    .on_click(CommandId::from("gallery.progress.reset"))
                                                    .into_element(cx),
                                                shadcn::Button::new("Disabled")
                                                    .disabled(true)
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    shadcn::Progress::new(progress)
                                        .range(0.0, 100.0)
                                        .into_element(cx),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(12.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                    |cx| {
                                        cx.observe_model(&checkbox, Invalidation::Layout);
                                        cx.observe_model(&switch, Invalidation::Layout);
                                        let checkbox_value =
                                            cx.app.models().get_copied(&checkbox).unwrap_or(false);
                                        let switch_value =
                                            cx.app.models().get_copied(&switch).unwrap_or(false);

                                            vec![
                                                shadcn::Checkbox::new(checkbox)
                                                    .a11y_label("Demo checkbox")
                                                    .into_element(cx),
                                                cx.text(format!("checkbox: {checkbox_value}")),
                                                shadcn::Switch::new(switch)
                                                    .a11y_label("Demo switch")
                                                    .into_element(cx),
                                                cx.text(format!("switch: {switch_value}")),
                                            ]
                                        },
                                    ),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Vertical,
                                            gap: Px(8.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Stretch,
                                        wrap: false,
                                    },
                                    |cx| {
                                        cx.observe_model(&radio, Invalidation::Layout);
                                        let value = cx
                                            .app
                                            .models()
                                            .get_cloned(&radio)
                                            .flatten()
                                            .map(|v| v.to_string())
                                            .unwrap_or_else(|| "<none>".to_string());

                                            vec![
                                                cx.text(format!("radio: {value}")),
                                                shadcn::RadioGroup::new(radio)
                                                    .a11y_label("Demo radio group")
                                                    .item(shadcn::RadioGroupItem::new("a", "Alpha"))
                                                    .item(shadcn::RadioGroupItem::new("b", "Beta"))
                                                    .item(
                                                        shadcn::RadioGroupItem::new("c", "Disabled")
                                                            .disabled(true),
                                                    )
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Vertical,
                                            gap: Px(8.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Stretch,
                                        wrap: false,
                                    },
                                    |cx| {
                                        cx.observe_model(&select, Invalidation::Layout);
                                        let value = cx
                                            .app
                                            .models()
                                            .get_cloned(&select)
                                            .flatten()
                                            .as_deref()
                                            .unwrap_or("<none>")
                                            .to_owned();

                                        vec![
                                            shadcn::Select::new(select, select_open)
                                                .a11y_label("Demo select")
                                                    .placeholder("Pick a fruit")
                                                .items([
                                                    shadcn::SelectItem::new("apple", "Apple"),
                                                    shadcn::SelectItem::new("banana", "Banana"),
                                                    shadcn::SelectItem::new("cherry", "Cherry"),
                                                ])
                                                .into_element(cx),
                                            cx.text(format!("select: {value}")),
                                        ]
                                    },
                                ),
                                cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle::default(),
                                        direction: fret_core::Axis::Vertical,
                                        gap: Px(8.0),
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Stretch,
                                        wrap: false,
                                    },
                                    |cx| {
                                        cx.observe_model(&last_action, Invalidation::Layout);
                                        let last_action = cx.app.models().get_cloned(&last_action);

                                        let overlays = cx.flex(
                                            FlexProps {
                                                layout: LayoutStyle::default(),
                                                direction: fret_core::Axis::Horizontal,
                                                gap: Px(8.0),
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: true,
                                            },
                                            |cx| {
                                                let tooltip = shadcn::Tooltip::new(
                                                    shadcn::Button::new("Tooltip (hover)")
                                                        .variant(shadcn::ButtonVariant::Outline)
                                                        .into_element(cx),
                                                    shadcn::TooltipContent::new(
                                                        "Tooltip: hover intent + placement",
                                                    )
                                                    .into_element(cx),
                                                )
                                                .open_delay_frames(10)
                                                .close_delay_frames(10)
                                                .side(shadcn::TooltipSide::Top)
                                                .into_element(cx);

                                                let hover_card = {
                                                    let theme = Theme::global(&*cx.app);
                                                    cx.container(
                                                        ContainerProps {
                                                            layout: {
                                                                let mut layout = LayoutStyle::default();
                                                                layout.size.width = Length::Px(Px(240.0));
                                                                layout.size.height = Length::Px(Px(72.0));
                                                                layout.overflow = Overflow::Clip;
                                                                layout
                                                            },
                                                            padding: Edges::all(Px(8.0)),
                                                            background: Some(theme.colors.panel_background),
                                                            border: Edges::all(Px(1.0)),
                                                            border_color: Some(theme.colors.panel_border),
                                                            ..Default::default()
                                                        },
                                                        |cx| {
                                                            vec![cx.flex(
                                                                FlexProps {
                                                                    layout: {
                                                                        let mut layout = LayoutStyle::default();
                                                                        layout.size.width = Length::Fill;
                                                                        layout.size.height = Length::Fill;
                                                                        layout
                                                                    },
                                                                    direction: fret_core::Axis::Vertical,
                                                                    gap: Px(0.0),
                                                                    padding: Edges::all(Px(0.0)),
                                                                    justify: MainAlign::End,
                                                                    align: CrossAlign::Start,
                                                                    wrap: false,
                                                                },
                                                                |cx| {
                                                                    vec![shadcn::HoverCard::new(
                                                                        shadcn::Button::new("HoverCard (hover, not clipped)")
                                                                            .variant(shadcn::ButtonVariant::Outline)
                                                                            .into_element(cx),
                                                                        shadcn::HoverCardContent::new(vec![
                                                                            cx.text("HoverCard content (overlay-root)"),
                                                                            cx.text("Move pointer from trigger to content."),
                                                                        ])
                                                                        .into_element(cx),
                                                                    )
                                                                    .close_delay_frames(10)
                                                                    .into_element(cx)]
                                                                },
                                                            )]
                                                        },
                                                    )
                                                };

                                                let dropdown =
                                                    shadcn::DropdownMenu::new(dropdown_open.clone())
                                                    .into_element(
                                                        cx,
                                                        |cx| {
                                                            shadcn::Button::new("DropdownMenu")
                                                                .variant(shadcn::ButtonVariant::Outline)
                                                                .toggle_model(dropdown_open.clone())
                                                                .into_element(cx)
                                                        },
                                                        |_cx| {
                                                            vec![
                                                                shadcn::DropdownMenuEntry::Item(
                                                                    shadcn::DropdownMenuItem::new("Apple")
                                                                        .on_select(
                                                                            "gallery.dropdown.select.apple",
                                                                        ),
                                                                ),
                                                                shadcn::DropdownMenuEntry::Item(
                                                                    shadcn::DropdownMenuItem::new("Banana")
                                                                        .on_select(
                                                                            "gallery.dropdown.select.banana",
                                                                        ),
                                                                ),
                                                                shadcn::DropdownMenuEntry::Separator,
                                                                shadcn::DropdownMenuEntry::Item(
                                                                    shadcn::DropdownMenuItem::new("Disabled")
                                                                        .disabled(true),
                                                                ),
                                                            ]
                                                        },
                                                    );

                                                let context_menu =
                                                    shadcn::ContextMenu::new(context_menu_open.clone())
                                                        .into_element(
                                                            cx,
                                                            |cx| {
                                                                shadcn::Button::new("ContextMenu (right click / Shift+F10)")
                                                                .variant(shadcn::ButtonVariant::Outline)
                                                                .into_element(cx)
                                                            },
                                                            |_cx| {
                                                                vec![
                                                                    shadcn::ContextMenuEntry::Item(
                                                                        shadcn::ContextMenuItem::new(
                                                                            "Action",
                                                                        )
                                                                        .on_select(
                                                                            "gallery.context_menu.action",
                                                                        ),
                                                                    ),
                                                                    shadcn::ContextMenuEntry::Separator,
                                                                    shadcn::ContextMenuEntry::Item(
                                                                        shadcn::ContextMenuItem::new(
                                                                            "Disabled",
                                                                        )
                                                                        .disabled(true),
                                                                    ),
                                                                ]
                                                            },
                                                        );

                                                let popover =
                                                    shadcn::Popover::new(popover_open.clone())
                                                    .auto_focus(true)
                                                    .into_element(
                                                        cx,
                                                        |cx| {
                                                            shadcn::Button::new("Popover")
                                                                .variant(shadcn::ButtonVariant::Outline)
                                                                .toggle_model(popover_open.clone())
                                                                .into_element(cx)
                                                        },
                                                        |cx| {
                                                            shadcn::PopoverContent::new(vec![
                                                                cx.text("Popover content"),
                                                                shadcn::Button::new("Close")
                                                                    .variant(shadcn::ButtonVariant::Secondary)
                                                                    .toggle_model(popover_open.clone())
                                                                    .into_element(cx),
                                                            ])
                                                            .into_element(cx)
                                                        },
                                                    );

                                                let dialog =
                                                    shadcn::Dialog::new(dialog_open.clone()).into_element(
                                                    cx,
                                                    |cx| {
                                                        shadcn::Button::new("Dialog")
                                                            .variant(shadcn::ButtonVariant::Outline)
                                                            .toggle_model(dialog_open.clone())
                                                            .into_element(cx)
                                                    },
                                                    |cx| {
                                                        shadcn::DialogContent::new(vec![
                                                            shadcn::DialogHeader::new(vec![
                                                                shadcn::DialogTitle::new("Dialog")
                                                                    .into_element(cx),
                                                                shadcn::DialogDescription::new(
                                                                    "Escape / overlay click closes",
                                                                )
                                                                .into_element(cx),
                                                            ])
                                                            .into_element(cx),
                                                            shadcn::DialogFooter::new(vec![
                                                                shadcn::Button::new("Close")
                                                                    .variant(shadcn::ButtonVariant::Secondary)
                                                                    .toggle_model(dialog_open.clone())
                                                                    .into_element(cx),
                                                            ])
                                                            .into_element(cx),
                                                        ])
                                                        .into_element(cx)
                                                    },
                                                );

                                                let alert_dialog =
                                                    shadcn::AlertDialog::new(alert_dialog_open.clone())
                                                    .into_element(
                                                        cx,
                                                        |cx| {
                                                            shadcn::Button::new("AlertDialog")
                                                                .variant(shadcn::ButtonVariant::Outline)
                                                                .toggle_model(alert_dialog_open.clone())
                                                                .into_element(cx)
                                                        },
                                                        |cx| {
                                                            shadcn::AlertDialogContent::new(vec![
                                                                shadcn::AlertDialogHeader::new(vec![
                                                                    shadcn::AlertDialogTitle::new(
                                                                        "Are you absolutely sure?",
                                                                    )
                                                                    .into_element(cx),
                                                                    shadcn::AlertDialogDescription::new(
                                                                        "This is non-closable by overlay click.",
                                                                    )
                                                                    .into_element(cx),
                                                                ])
                                                                .into_element(cx),
                                                                shadcn::AlertDialogFooter::new(vec![
                                                                    shadcn::AlertDialogCancel::new(
                                                                        "Cancel",
                                                                        alert_dialog_open.clone(),
                                                                    )
                                                                    .into_element(cx),
                                                                    shadcn::AlertDialogAction::new(
                                                                        "Continue",
                                                                        alert_dialog_open.clone(),
                                                                    )
                                                                    .into_element(cx),
                                                                ])
                                                                .into_element(cx),
                                                            ])
                                                            .into_element(cx)
                                                        },
                                                    );

                                                let sheet = shadcn::Sheet::new(sheet_open.clone())
                                                    .side(shadcn::SheetSide::Right)
                                                    .size(Px(360.0))
                                                    .into_element(
                                                        cx,
                                                        |cx| {
                                                            shadcn::Button::new("Sheet")
                                                                .variant(shadcn::ButtonVariant::Outline)
                                                                .toggle_model(sheet_open.clone())
                                                                .into_element(cx)
                                                        },
                                                        |cx| {
                                                            shadcn::SheetContent::new(vec![
                                                                shadcn::SheetHeader::new(vec![
                                                                    shadcn::SheetTitle::new("Sheet")
                                                                        .into_element(cx),
                                                                    shadcn::SheetDescription::new(
                                                                        "A modal side panel.",
                                                                    )
                                                                    .into_element(cx),
                                                                ])
                                                                .into_element(cx),
                                                                shadcn::SheetFooter::new(vec![
                                                                    shadcn::Button::new("Close")
                                                                        .variant(shadcn::ButtonVariant::Secondary)
                                                                        .toggle_model(sheet_open.clone())
                                                                        .into_element(cx),
                                                                ])
                                                                .into_element(cx),
                                                            ])
                                                            .into_element(cx)
                                                        },
                                                    );

                                            vec![
                                                tooltip,
                                                hover_card,
                                                dropdown,
                                                context_menu,
                                                popover,
                                                dialog,
                                                alert_dialog,
                                                    sheet,
                                                ]
                                            },
                                        );

                                        cx.observe_model(&cmdk_query, Invalidation::Layout);
                                        let query = cx
                                            .app
                                            .models()
                                            .get_cloned(&cmdk_query)
                                            .unwrap_or_default();
                                        let query = query.trim().to_ascii_lowercase();

                                        let cmdk_items: Vec<shadcn::CommandItem> = [
                                            ("Open", "open", false),
                                            ("Save", "save", false),
                                            ("Close", "close", false),
                                            ("Settings", "settings", false),
                                            ("Disabled", "disabled", true),
                                        ]
                                        .into_iter()
                                        .filter(|(label, _, _)| {
                                            query.is_empty()
                                                || label.to_ascii_lowercase().contains(&query)
                                        })
                                        .map(|(label, id, disabled)| {
                                            shadcn::CommandItem::new(label)
                                                .disabled(disabled)
                                                .on_select(CommandId::new(format!(
                                                    "gallery.cmdk.select.{id}"
                                                )))
                                        })
                                        .collect();

                                        let cmdk = shadcn::CommandDialog::new(
                                            cmdk_open.clone(),
                                            cmdk_query.clone(),
                                            cmdk_items,
                                        )
                                        .a11y_label("Command palette")
                                        .into_element(cx, |cx| {
                                            shadcn::Button::new("CommandDialog (Ctrl+K)")
                                                .variant(shadcn::ButtonVariant::Outline)
                                                .toggle_model(cmdk_open.clone())
                                                .into_element(cx)
                                        });

                                        vec![
                                            cx.text("overlays: tooltip / dropdown / context-menu / popover / dialog / alert-dialog / sheet"),
                                            overlays,
                                            cx.text(format!(
                                                "last action: {}",
                                                last_action
                                                    .as_deref()
                                                    .unwrap_or("<none>")
                                            )),
                                            cx.text("cmdk: Ctrl+K opens, arrows/hover highlight, Enter selects"),
                                            cmdk,
                                        ]
                                    },
                                ),
                                    cx.container(
                                        ContainerProps {
                                            layout: tree_slot_layout,
                                            ..Default::default()
                                        },
                                        |cx| {
                                            vec![fret_components_ui::declarative::tree::tree_view_with_renderer(
                                                cx,
                                                items,
                                                tree_state,
                                                fret_components_ui::Size::Medium,
                                                &mut renderer,
                                            )]
                                        },
                                    ),
                                ]
                            },
                        )]
                    },
                )]
            },
        );

        state.ui.set_root(root);
        fret_components_ui::window_overlays::render(&mut state.ui, app, services, window, bounds);
        state.root = Some(root);
    }

    fn handle_tree_command(
        app: &mut App,
        items: Model<Vec<TreeItem>>,
        state: Model<TreeState>,
        command: &CommandId,
    ) -> bool {
        if let Some(id) = command.as_str().strip_prefix("tree.select.") {
            let Ok(id) = id.parse::<TreeItemId>() else {
                return true;
            };
            let _ = app.models_mut().update(&state, |s| s.selected = Some(id));
            return true;
        }

        if let Some(id) = command.as_str().strip_prefix("tree.toggle.") {
            let Ok(id) = id.parse::<TreeItemId>() else {
                return true;
            };
            let _ = app.models_mut().update(&state, |s| {
                if !s.expanded.insert(id) {
                    s.expanded.remove(&id);
                }
            });
            return true;
        }

        let _ = items;
        false
    }

    fn handle_tree_key_event(
        app: &mut App,
        items: Model<Vec<TreeItem>>,
        state: Model<TreeState>,
        event: &Event,
    ) -> bool {
        let Event::KeyDown {
            key, repeat: false, ..
        } = event
        else {
            return false;
        };

        let items_value = app.models().get_cloned(&items).unwrap_or_default();
        let tree_state_value = app.models().get_cloned(&state).unwrap_or_default();
        let entries = fret_components_ui::flatten_tree(&items_value, &tree_state_value.expanded);
        if entries.is_empty() {
            return false;
        }

        let selected_id = tree_state_value.selected;
        let selected_index = selected_id
            .and_then(|id| entries.iter().position(|e| e.id == id))
            .unwrap_or(0);

        match key {
            KeyCode::ArrowUp => {
                let next = selected_index.saturating_sub(1);
                let id = entries[next].id;
                let _ = app.models_mut().update(&state, |s| s.selected = Some(id));
                true
            }
            KeyCode::ArrowDown => {
                let next = (selected_index + 1).min(entries.len().saturating_sub(1));
                let id = entries[next].id;
                let _ = app.models_mut().update(&state, |s| s.selected = Some(id));
                true
            }
            KeyCode::ArrowLeft => {
                let Some(cur) = entries.get(selected_index).cloned() else {
                    return true;
                };
                if tree_state_value.expanded.contains(&cur.id) {
                    let _ = app.models_mut().update(&state, |s| {
                        s.expanded.remove(&cur.id);
                    });
                    return true;
                }
                if let Some(parent) = cur.parent {
                    let _ = app
                        .models_mut()
                        .update(&state, |s| s.selected = Some(parent));
                    return true;
                }
                true
            }
            KeyCode::ArrowRight => {
                let Some(cur) = entries.get(selected_index).cloned() else {
                    return true;
                };
                if cur.has_children && !tree_state_value.expanded.contains(&cur.id) {
                    let _ = app.models_mut().update(&state, |s| {
                        s.expanded.insert(cur.id);
                    });
                    return true;
                }
                if cur.has_children {
                    if let Some(next) = entries.get(selected_index + 1)
                        && next.depth > cur.depth
                    {
                        let _ = app
                            .models_mut()
                            .update(&state, |s| s.selected = Some(next.id));
                    }
                    return true;
                }
                true
            }
            KeyCode::Home => {
                let id = entries[0].id;
                let _ = app.models_mut().update(&state, |s| s.selected = Some(id));
                true
            }
            KeyCode::End => {
                let id = entries[entries.len().saturating_sub(1)].id;
                let _ = app.models_mut().update(&state, |s| s.selected = Some(id));
                true
            }
            _ => false,
        }
    }
}

impl WinitDriver for ComponentsGalleryDriver {
    type WindowState = ComponentsGalleryWindowState;

    fn init(&mut self, _app: &mut App, _main_window: AppWindowId) {}

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
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

    fn handle_global_changes(
        &mut self,
        app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        changed: &[std::any::TypeId],
    ) {
        state.ui.propagate_global_changes(app, changed);
    }

    fn handle_command(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        command: CommandId,
    ) {
        if state.ui.dispatch_command(app, services, &command) {
            return;
        }

        if ComponentsGalleryDriver::handle_tree_command(
            app,
            state.items.clone(),
            state.tree_state.clone(),
            &command,
        ) {
            return;
        }

        if let Some(id) = command.as_str().strip_prefix("gallery.tree.action.") {
            tracing::info!(%id, "gallery tree row action");
            return;
        }

        if let Some(id) = command.as_str().strip_prefix("app.tree.action.") {
            tracing::info!(%id, "app tree row action");
            return;
        }

        if command.as_str() == "gallery.close" {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        if command.as_str() == "gallery.progress.inc" {
            let _ = app
                .models_mut()
                .update(&state.progress, |v| *v = (*v + 10.0).min(100.0));
        }

        if command.as_str() == "gallery.progress.dec" {
            let _ = app
                .models_mut()
                .update(&state.progress, |v| *v = (*v - 10.0).max(0.0));
        }

        if command.as_str() == "gallery.progress.reset" {
            let _ = app.models_mut().update(&state.progress, |v| *v = 35.0);
        }

        if let Some(item) = command.as_str().strip_prefix("gallery.dropdown.select.") {
            let msg: Arc<str> = Arc::from(format!("dropdown.select.{item}").into_boxed_str());
            let _ = app.models_mut().update(&state.last_action, |v| *v = msg);
        }

        if let Some(item) = command.as_str().strip_prefix("gallery.cmdk.select.") {
            let msg: Arc<str> = Arc::from(format!("cmdk.select.{item}").into_boxed_str());
            let _ = app.models_mut().update(&state.last_action, |v| *v = msg);
            let _ = app.models_mut().update(&state.cmdk_open, |v| *v = false);
            app.request_redraw(window);
        }

        if command.as_str() == "gallery.context_menu.action" {
            let _ = app.models_mut().update(&state.last_action, |v| {
                *v = Arc::<str>::from("context_menu.action");
            });
        }
    }

    fn handle_event(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
        event: &Event,
    ) {
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        let overlays_open = app.models().get_copied(&state.select_open).unwrap_or(false)
            || app
                .models()
                .get_copied(&state.dropdown_open)
                .unwrap_or(false)
            || app
                .models()
                .get_copied(&state.context_menu_open)
                .unwrap_or(false)
            || app
                .models()
                .get_copied(&state.popover_open)
                .unwrap_or(false)
            || app.models().get_copied(&state.dialog_open).unwrap_or(false)
            || app
                .models()
                .get_copied(&state.alert_dialog_open)
                .unwrap_or(false)
            || app.models().get_copied(&state.sheet_open).unwrap_or(false)
            || app.models().get_copied(&state.cmdk_open).unwrap_or(false);

        if overlays_open {
            state.ui.dispatch_event(app, services, event);
            return;
        }

        if let Event::KeyDown {
            key: KeyCode::KeyK,
            modifiers,
            repeat: false,
        } = event
        {
            let open_chord = if cfg!(target_os = "macos") {
                modifiers.meta || modifiers.ctrl
            } else {
                modifiers.ctrl
            };

            if open_chord {
                let _ = app.models_mut().update(&state.cmdk_open, |v| *v = true);
                let _ = app.models_mut().update(&state.cmdk_query, |v| v.clear());
                app.request_redraw(window);
                return;
            }
        }

        let focus = state.ui.focus();
        let focused_is_tree_item = focus.is_some_and(|focused| {
            state.ui.semantics_snapshot().is_some_and(|snap| {
                snap.nodes
                    .iter()
                    .find(|n| n.id == focused)
                    .is_some_and(|n| n.role == SemanticsRole::TreeItem)
            })
        });

        if focus.is_none() || focused_is_tree_item {
            if ComponentsGalleryDriver::handle_tree_key_event(
                app,
                state.items.clone(),
                state.tree_state.clone(),
                event,
            ) {
                return;
            }
        }

        state.ui.dispatch_event(app, services, event);
    }

    fn render(
        &mut self,
        app: &mut App,
        window: AppWindowId,
        state: &mut Self::WindowState,
        bounds: Rect,
        scale_factor: f32,
        services: &mut dyn UiServices,
        scene: &mut Scene,
    ) {
        ComponentsGalleryDriver::render_gallery(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);
        scene.clear();
        state.ui.layout_all(app, services, bounds, scale_factor);
        state
            .ui
            .paint_all(app, services, bounds, scene, scale_factor);
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

    fn accessibility_snapshot(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
    ) -> Option<Arc<fret_core::SemanticsSnapshot>> {
        state.ui.semantics_snapshot_arc()
    }

    fn accessibility_focus(
        &mut self,
        _app: &mut App,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        state.ui.set_focus(Some(target));
    }

    fn accessibility_invoke(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
    ) {
        fret_ui_app::accessibility_actions::invoke(&mut state.ui, app, services, target);
    }

    fn accessibility_set_value_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::set_value_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_value_numeric(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: f64,
    ) {
        fret_ui_app::accessibility_actions::set_value_numeric(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }

    fn accessibility_set_text_selection(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        anchor: u32,
        focus: u32,
    ) {
        fret_ui_app::accessibility_actions::set_text_selection(
            &mut state.ui,
            app,
            services,
            target,
            anchor,
            focus,
        );
    }

    fn accessibility_replace_selected_text(
        &mut self,
        app: &mut App,
        services: &mut dyn UiServices,
        _window: AppWindowId,
        state: &mut Self::WindowState,
        target: fret_core::NodeId,
        value: &str,
    ) {
        fret_ui_app::accessibility_actions::replace_selected_text(
            &mut state.ui,
            app,
            services,
            target,
            value,
        );
    }
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_runner_winit_wgpu=info".parse().unwrap()),
        )
        .try_init();

    let event_loop = EventLoop::<RunnerUserEvent>::with_user_event()
        .build()
        .context("create winit event loop")?;
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    app.with_global_mut(IconRegistry::default, |icons, _app| {
        fret_icons_lucide::register_icons(icons);
    });

    let mut config = WinitRunnerConfig {
        main_window_title: "fret-demo components_gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    };

    if let Some(settings) = fret_app::SettingsFileV1::load_json_if_exists(".fret/settings.json")
        .context("load .fret/settings.json")?
    {
        app.set_global(settings.docking_interaction_settings());
        config.text_font_families.ui_sans = settings.fonts.ui_sans;
        config.text_font_families.ui_serif = settings.fonts.ui_serif;
        config.text_font_families.ui_mono = settings.fonts.ui_mono;
    }

    let driver = ComponentsGalleryDriver;
    let mut runner = WinitRunner::new(config, app, driver);
    runner.set_event_loop_proxy(event_loop.create_proxy());
    event_loop.run_app(&mut runner)?;
    Ok(())
}
