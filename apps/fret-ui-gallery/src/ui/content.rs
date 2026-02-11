use super::*;

pub(crate) fn content_view(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    selected: &str,
    models: &UiGalleryModels,
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

            let theme_select = shadcn::Select::new(
                models.theme_preset.clone(),
                models.theme_preset_open.clone(),
            )
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
        models.view_cache_enabled.clone(),
        models.view_cache_cache_shell.clone(),
        models.view_cache_inner_enabled.clone(),
        models.view_cache_popover_open.clone(),
        models.view_cache_continuous.clone(),
        models.view_cache_counter.clone(),
        models.popover_open.clone(),
        models.dialog_open.clone(),
        models.alert_dialog_open.clone(),
        models.sheet_open.clone(),
        models.portal_geometry_popover_open.clone(),
        models.select_value.clone(),
        models.select_open.clone(),
        models.combobox_value.clone(),
        models.combobox_open.clone(),
        models.combobox_query.clone(),
        models.date_picker_open.clone(),
        models.date_picker_month.clone(),
        models.date_picker_selected.clone(),
        models.time_picker_open.clone(),
        models.time_picker_selected.clone(),
        models.resizable_h_fractions.clone(),
        models.resizable_v_fractions.clone(),
        models.data_table_state.clone(),
        models.data_grid_selected_row.clone(),
        models.tabs_value.clone(),
        models.accordion_value.clone(),
        models.avatar_demo_image.clone(),
        models.image_fit_demo_wide_image.clone(),
        models.image_fit_demo_tall_image.clone(),
        models.image_fit_demo_streaming_image.clone(),
        models.progress.clone(),
        models.checkbox.clone(),
        models.switch.clone(),
        models.material3_checkbox.clone(),
        models.material3_switch.clone(),
        models.material3_radio_value.clone(),
        models.material3_tabs_value.clone(),
        models.material3_list_value.clone(),
        models.material3_expressive.clone(),
        models.material3_navigation_bar_value.clone(),
        models.material3_navigation_rail_value.clone(),
        models.material3_navigation_drawer_value.clone(),
        models.material3_modal_navigation_drawer_open.clone(),
        models.material3_dialog_open.clone(),
        models.material3_text_field_value.clone(),
        models.material3_text_field_disabled.clone(),
        models.material3_text_field_error.clone(),
        models.material3_autocomplete_value.clone(),
        models.material3_autocomplete_disabled.clone(),
        models.material3_autocomplete_error.clone(),
        models.material3_autocomplete_dialog_open.clone(),
        models.material3_menu_open.clone(),
        models.text_input.clone(),
        models.text_area.clone(),
        models.dropdown_open.clone(),
        models.context_menu_open.clone(),
        models.context_menu_edge_open.clone(),
        models.cmdk_open.clone(),
        models.cmdk_query.clone(),
        models.last_action.clone(),
        models.sonner_position.clone(),
        models.virtual_list_torture_jump.clone(),
        models.virtual_list_torture_edit_row.clone(),
        models.virtual_list_torture_edit_text.clone(),
        models.virtual_list_torture_scroll.clone(),
        models.code_editor_syntax_rust.clone(),
        models.code_editor_boundary_identifier.clone(),
        models.code_editor_soft_wrap.clone(),
        models.code_editor_folds.clone(),
        models.code_editor_inlays.clone(),
    );

    let active_tab: Arc<str> = cx
        .watch_model(&models.content_tab)
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
        shadcn::Tabs::new(models.content_tab.clone())
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

    let content = content_inner.test_id("ui-gallery-content-scroll");

    cx.named("ui_gallery.content_view_root", |cx| {
        cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default()
                    .bg(ColorRef::Color(theme.color_required("background")))
                    .p(Space::N6),
                LayoutRefinement::default().w_full().h_full(),
            ),
            |_cx| [content],
        )
        .test_id(page_test_id)
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
    time_picker_open: Model<bool>,
    time_picker_selected: Model<time::Time>,
    resizable_h_fractions: Model<Vec<f32>>,
    resizable_v_fractions: Model<Vec<f32>>,
    data_table_state: Model<fret_ui_headless::table::TableState>,
    data_grid_selected_row: Model<Option<u64>>,
    tabs_value: Model<Option<Arc<str>>>,
    accordion_value: Model<Option<Arc<str>>>,
    avatar_demo_image: Model<Option<ImageId>>,
    image_fit_demo_wide_image: Model<Option<ImageId>>,
    image_fit_demo_tall_image: Model<Option<ImageId>>,
    image_fit_demo_streaming_image: Model<Option<ImageId>>,
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
    material3_autocomplete_value: Model<String>,
    material3_autocomplete_disabled: Model<bool>,
    material3_autocomplete_error: Model<bool>,
    material3_autocomplete_dialog_open: Model<bool>,
    material3_menu_open: Model<bool>,
    text_input: Model<String>,
    text_area: Model<String>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    context_menu_edge_open: Model<bool>,
    cmdk_open: Model<bool>,
    cmdk_query: Model<String>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
    virtual_list_torture_jump: Model<String>,
    virtual_list_torture_edit_row: Model<Option<u64>>,
    virtual_list_torture_edit_text: Model<String>,
    virtual_list_torture_scroll: VirtualListScrollHandle,
    code_editor_syntax_rust: Model<bool>,
    code_editor_boundary_identifier: Model<bool>,
    code_editor_soft_wrap: Model<bool>,
    code_editor_folds: Model<bool>,
    code_editor_inlays: Model<bool>,
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
        PAGE_HIT_TEST_ONLY_PAINT_CACHE_PROBE => preview_hit_test_only_paint_cache_probe(cx, theme),
        PAGE_VIRTUAL_LIST_TORTURE => preview_virtual_list_torture(
            cx,
            theme,
            virtual_list_torture_jump,
            virtual_list_torture_edit_row,
            virtual_list_torture_edit_text,
            virtual_list_torture_scroll,
        ),
        PAGE_UI_KIT_LIST_TORTURE => preview_ui_kit_list_torture(cx, theme),
        PAGE_CODE_VIEW_TORTURE => preview_code_view_torture(cx, theme),
        PAGE_CODE_EDITOR_MVP => preview_code_editor_mvp(
            cx,
            theme,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
            code_editor_soft_wrap,
        ),
        PAGE_CODE_EDITOR_TORTURE => preview_code_editor_torture(
            cx,
            theme,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
            code_editor_soft_wrap,
            code_editor_folds,
            code_editor_inlays,
        ),
        PAGE_MARKDOWN_EDITOR_SOURCE => preview_markdown_editor_source(
            cx,
            theme,
            code_editor_soft_wrap,
            code_editor_folds,
            code_editor_inlays,
        ),
        PAGE_TEXT_SELECTION_PERF => preview_text_selection_perf(cx, theme),
        PAGE_TEXT_BIDI_RTL_CONFORMANCE => preview_text_bidi_rtl_conformance(cx, theme),
        PAGE_TEXT_MEASURE_OVERLAY => preview_text_measure_overlay(cx, theme),
        PAGE_WEB_IME_HARNESS => preview_web_ime_harness(cx, theme, text_input, text_area),
        PAGE_CHART_TORTURE => preview_chart_torture(cx, theme),
        PAGE_CANVAS_CULL_TORTURE => preview_canvas_cull_torture(cx, theme),
        PAGE_NODE_GRAPH_CULL_TORTURE => preview_node_graph_cull_torture(cx, theme),
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
        PAGE_TABLE_RETAINED_TORTURE => preview_table_retained_torture(cx, theme),
        PAGE_AI_TRANSCRIPT_TORTURE => preview_ai_transcript_torture(cx, theme),
        PAGE_AI_CHAT_DEMO => preview_ai_chat_demo(cx, theme),
        PAGE_AI_FILE_TREE_DEMO => preview_ai_file_tree_demo(cx, theme),
        PAGE_INSPECTOR_TORTURE => preview_inspector_torture(cx, theme),
        PAGE_FILE_TREE_TORTURE => preview_file_tree_torture(cx, theme),
        PAGE_BUTTON => preview_button(cx),
        PAGE_CARD => preview_card(cx),
        PAGE_BADGE => preview_badge(cx),
        PAGE_AVATAR => preview_avatar(cx, avatar_demo_image),
        PAGE_IMAGE_OBJECT_FIT => preview_image_object_fit(
            cx,
            theme,
            avatar_demo_image,
            image_fit_demo_wide_image,
            image_fit_demo_tall_image,
            image_fit_demo_streaming_image,
        ),
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
        PAGE_SHADCN_EXTRAS => preview_shadcn_extras(cx),
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
        PAGE_SONNER => preview_sonner(cx, last_action.clone(), sonner_position.clone()),
        PAGE_ALERT => preview_alert(cx),
        PAGE_ALERT_DIALOG => preview_alert_dialog(cx, alert_dialog_open),
        PAGE_ASPECT_RATIO => preview_aspect_ratio(cx),
        PAGE_BREADCRUMB => preview_breadcrumb(cx, last_action.clone()),
        PAGE_BUTTON_GROUP => preview_button_group(cx),
        PAGE_CALENDAR => preview_calendar(cx, date_picker_month, date_picker_selected),
        PAGE_CAROUSEL => preview_carousel(cx),
        PAGE_CHART => preview_chart(cx),
        PAGE_CHECKBOX => preview_checkbox(cx, checkbox),
        PAGE_COLLAPSIBLE => preview_collapsible(cx),
        PAGE_CONTEXT_MENU => preview_context_menu(cx, context_menu_open, last_action.clone()),
        PAGE_DIALOG => preview_dialog(cx, dialog_open),
        PAGE_DRAWER => preview_drawer(cx),
        PAGE_DROPDOWN_MENU => preview_dropdown_menu(cx, dropdown_open, last_action.clone()),
        PAGE_EMPTY => preview_empty(cx),
        PAGE_FORM => preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_HOVER_CARD => preview_hover_card(cx),
        PAGE_INPUT => preview_input(cx, text_input),
        PAGE_INPUT_GROUP => preview_input_group(cx),
        PAGE_INPUT_OTP => preview_input_otp(cx),
        PAGE_ITEM => preview_item(cx),
        PAGE_KBD => preview_kbd(cx),
        PAGE_LABEL => preview_label(cx),
        PAGE_MENUBAR => preview_menubar(cx),
        PAGE_NATIVE_SELECT => preview_native_select(cx),
        PAGE_NAVIGATION_MENU => preview_navigation_menu(cx),
        PAGE_PAGINATION => preview_pagination(cx),
        PAGE_POPOVER => preview_popover(cx, popover_open),
        PAGE_RADIO_GROUP => preview_radio_group(cx),
        PAGE_SEPARATOR => preview_separator(cx),
        PAGE_SHEET => preview_sheet(cx, sheet_open),
        PAGE_SIDEBAR => preview_sidebar(cx),
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
        PAGE_MATERIAL3_BADGE => {
            material3_scoped_page(cx, material3_expressive.clone(), preview_material3_badge)
        }
        PAGE_MATERIAL3_TOP_APP_BAR => material3_scoped_page(
            cx,
            material3_expressive.clone(),
            preview_material3_top_app_bar,
        ),
        PAGE_MATERIAL3_BOTTOM_SHEET => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_bottom_sheet(cx, sheet_open)
            })
        }
        PAGE_MATERIAL3_DATE_PICKER => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_date_picker(
                    cx,
                    date_picker_open,
                    date_picker_month,
                    date_picker_selected,
                )
            })
        }
        PAGE_MATERIAL3_TIME_PICKER => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_time_picker(cx, time_picker_open, time_picker_selected)
            })
        }
        PAGE_MATERIAL3_SEGMENTED_BUTTON => material3_scoped_page(
            cx,
            material3_expressive.clone(),
            preview_material3_segmented_button,
        ),
        PAGE_MATERIAL3_SELECT => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_select(cx)
        }),
        PAGE_MATERIAL3_AUTOCOMPLETE => {
            material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                preview_material3_autocomplete(
                    cx,
                    material3_autocomplete_value,
                    material3_autocomplete_disabled,
                    material3_autocomplete_error,
                    material3_autocomplete_dialog_open,
                )
            })
        }
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
