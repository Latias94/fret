use super::*;
use fret::UiCx;

pub(crate) fn content_view(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    selected: &str,
    models: &UiGalleryModels,
) -> AnyElement {
    let bisect = ui_gallery_bisect_flags();

    let (title, origin) = page_meta(selected);
    let page_test_id: Arc<str> =
        Arc::from(format!("ui-gallery-page-{}", selected.replace('_', "-")));

    // Avoid viewport-size branching in the header because this content view can be wrapped in a
    // ViewCache root (`FRET_UI_GALLERY_VIEW_CACHE_SHELL=1`). Querying viewport bounds with
    // `Invalidation::Layout` would churn the view-cache key during interactive resize and defeat
    // reuse.
    let header_content = ui::v_flex(|cx| {
        let left = ui::v_flex(|cx| {
            [
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
                    align: fret_core::TextAlign::Start,
                    ink_overflow: fret_ui::element::TextInkOverflow::None,
                }),
                cx.text_props(TextProps {
                    layout: {
                        let mut layout = fret_ui::element::LayoutStyle::default();
                        layout.size.width = fret_ui::element::Length::Fill;
                        layout
                    },
                    text: Arc::from(origin),
                    style: None,
                    color: Some(theme.color_token("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Ellipsis,
                    align: fret_core::TextAlign::Start,
                    ink_overflow: fret_ui::element::TextInkOverflow::None,
                }),
            ]
        })
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N1)
        .items_start()
        .into_element(cx);

        let theme_select = shadcn::Select::new(
            models.theme_preset.clone(),
            models.theme_preset_open.clone(),
        )
        .value(shadcn::SelectValue::new().placeholder("Theme preset"))
        .trigger_test_id("ui-gallery-theme-preset-trigger")
        .items([
            shadcn::SelectItem::new("zinc/light", "Zinc (light)")
                .test_id("ui-gallery-theme-preset-item-zinc-light"),
            shadcn::SelectItem::new("zinc/dark", "Zinc (dark)")
                .test_id("ui-gallery-theme-preset-item-zinc-dark"),
            shadcn::SelectItem::new("slate/light", "Slate (light)")
                .test_id("ui-gallery-theme-preset-item-slate-light"),
            shadcn::SelectItem::new("slate/dark", "Slate (dark)")
                .test_id("ui-gallery-theme-preset-item-slate-dark"),
            shadcn::SelectItem::new("neutral/light", "Neutral (light)")
                .test_id("ui-gallery-theme-preset-item-neutral-light"),
            shadcn::SelectItem::new("neutral/dark", "Neutral (dark)")
                .test_id("ui-gallery-theme-preset-item-neutral-dark"),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(180.0))
                .max_w(Px(220.0))
                .flex_shrink(1.0),
        )
        .into_element(cx);

        let motion_select = shadcn::Select::new(
            models.motion_preset.clone(),
            models.motion_preset_open.clone(),
        )
        .value(shadcn::SelectValue::new().placeholder("Motion preset"))
        .trigger_test_id("ui-gallery-motion-preset-trigger")
        .items([
            shadcn::SelectItem::new("theme", "Theme (baseline)")
                .test_id("ui-gallery-motion-preset-item-theme"),
            shadcn::SelectItem::new("reduced", "Reduced motion (0)")
                .test_id("ui-gallery-motion-preset-item-reduced"),
            shadcn::SelectItem::new("snappy", "Snappy")
                .test_id("ui-gallery-motion-preset-item-snappy"),
            shadcn::SelectItem::new("bouncy", "Bouncy")
                .test_id("ui-gallery-motion-preset-item-bouncy"),
            shadcn::SelectItem::new("gentle", "Gentle")
                .test_id("ui-gallery-motion-preset-item-gentle"),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_px(Px(180.0))
                .max_w(Px(220.0))
                .flex_shrink(1.0),
        )
        .into_element(cx);

        let presets = ui::h_row(|_cx| [theme_select, motion_select])
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N3)
            .items_center()
            .into_element(cx);

        [left, presets]
    })
    .layout(LayoutRefinement::default().w_full())
    .gap(Space::N2)
    .items_start()
    .into_element(cx);

    let mut header_semantics_layout = fret_ui::element::LayoutStyle::default();
    header_semantics_layout.size.width = fret_ui::element::Length::Fill;
    let header = cx.semantics(
        fret_ui::element::SemanticsProps {
            layout: header_semantics_layout,
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::from("ui-gallery-content-header")),
            ..Default::default()
        },
        |_cx| [header_content],
    );

    let preview_panel_content = page_preview(cx, theme, selected, models);
    let mut preview_semantics_layout = fret_ui::element::LayoutStyle::default();
    preview_semantics_layout.size.width = fret_ui::element::Length::Fill;
    let preview_panel = cx.semantics(
        fret_ui::element::SemanticsProps {
            layout: preview_semantics_layout,
            role: fret_core::SemanticsRole::Group,
            test_id: Some(Arc::from("ui-gallery-page-preview")),
            ..Default::default()
        },
        |_cx| [preview_panel_content],
    );

    let content = if (bisect & BISECT_DISABLE_CONTENT_SCROLL) != 0 {
        // When content scroll is disabled, keep the header and page body in one static stack.
        ui::v_flex(|_cx| [header, preview_panel])
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N6)
            .into_element(cx)
            .test_id("ui-gallery-content-scroll")
    } else {
        // Keep the page header (theme/motion presets) pinned above the scroll viewport so scripts
        // can toggle themes without scrolling back to the top.
        //
        // Key the scroll area by the selected page so navigation resets scroll position.
        let scroll_body = cx.keyed(format!("ui_gallery.content_scroll_area.{selected}"), |cx| {
            // Provide an explicit per-page handle so scroll position cannot leak across navigation.
            // (We still key the subtree above to ensure the handle resets deterministically.)
            let scroll_handle =
                cx.slot_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
            let should_reset_scroll = cx.slot_state(
                || true,
                |reset| {
                    let out = *reset;
                    *reset = false;
                    out
                },
            );
            if should_reset_scroll {
                scroll_handle.scroll_to_offset(fret_core::Point::new(
                    fret_core::Px(0.0),
                    fret_core::Px(0.0),
                ));
            }
            #[cfg(feature = "gallery-dev")]
            let mut scroll = shadcn::ScrollArea::new([preview_panel])
                .scroll_handle(scroll_handle.clone())
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .flex_grow(1.0)
                        .min_h_0()
                        .min_w_0(),
                )
                .viewport_intrinsic_measure_mode(
                    fret_ui::element::ScrollIntrinsicMeasureMode::Viewport,
                );
            #[cfg(not(feature = "gallery-dev"))]
            let scroll = shadcn::ScrollArea::new([preview_panel])
                .scroll_handle(scroll_handle)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_full()
                        .flex_1()
                        .min_w_0()
                        .min_h_0(),
                )
                .viewport_test_id("ui-gallery-content-viewport")
                .viewport_intrinsic_measure_mode(
                    fret_ui::element::ScrollIntrinsicMeasureMode::Viewport,
                );
            #[cfg(feature = "gallery-dev")]
            if selected == PAGE_VIRTUAL_LIST_TORTURE {
                scroll =
                    scroll.viewport_test_id("ui-gallery-content-viewport-virtual_list_torture");
                scroll = scroll.viewport_intrinsic_measure_mode(
                    fret_ui::element::ScrollIntrinsicMeasureMode::Viewport,
                );
            }
            scroll.into_element(cx).test_id("ui-gallery-content-scroll")
        });

        ui::v_flex(|_cx| [header, scroll_body])
            .layout(LayoutRefinement::default().w_full().h_full())
            .gap(Space::N6)
            .into_element(cx)
    };

    cx.named("ui_gallery.content_view_root", |cx| {
        let base_padding = fret_ui_kit::MetricRef::space(Space::N6).resolve(theme);
        let chrome = ChromeRefinement {
            padding: Some(
                fret_ui_kit::declarative::window_insets_padding_refinement_or_zero(
                    cx,
                    fret_ui::Invalidation::Layout,
                    base_padding,
                ),
            ),
            background: Some(ColorRef::Color(theme.color_token("background"))),
            ..ChromeRefinement::default()
        };
        let page_root_content = cx.container(
            decl_style::container_props(
                theme,
                chrome,
                LayoutRefinement::default().w_full().h_full(),
            ),
            |_cx| [content],
        );

        let mut semantics_fill_layout = fret_ui::element::LayoutStyle::default();
        semantics_fill_layout.size.width = fret_ui::element::Length::Fill;
        semantics_fill_layout.size.height = fret_ui::element::Length::Fill;

        let page_root = cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: semantics_fill_layout,
                role: fret_core::SemanticsRole::Group,
                test_id: Some(page_test_id),
                ..Default::default()
            },
            |_cx| [page_root_content],
        );

        cx.semantics(
            fret_ui::element::SemanticsProps {
                layout: semantics_fill_layout,
                role: fret_core::SemanticsRole::Group,
                test_id: Some(Arc::from("ui-gallery-content-shell")),
                ..Default::default()
            },
            |_cx| [page_root],
        )
    })
}

fn page_preview(
    cx: &mut UiCx<'_>,
    theme: &Theme,
    selected: &str,
    models: &UiGalleryModels,
) -> AnyElement {
    let motion_preset = models.motion_preset.clone();
    let motion_preset_open = models.motion_preset_open.clone();
    let view_cache_enabled = models.view_cache_enabled.clone();
    let view_cache_cache_shell = models.view_cache_cache_shell.clone();
    let view_cache_cache_content = models.view_cache_cache_content.clone();
    let view_cache_inner_enabled = models.view_cache_inner_enabled.clone();
    let view_cache_popover_open = models.view_cache_popover_open.clone();
    let view_cache_continuous = models.view_cache_continuous.clone();
    let view_cache_counter = models.view_cache_counter.clone();
    #[cfg(feature = "gallery-dev")]
    let popover_open = models.popover_open.clone();
    #[cfg(feature = "gallery-dev")]
    let dialog_open = models.dialog_open.clone();
    #[cfg(feature = "gallery-dev")]
    let dialog_glass_open = models.dialog_glass_open.clone();
    #[cfg(feature = "gallery-dev")]
    let alert_dialog_open = models.alert_dialog_open.clone();
    #[cfg(any(feature = "gallery-dev", feature = "gallery-material3"))]
    let sheet_open = models.sheet_open.clone();
    #[cfg(feature = "gallery-dev")]
    let portal_geometry_popover_open = models.portal_geometry_popover_open.clone();
    let combobox_value = models.combobox_value.clone();
    let combobox_open = models.combobox_open.clone();
    let combobox_query = models.combobox_query.clone();
    let _date_picker_open = models.date_picker_open.clone();
    let date_picker_month = models.date_picker_month.clone();
    let date_picker_selected = models.date_picker_selected.clone();
    #[cfg(feature = "gallery-dev")]
    let data_grid_selected_row = models.data_grid_selected_row.clone();
    let _tabs_value = models.tabs_value.clone();
    let accordion_value = models.accordion_value.clone();
    let _progress = models.progress.clone();
    #[cfg(feature = "gallery-dev")]
    let checkbox = models.checkbox.clone();
    #[cfg(feature = "gallery-dev")]
    let switch = models.switch.clone();
    #[cfg(feature = "gallery-material3")]
    let material3_expressive = models.material3_expressive.clone();
    let text_input = models.text_input.clone();
    let text_area = models.text_area.clone();
    let _input_file_value = models.input_file_value.clone();
    #[cfg(feature = "gallery-dev")]
    let dropdown_open = models.dropdown_open.clone();
    #[cfg(feature = "gallery-dev")]
    let context_menu_open = models.context_menu_open.clone();
    #[cfg(feature = "gallery-dev")]
    let context_menu_edge_open = models.context_menu_edge_open.clone();
    let _cmdk_open = models.cmdk_open.clone();
    let _cmdk_query = models.cmdk_query.clone();
    #[allow(unused_variables)]
    let last_action = models.last_action.clone();
    #[cfg(feature = "gallery-dev")]
    let virtual_list_torture_jump = models.virtual_list_torture_jump.clone();
    #[cfg(feature = "gallery-dev")]
    let virtual_list_torture_edit_row = models.virtual_list_torture_edit_row.clone();
    #[cfg(feature = "gallery-dev")]
    let virtual_list_torture_edit_text = models.virtual_list_torture_edit_text.clone();
    #[cfg(feature = "gallery-dev")]
    let virtual_list_torture_scroll = models.virtual_list_torture_scroll.clone();
    #[cfg(feature = "gallery-dev")]
    let code_editor_syntax_rust = models.code_editor_syntax_rust.clone();
    #[cfg(feature = "gallery-dev")]
    let code_editor_boundary_identifier = models.code_editor_boundary_identifier.clone();
    #[cfg(feature = "gallery-dev")]
    let code_editor_soft_wrap = models.code_editor_soft_wrap.clone();
    #[cfg(feature = "gallery-dev")]
    let code_editor_folds = models.code_editor_folds.clone();
    #[cfg(feature = "gallery-dev")]
    let code_editor_inlays = models.code_editor_inlays.clone();
    #[cfg(feature = "gallery-dev")]
    let markdown_link_gate_last_activation = models.markdown_link_gate_last_activation.clone();

    let body: Vec<AnyElement> = match selected {
        PAGE_LAYOUT => preview_layout(cx, theme),
        PAGE_MOTION_PRESETS => preview_motion_presets(cx, motion_preset, motion_preset_open),
        PAGE_VIEW_CACHE => preview_view_cache(
            cx,
            theme,
            view_cache_enabled,
            view_cache_cache_shell,
            view_cache_cache_content,
            view_cache_inner_enabled,
            view_cache_popover_open,
            view_cache_continuous,
            view_cache_counter,
            text_input,
            text_area,
        ),
        #[cfg(feature = "gallery-dev")]
        PAGE_HIT_TEST_ONLY_PAINT_CACHE_PROBE => preview_hit_test_only_paint_cache_probe(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_VIRTUAL_LIST_TORTURE => preview_virtual_list_torture(
            cx,
            theme,
            virtual_list_torture_jump,
            virtual_list_torture_edit_row,
            virtual_list_torture_edit_text,
            virtual_list_torture_scroll,
        ),
        #[cfg(feature = "gallery-dev")]
        PAGE_UI_KIT_LIST_TORTURE => preview_ui_kit_list_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_CODE_VIEW_TORTURE => preview_code_view_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_CODE_EDITOR_MVP => preview_code_editor_mvp(
            cx,
            theme,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
            code_editor_soft_wrap,
        ),
        #[cfg(feature = "gallery-dev")]
        PAGE_CODE_EDITOR_TORTURE => preview_code_editor_torture(
            cx,
            theme,
            code_editor_syntax_rust,
            code_editor_boundary_identifier,
            code_editor_soft_wrap,
            code_editor_folds,
            code_editor_inlays,
        ),
        #[cfg(feature = "gallery-dev")]
        PAGE_MARKDOWN_EDITOR_SOURCE => preview_markdown_editor_source(
            cx,
            theme,
            code_editor_soft_wrap,
            code_editor_folds,
            code_editor_inlays,
            markdown_link_gate_last_activation,
        ),
        #[cfg(feature = "gallery-dev")]
        PAGE_TEXT_SELECTION_PERF => preview_text_selection_perf(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_TEXT_BIDI_RTL_CONFORMANCE => preview_text_bidi_rtl_conformance(cx, theme),
        #[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
        PAGE_TEXT_MIXED_SCRIPT_FALLBACK => preview_text_mixed_script_fallback(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_TEXT_MEASURE_OVERLAY => preview_text_measure_overlay(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_TEXT_FEATURE_TOGGLES => preview_text_feature_toggles(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_TEXT_OUTLINE_STROKE => preview_text_outline_stroke(cx, theme),
        #[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
        PAGE_WEB_IME_HARNESS => preview_web_ime_harness(cx, theme, text_input, text_area),
        #[cfg(feature = "gallery-dev")]
        PAGE_CHART_TORTURE => preview_chart_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_CANVAS_CULL_TORTURE => preview_canvas_cull_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_NODE_GRAPH_CULL_TORTURE => preview_node_graph_cull_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_CHROME_TORTURE => preview_chrome_torture(
            cx,
            theme,
            popover_open,
            dialog_open,
            dialog_glass_open,
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
        #[cfg(feature = "gallery-dev")]
        PAGE_WINDOWED_ROWS_SURFACE_TORTURE => preview_windowed_rows_surface_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE => {
            preview_windowed_rows_surface_interactive_torture(cx, theme)
        }
        #[cfg(feature = "gallery-dev")]
        PAGE_DATA_TABLE_TORTURE => preview_data_table_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_TREE_TORTURE => preview_tree_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_TABLE_RETAINED_TORTURE => preview_table_retained_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_INSPECTOR_TORTURE => preview_inspector_torture(cx, theme),
        #[cfg(feature = "gallery-dev")]
        PAGE_FILE_TREE_TORTURE => preview_file_tree_torture(cx, theme),
        PAGE_BUTTON => pages::preview_button(cx),
        PAGE_CARD => pages::preview_card(cx),
        PAGE_BADGE => pages::preview_badge(cx),
        PAGE_AVATAR => pages::preview_avatar(cx),
        PAGE_IMAGE_OBJECT_FIT => pages::preview_image_object_fit(cx),
        PAGE_SKELETON => pages::preview_skeleton(cx),
        PAGE_SCROLL_AREA => pages::preview_scroll_area(cx),
        PAGE_TOOLTIP => pages::preview_tooltip(cx),
        PAGE_SLIDER => pages::preview_slider(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_ICONS => pages::preview_icons(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_LENS => preview_magic_lens(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_MARQUEE => preview_magic_marquee(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_CARD => preview_magic_card(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_BORDER_BEAM => preview_magic_border_beam(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_DOCK => preview_magic_dock(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_PATTERNS => preview_magic_patterns(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_PATTERNS_TORTURE => preview_magic_patterns_torture(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_SPARKLES_TEXT => preview_magic_sparkles_text(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MAGIC_BLOOM => preview_magic_bloom(cx),
        PAGE_FIELD => pages::preview_field(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_OVERLAY => preview_overlay(
            cx,
            popover_open,
            dialog_open,
            dialog_glass_open,
            alert_dialog_open,
            sheet_open,
            portal_geometry_popover_open,
            dropdown_open,
            context_menu_open,
            context_menu_edge_open,
            last_action.clone(),
        ),
        #[cfg(feature = "gallery-dev")]
        PAGE_SHADCN_EXTRAS => pages::preview_shadcn_extras(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_FORMS => pages::preview_forms(cx),
        PAGE_SELECT => pages::preview_select(cx),
        PAGE_COMBOBOX => pages::preview_combobox(cx, combobox_value, combobox_open, combobox_query),
        PAGE_DATE_PICKER => pages::preview_date_picker(cx),
        PAGE_RESIZABLE => pages::preview_resizable(cx),
        PAGE_DATA_TABLE => pages::preview_data_table(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_DATA_GRID => preview_data_grid(cx, data_grid_selected_row),
        PAGE_TABS => pages::preview_tabs(cx),
        PAGE_ACCORDION => pages::preview_accordion(cx, accordion_value),
        PAGE_TABLE => pages::preview_table(cx),
        PAGE_PROGRESS => pages::preview_progress(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_MENUS => super::previews::gallery::preview_menus(
            cx,
            dropdown_open,
            context_menu_open,
            last_action.clone(),
        ),
        PAGE_COMMAND => pages::preview_command_palette(cx),
        PAGE_TOAST => pages::preview_toast(cx),
        PAGE_SONNER => pages::preview_sonner(cx),
        PAGE_ALERT => pages::preview_alert(cx),
        PAGE_ALERT_DIALOG => pages::preview_alert_dialog(cx),
        PAGE_ASPECT_RATIO => pages::preview_aspect_ratio(cx),
        PAGE_BREADCRUMB => pages::preview_breadcrumb(cx),
        PAGE_BUTTON_GROUP => pages::preview_button_group(cx),
        PAGE_CALENDAR => pages::preview_calendar(cx, date_picker_month, date_picker_selected),
        PAGE_CAROUSEL => pages::preview_carousel(cx),
        #[cfg(feature = "gallery-dev")]
        PAGE_CHART => pages::preview_chart(cx),
        PAGE_CHECKBOX => pages::preview_checkbox(cx),
        PAGE_COLLAPSIBLE => pages::preview_collapsible(cx),
        PAGE_CONTEXT_MENU => pages::preview_context_menu(cx),
        PAGE_DIALOG => pages::preview_dialog(cx),
        PAGE_DRAWER => pages::preview_drawer(cx),
        PAGE_DROPDOWN_MENU => pages::preview_dropdown_menu(cx),
        PAGE_EMPTY => pages::preview_empty(cx),
        PAGE_FORM => pages::preview_forms(cx),
        PAGE_HOVER_CARD => pages::preview_hover_card(cx),
        PAGE_INPUT => pages::preview_input(cx),
        PAGE_INPUT_GROUP => pages::preview_input_group(cx),
        PAGE_INPUT_OTP => pages::preview_input_otp(cx),
        PAGE_ITEM => pages::preview_item(cx),
        PAGE_KBD => pages::preview_kbd(cx),
        PAGE_LABEL => pages::preview_label(cx),
        PAGE_MENUBAR => pages::preview_menubar(cx),
        PAGE_NATIVE_SELECT => pages::preview_native_select(cx),
        PAGE_NAVIGATION_MENU => pages::preview_navigation_menu(cx),
        PAGE_PAGINATION => pages::preview_pagination(cx),
        PAGE_POPOVER => pages::preview_popover(cx),
        PAGE_RADIO_GROUP => pages::preview_radio_group(cx),
        PAGE_SEPARATOR => pages::preview_separator(cx),
        PAGE_SHEET => pages::preview_sheet(cx),
        PAGE_SIDEBAR => pages::preview_sidebar(cx),
        PAGE_SPINNER => pages::preview_spinner(cx),
        PAGE_SWITCH => pages::preview_switch(cx),
        PAGE_TEXTAREA => pages::preview_textarea(cx),
        PAGE_TOGGLE => pages::preview_toggle(cx),
        PAGE_TOGGLE_GROUP => pages::preview_toggle_group(cx),
        PAGE_TYPOGRAPHY => pages::preview_typography(cx),
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_GALLERY => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_gallery(cx, last_action.clone())
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_STATE_MATRIX => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_state_matrix(cx, last_action.clone())
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_TOUCH_TARGETS => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_touch_targets(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_BUTTON => pages::material3::material3_scoped_page(
            cx,
            material3_expressive.clone(),
            pages::material3::preview_material3_button,
        ),
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_ICON_BUTTON => pages::material3::material3_scoped_page(
            cx,
            material3_expressive.clone(),
            pages::material3::preview_material3_icon_button,
        ),
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_CHECKBOX => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_checkbox(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_SWITCH => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_switch(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_SLIDER => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_slider(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_RADIO => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_radio(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_BADGE => pages::material3::material3_scoped_page(
            cx,
            material3_expressive.clone(),
            pages::material3::preview_material3_badge,
        ),
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_TOP_APP_BAR => pages::material3::material3_scoped_page(
            cx,
            material3_expressive.clone(),
            pages::material3::preview_material3_top_app_bar,
        ),
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_BOTTOM_SHEET => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_bottom_sheet(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_DATE_PICKER => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_date_picker(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_TIME_PICKER => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_time_picker(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_SEGMENTED_BUTTON => pages::material3::material3_scoped_page(
            cx,
            material3_expressive.clone(),
            pages::material3::preview_material3_segmented_button,
        ),
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_SELECT => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_select(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_AUTOCOMPLETE => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_autocomplete(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_TEXT_FIELD => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_text_field(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_TABS => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_tabs(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_LIST => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_list(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_NAVIGATION_BAR => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_navigation_bar(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_NAVIGATION_RAIL => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_navigation_rail(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_NAVIGATION_DRAWER => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_navigation_drawer(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_modal_navigation_drawer(cx)
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_DIALOG => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_dialog(cx, last_action.clone())
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_MENU => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_menu(cx, last_action.clone())
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_SNACKBAR => {
            pages::material3::material3_scoped_page(cx, material3_expressive.clone(), |cx| {
                pages::material3::preview_material3_snackbar(cx, last_action.clone())
            })
        }
        #[cfg(feature = "gallery-material3")]
        PAGE_MATERIAL3_TOOLTIP => pages::material3::material3_scoped_page(
            cx,
            material3_expressive.clone(),
            pages::material3::preview_material3_tooltip,
        ),
        other if other.starts_with("ai_") => {
            #[cfg(feature = "gallery-ai")]
            {
                pages::preview_ai_by_id(cx, theme, other)
                    .unwrap_or_else(|| preview_ai_unwired(cx, theme, other))
            }

            #[cfg(not(feature = "gallery-ai"))]
            {
                preview_ai_unwired(cx, theme, other)
            }
        }
        _ => preview_intro(cx, theme),
    };

    shadcn::Card::new(vec![
        shadcn::CardHeader::new(vec![
            shadcn::CardTitle::new("Preview").into_element(cx),
            shadcn::CardDescription::new("Interactive preview for validating behaviors.")
                .into_element(cx),
        ])
        .into_element(cx)
        .test_id("ui-gallery-preview-card-header"),
        shadcn::CardContent::new(body)
            .into_element(cx)
            .test_id("ui-gallery-preview-card-content"),
    ])
    .refine_layout(LayoutRefinement::default().w_full())
    .into_element(cx)
    .test_id("ui-gallery-preview-card")
}

fn preview_ai_unwired(cx: &mut UiCx<'_>, _theme: &Theme, id: &str) -> Vec<AnyElement> {
    vec![
        shadcn::Alert::new(vec![
            shadcn::AlertTitle::new("AI demo not wired").into_element(cx),
            shadcn::AlertDescription::new(format!(
                "Page `{id}` exists in the nav spec, but does not have a preview implementation yet. See `docs/workstreams/ai-elements-port/ai-elements-port-todo.md`."
            ))
            .into_element(cx),
        ])
        .variant(shadcn::AlertVariant::Default)
        .into_element(cx)
        .test_id(format!("ui-gallery-ai-unwired-{}", id.replace('_', "-"))),
    ]
}
