use fret_app::{App, CommandId, Model};
use fret_core::ImageId;
use fret_markdown as markdown;
use fret_ui::Theme;
use fret_ui::elements::ContinuousFrames;
use fret_ui::scroll::VirtualListScrollHandle;
use fret_ui_kit::declarative::CachedSubtreeExt as _;
use fret_ui_kit::{WidgetStateProperty, WidgetStates};
use fret_ui_material3 as material3;
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;
use time::Date;

use crate::spec::*;

fn matches_query(query: &str, item: &PageSpec) -> bool {
    let q = query.trim();
    if q.is_empty() {
        return true;
    }

    let q_lower = q.to_ascii_lowercase();
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

    let query_input = shadcn::Input::new(nav_query)
        .a11y_label("Search components")
        .placeholder("Search (id / tag)")
        .into_element(cx);

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

                    if item.id == PAGE_VIRTUAL_LIST_TORTURE {
                        let on_activate: fret_ui::action::OnActivate =
                            Arc::new(move |host, action_cx, _reason| {
                                let _ =
                                    host.models_mut().update(&selected_page_for_activate, |v| {
                                        *v = page_id_for_activate.clone();
                                    });
                                let _ =
                                    host.models_mut().update(&workspace_tabs_for_activate, |t| {
                                        if !t
                                            .iter()
                                            .any(|id| id.as_ref() == page_id_for_activate.as_ref())
                                        {
                                            t.push(page_id_for_activate.clone());
                                        }
                                    });
                                host.request_redraw(action_cx.window);
                            });
                        button = button.on_activate(on_activate);
                    }

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
    let nav_scroll = if (bisect & BISECT_DISABLE_SIDEBAR_SCROLL) != 0 {
        nav_body
    } else {
        shadcn::ScrollArea::new(vec![nav_body])
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
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
            vec![stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().h_full())
                    .gap(Space::N4),
                |_cx| vec![title_row, query_input, nav_scroll],
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
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let (title, origin, docs_md, usage_md) = page_meta(selected);

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
                        cx.text(title),
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
                |_cx| vec![theme_select, copy_actions],
            );

            vec![left, right]
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
        cmdk_open,
        cmdk_query,
        last_action,
        virtual_list_torture_jump,
        virtual_list_torture_edit_row,
        virtual_list_torture_edit_text,
        virtual_list_torture_scroll,
    );
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

    let tabs = if (bisect & BISECT_DISABLE_TABS) != 0 {
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N6),
            |_cx| vec![preview_panel, usage_panel, docs_panel],
        )
    } else {
        shadcn::Tabs::new(content_tab)
            .refine_layout(LayoutRefinement::default().w_full())
            .list_full_width(true)
            .items([
                shadcn::TabsItem::new("preview", "Preview", vec![preview_panel]),
                shadcn::TabsItem::new("usage", "Usage", vec![usage_panel]),
                shadcn::TabsItem::new("docs", "Notes", vec![docs_panel]),
            ])
            .into_element(cx)
    };

    let body = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N6),
        |_cx| vec![header, tabs],
    );
    let content = if (bisect & BISECT_DISABLE_CONTENT_SCROLL) != 0 {
        body
    } else {
        shadcn::ScrollArea::new(vec![body])
            .refine_layout(LayoutRefinement::default().w_full().h_full())
            .into_element(cx)
    };

    cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .bg(ColorRef::Color(theme.color_required("background")))
                .p(Space::N6),
            LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| vec![content],
    )
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
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
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
            dropdown_open,
            context_menu_open,
            last_action.clone(),
        ),
        PAGE_FORMS => preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_SELECT => preview_select(cx, select_value, select_open),
        PAGE_MATERIAL3_SELECT => preview_material3_select(cx),
        PAGE_MATERIAL3_TEXT_FIELD => preview_material3_text_field(cx),
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
        PAGE_MATERIAL3_GALLERY => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_gallery(
                cx,
                material3_checkbox,
                material3_switch,
                material3_radio_value,
                material3_tabs_value,
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

fn material3_scoped_page(
    cx: &mut ElementContext<'_, App>,
    material3_expressive: Model<bool>,
    content: impl FnOnce(&mut ElementContext<'_, App>) -> Vec<AnyElement>,
) -> Vec<AnyElement> {
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
                cx.text(if enabled {
                    "Variant: Expressive"
                } else {
                    "Variant: Standard"
                }),
            ]
        },
    )
}

fn preview_intro(cx: &mut ElementContext<'_, App>, theme: &Theme) -> Vec<AnyElement> {
    let card = |cx: &mut ElementContext<'_, App>, title: &str, desc: &str| -> AnyElement {
        shadcn::Card::new(vec![
            shadcn::CardHeader::new(vec![shadcn::CardTitle::new(title).into_element(cx)])
                .into_element(cx),
            shadcn::CardContent::new(vec![cx.text(desc)]).into_element(cx),
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
            vec![cx.text("Phase 1: fixed two-pane layout + hardcoded docs strings (focus on validating component usability). Docking/multi-window views will come later.")]
        })
    };

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
                    .on_click(CMD_VIEW_CACHE_BUMP)
                    .into_element(cx),
                shadcn::Button::new("Reset counter")
                    .variant(shadcn::ButtonVariant::Outline)
                    .size(shadcn::ButtonSize::Sm)
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
                        .toggle_model(view_cache_popover_open.clone())
                        .into_element(cx)
                },
                |cx| {
                    shadcn::PopoverContent::new(vec![
                        cx.text("Popover content"),
                        shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
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

        let list = shadcn::ScrollArea::new(vec![stack::vstack(
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
                LayoutRefinement::default().w_full(),
            ),
            |cx| vec![cx.text(label)],
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
        cx.text("Layout mental model: LayoutRefinement (constraints) + stack (composition) + Theme tokens (color/spacing)."),
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

    let header_editing_row = cx
        .get_model_copied(&virtual_list_torture_edit_row, Invalidation::Layout)
        .flatten();

    let jump_input = {
        let mut props = fret_ui::element::TextInputProps::new(virtual_list_torture_jump.clone());
        props.a11y_label = Some(Arc::<str>::from("Jump to row"));
        props.test_id = Some(Arc::<str>::from("ui-gallery-virtual-list-jump-input"));
        props.placeholder = Some(Arc::<str>::from("Row index (e.g. 9000)"));
        props.layout.size.width = fret_ui::element::Length::Fill;
        cx.text_input(props)
    };

    let controls = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .items_center(),
        |cx| {
            let jump_model = virtual_list_torture_jump.clone();
            let scroll_for_jump = virtual_list_torture_scroll.clone();
            let on_jump: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
                let raw = host
                    .models_mut()
                    .get_cloned(&jump_model)
                    .unwrap_or_default();
                let index = raw.trim().parse::<usize>().unwrap_or(0);
                scroll_for_jump.scroll_to_item(index, fret_ui::scroll::ScrollStrategy::Start);
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
    );

    let editing_indicator = {
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
    };

    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2),
        |cx| {
            vec![
                cx.text("Goal: deterministic virtualization torture surface (10k rows + scroll-to-item + inline edit)."),
                controls,
                editing_indicator,
            ]
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

    let options = fret_ui::element::VirtualListOptions::new(Px(28.0), 10);

    let list = cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
        let editing_row = cx
            .get_model_copied(&virtual_list_torture_edit_row, Invalidation::Layout)
            .flatten();

        let list = cx.virtual_list_keyed_with_layout(
            list_layout,
            len,
            options,
            &virtual_list_torture_scroll,
            |i| i as fret_ui::ItemKey,
            |cx, index| {
                let index_u64 = index as u64;
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
                    .refine_layout(LayoutRefinement::default().flex_1())
                    .into_element(cx);

                let right = if is_editing {
                    let mut props = fret_ui::element::TextInputProps::new(
                        virtual_list_torture_edit_text.clone(),
                    );
                    props.a11y_label = Some(Arc::<str>::from("Inline edit"));
                    props.test_id = Some(Arc::<str>::from("ui-gallery-virtual-list-edit-input"));
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
                    LayoutRefinement::default().w_full().h_px(height_hint),
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
            },
        );

        let list = cx.semantics(
            fret_ui::element::SemanticsProps {
                role: fret_core::SemanticsRole::List,
                test_id: Some(Arc::<str>::from("ui-gallery-virtual-list-root")),
                ..Default::default()
            },
            |_cx| vec![list],
        );

        vec![list]
    });

    vec![header, list]
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

fn preview_material3_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::ButtonVariant,
               label: &'static str| {
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::Button::new(label)
                        .variant(variant)
                        .into_element(cx),
                    material3::Button::new("Disabled")
                        .variant(variant)
                        .disabled(true)
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
            ]
        },
    ));

    out.push(cx.text("— Selection —"));
    out.push(stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N3).items_center(),
        |cx| {
            vec![
                material3::Checkbox::new(material3_checkbox.clone())
                    .a11y_label("Checkbox")
                    .into_element(cx),
                material3::Switch::new(material3_switch.clone())
                    .a11y_label("Switch")
                    .into_element(cx),
                material3::RadioGroup::new(material3_radio_value.clone())
                    .a11y_label("Radio Group")
                    .items(vec![
                        material3::RadioGroupItem::new("Alpha").a11y_label("Radio Alpha"),
                        material3::RadioGroupItem::new("Beta").a11y_label("Radio Beta"),
                        material3::RadioGroupItem::new("Charlie")
                            .a11y_label("Radio Charlie")
                            .disabled(true),
                    ])
                    .into_element(cx),
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
    out.push(
        material3::TextField::new(material3_text_field_value)
            .label("Label")
            .placeholder("Placeholder")
            .disabled(disabled)
            .error(error)
            .into_element(cx),
    );

    out.push(cx.text("— Tabs —"));
    out.push(
        material3::Tabs::new(material3_tabs_value)
            .a11y_label("Tabs")
            .items(vec![
                material3::TabItem::new("overview", "Overview"),
                material3::TabItem::new("security", "Security"),
                material3::TabItem::new("settings", "Settings"),
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
        .refine_layout(
            LayoutRefinement::default()
                .w_px(MetricRef::Px(Px(280.0)))
                .min_w_0(),
        )
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

    let row = |cx: &mut ElementContext<'_, App>,
               variant: material3::IconButtonVariant,
               label: &'static str| {
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            move |cx| {
                vec![
                    material3::IconButton::new(ids::ui::CLOSE)
                        .variant(variant)
                        .a11y_label(label)
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
    let value = cx
        .get_model_copied(&checked, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                material3::Checkbox::new(checked.clone())
                    .a11y_label("Material 3 Checkbox")
                    .test_id("ui-gallery-material3-checkbox")
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
    let value = cx
        .get_model_copied(&selected, Invalidation::Layout)
        .unwrap_or(false);

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        move |cx| {
            vec![
                material3::Switch::new(selected.clone())
                    .a11y_label("Material 3 Switch")
                    .test_id("ui-gallery-material3-switch")
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
    let current = cx
        .get_model_cloned(&group_value, Invalidation::Layout)
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N4).items_center(),
        move |cx| {
            vec![
                material3::RadioGroup::new(group_value.clone())
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

    vec![
        cx.text(
            "Material 3 Radio: group-value binding + roving focus + typeahead + state layer + bounded ripple.",
        ),
        row,
    ]
}

fn preview_material3_text_field(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    disabled: Model<bool>,
    error: Model<bool>,
) -> Vec<AnyElement> {
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

    let filled_field = material3::TextField::new(value)
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
                "Active indicator bottom border + filled container + hover state layer (best-effort).",
            )
                .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![filled_field]).into_element(cx),
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
    ]
}

fn preview_material3_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Arc<str>>,
) -> Vec<AnyElement> {
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
        move |_cx| vec![rail],
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
        move |_cx| vec![drawer],
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
        move |_cx| vec![modal],
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

    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);

    let open_dialog: OnActivate = {
        let open = open.clone();
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = true);
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

    let dialog = material3::Dialog::new(open.clone())
        .headline("Discard draft?")
        .supporting_text("This action cannot be undone.")
        .actions(vec![
            material3::DialogAction::new("Cancel")
                .test_id("ui-gallery-material3-dialog-action-cancel")
                .on_activate(close_dialog.clone()),
            material3::DialogAction::new("Discard")
                .test_id("ui-gallery-material3-dialog-action-discard")
                .on_activate(confirm_action.clone()),
        ])
        .test_id("ui-gallery-material3-dialog")
        .into_element(
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
                                .on_activate(open_dialog.clone())
                                .test_id("ui-gallery-material3-dialog-open")
                                .into_element(cx),
                            material3::Button::new("Underlay focus probe")
                                .variant(material3::ButtonVariant::Outlined)
                                .test_id("ui-gallery-material3-dialog-underlay-probe")
                                .into_element(cx),
                            cx.text("Tip: press Esc or click the scrim to close; Tab should stay inside the dialog while open."),
                        ]
                    },
                )
            },
            |_cx| vec![],
        );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let mut layout = fret_ui::element::LayoutStyle::default();
    layout.size.width = fret_ui::element::Length::Fill;
    layout.size.height = fret_ui::element::Length::Px(Px(360.0));

    let container = cx.container(
        fret_ui::element::ContainerProps {
            layout,
            ..Default::default()
        },
        move |_cx| vec![dialog],
    );

    vec![
        cx.text(
            "Material 3 Dialog: modal barrier + focus trap/restore + token-shaped dialog actions.",
        ),
        container,
        cx.text(format!(
            "open={} last_action={}",
            is_open as u8,
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
        Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&open, |v| *v = !*v);
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

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let card = shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Menu").into_element(cx),
            shadcn::CardDescription::new(
                "Overlay MVP (dismissible, anchored) using the Menu list surface.",
            )
            .into_element(cx),
        ])
        .into_element(cx),
        shadcn::CardContent::new(vec![dropdown]).into_element(cx),
    ])
    .refine_layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx);

    vec![
        cx.text("Tip: Arrow keys / Home / End navigate; type to jump by prefix; Esc/outside press closes."),
        card,
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
        move |_cx| vec![standard, expressive],
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
    let content = material3::TooltipProvider::new().with(cx, |cx| {
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
                |_cx| vec![top, right, bottom, left],
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
    .refine_layout(LayoutRefinement::default().flex_1())
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
    .refine_layout(LayoutRefinement::default().flex_1())
    .into_element(cx);

    vec![stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N4)
            .items_stretch(),
        |_cx| vec![left, right],
    )]
}

fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let row = stack::hstack(
        cx,
        stack::HStackProps::default().gap(Space::N2).items_center(),
        |cx| {
            vec![
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
        shadcn::Avatar::new(vec![image, fallback]).into_element(cx)
    };

    let b = shadcn::Avatar::new(vec![shadcn::AvatarFallback::new("WK").into_element(cx)])
        .into_element(cx);

    let c = shadcn::Avatar::new(vec![shadcn::AvatarFallback::new("?").into_element(cx)])
        .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
        .into_element(cx);

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N3).items_center(),
            |_cx| vec![a, b, c],
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

    let scroll = shadcn::ScrollArea::new(vec![body])
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
    shadcn::TooltipProvider::new().with(cx, |cx| {
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
}

fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    cx.keyed("ui_gallery.slider_page", |cx| {
        let single = shadcn::Slider::new_controllable(cx, None, || vec![35.0])
            .range(0.0, 100.0)
            .into_element(cx);

        let range = shadcn::Slider::new_controllable(cx, None, || vec![20.0, 80.0])
            .range(0.0, 100.0)
            .min_steps_between_thumbs(5)
            .into_element(cx);

        let disabled = shadcn::Slider::new_controllable(cx, None, || vec![60.0])
            .disabled(true)
            .into_element(cx);

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
                |_cx| vec![row],
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
                            cx.text("Accept terms"),
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
                            cx.text("Enable feature"),
                        ]
                    },
                ),
            ]
        },
    );

    vec![
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N3),
            |_cx| vec![input, textarea, toggles],
        ),
        cx.text(
            "Tip: these are model-bound controls; values persist while you stay in the window.",
        ),
    ]
}

fn preview_select(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    let select = shadcn::Select::new(value.clone(), open)
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
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![select, cx.text(format!("Selected: {selected}"))]
}

fn preview_material3_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct Models {
        value: Option<Model<Option<Arc<str>>>>,
        open: Option<Model<bool>>,
        value_override: Option<Model<Option<Arc<str>>>>,
        open_override: Option<Model<bool>>,
        disabled: Option<Model<bool>>,
    }

    let (value, open, value_override, open_override, disabled) =
        cx.with_state(Models::default, |st| {
            (
                st.value.clone(),
                st.open.clone(),
                st.value_override.clone(),
                st.open_override.clone(),
                st.disabled.clone(),
            )
        });

    let (value, open, value_override, open_override, disabled) =
        match (value, open, value_override, open_override, disabled) {
            (
                Some(value),
                Some(open),
                Some(value_override),
                Some(open_override),
                Some(disabled),
            ) => (value, open, value_override, open_override, disabled),
            _ => {
                let value = cx.app.models_mut().insert(None::<Arc<str>>);
                let open = cx.app.models_mut().insert(false);
                let value_override = cx.app.models_mut().insert(None::<Arc<str>>);
                let open_override = cx.app.models_mut().insert(false);
                let disabled = cx.app.models_mut().insert(false);
                cx.with_state(Models::default, |st| {
                    st.value = Some(value.clone());
                    st.open = Some(open.clone());
                    st.value_override = Some(value_override.clone());
                    st.open_override = Some(open_override.clone());
                    st.disabled = Some(disabled.clone());
                });
                (value, open, value_override, open_override, disabled)
            }
        };

    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);

    let toggle_disabled = shadcn::Button::new(if disabled_now { "Enable" } else { "Disable" })
        .variant(shadcn::ButtonVariant::Outline)
        .toggle_model(disabled.clone())
        .into_element(cx);

    let theme = Theme::global(&*cx.app).clone();
    let override_style = material3::SelectStyle::default()
        .trigger_border_color(WidgetStateProperty::new(None).when(
            WidgetStates::FOCUS_VISIBLE,
            Some(ColorRef::Color(theme.color_required("destructive"))),
        ))
        .option_background(
            WidgetStateProperty::new(None).when(
                WidgetStates::SELECTED,
                Some(ColorRef::Color(
                    theme
                        .color_by_key("accent")
                        .unwrap_or_else(|| theme.color_required("accent")),
                )),
            ),
        );

    let select = material3::Select::new(value.clone(), open.clone())
        .placeholder("Default (pilot)")
        .disabled(disabled_now)
        .items([
            material3::SelectItem::new("apple", "Apple"),
            material3::SelectItem::new("banana", "Banana"),
            material3::SelectItem::new("orange", "Orange").disabled(true),
        ])
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
        .into_element(cx);

    let select_override = material3::Select::new(value_override.clone(), open_override.clone())
        .placeholder("Override (focus outline = destructive)")
        .disabled(disabled_now)
        .style(override_style)
        .items([
            material3::SelectItem::new("apple", "Apple"),
            material3::SelectItem::new("banana", "Banana"),
            material3::SelectItem::new("orange", "Orange").disabled(true),
        ])
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(240.0))))
        .into_element(cx);

    let selected = cx
        .app
        .models()
        .read(&value, |v| v.clone())
        .ok()
        .flatten()
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![toggle_disabled],
        ),
        stack::vstack(
            cx,
            stack::VStackProps::default().gap(Space::N3),
            |_cx| vec![select, select_override],
        ),
        cx.text(format!("Selected: {selected}")),
        cx.text(
            "Tip: Tab to focus the trigger (focus-visible ring), then Space/Enter/ArrowDown to open.",
        ),
    ]
}

fn preview_material3_text_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    #[derive(Default)]
    struct Models {
        model: Option<Model<String>>,
        model_override: Option<Model<String>>,
        disabled: Option<Model<bool>>,
    }

    let (model, model_override, disabled) = cx.with_state(Models::default, |st| {
        (
            st.model.clone(),
            st.model_override.clone(),
            st.disabled.clone(),
        )
    });
    let (model, model_override, disabled) = match (model, model_override, disabled) {
        (Some(model), Some(model_override), Some(disabled)) => (model, model_override, disabled),
        _ => {
            let model = cx.app.models_mut().insert(String::new());
            let model_override = cx.app.models_mut().insert(String::new());
            let disabled = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.model = Some(model.clone());
                st.model_override = Some(model_override.clone());
                st.disabled = Some(disabled.clone());
            });
            (model, model_override, disabled)
        }
    };

    let disabled_now = cx
        .get_model_copied(&disabled, Invalidation::Layout)
        .unwrap_or(false);

    let toggle_disabled = shadcn::Button::new(if disabled_now { "Enable" } else { "Disable" })
        .variant(shadcn::ButtonVariant::Outline)
        .toggle_model(disabled.clone())
        .into_element(cx);

    let clear = {
        let model_for_clear = model.clone();
        let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _reason| {
            let _ = host.models_mut().update(&model_for_clear, |v| v.clear());
            host.request_redraw(action_cx.window);
        });
        shadcn::Button::new("Clear")
            .variant(shadcn::ButtonVariant::Secondary)
            .on_activate(on_activate)
            .into_element(cx)
    };

    let theme = Theme::global(&*cx.app).clone();
    let override_style = material3::TextFieldStyle::default()
        .border_color_focused(WidgetStateProperty::new(None).when(
            WidgetStates::FOCUS_VISIBLE,
            Some(ColorRef::Color(theme.color_required("destructive"))),
        ))
        .focus_ring_color(WidgetStateProperty::new(None).when(
            WidgetStates::FOCUS_VISIBLE,
            Some(ColorRef::Color(theme.color_required("destructive"))),
        ));

    let field = material3::TextField::new(model.clone())
        .a11y_label("Material3 text field")
        .placeholder("Default (pilot)")
        .disabled(disabled_now)
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
        .into_element(cx);

    let field_override = material3::TextField::new(model_override.clone())
        .a11y_label("Material3 text field (override)")
        .placeholder("Override (focus ring = destructive)")
        .disabled(disabled_now)
        .style(override_style)
        .refine_layout(LayoutRefinement::default().w_px(MetricRef::Px(Px(320.0))))
        .into_element(cx);

    vec![
        stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2).items_center(),
            |_cx| vec![toggle_disabled, clear],
        ),
        stack::vstack(cx, stack::VStackProps::default().gap(Space::N3), |_cx| {
            vec![field, field_override]
        }),
        cx.text("Tip: use Tab to focus the field and observe focus-visible ring behavior."),
    ]
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
        cx.container(props, move |cx| vec![cx.text(title)])
    };

    let nested_vertical = shadcn::ResizablePanelGroup::new(v_fractions)
        .axis(fret_core::Axis::Vertical)
        .entries(vec![
            shadcn::ResizablePanel::new(vec![boxy(cx, "Viewport", "muted")])
                .min_px(Px(120.0))
                .into(),
            shadcn::ResizableHandle::new().into(),
            shadcn::ResizablePanel::new(vec![boxy(cx, "Console", "card")])
                .min_px(Px(80.0))
                .into(),
        ])
        .into_element(cx);

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
            |cx, col, row| match col.id.as_ref() {
                "name" => cx.text(row.name.as_ref()),
                "status" => cx.text(row.status.as_ref()),
                "cpu%" => cx.text(format!("{}%", row.cpu)),
                "mem_mb" => cx.text(format!("{} MB", row.mem_mb)),
                _ => cx.text("?"),
            },
        );

    vec![
        cx.text("Click header to sort; click row to toggle selection."),
        cx.text(format!("Selected rows: {selected_count}")),
        cx.text(sorting_text.as_ref()),
        table,
    ]
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
        .refine_layout(LayoutRefinement::default().w_full())
        .list_full_width(false)
        .items([
            shadcn::TabsItem::new(
                "overview",
                "Overview",
                vec![cx.text("Tabs are stateful and roving-focus friendly.")],
            ),
            shadcn::TabsItem::new(
                "details",
                "Details",
                vec![cx.text("Use Tabs for sections within a single page.")],
            ),
            shadcn::TabsItem::new(
                "notes",
                "Notes",
                vec![cx.text(
                    "In this gallery, the outer shell also uses Tabs (Preview/Usage/Notes).",
                )],
            ),
        ])
        .into_element(cx);

    vec![tabs, cx.text(format!("Selected: {selected}"))]
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
                shadcn::TableCell::new(cx.text("fret-ui")).into_element(cx),
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

    vec![table]
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
            |_cx| vec![dropdown, context_menu],
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
                    .on_click(CMD_TOAST_DEFAULT)
                    .into_element(cx),
                shadcn::Button::new("Success")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_SUCCESS)
                    .into_element(cx),
                shadcn::Button::new("Error")
                    .variant(shadcn::ButtonVariant::Outline)
                    .on_click(CMD_TOAST_ERROR)
                    .into_element(cx),
                shadcn::Button::new("Action + Cancel")
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
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
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
            |cx| vec![cx.text(text)],
        )
    };

    let overlays =
        cx.cached_subtree_with(CachedSubtreeProps::default().contained_layout(true), |cx| {
            let overlay_reset = {
                use fret_ui::action::OnActivate;

                let dropdown_open = dropdown_open.clone();
                let context_menu_open = context_menu_open.clone();
                let popover_open = popover_open.clone();
                let dialog_open = dialog_open.clone();
                let alert_dialog_open = alert_dialog_open.clone();
                let sheet_open = sheet_open.clone();
                let last_action = last_action.clone();

                let on_activate: OnActivate = Arc::new(move |host, _cx, _reason| {
                    let _ = host.models_mut().update(&dropdown_open, |v| *v = false);
                    let _ = host.models_mut().update(&context_menu_open, |v| *v = false);
                    let _ = host.models_mut().update(&popover_open, |v| *v = false);
                    let _ = host.models_mut().update(&dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&alert_dialog_open, |v| *v = false);
                    let _ = host.models_mut().update(&sheet_open, |v| *v = false);
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

            let dropdown = shadcn::DropdownMenu::new(dropdown_open.clone()).into_element(
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
                        let close = shadcn::Button::new("Close")
                            .variant(shadcn::ButtonVariant::Secondary)
                            .test_id("ui-gallery-popover-close")
                            .toggle_model(popover_open.clone())
                            .into_element(cx);

                        shadcn::PopoverContent::new(vec![cx.text("Popover content"), close])
                            .into_element(cx)
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
                    shadcn::DialogContent::new(vec![
                        shadcn::DialogHeader::new(vec![
                            shadcn::DialogTitle::new("Dialog").into_element(cx),
                            shadcn::DialogDescription::new("Escape / overlay click closes")
                                .into_element(cx),
                        ])
                        .into_element(cx),
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
                    .into_element(cx)
                },
            );

            let alert_dialog = shadcn::AlertDialog::new(alert_dialog_open.clone()).into_element(
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
                            shadcn::AlertDialogTitle::new("Are you absolutely sure?")
                                .into_element(cx),
                            shadcn::AlertDialogDescription::new(
                                "This is non-closable by overlay click.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::AlertDialogFooter::new(vec![
                            shadcn::AlertDialogCancel::new("Cancel", alert_dialog_open.clone())
                                .into_element(cx),
                            shadcn::AlertDialogAction::new("Continue", alert_dialog_open.clone())
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
                                shadcn::SheetTitle::new("Sheet").into_element(cx),
                                shadcn::SheetDescription::new("A modal side panel.")
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

            let body = stack::vstack(
                cx,
                stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
                |cx| {
                    vec![
                        stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |_cx| vec![dropdown, context_menu, overlay_reset],
                        ),
                        stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |_cx| vec![tooltip, hover_card, popover, underlay, dialog],
                        ),
                        stack::hstack(
                            cx,
                            stack::HStackProps::default().gap(Space::N2).items_center(),
                            |_cx| vec![alert_dialog, sheet],
                        ),
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
                |cx| vec![cx.text("Dialog open")],
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
                |cx| vec![cx.text("Popover dismissed")],
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

    out
}
