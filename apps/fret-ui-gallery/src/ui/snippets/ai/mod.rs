use fret::UiCx;
use fret_core::{ImageColorSpace, ImageId};
use fret_ui_assets::{ImageSource, ui::ImageSourceElementContextExt as _};
use std::sync::OnceLock;

pub(crate) fn shared_preview_image_id(cx: &mut UiCx<'_>) -> Option<ImageId> {
    cx.use_image_source_state(shared_preview_source()).image
}

fn shared_preview_source() -> &'static ImageSource {
    static SOURCE: OnceLock<ImageSource> = OnceLock::new();
    SOURCE.get_or_init(|| {
        ImageSource::rgba8(
            320,
            320,
            shared_preview_rgba8(320, 320),
            ImageColorSpace::Srgb,
        )
    })
}

fn shared_preview_rgba8(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
    let width_f = (width.saturating_sub(1)).max(1) as f32;
    let height_f = (height.saturating_sub(1)).max(1) as f32;

    for y in 0..height {
        for x in 0..width {
            let idx = ((y as usize) * (width as usize) + (x as usize)) * 4;
            let fx = x as f32 / width_f;
            let fy = y as f32 / height_f;
            let glow = (((fx * 6.0) + (fy * 4.0)).sin() * 0.5 + 0.5) * 26.0;

            out[idx] = (26.0 + 58.0 * (1.0 - fy) + 108.0 * fx + glow).min(255.0) as u8;
            out[idx + 1] = (34.0 + 68.0 * fy + 88.0 * (1.0 - fx) + glow * 0.65).min(255.0) as u8;
            out[idx + 2] = (58.0 + 126.0 * (1.0 - fy) + 72.0 * fx + glow * 0.45).min(255.0) as u8;
            out[idx + 3] = 255;
        }
    }

    out
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
pub mod prompt_input_docs_demo;
pub mod prompt_input_tooltip_demo;
pub mod reasoning_demo;
pub mod reasoning_hooks;
pub mod schema_display_demo;
pub mod shimmer_demo;
pub mod shimmer_duration_demo;
pub mod shimmer_elements_demo;
pub mod shimmer_typography_demo;
pub mod snippet_demo;
pub mod snippet_plain;
pub mod stack_trace_collapsed;
pub mod stack_trace_demo;
pub mod stack_trace_no_internal;
pub mod suggestions_demo;
pub mod task_demo;
pub mod terminal_demo;
pub mod test_results_basic;
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
