use super::*;

pub(crate) fn content_view(
    cx: &mut ElementContext<'_, App>,
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
    let header = stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full())
            .gap(Space::N2)
            .items_start(),
        |cx| {
            let left = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
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
                },
            );

            let theme_select = shadcn::Select::new(
                models.theme_preset.clone(),
                models.theme_preset_open.clone(),
            )
            .placeholder("Theme preset")
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
            .placeholder("Motion preset")
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

            let presets = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N3)
                    .items_center(),
                |_cx| [theme_select, motion_select],
            );
            let right = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N2)
                    .items_start(),
                |_cx| [presets],
            );

            vec![left, right]
        },
    );

    let preview_panel = page_preview(cx, theme, selected, models);

    let content = if (bisect & BISECT_DISABLE_CONTENT_SCROLL) != 0 {
        // When content scroll is disabled, keep the header and page body in one static stack.
        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .gap(Space::N6),
            |_cx| [header, preview_panel],
        )
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
                cx.with_state(fret_ui::scroll::ScrollHandle::default, |h| h.clone());
            let should_reset_scroll = cx.with_state(
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
            let mut scroll = shadcn::ScrollArea::new([preview_panel])
                .scroll_handle(scroll_handle)
                .refine_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .h_full()
                        .min_w_0()
                        .min_h_0(),
                )
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
            scroll.into_element(cx).test_id("ui-gallery-content-scroll")
        });

        let scroll = cx.container(
            decl_style::container_props(
                theme,
                ChromeRefinement::default(),
                LayoutRefinement::default()
                    .w_full()
                    .flex_1()
                    .min_h_0()
                    .min_w_0(),
            ),
            move |_cx| [scroll_body],
        );

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().h_full())
                .gap(Space::N6),
            |_cx| [header, scroll],
        )
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
        cx.container(
            decl_style::container_props(
                theme,
                chrome,
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
    let popover_open = models.popover_open.clone();
    let dialog_open = models.dialog_open.clone();
    let dialog_glass_open = models.dialog_glass_open.clone();
    let alert_dialog_open = models.alert_dialog_open.clone();
    let sheet_open = models.sheet_open.clone();
    let portal_geometry_popover_open = models.portal_geometry_popover_open.clone();
    let combobox_value = models.combobox_value.clone();
    let combobox_open = models.combobox_open.clone();
    let combobox_query = models.combobox_query.clone();
    let date_picker_open = models.date_picker_open.clone();
    let date_picker_month = models.date_picker_month.clone();
    let date_picker_selected = models.date_picker_selected.clone();
    let time_picker_open = models.time_picker_open.clone();
    let time_picker_selected = models.time_picker_selected.clone();
    let resizable_h_fractions = models.resizable_h_fractions.clone();
    let resizable_v_fractions = models.resizable_v_fractions.clone();
    let data_table_state = models.data_table_state.clone();
    let data_grid_selected_row = models.data_grid_selected_row.clone();
    let tabs_value = models.tabs_value.clone();
    let accordion_value = models.accordion_value.clone();
    let avatar_demo_image = models.avatar_demo_image.clone();
    let image_fit_demo_wide_image = models.image_fit_demo_wide_image.clone();
    let image_fit_demo_tall_image = models.image_fit_demo_tall_image.clone();
    let image_fit_demo_streaming_image = models.image_fit_demo_streaming_image.clone();
    let progress = models.progress.clone();
    let checkbox = models.checkbox.clone();
    let switch = models.switch.clone();
    let material3_checkbox = models.material3_checkbox.clone();
    let material3_switch = models.material3_switch.clone();
    let material3_slider_value = models.material3_slider_value.clone();
    let material3_radio_value = models.material3_radio_value.clone();
    let material3_tabs_value = models.material3_tabs_value.clone();
    let material3_list_value = models.material3_list_value.clone();
    let material3_expressive = models.material3_expressive.clone();
    let material3_navigation_bar_value = models.material3_navigation_bar_value.clone();
    let material3_navigation_rail_value = models.material3_navigation_rail_value.clone();
    let material3_navigation_drawer_value = models.material3_navigation_drawer_value.clone();
    let material3_modal_navigation_drawer_open =
        models.material3_modal_navigation_drawer_open.clone();
    let material3_dialog_open = models.material3_dialog_open.clone();
    let material3_text_field_value = models.material3_text_field_value.clone();
    let material3_text_field_disabled = models.material3_text_field_disabled.clone();
    let material3_text_field_error = models.material3_text_field_error.clone();
    let material3_autocomplete_value = models.material3_autocomplete_value.clone();
    let material3_autocomplete_disabled = models.material3_autocomplete_disabled.clone();
    let material3_autocomplete_error = models.material3_autocomplete_error.clone();
    let material3_autocomplete_dialog_open = models.material3_autocomplete_dialog_open.clone();
    let material3_menu_open = models.material3_menu_open.clone();
    let text_input = models.text_input.clone();
    let text_area = models.text_area.clone();
    let input_file_value = models.input_file_value.clone();
    let dropdown_open = models.dropdown_open.clone();
    let context_menu_open = models.context_menu_open.clone();
    let context_menu_edge_open = models.context_menu_edge_open.clone();
    let cmdk_open = models.cmdk_open.clone();
    let cmdk_query = models.cmdk_query.clone();
    let last_action = models.last_action.clone();
    let sonner_position = models.sonner_position.clone();
    let virtual_list_torture_jump = models.virtual_list_torture_jump.clone();
    let virtual_list_torture_edit_row = models.virtual_list_torture_edit_row.clone();
    let virtual_list_torture_edit_text = models.virtual_list_torture_edit_text.clone();
    let virtual_list_torture_scroll = models.virtual_list_torture_scroll.clone();
    let code_editor_syntax_rust = models.code_editor_syntax_rust.clone();
    let code_editor_boundary_identifier = models.code_editor_boundary_identifier.clone();
    let code_editor_soft_wrap = models.code_editor_soft_wrap.clone();
    let code_editor_folds = models.code_editor_folds.clone();
    let code_editor_inlays = models.code_editor_inlays.clone();
    let markdown_link_gate_last_activation = models.markdown_link_gate_last_activation.clone();

    let body: Vec<AnyElement> = match selected {
        PAGE_LAYOUT => preview_layout(cx, theme),
        PAGE_MOTION_PRESETS => {
            preview_motion_presets(cx, theme, motion_preset, motion_preset_open, dialog_open)
        }
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
            markdown_link_gate_last_activation,
        ),
        PAGE_TEXT_SELECTION_PERF => preview_text_selection_perf(cx, theme),
        PAGE_TEXT_BIDI_RTL_CONFORMANCE => preview_text_bidi_rtl_conformance(cx, theme),
        PAGE_TEXT_MIXED_SCRIPT_FALLBACK => preview_text_mixed_script_fallback(cx, theme),
        PAGE_TEXT_MEASURE_OVERLAY => preview_text_measure_overlay(cx, theme),
        PAGE_TEXT_FEATURE_TOGGLES => preview_text_feature_toggles(cx, theme),
        PAGE_TEXT_OUTLINE_STROKE => preview_text_outline_stroke(cx, theme),
        PAGE_WEB_IME_HARNESS => preview_web_ime_harness(cx, theme, text_input, text_area),
        PAGE_CHART_TORTURE => preview_chart_torture(cx, theme),
        PAGE_CANVAS_CULL_TORTURE => preview_canvas_cull_torture(cx, theme),
        PAGE_NODE_GRAPH_CULL_TORTURE => preview_node_graph_cull_torture(cx, theme),
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
        PAGE_WINDOWED_ROWS_SURFACE_TORTURE => preview_windowed_rows_surface_torture(cx, theme),
        PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE => {
            preview_windowed_rows_surface_interactive_torture(cx, theme)
        }
        PAGE_DATA_TABLE_TORTURE => preview_data_table_torture(cx, theme, data_table_state),
        PAGE_TREE_TORTURE => preview_tree_torture(cx, theme),
        PAGE_TABLE_RETAINED_TORTURE => preview_table_retained_torture(cx, theme),
        PAGE_AI_TRANSCRIPT_TORTURE => pages::preview_ai_transcript_torture(cx, theme),
        PAGE_AI_CHAT_DEMO => preview_ai_chat_demo(cx, theme),
        PAGE_AI_CONVERSATION_DEMO => pages::preview_ai_conversation_demo(cx, theme),
        PAGE_AI_MESSAGE_DEMO => pages::preview_ai_message_demo(cx, theme),
        PAGE_AI_CONTEXT_DEMO => pages::preview_ai_context_demo(cx, theme),
        PAGE_AI_TERMINAL_DEMO => pages::preview_ai_terminal_demo(cx, theme),
        PAGE_AI_PACKAGE_INFO_DEMO => pages::preview_ai_package_info_demo(cx, theme),
        PAGE_AI_FILE_TREE_DEMO => pages::preview_ai_file_tree_demo(cx, theme),
        PAGE_AI_TASK_DEMO => pages::preview_ai_task_demo(cx, theme),
        PAGE_AI_AUDIO_PLAYER_DEMO => pages::preview_ai_audio_player_demo(cx, theme),
        PAGE_AI_TRANSCRIPTION_DEMO => pages::preview_ai_transcription_demo(cx, theme),
        PAGE_AI_MIC_SELECTOR_DEMO => pages::preview_ai_mic_selector_demo(cx, theme),
        PAGE_AI_SPEECH_INPUT_DEMO => pages::preview_ai_speech_input_demo(cx, theme),
        PAGE_AI_VOICE_SELECTOR_DEMO => pages::preview_ai_voice_selector_demo(cx, theme),
        PAGE_AI_AGENT_DEMO => pages::preview_ai_agent_demo(cx, theme),
        PAGE_AI_SANDBOX_DEMO => pages::preview_ai_sandbox_demo(cx, theme),
        PAGE_AI_PERSONA_DEMO => pages::preview_ai_persona_demo(cx, theme),
        PAGE_AI_OPEN_IN_CHAT_DEMO => pages::preview_ai_open_in_chat_demo(cx, theme),
        PAGE_AI_WORKFLOW_CHROME_DEMO => pages::preview_ai_workflow_chrome_demo(cx, theme),
        PAGE_AI_WORKFLOW_CANVAS_DEMO => pages::preview_ai_workflow_canvas_demo(cx, theme),
        PAGE_AI_WORKFLOW_NODE_DEMO => pages::preview_ai_workflow_node_demo(cx, theme),
        PAGE_AI_WORKFLOW_EDGE_DEMO => pages::preview_ai_workflow_edge_demo(cx, theme),
        PAGE_AI_WORKFLOW_CONNECTION_DEMO => pages::preview_ai_workflow_connection_demo(cx, theme),
        PAGE_AI_WORKFLOW_CONTROLS_DEMO => pages::preview_ai_workflow_controls_demo(cx, theme),
        PAGE_AI_WORKFLOW_PANEL_DEMO => pages::preview_ai_workflow_panel_demo(cx, theme),
        PAGE_AI_WORKFLOW_TOOLBAR_DEMO => pages::preview_ai_workflow_toolbar_demo(cx, theme),
        PAGE_AI_WORKFLOW_NODE_GRAPH_DEMO => pages::preview_ai_workflow_node_graph_demo(cx, theme),
        PAGE_AI_CANVAS_WORLD_LAYER_SPIKE => pages::preview_ai_canvas_world_layer_spike(cx, theme),
        PAGE_AI_ARTIFACT_DEMO => pages::preview_ai_artifact_demo(cx, theme),
        PAGE_AI_ATTACHMENTS_DEMO => pages::preview_ai_attachments_demo(cx, theme),
        PAGE_AI_MESSAGE_BRANCH_DEMO => pages::preview_ai_message_branch_demo(cx, theme),
        PAGE_AI_MODEL_SELECTOR_DEMO => pages::preview_ai_model_selector_demo(cx, theme),
        PAGE_AI_CODE_BLOCK_DEMO => pages::preview_ai_code_block_demo(cx, theme),
        PAGE_AI_COMMIT_DEMO => pages::preview_ai_commit_demo(cx, theme),
        PAGE_AI_COMMIT_LARGE_DEMO => pages::preview_ai_commit_large_demo(cx, theme),
        PAGE_AI_STACK_TRACE_DEMO => pages::preview_ai_stack_trace_demo(cx, theme),
        PAGE_AI_STACK_TRACE_LARGE_DEMO => pages::preview_ai_stack_trace_large_demo(cx, theme),
        PAGE_AI_SCHEMA_DISPLAY_DEMO => pages::preview_ai_schema_display_demo(cx, theme),
        PAGE_AI_SHIMMER_DEMO => pages::preview_ai_shimmer_demo(cx, theme),
        PAGE_AI_SUGGESTIONS_DEMO => pages::preview_ai_suggestions_demo(cx, theme),
        PAGE_AI_REASONING_DEMO => pages::preview_ai_reasoning_demo(cx, theme),
        PAGE_AI_QUEUE_DEMO => pages::preview_ai_queue_demo(cx, theme),
        PAGE_AI_TEST_RESULTS_DEMO => pages::preview_ai_test_results_demo(cx, theme),
        PAGE_AI_TEST_RESULTS_LARGE_DEMO => pages::preview_ai_test_results_large_demo(cx, theme),
        PAGE_AI_CHECKPOINT_DEMO => pages::preview_ai_checkpoint_demo(cx, theme),
        PAGE_AI_CONFIRMATION_DEMO => pages::preview_ai_confirmation_demo(cx, theme),
        PAGE_AI_ENVIRONMENT_VARIABLES_DEMO => {
            pages::preview_ai_environment_variables_demo(cx, theme)
        }
        PAGE_AI_PLAN_DEMO => pages::preview_ai_plan_demo(cx, theme),
        PAGE_AI_TOOL_DEMO => pages::preview_ai_tool_demo(cx, theme),
        PAGE_AI_WEB_PREVIEW_DEMO => pages::preview_ai_web_preview_demo(cx, theme),
        PAGE_AI_PROMPT_INPUT_PROVIDER_DEMO => {
            pages::preview_ai_prompt_input_provider_demo(cx, theme)
        }
        PAGE_AI_PROMPT_INPUT_ACTION_MENU_DEMO => {
            pages::preview_ai_prompt_input_action_menu_demo(cx, theme)
        }
        PAGE_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO => {
            pages::preview_ai_prompt_input_referenced_sources_demo(cx, theme)
        }
        PAGE_AI_INLINE_CITATION_DEMO => pages::preview_ai_inline_citation_demo(cx, theme),
        PAGE_AI_SOURCES_DEMO => pages::preview_ai_sources_demo(cx, theme),
        PAGE_AI_CHAIN_OF_THOUGHT_DEMO => pages::preview_ai_chain_of_thought_demo(cx, theme),
        PAGE_AI_SNIPPET_DEMO => pages::preview_ai_snippet_demo(cx, theme),
        PAGE_AI_IMAGE_DEMO => preview_ai_image_demo(cx, theme),
        PAGE_INSPECTOR_TORTURE => preview_inspector_torture(cx, theme),
        PAGE_FILE_TREE_TORTURE => preview_file_tree_torture(cx, theme),
        PAGE_BUTTON => pages::preview_button(cx),
        PAGE_CARD => pages::preview_card(cx, image_fit_demo_wide_image),
        PAGE_BADGE => pages::preview_badge(cx),
        PAGE_AVATAR => pages::preview_avatar(cx, avatar_demo_image),
        PAGE_IMAGE_OBJECT_FIT => pages::preview_image_object_fit(
            cx,
            theme,
            avatar_demo_image,
            image_fit_demo_wide_image,
            image_fit_demo_tall_image,
            image_fit_demo_streaming_image,
        ),
        PAGE_SKELETON => pages::preview_skeleton(cx),
        PAGE_SCROLL_AREA => pages::preview_scroll_area(cx),
        PAGE_TOOLTIP => pages::preview_tooltip(cx),
        PAGE_SLIDER => pages::preview_slider(cx),
        PAGE_ICONS => pages::preview_icons(cx),
        PAGE_MAGIC_LENS => preview_magic_lens(cx),
        PAGE_MAGIC_MARQUEE => preview_magic_marquee(cx),
        PAGE_MAGIC_CARD => preview_magic_card(cx),
        PAGE_MAGIC_BORDER_BEAM => preview_magic_border_beam(cx),
        PAGE_MAGIC_DOCK => preview_magic_dock(cx),
        PAGE_MAGIC_PATTERNS => preview_magic_patterns(cx),
        PAGE_MAGIC_PATTERNS_TORTURE => preview_magic_patterns_torture(cx),
        PAGE_MAGIC_SPARKLES_TEXT => preview_magic_sparkles_text(cx),
        PAGE_MAGIC_BLOOM => preview_magic_bloom(cx),
        PAGE_FIELD => pages::preview_field(cx),
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
        PAGE_SHADCN_EXTRAS => pages::preview_shadcn_extras(cx),
        PAGE_FORMS => pages::preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_SELECT => pages::preview_select(cx),
        PAGE_COMBOBOX => pages::preview_combobox(cx, combobox_value, combobox_open, combobox_query),
        PAGE_DATE_PICKER => pages::preview_date_picker(
            cx,
            date_picker_open,
            date_picker_month,
            date_picker_selected,
        ),
        PAGE_RESIZABLE => {
            pages::preview_resizable(cx, resizable_h_fractions, resizable_v_fractions)
        }
        PAGE_DATA_TABLE => pages::preview_data_table(cx, data_table_state),
        PAGE_DATA_GRID => preview_data_grid(cx, data_grid_selected_row),
        PAGE_TABS => pages::preview_tabs(cx, tabs_value),
        PAGE_ACCORDION => pages::preview_accordion(cx, accordion_value),
        PAGE_TABLE => pages::preview_table(cx),
        PAGE_PROGRESS => pages::preview_progress(cx, progress),
        PAGE_MENUS => preview_menus(cx, dropdown_open, context_menu_open, last_action.clone()),
        PAGE_COMMAND => {
            pages::preview_command_palette(cx, cmdk_open, cmdk_query, last_action.clone())
        }
        PAGE_TOAST => pages::preview_toast(cx, last_action.clone()),
        PAGE_SONNER => pages::preview_sonner(cx, last_action.clone(), sonner_position.clone()),
        PAGE_ALERT => pages::preview_alert(cx),
        PAGE_ALERT_DIALOG => pages::preview_alert_dialog(cx, alert_dialog_open),
        PAGE_ASPECT_RATIO => pages::preview_aspect_ratio(cx),
        PAGE_BREADCRUMB => pages::preview_breadcrumb(cx, last_action.clone()),
        PAGE_BUTTON_GROUP => pages::preview_button_group(cx),
        PAGE_CALENDAR => pages::preview_calendar(cx, date_picker_month, date_picker_selected),
        PAGE_CAROUSEL => pages::preview_carousel(cx),
        PAGE_CHART => pages::preview_chart(cx),
        PAGE_CHECKBOX => pages::preview_checkbox(cx, checkbox),
        PAGE_COLLAPSIBLE => pages::preview_collapsible(cx),
        PAGE_CONTEXT_MENU => {
            pages::preview_context_menu(cx, context_menu_open, last_action.clone())
        }
        PAGE_DIALOG => pages::preview_dialog(cx, dialog_open),
        PAGE_DRAWER => pages::preview_drawer(cx),
        PAGE_DROPDOWN_MENU => pages::preview_dropdown_menu(cx, dropdown_open, last_action.clone()),
        PAGE_EMPTY => pages::preview_empty(cx),
        PAGE_FORM => pages::preview_forms(cx, text_input, text_area, checkbox, switch),
        PAGE_HOVER_CARD => pages::preview_hover_card(cx),
        PAGE_INPUT => pages::preview_input(cx, text_input, input_file_value),
        PAGE_INPUT_GROUP => pages::preview_input_group(cx),
        PAGE_INPUT_OTP => pages::preview_input_otp(cx),
        PAGE_ITEM => pages::preview_item(cx),
        PAGE_KBD => pages::preview_kbd(cx),
        PAGE_LABEL => pages::preview_label(cx),
        PAGE_MENUBAR => pages::preview_menubar(cx),
        PAGE_NATIVE_SELECT => pages::preview_native_select(cx),
        PAGE_NAVIGATION_MENU => pages::preview_navigation_menu(cx),
        PAGE_PAGINATION => pages::preview_pagination(cx),
        PAGE_POPOVER => pages::preview_popover(cx, popover_open),
        PAGE_RADIO_GROUP => pages::preview_radio_group(cx),
        PAGE_SEPARATOR => pages::preview_separator(cx),
        PAGE_SHEET => pages::preview_sheet(cx, sheet_open),
        PAGE_SIDEBAR => pages::preview_sidebar(cx),
        PAGE_SPINNER => pages::preview_spinner(cx),
        PAGE_SWITCH => pages::preview_switch(cx, switch),
        PAGE_TEXTAREA => pages::preview_textarea(cx, text_area),
        PAGE_TOGGLE => pages::preview_toggle(cx),
        PAGE_TOGGLE_GROUP => pages::preview_toggle_group(cx),
        PAGE_TYPOGRAPHY => pages::preview_typography(cx),
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
        PAGE_MATERIAL3_SLIDER => material3_scoped_page(cx, material3_expressive.clone(), |cx| {
            preview_material3_slider(cx, material3_slider_value)
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
        other if other.starts_with("ai_") => preview_ai_unwired(cx, theme, other),
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

fn preview_ai_unwired(
    cx: &mut ElementContext<'_, App>,
    _theme: &Theme,
    id: &str,
) -> Vec<AnyElement> {
    vec![
        shadcn::Alert::new(vec![
            shadcn::AlertTitle::new("AI demo not wired").into_element(cx),
            shadcn::AlertDescription::new(format!(
                "Page `{id}` exists in the nav spec, but does not have a preview implementation yet. See `docs/workstreams/ai-elements-port-todo.md`."
            ))
            .into_element(cx),
        ])
        .variant(shadcn::AlertVariant::Default)
        .into_element(cx)
        .test_id(format!("ui-gallery-ai-unwired-{}", id.replace('_', "-"))),
    ]
}
