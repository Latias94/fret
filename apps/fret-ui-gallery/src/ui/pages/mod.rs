use super::*;

mod accordion;
mod ai_audio_player_demo;
mod ai_agent_demo;
mod ai_artifact_demo;
mod ai_attachments_demo;
mod ai_canvas_world_layer_spike;
mod ai_chain_of_thought_demo;
mod ai_checkpoint_demo;
mod ai_code_block_demo;
mod ai_commit_demo;
mod ai_commit_large_demo;
mod ai_confirmation_demo;
mod ai_context_demo;
mod ai_conversation_demo;
mod ai_file_tree_demo;
mod ai_environment_variables_demo;
mod ai_image_demo;
mod ai_inline_citation_demo;
mod ai_message_branch_demo;
mod ai_message_demo;
mod ai_mic_selector_demo;
mod ai_model_selector_demo;
mod ai_open_in_chat_demo;
mod ai_package_info_demo;
mod ai_persona_demo;
mod ai_plan_demo;
mod ai_prompt_input_action_menu_demo;
mod ai_prompt_input_provider_demo;
mod ai_prompt_input_referenced_sources_demo;
mod ai_queue_demo;
mod ai_reasoning_demo;
mod ai_schema_display_demo;
mod ai_shimmer_demo;
mod ai_snippet_demo;
mod ai_sources_demo;
mod ai_speech_input_demo;
mod ai_sandbox_demo;
mod ai_stack_trace_demo;
mod ai_stack_trace_large_demo;
mod ai_suggestions_demo;
mod ai_task_demo;
mod ai_terminal_demo;
mod ai_test_results_demo;
mod ai_test_results_large_demo;
mod ai_transcription_demo;
mod ai_transcript_torture;
mod ai_tool_demo;
mod ai_voice_selector_demo;
mod ai_web_preview_demo;
mod ai_workflow_connection_demo;
mod ai_workflow_controls_demo;
mod ai_workflow_canvas_demo;
mod ai_workflow_chrome_demo;
mod ai_workflow_edge_demo;
mod ai_workflow_node_demo;
mod ai_workflow_node_graph_demo;
mod ai_workflow_panel_demo;
mod ai_workflow_toolbar_demo;
mod alert;
mod alert_dialog;
mod aspect_ratio;
mod avatar;
mod badge;
mod breadcrumb;
mod button;
mod button_group;
mod calendar;
mod card;
mod carousel;
mod chart;
mod checkbox;
mod collapsible;
mod combobox;
mod command;
mod context_menu;
mod data_table;
mod date_picker;
mod dialog;
mod drawer;
mod dropdown_menu;
mod empty;
mod field;
mod form;
mod hover_card;
mod icons;
mod image_object_fit;
mod input;
mod input_group;
mod input_otp;
mod item;
mod kbd;
mod label;
mod menubar;
mod motion_presets;
mod native_select;
mod navigation_menu;
mod pagination;
mod popover;
mod progress;
mod radio_group;
mod resizable;
mod scroll_area;
mod select;
mod separator;
mod shadcn_extras;
mod sheet;
mod sidebar;
mod skeleton;
mod slider;
mod sonner;
mod spinner;
mod switch;
mod table;
mod tabs;
mod textarea;
mod toast;
mod toggle;
mod toggle_group;
mod tooltip;
mod typography;

pub(super) fn preview_alert(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    alert::preview_alert(cx)
}

pub(super) fn preview_ai_attachments_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_attachments_demo::preview_ai_attachments_demo(cx, theme)
}

pub(super) fn preview_ai_audio_player_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_audio_player_demo::preview_ai_audio_player_demo(cx, theme)
}

pub(super) fn preview_ai_agent_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_agent_demo::preview_ai_agent_demo(cx, theme)
}

pub(super) fn preview_ai_artifact_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_artifact_demo::preview_ai_artifact_demo(cx, theme)
}

pub(super) fn preview_ai_canvas_world_layer_spike(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_canvas_world_layer_spike::preview_ai_canvas_world_layer_spike(cx, theme)
}

pub(super) fn preview_ai_checkpoint_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_checkpoint_demo::preview_ai_checkpoint_demo(cx, theme)
}

pub(super) fn preview_ai_chain_of_thought_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_chain_of_thought_demo::preview_ai_chain_of_thought_demo(cx, theme)
}

pub(super) fn preview_ai_code_block_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_code_block_demo::preview_ai_code_block_demo(cx, theme)
}

pub(super) fn preview_ai_commit_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_commit_demo::preview_ai_commit_demo(cx, theme)
}

pub(super) fn preview_ai_commit_large_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_commit_large_demo::preview_ai_commit_large_demo(cx, theme)
}

pub(super) fn preview_ai_confirmation_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_confirmation_demo::preview_ai_confirmation_demo(cx, theme)
}

pub(super) fn preview_ai_conversation_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_conversation_demo::preview_ai_conversation_demo(cx, theme)
}

pub(super) fn preview_ai_context_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_context_demo::preview_ai_context_demo(cx, theme)
}

pub(super) fn preview_ai_file_tree_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_file_tree_demo::preview_ai_file_tree_demo(cx, theme)
}

pub(super) fn preview_ai_image_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_image_demo::preview_ai_image_demo(cx, theme)
}

pub(super) fn preview_ai_snippet_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_snippet_demo::preview_ai_snippet_demo(cx, theme)
}

pub(super) fn preview_ai_environment_variables_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_environment_variables_demo::preview_ai_environment_variables_demo(cx, theme)
}

pub(super) fn preview_ai_message_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_message_demo::preview_ai_message_demo(cx, theme)
}

pub(super) fn preview_ai_message_branch_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_message_branch_demo::preview_ai_message_branch_demo(cx, theme)
}

pub(super) fn preview_ai_open_in_chat_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_open_in_chat_demo::preview_ai_open_in_chat_demo(cx, theme)
}

pub(super) fn preview_ai_persona_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_persona_demo::preview_ai_persona_demo(cx, theme)
}

pub(super) fn preview_ai_plan_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_plan_demo::preview_ai_plan_demo(cx, theme)
}

pub(super) fn preview_ai_schema_display_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_schema_display_demo::preview_ai_schema_display_demo(cx, theme)
}

pub(super) fn preview_ai_queue_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_queue_demo::preview_ai_queue_demo(cx, theme)
}

pub(super) fn preview_ai_terminal_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_terminal_demo::preview_ai_terminal_demo(cx, theme)
}

pub(super) fn preview_ai_stack_trace_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_stack_trace_demo::preview_ai_stack_trace_demo(cx, theme)
}

pub(super) fn preview_ai_stack_trace_large_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_stack_trace_large_demo::preview_ai_stack_trace_large_demo(cx, theme)
}

pub(super) fn preview_ai_tool_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_tool_demo::preview_ai_tool_demo(cx, theme)
}

pub(super) fn preview_ai_reasoning_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_reasoning_demo::preview_ai_reasoning_demo(cx, theme)
}

pub(super) fn preview_ai_shimmer_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_shimmer_demo::preview_ai_shimmer_demo(cx, theme)
}

pub(super) fn preview_ai_suggestions_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_suggestions_demo::preview_ai_suggestions_demo(cx, theme)
}

pub(super) fn preview_ai_task_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_task_demo::preview_ai_task_demo(cx, theme)
}

pub(super) fn preview_ai_test_results_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_test_results_demo::preview_ai_test_results_demo(cx, theme)
}

pub(super) fn preview_ai_test_results_large_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_test_results_large_demo::preview_ai_test_results_large_demo(cx, theme)
}

pub(super) fn preview_ai_transcription_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_transcription_demo::preview_ai_transcription_demo(cx, theme)
}

pub(super) fn preview_ai_transcript_torture(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_transcript_torture::preview_ai_transcript_torture(cx, theme)
}

pub(super) fn preview_ai_prompt_input_provider_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_prompt_input_provider_demo::preview_ai_prompt_input_provider_demo(cx, theme)
}

pub(super) fn preview_ai_prompt_input_action_menu_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_prompt_input_action_menu_demo::preview_ai_prompt_input_action_menu_demo(cx, theme)
}

pub(super) fn preview_ai_prompt_input_referenced_sources_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_prompt_input_referenced_sources_demo::preview_ai_prompt_input_referenced_sources_demo(
        cx, theme,
    )
}

pub(super) fn preview_ai_package_info_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_package_info_demo::preview_ai_package_info_demo(cx, theme)
}

pub(super) fn preview_ai_model_selector_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_model_selector_demo::preview_ai_model_selector_demo(cx, theme)
}

pub(super) fn preview_ai_mic_selector_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_mic_selector_demo::preview_ai_mic_selector_demo(cx, theme)
}

pub(super) fn preview_ai_voice_selector_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_voice_selector_demo::preview_ai_voice_selector_demo(cx, theme)
}

pub(super) fn preview_ai_inline_citation_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_inline_citation_demo::preview_ai_inline_citation_demo(cx, theme)
}

pub(super) fn preview_ai_sources_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_sources_demo::preview_ai_sources_demo(cx, theme)
}

pub(super) fn preview_ai_speech_input_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_speech_input_demo::preview_ai_speech_input_demo(cx, theme)
}

pub(super) fn preview_ai_sandbox_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_sandbox_demo::preview_ai_sandbox_demo(cx, theme)
}

pub(super) fn preview_ai_web_preview_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_web_preview_demo::preview_ai_web_preview_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_toolbar_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_toolbar_demo::preview_ai_workflow_toolbar_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_panel_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_panel_demo::preview_ai_workflow_panel_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_connection_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_connection_demo::preview_ai_workflow_connection_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_chrome_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_chrome_demo::preview_ai_workflow_chrome_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_controls_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_controls_demo::preview_ai_workflow_controls_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_canvas_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_canvas_demo::preview_ai_workflow_canvas_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_node_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_node_demo::preview_ai_workflow_node_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_edge_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_edge_demo::preview_ai_workflow_edge_demo(cx, theme)
}

pub(super) fn preview_ai_workflow_node_graph_demo(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
) -> Vec<AnyElement> {
    ai_workflow_node_graph_demo::preview_ai_workflow_node_graph_demo(cx, theme)
}

pub(super) fn preview_accordion(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    accordion::preview_accordion(cx, value)
}

pub(super) fn preview_avatar(
    cx: &mut ElementContext<'_, App>,
    avatar_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    avatar::preview_avatar(cx, avatar_image)
}

pub(super) fn preview_button(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    button::preview_button(cx)
}

pub(super) fn preview_button_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    button_group::preview_button_group(cx)
}

pub(super) fn preview_calendar(
    cx: &mut ElementContext<'_, App>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    calendar::preview_calendar(cx, month, selected)
}

pub(super) fn preview_alert_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    alert_dialog::preview_alert_dialog(cx, open)
}

pub(super) fn preview_aspect_ratio(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    aspect_ratio::preview_aspect_ratio(cx)
}

pub(super) fn preview_card(
    cx: &mut ElementContext<'_, App>,
    event_cover_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    card::preview_card(cx, event_cover_image)
}

pub(super) fn preview_icons(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    icons::preview_icons(cx)
}

pub(super) fn preview_image_object_fit(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    square_image: Model<Option<ImageId>>,
    wide_image: Model<Option<ImageId>>,
    tall_image: Model<Option<ImageId>>,
    streaming_image: Model<Option<ImageId>>,
) -> Vec<AnyElement> {
    image_object_fit::preview_image_object_fit(
        cx,
        theme,
        square_image,
        wide_image,
        tall_image,
        streaming_image,
    )
}

pub(super) fn preview_badge(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    badge::preview_badge(cx)
}

pub(super) fn preview_breadcrumb(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    breadcrumb::preview_breadcrumb(cx, last_action)
}

pub(super) fn preview_checkbox(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    checkbox::preview_checkbox(cx, model)
}

pub(super) fn preview_carousel(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    carousel::preview_carousel(cx)
}

pub(super) fn preview_chart(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    chart::preview_chart(cx)
}

pub(super) fn preview_collapsible(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    collapsible::preview_collapsible(cx)
}

pub(super) fn preview_combobox(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    combobox::preview_combobox(cx, value, open, query)
}

pub(super) fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    command::preview_command_palette(cx, open, query, last_action)
}

pub(super) fn preview_toast(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    toast::preview_toast(cx, last_action)
}

pub(super) fn preview_context_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    context_menu::preview_context_menu(cx, open, last_action)
}

pub(super) fn preview_data_table(
    cx: &mut ElementContext<'_, App>,
    state: Model<fret_ui_headless::table::TableState>,
) -> Vec<AnyElement> {
    data_table::preview_data_table(cx, state)
}
pub(super) fn preview_date_picker(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    month: Model<fret_ui_headless::calendar::CalendarMonth>,
    selected: Model<Option<Date>>,
) -> Vec<AnyElement> {
    date_picker::preview_date_picker(cx, open, month, selected)
}

pub(super) fn preview_dialog(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    dialog::preview_dialog(cx, open)
}

pub(super) fn preview_popover(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    popover::preview_popover(cx, open)
}

pub(super) fn preview_progress(
    cx: &mut ElementContext<'_, App>,
    progress: Model<f32>,
) -> Vec<AnyElement> {
    progress::preview_progress(cx, progress)
}

pub(super) fn preview_sheet(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
) -> Vec<AnyElement> {
    sheet::preview_sheet(cx, open)
}

pub(super) fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    field::preview_field(cx)
}

pub(super) fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    form::preview_forms(cx, text_input, text_area, checkbox, switch)
}

pub(super) fn preview_hover_card(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    hover_card::preview_hover_card(cx)
}

pub(super) fn preview_input(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
    file_value: Model<String>,
) -> Vec<AnyElement> {
    input::preview_input(cx, value, file_value)
}

pub(super) fn preview_input_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    input_group::preview_input_group(cx)
}

pub(super) fn preview_input_otp(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    input_otp::preview_input_otp(cx)
}

pub(super) fn preview_item(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    item::preview_item(cx)
}

pub(super) fn preview_kbd(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    kbd::preview_kbd(cx)
}

pub(super) fn preview_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    select::preview_select(cx)
}

pub(super) fn preview_label(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    label::preview_label(cx)
}

pub(super) fn preview_menubar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    menubar::preview_menubar(cx)
}

pub(super) fn preview_native_select(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    native_select::preview_native_select(cx)
}

pub(super) fn preview_navigation_menu(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    navigation_menu::preview_navigation_menu(cx)
}

pub(super) fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    radio_group::preview_radio_group(cx)
}

pub(super) fn preview_resizable(
    cx: &mut ElementContext<'_, App>,
    h_fractions: Model<Vec<f32>>,
    v_fractions: Model<Vec<f32>>,
) -> Vec<AnyElement> {
    resizable::preview_resizable(cx, h_fractions, v_fractions)
}

pub(super) fn preview_pagination(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    pagination::preview_pagination(cx)
}

pub(super) fn preview_scroll_area(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    scroll_area::preview_scroll_area(cx)
}

pub(super) fn preview_separator(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    separator::preview_separator(cx)
}

pub(super) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    sidebar::preview_sidebar(cx)
}

pub(super) fn preview_motion_presets(
    cx: &mut ElementContext<'_, App>,
    theme: &Theme,
    motion_preset: Model<Option<Arc<str>>>,
    motion_preset_open: Model<bool>,
    dialog_open: Model<bool>,
) -> Vec<AnyElement> {
    motion_presets::preview_motion_presets(
        cx,
        theme,
        motion_preset,
        motion_preset_open,
        dialog_open,
    )
}

pub(super) fn preview_shadcn_extras(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    shadcn_extras::preview_shadcn_extras(cx)
}

pub(super) fn preview_skeleton(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    skeleton::preview_skeleton(cx)
}

pub(super) fn preview_slider(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    slider::preview_slider(cx)
}

pub(super) fn preview_sonner(
    cx: &mut ElementContext<'_, App>,
    last_action: Model<Arc<str>>,
    sonner_position: Model<shadcn::ToastPosition>,
) -> Vec<AnyElement> {
    sonner::preview_sonner(cx, last_action, sonner_position)
}

pub(super) fn preview_tabs(
    cx: &mut ElementContext<'_, App>,
    value: Model<Option<Arc<str>>>,
) -> Vec<AnyElement> {
    tabs::preview_tabs(cx, value)
}

pub(super) fn preview_table(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    table::preview_table(cx)
}

pub(super) fn preview_spinner(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    spinner::preview_spinner(cx)
}

pub(super) fn preview_switch(
    cx: &mut ElementContext<'_, App>,
    model: Model<bool>,
) -> Vec<AnyElement> {
    switch::preview_switch(cx, model)
}

pub(super) fn preview_textarea(
    cx: &mut ElementContext<'_, App>,
    value: Model<String>,
) -> Vec<AnyElement> {
    textarea::preview_textarea(cx, value)
}

pub(super) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    drawer::preview_drawer(cx)
}

pub(super) fn preview_dropdown_menu(
    cx: &mut ElementContext<'_, App>,
    open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    dropdown_menu::preview_dropdown_menu(cx, open, last_action)
}

pub(super) fn preview_empty(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    empty::preview_empty(cx)
}

pub(super) fn preview_toggle(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    toggle::preview_toggle(cx)
}

pub(super) fn preview_toggle_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    toggle_group::preview_toggle_group(cx)
}

pub(super) fn preview_tooltip(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    tooltip::preview_tooltip(cx)
}

pub(super) fn preview_typography(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    typography::preview_typography(cx)
}
