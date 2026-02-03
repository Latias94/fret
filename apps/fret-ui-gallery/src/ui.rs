use fret_app::{App, CommandId, Model};
use fret_code_editor as code_editor;
use fret_code_view as code_view;
use fret_core::{
    AttributedText, CaretAffinity, Color as CoreColor, Corners, DrawOrder, Edges, FontId, ImageId,
    Point, Px, Rect, SceneOp, Size, TextConstraints, TextOverflow, TextSpan, TextStyle, TextWrap,
};
use fret_kit::prelude::ModelWatchExt as _;
use fret_markdown as markdown;
use fret_ui::Theme;
use fret_ui::element::{CanvasProps, StackProps};
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_kit::declarative::CachedSubtreeExt as _;
use fret_ui_kit::ui;
use fret_ui_material3 as material3;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use time::Date;

use crate::spec::*;

fn matches_query(query: &str, item: &PageSpec) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return true;
    }

    let q_lower = q.to_ascii_lowercase();
    if item.id.to_ascii_lowercase().contains(&q_lower) {
        return true;
    }
    if item.label.to_ascii_lowercase().contains(&q_lower) {
        return true;
    }
    if item.origin.to_ascii_lowercase().contains(&q_lower) {
        return true;
    }
    item.tags
        .iter()
        .any(|t| t.to_ascii_lowercase().contains(&q_lower))
}

pub(crate) fn sidebar_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    query: &str,
    nav_query: Model<String>,
    selected_page: Model<Arc<str>>,
    workspace_tabs: Model<Vec<Arc<str>>>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let title_row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            vec![
                cx.text("Fret UI Gallery"),
                shadcn::Badge::new("WIP")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
            ]
        },
    );

    let query_input = {
        let nav_query = nav_query.clone();
        cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::<str>::from("ui-gallery-nav-search")),
                ..Default::default()
            },
            move |cx| {
                [shadcn::Input::new(nav_query.clone())
                    .a11y_label("Search components")
                    .placeholder("Search (id / tag)")
                    .into_element(cx)]
            },
        )
    };

    let mut nav_sections: Vec<AnyElement> = Vec::new();
    for group in PAGE_GROUPS {
        let group_sections = cx.keyed(group.title, |cx| {
            let mut group_items: Vec<AnyElement> = Vec::new();
            for item in group.items {
                if !matches_query(query, item) {
                    continue;
                }

                let is_selected = selected == item.id;
                let variant = if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Ghost
                };

                group_items.push(cx.keyed(item.id, |cx| {
                    let selected_page_for_activate = selected_page.clone();
                    let workspace_tabs_for_activate = workspace_tabs.clone();
                    let page_id_for_activate: Arc<str> = Arc::from(item.id);

                    let mut button = shadcn::Button::new(item.label)
                        .variant(variant)
                        .on_click(item.command)
                        .refine_layout(LayoutRefinement::default().w_full());

                    button =
                        button.test_id(format!("ui-gallery-nav-{}", item.id.replace('_', "-")));

                    let on_activate: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host.models_mut().update(&selected_page_for_activate, |v| {
                                *v = page_id_for_activate.clone();
                            });
                            let _ = host.models_mut().update(&workspace_tabs_for_activate, |t| {
                                if !t
                                    .iter()
                                    .any(|id| id.as_ref() == page_id_for_activate.as_ref())
                                {
                                    t.push(page_id_for_activate.clone());
                                }
                            });
                            host.request_redraw(action_cx.window);
                            // `request_redraw()` may be coalesced or fail to wake the event loop on some
                            // platforms/driver configurations. Ensure we get at least one follow-up turn
                            // so the new page presents promptly after navigation.
                            host.push_effect(fret_runtime::Effect::RequestAnimationFrame(
                                action_cx.window,
                            ));
                        });
                    button = button.on_activate(on_activate);

                    button.into_element(cx)
                }));
            }

            if group_items.is_empty() {
                return Vec::new();
            }

            vec![
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::from(group.title),
                    style: None,
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                }),
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full())
                        .gap(Space::N1),
                    |_cx| group_items,
                ),
            ]
        });

        nav_sections.extend(group_sections);
    }

    let nav_body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4),
        |_cx| nav_sections,
    );
    let nav_scroll = {
        let nav_scroll = if (bisect & BISECT_DISABLE_SIDEBAR_SCROLL) != 0 {
            nav_body
        } else {
            shadcn::ScrollArea::new([nav_body])
                .refine_layout(LayoutRefinement::default().w_full().h_full())
                .into_element(cx)
        };
        cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::<str>::from("ui-gallery-nav-scroll")),
                ..Default::default()
            },
            move |_cx| [nav_scroll],
        )
    };

    let container = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .p(Space::N4),
            LayoutRefinement::default().w_px(Px(280.0)).h_full(),
        ),
        |cx| {
            [stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N4),
                |_cx| [title_row, query_input, nav_scroll],
            )]
        },
    );

    container
}

pub(crate) fn content_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    content_tab: Model<Option<Arc<str>>>,
    theme_preset: Model<Option<Arc<str>>>,
    theme_preset_open: Model<bool>,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    select_value: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    avatar_demo_image: Model<Option<ImageId>>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_expressive: Model<bool>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_navigation_rail_value: Model<Arc<str>>,
    material3_navigation_drawer_value: Model<Arc<str>>,
    material3_modal_navigation_drawer_open: Model<bool>,
    material3_dialog_open: Model<bool>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
    code_editor_syntax_rust: Model<bool>,
    code_editor_boundary_identifier: Model<bool>,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let (title, origin, docs_md, usage_md) = page_meta(selected);
    let page_test_id: Arc<str> =
        Arc::from(format!("ui-gallery-page-{}", selected.replace('_', "-")));

    let header = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .justify_between()
            .items_center(),
        |cx| {
            let left = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().flex_1().min_w_0())
                    .gap(Space::N1)
                    .items_start(),
                |cx| {
                    vec![
                        cx.text_props(TextProps {
                            layout: {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = fret_ui::element::Length::Fill;
                                layout
                            },
                            text: Arc::from(title),
                            style: None,
                            color: None,
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                        }),
                        cx.text_props(TextProps {
                            layout: {
                                let mut layout = fret_ui::element::LayoutStyle::default();
                                layout.size.width = fret_ui::element::Length::Fill;
                                layout
                            },
                            text: Arc::from(origin),
                            style: None,
                            color: Some(theme.color_required("muted-foreground")),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                        }),
                    ]
                },
            );

            let theme_select = shadcn::Select::new(theme_preset, theme_preset_open)
                .placeholder("Theme preset")
                .items([
                    shadcn::SelectItem::new("zinc/light", "Zinc (light)"),
                    shadcn::SelectItem::new("zinc/dark", "Zinc (dark)"),
                    shadcn::SelectItem::new("slate/light", "Slate (light)"),
                    shadcn::SelectItem::new("slate/dark", "Slate (dark)"),
                    shadcn::SelectItem::new("neutral/light", "Neutral (light)"),
                    shadcn::SelectItem::new("neutral/dark", "Neutral (dark)"),
                ])
                .refine_layout(LayoutRefinement::default().w_px(Px(220.0)))
                .into_element(cx);

            let copy_actions = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        shadcn::Button::new("Copy link")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_CLIPBOARD_COPY_LINK)
                            .into_element(cx),
                        shadcn::Button::new("Copy usage")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_CLIPBOARD_COPY_USAGE)
                            .into_element(cx),
                        shadcn::Button::new("Copy notes")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .on_click(CMD_CLIPBOARD_COPY_NOTES)
                            .into_element(cx),
                    ]
                },
            );

            let right = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N3).items_center(),
                |_cx| [theme_select, copy_actions],
            );

            [left, right]
        },
    );

    let preview_panel = page_preview(
        cx,
        theme,
        selected,
        view_cache_enabled,
        view_cache_cache_shell,
        view_cache_inner_enabled,
        view_cache_popover_open,
        view_cache_continuous,
        view_cache_counter,
        popover_open,
        dialog_open,
        alert_dialog_open,
        sheet_open,
        portal_geometry_popover_open,
        select_value,
        select_open,
        combobox_value,
        combobox_open,
        combobox_query,
        date_picker_open,
        date_picker_month,
        date_picker_selected,
        resizable_h_fractions,
        resizable_v_fractions,
        data_table_state,
        data_grid_selected_row,
        tabs_value,
        accordion_value,
        avatar_demo_image,
        progress,
        checkbox,
        switch,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_list_value,
        material3_expressive,
        material3_navigation_bar_value,
        material3_navigation_rail_value,
        material3_navigation_drawer_value,
        material3_modal_navigation_drawer_open,
        material3_dialog_open,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
        material3_menu_open,
        text_input,
        text_area,
        dropdown_open,
        context_menu_open,
        context_menu_edge_open,
        cmdk_open,
        cmdk_query,
        last_action,
        virtual_list_torture_jump,
        virtual_list_torture_edit_row,
        virtual_list_torture_edit_text,
        virtual_list_torture_scroll,
        code_editor_syntax_rust,
        code_editor_boundary_identifier,
    );

    let active_tab: Arc<str> = cx
        .watch_model(&content_tab)
        .layout()
        .cloned()
        .flatten()
        .unwrap_or_else(|| Arc::from("preview"));

    let docs_panel = if active_tab.as_ref() != "docs" {
        Vec::new()
    } else if (bisect & BISECT_DISABLE_MARKDOWN) != 0 {
        vec![cx.text(docs_md)]
    } else {
        vec![markdown::Markdown::new(Arc::from(docs_md)).into_element(cx)]
    };
    let usage_panel = if active_tab.as_ref() != "usage" {
        Vec::new()
    } else if (bisect & BISECT_DISABLE_MARKDOWN) != 0 {
        vec![cx.text(usage_md)]
    } else {
        vec![markdown::Markdown::new(Arc::from(usage_md)).into_element(cx)]
    };

    let tabs = if (bisect & BISECT_DISABLE_TABS) != 0 {
        let docs_panel = if (bisect & BISECT_DISABLE_MARKDOWN) != 0 {
            cx.text(docs_md)
        } else {
            markdown::Markdown::new(Arc::from(docs_md)).into_element(cx)
        };
        let usage_panel = if (bisect & BISECT_DISABLE_MARKDOWN) != 0 {
            cx.text(usage_md)
        } else {
            markdown::Markdown::new(Arc::from(usage_md)).into_element(cx)
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N6),
            |_cx| [preview_panel, usage_panel, docs_panel],
        )
    } else {
        shadcn::Tabs::new(content_tab)
            .refine_layout(LayoutRefinement::default().w_full())
            .list_full_width(true)
            .items([
                shadcn::TabsItem::new("preview", "Preview", [preview_panel]),
                shadcn::TabsItem::new("usage", "Usage", usage_panel),
                shadcn::TabsItem::new("docs", "Notes", docs_panel),
            ])
            .into_element(cx)
    };

    let body = cx.keyed("ui_gallery.content_body", |cx| {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N6),
            |_cx| [header, tabs],
        )
    });

    let content_inner = if (bisect & BISECT_DISABLE_CONTENT_SCROLL) != 0 {
        body
    } else {
        cx.keyed("ui_gallery.content_scroll_area", |cx| {
            let mut scroll = shadcn::ScrollArea::new([body])
                .refine_layout(LayoutRefinement::default().w_full().h_full())
                .viewport_test_id("ui-gallery-content-viewport")
                .viewport_intrinsic_measure_mode(
                    fret_ui::element::ScrollIntrinsicMeasureMode::Viewport,
                );
            if selected == PAGE_VIRTUAL_LIST_TORTURE {
                scroll =
                    scroll.viewport_test_id("ui-gallery-content-viewport-virtual_list_torture");
                scroll = scroll.viewport_intrinsic_measure_mode(
                    fret_ui::element::ScrollIntrinsicMeasureMode::Viewport,
                );
            }
            scroll.into_element(cx)
        })
    };

    let content = cx.semantics(
        fret_ui::element::SemanticsProps {
            test_id: Some(Arc::<str>::from("ui-gallery-content-scroll")),
            ..Default::default()
        },
        move |_cx| [content_inner],
    );

    cx.named("ui_gallery.content_view_root", |cx| {
        cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(page_test_id),
                ..Default::default()
            },
            move |cx| {
                [cx.container(
                    decl_style::container_props(
                        theme,
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(theme.color_required("background")))
                            .p(Space::N6),
                        LayoutRefinement::default().w_full().h_full(),
                    ),
                    |_cx| [content],
                )]
            },
        )
    })
}

fn page_preview(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    select_value: Model<Option<Arc<str>>>,
    select_open: Model<bool>,
    combobox_value: Model<Option<Arc<str>>>,
    combobox_open: Model<bool>,
    combobox_query: Model<String>,
    date_picker_open: Model<bool>,
    date_picker_month: Model<fret_ui_headless::calendar::CalendarMonth>,
    date_picker_selected: Model<Option<Date>>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    avatar_demo_image: Model<Option<ImageId>>,
    progress: Model<f32>,
    checkbox: Model<bool>,
    switch: Model<bool>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_expressive: Model<bool>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_navigation_rail_value: Model<Arc<str>>,
    material3_navigation_drawer_value: Model<Arc<str>>,
    material3_modal_navigation_drawer_open: Model<bool>,
    material3_dialog_open: Model<bool>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
    code_editor_syntax_rust: Model<bool>,
    code_editor_boundary_identifier: Model<bool>,
) -> AnyElement {
    let body: Vec<AnyElement> = match selected {
        PAGE_LAYOUT => preview_layout(cx, theme),
        PAGE_VIEW_CACHE => preview_view_cache(
            cx,
            theme,
            view_cache_enabled,
            view_cache_cache_shell,
            view_cache_inner_enabled,
            view_cache_popover_open,
            view_cache_continuous,
            view_cache_counter,
            text_input,
            text_area,
        ),
        PAGE_VIRTUAL_LIST_TORTURE => preview_virtual_list_torture(
            cx,
            theme,
            virtual_list_torture_jump,
            virtual_list_torture_edit_row,
            virtual_list_torture_edit_text,
            virtual_list_torture_scroll,
        ),
        PAGE_CODE_VIEW_TORTURE => preview_code_view_torture(cx, theme),
        PAGE_CODE_EDITOR_MVP => preview_code_editor_mvp(
            cx,
            theme,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
        ),
        PAGE_CODE_EDITOR_TORTURE => preview_code_editor_torture(
            cx,
            theme,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
        ),
        PAGE_TEXT_SELECTION_PERF => preview_text_selection_perf(cx, theme),
        PAGE_TEXT_BIDI_RTL_CONFORMANCE => preview_text_bidi_rtl_conformance(cx, theme),
        PAGE_TEXT_MEASURE_OVERLAY => preview_text_measure_overlay(cx, theme),
        PAGE_WEB_IME_HARNESS => preview_web_ime_harness(cx, theme, text_input, text_area),
        PAGE_CHART_TORTURE => preview_chart_torture(cx, theme),
        PAGE_CANVAS_CULL_TORTURE => preview_canvas_cull_torture(cx, theme),
        PAGE_CHROME_TORTURE => preview_chrome_torture(
            cx,
            theme,
            popover_open,
            dialog_open,
            alert_dialog_open,
            sheet_open,
            portal_geometry_popover_open,
            dropdown_open,
            context_menu_open,
            context_menu_edge_open,
            last_action,
            text_input,
            text_area,
            checkbox,
            switch,
        ),
        PAGE_WINDOWED_ROWS_SURFACE_TORTURE => preview_windowed_rows_surface_torture(cx, theme),
        PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE => {
            preview_windowed_rows_surface_interactive_torture(cx, theme)
        }
        PAGE_DATA_TABLE_TORTURE => preview_data_table_torture(cx, theme, data_table_state),
        PAGE_TREE_TORTURE => preview_tree_torture(cx, theme),
        PAGE_BUTTON => preview_button(cx),
        PAGE_CARD => preview_card(cx),
        PAGE_BADGE => preview_badge(cx),
        PAGE_AVATAR => preview_avatar(cx, avatar_demo_image),
        PAGE_SKELETON => preview_skeleton(cx),
        PAGE_SCROLL_AREA => preview_scroll_area(cx),
        PAGE_TOOLTIP => preview_tooltip(cx),
        PAGE_SLIDER => preview_slider(cx),
        PAGE_ICONS => preview_icons(cx),
        PAGE_FIELD => preview_field(cx),
        PAGE_OVERLAY => preview_overlay(
            cx,
            popover_open,
            dialog_open,
            alert_dialog_open,
            sheet_open,
            portal_geometry_popover_open,
            dropdown_open,
            context_menu_open,
            context_menu_edge_open,
            last_action.clone(),
        ),
        PAGE_FORMS => preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_SELECT => preview_select(cx, select_value, select_open),
        PAGE_COMBOBOX => preview_combobox(cx, combobox_value, combobox_open, combobox_query),
        PAGE_DATE_PICKER => preview_date_picker(
            cx,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
        ),
        PAGE_RESIZABLE => {
            preview_resizable(cx, theme, resizable_h_fractions, resizable_v_fractions)
        }
        PAGE_DATA_TABLE => preview_data_table(cx, data_table_state),
        PAGE_DATA_GRID => preview_data_grid(cx, data_grid_selected_row),
        PAGE_TABS => preview_tabs(cx, tabs_value),
        PAGE_ACCORDION => preview_accordion(cx, accordion_value),
        PAGE_TABLE => preview_table(cx),
        PAGE_PROGRESS => preview_progress(cx, progress),
        PAGE_MENUS => preview_menus(cx, dropdown_open, context_menu_open, last_action.clone()),
        PAGE_COMMAND => preview_command_palette(cx, cmdk_open, cmdk_query, last_action.clone()),
        PAGE_TOAST => preview_toast(cx, last_action.clone()),
        PAGE_SONNER => preview_toast(cx, last_action.clone()),
        PAGE_ALERT => preview_alert(cx),
        PAGE_ALERT_DIALOG => preview_alert_dialog(cx, alert_dialog_open),
        PAGE_ASPECT_RATIO => preview_aspect_ratio(cx),
        PAGE_BREADCRUMB => preview_breadcrumb(cx, last_action.clone()),
        PAGE_BUTTON_GROUP => preview_button_group(cx),
        PAGE_CALENDAR => preview_calendar(cx, date_picker_month, date_picker_selected),
        PAGE_CAROUSEL => preview_shadcn_placeholder(cx, "Carousel"),
        PAGE_CHART => preview_shadcn_placeholder(cx, "Chart"),
        PAGE_CHECKBOX => preview_checkbox(cx, checkbox),
        PAGE_COLLAPSIBLE => preview_collapsible(cx),
        PAGE_CONTEXT_MENU => {
            preview_menus(cx, dropdown_open, context_menu_open, last_action.clone())
        }
        PAGE_DIALOG => preview_dialog(cx, dialog_open),
        PAGE_DRAWER => preview_drawer(cx),
        PAGE_DROPDOWN_MENU => {
            preview_menus(cx, dropdown_open, context_menu_open, last_action.clone())
        }
        PAGE_EMPTY => preview_empty(cx),
        PAGE_FORM => preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_HOVER_CARD => preview_hover_card(cx),
        PAGE_INPUT => preview_input(cx, text_input),
        PAGE_INPUT_GROUP => preview_input_group(cx),
        PAGE_INPUT_OTP => preview_input_otp(cx),
        PAGE_ITEM => preview_shadcn_placeholder(cx, "Item"),
        PAGE_KBD => preview_kbd(cx),
        PAGE_LABEL => preview_label(cx),
        PAGE_MENUBAR => preview_menubar(cx),
        PAGE_NATIVE_SELECT => preview_shadcn_placeholder(cx, "Native Select"),
        PAGE_NAVIGATION_MENU => preview_navigation_menu(cx),
        PAGE_PAGINATION => preview_pagination(cx),
        PAGE_POPOVER => preview_popover(cx, popover_open),
        PAGE_RADIO_GROUP => preview_radio_group(cx),
        PAGE_SEPARATOR => preview_separator(cx),
        PAGE_SHEET => preview_sheet(cx, sheet_open),
        PAGE_SIDEBAR => preview_shadcn_placeholder(cx, "Sidebar"),
        PAGE_SPINNER => preview_spinner(cx),
        PAGE_SWITCH => preview_switch(cx, switch),
        PAGE_TEXTAREA => preview_textarea(cx, text_area),
        PAGE_TOGGLE => preview_toggle(cx),
        PAGE_TOGGLE_GROUP => preview_toggle_group(cx),
        PAGE_TYPOGRAPHY => preview_typography(cx),
        PAGE_MATERIAL3_GALLERY => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_gallery(
                cx,
                material3_checkbox,
                material3_switch,
                material3_radio_value,
                material3_tabs_value,
                material3_list_value,
                material3_navigation_bar_value,
                material3_text_field_value,
                material3_text_field_disabled,
                material3_text_field_error,
                last_action.clone(),
            )
        }),
        PAGE_MATERIAL3_STATE_MATRIX => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_state_matrix(
                    cx,
                    material3_checkbox,
                    material3_switch,
                    material3_radio_value,
                    material3_tabs_value,
                    material3_navigation_bar_value,
                    material3_text_field_value,
                    material3_text_field_disabled,
                    material3_text_field_error,
                    material3_menu_open,
                    last_action.clone(),
                )
            })
        }
        PAGE_MATERIAL3_TOUCH_TARGETS => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_touch_targets(
                    cx,
                    material3_checkbox,
                    material3_switch,
                    material3_radio_value,
                    material3_tabs_value,
                )
            })
        }
        PAGE_MATERIAL3_BUTTON => {
            material3_scoped_page(cx, material3_expressive.clone(), preview_material3_button)
        }
        PAGE_MATERIAL3_ICON_BUTTON => material3_scoped_page(
            cx,
            material3_expressive.clone(),
            preview_material3_icon_button,
        ),
        PAGE_MATERIAL3_CHECKBOX => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_checkbox(cx, material3_checkbox)
        }),
        PAGE_MATERIAL3_SWITCH => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_switch(cx, material3_switch)
        }),
        PAGE_MATERIAL3_RADIO => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_radio(cx, material3_radio_value)
        }),
        PAGE_MATERIAL3_SELECT => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_select(cx)
        }),
        PAGE_MATERIAL3_TEXT_FIELD => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_text_field(
                    cx,
                    material3_text_field_value,
                    material3_text_field_disabled,
                    material3_text_field_error,
                )
            })
        }
        PAGE_MATERIAL3_TABS => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_tabs(cx, material3_tabs_value)
        }),
        PAGE_MATERIAL3_LIST => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_list(cx, material3_list_value)
        }),
        PAGE_MATERIAL3_NAVIGATION_BAR => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_navigation_bar(cx, material3_navigation_bar_value)
            })
        }
        PAGE_MATERIAL3_NAVIGATION_RAIL => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_navigation_rail(cx, material3_navigation_rail_value)
            })
        }
        PAGE_MATERIAL3_NAVIGATION_DRAWER => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_navigation_drawer(cx, material3_navigation_drawer_value)
            })
        }
        PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_modal_navigation_drawer(
                    cx,
                    material3_modal_navigation_drawer_open,
                    material3_navigation_drawer_value,
                )
            })
        }
        PAGE_MATERIAL3_DIALOG => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_dialog(cx, material3_dialog_open, last_action.clone())
        }),
        PAGE_MATERIAL3_MENU => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_menu(cx, material3_menu_open, last_action.clone())
        }),
        PAGE_MATERIAL3_SNACKBAR => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_snackbar(cx, last_action.clone())
        }),
        PAGE_MATERIAL3_TOOLTIP => {
            material3_scoped_page(cx, material3_expressive.clone(), preview_material3_tooltip)
        }
        _ => preview_intro(cx, theme),
    };

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Preview").into_element(cx),
            shadcn::CardDescription::new("Interactive preview for validating behaviors.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(body).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
}

fn material3_scoped_page<I, F>(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
    content: F,
) -> Vec<AnyElement>
where
    F: FnOnce(&mut ElementContext<'_, App>) -> I,
    I: IntoIterator<Item = AnyElement>,
{
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(material3_variant_toggle_row(cx, material3_expressive));

    let body = if enabled {
        material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            content,
        )
    } else {
        content(cx)
    };

    out.extend(body);
    out
}

fn material3_variant_toggle_row(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
) -> AnyElement {
    let enabled = cx
        .get_model_copied(&material3_expressive, Invalidation::Layout)
        .unwrap_or(false);

    stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Switch::new(material3_expressive.clone())
                    .a11y_label("Enable Material 3 Expressive variant")
                    .into_element(cx),
                ui::label(
                    cx,
                    if enabled {
                        "Variant: Expressive"
                    } else {
                        "Variant: Standard"
                    },
                )
                .into_element(cx),
            ]
        },
    )
}

fn preview_intro(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![ui::text_block(cx, desc).into_element(cx)])
                .into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
        .into_element(cx)
    };

    let grid = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |cx| {
            vec![
                card(
                    cx,
                    "Core",
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                ),
                card(
                    cx,
                    "UI Kit",
                    "Headless interaction policies: focus trap, dismiss, hover intent, etc.",
                ),
                card(
                    cx,
                    "Shadcn",
                    "Visual recipes: composed defaults built on the Kit layer",
                ),
            ]
        },
    );
    let grid = cx.semantics(
        fret_ui::element::SemanticsProps {
            label: Some(Arc::<str>::from("Debug:ui-gallery:intro:preview-grid")),
            test_id: Some(Arc::<str>::from("ui-gallery-intro-preview-grid")),
            ..Default::default()
        },
        move |_cx| [grid],
    );

    let note = {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("muted")))
                .rounded(Radius::Md)
                .p(Space::N4),
            LayoutRefinement::default().w_full().min_w_0(),
        );
        cx.container(props, |cx| {
            vec![ui::text_block(cx, "Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.").into_element(cx)]
        })
    };
    let note = cx.semantics(
        fret_ui::element::SemanticsProps {
            label: Some(Arc::<str>::from("Debug:ui-gallery:intro:preview-note")),
            test_id: Some(Arc::<str>::from("ui-gallery-intro-preview-note")),
            ..Default::default()
        },
        move |_cx| [note],
    );

    vec![grid, note]
}

fn preview_view_cache(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    view_cache_enabled: Model<bool>,
    view_cache_cache_shell: Model<bool>,
    view_cache_inner_enabled: Model<bool>,
    view_cache_popover_open: Model<bool>,
    view_cache_continuous: Model<bool>,
    view_cache_counter: Model<u64>,
    text_input: Model<String>,
    text_area: Model<String>,
) -> Vec<AnyElement> {
    let enabled = cx
        .get_model_copied(&view_cache_enabled, Invalidation::Layout)
        .unwrap_or(false);
    let cache_shell = cx
        .get_model_copied(&view_cache_cache_shell, Invalidation::Layout)
        .unwrap_or(false);
    let cache_inner = cx
        .get_model_copied(&view_cache_inner_enabled, Invalidation::Layout)
        .unwrap_or(true);
    let continuous = cx
        .get_model_copied(&view_cache_continuous, Invalidation::Layout)
        .unwrap_or(false);

    let toggles = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_enabled.clone())
                                .a11y_label("Enable view-cache mode")
                                .test_id("ui-gallery-view-cache-enabled")
                                .into_element(cx),
                            cx.text("Enable view-cache mode (global UiTree flag)"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_cache_shell.clone())
                                .a11y_label("Cache the gallery shell")
                                .test_id("ui-gallery-view-cache-cache-shell")
                                .into_element(cx),
                            cx.text("Cache shell (sidebar/content wrappers)"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_inner_enabled.clone())
                                .a11y_label("Enable inner ViewCache boundary")
                                .test_id("ui-gallery-view-cache-inner-cache")
                                .into_element(cx),
                            cx.text("Enable inner ViewCache boundary (torture subtree)"),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(view_cache_continuous.clone())
                                .a11y_label("Request continuous frames")
                                .test_id("ui-gallery-view-cache-continuous")
                                .into_element(cx),
                            cx.text("Continuous frames (cache-hit should still keep state alive)"),
                        ]
                    },
                ),
            ]
        },
    );

    let actions = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Bump counter")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-view-cache-bump-counter")
                    .on_click(CMD_VIEW_CACHE_BUMP)
                    .into_element(cx),
                shadcn::Button::new("Reset counter")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
                    .test_id("ui-gallery-view-cache-reset-counter")
                    .on_click(CMD_VIEW_CACHE_RESET)
                    .into_element(cx),
            ]
        },
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx| {
            vec![
                cx.text("Goal: validate cached-subtree correctness under real interaction."),
                cx.text(format!(
                    "Current settings: view_cache={} shell_cache={} inner_cache={} continuous={}",
                    enabled as u8, cache_shell as u8, cache_inner as u8, continuous as u8
                )),
                toggles,
                actions,
            ]
        },
    );

    let subtree_body = |cx: &mut ElementContext<'_, App>| -> Vec<AnyElement> {
        let render_count = cx.with_state(
            || 0u64,
            |v| {
                *v = v.saturating_add(1);
                *v
            },
        );

        let mut needs_lease = false;
        cx.with_state(
            || None::<ContinuousFrames>,
            |lease| {
                if continuous {
                    if lease.is_none() {
                        needs_lease = true;
                    }
                } else {
                    *lease = None;
                }
            },
        );
        if needs_lease {
            let lease = cx.begin_continuous_frames();
            cx.with_state(
                || None::<ContinuousFrames>,
                |slot| {
                    *slot = Some(lease);
                },
            );
        }

        let counter = cx
            .get_model_copied(&view_cache_counter, Invalidation::Layout)
            .unwrap_or(0);

        let input = shadcn::Input::new(text_input.clone())
            .a11y_label("Cached input")
            .placeholder("Type to invalidate the cache root")
            .into_element(cx);
        let textarea = shadcn::Textarea::new(text_area.clone())
            .a11y_label("Cached textarea")
            .into_element(cx);

        let popover = shadcn::Popover::new(view_cache_popover_open.clone())
            .auto_focus(true)
            .into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Popover (cached trigger)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-view-cache-popover-trigger")
                        .toggle_model(view_cache_popover_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new([
                        cx.text("Popover content"),
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-view-cache-popover-close")
                            .toggle_model(view_cache_popover_open.clone())
                            .into_element(cx),
                    ])
                    .into_element(cx)
                },
            );

        let mut rows: Vec<AnyElement> = Vec::new();
        rows.reserve(240);
        for i in 0..240u32 {
            rows.push(cx.keyed(i, |cx| {
                shadcn::Button::new(format!("Row {i}"))
                    .variant(shadcn::ButtonVariant::Ghost)
                    .size(shadcn::ButtonSize::Sm)
                    .refine_layout(LayoutRefinement::default().w_full())
                    .into_element(cx)
            }));
        }

        let list = shadcn::ScrollArea::new([stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N1),
            |_cx| rows,
        )])
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element(cx);

        vec![
            shadcn::Card::new(vec![
                shadcn::CardHeader::new(vec![
                    shadcn::CardTitle::new("Cached subtree").into_element(cx),
                    shadcn::CardDescription::new(format!(
                        "render_count={} counter={}",
                        render_count, counter
                    ))
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::CardContent::new(vec![input, textarea, popover, list]).into_element(cx),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx),
        ]
    };

    let subtree = if cache_inner {
        cx.cached_subtree(subtree_body)
    } else {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Uncached subtree").into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(subtree_body(cx)).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx)
    };

    vec![cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Generic,
            test_id: Some(Arc::<str>::from("ui-gallery-view-cache-root")),
            ..Default::default()
        },
        move |cx| {
            vec![
                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("View Cache Torture").into_element(cx),
                        shadcn::CardDescription::new(
                            "Compare cached vs uncached subtree execution and state retention.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![header]).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx),
                subtree,
                cx.text_props(TextProps {
                    layout: Default::default(),
                    text: Arc::from(
                        "Tip: keep 'Cache shell' off while iterating so the status bar updates every frame.",
                    ),
                    style: None,
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::Word,
                    overflow: TextOverflow::Clip,
                }),
            ]
        },
    )]
}

fn preview_layout(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, label: &str, color: fret_core::Color| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(color))
                    .rounded(Radius::Md)
                    .p(Space::N3),
                // In a horizontal flex row, we want "equal columns" semantics (`flex-1`), not
                // `w-full` (percent sizing). Percent sizing is fragile under intrinsic sizing
                // probes and can cause transient wrap widths (0px) to leak into final layout.
                LayoutRefinement::default().flex_1().min_w_0(),
            ),
            |cx| [ui::label(cx, label).w_full().into_element(cx)],
        )
    };

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3)
            .items_stretch(),
        |cx| {
            vec![
                boxy(cx, "Left (fill)", theme.color_required("accent")),
                boxy(cx, "Center (fill)", theme.color_required("muted")),
                boxy(cx, "Right (fill)", theme.color_required("card")),
            ]
        },
    );

    vec![
        ui::text_block(
            cx,
            "Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing).",
        )
        .into_element(cx),
        row,
    ]
}

fn preview_virtual_list_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
) -> Vec<AnyElement> {
    let len: usize = 10_000;

    let minimal_harness =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_MINIMAL").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let known_heights =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_KNOWN_HEIGHTS").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let retained_host =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_RETAINED").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let row_cache =
        match std::env::var_os("FRET_UI_GALLERY_VLIST_ROW_CACHE").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => false,
        };

    let header_editing_row = (!minimal_harness)
        .then(|| {
            cx.get_model_copied(&virtual_list_torture_edit_row, Invalidation::Layout)
                .flatten()
        })
        .flatten();

    let controls = (!minimal_harness).then(|| {
        let jump_input = {
            let mut props =
                fret_ui::element::TextInputProps::new(virtual_list_torture_jump.clone());
            props.a11y_label = Some(Arc::<str>::from("Jump to row"));
            props.test_id = Some(Arc::<str>::from("ui-gallery-virtual-list-jump-input"));
            props.placeholder = Some(Arc::<str>::from("Row index (e.g. 9000)"));
            props.layout.size.width = fret_ui::element::Length::Fill;
            cx.text_input(props)
        };

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N2)
                .items_center(),
            |cx| {
                let jump_model = virtual_list_torture_jump.clone();
                let scroll_for_jump = virtual_list_torture_scroll.clone();
                let on_jump: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let raw = host
                            .models_mut()
                            .get_cloned(&jump_model)
                            .unwrap_or_default();
                        let index = raw.trim().parse::<usize>().unwrap_or(0);
                        scroll_for_jump
                            .scroll_to_item(index, fret_ui::scroll::ScrollStrategy::Start);
                        host.request_redraw(action_cx.window);
                    });

                let scroll_for_bottom = virtual_list_torture_scroll.clone();
                let on_bottom: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        scroll_for_bottom.scroll_to_bottom();
                        host.request_redraw(action_cx.window);
                    });

                let edit_row_for_clear = virtual_list_torture_edit_row.clone();
                let edit_text_for_clear = virtual_list_torture_edit_text.clone();
                let on_clear_edit: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let _ = host.models_mut().update(&edit_row_for_clear, |v| *v = None);
                        let _ = host
                            .models_mut()
                            .update(&edit_text_for_clear, |v| v.clear());
                        host.request_redraw(action_cx.window);
                    });

                vec![
                    jump_input,
                    shadcn::Button::new("Jump")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-virtual-list-jump-button")
                        .on_activate(on_jump)
                        .into_element(cx),
                    shadcn::Button::new("Bottom")
                        .variant(shadcn::ButtonVariant::Outline)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-virtual-list-bottom-button")
                        .on_activate(on_bottom)
                        .into_element(cx),
                    shadcn::Button::new("Clear edit")
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id("ui-gallery-virtual-list-clear-edit-button")
                        .on_activate(on_clear_edit)
                        .into_element(cx),
                ]
            },
        )
    });

    let editing_indicator = (!minimal_harness).then(|| {
        let label = if let Some(row) = header_editing_row {
            Arc::<str>::from(format!("editing_row={row}"))
        } else {
            Arc::<str>::from("editing_row=<none>")
        };

        cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Text,
                label: Some(label),
                test_id: Some(Arc::<str>::from("ui-gallery-virtual-list-editing")),
                ..Default::default()
            },
            |cx| {
                if let Some(row) = header_editing_row {
                    vec![cx.text(format!("Editing row: {row}"))]
                } else {
                    vec![cx.text("Editing row: <none>")]
                }
            },
        )
    });

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            let mut out = vec![
                cx.text("Goal: deterministic virtualization torture surface (10k rows + scroll-to-item + inline edit)."),
                cx.text(if retained_host {
                    "Mode: retained host (virt-003 prototype; item subtrees can reattach without rerendering the parent cache root)."
                } else {
                    "Mode: render-driven (baseline; visible items update requires rerender when the window changes)."
                }),
                cx.text(if known_heights {
                    "Mode: known row heights (no measure pass; better for perf baselines)."
                } else {
                    "Mode: measured row heights (baseline)."
                }),
            ];

            if minimal_harness {
                out.push(cx.text("Harness: minimal (no focusable controls; reduces RAF/notify noise in perf bundles)."));
            } else {
                if let Some(controls) = controls {
                    out.push(controls);
                }
                if let Some(editing_indicator) = editing_indicator {
                    out.push(editing_indicator);
                }
            }

            out
        },
    );

    let list_layout = fret_ui::element::LayoutStyle {
        size: fret_ui::element::SizeStyle {
            width: fret_ui::element::Length::Fill,
            height: fret_ui::element::Length::Px(Px(420.0)),
            ..Default::default()
        },
        overflow: fret_ui::element::Overflow::Clip,
        ..Default::default()
    };

    let options = if known_heights {
        fret_ui::element::VirtualListOptions::known(Px(28.0), 10, |index| {
            if index % 15 == 0 { Px(44.0) } else { Px(28.0) }
        })
    } else {
        fret_ui::element::VirtualListOptions::new(Px(28.0), 10)
    };

    let list = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let list = if minimal_harness {
            if retained_host {
                let theme = theme.clone();
                let key_at = Arc::new(|i| i as fret_ui::ItemKey);
                let row = Arc::new(move |cx: &mut ElementContext<'_, App>, index: usize| {
                    let zebra = (index % 2) == 0;
                    let background = if zebra {
                        theme.color_required("muted")
                    } else {
                        theme.color_required("background")
                    };

                    let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };
                    let row_label = cx.text(format!("Row {index}"));

                    let mut container_props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(background))
                            .p(Space::N2),
                        LayoutRefinement::default()
                            .w_full()
                            .h_px(MetricRef::Px(height_hint)),
                    );
                    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                    let row_layout = container_props.layout;
                    let container = cx.container(container_props, |_cx| vec![row_label]);
                    let mut semantics = fret_ui::element::SemanticsProps::default();
                    semantics.layout = row_layout;
                    semantics.test_id = Some(std::sync::Arc::<str>::from(format!(
                        "ui-gallery-virtual-list-row-{index}-label"
                    )));
                    cx.semantics(semantics, |_cx| vec![container])
                });

                cx.virtual_list_keyed_retained_with_layout(
                    list_layout,
                    len,
                    options,
                    &virtual_list_torture_scroll,
                    key_at,
                    row,
                )
            } else {
                cx.virtual_list_keyed_with_layout(
                    list_layout,
                    len,
                    options,
                    &virtual_list_torture_scroll,
                    |i| i as fret_ui::ItemKey,
                    |cx, index| {
                        let zebra = (index % 2) == 0;
                        let background = if zebra {
                            theme.color_required("muted")
                        } else {
                            theme.color_required("background")
                        };

                        let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };
                        let row_label = cx.text(format!("Row {index}"));

                        let mut container_props = decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(background))
                                .p(Space::N2),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(height_hint)),
                        );
                        container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                        let row_layout = container_props.layout;
                        let container = cx.container(container_props, |_cx| vec![row_label]);
                        let mut semantics = fret_ui::element::SemanticsProps::default();
                        semantics.layout = row_layout;
                        semantics.test_id = Some(std::sync::Arc::<str>::from(format!(
                            "ui-gallery-virtual-list-row-{index}-label"
                        )));
                        cx.semantics(semantics, |_cx| vec![container])
                    },
                )
            }
        } else if retained_host {
            let theme = theme.clone();
            let edit_row = virtual_list_torture_edit_row.clone();
            let edit_text = virtual_list_torture_edit_text.clone();
            let row_cache = row_cache;

            let key_at = Arc::new(|i| i as fret_ui::ItemKey);
            let row = Arc::new(move |cx: &mut ElementContext<'_, App>, index: usize| {
                let index_u64 = index as u64;
                let row = |cx: &mut ElementContext<'_, App>| {
                    let editing_row = cx
                        .get_model_copied(&edit_row, Invalidation::Layout)
                        .flatten();
                    let is_editing = editing_row == Some(index_u64);

                    let zebra = (index % 2) == 0;
                    let background = if is_editing {
                        theme.color_required("accent")
                    } else if zebra {
                        theme.color_required("muted")
                    } else {
                        theme.color_required("background")
                    };

                    let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };

                    let edit_row_for_activate = edit_row.clone();
                    let edit_text_for_activate = edit_text.clone();
                    let on_select_row: fret_ui::action::OnActivate =
                        Arc::new(move |host, action_cx, _reason| {
                            let _ = host
                                .models_mut()
                                .update(&edit_row_for_activate, |v| *v = Some(index_u64));
                            let _ = host.models_mut().update(&edit_text_for_activate, |v| {
                                *v = format!("Row {index_u64}");
                            });
                            host.request_redraw(action_cx.window);
                        });

                    let row_label = shadcn::Button::new(format!("Row {index}"))
                        .variant(shadcn::ButtonVariant::Ghost)
                        .size(shadcn::ButtonSize::Sm)
                        .test_id(format!("ui-gallery-virtual-list-row-{index}-label"))
                        .on_activate(on_select_row.clone())
                        .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                        .into_element(cx);

                    let right = if is_editing {
                        let mut props = fret_ui::element::TextInputProps::new(edit_text.clone());
                        props.a11y_label = Some(Arc::<str>::from("Inline edit"));
                        props.test_id =
                            Some(Arc::<str>::from("ui-gallery-virtual-list-edit-input"));
                        props.placeholder = Some(Arc::<str>::from("Type to edit…"));
                        props.layout.size.width = fret_ui::element::Length::Fill;

                        stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full())
                                .gap(Space::N2)
                                .items_center(),
                            |cx| [cx.text_input(props)],
                        )
                    } else {
                        let edit_button = shadcn::Button::new("Edit")
                            .variant(shadcn::ButtonVariant::Outline)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id(format!("ui-gallery-virtual-list-row-{index}-edit"))
                            .on_activate(on_select_row)
                            .into_element(cx);

                        stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |_cx| [edit_button],
                        )
                    };

                    let mut container_props = decl_style::container_props(
                        &theme,
                        ChromeRefinement::default()
                            .bg(ColorRef::Color(background))
                            .p(Space::N2),
                        LayoutRefinement::default().w_full().h_px(height_hint),
                    );
                    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                    cx.container(container_props, |cx| {
                        [stack::hstack(
                            cx,
                            stack::HStackProps::default()
                                .layout(LayoutRefinement::default().w_full().h_full())
                                .gap(Space::N2)
                                .items_center(),
                            |_cx| [row_label, right],
                        )]
                    })
                };

                if row_cache {
                    cx.cached_subtree_with(
                        CachedSubtreeProps::default()
                            .contained_layout(true)
                            .cache_key(index_u64),
                        |cx| [row(cx)],
                    )
                } else {
                    row(cx)
                }
            });

            cx.virtual_list_keyed_retained_with_layout(
                list_layout,
                len,
                options,
                &virtual_list_torture_scroll,
                key_at,
                row,
            )
        } else {
            cx.virtual_list_keyed_with_layout(
                list_layout,
                len,
                options,
                &virtual_list_torture_scroll,
                |i| i as fret_ui::ItemKey,
                |cx, index| {
                    let index_u64 = index as u64;
                    let row = |cx: &mut ElementContext<'_, App>| {
                        let editing_row = cx
                            .get_model_copied(&virtual_list_torture_edit_row, Invalidation::Layout)
                            .flatten();
                        let is_editing = editing_row == Some(index_u64);

                        let zebra = (index % 2) == 0;
                        let background = if is_editing {
                            theme.color_required("accent")
                        } else if zebra {
                            theme.color_required("muted")
                        } else {
                            theme.color_required("background")
                        };

                        let height_hint = if index % 15 == 0 { Px(44.0) } else { Px(28.0) };

                        let edit_row_for_activate = virtual_list_torture_edit_row.clone();
                        let edit_text_for_activate = virtual_list_torture_edit_text.clone();
                        let on_select_row: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                let _ = host
                                    .models_mut()
                                    .update(&edit_row_for_activate, |v| *v = Some(index_u64));
                                let _ = host.models_mut().update(&edit_text_for_activate, |v| {
                                    *v = format!("Row {index_u64}");
                                });
                                host.request_redraw(action_cx.window);
                            });
                        let row_label = shadcn::Button::new(format!("Row {index}"))
                            .variant(shadcn::ButtonVariant::Ghost)
                            .size(shadcn::ButtonSize::Sm)
                            .test_id(format!("ui-gallery-virtual-list-row-{index}-label"))
                            .on_activate(on_select_row.clone())
                            .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
                            .into_element(cx);

                        let right = if is_editing {
                            let mut props = fret_ui::element::TextInputProps::new(
                                virtual_list_torture_edit_text.clone(),
                            );
                            props.a11y_label = Some(Arc::<str>::from("Inline edit"));
                            props.test_id =
                                Some(Arc::<str>::from("ui-gallery-virtual-list-edit-input"));
                            props.placeholder = Some(Arc::<str>::from("Type to edit…"));
                            props.layout.size.width = fret_ui::element::Length::Fill;

                            stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full())
                                    .gap(Space::N2)
                                    .items_center(),
                                |cx| vec![cx.text_input(props)],
                            )
                        } else {
                            let edit_button = shadcn::Button::new("Edit")
                                .variant(shadcn::ButtonVariant::Outline)
                                .size(shadcn::ButtonSize::Sm)
                                .test_id(format!("ui-gallery-virtual-list-row-{index}-edit"))
                                .on_activate(on_select_row)
                                .into_element(cx);

                            stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N2).items_center(),
                                |_cx| vec![edit_button],
                            )
                        };

                        let mut container_props = decl_style::container_props(
                            theme,
                            ChromeRefinement::default()
                                .bg(ColorRef::Color(background))
                                .p(Space::N2),
                            LayoutRefinement::default()
                                .w_full()
                                .h_px(MetricRef::Px(height_hint)),
                        );
                        container_props.layout.overflow = fret_ui::element::Overflow::Clip;

                        cx.container(container_props, |cx| {
                            vec![stack::hstack(
                                cx,
                                stack::HStackProps::default()
                                    .layout(LayoutRefinement::default().w_full().h_full())
                                    .gap(Space::N2)
                                    .items_center(),
                                |_cx| vec![row_label, right],
                            )]
                        })
                    };

                    if row_cache {
                        cx.cached_subtree_with(
                            CachedSubtreeProps::default()
                                .contained_layout(true)
                                .cache_key(index_u64),
                            |cx| vec![row(cx)],
                        )
                    } else {
                        row(cx)
                    }
                },
            )
        };

        let list = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::List,
                test_id: Some(Arc::<str>::from("ui-gallery-virtual-list-root")),
                ..Default::default()
            },
            |_cx| [list],
        );

        vec![list]
    });

    let root = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |_cx| vec![header, list],
    );

    let root = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-virtual-list-torture-root")),
            ..Default::default()
        },
        |_cx| [root],
    );

    vec![root]
}

fn code_view_torture_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code View Torture Harness\n");
            out.push_str("// Generated content: large line count + long lines\n\n");
            for i in 0..8_000 {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: fn example_{i}() {{ let x = {i}; let y = x.wrapping_mul(31); }}\n"
                    ),
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

fn preview_code_view_torture(cx: &mut ElementContext<'_, App>, _theme: &Theme) -> Vec<AnyElement> {
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress large scrollable code/text surfaces (candidate for prepaint-windowed lines)."),
                cx.text("Use scripted wheel steps + stale-paint checks to validate scroll stability."),
            ]
        },
    );

    let code = code_view_torture_source();

    let windowed =
        match std::env::var_os("FRET_UI_GALLERY_CODE_VIEW_WINDOWED").filter(|v| !v.is_empty()) {
            Some(v) => {
                let v = v.to_string_lossy().trim().to_ascii_lowercase();
                !(v == "0" || v == "false" || v == "no" || v == "off")
            }
            None => true,
        };

    let block = code_view::CodeBlock::new(code)
        .language("rust")
        .show_line_numbers(true)
        .windowed_lines(windowed)
        .show_scrollbar_y(true)
        .max_height(Px(420.0));
    let block = block.into_element(cx);

    let block = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-code-view-root")),
            ..Default::default()
        },
        |_cx| vec![block],
    );

    vec![header, block]
}

fn code_editor_mvp_source() -> String {
    [
        "// Code Editor MVP\n",
        "// Goals:\n",
        "// - Validate TextInputRegion focus + TextInput/Ime events\n",
        "// - Validate nested scrolling (editor owns its own scroll)\n",
        "// - Provide a base surface for code-editor-ecosystem-v1 workstream\n",
        "\n",
        "fn main() {\n",
        "    let mut sum = 0u64;\n",
        "    for i in 0..10_000 {\n",
        "        sum = sum.wrapping_add(i);\n",
        "    }\n",
        "    println!(\"sum={}\", sum);\n",
        "}\n",
        "\n",
        "struct Point { x: f32, y: f32 }\n",
        "\n",
        "impl Point {\n",
        "    fn len(&self) -> f32 {\n",
        "        (self.x * self.x + self.y * self.y).sqrt()\n",
        "    }\n",
        "}\n",
        "\n",
        "// Try: mouse drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, IME.\n",
    ]
    .concat()
}

fn code_editor_torture_source() -> String {
    static SOURCE: OnceLock<String> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            let mut out = String::new();
            out.push_str("// Code Editor Torture Harness\n");
            out.push_str("// Generated content: many lines + deterministic prefixes\n\n");
            for i in 0..20_000usize {
                let _ = std::fmt::Write::write_fmt(
                    &mut out,
                    format_args!(
                        "{i:05}: let value_{i} = {i}; // scrolling should never show stale lines\n"
                    ),
                );
            }
            out
        })
        .clone()
}

fn preview_code_editor_mvp(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    boundary_identifier: Model<bool>,
) -> Vec<AnyElement> {
    let syntax_enabled = cx
        .get_model_copied(&syntax_rust, Invalidation::Layout)
        .unwrap_or(false);
    let boundary_identifier_enabled = cx
        .get_model_copied(&boundary_identifier, Invalidation::Layout)
        .unwrap_or(true);
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            vec![
                cx.text("Goal: validate a paint-driven editable surface using TextInputRegion (focus + IME)."),
                cx.text("Try: drag selection, Ctrl+C/Ctrl+V, arrows, Backspace/Delete, Enter/Tab, IME preedit."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust.clone())
                                .a11y_label("Toggle Rust syntax highlighting")
                                .into_element(cx),
                            cx.text(if syntax_enabled {
                                "Syntax: Rust (tree-sitter)"
                            } else {
                                "Syntax: disabled"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(boundary_identifier.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .into_element(cx),
                            cx.text(if boundary_identifier_enabled {
                                "Word boundaries: Identifier"
                            } else {
                                "Word boundaries: UnicodeWord"
                            }),
                        ]
                    },
                ),
            ]
        },
    );

    let handle = cx.with_state(
        || code_editor::CodeEditorHandle::new(code_editor_mvp_source()),
        |h| h.clone(),
    );
    let last_applied = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_applied.get() != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        last_applied.set(Some(syntax_enabled));
    }
    let last_boundaries = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_boundaries.get() != Some(boundary_identifier_enabled) {
        handle.set_text_boundary_mode(if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        });
        last_boundaries.set(Some(boundary_identifier_enabled));
    }

    let editor = code_editor::CodeEditor::new(handle)
        .overscan(32)
        .into_element(cx);

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![editor],
    );

    let panel = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-code-editor-root")),
            ..Default::default()
        },
        |_cx| vec![panel],
    );

    vec![header, panel]
}

fn preview_code_editor_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    syntax_rust: Model<bool>,
    boundary_identifier: Model<bool>,
) -> Vec<AnyElement> {
    let syntax_enabled = cx
        .get_model_copied(&syntax_rust, Invalidation::Layout)
        .unwrap_or(false);
    let boundary_identifier_enabled = cx
        .get_model_copied(&boundary_identifier, Invalidation::Layout)
        .unwrap_or(true);
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        move |cx| {
            vec![
                cx.text("Goal: stress scroll stability + bounded text caching for the windowed code editor."),
                cx.text("Expect: auto-scroll bounce; line prefixes must stay consistent (no stale paint)."),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(syntax_rust.clone())
                                .a11y_label("Toggle Rust syntax highlighting")
                                .into_element(cx),
                            cx.text(if syntax_enabled {
                                "Syntax: Rust (tree-sitter)"
                            } else {
                                "Syntax: disabled"
                            }),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |cx| {
                        vec![
                            shadcn::Switch::new(boundary_identifier.clone())
                                .a11y_label("Toggle identifier word boundaries")
                                .into_element(cx),
                            cx.text(if boundary_identifier_enabled {
                                "Word boundaries: Identifier"
                            } else {
                                "Word boundaries: UnicodeWord"
                            }),
                        ]
                    },
                ),
            ]
        },
    );

    let handle = cx.with_state(
        || code_editor::CodeEditorHandle::new(code_editor_torture_source()),
        |h| h.clone(),
    );
    let last_applied = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_applied.get() != Some(syntax_enabled) {
        handle.set_language(if syntax_enabled { Some("rust") } else { None });
        last_applied.set(Some(syntax_enabled));
    }
    let last_boundaries = cx.with_state(|| Rc::new(Cell::new(None::<bool>)), |v| v.clone());
    if last_boundaries.get() != Some(boundary_identifier_enabled) {
        handle.set_text_boundary_mode(if boundary_identifier_enabled {
            fret_runtime::TextBoundaryMode::Identifier
        } else {
            fret_runtime::TextBoundaryMode::UnicodeWord
        });
        last_boundaries.set(Some(boundary_identifier_enabled));
    }

    let editor = code_editor::CodeEditor::new(handle)
        .overscan(128)
        .torture(code_editor::CodeEditorTorture::auto_scroll_bounce(Px(8.0)))
        .into_element(cx);

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(520.0))),
        ),
        |_cx| vec![editor],
    );

    let panel = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-code-editor-torture-root")),
            ..Default::default()
        },
        |_cx| vec![panel],
    );

    vec![header, panel]
}

fn selection_perf_source() -> Arc<str> {
    static SOURCE: OnceLock<Arc<str>> = OnceLock::new();
    SOURCE
        .get_or_init(|| {
            use std::fmt::Write;

            let mut out = String::with_capacity(320_000);
            for i in 0..5000usize {
                let _ = writeln!(
                    &mut out,
                    "{i:05}: The quick brown fox jumps over the lazy dog. 0123456789 ABC xyz"
                );
            }
            Arc::<str>::from(out)
        })
        .clone()
}

fn preview_text_selection_perf(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        max_width_bits: u32,
        scale_bits: u32,
    }

    #[derive(Default)]
    struct SelectionPerfState {
        scroll_y: Px,
        content_height: Px,
        viewport_height: Px,
        last_clipped_rects: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(SelectionPerfState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: track selection rect count for large selections."),
                cx.text("Expectation: rect generation scales with visible lines when clipped to the viewport (not document length)."),
                cx.text("Scroll with the mouse wheel over the demo surface."),
            ]
        },
    );

    let source = selection_perf_source();
    let source_len = source.len();

    let on_wheel_state = state.clone();
    let on_wheel: fret_ui::action::OnWheel = Arc::new(move |host, action_cx, wheel| {
        let mut st = on_wheel_state.borrow_mut();

        let max_scroll = (st.content_height.0 - st.viewport_height.0).max(0.0);
        if max_scroll <= 0.0 {
            st.scroll_y = Px(0.0);
        } else {
            st.scroll_y = Px((st.scroll_y.0 - wheel.delta.y.0).clamp(0.0, max_scroll));
        }

        host.invalidate(fret_ui::Invalidation::Paint);
        host.request_redraw(action_cx.window);
        true
    });

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(420.0))),
        ),
        move |cx| {
            let mut pointer = fret_ui::element::PointerRegionProps::default();
            pointer.layout.size.width = fret_ui::element::Length::Fill;
            pointer.layout.size.height = fret_ui::element::Length::Fill;
            pointer.layout.overflow = fret_ui::element::Overflow::Clip;

            let paint_state = state.clone();
            let paint_source = source.clone();

            let content = cx.pointer_region(pointer, move |cx| {
                cx.pointer_region_on_wheel(on_wheel.clone());

                let mut canvas = CanvasProps::default();
                canvas.layout.size.width = fret_ui::element::Length::Fill;
                canvas.layout.size.height = fret_ui::element::Length::Fill;
                canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                let canvas = cx.canvas(canvas, move |p| {
                    let bounds = p.bounds();
                    let pad = Px(12.0);

                    let inner = Rect::new(
                        Point::new(
                            Px(bounds.origin.x.0 + pad.0),
                            Px(bounds.origin.y.0 + pad.0),
                        ),
                        Size::new(
                            Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                            Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                        ),
                    );

                    let max_width = inner.size.width;
                    if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                        return;
                    }

                    let scale_factor = p.scale_factor();
                    let selection_bg = p.theme().color_required("selection.background");
                    let fg = p.theme().color_required("foreground");
                    let muted = p.theme().color_required("muted-foreground");

                    let key = PreparedKey {
                        max_width_bits: max_width.0.to_bits(),
                        scale_bits: scale_factor.to_bits(),
                    };

                    let (stats, stats_origin) = {
                        let (services, scene) = p.services_and_scene();
                        let mut st = paint_state.borrow_mut();

                        let needs_prepare = st.blob.is_none()
                            || st.metrics.is_none()
                            || st.prepared_key != Some(key);
                        if needs_prepare {
                            if let Some(blob) = st.blob.take() {
                                services.text().release(blob);
                            }

                            let style = fret_core::TextStyle {
                                font: fret_core::FontId::monospace(),
                                size: Px(12.0),
                                ..Default::default()
                            };

                            let constraints = fret_core::TextConstraints {
                                max_width: Some(max_width),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                                scale_factor,
                            };

                            let (blob, metrics) = services
                                .text()
                                .prepare_str(paint_source.as_ref(), &style, constraints);
                            st.prepared_key = Some(key);
                            st.blob = Some(blob);
                            st.metrics = Some(metrics);
                        }

                        let Some(blob) = st.blob else {
                            return;
                        };
                        let Some(metrics) = st.metrics else {
                            return;
                        };

                        st.content_height = metrics.size.height;
                        st.viewport_height = inner.size.height;
                        let max_scroll = (st.content_height.0 - st.viewport_height.0).max(0.0);
                        st.scroll_y = Px(st.scroll_y.0.clamp(0.0, max_scroll));

                        let clip = Rect::new(
                            Point::new(Px(0.0), st.scroll_y),
                            Size::new(max_width, st.viewport_height),
                        );

                        let mut rects: Vec<Rect> = Vec::new();
                        services.selection_rects_clipped(blob, (0, source_len), clip, &mut rects);
                        st.last_clipped_rects = rects.len();

                        scene.push(SceneOp::PushClipRect { rect: inner });
                        for r in rects {
                            let rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + r.origin.x.0),
                                    Px(inner.origin.y.0 + r.origin.y.0 - st.scroll_y.0),
                                ),
                                r.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background: selection_bg,
                                border: Edges::all(Px(0.0)),
                                border_color: CoreColor::TRANSPARENT,
                                corner_radii: Corners::all(Px(0.0)),
                            });
                        }

                        let text_origin = Point::new(
                            inner.origin.x,
                            Px(inner.origin.y.0 + metrics.baseline.0 - st.scroll_y.0),
                        );
                        scene.push(SceneOp::Text {
                            order: DrawOrder(1),
                            origin: text_origin,
                            text: blob,
                            color: fg,
                        });
                        scene.push(SceneOp::PopClip);

                        let stats = format!(
                            "clipped rects: {} | scroll_y: {:.1}/{:.1} | content_h: {:.1} | viewport_h: {:.1}",
                            st.last_clipped_rects,
                            st.scroll_y.0,
                            max_scroll,
                            st.content_height.0,
                            st.viewport_height.0
                        );
                        let stats_origin = Point::new(
                            Px(bounds.origin.x.0 + 12.0),
                            Px(bounds.origin.y.0 + 10.0),
                        );
                        (stats, stats_origin)
                    };

                    let stats_style = fret_core::TextStyle {
                        font: fret_core::FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let _ = p.text(
                        p.key(&"text_selection_perf_stats"),
                        DrawOrder(2),
                        stats_origin,
                        stats,
                        stats_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints::default(),
                        scale_factor,
                    );
                });

                vec![canvas]
            });

            vec![content]
        },
    );

    let panel = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-text-selection-perf-root")),
            ..Default::default()
        },
        |_cx| vec![panel],
    );

    vec![header, panel]
}

fn preview_text_bidi_rtl_conformance(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct BidiSample {
        label: &'static str,
        text: &'static str,
    }

    const SAMPLES: &[BidiSample] = &[
        BidiSample {
            label: "LTR baseline",
            text: "The quick brown fox (123) jumps.",
        },
        BidiSample {
            label: "Hebrew (RTL)",
            text: "עברית (123) אבגדה",
        },
        BidiSample {
            label: "Arabic (RTL)",
            text: "مرحبا بالعالم (123) أهلاً",
        },
        BidiSample {
            label: "Mixed LTR + Hebrew",
            text: "abc אבג DEF 123",
        },
        BidiSample {
            label: "Mixed punctuation + numbers",
            text: "abc (אבג) - 12:34 - xyz",
        },
        BidiSample {
            label: "Mixed LTR + Arabic",
            text: "hello مرحبا (123) world",
        },
        BidiSample {
            label: "Grapheme + RTL",
            text: "emoji 😀 אבג café",
        },
        BidiSample {
            label: "Controls (RLM)",
            text: "RLM:\u{200F}abc אבג 123",
        },
    ];

    #[derive(Clone, Copy, PartialEq, Eq)]
    struct PreparedKey {
        sample: usize,
        max_width_bits: u32,
        scale_bits: u32,
    }

    struct BidiState {
        selected_sample: usize,
        prepared_key: Option<PreparedKey>,
        blob: Option<fret_core::TextBlobId>,
        metrics: Option<fret_core::TextMetrics>,
        anchor: usize,
        caret: usize,
        affinity: CaretAffinity,
        pending_down: Option<(Point, bool)>,
        last_drag_pos: Option<Point>,
        dragging: bool,
    }

    impl Default for BidiState {
        fn default() -> Self {
            Self {
                selected_sample: 0,
                prepared_key: None,
                blob: None,
                metrics: None,
                anchor: 0,
                caret: 0,
                affinity: CaretAffinity::Downstream,
                pending_down: None,
                last_drag_pos: None,
                dragging: false,
            }
        }
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(BidiState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: sanity-check BiDi/RTL geometry queries (hit-test, caret rects, selection rects)."),
                cx.text("Use the selectable samples to validate editor-like selection behavior."),
                cx.text("Use the diagnostic panel to verify `hit_test_point` → caret/selection rendering under mixed-direction strings."),
            ]
        },
    );

    let sample_buttons = {
        let mut buttons: Vec<AnyElement> = Vec::new();
        for (i, s) in SAMPLES.iter().enumerate() {
            buttons.push(cx.keyed(format!("bidi-sample-btn-{i}"), |cx| {
                let state_for_click = state.clone();
                let is_selected = state.borrow().selected_sample == i;

                let variant = if is_selected {
                    shadcn::ButtonVariant::Secondary
                } else {
                    shadcn::ButtonVariant::Outline
                };

                let on_activate: fret_ui::action::OnActivate =
                    Arc::new(move |host, action_cx, _reason| {
                        let mut st = state_for_click.borrow_mut();
                        st.selected_sample = i;
                        st.anchor = 0;
                        st.caret = 0;
                        st.affinity = CaretAffinity::Downstream;
                        st.pending_down = None;
                        st.last_drag_pos = None;
                        st.dragging = false;
                        host.request_redraw(action_cx.window);
                    });

                shadcn::Button::new(s.label)
                    .variant(variant)
                    .size(shadcn::ButtonSize::Sm)
                    .on_activate(on_activate)
                    .into_element(cx)
            }));
        }

        let mut props = fret_ui::element::FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(8.0);
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |_cx| buttons)
    };

    let selectable_samples = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            let mut out: Vec<AnyElement> = Vec::new();
            out.push(cx.text("SelectableText samples:"));

            for (i, s) in SAMPLES.iter().enumerate() {
                out.push(cx.keyed(format!("bidi-sample-row-{i}"), |cx| {
                    let rich = AttributedText::new(
                        Arc::<str>::from(s.text),
                        Arc::<[TextSpan]>::from([TextSpan::new(s.text.len())]),
                    );

                    let mut props = fret_ui::element::SelectableTextProps::new(rich);
                    props.style = Some(TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    });
                    props.wrap = TextWrap::None;
                    props.overflow = TextOverflow::Clip;
                    props.layout.size.width = fret_ui::element::Length::Fill;

                    let text = cx.selectable_text_props(props);

                    let row = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N1),
                        |cx| {
                            vec![
                                cx.text_props(fret_ui::element::TextProps {
                                    layout: Default::default(),
                                    text: Arc::<str>::from(format!("{}:", s.label)),
                                    style: None,
                                    color: Some(theme.color_required("muted-foreground")),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                }),
                                cx.container(
                                    decl_style::container_props(
                                        theme,
                                        ChromeRefinement::default()
                                            .border_1()
                                            .rounded(Radius::Md)
                                            .p(Space::N2)
                                            .bg(ColorRef::Color(
                                                theme.color_required("background"),
                                            )),
                                        LayoutRefinement::default().w_full(),
                                    ),
                                    move |_cx| vec![text],
                                ),
                            ]
                        },
                    );

                    row
                }));
            }

            out
        },
    );

    let diagnostic = {
        let state_for_handlers = state.clone();
        let on_down: fret_ui::action::OnPointerDown = Arc::new(move |host, action_cx, down| {
            let mut st = state_for_handlers.borrow_mut();
            st.pending_down = Some((down.position, down.modifiers.shift));
            st.last_drag_pos = Some(down.position);
            st.dragging = true;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        let state_for_handlers = state.clone();
        let on_move: fret_ui::action::OnPointerMove = Arc::new(move |host, action_cx, mv| {
            let mut st = state_for_handlers.borrow_mut();
            if st.dragging && mv.buttons.left {
                st.last_drag_pos = Some(mv.position);
                host.invalidate(fret_ui::Invalidation::Paint);
                host.request_redraw(action_cx.window);
            }
            true
        });

        let state_for_handlers = state.clone();
        let on_up: fret_ui::action::OnPointerUp = Arc::new(move |host, action_cx, _up| {
            let mut st = state_for_handlers.borrow_mut();
            st.dragging = false;
            host.invalidate(fret_ui::Invalidation::Paint);
            host.request_redraw(action_cx.window);
            true
        });

        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(220.0))),
            ),
            move |cx| {
                let mut pointer = fret_ui::element::PointerRegionProps::default();
                pointer.layout.size.width = fret_ui::element::Length::Fill;
                pointer.layout.size.height = fret_ui::element::Length::Fill;
                pointer.layout.overflow = fret_ui::element::Overflow::Clip;

                let paint_state = state.clone();

                let content = cx.pointer_region(pointer, move |cx| {
                    cx.pointer_region_on_pointer_down(on_down.clone());
                    cx.pointer_region_on_pointer_move(on_move.clone());
                    cx.pointer_region_on_pointer_up(on_up.clone());

                    let mut canvas = CanvasProps::default();
                    canvas.layout.size.width = fret_ui::element::Length::Fill;
                    canvas.layout.size.height = fret_ui::element::Length::Fill;
                    canvas.layout.overflow = fret_ui::element::Overflow::Clip;
                    canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let canvas = cx.canvas(canvas, move |p| {
                        fn format_utf8_context(text: &str, index: usize) -> String {
                            let idx = index.min(text.len());
                            let mut prev = 0usize;
                            let mut next = text.len();

                            for (i, _) in text.char_indices() {
                                if i <= idx {
                                    prev = i;
                                }
                                if i >= idx {
                                    next = i;
                                    break;
                                }
                            }

                            let left = text[..prev].chars().rev().take(12).collect::<String>();
                            let left = left.chars().rev().collect::<String>();
                            let right = text[next..].chars().take(12).collect::<String>();
                            format!("{left}|{right}")
                        }

                        let bounds = p.bounds();
                        let pad = Px(12.0);

                        let inner = Rect::new(
                            Point::new(
                                Px(bounds.origin.x.0 + pad.0),
                                Px(bounds.origin.y.0 + pad.0),
                            ),
                            Size::new(
                                Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                                Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                            ),
                        );

                        let max_width = inner.size.width;
                        if max_width.0 <= 0.0 || inner.size.height.0 <= 0.0 {
                            return;
                        }

                        let scale_factor = p.scale_factor();
                        let selection_bg = p.theme().color_required("selection.background");
                        let fg = p.theme().color_required("foreground");
                        let muted = p.theme().color_required("muted-foreground");

                        let (stats, stats_origin) = {
                            let (services, scene) = p.services_and_scene();
                            let mut st = paint_state.borrow_mut();

                            let sample = SAMPLES
                                .get(st.selected_sample)
                                .copied()
                                .unwrap_or(SAMPLES[0]);

                            let key = PreparedKey {
                                sample: st.selected_sample,
                                max_width_bits: max_width.0.to_bits(),
                                scale_bits: scale_factor.to_bits(),
                            };

                            let needs_prepare = st.blob.is_none()
                                || st.metrics.is_none()
                                || st.prepared_key != Some(key);
                            if needs_prepare {
                                if let Some(blob) = st.blob.take() {
                                    services.text().release(blob);
                                }

                                let style = TextStyle {
                                    font: FontId::ui(),
                                    size: Px(18.0),
                                    ..Default::default()
                                };

                                let constraints = TextConstraints {
                                    max_width: Some(max_width),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                    scale_factor,
                                };

                                let (blob, metrics) =
                                    services.text().prepare_str(sample.text, &style, constraints);
                                st.prepared_key = Some(key);
                                st.blob = Some(blob);
                                st.metrics = Some(metrics);
                                st.anchor = 0;
                                st.caret = 0;
                                st.affinity = CaretAffinity::Downstream;
                            }

                            let Some(blob) = st.blob else {
                                return;
                            };
                            let Some(metrics) = st.metrics else {
                                return;
                            };

                            let click_to_local = |global: Point| -> Point {
                                Point::new(
                                    Px(global.x.0 - inner.origin.x.0),
                                    Px(global.y.0 - inner.origin.y.0),
                                )
                            };

                            if let Some((pos, extend)) = st.pending_down.take() {
                                let local = click_to_local(pos);
                                let hit = services.hit_test_point(blob, local);
                                st.caret = hit.index;
                                st.affinity = hit.affinity;
                                if !extend {
                                    st.anchor = st.caret;
                                }
                            }

                            if st.dragging {
                                if let Some(pos) = st.last_drag_pos {
                                    let local = click_to_local(pos);
                                    let hit = services.hit_test_point(blob, local);
                                    st.caret = hit.index;
                                    st.affinity = hit.affinity;
                                }
                            }

                            let range = if st.anchor <= st.caret {
                                (st.anchor, st.caret)
                            } else {
                                (st.caret, st.anchor)
                            };

                            let clip = Rect::new(Point::new(Px(0.0), Px(0.0)), inner.size);
                            let mut rects: Vec<Rect> = Vec::new();
                            services.selection_rects_clipped(blob, range, clip, &mut rects);

                            scene.push(SceneOp::PushClipRect { rect: inner });
                            for r in rects {
                                let rect = Rect::new(
                                    Point::new(
                                        Px(inner.origin.x.0 + r.origin.x.0),
                                        Px(inner.origin.y.0 + r.origin.y.0),
                                    ),
                                    r.size,
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(0),
                                    rect,
                                    background: selection_bg,
                                    border: Edges::all(Px(0.0)),
                                    border_color: CoreColor::TRANSPARENT,
                                    corner_radii: Corners::all(Px(0.0)),
                                });
                            }

                            let text_origin = Point::new(inner.origin.x, Px(inner.origin.y.0 + metrics.baseline.0));
                            scene.push(SceneOp::Text {
                                order: DrawOrder(1),
                                origin: text_origin,
                                text: blob,
                                color: fg,
                            });

                            let caret_rect = services.caret_rect(blob, st.caret, st.affinity);
                            let caret_rect = Rect::new(
                                Point::new(
                                    Px(inner.origin.x.0 + caret_rect.origin.x.0),
                                    Px(inner.origin.y.0 + caret_rect.origin.y.0),
                                ),
                                caret_rect.size,
                            );
                            scene.push(SceneOp::Quad {
                                order: DrawOrder(2),
                                rect: caret_rect,
                                background: fg,
                                border: Edges::all(Px(0.0)),
                                border_color: CoreColor::TRANSPARENT,
                                corner_radii: Corners::all(Px(0.0)),
                            });

                            if let Some(pos) = st.last_drag_pos {
                                let dot = Rect::new(
                                    Point::new(Px(pos.x.0 - 2.0), Px(pos.y.0 - 2.0)),
                                    Size::new(Px(4.0), Px(4.0)),
                                );
                                scene.push(SceneOp::Quad {
                                    order: DrawOrder(3),
                                    rect: dot,
                                    background: fg,
                                    border: Edges::all(Px(0.0)),
                                    border_color: CoreColor::TRANSPARENT,
                                    corner_radii: Corners::all(Px(2.0)),
                                });
                            }

                            scene.push(SceneOp::PopClip);

                            let sample_text: &str = sample.text;
                            let context = format_utf8_context(sample_text, st.caret);

                            let stats = format!(
                                "sample: {} | caret: {} ({:?}) | anchor: {} | range: {:?} | context: {}",
                                sample.label, st.caret, st.affinity, st.anchor, range, context
                            );
                            let stats_origin = Point::new(
                                Px(bounds.origin.x.0 + 12.0),
                                Px(bounds.origin.y.0 + 10.0),
                            );
                            (stats, stats_origin)
                        };

                        let stats_style = TextStyle {
                            font: FontId::ui(),
                            size: Px(12.0),
                            ..Default::default()
                        };
                        let _ = p.text(
                            p.key(&"text_bidi_rtl_conformance_stats"),
                            DrawOrder(10),
                            stats_origin,
                            stats,
                            stats_style,
                            muted,
                            fret_ui::canvas::CanvasTextConstraints::default(),
                            scale_factor,
                        );
                    });

                    vec![canvas]
                });

                vec![content]
            },
        )
    };

    let panel = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from(
                "ui-gallery-text-bidi-rtl-conformance-root",
            )),
            ..Default::default()
        },
        |_cx| vec![sample_buttons, selectable_samples, diagnostic],
    );

    vec![header, panel]
}

fn preview_web_ime_harness(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    text_input: Model<String>,
    text_area: Model<String>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ImeHarnessState {
        committed: String,
        preedit: Option<String>,
        ime_enabled: bool,
        text_input_count: u64,
        ime_commit_count: u64,
        ime_preedit_count: u64,
        ime_delete_surrounding_count: u64,
        ime_enabled_count: u64,
        ime_disabled_count: u64,
        last: String,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(ImeHarnessState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: validate the wasm textarea IME bridge (ADR 0195)."),
                cx.text("Try: CJK IME preedit → commit; ensure no double insert on compositionend + input."),
                cx.text("Click inside the region to focus it (IME should enable)."),
            ]
        },
    );

    let inputs = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default().w_full(),
        ),
        |cx| {
            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2),
                |cx| {
                    vec![
                        cx.text("Editable widgets (sanity check):"),
                        shadcn::Input::new(text_input)
                            .a11y_label("Web IME input")
                            .placeholder("Type here (IME should work on web)")
                            .into_element(cx),
                        shadcn::Textarea::new(text_area)
                            .a11y_label("Web IME textarea")
                            .into_element(cx),
                    ]
                },
            );
            vec![body]
        },
    );

    let mut region_props = fret_ui::element::TextInputRegionProps::default();
    region_props.layout.size.width = fret_ui::element::Length::Fill;
    region_props.layout.size.height = fret_ui::element::Length::Fill;

    let region = cx.text_input_region(region_props, |cx| {
        let state_for_text_input = state.clone();
        cx.text_input_region_on_text_input(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  text: &str| {
                let mut st = state_for_text_input.borrow_mut();
                st.text_input_count = st.text_input_count.saturating_add(1);
                st.last = format!("TextInput({:?})", text);
                st.committed.push_str(text);
                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let state_for_ime = state.clone();
        cx.text_input_region_on_ime(std::sync::Arc::new(
            move |host: &mut dyn fret_ui::action::UiActionHost,
                  action_cx: fret_ui::action::ActionCx,
                  ime: &fret_core::ImeEvent| {
                let mut st = state_for_ime.borrow_mut();
                match ime {
                    fret_core::ImeEvent::Enabled => {
                        st.ime_enabled = true;
                        st.ime_enabled_count = st.ime_enabled_count.saturating_add(1);
                        st.last = "Ime(Enabled)".to_string();
                    }
                    fret_core::ImeEvent::Disabled => {
                        st.ime_enabled = false;
                        st.preedit = None;
                        st.ime_disabled_count = st.ime_disabled_count.saturating_add(1);
                        st.last = "Ime(Disabled)".to_string();
                    }
                    fret_core::ImeEvent::Commit(text) => {
                        st.ime_commit_count = st.ime_commit_count.saturating_add(1);
                        st.last = format!("Ime(Commit({:?}))", text);
                        st.committed.push_str(text);
                        st.preedit = None;
                    }
                    fret_core::ImeEvent::Preedit { text, .. } => {
                        st.ime_preedit_count = st.ime_preedit_count.saturating_add(1);
                        st.last = format!("Ime(Preedit({:?}))", text);
                        st.preedit = (!text.is_empty()).then(|| text.clone());
                    }
                    fret_core::ImeEvent::DeleteSurrounding {
                        before_bytes,
                        after_bytes,
                    } => {
                        st.ime_delete_surrounding_count =
                            st.ime_delete_surrounding_count.saturating_add(1);
                        st.last = format!(
                            "Ime(DeleteSurrounding(before_bytes={before_bytes}, after_bytes={after_bytes}))"
                        );
                    }
                }

                host.notify(action_cx);
                host.request_redraw(action_cx.window);
                true
            },
        ));

        let st = state.borrow();
        let committed_tail = {
            const MAX_CHARS: usize = 120;
            let total = st.committed.chars().count();
            if total <= MAX_CHARS {
                st.committed.clone()
            } else {
                let tail: String = st
                    .committed
                    .chars()
                    .skip(total.saturating_sub(MAX_CHARS))
                    .collect();
                format!("…{tail}")
            }
        };

        let preedit = st
            .preedit
            .as_deref()
            .unwrap_or("<none>");
        let ime_enabled = st.ime_enabled as u8;

        let panel = cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .border_1()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(theme.color_required("background"))),
                LayoutRefinement::default()
                    .w_full()
                    .h_px(MetricRef::Px(Px(240.0))),
            ),
            |cx| {
                let body = stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N2),
                    |cx| {
                        let mut lines = vec![
                            cx.text(format!("ime_enabled={ime_enabled}")),
                            cx.text(format!("preedit={preedit:?}")),
                            cx.text(format!("committed_tail={committed_tail:?}")),
                            cx.text(format!("last_event={:?}", st.last)),
                            cx.text(format!(
                                "counts: text_input={} ime_commit={} ime_preedit={} ime_delete_surrounding={} enabled={} disabled={}",
                                st.text_input_count,
                                st.ime_commit_count,
                                st.ime_preedit_count,
                                st.ime_delete_surrounding_count,
                                st.ime_enabled_count,
                                st.ime_disabled_count
                            )),
                        ];

                        let snapshot = cx
                            .app
                            .global::<fret_core::input::WebImeBridgeDebugSnapshot>()
                            .cloned();
                        if let Some(snapshot) = snapshot {
                            lines.push(cx.text("bridge_debug_snapshot (wasm textarea):"));
                            lines.push(cx.text(format!(
                                "  enabled={} composing={} suppress_next_input={}",
                                snapshot.enabled as u8,
                                snapshot.composing as u8,
                                snapshot.suppress_next_input as u8
                            )));
                            lines.push(cx.text(format!(
                                "  last_input_type={:?}",
                                snapshot.last_input_type.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_beforeinput_data={:?}",
                                snapshot.last_beforeinput_data.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_input_data={:?}",
                                snapshot.last_input_data.as_deref()
                            )));
                            lines.push(cx.text(format!(
                                "  last_key_code={:?} last_cursor_area={:?}",
                                snapshot.last_key_code, snapshot.last_cursor_area
                            )));
                            lines.push(cx.text(format!(
                                "  counts: beforeinput={} input={} suppressed={} comp_start={} comp_update={} comp_end={} cursor_area_set={}",
                                snapshot.beforeinput_seen,
                                snapshot.input_seen,
                                snapshot.suppressed_input_seen,
                                snapshot.composition_start_seen,
                                snapshot.composition_update_seen,
                                snapshot.composition_end_seen,
                                snapshot.cursor_area_set_seen,
                            )));
                        } else {
                            lines.push(cx.text("bridge_debug_snapshot: <unavailable>"));
                        }

                        lines
                    },
                );
                vec![body]
            },
        );

        vec![panel]
    });

    vec![header, inputs, region]
}

fn preview_text_measure_overlay(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    #[derive(Clone, Copy)]
    struct Case {
        label: &'static str,
        text: &'static str,
        wrap: TextWrap,
        overflow: TextOverflow,
        height: Px,
    }

    const CASES: &[Case] = &[
        Case {
            label: "Wrap=None, Overflow=Clip (expect overflow past measured width)",
            text: "Left (fill) • A_very_long_token_without_spaces_that_should_not_wrap_but_can_overflow_the_box",
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            height: Px(56.0),
        },
        Case {
            label: "Wrap=Word, Overflow=Clip (expect multi-line height growth)",
            text: "Word wrap should break on spaces and increase measured height when max_width is tight.",
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=Grapheme, Overflow=Clip (expect long tokens to wrap)",
            text: "GraphemeWrap: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa (and emoji 😀😀😀) should wrap without whitespace.",
            wrap: TextWrap::Grapheme,
            overflow: TextOverflow::Clip,
            height: Px(88.0),
        },
        Case {
            label: "Wrap=None, Overflow=Ellipsis (expect measured width ~= max_width)",
            text: "Ellipsis overflow should clamp the visual width and replace the suffix…",
            wrap: TextWrap::None,
            overflow: TextOverflow::Ellipsis,
            height: Px(56.0),
        },
    ];

    #[derive(Default)]
    struct MeasureOverlayState {
        last_metrics: Vec<Option<fret_core::TextMetrics>>,
    }

    let state = cx.with_state(
        || std::rc::Rc::new(std::cell::RefCell::new(MeasureOverlayState::default())),
        |st| st.clone(),
    );

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: visualize measured text bounds vs allocated container bounds."),
                cx.text("Green = container bounds; Yellow = measured TextMetrics.size; Cyan = baseline."),
            ]
        },
    );

    let panel = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .border_1()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(theme.color_required("background"))),
            LayoutRefinement::default()
                .w_full()
                .h_px(MetricRef::Px(Px(440.0))),
        ),
        move |cx| {
            let mut canvas = CanvasProps::default();
            canvas.layout.size.width = fret_ui::element::Length::Fill;
            canvas.layout.size.height = fret_ui::element::Length::Fill;
            canvas.layout.overflow = fret_ui::element::Overflow::Clip;
            canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            let paint_state = state.clone();

            let canvas = cx.canvas(canvas, move |p| {
                let bounds = p.bounds();
                let pad = Px(14.0);
                let gap = Px(14.0);

                let outer = Rect::new(
                    Point::new(Px(bounds.origin.x.0 + pad.0), Px(bounds.origin.y.0 + pad.0)),
                    Size::new(
                        Px((bounds.size.width.0 - 2.0 * pad.0).max(0.0)),
                        Px((bounds.size.height.0 - 2.0 * pad.0).max(0.0)),
                    ),
                );
                if outer.size.width.0 <= 0.0 || outer.size.height.0 <= 0.0 {
                    return;
                }

                let green = fret_core::Color {
                    r: 0.20,
                    g: 0.85,
                    b: 0.35,
                    a: 1.0,
                };
                let yellow = fret_core::Color {
                    r: 0.95,
                    g: 0.85,
                    b: 0.10,
                    a: 1.0,
                };
                let cyan = fret_core::Color {
                    r: 0.10,
                    g: 0.80,
                    b: 0.95,
                    a: 1.0,
                };

                let fg = p.theme().color_required("foreground");
                let muted = p.theme().color_required("muted-foreground");
                let bg = p.theme().color_required("background");
                let border = p.theme().color_required("border");

                let scale = p.scale_factor();
                let mut y = outer.origin.y;
                let scope = p.key_scope(&"text_measure_overlay");

                let mut st = paint_state.borrow_mut();
                if st.last_metrics.len() < CASES.len() {
                    st.last_metrics.resize(CASES.len(), None);
                }

                for (i, case) in CASES.iter().enumerate() {
                    let case_rect = Rect::new(
                        Point::new(outer.origin.x, y),
                        Size::new(outer.size.width, case.height),
                    );

                    // Case chrome.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(0),
                        rect: case_rect,
                        background: bg,
                        border: Edges::all(Px(1.0)),
                        border_color: border,
                        corner_radii: Corners::all(Px(8.0)),
                    });

                    let label_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(12.0),
                        ..Default::default()
                    };
                    let label_metrics = p.text(
                        p.child_key(scope, &format!("label_{i}")).0,
                        DrawOrder(1),
                        Point::new(case_rect.origin.x + Px(10.0), case_rect.origin.y + Px(16.0)),
                        case.label,
                        label_style,
                        muted,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(Px((case_rect.size.width.0 - 20.0).max(0.0))),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Clip,
                        },
                        scale,
                    );

                    let text_box = Rect::new(
                        Point::new(
                            case_rect.origin.x + Px(10.0),
                            Px(case_rect.origin.y.0 + 16.0 + label_metrics.size.height.0 + 8.0),
                        ),
                        Size::new(
                            Px((case_rect.size.width.0 - 20.0).max(0.0)),
                            Px((case_rect.size.height.0
                                - 16.0
                                - label_metrics.size.height.0
                                - 18.0)
                                .max(0.0)),
                        ),
                    );

                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(1),
                        rect: text_box,
                        background: CoreColor::TRANSPARENT,
                        border: Edges::all(Px(1.0)),
                        border_color: green,
                        corner_radii: Corners::all(Px(6.0)),
                    });

                    let text_style = TextStyle {
                        font: FontId::ui(),
                        size: Px(16.0),
                        ..Default::default()
                    };

                    let baseline_y = match st.last_metrics[i] {
                        Some(m) => text_box.origin.y + m.baseline,
                        None => text_box.origin.y + Px(text_style.size.0 * 0.8),
                    };

                    let metrics = p.text(
                        p.child_key(scope, &format!("text_{i}")).0,
                        DrawOrder(2),
                        Point::new(text_box.origin.x, baseline_y),
                        case.text,
                        text_style,
                        fg,
                        fret_ui::canvas::CanvasTextConstraints {
                            max_width: Some(text_box.size.width),
                            wrap: case.wrap,
                            overflow: case.overflow,
                        },
                        scale,
                    );
                    st.last_metrics[i] = Some(metrics);

                    // Baseline.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(3),
                        rect: Rect::new(
                            Point::new(text_box.origin.x, text_box.origin.y + metrics.baseline),
                            Size::new(text_box.size.width, Px(1.0)),
                        ),
                        background: cyan,
                        border: Edges::all(Px(0.0)),
                        border_color: CoreColor::TRANSPARENT,
                        corner_radii: Corners::all(Px(0.0)),
                    });

                    // Measured text box.
                    p.scene().push(SceneOp::Quad {
                        order: DrawOrder(4),
                        rect: Rect::new(text_box.origin, metrics.size),
                        background: CoreColor::TRANSPARENT,
                        border: Edges::all(Px(1.0)),
                        border_color: yellow,
                        corner_radii: Corners::all(Px(0.0)),
                    });

                    y = Px(y.0 + case.height.0 + gap.0);
                    if y.0 >= outer.origin.y.0 + outer.size.height.0 {
                        break;
                    }
                }
            });

            vec![canvas]
        },
    );

    let panel = cx.semantics(
        fret_ui::element::SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-text-measure-overlay-root")),
            ..Default::default()
        },
        |_cx| vec![panel],
    );

    vec![header, panel]
}

fn preview_chart_torture(cx: &mut ElementContext<'_, App>, _theme: &Theme) -> Vec<AnyElement> {
    use delinea::data::{Column, DataTable};
    use delinea::engine::ChartEngine;
    use delinea::{
        AxisKind, AxisPointerSpec, AxisPointerTrigger, AxisPointerType, AxisRange, AxisScale,
        ChartSpec, DatasetSpec, FieldSpec, GridSpec, SeriesEncode, SeriesKind, SeriesSpec,
        TimeAxisScale,
    };
    use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};
    use fret_ui::element::SemanticsProps;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress canvas charts with pan/zoom (candidate for prepaint-windowed sampling)."),
                cx.text("Use scripted drag+wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    struct EngineState {
        model: Option<Model<ChartEngine>>,
        spec: Option<ChartSpec>,
    }

    impl Default for EngineState {
        fn default() -> Self {
            Self {
                model: None,
                spec: None,
            }
        }
    }

    let existing = cx.with_state(EngineState::default, |st| {
        match (st.model.clone(), st.spec.clone()) {
            (Some(engine), Some(spec)) => Some((engine, spec)),
            _ => None,
        }
    });

    let (engine, spec) = if let Some((engine, spec)) = existing {
        (engine, spec)
    } else {
        let dataset_id = delinea::ids::DatasetId::new(1);
        let grid_id = delinea::ids::GridId::new(1);
        let x_axis = delinea::AxisId::new(1);
        let y_axis = delinea::AxisId::new(2);
        let series_id = delinea::ids::SeriesId::new(1);
        let x_field = delinea::FieldId::new(1);
        let y_field = delinea::FieldId::new(2);

        let spec = ChartSpec {
            id: delinea::ids::ChartId::new(1),
            viewport: None,
            datasets: vec![DatasetSpec {
                id: dataset_id,
                fields: vec![
                    FieldSpec {
                        id: x_field,
                        column: 0,
                    },
                    FieldSpec {
                        id: y_field,
                        column: 1,
                    },
                ],
            }],
            grids: vec![GridSpec { id: grid_id }],
            axes: vec![
                delinea::AxisSpec {
                    id: x_axis,
                    name: Some("Time".to_string()),
                    kind: AxisKind::X,
                    grid: grid_id,
                    position: None,
                    scale: AxisScale::Time(TimeAxisScale),
                    range: Some(AxisRange::Auto),
                },
                delinea::AxisSpec {
                    id: y_axis,
                    name: Some("Value".to_string()),
                    kind: AxisKind::Y,
                    grid: grid_id,
                    position: None,
                    scale: Default::default(),
                    range: Some(AxisRange::Auto),
                },
            ],
            data_zoom_x: vec![],
            data_zoom_y: vec![],
            tooltip: None,
            axis_pointer: Some(AxisPointerSpec {
                enabled: true,
                trigger: AxisPointerTrigger::Axis,
                pointer_type: AxisPointerType::Line,
                label: Default::default(),
                snap: false,
                trigger_distance_px: 12.0,
                throttle_px: 0.75,
            }),
            visual_maps: vec![],
            series: vec![SeriesSpec {
                id: series_id,
                name: Some("Series".to_string()),
                kind: SeriesKind::Line,
                dataset: dataset_id,
                encode: SeriesEncode {
                    x: x_field,
                    y: y_field,
                    y2: None,
                },
                x_axis,
                y_axis,
                stack: None,
                stack_strategy: Default::default(),
                bar_layout: Default::default(),
                area_baseline: None,
                lod: None,
            }],
        };

        let mut engine = ChartEngine::new(spec.clone()).expect("chart spec should be valid");

        let base_ms = 1_735_689_600_000.0;
        let interval_ms = 60_000.0;

        let n = 200_000usize;
        let mut x: Vec<f64> = Vec::with_capacity(n);
        let mut y: Vec<f64> = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            let xi = base_ms + interval_ms * i as f64;
            let theta = t * std::f64::consts::TAU;
            let yi = (theta * 8.0).sin() * 0.8;
            x.push(xi);
            y.push(yi);
        }

        let mut table = DataTable::default();
        table.push_column(Column::F64(x));
        table.push_column(Column::F64(y));
        engine.datasets_mut().insert(dataset_id, table);

        let engine = cx.app.models_mut().insert(engine);
        cx.with_state(EngineState::default, |st| {
            st.model = Some(engine.clone());
            st.spec = Some(spec.clone());
        });

        (engine, spec)
    };

    cx.observe_model(&engine, Invalidation::Paint);

    let mut props = ChartCanvasPanelProps::new(spec);
    props.engine = Some(engine);
    props.input_map = fret_chart::input_map::ChartInputMap::default();

    let chart = chart_canvas_panel(cx, props);
    let chart = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-chart-torture-root")),
            ..Default::default()
        },
        |_cx| vec![chart],
    );

    vec![header, chart]
}

fn preview_canvas_cull_torture(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    use fret_canvas::ui::{
        PanZoomCanvasSurfacePanelProps, PanZoomInputPreset, pan_zoom_canvas_surface_panel,
    };
    use fret_canvas::view::{PanZoom2D, visible_canvas_rect};
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{CanvasCachePolicy, Length, SemanticsProps};
    use std::cmp::Ordering;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: stress a pan/zoom canvas scene with viewport-driven culling (candidate for prepaint-windowed cull windows)."),
                cx.text("Use scripted middle-drag + wheel steps to validate correctness and collect perf bundles."),
            ]
        },
    );

    let canvas =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");
            let grid = theme.color_required("border");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(11.0),
                ..Default::default()
            };

            let mut props = PanZoomCanvasSurfacePanelProps::default();
            props.preset = PanZoomInputPreset::DesktopCanvasCad;
            props.pointer_region.layout.size.width = Length::Fill;
            props.pointer_region.layout.size.height = Length::Px(Px(520.0));
            props.canvas.cache_policy = CanvasCachePolicy::smooth_default();
            props.default_view = PanZoom2D {
                pan: fret_core::Point::new(Px(0.0), Px(0.0)),
                zoom: 1.0,
            };
            props.min_zoom = 0.05;
            props.max_zoom = 64.0;

            let cell_size = 48.0f32;
            let cell_pad = 3.0f32;
            let max_cells = 40_000i64;

            let canvas = pan_zoom_canvas_surface_panel(cx, props, move |painter, paint_cx| {
                let bounds = painter.bounds();

                let Some(transform) = paint_cx.view.render_transform(bounds) else {
                    return;
                };

                let vis = visible_canvas_rect(bounds, paint_cx.view);
                let min_x = vis.origin.x.0;
                let max_x = vis.origin.x.0 + vis.size.width.0;
                let min_y = vis.origin.y.0;
                let max_y = vis.origin.y.0 + vis.size.height.0;

                let start_x = (min_x / cell_size).floor() as i32 - 2;
                let end_x = (max_x / cell_size).ceil() as i32 + 2;
                let start_y = (min_y / cell_size).floor() as i32 - 2;
                let end_y = (max_y / cell_size).ceil() as i32 + 2;

                let x_count = (end_x - start_x + 1).max(0) as i64;
                let y_count = (end_y - start_y + 1).max(0) as i64;
                let estimated = x_count.saturating_mul(y_count);

                let stride = match estimated.cmp(&max_cells) {
                    Ordering::Less | Ordering::Equal => 1i32,
                    Ordering::Greater => {
                        ((estimated as f64 / max_cells as f64).ceil() as i32).max(1)
                    }
                };

                let clip = bounds;
                painter.with_clip_rect(clip, |painter| {
                    painter.with_transform(transform, |painter| {
                        let scope = painter.key_scope(&"ui-gallery-canvas-cull");

                        let mut y = start_y;
                        while y <= end_y {
                            let mut x = start_x;
                            while x <= end_x {
                                let ox = x as f32 * cell_size + cell_pad;
                                let oy = y as f32 * cell_size + cell_pad;
                                let size = cell_size - cell_pad * 2.0;
                                if size.is_finite() && size > 0.0 {
                                    let rect = fret_core::Rect::new(
                                        fret_core::Point::new(Px(ox), Px(oy)),
                                        fret_core::Size::new(Px(size), Px(size)),
                                    );

                                    let background =
                                        if ((x ^ y) & 1) == 0 { bg_even } else { bg_odd };
                                    painter.scene().push(fret_core::SceneOp::Quad {
                                        order: DrawOrder(0),
                                        rect,
                                        background,
                                        border: Edges::all(Px(1.0)),
                                        border_color: grid,
                                        corner_radii: Corners::all(Px(4.0)),
                                    });

                                    if x == 0 && y == 0 {
                                        painter.scene().push(fret_core::SceneOp::Quad {
                                            order: DrawOrder(1),
                                            rect,
                                            background: fret_core::Color::TRANSPARENT,
                                            border: Edges::all(Px(2.0)),
                                            border_color: fg,
                                            corner_radii: Corners::all(Px(4.0)),
                                        });
                                    }

                                    if (x % 20) == 0 && (y % 20) == 0 {
                                        let key: u64 = painter.child_key(scope, &(x, y)).into();
                                        let label = format!("{x},{y}");
                                        let origin = fret_core::Point::new(
                                            Px(rect.origin.x.0 + 6.0),
                                            Px(rect.origin.y.0 + 6.0),
                                        );
                                        let _ = painter.text(
                                            key,
                                            DrawOrder(2),
                                            origin,
                                            label,
                                            text_style.clone(),
                                            fg,
                                            CanvasTextConstraints {
                                                max_width: Some(Px(
                                                    (rect.size.width.0 - 12.0).max(0.0)
                                                )),
                                                wrap: TextWrap::None,
                                                overflow: TextOverflow::Clip,
                                            },
                                            painter.scale_factor(),
                                        );
                                    }
                                }

                                x = x.saturating_add(stride);
                            }
                            y = y.saturating_add(stride);
                        }
                    });
                });
            });

            vec![cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-canvas-cull-root")),
                    ..Default::default()
                },
                |_cx| vec![canvas],
            )]
        });

    vec![header, canvas]
}

fn preview_chrome_torture(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui::element::SemanticsProps;

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: exercise hover/focus/pressed chrome under view-cache + shell."),
                cx.text(
                    "This page intentionally mixes many focusable widgets and overlay triggers.",
                ),
            ]
        },
    );

    let content = cx.semantics(
        SemanticsProps {
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::<str>::from("ui-gallery-chrome-torture-root")),
            ..Default::default()
        },
        |cx| {
            let body = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4),
                |cx| {
                    let mut out = Vec::new();

                    out.extend(preview_overlay(
                        cx,
                        popover_open,
                        dialog_open,
                        alert_dialog_open,
                        sheet_open,
                        portal_geometry_popover_open,
                        dropdown_open,
                        context_menu_open,
                        context_menu_edge_open,
                        last_action,
                    ));

                    let controls = stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full())
                            .gap(Space::N3),
                        |cx| {
                            let mut out: Vec<AnyElement> = Vec::new();

                            let row = stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N2).items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Button::new("One")
                                            .test_id("ui-gallery-chrome-btn-1")
                                            .into_element(cx),
                                        shadcn::Button::new("Two")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .test_id("ui-gallery-chrome-btn-2")
                                            .into_element(cx),
                                        shadcn::Button::new("Three")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("ui-gallery-chrome-btn-3")
                                            .into_element(cx),
                                        shadcn::Button::new("Disabled")
                                            .disabled(true)
                                            .test_id("ui-gallery-chrome-btn-disabled")
                                            .into_element(cx),
                                    ]
                                },
                            );
                            out.push(row);

                            let fields = stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N2).items_start(),
                                |cx| {
                                    vec![
                                        stack::vstack(
                                            cx,
                                            stack::VStackProps::default().gap(Space::N1),
                                            |cx| {
                                                let input = shadcn::Input::new(text_input.clone())
                                                    .a11y_label("Chrome torture input")
                                                    .placeholder("Type")
                                                    .into_element(cx);
                                                let input = cx.semantics(
                                                    SemanticsProps {
                                                        role: fret_core::SemanticsRole::TextField,
                                                        test_id: Some(Arc::<str>::from(
                                                            "ui-gallery-chrome-text-input",
                                                        )),
                                                        ..Default::default()
                                                    },
                                                    |_cx| vec![input],
                                                );
                                                vec![cx.text("Text input"), input]
                                            },
                                        ),
                                        stack::vstack(
                                            cx,
                                            stack::VStackProps::default().gap(Space::N1),
                                            |cx| {
                                                let textarea =
                                                    shadcn::Textarea::new(text_area.clone())
                                                        .a11y_label("Chrome torture textarea")
                                                        .into_element(cx);
                                                let textarea = cx.semantics(
                                                    SemanticsProps {
                                                        role: fret_core::SemanticsRole::TextField,
                                                        test_id: Some(Arc::<str>::from(
                                                            "ui-gallery-chrome-text-area",
                                                        )),
                                                        ..Default::default()
                                                    },
                                                    |_cx| vec![textarea],
                                                );
                                                vec![cx.text("Text area"), textarea]
                                            },
                                        ),
                                    ]
                                },
                            );
                            out.push(fields);

                            let toggles = stack::hstack(
                                cx,
                                stack::HStackProps::default().gap(Space::N3).items_center(),
                                |cx| {
                                    vec![
                                        shadcn::Checkbox::new(checkbox.clone())
                                            .a11y_label("Chrome torture checkbox")
                                            .test_id("ui-gallery-chrome-checkbox")
                                            .into_element(cx),
                                        shadcn::Switch::new(switch.clone())
                                            .a11y_label("Chrome torture switch")
                                            .test_id("ui-gallery-chrome-switch")
                                            .into_element(cx),
                                    ]
                                },
                            );
                            out.push(toggles);

                            out
                        },
                    );
                    out.push(controls);

                    out
                },
            );

            vec![body]
        },
    );

    vec![header, content]
}

fn preview_windowed_rows_surface_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use fret_core::{
        Corners, DrawOrder, Edges, FontId, SemanticsRole, TextOverflow, TextStyle, TextWrap,
    };
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfaceProps, windowed_rows_surface,
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline scroll windowing via a stable element tree (Scroll + Canvas)."),
                cx.text("This is the 'single-node surface' escape hatch: paint only visible rows, avoid per-row subtrees."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let mut props = WindowedRowsSurfaceProps::default();
            props.scroll.layout.size.width = fret_ui::element::Length::Fill;
            props.scroll.layout.size.height = fret_ui::element::Length::Px(Px(420.0));
            props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
            props.len = len;
            props.row_height = row_h;
            props.overscan = overscan;
            props.scroll_handle = scroll_handle.clone();
            props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

            vec![cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-windowed-rows-root")),
                    ..Default::default()
                },
                |cx| {
                    vec![windowed_rows_surface(
                        cx,
                        props,
                        move |painter, index, rect| {
                            let background = if (index % 2) == 0 { bg_even } else { bg_odd };
                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background,
                                border: Edges::all(Px(0.0)),
                                border_color: fret_core::Color::TRANSPARENT,
                                corner_radii: Corners::all(Px(0.0)),
                            });

                            let label = format!("Row {index}");
                            let origin = fret_core::Point::new(
                                Px(rect.origin.x.0 + 8.0),
                                Px(rect.origin.y.0 + 4.0),
                            );
                            let scope = painter.key_scope(&"ui-gallery-windowed-rows");
                            let key: u64 = painter.child_key(scope, &index).into();
                            let _ = painter.text(
                                key,
                                DrawOrder(1),
                                origin,
                                label,
                                text_style.clone(),
                                fg,
                                CanvasTextConstraints {
                                    max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                                    wrap: TextWrap::None,
                                    overflow: TextOverflow::Clip,
                                },
                                painter.scale_factor(),
                            );
                        },
                    )]
                },
            )]
        });

    vec![header, surface]
}

fn preview_windowed_rows_surface_interactive_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    use std::cell::RefCell;
    use std::rc::Rc;

    use fret_core::{Corners, CursorIcon, DrawOrder, Edges, FontId, SemanticsRole, TextStyle};
    use fret_ui::Invalidation;
    use fret_ui::action::{ActionCx, PointerDownCx, PointerMoveCx};
    use fret_ui::canvas::CanvasTextConstraints;
    use fret_ui::element::{Length, PointerRegionProps, SemanticsProps};
    use fret_ui_kit::declarative::windowed_rows_surface::{
        WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
        windowed_rows_surface_with_pointer_region,
    };

    #[derive(Default)]
    struct RowChromeState {
        hovered: Option<usize>,
        selected: Option<usize>,
    }

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: demonstrate paint-only hover/selection chrome on a prepaint-windowed row surface (ADR 0190 + ADR 0181)."),
                cx.text("Pattern: stable tree (Scroll + PointerRegion + Canvas), row hit-testing in pointer hooks, paint-only visuals in Canvas."),
            ]
        },
    );

    let len = 200_000usize;
    let row_h = Px(22.0);
    let overscan = 16usize;

    let scroll_handle = cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());

    let surface =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let bg_even = theme.color_required("background");
            let bg_odd = theme.color_required("muted");
            let bg_hover = theme.color_required("accent");
            let fg = theme.color_required("foreground");

            let text_style = TextStyle {
                font: FontId::monospace(),
                size: Px(12.0),
                ..Default::default()
            };

            let root = cx.semantics_with_id(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-windowed-rows-interactive-root")),
                    ..Default::default()
                },
                move |cx, root_id| {
                    let state = cx.with_state_for(
                        root_id,
                        || Rc::new(RefCell::new(RowChromeState::default())),
                        |s| s.clone(),
                    );

                    let on_move_state = state.clone();
                    let on_pointer_move: fret_ui_kit::declarative::windowed_rows_surface::OnWindowedRowsPointerMove =
                        Arc::new(move |host, action_cx: ActionCx, idx, _mv: PointerMoveCx| {
                            host.set_cursor_icon(CursorIcon::Pointer);
                            let mut st = on_move_state.borrow_mut();
                            if st.hovered == idx {
                                return true;
                            }
                            st.hovered = idx;
                            host.invalidate(Invalidation::Paint);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let on_down_state = state.clone();
                    let on_pointer_down: fret_ui_kit::declarative::windowed_rows_surface::OnWindowedRowsPointerDown =
                        Arc::new(move |host, action_cx: ActionCx, idx, down: PointerDownCx| {
                            if down.button != fret_core::MouseButton::Left {
                                return false;
                            }
                            let mut st = on_down_state.borrow_mut();
                            st.selected = Some(idx);
                            st.hovered = Some(idx);
                            host.invalidate(Invalidation::Paint);
                            host.request_redraw(action_cx.window);
                            true
                        });

                    let handlers = WindowedRowsSurfacePointerHandlers {
                        on_pointer_down: Some(on_pointer_down),
                        on_pointer_move: Some(on_pointer_move),
                        ..Default::default()
                    };

                    let mut props = WindowedRowsSurfaceProps::default();
                    props.scroll.layout.size.width = Length::Fill;
                    props.scroll.layout.size.height = Length::Px(Px(420.0));
                    props.scroll.layout.overflow = fret_ui::element::Overflow::Clip;
                    props.len = len;
                    props.row_height = row_h;
                    props.overscan = overscan;
                    props.scroll_handle = scroll_handle.clone();
                    props.canvas.cache_policy = fret_ui::element::CanvasCachePolicy::smooth_default();

                    let mut pointer = PointerRegionProps::default();
                    pointer.layout.size.width = Length::Fill;
                    pointer.layout.size.height = Length::Fill;

                    let paint_state = state.clone();
                    let content_semantics = SemanticsProps {
                        role: SemanticsRole::Group,
                        test_id: Some(Arc::<str>::from(
                            "ui-gallery-windowed-rows-interactive-canvas",
                        )),
                        ..Default::default()
                    };

                    vec![windowed_rows_surface_with_pointer_region(
                        cx,
                        props,
                        pointer,
                        handlers,
                        Some(content_semantics),
                        move |painter, index, rect| {
                            let st = paint_state.borrow();
                            let hovered = st.hovered == Some(index);
                            let selected = st.selected == Some(index);

                            let background = if hovered || selected {
                                bg_hover
                            } else if (index % 2) == 0 {
                                bg_even
                            } else {
                                bg_odd
                            };

                            painter.scene().push(fret_core::SceneOp::Quad {
                                order: DrawOrder(0),
                                rect,
                                background,
                                border: if selected {
                                    Edges::all(Px(1.0))
                                } else {
                                    Edges::all(Px(0.0))
                                },
                                border_color: if selected {
                                    fg
                                } else {
                                    fret_core::Color::TRANSPARENT
                                },
                                corner_radii: Corners::all(Px(0.0)),
                            });

                            let label = format!("Row {index}");
                            let origin = fret_core::Point::new(
                                Px(rect.origin.x.0 + 8.0),
                                Px(rect.origin.y.0 + 4.0),
                            );
                            let scope = painter.key_scope(&"ui-gallery-windowed-rows-interactive");
                            let key: u64 = painter.child_key(scope, &index).into();
                            let _ = painter.text(
                                key,
                                DrawOrder(1),
                                origin,
                                label,
                                text_style.clone(),
                                fg,
                                CanvasTextConstraints {
                                    max_width: Some(Px(rect.size.width.0.max(0.0) - 16.0)),
                                    wrap: fret_core::TextWrap::None,
                                    overflow: fret_core::TextOverflow::Clip,
                                },
                                painter.scale_factor(),
                            );
                        },
                    )]
                },
            );

            vec![root]
        });

    vec![header, surface]
}

fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let variants = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default").into_element(cx),
                shadcn::Button::new("Secondary")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .into_element(cx),
                shadcn::Button::new("Outline")
                    .variant(shadcn::ButtonVariant::Outline)
                    .into_element(cx),
                shadcn::Button::new("Ghost")
                    .variant(shadcn::ButtonVariant::Ghost)
                    .into_element(cx),
                shadcn::Button::new("Destructive")
                    .variant(shadcn::ButtonVariant::Destructive)
                    .into_element(cx),
                shadcn::Button::new("Disabled")
                    .disabled(true)
                    .into_element(cx),
            ]
        },
    );

    let sizes = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Small")
                    .size(shadcn::ButtonSize::Sm)
                    .into_element(cx),
                shadcn::Button::new("Default")
                    .size(shadcn::ButtonSize::Default)
                    .into_element(cx),
                shadcn::Button::new("Large")
                    .size(shadcn::ButtonSize::Lg)
                    .into_element(cx),
            ]
        },
    );

    vec![variants, sizes]
}

fn preview_shadcn_placeholder(
    cx: &mut ElementContext<'_, App>,
    name: &'static str,
) -> Vec<AnyElement> {
    vec![cx.text(format!("{name}: gallery preview stub (expand as needed)."))]
}

fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![
        shadcn::Alert::new([
            shadcn::AlertTitle::new("Heads up!").into_element(cx),
            shadcn::AlertDescription::new("You can add components to your app.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::Alert::new([
            shadcn::AlertTitle::new("Error").into_element(cx),
            shadcn::AlertDescription::new("Something went wrong.").into_element(cx),
        ])
        .variant(shadcn::AlertVariant::Destructive)
        .into_element(cx),
    ]
}

fn preview_checkbox(cx: &mut ElementContext<'_, App>, model: Model<bool>) -> Vec<AnyElement> {
    let checked = cx
        .get_model_copied(&model, Invalidation::Layout)
        .unwrap_or(false);
    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Checkbox::new(model)
                    .a11y_label("Accept terms")
                    .into_element(cx),
                ui::label(cx, "Accept terms").into_element(cx),
                cx.text(format!("checked={}", checked as u8)),
            ]
        },
    )]
}

fn preview_switch(cx: &mut ElementContext<'_, App>, model: Model<bool>) -> Vec<AnyElement> {
    let on = cx
        .get_model_copied(&model, Invalidation::Layout)
        .unwrap_or(false);
    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Switch::new(model)
                    .a11y_label("Enable feature")
                    .into_element(cx),
                ui::label(cx, "Enable feature").into_element(cx),
                cx.text(format!("on={}", on as u8)),
            ]
        },
    )]
}

fn preview_input(cx: &mut ElementContext<'_, App>, value: Model<String>) -> Vec<AnyElement> {
    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        move |cx| {
            vec![
                shadcn::Label::new("Email").into_element(cx),
                shadcn::Input::new(value)
                    .a11y_label("Email")
                    .placeholder("name@example.com")
                    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                    .into_element(cx),
            ]
        },
    )]
}

fn preview_textarea(cx: &mut ElementContext<'_, App>, value: Model<String>) -> Vec<AnyElement> {
    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        move |cx| {
            vec![
                shadcn::Label::new("Message").into_element(cx),
                shadcn::Textarea::new(value)
                    .a11y_label("Message")
                    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)).h_px(Px(120.0)))
                    .into_element(cx),
            ]
        },
    )]
}

fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        move |cx| {
            vec![
                shadcn::Label::new("Label").into_element(cx),
                shadcn::Label::new("Required label").into_element(cx),
            ]
        },
    )]
}

fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Kbd::new("Ctrl").into_element(cx),
                shadcn::Kbd::new("K").into_element(cx),
                shadcn::KbdGroup::new([
                    shadcn::Kbd::new("⌘").into_element(cx),
                    shadcn::Kbd::new("P").into_element(cx),
                ])
                .into_element(cx),
            ]
        },
    )]
}

fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    cx.text("Left"),
                    shadcn::Separator::new()
                        .orientation(shadcn::SeparatorOrientation::Vertical)
                        .flex_stretch_cross_axis(true)
                        .into_element(cx),
                    cx.text("Right"),
                ]
            },
        ),
        shadcn::Separator::new().into_element(cx),
    ]
}

fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(24.0)).h_px(Px(24.0)))
                    .into_element(cx),
                shadcn::Spinner::new().speed(0.0).into_element(cx),
            ]
        },
    )]
}

fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let theme = Theme::global(&*cx.app).clone();
    let muted = theme.color_required("muted");
    let child = cx.container(
        fret_ui::element::ContainerProps {
            background: Some(muted),
            ..Default::default()
        },
        |cx| vec![cx.text("16:9")],
    );

    vec![
        shadcn::AspectRatio::new(16.0 / 9.0, child)
            .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
            .into_element(cx),
    ]
}

fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    vec![
        shadcn::Breadcrumb::new()
            .items([
                shadcn::BreadcrumbItem::new("Home"),
                shadcn::BreadcrumbItem::new("Components"),
                shadcn::BreadcrumbItem::new("Breadcrumb"),
            ])
            .into_element(cx),
        shadcn::Breadcrumb::new()
            .items([
                shadcn::BreadcrumbItem::new("Home"),
                shadcn::BreadcrumbItem::ellipsis(),
                shadcn::BreadcrumbItem::new("Examples"),
                shadcn::BreadcrumbItem::new("Data Fetching"),
            ])
            .into_element(cx),
    ]
}

fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let group = shadcn::ButtonGroup::new([
        shadcn::Button::new("Left").into(),
        shadcn::Button::new("Middle").into(),
        shadcn::Button::new("Right").into(),
    ])
    .a11y_label("Button group")
    .into_element(cx);

    vec![group]
}

fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let calendar = shadcn::Calendar::new(month, selected)
        .number_of_months(1)
        .into_element(cx);
    vec![calendar]
}

fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let collapsible = shadcn::Collapsible::uncontrolled(false).into_element_with_open_model(
        cx,
        |cx, open, is_open| {
            let label = if is_open {
                "Hide details"
            } else {
                "Show details"
            };
            shadcn::Button::new(label)
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open)
                .into_element(cx)
        },
        |cx| {
            shadcn::CollapsibleContent::new(vec![
                cx.text("This content is toggled by a Collapsible."),
                cx.text("Use it for disclosure panels, advanced options, etc."),
            ])
            .refine_layout(LayoutRefinement::default().w_full())
            .into_element(cx)
        },
    );

    vec![collapsible]
}

fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![
        shadcn::Drawer::new_controllable(cx, None, false).into_element(
            cx,
            |cx| shadcn::Button::new("Open drawer").into_element(cx),
            |cx| {
                shadcn::DrawerContent::new(vec![
                    shadcn::DrawerHeader::new(vec![
                        shadcn::DrawerTitle::new("Drawer").into_element(cx),
                        shadcn::DrawerDescription::new("A bottom sheet-style overlay.")
                            .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::DrawerFooter::new(vec![shadcn::Button::new("Done").into_element(cx)])
                        .into_element(cx),
                ])
                .into_element(cx)
            },
        ),
    ]
}

fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let trigger = shadcn::Button::new("Hover me")
        .variant(shadcn::ButtonVariant::Outline)
        .into_element(cx);
    let content = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("HoverCard").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![
            cx.text("HoverCard content lives in a hover overlay."),
            cx.text("Useful for previews, profile cards, etc."),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_px(Px(260.0)))
    .into_element(cx);

    vec![shadcn::HoverCard::new_controllable(cx, None, false, trigger, content).into_element(cx)]
}

fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct InputGroupModels {
        value: Option<Model<String>>,
    }

    let value = cx.with_state(InputGroupModels::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(InputGroupModels::default, |st| {
                st.value = Some(model.clone())
            });
            model
        }
    };

    let group = shadcn::InputGroup::new(value)
        .a11y_label("Search")
        .leading([shadcn::InputGroupText::new("Search").into_element(cx)])
        .trailing([shadcn::InputGroupButton::new("Go")
            .on_click(CMD_APP_OPEN)
            .into_element(cx)])
        .into_element(cx);

    vec![group]
}

fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct InputOtpModels {
        value: Option<Model<String>>,
    }

    let value = cx.with_state(InputOtpModels::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(String::new());
            cx.with_state(InputOtpModels::default, |st| st.value = Some(model.clone()));
            model
        }
    };

    let otp = shadcn::InputOtp::new(value).length(6).into_element(cx);
    vec![otp]
}

fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use shadcn::{MenubarEntry, MenubarItem, MenubarMenu};

    let file = MenubarMenu::new("File").entries([
        MenubarEntry::Item(MenubarItem::new("Open").on_select(CMD_APP_OPEN)),
        MenubarEntry::Item(MenubarItem::new("Save").on_select(CMD_APP_SAVE)),
        MenubarEntry::Separator,
        MenubarEntry::Item(MenubarItem::new("Settings").on_select(CMD_APP_SETTINGS)),
    ]);

    let edit = MenubarMenu::new("Edit").entries([
        MenubarEntry::Item(MenubarItem::new("Undo").on_select(fret_app::core_commands::EDIT_UNDO)),
        MenubarEntry::Item(MenubarItem::new("Redo").on_select(fret_app::core_commands::EDIT_REDO)),
    ]);

    vec![shadcn::Menubar::new([file, edit]).into_element(cx)]
}

fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let intro = shadcn::NavigationMenuItem::new(
        "intro",
        "Introduction",
        vec![
            cx.text("A basic NavigationMenu item."),
            cx.text("Content is shown in a popover."),
        ],
    );
    let components = shadcn::NavigationMenuItem::new(
        "components",
        "Components",
        vec![
            cx.text("This demo is intentionally lightweight."),
            cx.text("We can expand it to match shadcn/ui registry examples."),
        ],
    );

    vec![
        shadcn::NavigationMenu::uncontrolled(Some("intro"))
            .indicator(true)
            .list(shadcn::NavigationMenuList::new([intro, components]))
            .into_element(cx),
    ]
}

fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let content = shadcn::PaginationContent::new([
        shadcn::PaginationItem::new(
            shadcn::PaginationPrevious::new()
                .on_click(CMD_APP_OPEN)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([cx.text("1")])
                .on_click(CMD_APP_OPEN)
                .active(true)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([cx.text("2")])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(shadcn::PaginationEllipsis::new().into_element(cx))
            .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationLink::new([cx.text("10")])
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
        shadcn::PaginationItem::new(
            shadcn::PaginationNext::new()
                .on_click(CMD_APP_SAVE)
                .into_element(cx),
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let pagination = shadcn::Pagination::new([content]).into_element(cx);

    vec![pagination]
}

fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct RadioGroupModels {
        value: Option<Model<Option<Arc<str>>>>,
    }

    let model = cx.with_state(RadioGroupModels::default, |st| st.value.clone());
    let model = match model {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(RadioGroupModels::default, |st| {
                st.value = Some(model.clone())
            });
            model
        }
    };

    let group = shadcn::RadioGroup::new(model.clone())
        .a11y_label("Options")
        .item(shadcn::RadioGroupItem::new("default", "Default"))
        .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
        .item(shadcn::RadioGroupItem::new("compact", "Compact"))
        .into_element(cx);

    let value = cx
        .get_model_cloned(&model, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![group, cx.text(format!("value={}", value.as_ref()))]
}

fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ToggleModels {
        pressed: Option<Model<bool>>,
    }

    let pressed = cx.with_state(ToggleModels::default, |st| st.pressed.clone());
    let pressed = match pressed {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ToggleModels::default, |st| st.pressed = Some(model.clone()));
            model
        }
    };

    vec![stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                shadcn::Toggle::new(pressed.clone())
                    .label("Bold")
                    .a11y_label("Bold")
                    .into_element(cx),
                shadcn::Toggle::new(pressed.clone())
                    .label("Outline")
                    .variant(shadcn::ToggleVariant::Outline)
                    .a11y_label("Bold (outline)")
                    .into_element(cx),
            ]
        },
    )]
}

fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct ToggleGroupModels {
        value: Option<Model<Option<Arc<str>>>>,
    }

    let value = cx.with_state(ToggleGroupModels::default, |st| st.value.clone());
    let value = match value {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(ToggleGroupModels::default, |st| {
                st.value = Some(model.clone())
            });
            model
        }
    };

    let group = shadcn::ToggleGroup::single(value.clone())
        .item(shadcn::ToggleGroupItem::new(
            "bold",
            [ui::label(cx, "B").into_element(cx)],
        ))
        .item(shadcn::ToggleGroupItem::new(
            "italic",
            [ui::label(cx, "I").into_element(cx)],
        ))
        .item(shadcn::ToggleGroupItem::new(
            "underline",
            [ui::label(cx, "U").into_element(cx)],
        ))
        .into_element(cx);

    let selected = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![group, cx.text(format!("selected={}", selected.as_ref()))]
}

fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N3).items_start(),
        move |cx| {
            vec![
                shadcn::typography::h1(cx, "The Joke Tax Chronicles"),
                shadcn::typography::lead(cx, "Once upon a time, in a far-off land..."),
                shadcn::typography::p(
                    cx,
                    "The king, seeing how much happier his subjects were, realized the error of his ways and repealed the joke tax.",
                ),
                shadcn::typography::blockquote(
                    cx,
                    "After all, everyone enjoys a good joke, so it's only fair that they should pay for the privilege.",
                ),
                shadcn::typography::inline_code(cx, "cargo run -p fret-ui-gallery"),
            ]
        },
    )]
}

fn preview_alert_dialog(cx: &mut ElementContext<'_, App>, open: Model<bool>) -> Vec<AnyElement> {
    let open_for_children = open.clone();
    let dialog = shadcn::AlertDialog::new(open).into_element(
        cx,
        |cx| shadcn::Button::new("Open alert dialog").into_element(cx),
        |cx| {
            shadcn::AlertDialogContent::new(vec![
                shadcn::AlertDialogHeader::new(vec![
                    shadcn::AlertDialogTitle::new("Are you absolutely sure?").into_element(cx),
                    shadcn::AlertDialogDescription::new(
                        "This action cannot be undone. This will permanently delete your data.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::AlertDialogFooter::new(vec![
                    shadcn::AlertDialogCancel::new("Cancel", open_for_children.clone())
                        .into_element(cx),
                    shadcn::AlertDialogAction::new("Continue", open_for_children.clone())
                        .into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    vec![dialog]
}

fn preview_dialog(cx: &mut ElementContext<'_, App>, open: Model<bool>) -> Vec<AnyElement> {
    let open_for_close = open.clone();
    let dialog = shadcn::Dialog::new(open).into_element(
        cx,
        |cx| shadcn::Button::new("Open dialog").into_element(cx),
        |cx| {
            shadcn::DialogContent::new(vec![
                shadcn::DialogHeader::new(vec![
                    shadcn::DialogTitle::new("Edit profile").into_element(cx),
                    shadcn::DialogDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .into_element(cx),
                ])
                .into_element(cx),
                shadcn::DialogFooter::new(vec![
                    shadcn::Button::new("Save changes").into_element(cx),
                    shadcn::DialogClose::new(open_for_close.clone()).into_element(cx),
                ])
                .into_element(cx),
            ])
            .into_element(cx)
        },
    );

    vec![dialog]
}

fn preview_popover(cx: &mut ElementContext<'_, App>, open: Model<bool>) -> Vec<AnyElement> {
    vec![shadcn::Popover::new(open).into_element(
        cx,
        |cx| shadcn::Button::new("Open popover").into_element(cx),
        |cx| {
            shadcn::PopoverContent::new(vec![
                shadcn::PopoverTitle::new("Popover").into_element(cx),
                shadcn::PopoverDescription::new("Non-modal overlay content.").into_element(cx),
            ])
            .into_element(cx)
        },
    )]
}

fn preview_sheet(cx: &mut ElementContext<'_, App>, open: Model<bool>) -> Vec<AnyElement> {
    vec![shadcn::Sheet::new(open).into_element(
        cx,
        |cx| shadcn::Button::new("Open sheet").into_element(cx),
        |cx| {
            shadcn::SheetContent::new(vec![
                shadcn::SheetHeader::new(vec![
                    shadcn::SheetTitle::new("Sheet").into_element(cx),
                    shadcn::SheetDescription::new("A side panel overlay.").into_element(cx),
                ])
                .into_element(cx),
                shadcn::SheetFooter::new(vec![shadcn::Button::new("Done").into_element(cx)])
                    .into_element(cx),
            ])
            .into_element(cx)
        },
    )]
}

fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    vec![
        shadcn::Empty::new([
            shadcn::empty::EmptyHeader::new([
                shadcn::empty::EmptyTitle::new("No results.").into_element(cx),
                shadcn::empty::EmptyDescription::new("Try adjusting your filters.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::empty::EmptyContent::new([
                shadcn::Button::new("Clear filters").into_element(cx)
            ])
            .into_element(cx),
        ])
        .into_element(cx),
    ]
}

fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let theme = fret_ui::Theme::global(&*cx.app).clone();

    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::ButtonVariant,
               label: &'static str| {
        let theme = theme.clone();
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                let hover_container = theme.color_required("md.sys.color.tertiary-container");
                let hover_label = theme.color_required("md.sys.color.on-tertiary-container");
                let hover_style = material3::ButtonStyle::default()
                    .container_background(WidgetStateProperty::new(None).when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(hover_container)),
                    ))
                    .label_color(
                        WidgetStateProperty::new(None)
                            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_label))),
                    );

                let accent = fret_core::Color {
                    r: 0.9,
                    g: 0.2,
                    b: 0.9,
                    a: 1.0,
                };
                let override_style = material3::ButtonStyle::default()
                    .label_color(WidgetStateProperty::new(Some(ColorRef::Color(accent))))
                    .state_layer_color(
                        WidgetStateProperty::new(None)
                            .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
                    );
                vec![
                    material3::Button::new(label)
                        .variant(variant)
                        .into_element(cx),
                    material3::Button::new("Override")
                        .variant(variant)
                        .style(override_style)
                        .into_element(cx),
                    material3::Button::new("Disabled")
                        .variant(variant)
                        .disabled(true)
                        .into_element(cx),
                    material3::Button::new("Hover Override")
                        .variant(variant)
                        .style(hover_style)
                        .into_element(cx),
                ]
            },
        )
    };

    vec![
        cx.text("Material 3 Buttons: token-driven colors + state layer + bounded ripple."),
        row(cx, material3::ButtonVariant::Filled, "Filled"),
        row(cx, material3::ButtonVariant::Tonal, "Tonal"),
        row(cx, material3::ButtonVariant::Elevated, "Elevated"),
        row(cx, material3::ButtonVariant::Outlined, "Outlined"),
        row(cx, material3::ButtonVariant::Text, "Text"),
    ]
}

fn preview_material3_gallery(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_list_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    _last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let disabled = cx
        .get_model_copied(&material3_text_field_disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error = cx
        .get_model_copied(&material3_text_field_error, Invalidation::Layout)
        .unwrap_or(false);

    let mut out: Vec<AnyElement> = Vec::new();
    out.push(cx.text("Material 3 Gallery: compact outcomes-first surface."));

    out.push(cx.text("— Buttons —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                material3::Button::new("Filled")
                    .variant(material3::ButtonVariant::Filled)
                    .into_element(cx),
                material3::Button::new("Tonal")
                    .variant(material3::ButtonVariant::Tonal)
                    .into_element(cx),
                material3::Button::new("Outlined")
                    .variant(material3::ButtonVariant::Outlined)
                    .into_element(cx),
                material3::Button::new("Text")
                    .variant(material3::ButtonVariant::Text)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Icon Buttons —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let hover_icon = fret_ui_shadcn::ColorRef::Color(
                theme.color_required("md.sys.color.on-tertiary-container"),
            );
            let hover_container = fret_ui_shadcn::ColorRef::Color(
                theme.color_required("md.sys.color.tertiary-container"),
            );
            let hover_style = material3::IconButtonStyle::default()
                .container_background(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_container)),
                )
                .icon_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                );

            vec![
                material3::IconButton::new(ids::ui::SEARCH)
                    .a11y_label("Search")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::SETTINGS)
                    .a11y_label("Settings")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CLOSE)
                    .a11y_label("Close")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::SEARCH)
                    .a11y_label("Override")
                    .style(hover_style)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Selection —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N3).items_center(),
        |cx| {
            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let hover_container = fret_ui_shadcn::ColorRef::Color(
                theme.color_required("md.sys.color.tertiary-container"),
            );
            let hover_icon = fret_ui_shadcn::ColorRef::Color(
                theme.color_required("md.sys.color.on-tertiary-container"),
            );
            let hover_outline =
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"));
            let hover_style = material3::CheckboxStyle::default()
                .container_background(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_container)),
                )
                .icon_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                )
                .outline_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_outline)),
                );

            vec![
                material3::Checkbox::new(material3_checkbox.clone())
                    .a11y_label("Checkbox")
                    .into_element(cx),
                material3::Checkbox::new(material3_checkbox.clone())
                    .a11y_label("Checkbox Override")
                    .style(hover_style)
                    .into_element(cx),
                material3::Switch::new(material3_switch.clone())
                    .a11y_label("Switch")
                    .into_element(cx),
                material3::Switch::new(material3_switch.clone())
                    .a11y_label("Switch Override")
                    .style({
                        let theme = fret_ui::Theme::global(&*cx.app).clone();
                        let hover_track = fret_ui_shadcn::ColorRef::Color(
                            theme.color_required("md.sys.color.tertiary-container"),
                        );
                        let hover_handle = fret_ui_shadcn::ColorRef::Color(
                            theme.color_required("md.sys.color.on-tertiary-container"),
                        );
                        material3::SwitchStyle::default()
                            .track_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_track)),
                            )
                            .handle_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_handle)),
                            )
                    })
                    .into_element(cx),
                stack::vstack(
                    cx,
                    stack::VStackProps::default().gap(Space::N1).items_start(),
                    |cx| {
                        let items = vec![
                            material3::RadioGroupItem::new("Alpha").a11y_label("Radio Alpha"),
                            material3::RadioGroupItem::new("Beta").a11y_label("Radio Beta"),
                            material3::RadioGroupItem::new("Charlie")
                                .a11y_label("Radio Charlie")
                                .disabled(true),
                        ];

                        let theme = fret_ui::Theme::global(&*cx.app).clone();
                        let hover_icon = fret_ui_shadcn::ColorRef::Color(
                            theme.color_required("md.sys.color.tertiary"),
                        );
                        let hover_state_layer = fret_ui_shadcn::ColorRef::Color(
                            theme.color_required("md.sys.color.tertiary"),
                        );
                        let hover_style = material3::RadioStyle::default()
                            .icon_color(
                                fret_ui_kit::WidgetStateProperty::new(None)
                                    .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_icon)),
                            )
                            .state_layer_color(
                                fret_ui_kit::WidgetStateProperty::new(None).when(
                                    fret_ui_kit::WidgetStates::HOVERED,
                                    Some(hover_state_layer),
                                ),
                            );

                        vec![
                            cx.text("Radio Group"),
                            material3::RadioGroup::new(material3_radio_value.clone())
                                .a11y_label("Radio Group")
                                .items(items.clone())
                                .into_element(cx),
                            cx.text("Radio Group Override"),
                            material3::RadioGroup::new(material3_radio_value.clone())
                                .a11y_label("Radio Group Override")
                                .style(hover_style)
                                .items(items)
                                .into_element(cx),
                        ]
                    },
                ),
            ]
        },
    ));

    out.push(cx.text("— Text Field —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Switch::new(material3_text_field_disabled.clone())
                    .a11y_label("Disable Text Field")
                    .into_element(cx),
                cx.text("Disabled"),
                shadcn::Switch::new(material3_text_field_error.clone())
                    .a11y_label("Text Field Error")
                    .into_element(cx),
                cx.text("Error"),
            ]
        },
    ));
    out.push(stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N1).items_start(),
        |cx| {
            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let hover =
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"));
            let hover_style = material3::TextFieldStyle::default()
                .outline_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover.clone())),
                )
                .label_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover)),
                );

            vec![
                cx.text("Text Field"),
                material3::TextField::new(material3_text_field_value.clone())
                    .label("Label")
                    .placeholder("Placeholder")
                    .disabled(disabled)
                    .error(error)
                    .into_element(cx),
                cx.text("Text Field Override"),
                material3::TextField::new(material3_text_field_value)
                    .label("Label")
                    .placeholder("Placeholder")
                    .style(hover_style)
                    .disabled(disabled)
                    .error(error)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Tabs —"));
    out.push(stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N1).items_start(),
        |cx| {
            let items = vec![
                material3::TabItem::new("overview", "Overview"),
                material3::TabItem::new("security", "Security"),
                material3::TabItem::new("settings", "Settings"),
            ];

            let theme = fret_ui::Theme::global(&*cx.app).clone();
            let hover_label =
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"));
            let hover_state_layer =
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"));
            let indicator =
                fret_ui_shadcn::ColorRef::Color(theme.color_required("md.sys.color.tertiary"));
            let hover_style = material3::TabsStyle::default()
                .label_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_label)),
                )
                .state_layer_color(
                    fret_ui_kit::WidgetStateProperty::new(None)
                        .when(fret_ui_kit::WidgetStates::HOVERED, Some(hover_state_layer)),
                )
                .active_indicator_color(fret_ui_kit::WidgetStateProperty::new(Some(indicator)));

            vec![
                cx.text("Tabs"),
                material3::Tabs::new(material3_tabs_value.clone())
                    .a11y_label("Tabs")
                    .items(items.clone())
                    .into_element(cx),
                cx.text("Tabs Override"),
                material3::Tabs::new(material3_tabs_value)
                    .a11y_label("Tabs Override")
                    .style(hover_style)
                    .items(items)
                    .into_element(cx),
            ]
        },
    ));

    out.push(cx.text("— Navigation Bar —"));
    out.push(
        material3::NavigationBar::new(material3_navigation_bar_value)
            .a11y_label("Navigation bar")
            .items(vec![
                material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH),
                material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS),
                material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL),
            ])
            .into_element(cx),
    );

    out.push(cx.text("— List —"));
    out.push(
        material3::List::new(material3_list_value)
            .a11y_label("List")
            .items(vec![
                material3::ListItem::new("alpha", "Alpha").leading_icon(ids::ui::SEARCH),
                material3::ListItem::new("beta", "Beta").leading_icon(ids::ui::SETTINGS),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true),
            ])
            .into_element(cx),
    );

    out
}

fn preview_material3_state_matrix(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();

    out.push(cx.text(
        "Material 3 State Matrix: exercise hover/focus/press/disabled/selected states across multiple components.",
    ));
    out.push(cx.text(
        "Tip: use keyboard Tab/Arrow/Home/End on Tabs/Radio/Menu; use Esc/outside press to close Menu.",
    ));

    out.extend(material3_state_matrix_content(
        cx,
        material3_checkbox,
        material3_switch,
        material3_radio_value,
        material3_tabs_value,
        material3_navigation_bar_value,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
        material3_menu_open,
        last_action,
    ));
    out
}

fn material3_state_matrix_content(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
    material3_navigation_bar_value: Model<Arc<str>>,
    material3_text_field_value: Model<String>,
    material3_text_field_disabled: Model<bool>,
    material3_text_field_error: Model<bool>,
    material3_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let mut out: Vec<AnyElement> = Vec::new();

    out.push(cx.text("— Buttons —"));
    out.extend(preview_material3_button(cx));

    out.push(cx.text("— Chips —"));
    out.extend(preview_material3_chip(cx, last_action.clone()));

    out.push(cx.text("— Cards —"));
    out.extend(preview_material3_card(cx, last_action.clone()));

    out.push(cx.text("— Icon Buttons —"));
    out.extend(preview_material3_icon_button(cx));

    out.push(cx.text("— Checkbox —"));
    out.extend(preview_material3_checkbox(cx, material3_checkbox));

    out.push(cx.text("— Switch —"));
    out.extend(preview_material3_switch(cx, material3_switch));

    out.push(cx.text("— Radio —"));
    out.extend(preview_material3_radio(cx, material3_radio_value));

    out.push(cx.text("— Text Field —"));
    out.extend(preview_material3_text_field(
        cx,
        material3_text_field_value,
        material3_text_field_disabled,
        material3_text_field_error,
    ));

    out.push(cx.text("— Tabs —"));
    out.extend(preview_material3_tabs(cx, material3_tabs_value));

    out.push(cx.text("— Navigation Bar —"));
    out.extend(preview_material3_navigation_bar(
        cx,
        material3_navigation_bar_value,
    ));

    out.push(cx.text("— Menu —"));
    out.extend(preview_material3_menu(cx, material3_menu_open, last_action));

    out
}

fn preview_material3_chip(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    #[derive(Default)]
    struct ChipPageModels {
        filter_selected: Option<Model<bool>>,
        filter_unselected: Option<Model<bool>>,
        input_selected: Option<Model<bool>>,
        input_unselected: Option<Model<bool>>,
    }

    let filter_selected = cx.with_state(ChipPageModels::default, |st| st.filter_selected.clone());
    let filter_selected = match filter_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(ChipPageModels::default, |st| {
                st.filter_selected = Some(model.clone())
            });
            model
        }
    };

    let filter_unselected =
        cx.with_state(ChipPageModels::default, |st| st.filter_unselected.clone());
    let filter_unselected = match filter_unselected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ChipPageModels::default, |st| {
                st.filter_unselected = Some(model.clone())
            });
            model
        }
    };

    let input_selected = cx.with_state(ChipPageModels::default, |st| st.input_selected.clone());
    let input_selected = match input_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(true);
            cx.with_state(ChipPageModels::default, |st| {
                st.input_selected = Some(model.clone())
            });
            model
        }
    };

    let input_unselected = cx.with_state(ChipPageModels::default, |st| st.input_unselected.clone());
    let input_unselected = match input_unselected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(ChipPageModels::default, |st| {
                st.input_unselected = Some(model.clone())
            });
            model
        }
    };

    let theme = Theme::global(&*cx.app).clone();

    let last_action_for_activate = last_action.clone();
    let activate: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&last_action_for_activate, |v| {
            *v = Arc::<str>::from("material3.assist_chip.activated");
        });
    });

    let hover_container = theme.color_required("md.sys.color.tertiary-container");
    let hover_label = theme.color_required("md.sys.color.on-tertiary-container");
    let accent = fret_core::Color {
        r: 0.9,
        g: 0.2,
        b: 0.9,
        a: 1.0,
    };

    let override_style = material3::AssistChipStyle::default()
        .label_color(WidgetStateProperty::new(Some(ColorRef::Color(accent))))
        .state_layer_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        )
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        )
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ));

    let hover_style = material3::AssistChipStyle::default()
        .label_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_label))),
        )
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ));

    let filter_override_style = material3::FilterChipStyle::default()
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(hover_container)),
        ))
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(accent))),
        );

    let activate_row1 = activate.clone();
    let activate_row2 = activate.clone();
    let activate_row3 = activate.clone();
    let _activate_row4 = activate.clone();

    let last_action_for_input_selected = last_action.clone();
    let activate_input_selected_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_selected, |v| {
                *v = Arc::<str>::from("material3.input_chip.primary.activated");
            });
    });

    let last_action_for_input_unselected = last_action.clone();
    let activate_input_unselected_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_unselected, |v| {
                *v = Arc::<str>::from("material3.input_chip.primary.activated");
            });
    });

    let last_action_for_input_unselected_trailing = last_action.clone();
    let activate_input_unselected_trailing: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_input_unselected_trailing, |v| {
                *v = Arc::<str>::from("material3.input_chip.trailing_icon.activated");
            });
    });

    let override_style_row1 = override_style.clone();
    let override_style_row2 = override_style.clone();
    let hover_style_row1 = hover_style.clone();
    let hover_style_row2 = hover_style.clone();
    let filter_override_style_row = filter_override_style.clone();

    let last_action_for_filter_primary = last_action.clone();
    let activate_filter_primary: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_filter_primary, |v| {
                *v = Arc::<str>::from("material3.filter_chip.primary.activated");
            });
    });

    let last_action_for_filter_trailing = last_action.clone();
    let activate_filter_trailing: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host
            .models_mut()
            .update(&last_action_for_filter_trailing, |v| {
                *v = Arc::<str>::from("material3.filter_chip.trailing_icon.activated");
            });
    });

    let filter_selected_row1 = filter_selected.clone();
    let filter_unselected_row1 = filter_unselected.clone();
    let filter_selected_row2 = filter_selected.clone();
    let filter_unselected_row2 = filter_unselected.clone();
    let input_selected_row1 = input_selected.clone();
    let input_unselected_row1 = input_unselected.clone();
    let input_unselected_row2 = input_unselected.clone();

    vec![
        cx.text("Material 3 AssistChip: token-driven shape/colors + state layer + bounded ripple."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::AssistChip::new("Flat")
                        .on_activate(activate_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat")
                        .into_element(cx),
                    material3::AssistChip::new("Override")
                        .on_activate(activate_row1.clone())
                        .style(override_style_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat-override")
                        .into_element(cx),
                    material3::AssistChip::new("Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-chip-flat-disabled")
                        .into_element(cx),
                    material3::AssistChip::new("Hover Override")
                        .on_activate(activate_row1.clone())
                        .style(hover_style_row1.clone())
                        .test_id("ui-gallery-material3-chip-flat-hover-override")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::AssistChip::new("Elevated")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated")
                        .into_element(cx),
                    material3::AssistChip::new("Override")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .style(override_style_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated-override")
                        .into_element(cx),
                    material3::AssistChip::new("Disabled")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SLASH)
                        .disabled(true)
                        .test_id("ui-gallery-material3-chip-elevated-disabled")
                        .into_element(cx),
                    material3::AssistChip::new("Hover Override")
                        .variant(material3::AssistChipVariant::Elevated)
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_row2.clone())
                        .style(hover_style_row2.clone())
                        .test_id("ui-gallery-material3-chip-elevated-hover-override")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::SuggestionChip::new("Suggestion")
                        .on_activate(activate_row3.clone())
                        .test_id("ui-gallery-material3-suggestion-chip-flat")
                        .into_element(cx),
                    material3::SuggestionChip::new("Suggestion (icon)")
                        .leading_icon(ids::ui::SEARCH)
                        .variant(material3::SuggestionChipVariant::Elevated)
                        .on_activate(activate_row3.clone())
                        .test_id("ui-gallery-material3-suggestion-chip-elevated")
                        .into_element(cx),
                    material3::SuggestionChip::new("Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-suggestion-chip-disabled")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::FilterChip::new(filter_selected_row1.clone(), "Filter")
                        .trailing_icon(ids::ui::CLOSE)
                        .on_activate(activate_filter_primary.clone())
                        .on_trailing_icon_activate(activate_filter_trailing.clone())
                        .test_id("ui-gallery-material3-filter-chip-selected")
                        .into_element(cx),
                    material3::FilterChip::new(filter_unselected_row1.clone(), "Filter")
                        .on_activate(activate_filter_primary.clone())
                        .test_id("ui-gallery-material3-filter-chip-unselected")
                        .into_element(cx),
                    material3::FilterChip::new(filter_selected_row2.clone(), "Override")
                        .variant(material3::FilterChipVariant::Elevated)
                        .style(filter_override_style_row.clone())
                        .on_activate(activate_filter_primary.clone())
                        .test_id("ui-gallery-material3-filter-chip-override")
                        .into_element(cx),
                    material3::FilterChip::new(filter_unselected_row2.clone(), "Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-filter-chip-disabled")
                        .into_element(cx),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::InputChip::new(input_selected_row1.clone(), "Input")
                        .leading_icon(ids::ui::SETTINGS)
                        .on_activate(activate_input_selected_primary.clone())
                        .test_id("ui-gallery-material3-input-chip-selected")
                        .into_element(cx),
                    material3::InputChip::new(input_unselected_row1.clone(), "Input")
                        .trailing_icon(ids::ui::CLOSE)
                        .on_activate(activate_input_unselected_primary.clone())
                        .on_trailing_icon_activate(activate_input_unselected_trailing.clone())
                        .test_id("ui-gallery-material3-input-chip-unselected")
                        .into_element(cx),
                    material3::InputChip::new(input_unselected_row2.clone(), "Disabled")
                        .disabled(true)
                        .test_id("ui-gallery-material3-input-chip-disabled")
                        .into_element(cx),
                ]
            },
        ),
    ]
}

fn preview_material3_card(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui::element::{ContainerProps, Length, TextProps};
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let theme = Theme::global(&*cx.app).clone();

    let activate: OnActivate = Arc::new(move |host, _acx, _reason| {
        let _ = host.models_mut().update(&last_action, |v| {
            *v = Arc::<str>::from("material3.card.activated");
        });
    });

    let body_style = theme
        .text_style_by_key("md.sys.typescale.body-medium")
        .unwrap_or_else(|| fret_core::TextStyle::default());
    let body_color = theme.color_required("md.sys.color.on-surface");

    let hover_container = theme.color_required("md.sys.color.tertiary-container");
    let hover_outline = theme.color_required("md.sys.color.tertiary");

    let override_style = material3::CardStyle::default()
        .container_background(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(hover_container)),
        ))
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_outline))),
        );

    let activate_row1 = activate.clone();
    let activate_row2 = activate.clone();
    let override_style_row1 = override_style.clone();
    let override_style_row2 = override_style.clone();

    let card_content_row1 = {
        let body_style = body_style.clone();
        let body_color = body_color;
        move |cx: &mut ElementContext<'_, App>, label: &'static str| {
            let mut container = ContainerProps::default();
            container.layout.size.width = Length::Px(Px(280.0));
            container.layout.size.height = Length::Px(Px(72.0));
            container.padding = Edges::all(Px(12.0));

            let mut text = TextProps::new(Arc::<str>::from(label));
            text.style = Some(body_style.clone());
            text.color = Some(body_color);
            cx.container(container, move |cx| vec![cx.text_props(text)])
        }
    };

    let card_content_row2 = {
        let body_style = body_style.clone();
        let body_color = body_color;
        move |cx: &mut ElementContext<'_, App>, label: &'static str| {
            let mut container = ContainerProps::default();
            container.layout.size.width = Length::Px(Px(280.0));
            container.layout.size.height = Length::Px(Px(72.0));
            container.padding = Edges::all(Px(12.0));

            let mut text = TextProps::new(Arc::<str>::from(label));
            text.style = Some(body_style.clone());
            text.color = Some(body_color);
            cx.container(container, move |cx| vec![cx.text_props(text)])
        }
    };

    vec![
        cx.text("Material 3 Card: token-driven surface + outline + ink (interactive only when on_activate is set)."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .test_id("ui-gallery-material3-card-filled")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Filled")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .style(override_style_row1.clone())
                        .test_id("ui-gallery-material3-card-filled-override")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Override")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Filled)
                        .on_activate(activate_row1.clone())
                        .disabled(true)
                        .test_id("ui-gallery-material3-card-filled-disabled")
                        .into_element(cx, |cx| vec![card_content_row1(cx, "Disabled")]),
                ]
            },
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Card::new()
                        .variant(material3::CardVariant::Elevated)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-card-elevated")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Elevated")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Outlined)
                        .on_activate(activate_row2.clone())
                        .test_id("ui-gallery-material3-card-outlined")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Outlined")]),
                    material3::Card::new()
                        .variant(material3::CardVariant::Outlined)
                        .on_activate(activate_row2.clone())
                        .style(override_style_row2.clone())
                        .test_id("ui-gallery-material3-card-outlined-override")
                        .into_element(cx, |cx| vec![card_content_row2(cx, "Outline override")]),
                ]
            },
        ),
    ]
}

fn preview_material3_touch_targets(
    cx: &mut ElementContext<'_, App>,
    material3_checkbox: Model<bool>,
    material3_switch: Model<bool>,
    material3_radio_value: Model<Option<Arc<str>>>,
    material3_tabs_value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let theme = Theme::global(&*cx.app).clone();
    let min = theme
        .metric_by_key("md.sys.layout.minimum-touch-target.size")
        .unwrap_or(Px(48.0));

    let target_overlay = |cx: &mut ElementContext<'_, App>,
                          label: &'static str,
                          chrome: Option<Size>,
                          child: AnyElement| {
        let min = min;

        let stack = cx.stack_props(
            StackProps {
                layout: {
                    let mut l = fret_ui::element::LayoutStyle::default();
                    l.overflow = fret_ui::element::Overflow::Visible;
                    l
                },
            },
            move |cx| {
                let mut canvas = CanvasProps::default();
                canvas.layout.position = fret_ui::element::PositionStyle::Absolute;
                canvas.layout.inset.top = Some(Px(0.0));
                canvas.layout.inset.right = Some(Px(0.0));
                canvas.layout.inset.bottom = Some(Px(0.0));
                canvas.layout.inset.left = Some(Px(0.0));

                let overlay = cx.canvas(canvas, move |p| {
                    let bounds = p.bounds();
                    let center = Point::new(
                        Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                        Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
                    );

                    let min_rect = Rect::new(
                        Point::new(Px(center.x.0 - min.0 * 0.5), Px(center.y.0 - min.0 * 0.5)),
                        Size::new(min, min),
                    );

                    let chrome_rect = chrome.map(|chrome| {
                        Rect::new(
                            Point::new(
                                Px(center.x.0 - chrome.width.0 * 0.5),
                                Px(center.y.0 - chrome.height.0 * 0.5),
                            ),
                            chrome,
                        )
                    });

                    fn outline(
                        p: &mut fret_ui::canvas::CanvasPainter<'_>,
                        order: u32,
                        rect: Rect,
                        color: CoreColor,
                    ) {
                        p.scene().push(SceneOp::Quad {
                            order: DrawOrder(order),
                            rect,
                            background: CoreColor::TRANSPARENT,
                            border: Edges::all(Px(1.0)),
                            border_color: color,
                            corner_radii: Corners::all(Px(0.0)),
                        });
                    }

                    outline(
                        p,
                        0,
                        bounds,
                        CoreColor {
                            r: 0.1,
                            g: 0.8,
                            b: 0.2,
                            a: 0.8,
                        },
                    );
                    outline(
                        p,
                        1,
                        min_rect,
                        CoreColor {
                            r: 0.95,
                            g: 0.75,
                            b: 0.2,
                            a: 0.9,
                        },
                    );
                    if let Some(chrome_rect) = chrome_rect {
                        outline(
                            p,
                            2,
                            chrome_rect,
                            CoreColor {
                                r: 0.2,
                                g: 0.75,
                                b: 0.95,
                                a: 0.9,
                            },
                        );
                    }
                });

                vec![child, overlay]
            },
        );

        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new(label).into_element(cx),
                shadcn::CardDescription::new(match chrome {
                    Some(chrome) => format!(
                        "min={}px, chrome={}x{}px",
                        min.0, chrome.width.0, chrome.height.0
                    ),
                    None => format!("min={}px", min.0),
                })
                .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![stack]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_px(Px(280.0)).min_w_0())
        .into_element(cx)
    };

    let checkbox_chrome = {
        let size = theme
            .metric_by_key("md.comp.checkbox.state-layer.size")
            .unwrap_or(Px(40.0));
        Size::new(size, size)
    };
    let radio_chrome = {
        let size = theme
            .metric_by_key("md.comp.radio-button.state-layer.size")
            .unwrap_or(Px(40.0));
        Size::new(size, size)
    };
    let switch_chrome = {
        let width = theme
            .metric_by_key("md.comp.switch.track.width")
            .unwrap_or(Px(52.0));
        let height = theme
            .metric_by_key("md.comp.switch.state-layer.size")
            .unwrap_or(Px(40.0));
        Size::new(width, height)
    };
    let icon_button_chrome = {
        let size = theme
            .metric_by_key("md.comp.icon-button.small.container.height")
            .unwrap_or(Px(40.0));
        Size::new(size, size)
    };

    let grid = {
        let mut props = fret_ui::element::FlexProps::default();
        props.layout = fret_ui::element::LayoutStyle::default();
        props.layout.size.width = fret_ui::element::Length::Fill;
        props.direction = fret_core::Axis::Horizontal;
        props.wrap = true;
        props.gap = Px(16.0);
        props.align = fret_ui::element::CrossAlign::Start;
        props.justify = fret_ui::element::MainAlign::Start;

        cx.flex(props, move |cx| {
            let checkbox = material3::Checkbox::new(material3_checkbox.clone())
                .a11y_label("Material3 checkbox")
                .test_id("ui-gallery-material3-touch-target-checkbox")
                .into_element(cx);
            let radio = material3::Radio::new_value("alpha", material3_radio_value.clone())
                .a11y_label("Material3 radio")
                .test_id("ui-gallery-material3-touch-target-radio")
                .into_element(cx);
            let switch = material3::Switch::new(material3_switch.clone())
                .a11y_label("Material3 switch")
                .test_id("ui-gallery-material3-touch-target-switch")
                .into_element(cx);
            let icon_button = material3::IconButton::new(ids::ui::SETTINGS)
                .a11y_label("Material3 icon button")
                .test_id("ui-gallery-material3-touch-target-icon-button")
                .into_element(cx);
            let tabs = material3::Tabs::new(material3_tabs_value.clone())
                .a11y_label("Material3 tabs (touch targets)")
                .test_id("ui-gallery-material3-touch-target-tabs")
                .scrollable(true)
                .items(vec![
                    material3::TabItem::new("overview", "A")
                        .a11y_label("Material3 tab")
                        .test_id("ui-gallery-material3-touch-target-tab"),
                ])
                .into_element(cx);

            vec![
                target_overlay(cx, "Checkbox", Some(checkbox_chrome), checkbox),
                target_overlay(cx, "Radio", Some(radio_chrome), radio),
                target_overlay(cx, "Switch", Some(switch_chrome), switch),
                target_overlay(cx, "Icon Button", Some(icon_button_chrome), icon_button),
                target_overlay(cx, "Tabs (scrollable, 1 item)", None, tabs),
            ]
        })
    };

    vec![
        cx.text("Touch target overlay legend: green=bounds, yellow=min 48x48, cyan=token chrome (if shown)."),
        grid,
    ]
}

fn preview_material3_icon_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::IconButtonVariant,
               label: &'static str| {
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                let override_style = material3::IconButtonStyle::default()
                    .icon_color(WidgetStateProperty::new(None).when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(fret_core::Color {
                            r: 0.9,
                            g: 0.2,
                            b: 0.9,
                            a: 1.0,
                        })),
                    ))
                    .state_layer_color(WidgetStateProperty::new(None).when(
                        WidgetStates::HOVERED,
                        Some(ColorRef::Color(fret_core::Color {
                            r: 0.9,
                            g: 0.2,
                            b: 0.9,
                            a: 1.0,
                        })),
                    ));
                vec![
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label(label)
                        .into_element(cx),
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label("Override")
                        .style(override_style)
                        .into_element(cx),
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label("Disabled")
                        .disabled(true)
                        .into_element(cx),
                ]
            },
        )
    };

    let toggles = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Filled)
                    .toggle(true)
                    .selected(false)
                    .a11y_label("Toggle off")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Filled)
                    .toggle(true)
                    .selected(true)
                    .a11y_label("Toggle on")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Outlined)
                    .toggle(true)
                    .selected(false)
                    .a11y_label("Outlined off")
                    .into_element(cx),
                material3::IconButton::new(ids::ui::CHECK)
                    .variant(material3::IconButtonVariant::Outlined)
                    .toggle(true)
                    .selected(true)
                    .a11y_label("Outlined on")
                    .into_element(cx),
            ]
        },
    );

    vec![
        cx.text("Material 3 Icon Buttons: token-driven colors + state layer + bounded ripple."),
        row(cx, material3::IconButtonVariant::Standard, "Standard"),
        row(cx, material3::IconButtonVariant::Filled, "Filled"),
        row(cx, material3::IconButtonVariant::Tonal, "Tonal"),
        row(cx, material3::IconButtonVariant::Outlined, "Outlined"),
        toggles,
    ]
}

fn preview_material3_checkbox(
    cx: &mut ElementContext<'_, App>,
    checked: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let value = cx
        .get_model_copied(&checked, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let override_style = material3::CheckboxStyle::default()
                .icon_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.4,
                        a: 1.0,
                    })),
                ))
                .outline_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.4,
                        a: 1.0,
                    })),
                ));
            vec![
                material3::Checkbox::new(checked.clone())
                    .a11y_label("Material 3 Checkbox")
                    .test_id("ui-gallery-material3-checkbox")
                    .into_element(cx),
                material3::Checkbox::new(checked.clone())
                    .a11y_label("Material 3 Checkbox (override)")
                    .style(override_style)
                    .test_id("ui-gallery-material3-checkbox-overridden")
                    .into_element(cx),
                cx.text(format!("checked={}", value as u8)),
                material3::Checkbox::new(checked.clone())
                    .a11y_label("Disabled Material 3 Checkbox")
                    .disabled(true)
                    .test_id("ui-gallery-material3-checkbox-disabled")
                    .into_element(cx),
            ]
        },
    );

    vec![
        cx.text("Material 3 Checkbox: token-driven sizing/colors + state layer + bounded ripple."),
        row,
    ]
}

fn preview_material3_switch(
    cx: &mut ElementContext<'_, App>,
    selected: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let value = cx
        .get_model_copied(&selected, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            let override_style = material3::SwitchStyle::default()
                .track_color(WidgetStateProperty::new(None).when(
                    WidgetStates::SELECTED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.2,
                        g: 0.8,
                        b: 0.4,
                        a: 1.0,
                    })),
                ))
                .state_layer_color(WidgetStateProperty::new(None).when(
                    WidgetStates::HOVERED,
                    Some(ColorRef::Color(fret_core::Color {
                        r: 0.9,
                        g: 0.2,
                        b: 0.9,
                        a: 1.0,
                    })),
                ));
            vec![
                material3::Switch::new(selected.clone())
                    .a11y_label("Material 3 Switch")
                    .test_id("ui-gallery-material3-switch")
                    .into_element(cx),
                material3::Switch::new(selected.clone())
                    .a11y_label("Material 3 Switch (override)")
                    .style(override_style)
                    .test_id("ui-gallery-material3-switch-overridden")
                    .into_element(cx),
                cx.text(format!("selected={}", value as u8)),
                material3::Switch::new(selected.clone())
                    .a11y_label("Disabled Material 3 Switch")
                    .disabled(true)
                    .test_id("ui-gallery-material3-switch-disabled")
                    .into_element(cx),
            ]
        },
    );

    vec![
        cx.text("Material 3 Switch: token-driven sizing/colors + state layer + bounded ripple."),
        row,
    ]
}

fn preview_material3_radio(
    cx: &mut ElementContext<'_, App>,
    group_value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    #[derive(Default)]
    struct RadioPageModels {
        standalone_selected: Option<Model<bool>>,
    }

    let standalone_selected = cx.with_state(RadioPageModels::default, |st| {
        st.standalone_selected.clone()
    });
    let standalone_selected = match standalone_selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(RadioPageModels::default, |st| {
                st.standalone_selected = Some(model.clone())
            });
            model
        }
    };

    let current = cx
        .get_model_cloned(&group_value, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let group_value_for_row = group_value.clone();
    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::RadioGroup::new(group_value_for_row.clone())
                    .a11y_label("Material 3 RadioGroup")
                    .orientation(material3::RadioGroupOrientation::Horizontal)
                    .gap(Px(8.0))
                    .items(vec![
                        material3::RadioGroupItem::new("Alpha")
                            .a11y_label("Radio Alpha")
                            .test_id("ui-gallery-material3-radio-a"),
                        material3::RadioGroupItem::new("Beta")
                            .a11y_label("Radio Beta")
                            .test_id("ui-gallery-material3-radio-b"),
                        material3::RadioGroupItem::new("Charlie")
                            .a11y_label("Radio Charlie (disabled)")
                            .disabled(true)
                            .test_id("ui-gallery-material3-radio-c-disabled"),
                    ])
                    .into_element(cx),
                cx.text(format!("value={}", current.as_ref())),
            ]
        },
    );

    let override_style = material3::RadioStyle::default()
        .icon_color(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ))
        .state_layer_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ));

    let group_value_for_group_overridden = group_value.clone();
    let override_style_for_group = override_style.clone();
    let override_style_for_standalone = override_style.clone();
    let group_overridden = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::RadioGroup::new(group_value_for_group_overridden.clone())
                    .a11y_label("Material 3 RadioGroup (override)")
                    .style(override_style_for_group.clone())
                    .orientation(material3::RadioGroupOrientation::Horizontal)
                    .gap(Px(8.0))
                    .items(vec![
                        material3::RadioGroupItem::new("Alpha")
                            .a11y_label("Radio Alpha (override)")
                            .test_id("ui-gallery-material3-radio-a-overridden"),
                        material3::RadioGroupItem::new("Beta")
                            .a11y_label("Radio Beta (override)")
                            .test_id("ui-gallery-material3-radio-b-overridden"),
                        material3::RadioGroupItem::new("Charlie")
                            .a11y_label("Radio Charlie (disabled)")
                            .disabled(true)
                            .test_id("ui-gallery-material3-radio-c-disabled-overridden"),
                    ])
                    .into_element(cx),
            ]
        },
    );
    let standalone = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::Radio::new(standalone_selected.clone())
                    .a11y_label("Material 3 Radio (standalone)")
                    .test_id("ui-gallery-material3-radio-standalone")
                    .into_element(cx),
                material3::Radio::new(standalone_selected.clone())
                    .a11y_label("Material 3 Radio (override)")
                    .style(override_style_for_standalone.clone())
                    .test_id("ui-gallery-material3-radio-standalone-overridden")
                    .into_element(cx),
            ]
        },
    );

    vec![
        cx.text(
            "Material 3 Radio: group-value binding + roving focus + typeahead + state layer + bounded ripple.",
        ),
        row,
        cx.text("Override preview: RadioGroup::style(...) using RadioStyle."),
        group_overridden,
        cx.text("Override preview: standalone Radio::style(...) using RadioStyle."),
        standalone,
    ]
}

fn preview_material3_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    #[derive(Default)]
    struct SelectPageModels {
        selected: Option<Model<Option<Arc<str>>>>,
    }

    let selected = cx.with_state(SelectPageModels::default, |st| st.selected.clone());
    let selected = match selected {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(None::<Arc<str>>);
            cx.with_state(SelectPageModels::default, |st| {
                st.selected = Some(model.clone())
            });
            model
        }
    };

    let theme = Theme::global(&*cx.app).clone();

    let items: Arc<[material3::SelectItem]> = vec![
        material3::SelectItem::new("alpha", "Alpha").test_id("ui-gallery-material3-select-a"),
        material3::SelectItem::new("beta", "Beta").test_id("ui-gallery-material3-select-b"),
        material3::SelectItem::new("charlie", "Charlie (disabled)")
            .disabled(true)
            .test_id("ui-gallery-material3-select-c-disabled"),
    ]
    .into();

    let default = material3::Select::new(selected.clone())
        .a11y_label("Material 3 Select")
        .placeholder("Pick one")
        .items(items.clone())
        .test_id("ui-gallery-material3-select")
        .into_element(cx);

    let primary = theme.color_required("md.sys.color.primary");
    let primary_container = theme.color_required("md.sys.color.primary-container");
    let secondary_container = theme.color_required("md.sys.color.secondary-container");

    let override_style = material3::SelectStyle::default()
        .container_background(
            WidgetStateProperty::new(None)
                .when(WidgetStates::OPEN, Some(ColorRef::Color(primary_container))),
        )
        .outline_color(
            WidgetStateProperty::new(None)
                .when(WidgetStates::FOCUS_VISIBLE, Some(ColorRef::Color(primary))),
        )
        .trailing_icon_color(
            WidgetStateProperty::new(None).when(WidgetStates::OPEN, Some(ColorRef::Color(primary))),
        )
        .menu_selected_container_color(WidgetStateProperty::new(Some(ColorRef::Color(
            secondary_container,
        ))));

    let overridden = material3::Select::new(selected.clone())
        .a11y_label("Material 3 Select (override)")
        .placeholder("Pick one")
        .items(items)
        .style(override_style)
        .test_id("ui-gallery-material3-select-overridden")
        .into_element(cx);

    vec![
        cx.text(
            "Material 3 Select: token-driven trigger + listbox overlay + ADR 1159 style overrides.",
        ),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |_cx| vec![default, overridden],
        ),
    ]
}

fn preview_material3_text_field(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);
    let error_now = cx
        .get_model_copied(&error, Invalidation::Layout)
        .unwrap_or(false);

    let toggles = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                cx.text("disabled"),
                material3::Switch::new(disabled.clone())
                    .a11y_label("Disable text field")
                    .test_id("ui-gallery-material3-text-field-disabled")
                    .into_element(cx),
                cx.text("error"),
                material3::Switch::new(error.clone())
                    .a11y_label("Toggle error state")
                    .test_id("ui-gallery-material3-text-field-error")
                    .into_element(cx),
            ]
        },
    );

    let supporting = if error_now {
        "Error: required"
    } else {
        "Supporting text"
    };

    let outlined_field = material3::TextField::new(value.clone())
        .variant(material3::TextFieldVariant::Outlined)
        .label("Name")
        .placeholder("Type here")
        .supporting_text(supporting)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field")
        .into_element(cx);

    let outlined_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Outlined").into_element(cx),
            shadcn::CardDescription::new("Animated label + outline \"notch\" patch (best-effort).")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![outlined_field]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let filled_field = material3::TextField::new(value.clone())
        .variant(material3::TextFieldVariant::Filled)
        .label("Email")
        .placeholder("name@example.com")
        .supporting_text(supporting)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field-filled")
        .into_element(cx);

    let filled_card = shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Filled").into_element(cx),
                shadcn::CardDescription::new(
                    "Active indicator bottom border + filled container + hover state layer via foundation indication (best-effort).",
                )
                .into_element(cx),
            ])
            .into_element(cx),
        shadcn::CardContent::new(vec![filled_field]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    let override_style = material3::TextFieldStyle::default()
        .outline_color(WidgetStateProperty::new(None).when(
            WidgetStates::FOCUS_VISIBLE,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ))
        .caret_color(WidgetStateProperty::new(Some(ColorRef::Color(
            fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            },
        ))))
        .placeholder_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ));
    let override_field = material3::TextField::new(value)
        .variant(material3::TextFieldVariant::Outlined)
        .label("Override")
        .placeholder("Hover/focus to see overrides")
        .supporting_text("Caret + focus outline + hover placeholder via TextFieldStyle")
        .style(override_style)
        .disabled(disabled_now)
        .error(error_now)
        .test_id("ui-gallery-material3-text-field-overridden")
        .into_element(cx);
    let override_card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Override").into_element(cx),
            shadcn::CardDescription::new(
                "ADR 1159: partial per-state overrides via TextFieldStyle.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![override_field]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![
        cx.text(
            "Material 3 Text Field: outlined + filled variants (token-driven chrome + label/placeholder outcomes).",
        ),
        toggles,
        outlined_card,
        filled_card,
        override_card,
    ]
}

fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui_kit::{ColorRef, WidgetStateProperty, WidgetStates};

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let fixed_tabs = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs")
        .test_id("ui-gallery-material3-tabs")
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled"),
        ])
        .into_element(cx);

    let override_style = material3::TabsStyle::default()
        .label_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ))
        .state_layer_color(WidgetStateProperty::new(None).when(
            WidgetStates::HOVERED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.9,
                g: 0.2,
                b: 0.9,
                a: 1.0,
            })),
        ))
        .active_indicator_color(WidgetStateProperty::new(None).when(
            WidgetStates::SELECTED,
            Some(ColorRef::Color(fret_core::Color {
                r: 0.2,
                g: 0.8,
                b: 0.4,
                a: 1.0,
            })),
        ));
    let fixed_tabs_overridden = material3::Tabs::new(value.clone())
        .a11y_label("Material 3 Tabs (overridden)")
        .test_id("ui-gallery-material3-tabs-overridden")
        .style(override_style)
        .items(vec![
            material3::TabItem::new("overview", "Overview")
                .a11y_label("Tab Overview")
                .test_id("ui-gallery-material3-tab-overview-overridden"),
            material3::TabItem::new("settings", "Settings")
                .a11y_label("Tab Settings")
                .test_id("ui-gallery-material3-tab-settings-overridden"),
            material3::TabItem::new("disabled", "Disabled")
                .disabled(true)
                .a11y_label("Tab Disabled")
                .test_id("ui-gallery-material3-tab-disabled-overridden"),
        ])
        .into_element(cx);

    let scrollable_tabs = material3::Tabs::new(value)
        .a11y_label("Material 3 Tabs (scrollable)")
        .test_id("ui-gallery-material3-tabs-scrollable")
        .scrollable(true)
        .items(vec![
            material3::TabItem::new("overview", "Overview"),
            material3::TabItem::new("settings", "Settings"),
            material3::TabItem::new("typography", "Typography"),
            material3::TabItem::new("very_long_label", "Very Long Label For Layout Probe"),
            material3::TabItem::new("tokens", "Tokens"),
            material3::TabItem::new("motion", "Motion"),
            material3::TabItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    vec![
        cx.text("Material 3 Tabs: roving focus + state layer + bounded ripple."),
        fixed_tabs,
        cx.text(
            "Override preview: hover label/state-layer + active-indicator color via TabsStyle.",
        ),
        fixed_tabs_overridden,
        cx.text("Scrollable/variable width preview (measurement-driven indicator)."),
        scrollable_tabs,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

fn preview_material3_navigation_bar(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let bar = material3::NavigationBar::new(value)
        .a11y_label("Material 3 Navigation Bar")
        .test_id("ui-gallery-material3-navigation-bar")
        .items(vec![
            material3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH)
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-nav-search"),
            material3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-nav-settings"),
            material3::NavigationBarItem::new("more", "More", ids::ui::MORE_HORIZONTAL)
                .a11y_label("Destination More")
                .test_id("ui-gallery-material3-nav-more"),
        ])
        .into_element(cx);

    vec![
        cx.text("Material 3 Navigation Bar: roving focus + state layer + bounded ripple."),
        bar,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

fn preview_material3_navigation_rail(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let rail = material3::NavigationRail::new(value)
        .a11y_label("Material 3 Navigation Rail")
        .test_id("ui-gallery-material3-navigation-rail")
        .items(vec![
            material3::NavigationRailItem::new("search", "Search", ids::ui::SEARCH)
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-rail-search"),
            material3::NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-rail-settings"),
            material3::NavigationRailItem::new("play", "Play", ids::ui::PLAY)
                .a11y_label("Destination Play")
                .test_id("ui-gallery-material3-rail-play"),
            material3::NavigationRailItem::new("disabled", "Disabled", ids::ui::SLASH)
                .disabled(true)
                .a11y_label("Destination Disabled")
                .test_id("ui-gallery-material3-rail-disabled"),
        ])
        .into_element(cx);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [rail],
    );

    vec![
        cx.text("Material 3 Navigation Rail: roving focus + state layer + bounded ripple."),
        container,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

fn preview_material3_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let drawer = material3::NavigationDrawer::new(value)
        .a11y_label("Material 3 Navigation Drawer")
        .test_id("ui-gallery-material3-navigation-drawer")
        .items(vec![
            material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                .a11y_label("Destination Search")
                .test_id("ui-gallery-material3-drawer-search"),
            material3::NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS)
                .a11y_label("Destination Settings")
                .test_id("ui-gallery-material3-drawer-settings"),
            material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                .a11y_label("Destination Play")
                .test_id("ui-gallery-material3-drawer-play"),
            material3::NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                .disabled(true)
                .a11y_label("Destination Disabled")
                .test_id("ui-gallery-material3-drawer-disabled"),
        ])
        .into_element(cx);

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(280.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [drawer],
    );

    vec![
        cx.text("Material 3 Navigation Drawer: roving focus + state layer + bounded ripple."),
        container,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

fn preview_material3_modal_navigation_drawer(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;
    use fret_ui::action::OnActivate;

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let open_drawer: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };

    let modal = material3::ModalNavigationDrawer::new(open.clone())
        .test_id("ui-gallery-material3-modal-navigation-drawer")
        .into_element(
            cx,
            move |cx| {
                material3::NavigationDrawer::new(value)
                    .variant(material3::NavigationDrawerVariant::Modal)
                    .a11y_label("Material 3 Modal Navigation Drawer")
                    .test_id("ui-gallery-material3-modal-navigation-drawer-panel")
                    .items(vec![
                        material3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH)
                            .a11y_label("Destination Search")
                            .test_id("ui-gallery-material3-modal-drawer-search"),
                        material3::NavigationDrawerItem::new(
                            "settings",
                            "Settings",
                            ids::ui::SETTINGS,
                        )
                        .a11y_label("Destination Settings")
                        .test_id("ui-gallery-material3-modal-drawer-settings"),
                        material3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY)
                            .a11y_label("Destination Play")
                            .test_id("ui-gallery-material3-modal-drawer-play"),
                        material3::NavigationDrawerItem::new("disabled", "Disabled", ids::ui::SLASH)
                            .disabled(true)
                            .a11y_label("Destination Disabled")
                            .test_id("ui-gallery-material3-modal-drawer-disabled"),
                    ])
                    .into_element(cx)
            },
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open drawer")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_drawer.clone())
                                .test_id("ui-gallery-material3-modal-drawer-open")
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id("ui-gallery-material3-modal-drawer-underlay-probe")
                                .into_element(cx),
                            cx.text(
                                "Tip: click the scrim or press Esc to close; Tab/Shift+Tab should stay inside the drawer while open.",
                            ),
                        ]
                    },
                )
            },
        );

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| [modal],
    );

    vec![
        cx.text("Material 3 Modal Navigation Drawer: modal scrim + focus trap/restore + token-driven motion."),
        container,
        cx.text(format!(
            "open={} value={}",
            is_open as u8,
            current.as_ref()
        )),
    ]
}

fn preview_material3_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};

    #[derive(Default)]
    struct DialogPageModels {
        override_open: Option<Model<bool>>,
    }

    let override_open = cx.with_state(DialogPageModels::default, |st| st.override_open.clone());
    let override_open = match override_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(DialogPageModels::default, |st| {
                st.override_open = Some(model.clone())
            });
            model
        }
    };

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let override_is_open = cx
        .get_model_copied(&override_open, Invalidation::Layout)
        .unwrap_or(false);

    let open_dialog: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action: OnActivate = {
        let open = open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm")
            });
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let theme = cx.theme().clone();
    let override_style = material3::DialogStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.secondary-container"),
        ))))
        .headline_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))))
        .supporting_text_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))));

    let open_dialog_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = true);
            host.request_redraw(action_cx.window);
        })
    };
    let close_dialog_override: OnActivate = {
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let confirm_action_override: OnActivate = {
        let override_open = override_open.clone();
        let last_action = last_action.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from("material3.dialog.confirm.override")
            });
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };

    let build_dialog = |cx: &mut ElementContext<'_, App>,
                        open_model: Model<bool>,
                        style: Option<material3::DialogStyle>,
                        id_prefix: &'static str,
                        open_action: OnActivate,
                        close_action: OnActivate,
                        confirm_action: OnActivate|
     -> AnyElement {
        let mut dialog = material3::Dialog::new(open_model.clone())
            .headline("Discard draft?")
            .supporting_text("This action cannot be undone.")
            .actions(vec![
                material3::DialogAction::new("Cancel")
                    .test_id(format!("{id_prefix}-action-cancel"))
                    .on_activate(close_action.clone()),
                material3::DialogAction::new("Discard")
                    .test_id(format!("{id_prefix}-action-discard"))
                    .on_activate(confirm_action.clone()),
            ])
            .test_id(format!("{id_prefix}"));

        if let Some(style) = style {
            dialog = dialog.style(style);
        }

        dialog.into_element(
            cx,
            move |cx| {
                stack::vstack(
                    cx,
                    stack::VStackProps::default()
                        .layout(LayoutRefinement::default().w_full().h_full())
                        .gap(Space::N4),
                    move |cx| {
                        vec![
                            material3::Button::new("Open dialog")
                                .variant(material3::ButtonVariant::Filled)
                                .on_activate(open_action.clone())
                                .test_id(format!("{id_prefix}-open"))
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id(format!("{id_prefix}-underlay-probe"))
                                .into_element(cx),
                            cx.text("Tip: press Esc or click the scrim to close; Tab should stay inside the dialog while open."),
                        ]
                    },
                )
            },
            |_cx| std::iter::empty::<AnyElement>(),
        )
    };

    let default_dialog = build_dialog(
        cx,
        open.clone(),
        None,
        "ui-gallery-material3-dialog",
        open_dialog.clone(),
        close_dialog.clone(),
        confirm_action.clone(),
    );
    let override_dialog = build_dialog(
        cx,
        override_open.clone(),
        Some(override_style),
        "ui-gallery-material3-dialog-override",
        open_dialog_override.clone(),
        close_dialog_override.clone(),
        confirm_action_override.clone(),
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_container = |cx: &mut ElementContext<'_, App>, dialog: AnyElement| -> AnyElement {
        let mut layout = fret_ui::element::LayoutStyle::default();
        layout.size.width = fret_ui::element::Length::Fill;
        layout.size.height = fret_ui::element::Length::Px(Px(360.0));
        cx.container(
            fret_ui::element::ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| [dialog],
        )
    };

    let containers = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                build_container(cx, default_dialog),
                build_container(cx, override_dialog),
            ]
        },
    );

    vec![
        cx.text(
            "Material 3 Dialog: modal barrier + focus trap/restore + token-shaped dialog actions.",
        ),
        containers,
        cx.text(format!(
            "open={} override_open={} last_action={}",
            is_open as u8,
            override_is_open as u8,
            last.as_ref()
        )),
    ]
}

fn preview_material3_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnActivate;
    use fret_ui_kit::{ColorRef, WidgetStateProperty};

    #[derive(Default)]
    struct MenuPageModels {
        override_open: Option<Model<bool>>,
    }

    let override_open = cx.with_state(MenuPageModels::default, |st| st.override_open.clone());
    let override_open = match override_open {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(MenuPageModels::default, |st| {
                st.override_open = Some(model.clone())
            });
            model
        }
    };

    fn on_select(id: &'static str, last_action: Model<Arc<str>>) -> OnActivate {
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&last_action, |v| {
                *v = Arc::<str>::from(id);
            });
            host.request_redraw(action_cx.window);
        })
    }

    let toggle_open: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
            let _ = host.models_mut().update(&override_open, |v| *v = false);
            host.request_redraw(action_cx.window);
        })
    };
    let toggle_open_override: OnActivate = {
        let open = open.clone();
        let override_open = override_open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = false);
            let _ = host.models_mut().update(&override_open, |v| *v = !*v);
            host.request_redraw(action_cx.window);
        })
    };

    let last_action_for_entries = last_action.clone();
    let dropdown = material3::DropdownMenu::new(open.clone())
        .a11y_label("Material 3 Menu")
        .test_id("ui-gallery-material3-menu")
        .into_element(
            cx,
            move |cx| {
                material3::Button::new("Open menu")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(toggle_open.clone())
                    .test_id("ui-gallery-material3-menu-trigger")
                    .into_element(cx)
            },
            move |_cx| {
                vec![
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Cut")
                            .test_id("ui-gallery-material3-menu-item-cut")
                            .on_select(on_select(
                                "material3.menu.cut",
                                last_action_for_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Copy")
                            .test_id("ui-gallery-material3-menu-item-copy")
                            .on_select(on_select(
                                "material3.menu.copy",
                                last_action_for_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Paste")
                            .test_id("ui-gallery-material3-menu-item-paste")
                            .disabled(true),
                    ),
                    material3::MenuEntry::Separator,
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Settings")
                            .test_id("ui-gallery-material3-menu-item-settings")
                            .on_select(on_select(
                                "material3.menu.settings",
                                last_action_for_entries.clone(),
                            )),
                    ),
                ]
            },
        );

    let theme = cx.theme().clone();
    let override_style = material3::MenuStyle::default()
        .container_background(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.secondary-container"),
        ))))
        .item_label_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))))
        .item_state_layer_color(WidgetStateProperty::new(Some(ColorRef::Color(
            theme.color_required("md.sys.color.on-secondary-container"),
        ))));

    let last_action_for_override_entries = last_action.clone();
    let dropdown_override = material3::DropdownMenu::new(override_open.clone())
        .a11y_label("Material 3 Menu (override)")
        .test_id("ui-gallery-material3-menu-override")
        .menu_style(override_style)
        .into_element(
            cx,
            move |cx| {
                material3::Button::new("Open menu (override)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(toggle_open_override.clone())
                    .test_id("ui-gallery-material3-menu-trigger-override")
                    .into_element(cx)
            },
            move |_cx| {
                vec![
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Cut")
                            .test_id("ui-gallery-material3-menu-item-cut-override")
                            .on_select(on_select(
                                "material3.menu.cut.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Copy")
                            .test_id("ui-gallery-material3-menu-item-copy-override")
                            .on_select(on_select(
                                "material3.menu.copy.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Paste")
                            .test_id("ui-gallery-material3-menu-item-paste-override")
                            .disabled(true),
                    ),
                    material3::MenuEntry::Separator,
                    material3::MenuEntry::Item(
                        material3::MenuItem::new("Settings")
                            .test_id("ui-gallery-material3-menu-item-settings-override")
                            .on_select(on_select(
                                "material3.menu.settings.override",
                                last_action_for_override_entries.clone(),
                            )),
                    ),
                ]
            },
        );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let card_default = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Default").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![dropdown]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let card_override = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Override").into_element(cx),
            shadcn::CardDescription::new(
                "ADR 1159: MenuStyle overrides (container + item colors).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![dropdown_override]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    vec![
        cx.text("Tip: Arrow keys / Home / End navigate; type to jump by prefix; Esc/outside press closes."),
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N4).items_center(),
            move |_cx| vec![card_default, card_override],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

fn preview_material3_list(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_icons::ids;

    let current = cx
        .get_model_cloned(&value, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let build_list = |cx: &mut ElementContext<'_, App>, id_prefix: &str| -> AnyElement {
        material3::List::new(value.clone())
            .a11y_label("Material 3 List")
            .test_id(format!("{id_prefix}-list"))
            .items(vec![
                material3::ListItem::new("alpha", "Alpha")
                    .leading_icon(ids::ui::SEARCH)
                    .a11y_label("List item alpha")
                    .test_id(format!("{id_prefix}-list-item-alpha")),
                material3::ListItem::new("beta", "Beta")
                    .leading_icon(ids::ui::SETTINGS)
                    .a11y_label("List item beta")
                    .test_id(format!("{id_prefix}-list-item-beta")),
                material3::ListItem::new("disabled", "Disabled")
                    .leading_icon(ids::ui::SLASH)
                    .disabled(true)
                    .a11y_label("List item disabled")
                    .test_id(format!("{id_prefix}-list-item-disabled")),
            ])
            .into_element(cx)
    };

    let standard = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Standard").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![build_list(cx, "ui-gallery-material3-standard")])
            .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let expressive = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![shadcn::CardTitle::new("Expressive").into_element(cx)])
            .into_element(cx),
        shadcn::CardContent::new(vec![material3::context::with_material_design_variant(
            cx,
            material3::MaterialDesignVariant::Expressive,
            |cx| build_list(cx, "ui-gallery-material3-expressive"),
        )])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let variants = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        move |_cx| [standard, expressive],
    );

    vec![
        cx.text("Material 3 List: roving focus (Up/Down/Home/End) + selection follows focus."),
        cx.text("Compare Standard vs Expressive via subtree override (shape + icon size)."),
        variants,
        cx.text(format!("value={}", current.as_ref())),
    ]
}

fn preview_material3_snackbar(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_runtime::CommandId;
    use fret_ui::action::OnActivate;
    use fret_ui_kit::ToastStore;

    #[derive(Default)]
    struct State {
        store: Option<Model<ToastStore>>,
    }

    let store = cx.with_state(State::default, |st| st.store.clone());
    let store = store.unwrap_or_else(|| {
        let store = cx.app.models_mut().insert(ToastStore::default());
        cx.with_state(State::default, |st| st.store = Some(store.clone()));
        store
    });

    let host_layer = material3::SnackbarHost::new(store.clone())
        .max_snackbars(1)
        .into_element(cx);

    let show_short: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Saved").action("Undo", CommandId::new(CMD_TOAST_ACTION)),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_two_line: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Update available")
                    .supporting_text("Restart the app to apply the latest changes.")
                    .action("Restart", CommandId::new(CMD_TOAST_ACTION))
                    .duration(material3::SnackbarDuration::Long),
            );
            host.request_redraw(acx.window);
        })
    };

    let show_indefinite: OnActivate = {
        let store = store.clone();
        Arc::new(move |host, acx, _reason| {
            let controller = material3::SnackbarController::new(store.clone());
            let _ = controller.show(
                host,
                acx.window,
                material3::Snackbar::new("Connection lost")
                    .supporting_text("Trying to reconnect...")
                    .duration(material3::SnackbarDuration::Indefinite),
            );
            host.request_redraw(acx.window);
        })
    };

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = stack::hstack(
        cx,
        stack::HStackProps::default()
            .gap(Space::N4)
            .layout(LayoutRefinement::default().w_full()),
        |cx| {
            vec![
                material3::Button::new("Show (short)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_short.clone())
                    .test_id("ui-gallery-material3-snackbar-show-short")
                    .into_element(cx),
                material3::Button::new("Show (two-line)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_two_line.clone())
                    .test_id("ui-gallery-material3-snackbar-show-two-line")
                    .into_element(cx),
                material3::Button::new("Show (indefinite)")
                    .variant(material3::ButtonVariant::Outlined)
                    .on_activate(show_indefinite.clone())
                    .test_id("ui-gallery-material3-snackbar-show-indefinite")
                    .into_element(cx),
            ]
        },
    );

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Snackbar").into_element(cx),
            shadcn::CardDescription::new(
                "Snackbar MVP: Material token-driven toast-layer skin (md.comp.snackbar.*).",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            host_layer,
            buttons,
            cx.text(format!("last action: {last}")),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}

fn preview_material3_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let content = material3::TooltipProvider::new().with_elements(cx, |cx| {
        let outlined = material3::ButtonVariant::Outlined;

        let top = material3::PlainTooltip::new(
            material3::Button::new("Hover (Top)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-top-trigger")
                .into_element(cx),
            "Plain tooltip (top)",
        )
        .side(material3::TooltipSide::Top)
        .into_element(cx);

        let right = material3::PlainTooltip::new(
            material3::Button::new("Hover (Right)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-right-trigger")
                .into_element(cx),
            "Plain tooltip (right)",
        )
        .side(material3::TooltipSide::Right)
        .into_element(cx);

        let bottom = material3::PlainTooltip::new(
            material3::Button::new("Hover (Bottom)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-bottom-trigger")
                .into_element(cx),
            "Plain tooltip (bottom)",
        )
        .side(material3::TooltipSide::Bottom)
        .into_element(cx);

        let left = material3::PlainTooltip::new(
            material3::Button::new("Hover (Left)")
                .variant(outlined)
                .test_id("ui-gallery-material3-tooltip-left-trigger")
                .into_element(cx),
            "Plain tooltip (left)",
        )
        .side(material3::TooltipSide::Left)
        .into_element(cx);

        vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .gap(Space::N4)
                        .layout(LayoutRefinement::default().w_full()),
                    |_cx| [top, right, bottom, left],
                ),
                cx.text("Note: Tooltip open delay is controlled via Material3 TooltipProvider (delay-group)."),
            ]
    });

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Tooltip").into_element(cx),
            shadcn::CardDescription::new(
                "Plain tooltip MVP: delay group + hover intent + safe-hover corridor + token-driven styling.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(content).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![card]
}

fn preview_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let left = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Card").into_element(cx),
            shadcn::CardDescription::new("A composed surface primitive.").into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            cx.text("Cards are used as building blocks for many gallery sections."),
            stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        shadcn::Badge::new("layout").into_element(cx),
                        shadcn::Badge::new("chrome")
                            .variant(shadcn::BadgeVariant::Secondary)
                            .into_element(cx),
                        shadcn::Badge::new("token")
                            .variant(shadcn::BadgeVariant::Outline)
                            .into_element(cx),
                    ]
                },
            ),
        ])
        .into_element(cx),
        shadcn::CardFooter::new(vec![
            shadcn::Button::new("Cancel")
                .variant(shadcn::ButtonVariant::Secondary)
                .into_element(cx),
            shadcn::Button::new("Continue").into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    let right = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Embedded Content").into_element(cx),
            shadcn::CardDescription::new("Cards stretch their sections by default.")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![
            shadcn::Button::new("Primary").into_element(cx),
            shadcn::Button::new("Outline")
                .variant(shadcn::ButtonVariant::Outline)
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().flex_1().min_w_0())
    .into_element(cx);

    vec![stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |_cx| [left, right],
    )]
}

fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            [
                shadcn::Badge::new("Default").into_element(cx),
                shadcn::Badge::new("Secondary")
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx),
                shadcn::Badge::new("Destructive")
                    .variant(shadcn::BadgeVariant::Destructive)
                    .into_element(cx),
                shadcn::Badge::new("Outline")
                    .variant(shadcn::BadgeVariant::Outline)
                    .into_element(cx),
            ]
        },
    );

    vec![
        row,
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |cx| {
                vec![
                    shadcn::Button::new("Filter")
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                    shadcn::Badge::new("new")
                        .variant(shadcn::BadgeVariant::Secondary)
                        .into_element(cx),
                    cx.text("Badges work well inline with buttons and text."),
                ]
            },
        ),
    ]
}

fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    let a = {
        let image = shadcn::AvatarImage::model(avatar_image.clone()).into_element(cx);
        let fallback = shadcn::AvatarFallback::new("FR")
            .when_image_missing_model(avatar_image.clone())
            .delay_ms(120)
            .into_element(cx);
        shadcn::Avatar::new([image, fallback]).into_element(cx)
    };

    let b =
        shadcn::Avatar::new([shadcn::AvatarFallback::new("WK").into_element(cx)]).into_element(cx);

    let c = shadcn::Avatar::new([shadcn::AvatarFallback::new("?").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
        .into_element(cx);

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| [a, b, c],
        ),
        cx.text("Tip: use AvatarImage when you have an ImageId; AvatarFallback covers missing/slow loads."),
    ]
}

fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let content = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                shadcn::Skeleton::new()
                    .refine_layout(LayoutRefinement::default().w_px(Px(180.0)))
                    .into_element(cx),
                shadcn::Skeleton::new().into_element(cx),
                shadcn::Skeleton::new()
                    .secondary()
                    .refine_layout(LayoutRefinement::default().w_px(Px(320.0)))
                    .into_element(cx),
                shadcn::Skeleton::new()
                    .secondary()
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
                    .into_element(cx),
            ]
        },
    );

    vec![
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Loading…").into_element(cx),
                shadcn::CardDescription::new("Skeleton requests animation frames while rendered.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![content]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx),
    ]
}

fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let items = (1..=64)
        .map(|i| cx.text(format!("Item {i:02}")))
        .collect::<Vec<_>>();

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |_cx| items,
    );

    let scroll = shadcn::ScrollArea::new([body])
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(240.0)))
        .into_element(cx);

    vec![
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![
                shadcn::CardTitle::new("Scroll Area").into_element(cx),
                shadcn::CardDescription::new("Fixed-height viewport with scrollbars.")
                    .into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new(vec![scroll]).into_element(cx),
        ])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx),
    ]
}

fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    shadcn::TooltipProvider::new()
        .with_elements(cx, |cx| {
            let mk = |cx: &mut ElementContext<'_, App>, label: &str, side: shadcn::TooltipSide| {
                shadcn::Tooltip::new(
                    shadcn::Button::new(label)
                        .variant(shadcn::ButtonVariant::Outline)
                        .into_element(cx),
                    shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                        cx,
                        format!("Tooltip on {label}"),
                    )])
                    .into_element(cx),
                )
                .arrow(true)
                .side(side)
                .open_delay_frames(10)
                .close_delay_frames(10)
                .into_element(cx)
            };

            vec![
            stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
                    vec![
                        mk(cx, "Top", shadcn::TooltipSide::Top),
                        mk(cx, "Right", shadcn::TooltipSide::Right),
                        mk(cx, "Bottom", shadcn::TooltipSide::Bottom),
                        mk(cx, "Left", shadcn::TooltipSide::Left),
                    ]
                },
            ),
            cx.text(
                "Hover the buttons to validate hover intent, delay group, and overlay placement.",
            ),
        ]
        })
        .into_vec()
}

fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        let single = cx.keyed("ui_gallery.slider.single", |cx| {
            shadcn::Slider::new_controllable(cx, None, || vec![35.0])
                .range(0.0, 100.0)
                .test_id("ui-gallery-slider-single")
                .a11y_label("Single value slider")
                .into_element(cx)
        });

        let range = cx.keyed("ui_gallery.slider.range", |cx| {
            shadcn::Slider::new_controllable(cx, None, || vec![20.0, 80.0])
                .range(0.0, 100.0)
                .min_steps_between_thumbs(5)
                .test_id("ui-gallery-slider-range")
                .a11y_label("Range slider")
                .into_element(cx)
        });

        let disabled = cx.keyed("ui_gallery.slider.disabled", |cx| {
            shadcn::Slider::new_controllable(cx, None, || vec![60.0])
                .disabled(true)
                .test_id("ui-gallery-slider-disabled")
                .a11y_label("Disabled slider")
                .into_element(cx)
        });

        let items: Vec<AnyElement> = vec![
            cx.text("Single value"),
            single,
            cx.text("Range (two thumbs)"),
            range,
            cx.text("Disabled"),
            disabled,
        ];

        vec![
            stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N4),
                move |_cx| items,
            ),
            cx.text("Note: this page uses uncontrolled sliders; state is stored in element state under a stable key."),
        ]
    })
}

fn preview_icons(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    use fret_icons::ids;

    let icon_cell =
        |cx: &mut ElementContext<'_, App>, label: &str, icon_id: IconId| -> AnyElement {
            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full())
                    .gap(Space::N2)
                    .items_center(),
                |cx| {
                    vec![
                        icon::icon_with(cx, icon_id, Some(Px(16.0)), None),
                        cx.text(label),
                    ]
                },
            );

            let theme = Theme::global(&*cx.app);
            cx.container(
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .rounded(Radius::Md)
                        .border_1()
                        .p(Space::N3),
                    LayoutRefinement::default().w_full(),
                ),
                |_cx| [row],
            )
        };

    let grid = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                icon_cell(cx, "ui.search", ids::ui::SEARCH),
                icon_cell(cx, "ui.settings", ids::ui::SETTINGS),
                icon_cell(cx, "ui.chevron.right", ids::ui::CHEVRON_RIGHT),
                icon_cell(cx, "ui.close", ids::ui::CLOSE),
                icon_cell(
                    cx,
                    "lucide.loader-circle",
                    IconId::new_static("lucide.loader-circle"),
                ),
            ]
        },
    );

    let spinner_row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Spinner::new().into_element(cx),
                shadcn::Spinner::new().speed(0.0).into_element(cx),
                cx.text("Spinner (animated / static)"),
            ]
        },
    );

    vec![grid, spinner_row]
}

fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct FieldPageModels {
        name: Option<Model<String>>,
        email: Option<Model<String>>,
    }

    let (name, email) = cx.with_state(FieldPageModels::default, |st| {
        (st.name.clone(), st.email.clone())
    });
    let (name, email) = match (name, email) {
        (Some(name), Some(email)) => (name, email),
        _ => {
            let name = cx.app.models_mut().insert(String::new());
            let email = cx.app.models_mut().insert(String::new());
            cx.with_state(FieldPageModels::default, |st| {
                st.name = Some(name.clone());
                st.email = Some(email.clone());
            });
            (name, email)
        }
    };

    let field_name = shadcn::Field::new(vec![
        shadcn::FieldLabel::new("Name").into_element(cx),
        shadcn::FieldDescription::new("Shown in the sidebar and status bar.").into_element(cx),
        shadcn::FieldContent::new(vec![
            shadcn::Input::new(name)
                .a11y_label("Name")
                .placeholder("Rustacean")
                .into_element(cx),
        ])
        .into_element(cx),
    ])
    .into_element(cx);

    let field_email = shadcn::Field::new(vec![
        shadcn::FieldLabel::new("Email").into_element(cx),
        shadcn::FieldDescription::new("Used for notifications (demo only).").into_element(cx),
        shadcn::FieldContent::new(vec![
            shadcn::Input::new(email)
                .a11y_label("Email")
                .placeholder("name@example.com")
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::FieldError::new("Invalid email address").into_element(cx),
    ])
    .into_element(cx);

    vec![shadcn::FieldSet::new(vec![field_name, field_email]).into_element(cx)]
}

fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let input = shadcn::Input::new(text_input)
        .a11y_label("Email")
        .placeholder("name@example.com")
        .into_element(cx);

    let textarea = shadcn::Textarea::new(text_area)
        .a11y_label("Message")
        .into_element(cx);

    let toggles = stack::vstack(
        cx,
        stack::VStackProps::default().gap(Space::N2).items_start(),
        |cx| {
            vec![
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Checkbox::new(checkbox)
                                .a11y_label("Accept terms")
                                .into_element(cx),
                            ui::label(cx, "Accept terms").into_element(cx),
                        ]
                    },
                ),
                stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    |cx| {
                        vec![
                            shadcn::Switch::new(switch)
                                .a11y_label("Enable feature")
                                .into_element(cx),
                            ui::label(cx, "Enable feature").into_element(cx),
                        ]
                    },
                ),
            ]
        },
    );

    vec![stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N3),
        |cx| {
            let tip = ui::text_block(
                cx,
                "Tip: these are model-bound controls; values persist while you stay in the window.",
            )
            .into_element(cx);
            [input, textarea, toggles, tip]
        },
    )]
}

fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
        .trigger_test_id("ui-gallery-select-trigger")
        .placeholder("Pick a fruit")
        .items(
            [
                shadcn::SelectItem::new("apple", "Apple").test_id("ui-gallery-select-item-apple"),
                shadcn::SelectItem::new("banana", "Banana")
                    .test_id("ui-gallery-select-item-banana"),
                shadcn::SelectItem::new("orange", "Orange")
                    .test_id("ui-gallery-select-item-orange"),
            ]
            .into_iter()
            .chain((1..=40).map(|i| {
                let value: Arc<str> = Arc::from(format!("item-{i:02}"));
                let label: Arc<str> = Arc::from(format!("Item {i:02}"));
                let test_id: Arc<str> = Arc::from(format!("ui-gallery-select-item-{value}"));
                shadcn::SelectItem::new(value, label).test_id(test_id)
            })),
        )
        .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
        .into_element(cx);

    let selected = cx
        .watch_model(&value)
        .layout()
        .cloned()
        .unwrap_or_default()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![select, cx.text(format!("Selected: {selected}"))]
}

fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let combo = shadcn::Combobox::new(value.clone(), open)
        .a11y_label("Combobox")
        .width(Px(240.0))
        .placeholder("Pick a fruit")
        .query_model(query.clone())
        .items([
            shadcn::ComboboxItem::new("apple", "Apple"),
            shadcn::ComboboxItem::new("banana", "Banana"),
            shadcn::ComboboxItem::new("orange", "Orange"),
            shadcn::ComboboxItem::new("disabled", "Disabled").disabled(true),
        ])
        .into_element(cx);

    let selected = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let query_text = cx
        .get_model_cloned(&query, Invalidation::Layout)
        .unwrap_or_default();

    vec![
        combo,
        cx.text(format!("Selected: {selected}")),
        cx.text(format!("Query: {query_text}")),
    ]
}

fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    let picker = shadcn::DatePicker::new(open, month, selected.clone())
        .placeholder("Pick a date")
        .into_element(cx);

    let selected_text: Arc<str> = cx
        .app
        .models()
        .read(&selected, |v| v.map(|d| Arc::<str>::from(d.to_string())))
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![picker, cx.text(format!("Selected: {selected_text}"))]
}

fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    let boxy = |cx: &mut ElementContext<'_, App>, title: &str, color_key: &str| -> AnyElement {
        let props = decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required(color_key)))
                .rounded(Radius::Md)
                .p(Space::N3),
            LayoutRefinement::default().w_full().h_full(),
        );
        cx.container(props, move |cx| [cx.text(title)])
    };

    let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
        .axis(fret_core::Axis::Vertical)
        .entries([
            shadcn::ResizablePanel::new([boxy(cx, "Viewport", "muted")])
                .min_px(Px(120.0))
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new([boxy(cx, "Console", "card")])
                .min_px(Px(80.0))
                .into(),
        ])
        .into_element(cx);

    let root = {
        let root = shadcn::ResizablePanelGroup::new(h_fractions)
            .axis(fret_core::Axis::Horizontal)
            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
            .entries(vec![
                shadcn::ResizablePanel::new(vec![boxy(cx, "Explorer", "accent")])
                    .min_px(Px(140.0))
                    .into(),
                shadcn::ResizableHandle::new().into(),
                shadcn::ResizablePanel::new(vec![nested_vertical])
                    .min_px(Px(240.0))
                    .into(),
            ])
            .into_element(cx);

        cx.semantics(
            fret_ui::element::SemanticsProps {
                label: Some(Arc::<str>::from("Debug:ui-gallery:resizable-panels")),
                test_id: Some(Arc::<str>::from("ui-gallery-resizable-panels")),
                ..Default::default()
            },
            move |_cx| [root],
        )
    };

    vec![cx.text("Drag the handles to resize panels."), root]
}

#[derive(Debug, Clone)]
struct DemoProcessRow {
    id: u64,
    name: Arc<str>,
    status: Arc<str>,
    cpu: u64,
    mem_mb: u64,
}

#[derive(Debug, Clone)]
struct DemoProcessTableAssets {
    data: Arc<[DemoProcessRow]>,
    columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]>,
}

fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    let assets = cx.with_state(
        || {
            let data: Arc<[DemoProcessRow]> = Arc::from(vec![
                DemoProcessRow {
                    id: 1,
                    name: Arc::from("Renderer"),
                    status: Arc::from("Running"),
                    cpu: 12,
                    mem_mb: 420,
                },
                DemoProcessRow {
                    id: 2,
                    name: Arc::from("Asset Cache"),
                    status: Arc::from("Idle"),
                    cpu: 0,
                    mem_mb: 128,
                },
                DemoProcessRow {
                    id: 3,
                    name: Arc::from("Indexer"),
                    status: Arc::from("Running"),
                    cpu: 38,
                    mem_mb: 860,
                },
                DemoProcessRow {
                    id: 4,
                    name: Arc::from("Spellcheck"),
                    status: Arc::from("Disabled"),
                    cpu: 0,
                    mem_mb: 0,
                },
                DemoProcessRow {
                    id: 5,
                    name: Arc::from("Language Server"),
                    status: Arc::from("Running"),
                    cpu: 7,
                    mem_mb: 512,
                },
            ]);

            let columns: Arc<[fret_ui_headless::table::ColumnDef<DemoProcessRow>]> =
                Arc::from(vec![
                    fret_ui_headless::table::ColumnDef::new("name")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.name.cmp(&b.name))
                        .size(220.0),
                    fret_ui_headless::table::ColumnDef::new("status")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.status.cmp(&b.status))
                        .size(140.0),
                    fret_ui_headless::table::ColumnDef::new("cpu%")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.cpu.cmp(&b.cpu))
                        .size(90.0),
                    fret_ui_headless::table::ColumnDef::new("mem_mb")
                        .sort_by(|a: &DemoProcessRow, b: &DemoProcessRow| a.mem_mb.cmp(&b.mem_mb))
                        .size(110.0),
                ]);

            DemoProcessTableAssets { data, columns }
        },
        |st| st.clone(),
    );

    let selected_count = cx
        .app
        .models()
        .read(&state, |st| st.row_selection.len())
        .ok()
        .unwrap_or(0);
    let sorting = cx
        .app
        .models()
        .read(&state, |st| {
            st.sorting.first().map(|s| (s.column.clone(), s.desc))
        })
        .ok()
        .flatten();

    let sorting_text: Arc<str> = sorting
        .map(|(col, desc)| {
            Arc::<str>::from(format!(
                "Sorting: {} {}",
                col,
                if desc { "desc" } else { "asc" }
            ))
        })
        .unwrap_or_else(|| Arc::<str>::from("Sorting: <none>"));

    let normalize_col_id =
        |id: &str| -> Arc<str> { Arc::<str>::from(id.replace('%', "pct").replace('_', "-")) };

    let table = shadcn::DataTable::new()
        .row_height(Px(36.0))
        .refine_layout(LayoutRefinement::default().w_full().h_px(Px(280.0)))
        .into_element(
            cx,
            assets.data.clone(),
            1,
            state,
            assets.columns.clone(),
            |row, _index, _parent| fret_ui_headless::table::RowKey(row.id),
            |col| col.id.clone(),
            move |cx, col, row| {
                let col_id = normalize_col_id(col.id.as_ref());
                let cell = match col.id.as_ref() {
                    "name" => cx.text(row.name.as_ref()),
                    "status" => cx.text(row.status.as_ref()),
                    "cpu%" => cx.text(format!("{}%", row.cpu)),
                    "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                    _ => cx.text("?"),
                };

                cx.semantics(
                    fret_ui::element::SemanticsProps {
                        test_id: Some(Arc::<str>::from(format!(
                            "ui-gallery-data-table-cell-{}-{}",
                            row.id, col_id
                        ))),
                        ..Default::default()
                    },
                    move |_cx| vec![cell],
                )
            },
        );

    let table = cx.semantics(
        fret_ui::element::SemanticsProps {
            test_id: Some(Arc::<str>::from("ui-gallery-data-table-root")),
            ..Default::default()
        },
        move |_cx| vec![table],
    );

    vec![
        cx.text("Click header to sort; click row to toggle selection."),
        cx.text(format!("Selected rows: {selected_count}")),
        cx.text(sorting_text.as_ref()),
        table,
    ]
}

fn preview_data_table_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    _state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    use fret_ui_headless::table::{ColumnDef, RowKey};

    #[derive(Debug, Clone)]
    struct Row {
        id: u64,
        name: Arc<str>,
        status: Arc<str>,
        cpu: u64,
        mem_mb: u64,
    }

    let (data, columns) = cx.with_state(
        || {
            let mut rows: Vec<Row> = Vec::with_capacity(50_000);
            for i in 0..50_000u64 {
                let status = match i % 4 {
                    0 => "Running",
                    1 => "Idle",
                    2 => "Sleeping",
                    _ => "Blocked",
                };
                rows.push(Row {
                    id: i,
                    name: Arc::from(format!("Process {i}")),
                    status: Arc::from(status),
                    cpu: (i * 7) % 100,
                    mem_mb: 32 + ((i * 13) % 4096),
                });
            }

            let columns: Arc<[ColumnDef<Row>]> = Arc::from(vec![
                ColumnDef::new("name")
                    .sort_by(|a: &Row, b: &Row| a.name.cmp(&b.name))
                    .size(220.0),
                ColumnDef::new("status")
                    .sort_by(|a: &Row, b: &Row| a.status.cmp(&b.status))
                    .size(140.0),
                ColumnDef::new("cpu%")
                    .sort_by(|a: &Row, b: &Row| a.cpu.cmp(&b.cpu))
                    .size(90.0),
                ColumnDef::new("mem_mb")
                    .sort_by(|a: &Row, b: &Row| a.mem_mb.cmp(&b.mem_mb))
                    .size(110.0),
            ]);

            (Arc::<[Row]>::from(rows), columns)
        },
        |(data, columns)| (data.clone(), columns.clone()),
    );

    #[derive(Default)]
    struct DataTableTortureModels {
        state: Option<Model<fret_ui_headless::table::TableState>>,
    }

    let state = cx.with_state(DataTableTortureModels::default, |st| st.state.clone());
    let state = match state {
        Some(state) => state,
        None => {
            let mut state_value = fret_ui_headless::table::TableState::default();
            state_value.pagination.page_size = data.len();
            state_value.pagination.page_index = 0;
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(DataTableTortureModels::default, |st| {
                st.state = Some(state.clone());
            });
            state
        }
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized business table (TanStack-aligned headless engine + VirtualList)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
            ]
        },
    );

    let table =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            vec![cx.semantics(
                fret_ui::element::SemanticsProps {
                    role: fret_core::SemanticsRole::Group,
                    test_id: Some(Arc::<str>::from("ui-gallery-data-table-torture-root")),
                    ..Default::default()
                },
                |cx| {
                    vec![
                        shadcn::DataTable::new()
                            .overscan(10)
                            .row_height(Px(28.0))
                            .refine_layout(LayoutRefinement::default().w_full().h_px(Px(420.0)))
                            .into_element(
                                cx,
                                data.clone(),
                                1,
                                state,
                                columns.clone(),
                                |row, _index, _parent| RowKey(row.id),
                                |col| Arc::<str>::from(col.id.as_ref()),
                                |cx, col, row| match col.id.as_ref() {
                                    "name" => cx.text(row.name.as_ref()),
                                    "status" => cx.text(row.status.as_ref()),
                                    "cpu%" => cx.text(format!("{}%", row.cpu)),
                                    "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                                    _ => cx.text("?"),
                                },
                            ),
                    ]
                },
            )]
        });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full(),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![table])]
}

fn preview_tree_torture(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    use std::collections::HashSet;

    use fret_ui_kit::TreeItem;
    use fret_ui_kit::TreeState;

    #[derive(Default)]
    struct TreeTortureModels {
        items: Option<Model<Vec<TreeItem>>>,
        state: Option<Model<TreeState>>,
    }

    let (items, state) = cx.with_state(TreeTortureModels::default, |st| {
        (st.items.clone(), st.state.clone())
    });
    let (items, state) = match (items, state) {
        (Some(items), Some(state)) => (items, state),
        _ => {
            let (items_value, state_value) = {
                let root_count = 200u64;
                let folders_per_root = 10u64;
                let leaves_per_folder = 25u64;

                let mut expanded: HashSet<u64> = HashSet::new();
                let mut roots: Vec<TreeItem> = Vec::with_capacity(root_count as usize);

                for r in 0..root_count {
                    let root_id = r;
                    expanded.insert(root_id);

                    let mut folders: Vec<TreeItem> = Vec::with_capacity(folders_per_root as usize);
                    for f in 0..folders_per_root {
                        let folder_id = 1_000_000 + r * 100 + f;
                        expanded.insert(folder_id);

                        let mut leaves: Vec<TreeItem> =
                            Vec::with_capacity(leaves_per_folder as usize);
                        for l in 0..leaves_per_folder {
                            let leaf_id = 2_000_000 + r * 10_000 + f * 100 + l;
                            leaves.push(
                                TreeItem::new(leaf_id, format!("Leaf {r}/{f}/{l} (id={leaf_id})"))
                                    .disabled(leaf_id % 97 == 0),
                            );
                        }

                        folders.push(
                            TreeItem::new(folder_id, format!("Folder {r}/{f}")).children(leaves),
                        );
                    }

                    roots.push(TreeItem::new(root_id, format!("Root {r}")).children(folders));
                }

                (
                    roots,
                    TreeState {
                        selected: None,
                        expanded,
                    },
                )
            };

            let items = cx.app.models_mut().insert(items_value);
            let state = cx.app.models_mut().insert(state_value);
            cx.with_state(TreeTortureModels::default, |st| {
                st.items = Some(items.clone());
                st.state = Some(state.clone());
            });
            (items, state)
        }
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: baseline perf harness for a virtualized tree (expand/collapse + selection + scroll)."),
                cx.text("Use scripted scroll + bundle stats to validate cache-root reuse and prepaint-driven windowing refactors."),
            ]
        },
    );

    let tree = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        vec![cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::Group,
                test_id: Some(Arc::<str>::from("ui-gallery-tree-torture-root")),
                ..Default::default()
            },
            |cx| {
                vec![fret_ui_kit::declarative::tree::tree_view(
                    cx,
                    items,
                    state,
                    fret_ui_kit::Size::Medium,
                )]
            },
        )]
    });

    let mut container_props = decl_style::container_props(
        theme,
        ChromeRefinement::default(),
        LayoutRefinement::default().w_full().h_px(Px(460.0)),
    );
    container_props.layout.overflow = fret_ui::element::Overflow::Clip;

    vec![header, cx.container(container_props, |_cx| vec![tree])]
}

fn preview_data_grid(
    cx: &mut ElementContext<'_, App>,
    selected_row: Model<Option<u64>>,
) -> Vec<AnyElement> {
    let selected = cx
        .get_model_copied(&selected_row, Invalidation::Paint)
        .flatten();

    let selected_text: Arc<str> = selected
        .map(|v| Arc::<str>::from(v.to_string()))
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let grid = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let selected = cx
            .get_model_copied(&selected_row, Invalidation::Layout)
            .flatten();

        let grid =
            shadcn::experimental::DataGridElement::new(["PID", "Name", "State", "CPU%"], 200)
                .refine_layout(LayoutRefinement::default().w_full().h_px(Px(320.0)))
                .into_element(
                    cx,
                    1,
                    1,
                    |row| row as u64,
                    move |row| {
                        let is_selected = selected == Some(row as u64);
                        let cmd = CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}"));
                        shadcn::experimental::DataGridRowState {
                            selected: is_selected,
                            enabled: row % 17 != 0,
                            on_click: Some(cmd),
                        }
                    },
                    |cx, row, col| {
                        let pid = 1000 + row as u64;
                        match col {
                            0 => cx.text(pid.to_string()),
                            1 => cx.text(format!("Process {row}")),
                            2 => cx.text(if row % 3 == 0 { "Running" } else { "Idle" }),
                            _ => cx.text(((row * 7) % 100).to_string()),
                        }
                    },
                );

        vec![grid]
    });

    vec![
        cx.text("Virtualized rows/cols viewport; click a row to select (disabled every 17th row)."),
        cx.text(format!("Selected row: {selected_text}")),
        grid,
    ]
}

fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let selected = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let tabs = shadcn::Tabs::new(value)
        .refine_layout(LayoutRefinement::default().w_px(Px(400.0)))
        .list_full_width(false)
        .items([
            shadcn::TabsItem::new(
                "account",
                "Account",
                vec![cx.text("Make changes to your account here.")],
            ),
            shadcn::TabsItem::new(
                "password",
                "Password",
                vec![cx.text("Change your password here.")],
            ),
        ])
        .into_element(cx);

    vec![
        tabs,
        cx.text("Note: this gallery uses Tabs for the Preview/Usage/Docs shell."),
        cx.text(format!("Selected: {selected}")),
    ]
}

fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    let open = cx
        .app
        .models()
        .get_cloned(&value)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let accordion = shadcn::Accordion::single(value)
        .collapsible(true)
        .refine_layout(LayoutRefinement::default().w_full())
        .items([
            shadcn::AccordionItem::new(
                "item-1",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 1")]),
                shadcn::AccordionContent::new(vec![cx.text("This section is collapsible.")]),
            ),
            shadcn::AccordionItem::new(
                "item-2",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 2")]),
                shadcn::AccordionContent::new(vec![
                    cx.text("Keyboard navigation uses roving focus."),
                ]),
            ),
            shadcn::AccordionItem::new(
                "item-3",
                shadcn::AccordionTrigger::new(vec![cx.text("Item 3")]),
                shadcn::AccordionContent::new(vec![
                    cx.text("Content lives in normal layout flow (no portal)."),
                ]),
            ),
        ])
        .into_element(cx);

    vec![accordion, cx.text(format!("Open item: {open}"))]
}

fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let header = shadcn::TableHeader::new(vec![
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableHead::new("Crate").into_element(cx),
                shadcn::TableHead::new("Layer").into_element(cx),
                shadcn::TableHead::new("Notes").into_element(cx),
            ],
        )
        .border_bottom(true)
        .into_element(cx),
    ])
    .into_element(cx);

    let body = shadcn::TableBody::new(vec![
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.semantics(
                    fret_ui::element::SemanticsProps {
                        test_id: Some(Arc::<str>::from("ui-gallery-table-cell-fret-ui")),
                        ..Default::default()
                    },
                    |cx| vec![cx.text("fret-ui")],
                ))
                .into_element(cx),
                shadcn::TableCell::new(cx.text("mechanisms")).into_element(cx),
                shadcn::TableCell::new(cx.text("Element tree + layout")).into_element(cx),
            ],
        )
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui-kit")).into_element(cx),
                shadcn::TableCell::new(cx.text("policies")).into_element(cx),
                shadcn::TableCell::new(cx.text("Dismiss / focus / menu / overlays"))
                    .into_element(cx),
            ],
        )
        .selected(true)
        .into_element(cx),
        shadcn::TableRow::new(
            3,
            vec![
                shadcn::TableCell::new(cx.text("fret-ui-shadcn")).into_element(cx),
                shadcn::TableCell::new(cx.text("recipes")).into_element(cx),
                shadcn::TableCell::new(cx.text("skinned components + defaults")).into_element(cx),
            ],
        )
        .into_element(cx),
    ])
    .into_element(cx);

    let caption = shadcn::TableCaption::new("Tip: TableRow has hover + selected styling parity.")
        .into_element(cx);

    let table = shadcn::Table::new(vec![header, body, caption])
        .refine_layout(LayoutRefinement::default().w_full())
        .into_element(cx);

    vec![cx.semantics(
        fret_ui::element::SemanticsProps {
            test_id: Some(Arc::<str>::from("ui-gallery-table-root")),
            ..Default::default()
        },
        move |_cx| vec![table],
    )]
}

fn preview_progress(cx: &mut ElementContext<'_, App>, progress: Model<f32>) -> Vec<AnyElement> {
    let bar = shadcn::Progress::new(progress).into_element(cx);

    let controls = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("-10")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_PROGRESS_DEC)
                    .into_element(cx),
                shadcn::Button::new("+10")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_PROGRESS_INC)
                    .into_element(cx),
                shadcn::Button::new("Reset")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .on_click(CMD_PROGRESS_RESET)
                    .into_element(cx),
            ]
        },
    );

    vec![bar, controls]
}

fn preview_menus(
    cx: &mut ElementContext<'_, App>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("DropdownMenu")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-menus-dropdown-trigger")
                .toggle_model(dropdown_open.clone())
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple")
                        .test_id("ui-gallery-menus-dropdown-item-apple")
                        .on_select(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").on_select(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
        cx,
        |cx| {
            shadcn::Button::new("ContextMenu (right click)")
                .variant(shadcn::ButtonVariant::Outline)
                .test_id("ui-gallery-menus-context-trigger")
                .into_element(cx)
        },
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-menus-context-item-action")
                        .on_select(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| [dropdown, context_menu],
        ),
        cx.text(format!("last action: {last}")),
    ]
}

fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let cmdk = shadcn::CommandDialog::new_with_host_commands(cx, open.clone(), query)
        .a11y_label("Command palette")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Command Palette")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open)
                .into_element(cx)
        });

    vec![
        cx.text("Tip: Ctrl/Cmd+P triggers the command palette command."),
        cx.text(format!("last action: {last}")),
        cmdk,
    ]
}

fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let buttons = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
                shadcn::Button::new("Default")
                    .test_id("ui-gallery-toast-default")
                    .on_click(CMD_TOAST_DEFAULT)
                    .into_element(cx),
                shadcn::Button::new("Success")
                    .test_id("ui-gallery-toast-success")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SUCCESS)
                    .into_element(cx),
                shadcn::Button::new("Error")
                    .test_id("ui-gallery-toast-error")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_ERROR)
                    .into_element(cx),
                shadcn::Button::new("Action + Cancel")
                    .test_id("ui-gallery-toast-action-cancel")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SHOW_ACTION_CANCEL)
                    .into_element(cx),
            ]
        },
    );

    vec![buttons, cx.text(format!("last action: {last}"))]
}

fn preview_overlay(
    cx: &mut ElementContext<'_, App>,
    popover_open: Model<bool>,
    dialog_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    sheet_open: Model<bool>,
    portal_geometry_popover_open: Model<bool>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    use fret_ui::action::OnDismissRequest;

    let last_action_status = {
        let last = cx
            .app
            .models()
            .get_cloned(&last_action)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        let text = format!("last action: {last}");
        cx.semantics(
            fret_ui::element::SemanticsProps {
                test_id: Some(Arc::from("ui-gallery-overlay-last-action")),
                ..Default::default()
            },
            |cx| [cx.text(text)],
        )
    };

    let overlays =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let overlay_reset = {
                use fret_ui::action::OnActivate;

                let dropdown_open = dropdown_open.clone();
                let context_menu_open = context_menu_open.clone();
                let context_menu_edge_open = context_menu_edge_open.clone();
                let popover_open = popover_open.clone();
                let dialog_open = dialog_open.clone();
                let alert_dialog_open = alert_dialog_open.clone();
                let sheet_open = sheet_open.clone();
                let portal_geometry_popover_open = portal_geometry_popover_open.clone();
                let last_action = last_action.clone();

                let on_activate: OnActivate = Arc::new(move |host, _cx, _reason| {
                    let _ = host.models_mut().update(&dropdown_open, |v| *v = false);
                    let _ = host.models_mut().update(&context_menu_open, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&context_menu_edge_open, |v| *v = false);
                    let _ = host.models_mut().update(&popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&alert_dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&sheet_open, |v| *v = false);
                    let _ = host
                        .models_mut()
                        .update(&portal_geometry_popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&last_action, |v| {
                        *v = Arc::<str>::from("overlay:reset");
                    });
                });

                shadcn::Button::new("Reset overlays")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .test_id("ui-gallery-overlay-reset")
                    .on_activate(on_activate)
                    .into_element(cx)
            };

            let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone())
                .modal(false)
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("DropdownMenu")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-dropdown-trigger")
                            .toggle_model(dropdown_open.clone())
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Apple")
                                    .test_id("ui-gallery-dropdown-item-apple")
                                    .on_select(CMD_MENU_DROPDOWN_APPLE),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("More")
                                    .test_id("ui-gallery-dropdown-item-more")
                                    .close_on_select(false)
                                    .submenu(vec![
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Nested action")
                                                .test_id("ui-gallery-dropdown-submenu-item-nested")
                                                .on_select(CMD_MENU_CONTEXT_ACTION),
                                        ),
                                        shadcn::DropdownMenuEntry::Separator,
                                        shadcn::DropdownMenuEntry::Item(
                                            shadcn::DropdownMenuItem::new("Nested disabled")
                                                .disabled(true),
                                        ),
                                    ]),
                            ),
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Orange")
                                    .on_select(CMD_MENU_DROPDOWN_ORANGE),
                            ),
                            shadcn::DropdownMenuEntry::Separator,
                            shadcn::DropdownMenuEntry::Item(
                                shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let context_menu = shadcn::ContextMenu::new(context_menu_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("ContextMenu (right click)")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-context-trigger")
                        .into_element(cx)
                },
                |_cx| {
                    vec![
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Action")
                                .test_id("ui-gallery-context-item-action")
                                .on_select(CMD_MENU_CONTEXT_ACTION),
                        ),
                        shadcn::ContextMenuEntry::Separator,
                        shadcn::ContextMenuEntry::Item(
                            shadcn::ContextMenuItem::new("Disabled").disabled(true),
                        ),
                    ]
                },
            );

            let context_menu_edge = shadcn::ContextMenu::new(context_menu_edge_open.clone())
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("ContextMenu (edge, right click)")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-context-trigger-edge")
                            .into_element(cx)
                    },
                    |_cx| {
                        vec![
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Action")
                                    .test_id("ui-gallery-context-edge-item-action")
                                    .on_select(CMD_MENU_CONTEXT_ACTION),
                            ),
                            shadcn::ContextMenuEntry::Separator,
                            shadcn::ContextMenuEntry::Item(
                                shadcn::ContextMenuItem::new("Disabled").disabled(true),
                            ),
                        ]
                    },
                );

            let underlay = shadcn::Button::new("Underlay (outside-press target)")
                .variant(shadcn::ButtonVariant::Secondary)
                .test_id("ui-gallery-overlay-underlay")
                .into_element(cx);

            let tooltip = shadcn::Tooltip::new(
                shadcn::Button::new("Tooltip (hover)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-tooltip-trigger")
                    .into_element(cx),
                cx.semantics(
                    fret_ui::element::SemanticsProps {
                        test_id: Some(Arc::from("ui-gallery-tooltip-content")),
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            shadcn::TooltipContent::new(vec![shadcn::TooltipContent::text(
                                cx,
                                "Tooltip: hover intent + placement",
                            )])
                            .into_element(cx),
                        ]
                    },
                ),
            )
            .arrow(true)
            .arrow_test_id("ui-gallery-tooltip-arrow")
            .panel_test_id("ui-gallery-tooltip-panel")
            .open_delay_frames(10)
            .close_delay_frames(10)
            .side(shadcn::TooltipSide::Top)
            .into_element(cx);

            let hover_card = shadcn::HoverCard::new(
                shadcn::Button::new("HoverCard (hover)")
                    .variant(shadcn::ButtonVariant::Outline)
                    .test_id("ui-gallery-hovercard-trigger")
                    .into_element(cx),
                cx.semantics(
                    fret_ui::element::SemanticsProps {
                        test_id: Some(Arc::from("ui-gallery-hovercard-content")),
                        ..Default::default()
                    },
                    |cx| {
                        vec![
                            shadcn::HoverCardContent::new(vec![
                                cx.text("HoverCard content (overlay-root)"),
                                cx.text("Move pointer from trigger to content."),
                            ])
                            .into_element(cx),
                        ]
                    },
                ),
            )
            .open_delay_frames(10)
            .close_delay_frames(10)
            .into_element(cx);

            let popover_open_for_dismiss = popover_open.clone();
            let last_action_for_dismiss = last_action.clone();
            let popover_on_dismiss: OnDismissRequest = Arc::new(move |host, _cx, _reason| {
                let _ = host
                    .models_mut()
                    .update(&popover_open_for_dismiss, |open| *open = false);
                let _ = host.models_mut().update(&last_action_for_dismiss, |cur| {
                    *cur = Arc::<str>::from("popover:dismissed");
                });
            });

            let popover = shadcn::Popover::new(popover_open.clone())
                .auto_focus(true)
                .on_dismiss_request(Some(popover_on_dismiss))
                .into_element(
                    cx,
                    |cx| {
                        shadcn::Button::new("Popover")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-popover-trigger")
                            .toggle_model(popover_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        let open_dialog = shadcn::Button::new("Open dialog")
                            .variant(shadcn::ButtonVariant::Outline)
                            .test_id("ui-gallery-popover-dialog-trigger")
                            .toggle_model(dialog_open.clone())
                            .into_element(cx);

                        let close = shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-popover-close")
                            .toggle_model(popover_open.clone())
                            .into_element(cx);

                        cx.semantics(
                            fret_ui::element::SemanticsProps {
                                test_id: Some(Arc::from("ui-gallery-popover-content")),
                                ..Default::default()
                            },
                            |cx| {
                                vec![
                                    shadcn::PopoverContent::new(vec![
                                        cx.text("Popover content"),
                                        open_dialog,
                                        close,
                                    ])
                                    .into_element(cx),
                                ]
                            },
                        )
                    },
                );

            let dialog = shadcn::Dialog::new(dialog_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("Dialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-dialog-trigger")
                        .toggle_model(dialog_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            test_id: Some(Arc::from("ui-gallery-dialog-content")),
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                shadcn::DialogContent::new(vec![
                                    shadcn::DialogHeader::new(vec![
                                        shadcn::DialogTitle::new("Dialog").into_element(cx),
                                        shadcn::DialogDescription::new(
                                            "Escape / overlay click closes",
                                        )
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                    {
                                        let body = stack::vstack(
                                            cx,
                                            stack::VStackProps::default().gap(Space::N2).layout(
                                                LayoutRefinement::default()
                                                    .w_full()
                                                    .min_w_0()
                                                    .min_h_0(),
                                            ),
                                            |cx| {
                                                (0..64)
                                                    .map(|i| {
                                                        cx.text(format!(
                                                            "Scrollable content line {}",
                                                            i + 1
                                                        ))
                                                    })
                                                    .collect::<Vec<_>>()
                                            },
                                        );

                                        shadcn::ScrollArea::new([body])
                                            .refine_layout(
                                                LayoutRefinement::default()
                                                    .w_full()
                                                    .h_px(Px(240.0))
                                                    .min_w_0()
                                                    .min_h_0(),
                                            )
                                            .viewport_test_id("ui-gallery-dialog-scroll-viewport")
                                            .into_element(cx)
                                    },
                                    shadcn::DialogFooter::new(vec![
                                        shadcn::Button::new("Close")
                                            .variant(shadcn::ButtonVariant::Secondary)
                                            .test_id("ui-gallery-dialog-close")
                                            .toggle_model(dialog_open.clone())
                                            .into_element(cx),
                                        shadcn::Button::new("Confirm")
                                            .variant(shadcn::ButtonVariant::Outline)
                                            .test_id("ui-gallery-dialog-confirm")
                                            .into_element(cx),
                                    ])
                                    .into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    )
                },
            );

            let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
                cx,
                |cx| {
                    shadcn::Button::new("AlertDialog")
                        .variant(shadcn::ButtonVariant::Outline)
                        .test_id("ui-gallery-alert-dialog-trigger")
                        .toggle_model(alert_dialog_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    cx.semantics(
                        fret_ui::element::SemanticsProps {
                            test_id: Some(Arc::from("ui-gallery-alert-dialog-content")),
                            ..Default::default()
                        },
                        |cx| {
                            vec![
                                shadcn::AlertDialogContent::new(vec![
                                    shadcn::AlertDialogHeader::new(vec![
                                        shadcn::AlertDialogTitle::new("Are you absolutely sure?")
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
                                        .test_id("ui-gallery-alert-dialog-cancel")
                                        .into_element(cx),
                                        shadcn::AlertDialogAction::new(
                                            "Continue",
                                            alert_dialog_open.clone(),
                                        )
                                        .test_id("ui-gallery-alert-dialog-action")
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                ])
                                .into_element(cx),
                            ]
                        },
                    )
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
                            .test_id("ui-gallery-sheet-trigger")
                            .toggle_model(sheet_open.clone())
                            .into_element(cx)
                    },
                    |cx| {
                        cx.semantics(
                            fret_ui::element::SemanticsProps {
                                test_id: Some(Arc::from("ui-gallery-sheet-content")),
                                ..Default::default()
                            },
                            |cx| {
                                vec![
                                    shadcn::SheetContent::new(vec![
                                        shadcn::SheetHeader::new(vec![
                                            shadcn::SheetTitle::new("Sheet").into_element(cx),
                                            shadcn::SheetDescription::new("A modal side panel.")
                                                .into_element(cx),
                                        ])
                                        .into_element(cx),
                                        {
                                            let body = stack::vstack(
                                                cx,
                                                stack::VStackProps::default()
                                                    .gap(Space::N2)
                                                    .layout(
                                                        LayoutRefinement::default()
                                                            .w_full()
                                                            .min_w_0()
                                                            .min_h_0(),
                                                    ),
                                                |cx| {
                                                    (0..96)
                                                        .map(|i| {
                                                            cx.text(format!(
                                                                "Sheet body line {}",
                                                                i + 1
                                                            ))
                                                        })
                                                        .collect::<Vec<_>>()
                                                },
                                            );

                                            shadcn::ScrollArea::new([body])
                                                .refine_layout(
                                                    LayoutRefinement::default()
                                                        .flex_1()
                                                        .w_full()
                                                        .min_w_0()
                                                        .min_h_0(),
                                                )
                                                .viewport_test_id(
                                                    "ui-gallery-sheet-scroll-viewport",
                                                )
                                                .into_element(cx)
                                        },
                                        shadcn::SheetFooter::new(vec![
                                            shadcn::Button::new("Close")
                                                .variant(shadcn::ButtonVariant::Secondary)
                                                .test_id("ui-gallery-sheet-close")
                                                .toggle_model(sheet_open.clone())
                                                .into_element(cx),
                                        ])
                                        .into_element(cx),
                                    ])
                                    .into_element(cx),
                                ]
                            },
                        )
                    },
                );

            let portal_geometry = {
                let popover = shadcn::Popover::new(portal_geometry_popover_open.clone())
                    .side(shadcn::PopoverSide::Right)
                    .align(shadcn::PopoverAlign::Start)
                    .side_offset(Px(8.0))
                    .window_margin(Px(8.0))
                    .arrow(true)
                    .into_element(
                        cx,
                        |cx| {
                            shadcn::Button::new("Portal geometry (scroll + clamp)")
                                .variant(shadcn::ButtonVariant::Outline)
                                .test_id("ui-gallery-portal-geometry-trigger")
                                .toggle_model(portal_geometry_popover_open.clone())
                                .into_element(cx)
                        },
                        |cx| {
                            let close = shadcn::Button::new("Close")
                                .variant(shadcn::ButtonVariant::Secondary)
                                .test_id("ui-gallery-portal-geometry-popover-close")
                                .toggle_model(portal_geometry_popover_open.clone())
                                .into_element(cx);

                            cx.semantics(
                                fret_ui::element::SemanticsProps {
                                    test_id: Some(Arc::from(
                                        "ui-gallery-portal-geometry-popover-content",
                                    )),
                                    ..Default::default()
                                },
                                |cx| {
                                    vec![
                                        shadcn::PopoverContent::new(vec![
                                            cx.text("Popover content (placement + clamp)"),
                                            cx.text("Wheel-scroll the viewport while open."),
                                            close,
                                        ])
                                        .refine_layout(
                                            LayoutRefinement::default()
                                                .w_px(Px(360.0))
                                                .h_px(Px(220.0)),
                                        )
                                        .into_element(cx),
                                    ]
                                },
                            )
                        },
                    );

                let items = (1..=48)
                    .map(|i| cx.text(format!("Scroll item {i:02}")))
                    .collect::<Vec<_>>();

                let body = stack::vstack(cx, stack::VStackProps::default().gap(Space::N2), |_cx| {
                    let mut out: Vec<AnyElement> = Vec::with_capacity(items.len() + 2);
                    out.push(popover);
                    out.extend(items);
                    out
                });

                let scroll = shadcn::ScrollArea::new(vec![body])
                    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)).h_px(Px(160.0)))
                    .into_element(cx);

                let scroll = cx.semantics(
                    fret_ui::element::SemanticsProps {
                        test_id: Some(Arc::from("ui-gallery-portal-geometry-scroll-area")),
                        ..Default::default()
                    },
                    |_cx| vec![scroll],
                );

                shadcn::Card::new(vec![
                    shadcn::CardHeader::new(vec![
                        shadcn::CardTitle::new("Portal geometry").into_element(cx),
                        shadcn::CardDescription::new(
                            "Validates floating placement under scroll + window clamp.",
                        )
                        .into_element(cx),
                    ])
                    .into_element(cx),
                    shadcn::CardContent::new(vec![scroll]).into_element(cx),
                ])
                .refine_layout(LayoutRefinement::default().w_full())
                .into_element(cx)
            };

            let body = stack::vstack(
                cx,
                stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
                |cx| {
                    let theme = Theme::global(&*cx.app).clone();
                    let gap = fret_ui_kit::MetricRef::space(Space::N2).resolve(&theme);

                    let row = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default().w_full().min_w_0(),
                        );
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::Start,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: true,
                            },
                            |_cx| children,
                        )
                    };

                    let row_end = |cx: &mut ElementContext<'_, App>, children: Vec<AnyElement>| {
                        let layout = decl_style::layout_style(
                            &theme,
                            LayoutRefinement::default().w_full().min_w_0(),
                        );
                        cx.flex(
                            fret_ui::element::FlexProps {
                                layout,
                                direction: fret_core::Axis::Horizontal,
                                gap,
                                padding: Edges::all(Px(0.0)),
                                justify: fret_ui::element::MainAlign::End,
                                align: fret_ui::element::CrossAlign::Center,
                                wrap: false,
                            },
                            |_cx| children,
                        )
                    };

                    vec![
                        row(cx, vec![dropdown, context_menu, overlay_reset]),
                        row_end(cx, vec![context_menu_edge]),
                        row(cx, vec![tooltip, hover_card, popover, underlay, dialog]),
                        row(cx, vec![alert_dialog, sheet]),
                        portal_geometry,
                    ]
                },
            );

            vec![body]
        });

    let dialog_open_flag = {
        let open = cx
            .get_model_copied(&dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(cx.semantics(
                fret_ui::element::SemanticsProps {
                    test_id: Some(Arc::from("ui-gallery-dialog-open")),
                    ..Default::default()
                },
                |cx| [cx.text("Dialog open")],
            ))
        } else {
            None
        }
    };

    let alert_dialog_open_flag = {
        let open = cx
            .get_model_copied(&alert_dialog_open, Invalidation::Layout)
            .unwrap_or(false);
        if open {
            Some(cx.semantics(
                fret_ui::element::SemanticsProps {
                    test_id: Some(Arc::from("ui-gallery-alert-dialog-open")),
                    ..Default::default()
                },
                |cx| vec![cx.text("AlertDialog open")],
            ))
        } else {
            None
        }
    };

    let popover_dismissed_flag = {
        let last = cx
            .get_model_cloned(&last_action, Invalidation::Layout)
            .unwrap_or_else(|| Arc::<str>::from("<none>"));
        if last.as_ref() == "popover:dismissed" {
            Some(cx.semantics(
                fret_ui::element::SemanticsProps {
                    test_id: Some(Arc::from("ui-gallery-popover-dismissed")),
                    ..Default::default()
                },
                |cx| [cx.text("Popover dismissed")],
            ))
        } else {
            None
        }
    };

    let mut out: Vec<AnyElement> = vec![overlays, last_action_status];

    if let Some(flag) = popover_dismissed_flag {
        out.push(flag);
    }
    if let Some(flag) = dialog_open_flag {
        out.push(flag);
    }
    if let Some(flag) = alert_dialog_open_flag {
        out.push(flag);
    }

    out
}
