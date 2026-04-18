use fret::AppComponentCx;
use fret_core::ImageId;
use fret_ui_assets::ui::ImageSourceElementContextExt as _;

use crate::demo_assets;

pub(crate) fn shared_preview_image_id(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(
        &demo_assets::ui_gallery_shared_media_preview_request(),
    )
    .image
}

#[cfg(any(test, feature = "gallery-dev"))]
pub(crate) fn attachment_landscape_image_id(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(
        &demo_assets::ui_gallery_ai_attachment_landscape_request(),
    )
    .image
}

#[cfg(any(test, feature = "gallery-dev"))]
pub(crate) fn attachment_portrait_image_id(cx: &mut AppComponentCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state_from_asset_request(
        &demo_assets::ui_gallery_ai_attachment_portrait_request(),
    )
    .image
}

#[cfg(feature = "gallery-dev")]
pub mod artifact_code_display;
pub mod artifact_demo;
pub mod chain_of_thought_composable;
pub mod chain_of_thought_demo;
pub mod code_block_demo;
pub mod code_block_usage;
pub mod commit_custom_children;
pub mod commit_demo;
pub mod confirmation_accepted;
pub mod confirmation_demo;
pub mod confirmation_rejected;
pub mod confirmation_request;
pub mod context_default;
pub mod context_demo;
pub mod conversation_demo;
pub mod environment_variables_custom_children;
pub mod environment_variables_demo;
pub mod message_demo;
pub mod message_usage;
pub mod open_in_chat_demo;
pub mod package_info_demo;
pub mod plan_demo;
pub mod prompt_input_cursor_demo;
pub mod prompt_input_docs_demo;
pub mod prompt_input_tooltip_demo;
pub mod reasoning_demo;
pub mod reasoning_hooks;
pub mod schema_display_basic;
pub mod schema_display_body;
pub mod schema_display_composable;
pub mod schema_display_demo;
pub mod schema_display_nested;
pub mod schema_display_params;
pub mod shimmer_demo;
pub mod shimmer_duration_demo;
pub mod shimmer_elements_demo;
pub mod shimmer_typography_demo;
pub mod snippet_composable;
pub mod snippet_demo;
pub mod snippet_plain;
pub mod stack_trace_collapsed;
pub mod stack_trace_demo;
pub mod stack_trace_no_internal;
pub mod stack_trace_usage;
pub mod suggestions_demo;
pub mod task_demo;
pub mod terminal_demo;
pub mod test_results_basic;
pub mod test_results_composable;
pub mod test_results_demo;
pub mod test_results_errors;
pub mod test_results_suites;

#[cfg(feature = "gallery-dev")]
pub mod agent_demo;
#[cfg(feature = "gallery-dev")]
pub mod attachments_empty;
#[cfg(feature = "gallery-dev")]
pub mod attachments_grid;
#[cfg(feature = "gallery-dev")]
pub mod attachments_inline;
#[cfg(feature = "gallery-dev")]
pub mod attachments_list;
#[cfg(feature = "gallery-dev")]
pub mod attachments_usage;
#[cfg(feature = "gallery-dev")]
pub mod audio_player_demo;
#[cfg(feature = "gallery-dev")]
pub mod audio_player_remote_demo;
#[cfg(feature = "gallery-dev")]
pub mod canvas_world_layer_spike;
#[cfg(feature = "gallery-dev")]
pub mod chat_demo;
#[cfg(feature = "gallery-dev")]
pub mod checkpoint_demo;
#[cfg(feature = "gallery-dev")]
pub mod commit_large_demo;
#[cfg(feature = "gallery-dev")]
pub mod file_tree_basic;
#[cfg(feature = "gallery-dev")]
pub mod file_tree_demo;
#[cfg(feature = "gallery-dev")]
pub mod file_tree_expanded;
#[cfg(feature = "gallery-dev")]
pub mod file_tree_large;
#[cfg(feature = "gallery-dev")]
pub mod file_tree_selection;
#[cfg(feature = "gallery-dev")]
pub mod image_demo;
#[cfg(feature = "gallery-dev")]
pub mod inline_citation_demo;
#[cfg(feature = "gallery-dev")]
pub mod message_branch_demo;
#[cfg(feature = "gallery-dev")]
pub mod mic_selector_demo;
#[cfg(feature = "gallery-dev")]
pub mod model_selector_demo;
#[cfg(feature = "gallery-dev")]
pub mod persona_basic;
#[cfg(feature = "gallery-dev")]
pub mod persona_custom_styling;
#[cfg(feature = "gallery-dev")]
pub mod persona_custom_visual;
#[cfg(feature = "gallery-dev")]
pub mod persona_demo;
#[cfg(feature = "gallery-dev")]
pub mod persona_state_management;
#[cfg(feature = "gallery-dev")]
pub mod persona_variants;
#[cfg(feature = "gallery-dev")]
pub mod prompt_input_action_menu_demo;
#[cfg(feature = "gallery-dev")]
pub mod prompt_input_provider_demo;
#[cfg(feature = "gallery-dev")]
pub mod prompt_input_referenced_sources_demo;
#[cfg(feature = "gallery-dev")]
pub mod queue_demo;
#[cfg(feature = "gallery-dev")]
pub mod queue_prompt_input_demo;
#[cfg(feature = "gallery-dev")]
pub mod sandbox_demo;
#[cfg(feature = "gallery-dev")]
pub mod sources_custom_demo;
#[cfg(feature = "gallery-dev")]
pub mod sources_demo;
#[cfg(feature = "gallery-dev")]
pub mod speech_input_demo;
#[cfg(feature = "gallery-dev")]
pub mod stack_trace_large_demo;
#[cfg(feature = "gallery-dev")]
pub mod test_results_large_demo;
#[cfg(feature = "gallery-dev")]
pub mod tool_demo;
#[cfg(feature = "gallery-dev")]
pub mod transcript_torture;
#[cfg(feature = "gallery-dev")]
pub mod transcription_demo;
#[cfg(feature = "gallery-dev")]
pub mod voice_selector_demo;
#[cfg(feature = "gallery-dev")]
pub mod web_preview_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_canvas_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_chrome_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_connection_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_controls_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_edge_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_node_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_node_graph_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_panel_demo;
#[cfg(feature = "gallery-dev")]
pub mod workflow_toolbar_demo;

#[cfg(all(test, feature = "gallery-dev"))]
mod tests {
    const MODULE_SOURCE: &str = include_str!("mod.rs");
    const ATTACHMENTS_USAGE_SOURCE: &str = include_str!("attachments_usage.rs");
    const ATTACHMENTS_GRID_SOURCE: &str = include_str!("attachments_grid.rs");
    const ATTACHMENTS_INLINE_SOURCE: &str = include_str!("attachments_inline.rs");
    const ATTACHMENTS_LIST_SOURCE: &str = include_str!("attachments_list.rs");
    const PROMPT_INPUT_DOCS_DEMO_SOURCE: &str = include_str!("prompt_input_docs_demo.rs");

    #[test]
    fn ai_gallery_preview_helpers_resolve_gallery_demo_assets_via_asset_requests() {
        let helper_source = MODULE_SOURCE
            .split("#[cfg(all(test, feature = \"gallery-dev\"))]")
            .next()
            .expect("ai snippet module keeps helper section before tests");
        assert!(helper_source.contains("use_image_source_state_from_asset_request"));
        assert!(helper_source.contains("ui_gallery_shared_media_preview_request"));
        assert!(helper_source.contains("ui_gallery_ai_attachment_landscape_request"));
        assert!(helper_source.contains("ui_gallery_ai_attachment_portrait_request"));
        assert!(!helper_source.contains("ImageSource::rgba8("));
    }

    #[test]
    fn ai_gallery_preview_snippets_do_not_synthesize_inline_rgba_demo_images() {
        assert!(ATTACHMENTS_USAGE_SOURCE.contains("attachment_landscape_image_id(cx)"));
        assert!(!ATTACHMENTS_USAGE_SOURCE.contains("ImageSource::rgba8("));

        assert!(ATTACHMENTS_GRID_SOURCE.contains("attachment_landscape_image_id(cx)"));
        assert!(ATTACHMENTS_GRID_SOURCE.contains("attachment_portrait_image_id(cx)"));
        assert!(!ATTACHMENTS_GRID_SOURCE.contains("ImageSource::rgba8("));

        assert!(ATTACHMENTS_INLINE_SOURCE.contains("attachment_landscape_image_id(cx)"));
        assert!(!ATTACHMENTS_INLINE_SOURCE.contains("ImageSource::rgba8("));

        assert!(ATTACHMENTS_LIST_SOURCE.contains("attachment_landscape_image_id(cx)"));
        assert!(!ATTACHMENTS_LIST_SOURCE.contains("ImageSource::rgba8("));

        assert!(PROMPT_INPUT_DOCS_DEMO_SOURCE.contains("shared_preview_image_id(cx)"));
        assert!(!PROMPT_INPUT_DOCS_DEMO_SOURCE.contains("ImageSource::rgba8("));
    }
}
