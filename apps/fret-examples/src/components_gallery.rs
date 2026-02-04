#[cfg(not(target_arch = "wasm32"))]
use anyhow::Context as _;
use fret_app::{App, CommandId, CommandMeta, Effect, Model, WhenExpr, WindowRequest};
use fret_bootstrap::ui_diagnostics::UiDiagnosticsService;
use fret_core::{
    AppWindowId, Corners, Edges, Event, FileDialogFilter, FileDialogOptions, FileDialogToken,
    FontId, KeyCode, Px, Rect, SemanticsRole, TextStyle, UiServices,
};
use fret_launch::{
    WindowCreateSpec, WinitAppDriver, WinitCommandContext, WinitEventContext, WinitRenderContext,
    WinitRunnerConfig, WinitWindowContext,
};
use fret_markdown as markdown;
use fret_runtime::PlatformCapabilities;
use fret_ui::declarative;
use fret_ui::element::{
    ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    SemanticsProps, TextProps,
};
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui::{ElementContext, Invalidation, Theme, UiTree};
use fret_ui_kit::declarative::cached_subtree::{CachedSubtreeExt, CachedSubtreeProps};
use fret_ui_kit::declarative::file_tree::{FileTreeViewProps, file_tree_view_retained_v0};
use fret_ui_kit::headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::tree::{TreeItem, TreeItemId, TreeState};
use fret_ui_kit::{ColorRef, LayoutRefinement, OverlayController, Space, UiExt, ui};
use fret_ui_shadcn as shadcn;
use std::collections::HashSet;
use std::sync::Arc;

struct ComponentsGalleryWindowState {
    ui: UiTree<App>,
    root: Option<fret_core::NodeId>,
    items: Model<Vec<TreeItem>>,
    tree_state: Model<TreeState>,
    file_tree_scroll: VirtualListScrollHandle,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    radio: Model<Option<Arc<str>>>,
    select: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    applied_theme_preset: Option<Arc<str>>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    ui_font_override: Model<Option<Arc<str>>>,
    ui_font_override_open: Model<bool>,
    emoji_font_override: Model<Option<Arc<str>>>,
    emoji_font_override_open: Model<bool>,
    pending_font_dialog: Option<FileDialogToken>,
    awaiting_font_dialog: bool,
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
                .child(TreeItem::new(12, "fret-ui-kit"))
                .child(TreeItem::new(13, "fret-demo").disabled(true)),
            TreeItem::new(20, "docs").child(TreeItem::new(21, "adr")),
        ]
    }

    fn tree_items_for_demo() -> (Vec<TreeItem>, TreeState) {
        if std::env::var_os("FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE").is_some() {
            let n: u64 = std::env::var("FRET_COMPONENTS_GALLERY_FILE_TREE_TORTURE_N")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(50_000);

            // Keep a large, mostly-flat surface for window-boundary perf/regression suites, but
            // also include at least one expandable folder near the top so scripted harnesses can
            // exercise expand/collapse (items_revision + entry-list churn) without collapsing the
            // entire surface.
            let nested_count: u64 = 128;
            let mut children: Vec<TreeItem> =
                Vec::with_capacity((n as usize).saturating_add(nested_count as usize));

            let folder_id: TreeItemId = 1;
            let folder_children: Vec<TreeItem> = (0..nested_count)
                .map(|i| TreeItem::new(n + 1 + i, format!("nested_{i}.rs")))
                .collect();
            children.push(TreeItem::new(folder_id, "folder_1").children(folder_children));

            for i in 2..=n {
                children.push(TreeItem::new(i, format!("file_{i}.rs")));
            }

            let root_id: TreeItemId = 0;
            let items = vec![TreeItem::new(root_id, "root").children(children)];
            let initial_state = TreeState {
                selected: Some(root_id),
                expanded: [root_id].into_iter().collect(),
            };
            return (items, initial_state);
        }

        let initial_state = TreeState {
            selected: Some(1),
            expanded: [1, 10, 20].into_iter().collect(),
        };
        (Self::sample_tree_items(), initial_state)
    }

    fn build_ui(app: &mut App, window: AppWindowId) -> ComponentsGalleryWindowState {
        let (items_value, initial_state) = Self::tree_items_for_demo();
        let items = app.models_mut().insert(items_value);
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
        let theme_preset = app
            .models_mut()
            .insert(Option::<Arc<str>>::Some(Arc::from("zinc/dark")));
        let theme_preset_open = app.models_mut().insert(false);
        let dropdown_open = app.models_mut().insert(false);
        let context_menu_open = app.models_mut().insert(false);
        let popover_open = app.models_mut().insert(false);
        let dialog_open = app.models_mut().insert(false);
        let alert_dialog_open = app.models_mut().insert(false);
        let sheet_open = app.models_mut().insert(false);
        let cmdk_open = app.models_mut().insert(false);
        let cmdk_query = app.models_mut().insert(String::new());
        let last_action = app.models_mut().insert(Arc::<str>::from("<none>"));

        let ui_font_override = app.models_mut().insert(None::<Arc<str>>);
        let ui_font_override_open = app.models_mut().insert(false);
        let emoji_font_override = app.models_mut().insert(None::<Arc<str>>);
        let emoji_font_override_open = app.models_mut().insert(false);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);
        ui.set_view_cache_enabled(std::env::var_os("FRET_EXAMPLES_VIEW_CACHE").is_some());
        ui.set_debug_enabled(std::env::var_os("FRET_DIAG").is_some_and(|v| !v.is_empty()));

        ComponentsGalleryWindowState {
            ui,
            root: None,
            items,
            tree_state,
            file_tree_scroll: VirtualListScrollHandle::new(),
            progress,
            checkbox,
            switch,
            radio,
            select,
            select_open,
            theme_preset,
            theme_preset_open,
            applied_theme_preset: Some(Arc::from("zinc/dark")),
            dropdown_open,
            context_menu_open,
            popover_open,
            dialog_open,
            alert_dialog_open,
            sheet_open,
            cmdk_open,
            cmdk_query,
            last_action,
            ui_font_override,
            ui_font_override_open,
            emoji_font_override,
            emoji_font_override_open,
            pending_font_dialog: None,
            awaiting_font_dialog: false,
        }
    }

    fn sync_gallery_shadcn_theme(app: &mut App, state: &mut ComponentsGalleryWindowState) {
        let preset = app.models().get_cloned(&state.theme_preset).flatten();
        if preset.as_deref() == state.applied_theme_preset.as_deref() {
            return;
        }

        let Some(preset) = preset else {
            return;
        };

        let Some((base, scheme)) = preset.split_once('/') else {
            return;
        };

        let base = match base {
            "neutral" => shadcn::shadcn_themes::ShadcnBaseColor::Neutral,
            "zinc" => shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
            "slate" => shadcn::shadcn_themes::ShadcnBaseColor::Slate,
            "stone" => shadcn::shadcn_themes::ShadcnBaseColor::Stone,
            "gray" => shadcn::shadcn_themes::ShadcnBaseColor::Gray,
            _ => return,
        };

        let scheme = match scheme {
            "light" => shadcn::shadcn_themes::ShadcnColorScheme::Light,
            "dark" => shadcn::shadcn_themes::ShadcnColorScheme::Dark,
            _ => return,
        };

        shadcn::shadcn_themes::apply_shadcn_new_york_v4(app, base, scheme);
        state.applied_theme_preset = Some(preset);
    }

    fn render_gallery(
        app: &mut App,
        services: &mut dyn UiServices,
        window: AppWindowId,
        state: &mut ComponentsGalleryWindowState,
        bounds: Rect,
    ) {
        OverlayController::begin_frame(app, window);

        let items = state.items.clone();
        let tree_state = state.tree_state.clone();
        let file_tree_scroll = state.file_tree_scroll.clone();
        let progress = state.progress.clone();
        let checkbox = state.checkbox.clone();
        let switch = state.switch.clone();
        let radio = state.radio.clone();
        let select = state.select.clone();
        let select_open = state.select_open.clone();
        let theme_preset = state.theme_preset.clone();
        let theme_preset_open = state.theme_preset_open.clone();
        let ui_font_override = state.ui_font_override.clone();
        let ui_font_override_open = state.ui_font_override_open.clone();
        let emoji_font_override = state.emoji_font_override.clone();
        let emoji_font_override_open = state.emoji_font_override_open.clone();
        let dropdown_open = state.dropdown_open.clone();
        let context_menu_open = state.context_menu_open.clone();
        let popover_open = state.popover_open.clone();
        let dialog_open = state.dialog_open.clone();
        let alert_dialog_open = state.alert_dialog_open.clone();
        let sheet_open = state.sheet_open.clone();
        let cmdk_open = state.cmdk_open.clone();
        let cmdk_query = state.cmdk_query.clone();
        let last_action = state.last_action.clone();

        Self::sync_gallery_shadcn_theme(app, state);

        let root = declarative::RenderRootContext::new(&mut state.ui, app, services, window, bounds)
            .render_root("components-gallery", |cx| {
                if std::env::var_os("FRET_COMPONENTS_GALLERY_TABLE_TORTURE").is_some() {
                    #[derive(Default)]
                    struct TableTortureModels {
                        data: Option<Arc<[u64]>>,
                        columns: Option<Arc<[ColumnDef<u64>]>>,
                        state: Option<Model<TableState>>,
                    }

                    let (data, columns, table_state) =
                        cx.with_state(TableTortureModels::default, |st| {
                            (st.data.clone(), st.columns.clone(), st.state.clone())
                        });

                    let (data, columns, table_state) = match (data, columns, table_state) {
                        (Some(data), Some(columns), Some(state)) => (data, columns, state),
                        _ => {
                            let n: u64 =
                                std::env::var("FRET_COMPONENTS_GALLERY_TABLE_TORTURE_N")
                                    .ok()
                                    .and_then(|v| v.parse().ok())
                                    .unwrap_or(50_000);

                            let data: Arc<[u64]> = Arc::from((0..n).collect::<Vec<u64>>());

                            let cols: Vec<ColumnDef<u64>> = vec![
                                ColumnDef::new("id").sort_by(|a: &u64, b: &u64| a.cmp(b)),
                                ColumnDef::new("status").sort_by(|a: &u64, b: &u64| {
                                    (a % 3).cmp(&(b % 3))
                                }),
                                ColumnDef::new("cpu").sort_by(|a: &u64, b: &u64| {
                                    (((a * 7) % 100) as u32)
                                        .cmp(&(((b * 7) % 100) as u32))
                                }),
                                ColumnDef::new("mem_mb").sort_by(|a: &u64, b: &u64| {
                                    ((128 + (a % 4096)) as u32)
                                        .cmp(&((128 + (b % 4096)) as u32))
                                }),
                            ];
                            let columns: Arc<[ColumnDef<u64>]> = Arc::from(cols);

                            let state = cx.app.models_mut().insert(TableState::default());

                            cx.with_state(TableTortureModels::default, |st| {
                                st.data = Some(data.clone());
                                st.columns = Some(columns.clone());
                                st.state = Some(state.clone());
                            });

                            (data, columns, state)
                        }
                    };

                    let theme = Theme::global(&*cx.app).clone();
                    let padding = theme.metric_required("metric.padding.md");

                    let mut root_layout = LayoutStyle::default();
                    root_layout.size.width = Length::Fill;
                    root_layout.size.height = Length::Fill;

                    let header: Arc<str> = Arc::from(
                        "Table torture (retained host): click headers to sort; wheel scroll crosses overscan boundaries.",
                    );

                    let header = cx.text(header);

                    let table = cx.cached_subtree_with(
                        CachedSubtreeProps::default().contained_layout(true),
                        |cx| {
                            vec![cx.semantics(
                                SemanticsProps {
                                    role: SemanticsRole::List,
                                    test_id: Some(Arc::<str>::from("components-gallery-table-root")),
                                    ..Default::default()
                                },
                                |cx| {
                                    let scroll_handle =
                                        cx.with_state(VirtualListScrollHandle::new, |h| h.clone());

                                    let state_revision =
                                        cx.app.models().revision(&table_state).unwrap_or(0);
                                    let items_revision = 1 ^ state_revision.rotate_left(17);

                                    let mut props =
                                        fret_ui_kit::declarative::table::TableViewProps::default();
                                    props.overscan = 10;
                                    props.row_height = Some(Px(28.0));
                                    props.row_measure_mode =
                                        fret_ui_kit::declarative::table::TableRowMeasureMode::Fixed;
                                    props.keep_alive = std::env::var(
                                        "FRET_COMPONENTS_GALLERY_TABLE_KEEP_ALIVE",
                                    )
                                    .ok()
                                    .and_then(|v| v.parse().ok());
                                    props.enable_column_grouping = false;
                                    props.enable_column_resizing = false;

                                    let header_label = Arc::new(|col: &ColumnDef<u64>| {
                                        Arc::<str>::from(col.id.as_ref())
                                    });
                                    let row_key_at =
                                        Arc::new(|row: &u64, _index: usize| RowKey(*row));
                                    let cell_at = Arc::new(
                                        move |cx: &mut ElementContext<'_, App>,
                                              col: &ColumnDef<u64>,
                                              row: &u64| {
                                            match col.id.as_ref() {
                                                "id" => cx.text(row.to_string()),
                                                "status" => cx.text(if row % 3 == 0 {
                                                    "idle"
                                                } else if row % 3 == 1 {
                                                    "busy"
                                                } else {
                                                    "offline"
                                                }),
                                                "cpu" => cx.text(format!("{}%", (row * 7) % 100)),
                                                "mem_mb" => cx.text(format!(
                                                    "{} MB",
                                                    128 + (row % 4096)
                                                )),
                                                _ => cx.text("?"),
                                            }
                                        },
                                    );

                                    vec![fret_ui_kit::declarative::table::table_virtualized_retained_v0(
                                        cx,
                                        data.clone(),
                                        columns.clone(),
                                        table_state.clone(),
                                        &scroll_handle,
                                        items_revision,
                                        row_key_at,
                                        Some(Arc::new(|row: &u64, _index: usize| {
                                            Arc::from(row.to_string())
                                        })),
                                        props,
                                        header_label,
                                        cell_at,
                                        Some(Arc::<str>::from(
                                            "components-gallery-table-header-",
                                        )),
                                        Some(Arc::<str>::from("components-gallery-table-row-")),
                                    )]
                                },
                            )]
                        },
                    );

                    let mut props = ContainerProps::default();
                    props.layout = root_layout;
                    props.padding = Edges::all(padding);
                    props.background = Some(theme.color_required("background"));

                    return vec![cx.container(props, |_cx| vec![header, table])];
                }

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
                let text_smoke_title: Arc<str> =
                    Arc::from("Text smoke (verify emoji renders in color)");
                let markdown_sample: Arc<str> = Arc::from(
                    "## Markdown (MVP)\n\nFenced code blocks render with monospace text. When enabled, tree-sitter highlighting is applied.\n\n```rust\nfn main() {\n    let answer = 42;\n    println!(\"{answer}\");\n}\n```\n",
                );
                let text_smoke_lines: [Arc<str>; 5] = [
                    Arc::from("ASCII: The quick brown fox 0123456789"),
                    Arc::from("IME provisional (fullwidth): ＡＢＣＤＥＦＧ １２３４５"),
                    Arc::from("Kana: ひらがな カタカナ"),
                    Arc::from("CJK: 汉字 漢字 한국어"),
                    Arc::from("Emoji: 😀 😺 🧑‍💻 ❤️ 👨‍👩‍👧‍👦 🇯🇵 🏳️‍🌈"),
                ];

                let mut tree_slot_layout = LayoutStyle::default();
                tree_slot_layout.size.width = Length::Fill;
                tree_slot_layout.size.height = Length::Fill;
                tree_slot_layout.flex.grow = 1.0;
                tree_slot_layout.flex.basis = Length::Px(Px(0.0));

                let padding = theme.metric_required("metric.padding.md");
                let bg = theme.color_required("background");

                 vec![ui::v_flex(cx, |cx| {
                                 vec![
                                     cx.text(title),
                                     cx.text(subtitle),
                                     markdown::Markdown::new(markdown_sample.clone())
                                        .into_element(cx),
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
                                                cx.text(Arc::<str>::from("Theme:")),
                                                shadcn::Select::new(theme_preset, theme_preset_open)
                                                    .a11y_label(
                                                        "Demo theme preset (shadcn new-york-v4)",
                                                    )
                                                    .placeholder("Pick a theme")
                                                    .refine_layout(
                                                        LayoutRefinement::default()
                                                            .w_px(Px(260.0)),
                                                    )
                                                    .items([
                                                        shadcn::SelectItem::new(
                                                            "neutral/light",
                                                            "Neutral (light)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "neutral/dark",
                                                            "Neutral (dark)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "zinc/light",
                                                            "Zinc (light)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "zinc/dark",
                                                            "Zinc (dark)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "slate/light",
                                                            "Slate (light)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "slate/dark",
                                                            "Slate (dark)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "stone/light",
                                                            "Stone (light)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "stone/dark",
                                                            "Stone (dark)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "gray/light",
                                                            "Gray (light)",
                                                        ),
                                                        shadcn::SelectItem::new(
                                                            "gray/dark",
                                                            "Gray (dark)",
                                                        ),
                                                    ])
                                                    .into_element(cx),
                                            ]
                                        },
                                    ),
                                    cx.text(Arc::<str>::from(format!(
                                        "Theme config: {}",
                                        theme.name
                                    ))),
                                    cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle::default(),
                                            direction: fret_core::Axis::Horizontal,
                                            gap: Px(10.0),
                                            padding: Edges::all(Px(0.0)),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Center,
                                            wrap: true,
                                        },
                                        |cx| {
                                            let border = theme.color_required("border");
                                            let swatches = [
                                                ("background", theme.color_required("background")),
                                                ("foreground", theme.color_required("foreground")),
                                                ("card", theme.color_required("card")),
                                                ("muted", theme.color_required("muted")),
                                                ("primary", theme.color_required("primary")),
                                                ("ring", theme.color_required("ring")),
                                                ("border", border),
                                            ];

                                            swatches
                                                .into_iter()
                                                .map(|(label, color)| {
                                                    cx.flex(
                                                        FlexProps {
                                                            layout: LayoutStyle::default(),
                                                            direction: fret_core::Axis::Horizontal,
                                                            gap: Px(6.0),
                                                            padding: Edges::all(Px(0.0)),
                                                            justify: MainAlign::Start,
                                                            align: CrossAlign::Center,
                                                            wrap: false,
                                                        },
                                                        |cx| {
                                                            vec![
                                                                cx.container(
                                                                    ContainerProps {
                                                                        layout: {
                                                                            let mut layout =
                                                                                LayoutStyle::default();
                                                                            layout.size.width =
                                                                                Length::Px(Px(14.0));
                                                                            layout.size.height =
                                                                                Length::Px(Px(14.0));
                                                                            layout
                                                                        },
                                                                        background: Some(color),
                                                                        border: Edges::all(Px(1.0)),
                                                                        border_color: Some(border),
                                                                        corner_radii: Corners::all(Px(3.0)),
                                                                        ..Default::default()
                                                                    },
                                                                    |_cx| Vec::new(),
                                                                ),
                                                                cx.text(label),
                                                            ]
                                                        },
                                                    )
                                                })
                                                .collect::<Vec<_>>()
                                        },
                                    ),
                                    cx.container(
                                        ContainerProps {
                                            layout: {
                                                let mut layout = LayoutStyle::default();
                                                layout.size.width = Length::Fill;
                                                layout.overflow = Overflow::Clip;
                                                layout
                                            },
                                            padding: Edges::all(
                                                theme.metric_required("metric.padding.md"),
                                            ),
                                            background: Some(theme.color_required("card")),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(theme.color_required("border")),
                                            corner_radii: Corners::all(Px(8.0)),
                                            ..Default::default()
                                        },
                                        |cx| {
                                            let selected_emoji_font = cx
                                                .app
                                                .models()
                                                .read(&emoji_font_override, |v| v.clone())
                                                .ok()
                                                .flatten();
                                            let selected_ui_font = cx
                                                .app
                                                .models()
                                                .read(&ui_font_override, |v| v.clone())
                                                .ok()
                                                .flatten();
                                            let available_fonts: Arc<[Arc<str>]> = cx
                                                .app
                                                .global::<fret_app::FontCatalogCache>()
                                                .map(|c| c.families_arc())
                                                .or_else(|| {
                                                    cx.app
                                                        .global::<fret_runtime::FontCatalog>()
                                                        .map(|c| {
                                                            let families: Vec<Arc<str>> = c
                                                                .families
                                                                .iter()
                                                                .map(|s| Arc::from(s.as_str()))
                                                                .collect();
                                                            Arc::from(families)
                                                        })
                                                })
                                                .unwrap_or_else(|| Arc::from([]));

                                            vec![cx.flex(
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
                                                    let mut out = Vec::with_capacity(
                                                        4 + text_smoke_lines.len()
                                                            + selected_emoji_font.is_some()
                                                                as usize,
                                                    );
                                                    out.push(cx.text(text_smoke_title));
                                                    out.push(cx.flex(
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
                                                            let mut seen: HashSet<Arc<str>> =
                                                                HashSet::new();
                                                            let mut items: Vec<
                                                                shadcn::SelectItem,
                                                            > = Vec::new();

                                                            for preferred in [
                                                                "Apple Color Emoji",
                                                                "Segoe UI Emoji",
                                                                "Noto Color Emoji",
                                                            ] {
                                                                let Some(found) = available_fonts
                                                                    .iter()
                                                                    .find(|n| {
                                                                        n.as_ref().eq_ignore_ascii_case(preferred)
                                                                    })
                                                                else {
                                                                    continue;
                                                                };

                                                                let name: Arc<str> = found.clone();
                                                                if seen.insert(name.clone()) {
                                                                    items.push(
                                                                        shadcn::SelectItem::new(
                                                                            name.clone(),
                                                                            name,
                                                                        ),
                                                                    );
                                                                }
                                                            }

                                                            for name in available_fonts.iter() {
                                                                if items.len() >= 32 {
                                                                    break;
                                                                }
                                                                let name: Arc<str> = name.clone();
                                                                if seen.insert(name.clone()) {
                                                                    items.push(
                                                                        shadcn::SelectItem::new(
                                                                            name.clone(),
                                                                            name.clone(),
                                                                        ),
                                                                    );
                                                                }
                                                            }

                                                            vec![
                                                                shadcn::Select::new(
                                                                    ui_font_override.clone(),
                                                                    ui_font_override_open.clone(),
                                                                )
                                                                .placeholder(
                                                                    "Force UI font (optional)",
                                                                )
                                                                .refine_layout(
                                                                    LayoutRefinement::default()
                                                                        .w_px(Px(260.0)),
                                                                )
                                                                .items(items.clone())
                                                                .into_element(cx),
                                                                shadcn::Select::new(
                                                                    emoji_font_override.clone(),
                                                                    emoji_font_override_open.clone(),
                                                                )
                                                                .placeholder(
                                                                    "Force emoji font (optional)",
                                                                )
                                                                .refine_layout(
                                                                    LayoutRefinement::default()
                                                                        .w_px(Px(260.0)),
                                                                )
                                                                .items(items)
                                                                .into_element(cx),
                                                                shadcn::Button::new("Load fonts...")
                                                                    .variant(
                                                                        shadcn::ButtonVariant::Outline,
                                                                    )
                                                                    .on_click(CommandId::from(
                                                                        "gallery.text_smoke.fonts.load",
                                                                    ))
                                                                    .ui()
                                                                    .px_3()
                                                                    .into_element(cx),
                                                                shadcn::Button::new("Reset UI")
                                                                    .variant(
                                                                        shadcn::ButtonVariant::Outline,
                                                                    )
                                                                    .on_click(CommandId::from(
                                                                        "gallery.text_smoke.ui_font.reset",
                                                                    ))
                                                                    .into_element(cx),
                                                                shadcn::Button::new("Reset")
                                                                    .variant(
                                                                        shadcn::ButtonVariant::Outline,
                                                                    )
                                                                    .on_click(CommandId::from(
                                                                        "gallery.text_smoke.emoji_font.reset",
                                                                    ))
                                                                    .into_element(cx),
                                                            ]
                                                        },
                                                    ));
                                                    if let Some(name) = selected_ui_font.as_deref() {
                                                        out.push(cx.text(format!(
                                                            "Forced UI font: {name}"
                                                        )));
                                                    }
                                                    if let Some(name) = selected_emoji_font.as_deref()
                                                    {
                                                        out.push(cx.text(format!(
                                                            "Forced emoji font: {name}"
                                                        )));
                                                    }
                                                    for line in text_smoke_lines {
                                                        let is_emoji_line =
                                                            line.starts_with("Emoji:");
                                                        if selected_emoji_font.is_some()
                                                            && is_emoji_line
                                                        {
                                                            let theme =
                                                                Theme::global(&*cx.app);
                                                            let mut style = TextStyle::default();
                                                            if let Some(name) =
                                                                selected_emoji_font.as_deref()
                                                            {
                                                                style.font = FontId::family(name);
                                                            }
                                                            style.size =
                                                                theme.metric_required("font.size");
                                                            style.line_height = Some(
                                                                theme.metric_required(
                                                                    "font.line_height",
                                                                ),
                                                            );

                                                            let mut props = TextProps::new(
                                                                format!(
                                                                    "Emoji (forced): {}",
                                                                    line.strip_prefix("Emoji: ")
                                                                        .unwrap_or(line.as_ref())
                                                                ),
                                                            );
                                                            props.style = Some(style);
                                                            props.wrap = fret_core::TextWrap::None;
                                                            props.overflow = fret_core::TextOverflow::Ellipsis;
                                                            out.push(cx.text_props(props));
                                                            continue;
                                                        }
                                                        if is_emoji_line {
                                                            let mut props = TextProps::new(line.clone());
                                                            props.wrap = fret_core::TextWrap::None;
                                                            props.overflow = fret_core::TextOverflow::Ellipsis;
                                                            out.push(cx.text_props(props));
                                                            continue;
                                                        }
                                                        if let Some(name) = selected_ui_font.as_deref() {
                                                            let theme =
                                                                Theme::global(&*cx.app);
                                                            let mut style = TextStyle::default();
                                                            style.font = FontId::family(name);
                                                            style.size =
                                                                theme.metric_required("font.size");
                                                            style.line_height =
                                                                Some(theme.metric_required(
                                                                    "font.line_height",
                                                                ));

                                                            let mut props = TextProps::new(line.clone());
                                                            props.style = Some(style);
                                                            out.push(cx.text_props(props));
                                                        } else {
                                                            out.push(cx.text(line));
                                                        }
                                                    }
                                                    out
                                                },
                                            )]
                                        },
                                    ),
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
                                    |_cx| {
                                        Vec::new()
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
                                                    shadcn::TooltipContent::new(vec![
                                                        shadcn::TooltipContent::text(
                                                            cx,
                                                            "Tooltip: hover intent + placement",
                                                        ),
                                                    ])
                                                    .into_element(cx),
                                                )
                                                .arrow(true)
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
                                                            background: Some(
                                                                theme.color_required("card"),
                                                            ),
                                                            border: Edges::all(Px(1.0)),
                                                            border_color: Some(
                                                                theme.color_required("border"),
                                                            ),
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
                                                                        shadcn::HoverCardContent::new(vec![cx.flex(
                                                                            FlexProps {
                                                                                layout: LayoutStyle::default(),
                                                                                direction: fret_core::Axis::Vertical,
                                                                                gap: Px(4.0),
                                                                                padding: Edges::all(Px(0.0)),
                                                                                justify: MainAlign::Start,
                                                                                align: CrossAlign::Start,
                                                                                wrap: false,
                                                                            },
                                                                            |cx| {
                                                                                vec![
                                                                                    cx.text("HoverCard content (overlay-root)"),
                                                                                    cx.text("Move pointer from trigger to content."),
                                                                                ]
                                                                            },
                                                                        )])
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

                                        let cmdk = shadcn::CommandDialog::new_with_host_commands(
                                            cx,
                                            cmdk_open.clone(),
                                            cmdk_query.clone(),
                                        )
                                        .a11y_label("Command palette")
                                        .into_element(cx, |cx| {
                                            shadcn::Button::new("CommandDialog (Ctrl/Cmd+P)")
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
                                            cx.text(
                                                "cmdk: Ctrl/Cmd+P opens, arrows/hover highlight, Enter selects",
                                            ),
                                             cmdk,
                                         ]
                                     },
                                 ),
                                file_tree_view_retained_v0(
                                    cx,
                                    items,
                                    tree_state,
                                    &file_tree_scroll,
                                    FileTreeViewProps {
                                        layout: tree_slot_layout,
                                        row_height: Px(26.0),
                                        overscan: 12,
                                        keep_alive: std::env::var(
                                            "FRET_COMPONENTS_GALLERY_FILE_TREE_KEEP_ALIVE",
                                        )
                                        .ok()
                                        .and_then(|v| v.parse().ok()),
                                        debug_root_test_id: Some(Arc::<str>::from(
                                            "components-gallery-file-tree-root",
                                        )),
                                        debug_row_test_id_prefix: Some(Arc::<str>::from(
                                            "components-gallery-file-tree-node",
                                        )),
                                    },
                                ),
                                ]
                    })
                    .size_full()
                    .gap(Space::N3)
                    .padding_px(padding)
                    .bg(ColorRef::Color(bg))
                    .into_element(cx)]
            });

        state.ui.set_root(root);
        OverlayController::render(&mut state.ui, app, services, window, bounds);
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
        let entries = fret_ui_kit::flatten_tree(&items_value, &tree_state_value.expanded);
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

impl WinitAppDriver for ComponentsGalleryDriver {
    type WindowState = ComponentsGalleryWindowState;

    fn init(&mut self, _app: &mut App, _main_window: AppWindowId) {}

    fn gpu_ready(
        &mut self,
        _app: &mut App,
        _context: &fret_render::WgpuContext,
        _renderer: &mut fret_render::Renderer,
    ) {
    }

    fn create_window_state(&mut self, app: &mut App, window: AppWindowId) -> Self::WindowState {
        Self::build_ui(app, window)
    }

    fn hot_reload_window(
        &mut self,
        app: &mut App,
        _services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        state: &mut Self::WindowState,
    ) {
        crate::hotpatch::reset_ui_tree(app, window, &mut state.ui);
        state.root = None;
        state.pending_font_dialog = None;
        state.awaiting_font_dialog = false;

        let _ = app.models_mut().update(&state.select_open, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&state.theme_preset_open, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&state.dropdown_open, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&state.context_menu_open, |v| *v = false);
        let _ = app.models_mut().update(&state.popover_open, |v| *v = false);
        let _ = app.models_mut().update(&state.dialog_open, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&state.alert_dialog_open, |v| *v = false);
        let _ = app.models_mut().update(&state.sheet_open, |v| *v = false);
        let _ = app.models_mut().update(&state.cmdk_open, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&state.ui_font_override_open, |v| *v = false);
        let _ = app
            .models_mut()
            .update(&state.emoji_font_override_open, |v| *v = false);
    }

    fn handle_model_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[fret_app::ModelId],
    ) {
        let WinitWindowContext {
            app, state, window, ..
        } = context;

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
            svc.record_model_changes(window, changed);
        });
        state.ui.propagate_model_changes(app, changed);

        if changed.contains(&state.ui_font_override.id()) {
            let selected = app
                .models()
                .read(&state.ui_font_override, |v| v.clone())
                .ok()
                .flatten();

            let mut config = app
                .global::<fret_core::TextFontFamilyConfig>()
                .cloned()
                .unwrap_or_default();
            config.ui_sans = selected
                .as_deref()
                .map(|name| vec![name.to_string()])
                .unwrap_or_default();
            app.set_global::<fret_core::TextFontFamilyConfig>(config);
        }
    }

    fn handle_global_changes(
        &mut self,
        context: WinitWindowContext<'_, Self::WindowState>,
        changed: &[std::any::TypeId],
    ) {
        let WinitWindowContext {
            app, state, window, ..
        } = context;
        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            svc.record_global_changes(app, window, changed);
        });
        state.ui.propagate_global_changes(app, changed);
    }

    fn handle_command(
        &mut self,
        context: WinitCommandContext<'_, Self::WindowState>,
        command: CommandId,
    ) {
        let WinitCommandContext {
            app,
            services,
            window,
            state,
        } = context;

        if command.as_str() == fret_app::core_commands::COMMAND_PALETTE
            || command.as_str() == fret_app::core_commands::COMMAND_PALETTE_LEGACY
        {
            let _ = app.models_mut().update(&state.cmdk_open, |v| *v = true);
            let _ = app.models_mut().update(&state.cmdk_query, |v| v.clear());
            app.request_redraw(window);
            return;
        }

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
            return;
        }

        if command.as_str() == "gallery.context_menu.action" {
            let _ = app.models_mut().update(&state.last_action, |v| {
                *v = Arc::<str>::from("context_menu.action");
            });
        }

        if command.as_str() == "gallery.text_smoke.emoji_font.reset" {
            let _ = app
                .models_mut()
                .update(&state.emoji_font_override, |v| *v = None);
        }

        if command.as_str() == "gallery.text_smoke.ui_font.reset" {
            let _ = app
                .models_mut()
                .update(&state.ui_font_override, |v| *v = None);
        }

        if command.as_str() == "gallery.text_smoke.fonts.load" {
            let caps = app
                .global::<PlatformCapabilities>()
                .cloned()
                .unwrap_or_default();
            if !caps.fs.file_dialogs {
                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("fonts.load: file dialogs not available");
                });
                return;
            }

            state.pending_font_dialog = None;
            state.awaiting_font_dialog = true;

            app.push_effect(Effect::FileDialogOpen {
                window,
                options: FileDialogOptions {
                    title: Some("Load fonts".to_string()),
                    multiple: true,
                    filters: vec![FileDialogFilter {
                        name: "Fonts".to_string(),
                        extensions: vec!["ttf".to_string(), "otf".to_string(), "ttc".to_string()],
                    }],
                },
            });

            let _ = app.models_mut().update(&state.last_action, |v| {
                *v = Arc::<str>::from("fonts.load: opening file dialog...");
            });
            return;
        }
    }

    fn handle_event(&mut self, context: WinitEventContext<'_, Self::WindowState>, event: &Event) {
        let WinitEventContext {
            app,
            services,
            window,
            state,
        } = context;

        let consumed = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            if !svc.is_enabled() {
                return false;
            }
            if svc.maybe_intercept_event_for_inspect_shortcuts(app, window, event) {
                return true;
            }
            svc.maybe_intercept_event_for_picking(app, window, event)
        });
        if consumed {
            return;
        }
        if matches!(event, Event::WindowCloseRequested) {
            app.push_effect(Effect::Window(WindowRequest::Close(window)));
            return;
        }

        match event {
            Event::FileDialogSelection(selection) => {
                if !state.awaiting_font_dialog {
                    state.ui.dispatch_event(app, services, event);
                    return;
                }
                state.awaiting_font_dialog = false;
                state.pending_font_dialog = Some(selection.token);

                app.push_effect(Effect::FileDialogReadAll {
                    window,
                    token: selection.token,
                });

                let _ = app.models_mut().update(&state.last_action, |v| {
                    *v = Arc::<str>::from("fonts.load: reading selected files...");
                });
                return;
            }
            Event::FileDialogData(data) => {
                if state.pending_font_dialog != Some(data.token) {
                    state.ui.dispatch_event(app, services, event);
                    return;
                }
                state.pending_font_dialog = None;

                let fonts: Vec<Vec<u8>> = data.files.iter().map(|f| f.bytes.clone()).collect();
                if !fonts.is_empty() {
                    app.push_effect(Effect::TextAddFonts { fonts });
                }
                app.push_effect(Effect::FileDialogRelease { token: data.token });

                let msg: Arc<str> = Arc::from(
                    format!(
                        "fonts.load: loaded_files={} errors={}",
                        data.files.len(),
                        data.errors.len()
                    )
                    .into_boxed_str(),
                );
                let _ = app.models_mut().update(&state.last_action, |v| *v = msg);
                return;
            }
            Event::FileDialogCanceled => {
                if state.awaiting_font_dialog || state.pending_font_dialog.is_some() {
                    state.awaiting_font_dialog = false;
                    state.pending_font_dialog = None;
                    let _ = app.models_mut().update(&state.last_action, |v| {
                        *v = Arc::<str>::from("fonts.load: canceled");
                    });
                    return;
                }
            }
            _ => {}
        }

        let overlays_open = app.models().get_copied(&state.select_open).unwrap_or(false)
            || app
                .models()
                .get_copied(&state.theme_preset_open)
                .unwrap_or(false)
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

    fn render(&mut self, context: WinitRenderContext<'_, Self::WindowState>) {
        let WinitRenderContext {
            app,
            services,
            window,
            state,
            bounds,
            scale_factor,
            scene,
        } = context;
        ComponentsGalleryDriver::render_gallery(app, services, window, state, bounds);

        state.ui.request_semantics_snapshot();
        state.ui.ingest_paint_cache_source(scene);

        let inspection_active = app
            .with_global_mut_untracked(UiDiagnosticsService::default, |svc, _app| {
                svc.wants_inspection_active(window)
            });
        state.ui.set_inspection_active(inspection_active);

        scene.clear();
        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.layout_all();

        let semantics_snapshot = state.ui.semantics_snapshot();
        let drive = app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.drive_script_for_window(
                app,
                window,
                bounds,
                scale_factor,
                semantics_snapshot,
                element_runtime,
            )
        });

        if drive.request_redraw {
            app.request_redraw(window);
            // Script-driven `wait_frames` needs a reliable way to advance frames even when the
            // scene is otherwise idle. Requesting an animation frame ensures the runner
            // schedules another render tick.
            app.push_effect(Effect::RequestAnimationFrame(window));
        }

        let mut injected_any = false;
        for event in drive.events {
            injected_any = true;
            state.ui.dispatch_event(app, services, &event);
        }

        if injected_any {
            let mut deferred_effects: Vec<Effect> = Vec::new();
            loop {
                let effects = app.flush_effects();
                if effects.is_empty() {
                    break;
                }

                let mut applied_any_command = false;
                for effect in effects {
                    match effect {
                        Effect::Command { window: w, command } => {
                            if w.is_none() || w == Some(window) {
                                let _ = state.ui.dispatch_command(app, services, &command);
                                applied_any_command = true;
                            } else {
                                deferred_effects.push(Effect::Command { window: w, command });
                            }
                        }
                        other => deferred_effects.push(other),
                    }
                }

                if !applied_any_command {
                    break;
                }
            }
            for effect in deferred_effects {
                app.push_effect(effect);
            }

            state.ui.request_semantics_snapshot();
            let mut frame =
                fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
            frame.layout_all();
        }

        let mut frame =
            fret_ui::UiFrameCx::new(&mut state.ui, app, services, window, bounds, scale_factor);
        frame.paint_all(scene);

        app.with_global_mut_untracked(UiDiagnosticsService::default, |svc, app| {
            let element_runtime = app.global::<fret_ui::elements::ElementRuntime>();
            svc.record_snapshot(
                app,
                window,
                bounds,
                scale_factor,
                &state.ui,
                element_runtime,
                scene,
            );
            let _ = svc.maybe_dump_if_triggered();
            if svc.is_enabled() {
                app.push_effect(Effect::RequestAnimationFrame(window));
            }
        });
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

pub fn build_app() -> App {
    let mut app = App::new();
    app.set_global(PlatformCapabilities::default());
    shadcn::shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn::shadcn_themes::ShadcnBaseColor::Zinc,
        shadcn::shadcn_themes::ShadcnColorScheme::Dark,
    );

    // Demo: register a minimal command surface for the command palette to discover.
    app.commands_mut().register(
        CommandId::new("gallery.cmdk.select.open"),
        CommandMeta::new("Open")
            .with_category("File")
            .with_keywords(["open", "file"]),
    );
    app.commands_mut().register(
        CommandId::new("gallery.cmdk.select.save"),
        CommandMeta::new("Save")
            .with_category("File")
            .with_keywords(["save", "file"]),
    );
    app.commands_mut().register(
        CommandId::new("gallery.cmdk.select.close"),
        CommandMeta::new("Close")
            .with_category("File")
            .with_keywords(["close", "file"]),
    );
    app.commands_mut().register(
        CommandId::new("gallery.cmdk.select.settings"),
        CommandMeta::new("Settings")
            .with_category("View")
            .with_keywords(["settings", "preferences", "prefs"]),
    );
    app.commands_mut().register(
        CommandId::new("gallery.cmdk.select.disabled"),
        CommandMeta::new("Disabled")
            .with_category("Gallery")
            .with_keywords(["disabled"])
            .with_when(WhenExpr::parse("false").expect("valid when expression")),
    );
    app
}

pub fn build_runner_config() -> WinitRunnerConfig {
    WinitRunnerConfig {
        main_window_title: "fret-demo components_gallery".to_string(),
        main_window_size: winit::dpi::LogicalSize::new(980.0, 720.0),
        ..Default::default()
    }
}

pub fn build_driver() -> impl WinitAppDriver {
    ComponentsGalleryDriver::default()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let app = build_app();
    let config = build_runner_config();

    fret_kit::run_native_demo(config, app, ComponentsGalleryDriver)
        .context("run components_gallery app")
}
