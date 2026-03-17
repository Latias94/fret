use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use fret::router::{RouteCodec, RouteLocation};
#[cfg(feature = "gallery-dev")]
use fret_runtime::CommandId;

pub(crate) const ENV_UI_GALLERY_BISECT: &str = "FRET_UI_GALLERY_BISECT";
pub(crate) const ENV_UI_GALLERY_START_PAGE: &str = "FRET_UI_GALLERY_START_PAGE";
pub(crate) const ENV_UI_GALLERY_NAV_QUERY: &str = "FRET_UI_GALLERY_NAV_QUERY";
pub(crate) const ENV_UI_GALLERY_DIAG_PROFILE: &str = "FRET_UI_GALLERY_DIAG_PROFILE";
pub(crate) const UI_GALLERY_DIAG_PROFILE_WORKSPACE_SHELL: &str = "workspace_shell";

pub(crate) const BISECT_MINIMAL_ROOT: u32 = 1 << 0;
pub(crate) const BISECT_DISABLE_OVERLAY_CONTROLLER: u32 = 1 << 1;
pub(crate) const BISECT_DISABLE_TOASTER: u32 = 1 << 2;
pub(crate) const BISECT_DISABLE_TAB_STRIP: u32 = 1 << 3;
pub(crate) const BISECT_SIMPLE_SIDEBAR: u32 = 1 << 4;
pub(crate) const BISECT_SIMPLE_CONTENT: u32 = 1 << 5;
pub(crate) const BISECT_DISABLE_SIDEBAR_SCROLL: u32 = 1 << 6;
pub(crate) const BISECT_DISABLE_CONTENT_SCROLL: u32 = 1 << 7;
pub(crate) const BISECT_DISABLE_CARD_SECTION_DEMO: u32 = 1 << 8;
pub(crate) const BISECT_DISABLE_CARD_SECTION_USAGE: u32 = 1 << 9;
pub(crate) const BISECT_DISABLE_CARD_SECTION_SIZE: u32 = 1 << 10;
pub(crate) const BISECT_DISABLE_CARD_SECTION_CARD_CONTENT: u32 = 1 << 11;
pub(crate) const BISECT_DISABLE_CARD_SECTION_MEETING_NOTES: u32 = 1 << 12;
pub(crate) const BISECT_DISABLE_CARD_SECTION_IMAGE: u32 = 1 << 13;
pub(crate) const BISECT_DISABLE_CARD_SECTION_RTL: u32 = 1 << 14;
pub(crate) const BISECT_DISABLE_CARD_SECTION_COMPOSITIONS: u32 = 1 << 15;
pub(crate) const BISECT_DISABLE_CARD_SECTION_NOTES: u32 = 1 << 16;
pub(crate) const BISECT_DISABLE_CARD_CODE_TABS: u32 = 1 << 17;
pub(crate) const BISECT_DISABLE_CARD_PAGE_INTRO: u32 = 1 << 18;

const UI_GALLERY_ROUTE_PATH: &str = "/gallery";
const UI_GALLERY_QUERY_PAGE: &str = "page";
const UI_GALLERY_QUERY_SOURCE: &str = "source";
const UI_GALLERY_QUERY_START_PAGE: &str = "start_page";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum UiGalleryAppRoute {
    Gallery { page: Arc<str> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UiGalleryRouteDecodeError {
    MissingPage,
    UnknownPage,
}

pub(crate) struct UiGalleryRouteCodec;

pub(crate) const UI_GALLERY_ROUTE_CODEC: UiGalleryRouteCodec = UiGalleryRouteCodec;

impl RouteCodec for UiGalleryRouteCodec {
    type Route = UiGalleryAppRoute;
    type Error = UiGalleryRouteDecodeError;

    fn encode(&self, route: &Self::Route) -> RouteLocation {
        match route {
            UiGalleryAppRoute::Gallery { page } => RouteLocation::from_path(UI_GALLERY_ROUTE_PATH)
                .with_query_value(UI_GALLERY_QUERY_PAGE, Some(page.to_string()))
                .with_query_value(UI_GALLERY_QUERY_SOURCE, Some("nav".to_string())),
        }
    }

    fn decode(&self, location: &RouteLocation) -> Result<Self::Route, Self::Error> {
        let page = location
            .query_value(UI_GALLERY_QUERY_PAGE)
            .or_else(|| location.query_value(UI_GALLERY_QUERY_START_PAGE))
            .ok_or(UiGalleryRouteDecodeError::MissingPage)?;
        let page =
            ui_gallery_start_page_from_id(page).ok_or(UiGalleryRouteDecodeError::UnknownPage)?;
        Ok(UiGalleryAppRoute::Gallery { page })
    }
}

fn ui_gallery_route_owns_query_key(key: &str) -> bool {
    matches!(
        key,
        UI_GALLERY_QUERY_PAGE | UI_GALLERY_QUERY_SOURCE | UI_GALLERY_QUERY_START_PAGE
    )
}

pub(crate) fn ui_gallery_page_from_route_location(location: &RouteLocation) -> Option<Arc<str>> {
    match UI_GALLERY_ROUTE_CODEC.decode_canonical(location).ok()? {
        UiGalleryAppRoute::Gallery { page } => Some(page),
    }
}

pub(crate) fn ui_gallery_route_location_for_page(
    from: &RouteLocation,
    page: &Arc<str>,
) -> RouteLocation {
    let mut location =
        UI_GALLERY_ROUTE_CODEC.encode_canonical(&UiGalleryAppRoute::Gallery { page: page.clone() });
    let from = from.canonicalized();
    location.query.extend(
        from.query
            .into_iter()
            .filter(|pair| !ui_gallery_route_owns_query_key(pair.key.as_str())),
    );
    location.canonicalize_query();
    location.fragment = from.fragment;
    location
}

pub(crate) fn ui_gallery_bisect_flags() -> u32 {
    static FLAGS: OnceLock<u32> = OnceLock::new();
    *FLAGS.get_or_init(|| {
        std::env::var(ENV_UI_GALLERY_BISECT)
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(0)
    })
}

pub(crate) fn ui_gallery_start_page() -> Option<Arc<str>> {
    #[cfg(target_arch = "wasm32")]
    {
        ui_gallery_start_page_from_url()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let id = std::env::var(ENV_UI_GALLERY_START_PAGE).ok()?;
        ui_gallery_start_page_from_id(&id)
    }
}

pub(crate) fn ui_gallery_diag_profile() -> Option<Arc<str>> {
    std::env::var(ENV_UI_GALLERY_DIAG_PROFILE)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(Arc::<str>::from)
}

fn ui_gallery_start_page_from_id(id: &str) -> Option<Arc<str>> {
    let id = id.trim();
    if id.is_empty() {
        return None;
    }

    if page_spec(id).is_some() {
        Some(Arc::<str>::from(id))
    } else {
        None
    }
}

#[cfg(any(target_arch = "wasm32", test))]
fn ui_gallery_legacy_start_page_from_search_or_hash(search: &str, hash: &str) -> Option<Arc<str>> {
    let id = fret::router::core::first_query_value_from_search_or_hash(
        search,
        hash,
        UI_GALLERY_QUERY_PAGE,
    )
    .or_else(|| {
        fret::router::core::first_query_value_from_search_or_hash(
            search,
            hash,
            UI_GALLERY_QUERY_START_PAGE,
        )
    })?;
    ui_gallery_start_page_from_id(&id)
}

#[cfg(target_arch = "wasm32")]
fn ui_gallery_start_page_from_url() -> Option<Arc<str>> {
    if let Some(location) = fret::router::core::web::current_route_location() {
        if let Some(page) = ui_gallery_page_from_route_location(&location) {
            return Some(page);
        }
    }

    let location = fret::router::core::web::current_location()?;
    ui_gallery_legacy_start_page_from_search_or_hash(&location.search, &location.hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_gallery_route_codec_encodes_canonical_gallery_page_href() {
        let href = UI_GALLERY_ROUTE_CODEC.href_for(&UiGalleryAppRoute::Gallery {
            page: Arc::from(PAGE_BUTTON_GROUP),
        });

        assert_eq!(href, "/gallery?page=button_group&source=nav");
    }

    #[test]
    fn ui_gallery_page_from_route_location_accepts_legacy_query_aliases() {
        let page =
            ui_gallery_page_from_route_location(&RouteLocation::parse("/legacy?start_page=button"))
                .expect("legacy gallery page should decode");

        assert_eq!(page.as_ref(), PAGE_BUTTON);
    }

    #[test]
    fn ui_gallery_route_location_for_page_preserves_passthrough_query_and_fragment() {
        let from = RouteLocation::parse(
            "/legacy?source=legacy&ws=1&mode=dev&start_page=intro&page=dialog#sheet 1",
        );

        let next = ui_gallery_route_location_for_page(&from, &Arc::from(PAGE_BUTTON_GROUP));

        assert_eq!(
            next.to_url(),
            "/gallery?mode=dev&page=button_group&source=nav&ws=1#sheet%201"
        );
    }

    #[test]
    fn ui_gallery_legacy_start_page_from_search_or_hash_supports_hash_queries() {
        let page =
            ui_gallery_legacy_start_page_from_search_or_hash("", "#/gallery?page=button_group")
                .expect("hash route query should resolve a gallery page");

        assert_eq!(page.as_ref(), PAGE_BUTTON_GROUP);
    }
}

#[cfg(feature = "gallery-dev")]
pub(crate) const CMD_DATA_GRID_ROW_PREFIX: &str = "ui_gallery.data_grid.row.";
#[cfg(feature = "gallery-dev")]
pub(crate) const DATA_GRID_ROWS: usize = 200;

pub(crate) const PAGE_INTRO: &str = "intro";
pub(crate) const PAGE_LAYOUT: &str = "layout";
pub(crate) const PAGE_MOTION_PRESETS: &str = "motion_presets";
pub(crate) const PAGE_VIEW_CACHE: &str = "view_cache";
#[allow(dead_code)]
pub(crate) const PAGE_EFFECTS_BLUR_TORTURE: &str = "effects_blur_torture";
#[allow(dead_code)]
pub(crate) const PAGE_SVG_UPLOAD_TORTURE: &str = "svg_upload_torture";
#[allow(dead_code)]
pub(crate) const PAGE_SVG_SCROLL_TORTURE: &str = "svg_scroll_torture";
#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
mod gallery_dev_page_ids {
    pub(crate) const PAGE_HIT_TEST_TORTURE: &str = "hit_test_torture";
    pub(crate) const PAGE_HIT_TEST_ONLY_PAINT_CACHE_PROBE: &str = "hit_test_only_paint_cache_probe";
    pub(crate) const PAGE_VIRTUAL_LIST_TORTURE: &str = "virtual_list_torture";
    pub(crate) const PAGE_UI_KIT_LIST_TORTURE: &str = "ui_kit_list_torture";
    pub(crate) const PAGE_CODE_VIEW_TORTURE: &str = "code_view_torture";
    pub(crate) const PAGE_CODE_EDITOR_MVP: &str = "code_editor_mvp";
    pub(crate) const PAGE_CODE_EDITOR_TORTURE: &str = "code_editor_torture";
    pub(crate) const PAGE_MARKDOWN_EDITOR_SOURCE: &str = "markdown_editor_source";
    pub(crate) const PAGE_TEXT_SELECTION_PERF: &str = "text_selection_perf";
    pub(crate) const PAGE_TEXT_BIDI_RTL_CONFORMANCE: &str = "text_bidi_rtl_conformance";
    pub(crate) const PAGE_TEXT_MIXED_SCRIPT_FALLBACK: &str = "text_mixed_script_fallback";
    pub(crate) const PAGE_TEXT_MEASURE_OVERLAY: &str = "text_measure_overlay";
    pub(crate) const PAGE_TEXT_FEATURE_TOGGLES: &str = "text_feature_toggles";
    pub(crate) const PAGE_TEXT_OUTLINE_STROKE: &str = "text_outline_stroke";
    pub(crate) const PAGE_WEB_IME_HARNESS: &str = "web_ime_harness";
    pub(crate) const PAGE_CHART_TORTURE: &str = "chart_torture";
    pub(crate) const PAGE_CANVAS_CULL_TORTURE: &str = "canvas_cull_torture";
    pub(crate) const PAGE_NODE_GRAPH_CULL_TORTURE: &str = "node_graph_cull_torture";
    pub(crate) const PAGE_CHROME_TORTURE: &str = "chrome_torture";
    pub(crate) const PAGE_WINDOWED_ROWS_SURFACE_TORTURE: &str = "windowed_rows_surface_torture";
    pub(crate) const PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str =
        "windowed_rows_surface_interactive_torture";
    pub(crate) const PAGE_DATA_TABLE_TORTURE: &str = "data_table_torture";
    pub(crate) const PAGE_TREE_TORTURE: &str = "tree_torture";
    pub(crate) const PAGE_TABLE_RETAINED_TORTURE: &str = "table_retained_torture";
    pub(crate) const PAGE_AI_TRANSCRIPT_TORTURE: &str = "ai_transcript_torture";
    pub(crate) const PAGE_AI_CHAT_DEMO: &str = "ai_chat_demo";
    pub(crate) const PAGE_AI_AUDIO_PLAYER_DEMO: &str = "ai_audio_player_demo";
    pub(crate) const PAGE_AI_TRANSCRIPTION_DEMO: &str = "ai_transcription_demo";
    pub(crate) const PAGE_AI_SPEECH_INPUT_DEMO: &str = "ai_speech_input_demo";
    pub(crate) const PAGE_AI_MIC_SELECTOR_DEMO: &str = "ai_mic_selector_demo";
    pub(crate) const PAGE_AI_VOICE_SELECTOR_DEMO: &str = "ai_voice_selector_demo";
    pub(crate) const PAGE_AI_AGENT_DEMO: &str = "ai_agent_demo";
    pub(crate) const PAGE_AI_SANDBOX_DEMO: &str = "ai_sandbox_demo";
    pub(crate) const PAGE_AI_PERSONA_DEMO: &str = "ai_persona_demo";
    pub(crate) const PAGE_AI_WORKFLOW_CHROME_DEMO: &str = "ai_workflow_chrome_demo";
    pub(crate) const PAGE_AI_WORKFLOW_CANVAS_DEMO: &str = "ai_workflow_canvas_demo";
    pub(crate) const PAGE_AI_WORKFLOW_NODE_DEMO: &str = "ai_workflow_node_demo";
    pub(crate) const PAGE_AI_WORKFLOW_EDGE_DEMO: &str = "ai_workflow_edge_demo";
    pub(crate) const PAGE_AI_WORKFLOW_CONNECTION_DEMO: &str = "ai_workflow_connection_demo";
    pub(crate) const PAGE_AI_WORKFLOW_CONTROLS_DEMO: &str = "ai_workflow_controls_demo";
    pub(crate) const PAGE_AI_WORKFLOW_PANEL_DEMO: &str = "ai_workflow_panel_demo";
    pub(crate) const PAGE_AI_WORKFLOW_TOOLBAR_DEMO: &str = "ai_workflow_toolbar_demo";
    pub(crate) const PAGE_AI_WORKFLOW_NODE_GRAPH_DEMO: &str = "ai_workflow_node_graph_demo";
    pub(crate) const PAGE_AI_CANVAS_WORLD_LAYER_SPIKE: &str = "ai_canvas_world_layer_spike";
    pub(crate) const PAGE_AI_PROMPT_INPUT_PROVIDER_DEMO: &str = "ai_prompt_input_provider_demo";
    pub(crate) const PAGE_AI_PROMPT_INPUT_ACTION_MENU_DEMO: &str =
        "ai_prompt_input_action_menu_demo";
    pub(crate) const PAGE_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO: &str =
        "ai_prompt_input_referenced_sources_demo";
    pub(crate) const PAGE_AI_INLINE_CITATION_DEMO: &str = "ai_inline_citation_demo";
    pub(crate) const PAGE_AI_SOURCES_DEMO: &str = "ai_sources_demo";
    pub(crate) const PAGE_AI_QUEUE_DEMO: &str = "ai_queue_demo";
    pub(crate) const PAGE_AI_ATTACHMENTS_DEMO: &str = "ai_attachments_demo";
    pub(crate) const PAGE_AI_SUGGESTIONS_DEMO: &str = "ai_suggestions_demo";
    pub(crate) const PAGE_AI_MESSAGE_BRANCH_DEMO: &str = "ai_message_branch_demo";
    pub(crate) const PAGE_AI_FILE_TREE_DEMO: &str = "ai_file_tree_demo";
    pub(crate) const PAGE_AI_COMMIT_LARGE_DEMO: &str = "ai_commit_large_demo";
    pub(crate) const PAGE_AI_STACK_TRACE_LARGE_DEMO: &str = "ai_stack_trace_large_demo";
    pub(crate) const PAGE_AI_TEST_RESULTS_LARGE_DEMO: &str = "ai_test_results_large_demo";
    pub(crate) const PAGE_AI_CHECKPOINT_DEMO: &str = "ai_checkpoint_demo";
    pub(crate) const PAGE_AI_TOOL_DEMO: &str = "ai_tool_demo";
    pub(crate) const PAGE_AI_WEB_PREVIEW_DEMO: &str = "ai_web_preview_demo";
    pub(crate) const PAGE_AI_MODEL_SELECTOR_DEMO: &str = "ai_model_selector_demo";
    pub(crate) const PAGE_AI_IMAGE_DEMO: &str = "ai_image_demo";
    pub(crate) const PAGE_INSPECTOR_TORTURE: &str = "inspector_torture";
    pub(crate) const PAGE_FILE_TREE_TORTURE: &str = "file_tree_torture";
}
#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
pub(crate) use gallery_dev_page_ids::*;

#[cfg(feature = "gallery-ai")]
mod gallery_ai_page_ids {
    pub(crate) const PAGE_AI_CONVERSATION_DEMO: &str = "ai_conversation_demo";
    pub(crate) const PAGE_AI_MESSAGE_DEMO: &str = "ai_message_demo";
    pub(crate) const PAGE_AI_CONTEXT_DEMO: &str = "ai_context_demo";
    pub(crate) const PAGE_AI_TERMINAL_DEMO: &str = "ai_terminal_demo";
    pub(crate) const PAGE_AI_PACKAGE_INFO_DEMO: &str = "ai_package_info_demo";
    pub(crate) const PAGE_AI_OPEN_IN_CHAT_DEMO: &str = "ai_open_in_chat_demo";
    pub(crate) const PAGE_AI_TASK_DEMO: &str = "ai_task_demo";
    pub(crate) const PAGE_AI_PROMPT_INPUT_DOCS_DEMO: &str = "ai_prompt_input_docs_demo";
    pub(crate) const PAGE_AI_ARTIFACT_DEMO: &str = "ai_artifact_demo";
    pub(crate) const PAGE_AI_SHIMMER_DEMO: &str = "ai_shimmer_demo";
    pub(crate) const PAGE_AI_REASONING_DEMO: &str = "ai_reasoning_demo";
    pub(crate) const PAGE_AI_CODE_BLOCK_DEMO: &str = "ai_code_block_demo";
    pub(crate) const PAGE_AI_SNIPPET_DEMO: &str = "ai_snippet_demo";
    pub(crate) const PAGE_AI_COMMIT_DEMO: &str = "ai_commit_demo";
    pub(crate) const PAGE_AI_STACK_TRACE_DEMO: &str = "ai_stack_trace_demo";
    pub(crate) const PAGE_AI_SCHEMA_DISPLAY_DEMO: &str = "ai_schema_display_demo";
    pub(crate) const PAGE_AI_TEST_RESULTS_DEMO: &str = "ai_test_results_demo";
    pub(crate) const PAGE_AI_CONFIRMATION_DEMO: &str = "ai_confirmation_demo";
    pub(crate) const PAGE_AI_ENVIRONMENT_VARIABLES_DEMO: &str = "ai_environment_variables_demo";
    pub(crate) const PAGE_AI_PLAN_DEMO: &str = "ai_plan_demo";
    pub(crate) const PAGE_AI_CHAIN_OF_THOUGHT_DEMO: &str = "ai_chain_of_thought_demo";
}
#[cfg(feature = "gallery-ai")]
pub(crate) use gallery_ai_page_ids::*;
pub(crate) const PAGE_BUTTON: &str = "button";
pub(crate) const PAGE_CARD: &str = "card";
pub(crate) const PAGE_BADGE: &str = "badge";
pub(crate) const PAGE_AVATAR: &str = "avatar";
pub(crate) const PAGE_IMAGE_OBJECT_FIT: &str = "image_object_fit";
#[cfg(feature = "gallery-dev")]
mod gallery_dev_recipe_page_ids {
    pub(crate) const PAGE_MAGIC_LENS: &str = "magic_lens";
    pub(crate) const PAGE_MAGIC_MARQUEE: &str = "magic_marquee";
    pub(crate) const PAGE_MAGIC_CARD: &str = "magic_card";
    pub(crate) const PAGE_MAGIC_BORDER_BEAM: &str = "magic_border_beam";
    pub(crate) const PAGE_MAGIC_DOCK: &str = "magic_dock";
    pub(crate) const PAGE_MAGIC_PATTERNS: &str = "magic_patterns";
    pub(crate) const PAGE_MAGIC_PATTERNS_TORTURE: &str = "magic_patterns_torture";
    pub(crate) const PAGE_MAGIC_SPARKLES_TEXT: &str = "magic_sparkles_text";
    pub(crate) const PAGE_MAGIC_BLOOM: &str = "magic_bloom";
    pub(crate) const PAGE_ICONS: &str = "icons";
    pub(crate) const PAGE_OVERLAY: &str = "overlay";
    pub(crate) const PAGE_SHADCN_EXTRAS: &str = "shadcn_extras";
    pub(crate) const PAGE_FORMS: &str = "forms";
    pub(crate) const PAGE_DATA_GRID: &str = "data_grid";
    pub(crate) const PAGE_MENUS: &str = "menus";
}
#[cfg(feature = "gallery-dev")]
pub(crate) use gallery_dev_recipe_page_ids::*;
#[cfg(any(feature = "gallery-dev", feature = "gallery-chart"))]
pub(crate) const PAGE_CHART: &str = "chart";
pub(crate) const PAGE_SKELETON: &str = "skeleton";
pub(crate) const PAGE_SCROLL_AREA: &str = "scroll_area";
pub(crate) const PAGE_TOOLTIP: &str = "tooltip";
pub(crate) const PAGE_SLIDER: &str = "slider";
pub(crate) const PAGE_FIELD: &str = "field";
pub(crate) const PAGE_SELECT: &str = "select";
pub(crate) const PAGE_COMBOBOX: &str = "combobox";
pub(crate) const PAGE_DATE_PICKER: &str = "date_picker";
pub(crate) const PAGE_RESIZABLE: &str = "resizable";
pub(crate) const PAGE_DATA_TABLE: &str = "data_table";
pub(crate) const PAGE_TABS: &str = "tabs";
pub(crate) const PAGE_ACCORDION: &str = "accordion";
pub(crate) const PAGE_TABLE: &str = "table";
pub(crate) const PAGE_PROGRESS: &str = "progress";
pub(crate) const PAGE_COMMAND: &str = "command";
pub(crate) const PAGE_TOAST: &str = "toast";
pub(crate) const PAGE_ALERT: &str = "alert";
pub(crate) const PAGE_ALERT_DIALOG: &str = "alert_dialog";
pub(crate) const PAGE_ASPECT_RATIO: &str = "aspect_ratio";
pub(crate) const PAGE_BREADCRUMB: &str = "breadcrumb";
pub(crate) const PAGE_BUTTON_GROUP: &str = "button_group";
pub(crate) const PAGE_CALENDAR: &str = "calendar";
pub(crate) const PAGE_CAROUSEL: &str = "carousel";
pub(crate) const PAGE_CHECKBOX: &str = "checkbox";
pub(crate) const PAGE_COLLAPSIBLE: &str = "collapsible";
pub(crate) const PAGE_CONTEXT_MENU: &str = "context_menu";
pub(crate) const PAGE_DIALOG: &str = "dialog";
pub(crate) const PAGE_DRAWER: &str = "drawer";
pub(crate) const PAGE_DROPDOWN_MENU: &str = "dropdown_menu";
pub(crate) const PAGE_EMPTY: &str = "empty";
pub(crate) const PAGE_FORM: &str = "form";
pub(crate) const PAGE_HOVER_CARD: &str = "hover_card";
pub(crate) const PAGE_INPUT: &str = "input";
pub(crate) const PAGE_INPUT_GROUP: &str = "input_group";
pub(crate) const PAGE_INPUT_OTP: &str = "input_otp";
pub(crate) const PAGE_ITEM: &str = "item";
pub(crate) const PAGE_KBD: &str = "kbd";
pub(crate) const PAGE_LABEL: &str = "label";
pub(crate) const PAGE_MENUBAR: &str = "menubar";
pub(crate) const PAGE_NATIVE_SELECT: &str = "native_select";
pub(crate) const PAGE_NAVIGATION_MENU: &str = "navigation_menu";
pub(crate) const PAGE_PAGINATION: &str = "pagination";
pub(crate) const PAGE_POPOVER: &str = "popover";
pub(crate) const PAGE_RADIO_GROUP: &str = "radio_group";
pub(crate) const PAGE_SEPARATOR: &str = "separator";
pub(crate) const PAGE_SHEET: &str = "sheet";
pub(crate) const PAGE_SIDEBAR: &str = "sidebar";
pub(crate) const PAGE_SONNER: &str = "sonner";
pub(crate) const PAGE_SPINNER: &str = "spinner";
pub(crate) const PAGE_SWITCH: &str = "switch";
pub(crate) const PAGE_TEXTAREA: &str = "textarea";
pub(crate) const PAGE_TOGGLE: &str = "toggle";
pub(crate) const PAGE_TOGGLE_GROUP: &str = "toggle_group";
pub(crate) const PAGE_TYPOGRAPHY: &str = "typography";
#[cfg(feature = "gallery-material3")]
mod gallery_material3_page_ids {
    pub(crate) const PAGE_MATERIAL3_GALLERY: &str = "material3_gallery";
    pub(crate) const PAGE_MATERIAL3_BUTTON: &str = "material3_button";
    pub(crate) const PAGE_MATERIAL3_ICON_BUTTON: &str = "material3_icon_button";
    pub(crate) const PAGE_MATERIAL3_CHECKBOX: &str = "material3_checkbox";
    pub(crate) const PAGE_MATERIAL3_SWITCH: &str = "material3_switch";
    pub(crate) const PAGE_MATERIAL3_SLIDER: &str = "material3_slider";
    pub(crate) const PAGE_MATERIAL3_RADIO: &str = "material3_radio";
    pub(crate) const PAGE_MATERIAL3_BADGE: &str = "material3_badge";
    pub(crate) const PAGE_MATERIAL3_SEGMENTED_BUTTON: &str = "material3_segmented_button";
    pub(crate) const PAGE_MATERIAL3_TOP_APP_BAR: &str = "material3_top_app_bar";
    pub(crate) const PAGE_MATERIAL3_BOTTOM_SHEET: &str = "material3_bottom_sheet";
    pub(crate) const PAGE_MATERIAL3_DATE_PICKER: &str = "material3_date_picker";
    pub(crate) const PAGE_MATERIAL3_TIME_PICKER: &str = "material3_time_picker";
    pub(crate) const PAGE_MATERIAL3_AUTOCOMPLETE: &str = "material3_autocomplete";
    pub(crate) const PAGE_MATERIAL3_SELECT: &str = "material3_select";
    pub(crate) const PAGE_MATERIAL3_TEXT_FIELD: &str = "material3_text_field";
    pub(crate) const PAGE_MATERIAL3_TABS: &str = "material3_tabs";
    pub(crate) const PAGE_MATERIAL3_NAVIGATION_BAR: &str = "material3_navigation_bar";
    pub(crate) const PAGE_MATERIAL3_NAVIGATION_RAIL: &str = "material3_navigation_rail";
    pub(crate) const PAGE_MATERIAL3_NAVIGATION_DRAWER: &str = "material3_navigation_drawer";
    pub(crate) const PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str =
        "material3_modal_navigation_drawer";
    pub(crate) const PAGE_MATERIAL3_DIALOG: &str = "material3_dialog";
    pub(crate) const PAGE_MATERIAL3_MENU: &str = "material3_menu";
    pub(crate) const PAGE_MATERIAL3_LIST: &str = "material3_list";
    pub(crate) const PAGE_MATERIAL3_SNACKBAR: &str = "material3_snackbar";
    pub(crate) const PAGE_MATERIAL3_TOOLTIP: &str = "material3_tooltip";
    pub(crate) const PAGE_MATERIAL3_STATE_MATRIX: &str = "material3_state_matrix";
    pub(crate) const PAGE_MATERIAL3_TOUCH_TARGETS: &str = "material3_touch_targets";
}
#[cfg(feature = "gallery-material3")]
pub(crate) use gallery_material3_page_ids::*;

pub(crate) const CMD_NAV_INTRO: &str = "ui_gallery.nav.select.intro";
pub(crate) const CMD_NAV_LAYOUT: &str = "ui_gallery.nav.select.layout";
pub(crate) const CMD_NAV_MOTION_PRESETS: &str = "ui_gallery.nav.select.motion_presets";
pub(crate) const CMD_NAV_VIEW_CACHE: &str = "ui_gallery.nav.select.view_cache";
#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
mod gallery_dev_nav_commands {
    pub(crate) const CMD_NAV_HIT_TEST_TORTURE: &str = "ui_gallery.nav.select.hit_test_torture";
    pub(crate) const CMD_NAV_HIT_TEST_ONLY_PAINT_CACHE_PROBE: &str =
        "ui_gallery.nav.select.hit_test_only_paint_cache_probe";
    pub(crate) const CMD_NAV_VIRTUAL_LIST_TORTURE: &str =
        "ui_gallery.nav.select.virtual_list_torture";
    pub(crate) const CMD_NAV_UI_KIT_LIST_TORTURE: &str =
        "ui_gallery.nav.select.ui_kit_list_torture";
    pub(crate) const CMD_NAV_CODE_VIEW_TORTURE: &str = "ui_gallery.nav.select.code_view_torture";
    pub(crate) const CMD_NAV_CODE_EDITOR_MVP: &str = "ui_gallery.nav.select.code_editor_mvp";
    pub(crate) const CMD_NAV_CODE_EDITOR_TORTURE: &str =
        "ui_gallery.nav.select.code_editor_torture";
    pub(crate) const CMD_NAV_MARKDOWN_EDITOR_SOURCE: &str =
        "ui_gallery.nav.select.markdown_editor_source";
    pub(crate) const CMD_NAV_TEXT_SELECTION_PERF: &str =
        "ui_gallery.nav.select.text_selection_perf";
    pub(crate) const CMD_NAV_TEXT_BIDI_RTL_CONFORMANCE: &str =
        "ui_gallery.nav.select.text_bidi_rtl_conformance";
    pub(crate) const CMD_NAV_TEXT_MIXED_SCRIPT_FALLBACK: &str =
        "ui_gallery.nav.select.text_mixed_script_fallback";
    pub(crate) const CMD_NAV_TEXT_MEASURE_OVERLAY: &str =
        "ui_gallery.nav.select.text_measure_overlay";
    pub(crate) const CMD_NAV_TEXT_FEATURE_TOGGLES: &str =
        "ui_gallery.nav.select.text_feature_toggles";
    pub(crate) const CMD_NAV_TEXT_OUTLINE_STROKE: &str =
        "ui_gallery.nav.select.text_outline_stroke";
    pub(crate) const CMD_NAV_WEB_IME_HARNESS: &str = "ui_gallery.nav.select.web_ime_harness";
    pub(crate) const CMD_NAV_CHART_TORTURE: &str = "ui_gallery.nav.select.chart_torture";
    pub(crate) const CMD_NAV_CANVAS_CULL_TORTURE: &str =
        "ui_gallery.nav.select.canvas_cull_torture";
    pub(crate) const CMD_NAV_NODE_GRAPH_CULL_TORTURE: &str =
        "ui_gallery.nav.select.node_graph_cull_torture";
    pub(crate) const CMD_NAV_CHROME_TORTURE: &str = "ui_gallery.nav.select.chrome_torture";
    pub(crate) const CMD_NAV_WINDOWED_ROWS_SURFACE_TORTURE: &str =
        "ui_gallery.nav.select.windowed_rows_surface_torture";
    pub(crate) const CMD_NAV_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str =
        "ui_gallery.nav.select.windowed_rows_surface_interactive_torture";
    pub(crate) const CMD_NAV_DATA_TABLE_TORTURE: &str = "ui_gallery.nav.select.data_table_torture";
    pub(crate) const CMD_NAV_TREE_TORTURE: &str = "ui_gallery.nav.select.tree_torture";
    pub(crate) const CMD_NAV_TABLE_RETAINED_TORTURE: &str =
        "ui_gallery.nav.select.table_retained_torture";
    pub(crate) const CMD_NAV_AI_TRANSCRIPT_TORTURE: &str =
        "ui_gallery.nav.select.ai_transcript_torture";
    pub(crate) const CMD_NAV_AI_CHAT_DEMO: &str = "ui_gallery.nav.select.ai_chat_demo";
    pub(crate) const CMD_NAV_AI_AUDIO_PLAYER_DEMO: &str =
        "ui_gallery.nav.select.ai_audio_player_demo";
    pub(crate) const CMD_NAV_AI_TRANSCRIPTION_DEMO: &str =
        "ui_gallery.nav.select.ai_transcription_demo";
    pub(crate) const CMD_NAV_AI_SPEECH_INPUT_DEMO: &str =
        "ui_gallery.nav.select.ai_speech_input_demo";
    pub(crate) const CMD_NAV_AI_MIC_SELECTOR_DEMO: &str =
        "ui_gallery.nav.select.ai_mic_selector_demo";
    pub(crate) const CMD_NAV_AI_VOICE_SELECTOR_DEMO: &str =
        "ui_gallery.nav.select.ai_voice_selector_demo";
    pub(crate) const CMD_NAV_AI_AGENT_DEMO: &str = "ui_gallery.nav.select.ai_agent_demo";
    pub(crate) const CMD_NAV_AI_SANDBOX_DEMO: &str = "ui_gallery.nav.select.ai_sandbox_demo";
    pub(crate) const CMD_NAV_AI_PERSONA_DEMO: &str = "ui_gallery.nav.select.ai_persona_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_CHROME_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_chrome_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_CANVAS_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_canvas_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_NODE_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_node_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_EDGE_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_edge_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_CONNECTION_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_connection_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_CONTROLS_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_controls_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_PANEL_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_panel_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_TOOLBAR_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_toolbar_demo";
    pub(crate) const CMD_NAV_AI_WORKFLOW_NODE_GRAPH_DEMO: &str =
        "ui_gallery.nav.select.ai_workflow_node_graph_demo";
    pub(crate) const CMD_NAV_AI_CANVAS_WORLD_LAYER_SPIKE: &str =
        "ui_gallery.nav.select.ai_canvas_world_layer_spike";
    pub(crate) const CMD_NAV_AI_PROMPT_INPUT_PROVIDER_DEMO: &str =
        "ui_gallery.nav.select.ai_prompt_input_provider_demo";
    pub(crate) const CMD_NAV_AI_PROMPT_INPUT_ACTION_MENU_DEMO: &str =
        "ui_gallery.nav.select.ai_prompt_input_action_menu_demo";
    pub(crate) const CMD_NAV_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO: &str =
        "ui_gallery.nav.select.ai_prompt_input_referenced_sources_demo";
    pub(crate) const CMD_NAV_AI_INLINE_CITATION_DEMO: &str =
        "ui_gallery.nav.select.ai_inline_citation_demo";
    pub(crate) const CMD_NAV_AI_SOURCES_DEMO: &str = "ui_gallery.nav.select.ai_sources_demo";
    pub(crate) const CMD_NAV_AI_QUEUE_DEMO: &str = "ui_gallery.nav.select.ai_queue_demo";
    pub(crate) const CMD_NAV_AI_ATTACHMENTS_DEMO: &str =
        "ui_gallery.nav.select.ai_attachments_demo";
    pub(crate) const CMD_NAV_AI_SUGGESTIONS_DEMO: &str =
        "ui_gallery.nav.select.ai_suggestions_demo";
    pub(crate) const CMD_NAV_AI_MESSAGE_BRANCH_DEMO: &str =
        "ui_gallery.nav.select.ai_message_branch_demo";
    pub(crate) const CMD_NAV_AI_FILE_TREE_DEMO: &str = "ui_gallery.nav.select.ai_file_tree_demo";
    pub(crate) const CMD_NAV_AI_COMMIT_LARGE_DEMO: &str =
        "ui_gallery.nav.select.ai_commit_large_demo";
    pub(crate) const CMD_NAV_AI_STACK_TRACE_LARGE_DEMO: &str =
        "ui_gallery.nav.select.ai_stack_trace_large_demo";
    pub(crate) const CMD_NAV_AI_TEST_RESULTS_LARGE_DEMO: &str =
        "ui_gallery.nav.select.ai_test_results_large_demo";
    pub(crate) const CMD_NAV_AI_CHECKPOINT_DEMO: &str = "ui_gallery.nav.select.ai_checkpoint_demo";
    pub(crate) const CMD_NAV_AI_TOOL_DEMO: &str = "ui_gallery.nav.select.ai_tool_demo";
    pub(crate) const CMD_NAV_AI_WEB_PREVIEW_DEMO: &str =
        "ui_gallery.nav.select.ai_web_preview_demo";
    pub(crate) const CMD_NAV_AI_MODEL_SELECTOR_DEMO: &str =
        "ui_gallery.nav.select.ai_model_selector_demo";
    pub(crate) const CMD_NAV_AI_IMAGE_DEMO: &str = "ui_gallery.nav.select.ai_image_demo";
    pub(crate) const CMD_NAV_INSPECTOR_TORTURE: &str = "ui_gallery.nav.select.inspector_torture";
    pub(crate) const CMD_NAV_FILE_TREE_TORTURE: &str = "ui_gallery.nav.select.file_tree_torture";
}
#[cfg(any(feature = "gallery-dev", feature = "gallery-web-ime-harness"))]
pub(crate) use gallery_dev_nav_commands::*;

#[cfg(feature = "gallery-ai")]
mod gallery_ai_nav_commands {
    pub(crate) const CMD_NAV_AI_CONVERSATION_DEMO: &str =
        "ui_gallery.nav.select.ai_conversation_demo";
    pub(crate) const CMD_NAV_AI_MESSAGE_DEMO: &str = "ui_gallery.nav.select.ai_message_demo";
    pub(crate) const CMD_NAV_AI_CONTEXT_DEMO: &str = "ui_gallery.nav.select.ai_context_demo";
    pub(crate) const CMD_NAV_AI_TERMINAL_DEMO: &str = "ui_gallery.nav.select.ai_terminal_demo";
    pub(crate) const CMD_NAV_AI_PACKAGE_INFO_DEMO: &str =
        "ui_gallery.nav.select.ai_package_info_demo";
    pub(crate) const CMD_NAV_AI_OPEN_IN_CHAT_DEMO: &str =
        "ui_gallery.nav.select.ai_open_in_chat_demo";
    pub(crate) const CMD_NAV_AI_TASK_DEMO: &str = "ui_gallery.nav.select.ai_task_demo";
    pub(crate) const CMD_NAV_AI_PROMPT_INPUT_DOCS_DEMO: &str =
        "ui_gallery.nav.select.ai_prompt_input_docs_demo";
    pub(crate) const CMD_NAV_AI_ARTIFACT_DEMO: &str = "ui_gallery.nav.select.ai_artifact_demo";
    pub(crate) const CMD_NAV_AI_SHIMMER_DEMO: &str = "ui_gallery.nav.select.ai_shimmer_demo";
    pub(crate) const CMD_NAV_AI_REASONING_DEMO: &str = "ui_gallery.nav.select.ai_reasoning_demo";
    pub(crate) const CMD_NAV_AI_CODE_BLOCK_DEMO: &str = "ui_gallery.nav.select.ai_code_block_demo";
    pub(crate) const CMD_NAV_AI_SNIPPET_DEMO: &str = "ui_gallery.nav.select.ai_snippet_demo";
    pub(crate) const CMD_NAV_AI_COMMIT_DEMO: &str = "ui_gallery.nav.select.ai_commit_demo";
    pub(crate) const CMD_NAV_AI_STACK_TRACE_DEMO: &str =
        "ui_gallery.nav.select.ai_stack_trace_demo";
    pub(crate) const CMD_NAV_AI_SCHEMA_DISPLAY_DEMO: &str =
        "ui_gallery.nav.select.ai_schema_display_demo";
    pub(crate) const CMD_NAV_AI_TEST_RESULTS_DEMO: &str =
        "ui_gallery.nav.select.ai_test_results_demo";
    pub(crate) const CMD_NAV_AI_CONFIRMATION_DEMO: &str =
        "ui_gallery.nav.select.ai_confirmation_demo";
    pub(crate) const CMD_NAV_AI_ENVIRONMENT_VARIABLES_DEMO: &str =
        "ui_gallery.nav.select.ai_environment_variables_demo";
    pub(crate) const CMD_NAV_AI_PLAN_DEMO: &str = "ui_gallery.nav.select.ai_plan_demo";
    pub(crate) const CMD_NAV_AI_CHAIN_OF_THOUGHT_DEMO: &str =
        "ui_gallery.nav.select.ai_chain_of_thought_demo";
}
#[cfg(feature = "gallery-ai")]
pub(crate) use gallery_ai_nav_commands::*;
pub(crate) const CMD_NAV_BUTTON: &str = "ui_gallery.nav.select.button";
pub(crate) const CMD_NAV_CARD: &str = "ui_gallery.nav.select.card";
pub(crate) const CMD_NAV_BADGE: &str = "ui_gallery.nav.select.badge";
pub(crate) const CMD_NAV_AVATAR: &str = "ui_gallery.nav.select.avatar";
#[cfg(feature = "gallery-dev")]
mod gallery_dev_recipe_nav_commands {
    pub(crate) const CMD_NAV_IMAGE_OBJECT_FIT: &str = "ui_gallery.nav.select.image_object_fit";
    pub(crate) const CMD_NAV_MAGIC_MARQUEE: &str = "ui_gallery.nav.select.magic_marquee";
    pub(crate) const CMD_NAV_MAGIC_CARD: &str = "ui_gallery.nav.select.magic_card";
    pub(crate) const CMD_NAV_MAGIC_LENS: &str = "ui_gallery.nav.select.magic_lens";
    pub(crate) const CMD_NAV_MAGIC_BORDER_BEAM: &str = "ui_gallery.nav.select.magic_border_beam";
    pub(crate) const CMD_NAV_MAGIC_DOCK: &str = "ui_gallery.nav.select.magic_dock";
    pub(crate) const CMD_NAV_MAGIC_PATTERNS: &str = "ui_gallery.nav.select.magic_patterns";
    pub(crate) const CMD_NAV_MAGIC_PATTERNS_TORTURE: &str =
        "ui_gallery.nav.select.magic_patterns_torture";
    pub(crate) const CMD_NAV_MAGIC_SPARKLES_TEXT: &str =
        "ui_gallery.nav.select.magic_sparkles_text";
    pub(crate) const CMD_NAV_MAGIC_BLOOM: &str = "ui_gallery.nav.select.magic_bloom";
    pub(crate) const CMD_NAV_ICONS: &str = "ui_gallery.nav.select.icons";
    pub(crate) const CMD_NAV_OVERLAY: &str = "ui_gallery.nav.select.overlay";
    pub(crate) const CMD_NAV_SHADCN_EXTRAS: &str = "ui_gallery.nav.select.shadcn_extras";
    pub(crate) const CMD_NAV_FORMS: &str = "ui_gallery.nav.select.forms";
    pub(crate) const CMD_NAV_DATA_GRID: &str = "ui_gallery.nav.select.data_grid";
    pub(crate) const CMD_NAV_MENUS: &str = "ui_gallery.nav.select.menus";
    pub(crate) const CMD_NAV_FORM: &str = "ui_gallery.nav.select.form";
}
#[cfg(feature = "gallery-dev")]
pub(crate) use gallery_dev_recipe_nav_commands::*;
#[cfg(any(feature = "gallery-dev", feature = "gallery-chart"))]
pub(crate) const CMD_NAV_CHART: &str = "ui_gallery.nav.select.chart";
pub(crate) const CMD_NAV_SKELETON: &str = "ui_gallery.nav.select.skeleton";
pub(crate) const CMD_NAV_SCROLL_AREA: &str = "ui_gallery.nav.select.scroll_area";
pub(crate) const CMD_NAV_TOOLTIP: &str = "ui_gallery.nav.select.tooltip";
pub(crate) const CMD_NAV_SLIDER: &str = "ui_gallery.nav.select.slider";
pub(crate) const CMD_NAV_FIELD: &str = "ui_gallery.nav.select.field";
pub(crate) const CMD_NAV_SELECT: &str = "ui_gallery.nav.select.select";
pub(crate) const CMD_NAV_COMBOBOX: &str = "ui_gallery.nav.select.combobox";
pub(crate) const CMD_NAV_DATE_PICKER: &str = "ui_gallery.nav.select.date_picker";
pub(crate) const CMD_NAV_RESIZABLE: &str = "ui_gallery.nav.select.resizable";
pub(crate) const CMD_NAV_DATA_TABLE: &str = "ui_gallery.nav.select.data_table";
pub(crate) const CMD_NAV_TABS: &str = "ui_gallery.nav.select.tabs";
pub(crate) const CMD_NAV_ACCORDION: &str = "ui_gallery.nav.select.accordion";
pub(crate) const CMD_NAV_TABLE: &str = "ui_gallery.nav.select.table";
pub(crate) const CMD_NAV_PROGRESS: &str = "ui_gallery.nav.select.progress";
pub(crate) const CMD_NAV_COMMAND: &str = "ui_gallery.nav.select.command";
pub(crate) const CMD_NAV_TOAST: &str = "ui_gallery.nav.select.toast";
pub(crate) const CMD_NAV_ALERT: &str = "ui_gallery.nav.select.alert";
pub(crate) const CMD_NAV_ALERT_DIALOG: &str = "ui_gallery.nav.select.alert_dialog";
pub(crate) const CMD_NAV_ASPECT_RATIO: &str = "ui_gallery.nav.select.aspect_ratio";
pub(crate) const CMD_NAV_BREADCRUMB: &str = "ui_gallery.nav.select.breadcrumb";
pub(crate) const CMD_NAV_BUTTON_GROUP: &str = "ui_gallery.nav.select.button_group";
pub(crate) const CMD_NAV_CALENDAR: &str = "ui_gallery.nav.select.calendar";
pub(crate) const CMD_NAV_CAROUSEL: &str = "ui_gallery.nav.select.carousel";
pub(crate) const CMD_NAV_CHECKBOX: &str = "ui_gallery.nav.select.checkbox";
pub(crate) const CMD_NAV_COLLAPSIBLE: &str = "ui_gallery.nav.select.collapsible";
pub(crate) const CMD_NAV_CONTEXT_MENU: &str = "ui_gallery.nav.select.context_menu";
pub(crate) const CMD_NAV_DIALOG: &str = "ui_gallery.nav.select.dialog";
pub(crate) const CMD_NAV_DRAWER: &str = "ui_gallery.nav.select.drawer";
pub(crate) const CMD_NAV_DROPDOWN_MENU: &str = "ui_gallery.nav.select.dropdown_menu";
pub(crate) const CMD_NAV_EMPTY: &str = "ui_gallery.nav.select.empty";
pub(crate) const CMD_NAV_HOVER_CARD: &str = "ui_gallery.nav.select.hover_card";
pub(crate) const CMD_NAV_INPUT: &str = "ui_gallery.nav.select.input";
pub(crate) const CMD_NAV_INPUT_GROUP: &str = "ui_gallery.nav.select.input_group";
pub(crate) const CMD_NAV_INPUT_OTP: &str = "ui_gallery.nav.select.input_otp";
pub(crate) const CMD_NAV_ITEM: &str = "ui_gallery.nav.select.item";
pub(crate) const CMD_NAV_KBD: &str = "ui_gallery.nav.select.kbd";
pub(crate) const CMD_NAV_LABEL: &str = "ui_gallery.nav.select.label";
pub(crate) const CMD_NAV_MENUBAR: &str = "ui_gallery.nav.select.menubar";
pub(crate) const CMD_NAV_NATIVE_SELECT: &str = "ui_gallery.nav.select.native_select";
pub(crate) const CMD_NAV_NAVIGATION_MENU: &str = "ui_gallery.nav.select.navigation_menu";
pub(crate) const CMD_NAV_PAGINATION: &str = "ui_gallery.nav.select.pagination";
pub(crate) const CMD_NAV_POPOVER: &str = "ui_gallery.nav.select.popover";
pub(crate) const CMD_NAV_RADIO_GROUP: &str = "ui_gallery.nav.select.radio_group";
pub(crate) const CMD_NAV_SEPARATOR: &str = "ui_gallery.nav.select.separator";
pub(crate) const CMD_NAV_SHEET: &str = "ui_gallery.nav.select.sheet";
pub(crate) const CMD_NAV_SIDEBAR: &str = "ui_gallery.nav.select.sidebar";
pub(crate) const CMD_NAV_SONNER: &str = "ui_gallery.nav.select.sonner";
pub(crate) const CMD_NAV_SPINNER: &str = "ui_gallery.nav.select.spinner";
pub(crate) const CMD_NAV_SWITCH: &str = "ui_gallery.nav.select.switch";
pub(crate) const CMD_NAV_TEXTAREA: &str = "ui_gallery.nav.select.textarea";
pub(crate) const CMD_NAV_TOGGLE: &str = "ui_gallery.nav.select.toggle";
pub(crate) const CMD_NAV_TOGGLE_GROUP: &str = "ui_gallery.nav.select.toggle_group";
pub(crate) const CMD_NAV_TYPOGRAPHY: &str = "ui_gallery.nav.select.typography";
#[cfg(feature = "gallery-material3")]
mod gallery_material3_nav_commands {
    pub(crate) const CMD_NAV_MATERIAL3_GALLERY: &str = "ui_gallery.nav.select.material3_gallery";
    pub(crate) const CMD_NAV_MATERIAL3_BUTTON: &str = "ui_gallery.nav.select.material3_button";
    pub(crate) const CMD_NAV_MATERIAL3_ICON_BUTTON: &str =
        "ui_gallery.nav.select.material3_icon_button";
    pub(crate) const CMD_NAV_MATERIAL3_CHECKBOX: &str = "ui_gallery.nav.select.material3_checkbox";
    pub(crate) const CMD_NAV_MATERIAL3_SWITCH: &str = "ui_gallery.nav.select.material3_switch";
    pub(crate) const CMD_NAV_MATERIAL3_SLIDER: &str = "ui_gallery.nav.select.material3_slider";
    pub(crate) const CMD_NAV_MATERIAL3_RADIO: &str = "ui_gallery.nav.select.material3_radio";
    pub(crate) const CMD_NAV_MATERIAL3_BADGE: &str = "ui_gallery.nav.select.material3_badge";
    pub(crate) const CMD_NAV_MATERIAL3_SEGMENTED_BUTTON: &str =
        "ui_gallery.nav.select.material3_segmented_button";
    pub(crate) const CMD_NAV_MATERIAL3_TOP_APP_BAR: &str =
        "ui_gallery.nav.select.material3_top_app_bar";
    pub(crate) const CMD_NAV_MATERIAL3_BOTTOM_SHEET: &str =
        "ui_gallery.nav.select.material3_bottom_sheet";
    pub(crate) const CMD_NAV_MATERIAL3_DATE_PICKER: &str =
        "ui_gallery.nav.select.material3_date_picker";
    pub(crate) const CMD_NAV_MATERIAL3_TIME_PICKER: &str =
        "ui_gallery.nav.select.material3_time_picker";
    pub(crate) const CMD_NAV_MATERIAL3_AUTOCOMPLETE: &str =
        "ui_gallery.nav.select.material3_autocomplete";
    pub(crate) const CMD_NAV_MATERIAL3_SELECT: &str = "ui_gallery.nav.select.material3_select";
    pub(crate) const CMD_NAV_MATERIAL3_TEXT_FIELD: &str =
        "ui_gallery.nav.select.material3_text_field";
    pub(crate) const CMD_NAV_MATERIAL3_TABS: &str = "ui_gallery.nav.select.material3_tabs";
    pub(crate) const CMD_NAV_MATERIAL3_NAVIGATION_BAR: &str =
        "ui_gallery.nav.select.material3_navigation_bar";
    pub(crate) const CMD_NAV_MATERIAL3_NAVIGATION_RAIL: &str =
        "ui_gallery.nav.select.material3_navigation_rail";
    pub(crate) const CMD_NAV_MATERIAL3_NAVIGATION_DRAWER: &str =
        "ui_gallery.nav.select.material3_navigation_drawer";
    pub(crate) const CMD_NAV_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str =
        "ui_gallery.nav.select.material3_modal_navigation_drawer";
    pub(crate) const CMD_NAV_MATERIAL3_DIALOG: &str = "ui_gallery.nav.select.material3_dialog";
    pub(crate) const CMD_NAV_MATERIAL3_MENU: &str = "ui_gallery.nav.select.material3_menu";
    pub(crate) const CMD_NAV_MATERIAL3_LIST: &str = "ui_gallery.nav.select.material3_list";
    pub(crate) const CMD_NAV_MATERIAL3_SNACKBAR: &str = "ui_gallery.nav.select.material3_snackbar";
    pub(crate) const CMD_NAV_MATERIAL3_TOOLTIP: &str = "ui_gallery.nav.select.material3_tooltip";
    pub(crate) const CMD_NAV_MATERIAL3_STATE_MATRIX: &str =
        "ui_gallery.nav.select.material3_state_matrix";
    pub(crate) const CMD_NAV_MATERIAL3_TOUCH_TARGETS: &str =
        "ui_gallery.nav.select.material3_touch_targets";
}
#[cfg(feature = "gallery-material3")]
pub(crate) use gallery_material3_nav_commands::*;

pub(crate) const CMD_PROGRESS_INC: &str = "ui_gallery.progress.inc";
pub(crate) const CMD_PROGRESS_DEC: &str = "ui_gallery.progress.dec";
pub(crate) const CMD_PROGRESS_RESET: &str = "ui_gallery.progress.reset";

pub(crate) const CMD_VIEW_CACHE_BUMP: &str = "ui_gallery.view_cache.bump";
pub(crate) const CMD_VIEW_CACHE_RESET: &str = "ui_gallery.view_cache.reset";

#[cfg(feature = "gallery-dev")]
pub(crate) const CMD_VIRTUAL_LIST_TORTURE_JUMP: &str = "ui_gallery.virtual_list_torture.jump";
#[cfg(feature = "gallery-dev")]
pub(crate) const CMD_VIRTUAL_LIST_TORTURE_SCROLL_BOTTOM: &str =
    "ui_gallery.virtual_list_torture.scroll_bottom";
#[cfg(feature = "gallery-dev")]
pub(crate) const CMD_VIRTUAL_LIST_TORTURE_CLEAR_EDIT: &str =
    "ui_gallery.virtual_list_torture.clear_edit";

pub(crate) const CMD_MENU_DROPDOWN_APPLE: &str = "ui_gallery.menu.dropdown.apple";
pub(crate) const CMD_MENU_DROPDOWN_ORANGE: &str = "ui_gallery.menu.dropdown.orange";
pub(crate) const CMD_MENU_CONTEXT_ACTION: &str = "ui_gallery.menu.context.action";

pub(crate) const CMD_TOAST_DEFAULT: &str = "ui_gallery.toast.default";
pub(crate) const CMD_TOAST_SUCCESS: &str = "ui_gallery.toast.success";
pub(crate) const CMD_TOAST_ERROR: &str = "ui_gallery.toast.error";
pub(crate) const CMD_TOAST_SHOW_ACTION_CANCEL: &str = "ui_gallery.toast.show_action_cancel";
pub(crate) const CMD_TOAST_ACTION: &str = "ui_gallery.toast.action";
pub(crate) const CMD_TOAST_CANCEL: &str = "ui_gallery.toast.cancel";

pub(crate) const CMD_APP_OPEN: &str = "ui_gallery.app.open";
pub(crate) const CMD_APP_SAVE: &str = "ui_gallery.app.save";
pub(crate) const CMD_APP_SETTINGS: &str = "ui_gallery.app.settings";
pub(crate) const CMD_APP_SETTINGS_APPLY: &str = "ui_gallery.app.settings.apply";
pub(crate) const CMD_APP_SETTINGS_WRITE_PROJECT: &str = "ui_gallery.app.settings.write_project";
pub(crate) const CMD_APP_TOGGLE_PREFERENCES_ENABLED: &str =
    "ui_gallery.app.preferences.toggle_enabled";

pub(crate) const CMD_MENU_BAR_OS_AUTO: &str = "ui_gallery.menu_bar.os.auto";
pub(crate) const CMD_MENU_BAR_OS_ON: &str = "ui_gallery.menu_bar.os.on";
pub(crate) const CMD_MENU_BAR_OS_OFF: &str = "ui_gallery.menu_bar.os.off";

pub(crate) const CMD_MENU_BAR_IN_WINDOW_AUTO: &str = "ui_gallery.menu_bar.in_window.auto";
pub(crate) const CMD_MENU_BAR_IN_WINDOW_ON: &str = "ui_gallery.menu_bar.in_window.on";
pub(crate) const CMD_MENU_BAR_IN_WINDOW_OFF: &str = "ui_gallery.menu_bar.in_window.off";

pub(crate) const CMD_GALLERY_DEBUG_RECENT_ADD: &str = "ui_gallery.debug.recent.add";
pub(crate) const CMD_GALLERY_DEBUG_RECENT_CLEAR: &str = "ui_gallery.debug.recent.clear";
pub(crate) const CMD_GALLERY_DEBUG_WINDOW_OPEN: &str = "ui_gallery.debug.window.open";
pub(crate) const CMD_GALLERY_RECENT_OPEN_PREFIX: &str = "ui_gallery.recent.open.";
pub(crate) const CMD_GALLERY_WINDOW_ACTIVATE_PREFIX: &str = "ui_gallery.window.activate.";
pub(crate) const CMD_GALLERY_PAGE_BACK: &str = "ui_gallery.page.back";
pub(crate) const CMD_GALLERY_PAGE_FORWARD: &str = "ui_gallery.page.forward";

pub(crate) const CMD_SHELL_SHARE_SHEET_SMOKE: &str = "ui_gallery.shell.share_sheet_smoke";

pub(crate) const CMD_CODE_EDITOR_LOAD_FONTS: &str = "ui_gallery.code_editor.load_fonts";
pub(crate) const CMD_CODE_EDITOR_DUMP_TAFFY: &str = "ui_gallery.code_editor.dump_taffy";
pub(crate) const CMD_INPUT_PICTURE_BROWSE: &str = "ui_gallery.input.picture.browse";

#[derive(Clone, Copy)]
pub(crate) struct PageSpec {
    pub(crate) id: &'static str,
    pub(crate) label: &'static str,
    pub(crate) title: &'static str,
    pub(crate) origin: &'static str,
    pub(crate) command: &'static str,
    pub(crate) tags: &'static [&'static str],
}

impl PageSpec {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        id: &'static str,
        label: &'static str,
        title: &'static str,
        origin: &'static str,
        command: &'static str,
        tags: &'static [&'static str],
    ) -> Self {
        Self {
            id,
            label,
            title,
            origin,
            command,
            tags,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct PageGroupSpec {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [PageSpec],
}

pub(crate) static PAGE_GROUPS: &[PageGroupSpec] = &[
    PageGroupSpec {
        title: "Core",
        items: &[
            PageSpec::new(
                PAGE_INTRO,
                "Introduction",
                "Introduction",
                "Core contracts",
                CMD_NAV_INTRO,
                &["overview", "contracts"],
            ),
            PageSpec::new(
                PAGE_LAYOUT,
                "Layout",
                "Layout / Stacks & Constraints",
                "Layout system",
                CMD_NAV_LAYOUT,
                &["layout", "flex", "stack"],
            ),
            PageSpec::new(
                PAGE_MOTION_PRESETS,
                "Motion Presets",
                "Motion Presets / Theme Token Overrides",
                "Theme tokens (motion)",
                CMD_NAV_MOTION_PRESETS,
                &["motion", "tokens", "theme", "animation"],
            ),
            PageSpec::new(
                PAGE_VIEW_CACHE,
                "View Cache",
                "View Cache / Subtree Reuse",
                "fret-ui (runtime experiments)",
                CMD_NAV_VIEW_CACHE,
                &["cache", "performance", "gpui-parity"],
            ),
        ],
    },
    #[cfg(feature = "gallery-web-ime-harness")]
    PageGroupSpec {
        title: "Core (Web IME)",
        items: &[PageSpec::new(
            PAGE_WEB_IME_HARNESS,
            "Web IME (Harness)",
            "Web / IME + TextInput Bridge Harness",
            "fret-platform-web (textarea bridge, v1)",
            CMD_NAV_WEB_IME_HARNESS,
            &["web", "ime", "text-input", "wasm", "harness"],
        )],
    },
    #[cfg(all(feature = "gallery-web-ime-harness", not(feature = "gallery-dev")))]
    PageGroupSpec {
        title: "Core (Text Harness)",
        items: &[PageSpec::new(
            PAGE_TEXT_MIXED_SCRIPT_FALLBACK,
            "Text Mixed Script (Fallback)",
            "Text / Mixed-Script Fallback (Bundled Fonts)",
            "Font system workstream",
            CMD_NAV_TEXT_MIXED_SCRIPT_FALLBACK,
            &[
                "text",
                "fonts",
                "fallback",
                "cjk",
                "emoji",
                "diagnostics",
                "no-tofu",
            ],
        )],
    },
    #[cfg(feature = "gallery-dev")]
    PageGroupSpec {
        title: "Core (Dev)",
        items: &[
            PageSpec::new(
                PAGE_HIT_TEST_TORTURE,
                "Hit Test (Torture)",
                "Hit Test / Spatial Index Harness",
                "fret-ui (hit testing)",
                CMD_NAV_HIT_TEST_TORTURE,
                &["hit_test", "pointer", "dispatch", "performance", "harness"],
            ),
            PageSpec::new(
                PAGE_HIT_TEST_ONLY_PAINT_CACHE_PROBE,
                "HitTestOnly Paint-Cache Probe",
                "Hit Test / Paint Cache Gate Probe",
                "fret-ui (paint-cache diagnostics)",
                CMD_NAV_HIT_TEST_ONLY_PAINT_CACHE_PROBE,
                &[
                    "hit_test",
                    "paint_cache",
                    "diagnostics",
                    "performance",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_VIRTUAL_LIST_TORTURE,
                "Virtual List (Torture)",
                "Virtual List / Torture Harness",
                "fret-ui (virtualization contract)",
                CMD_NAV_VIRTUAL_LIST_TORTURE,
                &["virtual_list", "performance", "gpui-parity", "harness"],
            ),
            PageSpec::new(
                PAGE_UI_KIT_LIST_TORTURE,
                "List (UI Kit Torture)",
                "List / UI Kit Retained Virtualization Harness",
                "fret-ui-kit (retained-host list surface)",
                CMD_NAV_UI_KIT_LIST_TORTURE,
                &[
                    "list",
                    "virtual_list",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_CODE_VIEW_TORTURE,
                "Code View (Torture)",
                "Code View / Large Document Harness",
                "fret-code-view (windowed surface candidate)",
                CMD_NAV_CODE_VIEW_TORTURE,
                &[
                    "code",
                    "text",
                    "scroll",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_CODE_EDITOR_MVP,
                "Code Editor (MVP)",
                "Code Editor / TextInputRegion MVP",
                "fret-code-editor (ecosystem surface)",
                CMD_NAV_CODE_EDITOR_MVP,
                &["code", "editor", "ime", "text-input", "windowed-rows"],
            ),
            PageSpec::new(
                PAGE_CODE_EDITOR_TORTURE,
                "Code Editor (Torture)",
                "Code Editor / Scroll Stability Harness",
                "fret-code-editor (windowed surface + caching)",
                CMD_NAV_CODE_EDITOR_TORTURE,
                &[
                    "code",
                    "editor",
                    "scroll",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_MARKDOWN_EDITOR_SOURCE,
                "Markdown Editor (Source)",
                "Markdown / Source-mode Editor (v0)",
                "code-editor ecosystem milestone",
                CMD_NAV_MARKDOWN_EDITOR_SOURCE,
                &["markdown", "editor", "source-mode", "preview"],
            ),
            PageSpec::new(
                PAGE_TEXT_SELECTION_PERF,
                "Text Selection (Perf)",
                "Text Selection / Selection Rect Culling",
                "Text integration workstream",
                CMD_NAV_TEXT_SELECTION_PERF,
                &["text", "selection", "performance", "diagnostics", "tli1"],
            ),
            PageSpec::new(
                PAGE_TEXT_BIDI_RTL_CONFORMANCE,
                "Text BiDi/RTL",
                "Text / BiDi + RTL Conformance Harness",
                "Text integration workstream",
                CMD_NAV_TEXT_BIDI_RTL_CONFORMANCE,
                &["text", "bidi", "rtl", "geometry", "diagnostics", "tli1"],
            ),
            PageSpec::new(
                PAGE_TEXT_MIXED_SCRIPT_FALLBACK,
                "Text Mixed Script (Fallback)",
                "Text / Mixed-Script Fallback (Bundled Fonts)",
                "Font system workstream",
                CMD_NAV_TEXT_MIXED_SCRIPT_FALLBACK,
                &[
                    "text",
                    "fonts",
                    "fallback",
                    "cjk",
                    "emoji",
                    "diagnostics",
                    "no-tofu",
                ],
            ),
            PageSpec::new(
                PAGE_TEXT_MEASURE_OVERLAY,
                "Text Measure (Overlay)",
                "Text / Measured Bounds Overlay",
                "Text integration workstream",
                CMD_NAV_TEXT_MEASURE_OVERLAY,
                &["text", "layout", "measure", "diagnostics", "tli1"],
            ),
            PageSpec::new(
                PAGE_TEXT_OUTLINE_STROKE,
                "Text Outline/Stroke",
                "Text / Outline (v1) Surface Probe",
                "Text outline/stroke surface v1",
                CMD_NAV_TEXT_OUTLINE_STROKE,
                &["text", "outline", "stroke", "renderer", "wgpu", "tOSv1"],
            ),
            PageSpec::new(
                PAGE_TEXT_FEATURE_TOGGLES,
                "Text Features (OpenType)",
                "Text / OpenType Feature Toggles",
                "Text shaping surface v1",
                CMD_NAV_TEXT_FEATURE_TOGGLES,
                &[
                    "text", "shaping", "opentype", "features", "liga", "calt", "tsv1",
                ],
            ),
            PageSpec::new(
                PAGE_CHART_TORTURE,
                "Chart (Torture)",
                "Chart / Pan-Zoom Canvas Harness",
                "fret-chart + delinea (sampling/window candidate)",
                CMD_NAV_CHART_TORTURE,
                &[
                    "chart",
                    "plot",
                    "canvas",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_CANVAS_CULL_TORTURE,
                "Canvas Cull (Torture)",
                "Canvas / Pan-Zoom Culling Harness",
                "fret-canvas (viewport culling candidate)",
                CMD_NAV_CANVAS_CULL_TORTURE,
                &[
                    "canvas",
                    "node_graph",
                    "culling",
                    "pan_zoom",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_NODE_GRAPH_CULL_TORTURE,
                "Node Graph Cull (Torture)",
                "Node Graph / Pan-Zoom Culling Harness",
                "fret-node (viewport culling candidate)",
                CMD_NAV_NODE_GRAPH_CULL_TORTURE,
                &[
                    "node_graph",
                    "canvas",
                    "culling",
                    "pan_zoom",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_CHROME_TORTURE,
                "Chrome (Torture)",
                "Chrome / Hover-Focus Overlay Harness",
                "fret-ui-shadcn + fret-ui (paint-only candidate)",
                CMD_NAV_CHROME_TORTURE,
                &[
                    "hover",
                    "focus",
                    "overlay",
                    "chrome",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_WINDOWED_ROWS_SURFACE_TORTURE,
                "Windowed Rows Surface",
                "Windowed Rows Surface / Scroll + Canvas Harness",
                "fret-ui-kit (scroll + canvas pattern)",
                CMD_NAV_WINDOWED_ROWS_SURFACE_TORTURE,
                &["scroll", "performance", "gpui-parity", "canvas", "harness"],
            ),
            PageSpec::new(
                PAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE,
                "Windowed Rows (Interactive)",
                "Windowed Rows Surface / Pointer + Paint-only Chrome",
                "fret-ui-kit (windowed surface + pointer hit testing)",
                CMD_NAV_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE,
                &[
                    "scroll",
                    "performance",
                    "gpui-parity",
                    "canvas",
                    "pointer",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_DATA_TABLE_TORTURE,
                "DataTable (Torture)",
                "DataTable / Virtualized Table Harness",
                "fret-ui-shadcn + fret-ui-kit (virtualized table)",
                CMD_NAV_DATA_TABLE_TORTURE,
                &[
                    "table",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_TREE_TORTURE,
                "Tree (Torture)",
                "Tree / Virtualized Tree Harness",
                "fret-ui-kit (virtualized tree)",
                CMD_NAV_TREE_TORTURE,
                &[
                    "tree",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_TABLE_RETAINED_TORTURE,
                "Table (Retained Torture)",
                "UI Kit Table / Retained Host Harness",
                "fret-ui-kit (virt-003 retained table v0)",
                CMD_NAV_TABLE_RETAINED_TORTURE,
                &[
                    "table",
                    "virtualized",
                    "retained",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_AI_TRANSCRIPT_TORTURE,
                "AI Transcript (Torture)",
                "AI Transcript / Long Conversation Harness",
                "fret-ui-ai (conversation surface)",
                CMD_NAV_AI_TRANSCRIPT_TORTURE,
                &[
                    "ai",
                    "chat",
                    "conversation",
                    "scroll",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_AI_CHAT_DEMO,
                "AI Chat (Demo)",
                "AI Chat / Conversation + PromptInput Demo",
                "fret-ui-ai (chat surfaces)",
                CMD_NAV_AI_CHAT_DEMO,
                &[
                    "ai",
                    "chat",
                    "conversation",
                    "prompt",
                    "input",
                    "interaction",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_CONVERSATION_DEMO,
                "AI Conversation (Demo)",
                "AI Elements Conversation / Transcript Surface Demo",
                "fret-ui-ai (conversation transcript)",
                CMD_NAV_AI_CONVERSATION_DEMO,
                &[
                    "ai",
                    "conversation",
                    "transcript",
                    "scroll",
                    "virtualized",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_MESSAGE_DEMO,
                "AI Message (Demo)",
                "AI Elements Message / Bubble + Actions Demo",
                "fret-ui-ai (message building blocks)",
                CMD_NAV_AI_MESSAGE_DEMO,
                &["ai", "message", "bubble", "actions", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CONTEXT_DEMO,
                "AI Context (Demo)",
                "AI Elements Context / Context Usage HoverCard Demo",
                "fret-ui-ai (context hovercard)",
                CMD_NAV_AI_CONTEXT_DEMO,
                &["ai", "context", "tokens", "progress", "hovercard", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TERMINAL_DEMO,
                "AI Terminal (Demo)",
                "AI Elements Terminal / Output Viewer Demo",
                "fret-ui-ai (terminal viewer)",
                CMD_NAV_AI_TERMINAL_DEMO,
                &[
                    "ai", "terminal", "output", "copy", "clear", "scroll", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PACKAGE_INFO_DEMO,
                "AI PackageInfo (Demo)",
                "AI Elements PackageInfo / Package Versions Demo",
                "fret-ui-ai (package info)",
                CMD_NAV_AI_PACKAGE_INFO_DEMO,
                &["ai", "package", "versions", "dependencies", "badge", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_OPEN_IN_CHAT_DEMO,
                "AI OpenIn (Demo)",
                "AI Elements OpenIn / Open in Chat Providers Demo",
                "fret-ui-ai (open in chat menu)",
                CMD_NAV_AI_OPEN_IN_CHAT_DEMO,
                &["ai", "open", "chat", "menu", "providers", "url", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TASK_DEMO,
                "AI Task (Demo)",
                "AI Elements Task / Collapsible Task Demo",
                "fret-ui-ai (task collapsible)",
                CMD_NAV_AI_TASK_DEMO,
                &["ai", "task", "collapsible", "search", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_AUDIO_PLAYER_DEMO,
                "AI Audio Player (Demo)",
                "AI Elements AudioPlayer / Media Controls Chrome Demo",
                "fret-ui-ai (audio player chrome)",
                CMD_NAV_AI_AUDIO_PLAYER_DEMO,
                &["ai", "audio", "player", "seek", "mute", "volume", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TRANSCRIPTION_DEMO,
                "AI Transcription (Demo)",
                "AI Elements Transcription / Interactive Transcript Demo",
                "fret-ui-ai (transcription)",
                CMD_NAV_AI_TRANSCRIPTION_DEMO,
                &["ai", "transcription", "segments", "seek", "voice", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SPEECH_INPUT_DEMO,
                "AI Speech Input (Demo)",
                "AI Elements SpeechInput / Docs-Aligned Voice Input Demo",
                "fret-ui-ai (speech input)",
                CMD_NAV_AI_SPEECH_INPUT_DEMO,
                &["ai", "speech", "voice", "input", "record", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_MIC_SELECTOR_DEMO,
                "AI Mic Selector (Demo)",
                "AI Elements MicSelector / Popover + Search Demo",
                "fret-ui-ai (voice input chrome)",
                CMD_NAV_AI_MIC_SELECTOR_DEMO,
                &[
                    "ai",
                    "mic",
                    "microphone",
                    "selector",
                    "popover",
                    "search",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_VOICE_SELECTOR_DEMO,
                "AI Voice Selector (Demo)",
                "AI Elements VoiceSelector / Dialog + Search Demo",
                "fret-ui-ai (voice input chrome)",
                CMD_NAV_AI_VOICE_SELECTOR_DEMO,
                &["ai", "voice", "selector", "dialog", "search", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_AGENT_DEMO,
                "AI Agent (Demo)",
                "AI Elements Agent / Instructions + Tools + Output Schema Demo",
                "fret-ui-ai (agent chrome)",
                CMD_NAV_AI_AGENT_DEMO,
                &["ai", "agent", "tools", "schema", "accordion", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SANDBOX_DEMO,
                "AI Sandbox (Demo)",
                "AI Elements Sandbox / Collapsible + Tabs Demo",
                "fret-ui-ai (sandbox chrome)",
                CMD_NAV_AI_SANDBOX_DEMO,
                &["ai", "sandbox", "collapsible", "tabs", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_PERSONA_DEMO,
                "AI Persona (Demo)",
                "AI Elements Persona / Docs-aligned Placeholder + Custom Visual Slot",
                "fret-ui-ai (persona surface)",
                CMD_NAV_AI_PERSONA_DEMO,
                &["ai", "persona", "visual", "variants", "docs", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_CHROME_DEMO,
                "AI Workflow Chrome (Demo)",
                "AI Elements workflow wrappers / Panel + Toolbar Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_CHROME_DEMO,
                &["ai", "workflow", "panel", "toolbar", "chrome", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_CANVAS_DEMO,
                "AI Workflow Canvas (Demo)",
                "AI Elements workflow Canvas / Host Surface Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_CANVAS_DEMO,
                &["ai", "workflow", "canvas", "pan", "zoom", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_NODE_DEMO,
                "AI Workflow Node (Demo)",
                "AI Elements workflow Node / Chrome Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_NODE_DEMO,
                &["ai", "workflow", "node", "handles", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_EDGE_DEMO,
                "AI Workflow Edge (Demo)",
                "AI Elements workflow Edge / Dashed + Animated Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_EDGE_DEMO,
                &["ai", "workflow", "edge", "dash", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_CONNECTION_DEMO,
                "AI Workflow Connection (Demo)",
                "AI Elements workflow Connection / Line Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_CONNECTION_DEMO,
                &["ai", "workflow", "connection", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_CONTROLS_DEMO,
                "AI Workflow Controls (Demo)",
                "AI Elements workflow Controls / Button Stack Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_CONTROLS_DEMO,
                &["ai", "workflow", "controls", "buttons", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_PANEL_DEMO,
                "AI Workflow Panel (Demo)",
                "AI Elements workflow Panel / Container Chrome Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_PANEL_DEMO,
                &["ai", "workflow", "panel", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_TOOLBAR_DEMO,
                "AI Workflow Toolbar (Demo)",
                "AI Elements workflow Toolbar / Row Chrome Demo",
                "fret-ui-ai (workflow chrome)",
                CMD_NAV_AI_WORKFLOW_TOOLBAR_DEMO,
                &["ai", "workflow", "toolbar", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_WORKFLOW_NODE_GRAPH_DEMO,
                "AI Workflow Node Graph (Demo)",
                "Workflow editor surface / fret-node engine + fret-ui-ai chrome Demo",
                "fret-node + fret-ui-ai",
                CMD_NAV_AI_WORKFLOW_NODE_GRAPH_DEMO,
                &[
                    "ai", "workflow", "node", "graph", "canvas", "controls", "engine", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_CANVAS_WORLD_LAYER_SPIKE,
                "AI Canvas World Layer (Spike)",
                "Canvas world layer / nodes as element subtrees (pan/zoom)",
                "fret-canvas/ui (world layer helper)",
                CMD_NAV_AI_CANVAS_WORLD_LAYER_SPIKE,
                &[
                    "ai", "workflow", "canvas", "pan", "zoom", "world", "nodes", "spike", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PROMPT_INPUT_PROVIDER_DEMO,
                "AI PromptInput Provider (Demo)",
                "PromptInputProvider + PromptInputRoot (parts) Demo",
                "fret-ui-ai (prompt input parts)",
                CMD_NAV_AI_PROMPT_INPUT_PROVIDER_DEMO,
                &["ai", "prompt", "input", "provider", "parts", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_PROMPT_INPUT_DOCS_DEMO,
                "AI Prompt Input (Docs-aligned)",
                "Docs-aligned PromptInput composition (tools + model select + tooltips) Demo",
                "fret-ui-ai (prompt input)",
                CMD_NAV_AI_PROMPT_INPUT_DOCS_DEMO,
                &[
                    "ai", "prompt", "input", "docs", "tooltips", "select", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PROMPT_INPUT_ACTION_MENU_DEMO,
                "AI PromptInput Action Menu (Demo)",
                "PromptInputActionMenu (DropdownMenu) Demo",
                "fret-ui-ai (prompt input action menu)",
                CMD_NAV_AI_PROMPT_INPUT_ACTION_MENU_DEMO,
                &[
                    "ai", "prompt", "input", "menu", "dropdown", "actions", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO,
                "AI PromptInput Referenced Sources (Demo)",
                "PromptInput referenced sources (chips) Demo",
                "fret-ui-ai (prompt input referenced sources)",
                CMD_NAV_AI_PROMPT_INPUT_REFERENCED_SOURCES_DEMO,
                &[
                    "ai",
                    "prompt",
                    "input",
                    "referenced",
                    "sources",
                    "chips",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_INLINE_CITATION_DEMO,
                "AI Inline Citation (Demo)",
                "AI Elements InlineCitation / HoverCard + Pager Demo",
                "fret-ui-ai (sources)",
                CMD_NAV_AI_INLINE_CITATION_DEMO,
                &["ai", "inline", "citation", "hovercard", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SOURCES_DEMO,
                "AI Sources (Demo)",
                "AI Elements Sources / Collapsible List Demo",
                "fret-ui-ai (sources)",
                CMD_NAV_AI_SOURCES_DEMO,
                &["ai", "sources", "collapsible", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_ARTIFACT_DEMO,
                "AI Artifact (Demo)",
                "AI Elements Artifact / Header + Content + Actions Demo",
                "fret-ui-ai (artifact container)",
                CMD_NAV_AI_ARTIFACT_DEMO,
                &["ai", "artifact", "header", "actions", "tooltip", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SHIMMER_DEMO,
                "AI Shimmer (Demo)",
                "AI Elements Shimmer / Animated Text Demo",
                "fret-ui-ai (chatbot utility)",
                CMD_NAV_AI_SHIMMER_DEMO,
                &["ai", "shimmer", "loading", "text", "animation", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_REASONING_DEMO,
                "AI Reasoning (Demo)",
                "AI Elements Reasoning / Auto-open + Auto-close Disclosure Demo",
                "fret-ui-ai (chatbot utility)",
                CMD_NAV_AI_REASONING_DEMO,
                &[
                    "ai",
                    "reasoning",
                    "collapsible",
                    "markdown",
                    "timer",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_QUEUE_DEMO,
                "AI Queue (Demo)",
                "AI Elements Queue / Sections + Item Actions Demo",
                "fret-ui-ai (queue surface)",
                CMD_NAV_AI_QUEUE_DEMO,
                &[
                    "ai",
                    "queue",
                    "collapsible",
                    "scroll",
                    "actions",
                    "attachments",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_ATTACHMENTS_DEMO,
                "AI Attachments (Demo)",
                "AI Elements Attachments",
                "fret-ui-ai (attachments surface)",
                CMD_NAV_AI_ATTACHMENTS_DEMO,
                &["ai", "attachments", "remove", "hover", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SUGGESTIONS_DEMO,
                "AI Suggestions (Demo)",
                "AI Elements Suggestions / Horizontal Pills Demo",
                "fret-ui-ai (chatbot utility)",
                CMD_NAV_AI_SUGGESTIONS_DEMO,
                &["ai", "suggestion", "suggestions", "scroll", "chips", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_MESSAGE_BRANCH_DEMO,
                "AI Message Branch (Demo)",
                "AI Elements MessageBranch / Alternate Assistant Outputs Demo",
                "fret-ui-ai (message surface)",
                CMD_NAV_AI_MESSAGE_BRANCH_DEMO,
                &["ai", "message", "branch", "selector", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_FILE_TREE_DEMO,
                "AI File Tree (Demo)",
                "AI Elements FileTree / Nested Collapsible Demo",
                "fret-ui-ai (file tree surface)",
                CMD_NAV_AI_FILE_TREE_DEMO,
                &["ai", "file", "tree", "outline", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CODE_BLOCK_DEMO,
                "AI Code Block (Demo)",
                "AI Elements CodeBlock / Composable Header + Copy Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_CODE_BLOCK_DEMO,
                &["ai", "code", "block", "copy", "docs", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SNIPPET_DEMO,
                "AI Snippet (Demo)",
                "AI Elements Snippet / Inline Copy Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_SNIPPET_DEMO,
                &["ai", "snippet", "copy", "code", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_COMMIT_DEMO,
                "AI Commit (Demo)",
                "AI Elements Commit Disclosure Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_COMMIT_DEMO,
                &["ai", "commit", "git", "copy", "diff", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_COMMIT_LARGE_DEMO,
                "AI Commit Large (Demo)",
                "Commit Large List (scroll + click seams)",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_COMMIT_LARGE_DEMO,
                &["ai", "commit", "large", "scroll", "files", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_STACK_TRACE_DEMO,
                "AI Stack Trace (Demo)",
                "AI Elements StackTrace / Parsed Frames Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_STACK_TRACE_DEMO,
                &["ai", "stack", "trace", "error", "copy", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_STACK_TRACE_LARGE_DEMO,
                "AI Stack Trace Large (Demo)",
                "StackTrace Large List (scroll + click seams)",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_STACK_TRACE_LARGE_DEMO,
                &["ai", "stack", "trace", "large", "scroll", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TEST_RESULTS_DEMO,
                "AI Test Results (Demo)",
                "AI Elements TestResults / Suite & Test Rows Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_TEST_RESULTS_DEMO,
                &["ai", "test", "results", "suite", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TEST_RESULTS_LARGE_DEMO,
                "AI Test Results Large (Demo)",
                "TestResults Large List (scroll + click seams)",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_TEST_RESULTS_LARGE_DEMO,
                &["ai", "test", "results", "large", "scroll", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CHECKPOINT_DEMO,
                "Checkpoint",
                "A simple component for marking conversation history points and restoring the chat to a previous state.",
                "fret-ui-ai (chatbot chrome)",
                CMD_NAV_AI_CHECKPOINT_DEMO,
                &["ai", "checkpoint", "tooltip", "chatbot", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CONFIRMATION_DEMO,
                "AI Confirmation (Demo)",
                "AI Elements Confirmation / Approval Request Demo",
                "fret-ui-ai (tooling chrome)",
                CMD_NAV_AI_CONFIRMATION_DEMO,
                &["ai", "confirmation", "approval", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_ENVIRONMENT_VARIABLES_DEMO,
                "AI Environment Variables (Demo)",
                "AI Elements EnvironmentVariables / Show-Hide + Copy Demo",
                "fret-ui-ai (tooling chrome)",
                CMD_NAV_AI_ENVIRONMENT_VARIABLES_DEMO,
                &[
                    "ai",
                    "environment",
                    "variables",
                    "env",
                    "toggle",
                    "copy",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PLAN_DEMO,
                "AI Plan (Demo)",
                "AI Elements Plan / Collapsible Demo",
                "fret-ui-ai (tooling chrome)",
                CMD_NAV_AI_PLAN_DEMO,
                &["ai", "plan", "collapsible", "streaming", "shimmer", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TOOL_DEMO,
                "AI Tool (Demo)",
                "AI Elements Tool / Collapsible Tool Call Demo",
                "fret-ui-ai (tool call chrome)",
                CMD_NAV_AI_TOOL_DEMO,
                &[
                    "ai",
                    "tool",
                    "tool-call",
                    "collapsible",
                    "code-block",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_WEB_PREVIEW_DEMO,
                "AI Web Preview (Demo)",
                "AI Elements WebPreview / URL + Console Chrome Demo",
                "fret-ui-ai (web preview chrome)",
                CMD_NAV_AI_WEB_PREVIEW_DEMO,
                &["ai", "web", "preview", "url", "console", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_MODEL_SELECTOR_DEMO,
                "AI Model Selector (Demo)",
                "AI Elements ModelSelector / Command Palette-in-Dialog Demo",
                "fret-ui-ai (chatbot)",
                CMD_NAV_AI_MODEL_SELECTOR_DEMO,
                &["ai", "model", "selector", "command", "dialog", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CHAIN_OF_THOUGHT_DEMO,
                "AI Chain of Thought (Demo)",
                "AI Elements ChainOfThought / Collapsible Steps Demo",
                "fret-ui-ai (chatbot)",
                CMD_NAV_AI_CHAIN_OF_THOUGHT_DEMO,
                &["ai", "chain", "thought", "steps", "collapsible", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SCHEMA_DISPLAY_DEMO,
                "AI Schema Display (Demo)",
                "AI Elements SchemaDisplay / OpenAPI-ish Viewer Demo",
                "fret-ui-ai (schema display)",
                CMD_NAV_AI_SCHEMA_DISPLAY_DEMO,
                &["ai", "schema", "openapi", "json", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_IMAGE_DEMO,
                "AI Image (Demo)",
                "AI Elements Image / Generated Image Surface Demo",
                "fret-ui-ai (utilities)",
                CMD_NAV_AI_IMAGE_DEMO,
                &["ai", "image", "media", "demo"],
            ),
            PageSpec::new(
                PAGE_INSPECTOR_TORTURE,
                "Inspector (Torture)",
                "Inspector / Property List Harness",
                "virtualized property list (retained host)",
                CMD_NAV_INSPECTOR_TORTURE,
                &[
                    "inspector",
                    "properties",
                    "outline",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
            PageSpec::new(
                PAGE_FILE_TREE_TORTURE,
                "File Tree (Torture)",
                "File Tree / Outline Harness",
                "virtualized tree rows (retained host)",
                CMD_NAV_FILE_TREE_TORTURE,
                &[
                    "file",
                    "tree",
                    "outline",
                    "virtualized",
                    "performance",
                    "gpui-parity",
                    "harness",
                ],
            ),
        ],
    },
    #[cfg(all(feature = "gallery-ai", not(feature = "gallery-dev")))]
    PageGroupSpec {
        title: "AI Elements",
        items: &[
            PageSpec::new(
                PAGE_AI_CONVERSATION_DEMO,
                "AI Conversation (Demo)",
                "AI Elements Conversation / Transcript Surface Demo",
                "fret-ui-ai (conversation transcript)",
                CMD_NAV_AI_CONVERSATION_DEMO,
                &[
                    "ai",
                    "conversation",
                    "transcript",
                    "scroll",
                    "virtualized",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_MESSAGE_DEMO,
                "AI Message (Demo)",
                "AI Elements Message / Bubble + Actions Demo",
                "fret-ui-ai (message building blocks)",
                CMD_NAV_AI_MESSAGE_DEMO,
                &["ai", "message", "bubble", "actions", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CONTEXT_DEMO,
                "AI Context (Demo)",
                "AI Elements Context / Context Usage HoverCard Demo",
                "fret-ui-ai (context hovercard)",
                CMD_NAV_AI_CONTEXT_DEMO,
                &["ai", "context", "tokens", "progress", "hovercard", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TERMINAL_DEMO,
                "AI Terminal (Demo)",
                "AI Elements Terminal / Output Viewer Demo",
                "fret-ui-ai (terminal viewer)",
                CMD_NAV_AI_TERMINAL_DEMO,
                &[
                    "ai", "terminal", "output", "copy", "clear", "scroll", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PACKAGE_INFO_DEMO,
                "AI PackageInfo (Demo)",
                "AI Elements PackageInfo / Package Versions Demo",
                "fret-ui-ai (package info)",
                CMD_NAV_AI_PACKAGE_INFO_DEMO,
                &["ai", "package", "versions", "dependencies", "badge", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_OPEN_IN_CHAT_DEMO,
                "AI OpenIn (Demo)",
                "AI Elements OpenIn / Open in Chat Providers Demo",
                "fret-ui-ai (open in chat menu)",
                CMD_NAV_AI_OPEN_IN_CHAT_DEMO,
                &["ai", "open", "chat", "menu", "providers", "url", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TASK_DEMO,
                "AI Task (Demo)",
                "AI Elements Task / Collapsible Task Demo",
                "fret-ui-ai (task collapsible)",
                CMD_NAV_AI_TASK_DEMO,
                &["ai", "task", "collapsible", "search", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_ARTIFACT_DEMO,
                "AI Artifact (Demo)",
                "AI Elements Artifact / Header + Content + Actions Demo",
                "fret-ui-ai (artifact container)",
                CMD_NAV_AI_ARTIFACT_DEMO,
                &["ai", "artifact", "header", "actions", "tooltip", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SHIMMER_DEMO,
                "AI Shimmer (Demo)",
                "AI Elements Shimmer / Animated Text Demo",
                "fret-ui-ai (chatbot utility)",
                CMD_NAV_AI_SHIMMER_DEMO,
                &["ai", "shimmer", "loading", "text", "animation", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_REASONING_DEMO,
                "AI Reasoning (Demo)",
                "AI Elements Reasoning / Auto-open + Auto-close Disclosure Demo",
                "fret-ui-ai (chatbot utility)",
                CMD_NAV_AI_REASONING_DEMO,
                &[
                    "ai",
                    "reasoning",
                    "collapsible",
                    "markdown",
                    "timer",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_CODE_BLOCK_DEMO,
                "AI Code Block (Demo)",
                "AI Elements CodeBlock / Composable Header + Copy Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_CODE_BLOCK_DEMO,
                &["ai", "code", "block", "copy", "docs", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_COMMIT_DEMO,
                "AI Commit (Demo)",
                "AI Elements Commit Disclosure Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_COMMIT_DEMO,
                &["ai", "commit", "git", "copy", "diff", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_STACK_TRACE_DEMO,
                "AI Stack Trace (Demo)",
                "AI Elements StackTrace / Parsed Frames Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_STACK_TRACE_DEMO,
                &["ai", "stack", "trace", "error", "copy", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_TEST_RESULTS_DEMO,
                "AI Test Results (Demo)",
                "AI Elements TestResults / Suite & Test Rows Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_TEST_RESULTS_DEMO,
                &["ai", "test", "results", "suite", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_CONFIRMATION_DEMO,
                "AI Confirmation (Demo)",
                "AI Elements Confirmation / Approval Request Demo",
                "fret-ui-ai (tooling chrome)",
                CMD_NAV_AI_CONFIRMATION_DEMO,
                &["ai", "confirmation", "approval", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_ENVIRONMENT_VARIABLES_DEMO,
                "AI Environment Variables (Demo)",
                "AI Elements EnvironmentVariables / Show-Hide + Copy Demo",
                "fret-ui-ai (tooling chrome)",
                CMD_NAV_AI_ENVIRONMENT_VARIABLES_DEMO,
                &[
                    "ai",
                    "environment",
                    "variables",
                    "env",
                    "toggle",
                    "copy",
                    "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_PLAN_DEMO,
                "AI Plan (Demo)",
                "AI Elements Plan / Collapsible Demo",
                "fret-ui-ai (tooling chrome)",
                CMD_NAV_AI_PLAN_DEMO,
                &["ai", "plan", "collapsible", "streaming", "shimmer", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_PROMPT_INPUT_DOCS_DEMO,
                "AI Prompt Input (Docs-aligned)",
                "Docs-aligned PromptInput composition (tools + model select + tooltips) Demo",
                "fret-ui-ai (prompt input)",
                CMD_NAV_AI_PROMPT_INPUT_DOCS_DEMO,
                &[
                    "ai", "prompt", "input", "docs", "tooltips", "select", "demo",
                ],
            ),
            PageSpec::new(
                PAGE_AI_CHAIN_OF_THOUGHT_DEMO,
                "AI Chain of Thought (Demo)",
                "AI Elements ChainOfThought / Collapsible Steps Demo",
                "fret-ui-ai (chatbot)",
                CMD_NAV_AI_CHAIN_OF_THOUGHT_DEMO,
                &["ai", "chain", "thought", "steps", "collapsible", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SCHEMA_DISPLAY_DEMO,
                "AI Schema Display (Demo)",
                "AI Elements SchemaDisplay / OpenAPI-ish Viewer Demo",
                "fret-ui-ai (schema display)",
                CMD_NAV_AI_SCHEMA_DISPLAY_DEMO,
                &["ai", "schema", "openapi", "json", "demo"],
            ),
            PageSpec::new(
                PAGE_AI_SNIPPET_DEMO,
                "AI Snippet (Demo)",
                "AI Elements Snippet / Inline Copy Demo",
                "fret-ui-ai (code artifacts)",
                CMD_NAV_AI_SNIPPET_DEMO,
                &["ai", "snippet", "copy", "code", "demo"],
            ),
        ],
    },
    PageGroupSpec {
        title: "Shadcn",
        items: &[
            PageSpec::new(
                PAGE_ACCORDION,
                "Accordion",
                "Accordion",
                "fret-ui-shadcn",
                CMD_NAV_ACCORDION,
                &["accordion", "collapsible"],
            ),
            PageSpec::new(
                PAGE_ALERT_DIALOG,
                "Alert Dialog",
                "Alert Dialog",
                "fret-ui-shadcn",
                CMD_NAV_ALERT_DIALOG,
                &["alert_dialog", "dialog", "overlay"],
            ),
            PageSpec::new(
                PAGE_ALERT,
                "Alert",
                "Alert",
                "fret-ui-shadcn",
                CMD_NAV_ALERT,
                &["alert", "feedback"],
            ),
            PageSpec::new(
                PAGE_ASPECT_RATIO,
                "Aspect Ratio",
                "Aspect Ratio",
                "fret-ui-shadcn",
                CMD_NAV_ASPECT_RATIO,
                &["aspect_ratio", "layout"],
            ),
            PageSpec::new(
                PAGE_AVATAR,
                "Avatar",
                "Avatar",
                "fret-ui-shadcn",
                CMD_NAV_AVATAR,
                &["avatar", "image", "fallback"],
            ),
            PageSpec::new(
                PAGE_BADGE,
                "Badge",
                "Badge",
                "fret-ui-shadcn",
                CMD_NAV_BADGE,
                &["badge", "status", "tag"],
            ),
            PageSpec::new(
                PAGE_BREADCRUMB,
                "Breadcrumb",
                "Breadcrumb",
                "fret-ui-shadcn",
                CMD_NAV_BREADCRUMB,
                &["breadcrumb", "navigation"],
            ),
            PageSpec::new(
                PAGE_BUTTON_GROUP,
                "Button Group",
                "Button Group",
                "fret-ui-shadcn",
                CMD_NAV_BUTTON_GROUP,
                &["button", "group"],
            ),
            PageSpec::new(
                PAGE_BUTTON,
                "Button",
                "Button",
                "fret-ui-shadcn",
                CMD_NAV_BUTTON,
                &["button", "variant"],
            ),
            PageSpec::new(
                PAGE_CALENDAR,
                "Calendar",
                "Calendar",
                "fret-ui-shadcn",
                CMD_NAV_CALENDAR,
                &["calendar", "date"],
            ),
            PageSpec::new(
                PAGE_CARD,
                "Card",
                "Card",
                "fret-ui-shadcn",
                CMD_NAV_CARD,
                &["card", "layout", "surface"],
            ),
            PageSpec::new(
                PAGE_CAROUSEL,
                "Carousel",
                "Carousel",
                "fret-ui-shadcn",
                CMD_NAV_CAROUSEL,
                &["carousel", "scroll"],
            ),
            PageSpec::new(
                PAGE_CHECKBOX,
                "Checkbox",
                "Checkbox",
                "fret-ui-shadcn",
                CMD_NAV_CHECKBOX,
                &["checkbox", "input"],
            ),
            PageSpec::new(
                PAGE_COLLAPSIBLE,
                "Collapsible",
                "Collapsible",
                "fret-ui-shadcn",
                CMD_NAV_COLLAPSIBLE,
                &["collapsible", "disclosure"],
            ),
            PageSpec::new(
                PAGE_COMBOBOX,
                "Combobox",
                "Combobox",
                "fret-ui-shadcn",
                CMD_NAV_COMBOBOX,
                &["combobox", "cmdk", "search"],
            ),
            PageSpec::new(
                PAGE_COMMAND,
                "Command",
                "Command",
                "fret-ui-shadcn",
                CMD_NAV_COMMAND,
                &["cmdk", "command"],
            ),
            PageSpec::new(
                PAGE_CONTEXT_MENU,
                "Context Menu",
                "Context Menu",
                "fret-ui-shadcn",
                CMD_NAV_CONTEXT_MENU,
                &["context_menu", "menu"],
            ),
            PageSpec::new(
                PAGE_DATA_TABLE,
                "Data Table",
                "Data Table",
                "fret-ui-shadcn + fret-ui-headless",
                CMD_NAV_DATA_TABLE,
                &["table", "virtualized", "tanstack"],
            ),
            PageSpec::new(
                PAGE_DATE_PICKER,
                "Date Picker",
                "Date Picker",
                "fret-ui-shadcn",
                CMD_NAV_DATE_PICKER,
                &["date", "calendar", "popover"],
            ),
            PageSpec::new(
                PAGE_DIALOG,
                "Dialog",
                "Dialog",
                "fret-ui-shadcn",
                CMD_NAV_DIALOG,
                &["dialog", "overlay"],
            ),
            PageSpec::new(
                PAGE_DRAWER,
                "Drawer",
                "Drawer",
                "fret-ui-shadcn",
                CMD_NAV_DRAWER,
                &["drawer", "overlay"],
            ),
            PageSpec::new(
                PAGE_DROPDOWN_MENU,
                "Dropdown Menu",
                "Dropdown Menu",
                "fret-ui-shadcn",
                CMD_NAV_DROPDOWN_MENU,
                &["dropdown_menu", "menu"],
            ),
            PageSpec::new(
                PAGE_EMPTY,
                "Empty",
                "Empty",
                "fret-ui-shadcn",
                CMD_NAV_EMPTY,
                &["empty", "state"],
            ),
            PageSpec::new(
                PAGE_FIELD,
                "Field",
                "Field",
                "fret-ui-shadcn",
                CMD_NAV_FIELD,
                &["field", "form", "label", "error"],
            ),
            PageSpec::new(
                PAGE_HOVER_CARD,
                "Hover Card",
                "Hover Card",
                "fret-ui-shadcn",
                CMD_NAV_HOVER_CARD,
                &["hover_card", "overlay"],
            ),
            PageSpec::new(
                PAGE_INPUT_GROUP,
                "Input Group",
                "Input Group",
                "fret-ui-shadcn",
                CMD_NAV_INPUT_GROUP,
                &["input", "group"],
            ),
            PageSpec::new(
                PAGE_INPUT_OTP,
                "Input OTP",
                "Input OTP",
                "fret-ui-shadcn",
                CMD_NAV_INPUT_OTP,
                &["input", "otp"],
            ),
            PageSpec::new(
                PAGE_INPUT,
                "Input",
                "Input",
                "fret-ui-shadcn",
                CMD_NAV_INPUT,
                &["input", "text"],
            ),
            PageSpec::new(
                PAGE_ITEM,
                "Item",
                "Item",
                "fret-ui-shadcn",
                CMD_NAV_ITEM,
                &["item", "layout"],
            ),
            PageSpec::new(
                PAGE_KBD,
                "Kbd",
                "Kbd",
                "fret-ui-shadcn",
                CMD_NAV_KBD,
                &["kbd", "text"],
            ),
            PageSpec::new(
                PAGE_LABEL,
                "Label",
                "Label",
                "fret-ui-shadcn",
                CMD_NAV_LABEL,
                &["label", "form"],
            ),
            PageSpec::new(
                PAGE_MENUBAR,
                "Menubar",
                "Menubar",
                "fret-ui-shadcn",
                CMD_NAV_MENUBAR,
                &["menubar", "menu"],
            ),
            PageSpec::new(
                PAGE_NATIVE_SELECT,
                "Native Select",
                "Native Select",
                "fret-ui-shadcn",
                CMD_NAV_NATIVE_SELECT,
                &["native_select", "select"],
            ),
            PageSpec::new(
                PAGE_NAVIGATION_MENU,
                "Navigation Menu",
                "Navigation Menu",
                "fret-ui-shadcn",
                CMD_NAV_NAVIGATION_MENU,
                &["navigation_menu", "menu"],
            ),
            PageSpec::new(
                PAGE_PAGINATION,
                "Pagination",
                "Pagination",
                "fret-ui-shadcn",
                CMD_NAV_PAGINATION,
                &["pagination"],
            ),
            PageSpec::new(
                PAGE_POPOVER,
                "Popover",
                "Popover",
                "fret-ui-shadcn",
                CMD_NAV_POPOVER,
                &["popover", "overlay"],
            ),
            PageSpec::new(
                PAGE_PROGRESS,
                "Progress",
                "Progress",
                "fret-ui-shadcn",
                CMD_NAV_PROGRESS,
                &["progress"],
            ),
            PageSpec::new(
                PAGE_RADIO_GROUP,
                "Radio Group",
                "Radio Group",
                "fret-ui-shadcn",
                CMD_NAV_RADIO_GROUP,
                &["radio", "group"],
            ),
            PageSpec::new(
                PAGE_RESIZABLE,
                "Resizable",
                "Resizable",
                "fret-ui-shadcn",
                CMD_NAV_RESIZABLE,
                &["split", "panel", "resize"],
            ),
            PageSpec::new(
                PAGE_SCROLL_AREA,
                "Scroll Area",
                "Scroll Area",
                "fret-ui-shadcn",
                CMD_NAV_SCROLL_AREA,
                &["scroll", "scrollbar", "virtual"],
            ),
            PageSpec::new(
                PAGE_SELECT,
                "Select",
                "Select",
                "fret-ui-shadcn",
                CMD_NAV_SELECT,
                &["select", "popover", "listbox"],
            ),
            PageSpec::new(
                PAGE_SEPARATOR,
                "Separator",
                "Separator",
                "fret-ui-shadcn",
                CMD_NAV_SEPARATOR,
                &["separator"],
            ),
            PageSpec::new(
                PAGE_SHEET,
                "Sheet",
                "Sheet",
                "fret-ui-shadcn",
                CMD_NAV_SHEET,
                &["sheet", "overlay"],
            ),
            PageSpec::new(
                PAGE_SIDEBAR,
                "Sidebar",
                "Sidebar",
                "fret-ui-shadcn",
                CMD_NAV_SIDEBAR,
                &["sidebar", "navigation"],
            ),
            PageSpec::new(
                PAGE_SKELETON,
                "Skeleton",
                "Skeleton",
                "fret-ui-shadcn",
                CMD_NAV_SKELETON,
                &["skeleton", "loading", "animation"],
            ),
            PageSpec::new(
                PAGE_SLIDER,
                "Slider",
                "Slider",
                "fret-ui-shadcn",
                CMD_NAV_SLIDER,
                &["slider", "range", "input"],
            ),
            PageSpec::new(
                PAGE_SONNER,
                "Sonner",
                "Sonner",
                "fret-ui-shadcn",
                CMD_NAV_SONNER,
                &["sonner", "toast"],
            ),
            PageSpec::new(
                PAGE_SPINNER,
                "Spinner",
                "Spinner",
                "fret-ui-shadcn",
                CMD_NAV_SPINNER,
                &["spinner", "loading"],
            ),
            PageSpec::new(
                PAGE_SWITCH,
                "Switch",
                "Switch",
                "fret-ui-shadcn",
                CMD_NAV_SWITCH,
                &["switch", "input"],
            ),
            PageSpec::new(
                PAGE_TABLE,
                "Table",
                "Table",
                "fret-ui-shadcn",
                CMD_NAV_TABLE,
                &["table", "grid"],
            ),
            PageSpec::new(
                PAGE_TABS,
                "Tabs",
                "Tabs",
                "fret-ui-shadcn",
                CMD_NAV_TABS,
                &["tabs", "roving", "focus"],
            ),
            PageSpec::new(
                PAGE_TEXTAREA,
                "Textarea",
                "Textarea",
                "fret-ui-shadcn",
                CMD_NAV_TEXTAREA,
                &["textarea", "input"],
            ),
            PageSpec::new(
                PAGE_TOAST,
                "Toast",
                "Toast",
                "fret-ui-shadcn",
                CMD_NAV_TOAST,
                &["sonner", "toast"],
            ),
            PageSpec::new(
                PAGE_TOGGLE_GROUP,
                "Toggle Group",
                "Toggle Group",
                "fret-ui-shadcn",
                CMD_NAV_TOGGLE_GROUP,
                &["toggle_group"],
            ),
            PageSpec::new(
                PAGE_TOGGLE,
                "Toggle",
                "Toggle",
                "fret-ui-shadcn",
                CMD_NAV_TOGGLE,
                &["toggle"],
            ),
            PageSpec::new(
                PAGE_TOOLTIP,
                "Tooltip",
                "Tooltip",
                "fret-ui-shadcn",
                CMD_NAV_TOOLTIP,
                &["tooltip", "overlay", "hover"],
            ),
            PageSpec::new(
                PAGE_TYPOGRAPHY,
                "Typography",
                "Typography",
                "fret-ui-shadcn",
                CMD_NAV_TYPOGRAPHY,
                &["typography", "text"],
            ),
        ],
    },
    #[cfg(any(feature = "gallery-dev", feature = "gallery-chart"))]
    PageGroupSpec {
        title: "Shadcn (Chart)",
        items: &[PageSpec::new(
            PAGE_CHART,
            "Chart",
            "Chart",
            "fret-ui-shadcn",
            CMD_NAV_CHART,
            &["chart", "data_viz"],
        )],
    },
    #[cfg(feature = "gallery-dev")]
    PageGroupSpec {
        title: "Shadcn (Extras)",
        items: &[
            PageSpec::new(
                PAGE_SHADCN_EXTRAS,
                "Extras",
                "Shadcn Extras (blocks / recipes)",
                "fret-ui-shadcn extras",
                CMD_NAV_SHADCN_EXTRAS,
                &["extras", "blocks", "recipes", "kibo"],
            ),
            PageSpec::new(
                PAGE_DATA_GRID,
                "DataGrid",
                "DataGrid",
                "fret-ui-shadcn",
                CMD_NAV_DATA_GRID,
                &["grid", "viewport", "virtualized"],
            ),
            PageSpec::new(
                PAGE_FORMS,
                "Forms",
                "Inputs / TextArea / Checkbox / Switch",
                "fret-ui-shadcn",
                CMD_NAV_FORMS,
                &["input", "textarea", "checkbox", "switch"],
            ),
            PageSpec::new(
                PAGE_ICONS,
                "Icons",
                "Icons",
                "fret-icons + fret-icons-lucide",
                CMD_NAV_ICONS,
                &["icon", "svg", "lucide"],
            ),
            PageSpec::new(
                PAGE_MENUS,
                "Menus",
                "Menus (Dropdown / Context)",
                "fret-ui-shadcn",
                CMD_NAV_MENUS,
                &["dropdown", "context-menu"],
            ),
            PageSpec::new(
                PAGE_OVERLAY,
                "Overlay",
                "Overlay / Popover & Dialog",
                "Radix-shaped primitives",
                CMD_NAV_OVERLAY,
                &["dialog", "popover"],
            ),
            PageSpec::new(
                PAGE_IMAGE_OBJECT_FIT,
                "Image (Object Fit)",
                "Image / Object Fit",
                "SceneOp::Image + MediaImage",
                CMD_NAV_IMAGE_OBJECT_FIT,
                &[
                    "image",
                    "object_fit",
                    "cover",
                    "contain",
                    "stretch",
                    "thumbnail",
                    "streaming",
                ],
            ),
            PageSpec::new(
                PAGE_FORM,
                "Form",
                "Form",
                "fret-ui-shadcn",
                CMD_NAV_FORM,
                &["form", "field"],
            ),
        ],
    },
    #[cfg(feature = "gallery-dev")]
    PageGroupSpec {
        title: "Magic",
        items: &[
            PageSpec::new(
                PAGE_MAGIC_LENS,
                "Lens",
                "Lens (Phase 0)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_LENS,
                &["magic", "lens", "mask", "transform"],
            ),
            PageSpec::new(
                PAGE_MAGIC_MARQUEE,
                "Marquee",
                "Marquee (Phase 0)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_MARQUEE,
                &["magic", "marquee", "animation", "reduced-motion"],
            ),
            PageSpec::new(
                PAGE_MAGIC_CARD,
                "MagicCard",
                "MagicCard (Phase 0)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_CARD,
                &["magic", "card", "pointer-follow", "gradient"],
            ),
            PageSpec::new(
                PAGE_MAGIC_BORDER_BEAM,
                "BorderBeam",
                "BorderBeam (Phase 0)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_BORDER_BEAM,
                &["magic", "border", "beam", "glow", "blend"],
            ),
            PageSpec::new(
                PAGE_MAGIC_DOCK,
                "Dock",
                "Dock (Phase 0)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_DOCK,
                &["magic", "dock", "pointer", "magnify"],
            ),
            PageSpec::new(
                PAGE_MAGIC_PATTERNS,
                "Patterns",
                "Patterns (Tier B materials)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_PATTERNS,
                &["magic", "patterns", "materials", "tier-b"],
            ),
            PageSpec::new(
                PAGE_MAGIC_PATTERNS_TORTURE,
                "Patterns (Torture)",
                "Patterns (Tier B materials, fill-rate torture)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_PATTERNS_TORTURE,
                &[
                    "magic",
                    "patterns",
                    "materials",
                    "tier-b",
                    "performance",
                    "torture",
                ],
            ),
            PageSpec::new(
                PAGE_MAGIC_SPARKLES_TEXT,
                "SparklesText",
                "SparklesText (Phase 0)",
                "fret-ui-magic",
                CMD_NAV_MAGIC_SPARKLES_TEXT,
                &["magic", "sparkles", "text", "materials", "tier-b"],
            ),
            PageSpec::new(
                PAGE_MAGIC_BLOOM,
                "Bloom",
                "Bloom (Tier B recipe example)",
                "fret-ui-kit",
                CMD_NAV_MAGIC_BLOOM,
                &["bloom", "threshold", "blur", "blend"],
            ),
        ],
    },
    #[cfg(feature = "gallery-material3")]
    PageGroupSpec {
        title: "Material 3",
        items: &[
            PageSpec::new(
                PAGE_MATERIAL3_GALLERY,
                "Gallery",
                "Material 3 Gallery (outcomes-first snapshot surface)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_GALLERY,
                &["material3", "gallery", "regression", "outcomes"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TOP_APP_BAR,
                "Top App Bar",
                "Material 3 Top App Bar (primitives)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TOP_APP_BAR,
                &["material3", "top-app-bar", "toolbar", "app-bar"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_BOTTOM_SHEET,
                "Bottom Sheet",
                "Material 3 Bottom Sheet (modal + standard)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_BOTTOM_SHEET,
                &["material3", "bottom-sheet", "sheet", "overlay"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_DATE_PICKER,
                "Date Picker",
                "Material 3 Date Picker (modal + docked)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_DATE_PICKER,
                &["material3", "date-picker", "calendar", "overlay"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TIME_PICKER,
                "Time Picker",
                "Material 3 Time Picker (modal + docked)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TIME_PICKER,
                &["material3", "time-picker", "clock", "overlay"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_AUTOCOMPLETE,
                "Autocomplete",
                "Material 3 Autocomplete (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_AUTOCOMPLETE,
                &[
                    "material3",
                    "autocomplete",
                    "combobox",
                    "listbox",
                    "overlay",
                    "a11y",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_STATE_MATRIX,
                "State Matrix",
                "Material 3 State Matrix (manual regression harness)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_STATE_MATRIX,
                &["material3", "states", "regression", "matrix"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TOUCH_TARGETS,
                "Touch Targets",
                "Material 3 Touch Targets (minimum interactive size)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TOUCH_TARGETS,
                &[
                    "material3",
                    "touch-target",
                    "interactive-size",
                    "regression",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_BUTTON,
                "Button",
                "Material 3 Button (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_BUTTON,
                &["material3", "button", "state-layer", "ripple", "motion"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_ICON_BUTTON,
                "Icon Button",
                "Material 3 Icon Button (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_ICON_BUTTON,
                &[
                    "material3",
                    "icon-button",
                    "state-layer",
                    "ripple",
                    "motion",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_CHECKBOX,
                "Checkbox",
                "Material 3 Checkbox (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_CHECKBOX,
                &["material3", "checkbox", "state-layer", "ripple", "forms"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SWITCH,
                "Switch",
                "Material 3 Switch (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SWITCH,
                &["material3", "switch", "state-layer", "ripple", "forms"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SLIDER,
                "Slider",
                "Material 3 Slider (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SLIDER,
                &[
                    "material3",
                    "slider",
                    "state-layer",
                    "ripple",
                    "forms",
                    "value",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_RADIO,
                "Radio",
                "Material 3 Radio (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_RADIO,
                &["material3", "radio", "state-layer", "ripple", "forms"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_BADGE,
                "Badge",
                "Material 3 Badge (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_BADGE,
                &["material3", "badge", "status", "navigation"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SEGMENTED_BUTTON,
                "Segmented Button",
                "Material 3 Segmented Button (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SEGMENTED_BUTTON,
                &["material3", "segmented-button", "roving-focus", "selection"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SELECT,
                "Select",
                "Material 3 Select (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SELECT,
                &["material3", "select", "listbox", "forms", "overlay"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TEXT_FIELD,
                "Text Field",
                "Material 3 Text Field (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TEXT_FIELD,
                &["material3", "text-field", "forms"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TABS,
                "Tabs",
                "Material 3 Tabs (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TABS,
                &["material3", "tabs", "state-layer", "ripple", "roving-focus"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_NAVIGATION_BAR,
                "Navigation Bar",
                "Material 3 Navigation Bar (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_NAVIGATION_BAR,
                &[
                    "material3",
                    "navigation-bar",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                    "motion",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_NAVIGATION_RAIL,
                "Navigation Rail",
                "Material 3 Navigation Rail (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_NAVIGATION_RAIL,
                &[
                    "material3",
                    "navigation-rail",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                    "motion",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_NAVIGATION_DRAWER,
                "Navigation Drawer",
                "Material 3 Navigation Drawer (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_NAVIGATION_DRAWER,
                &[
                    "material3",
                    "navigation-drawer",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER,
                "Modal Navigation Drawer",
                "Material 3 Modal Navigation Drawer (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_MODAL_NAVIGATION_DRAWER,
                &[
                    "material3",
                    "navigation-drawer",
                    "modal",
                    "overlay",
                    "scrim",
                    "focus-trap",
                    "motion",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_DIALOG,
                "Dialog",
                "Material 3 Dialog (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_DIALOG,
                &[
                    "material3",
                    "dialog",
                    "modal",
                    "overlay",
                    "scrim",
                    "focus-trap",
                    "motion",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_MENU,
                "Menu",
                "Material 3 Menu (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_MENU,
                &[
                    "material3",
                    "menu",
                    "list",
                    "state-layer",
                    "ripple",
                    "roving-focus",
                    "typeahead",
                ],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_LIST,
                "List",
                "Material 3 List (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_LIST,
                &["material3", "list", "roving-focus", "selection"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_SNACKBAR,
                "Snackbar",
                "Material 3 Snackbar (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_SNACKBAR,
                &["material3", "snackbar", "toast-layer"],
            ),
            PageSpec::new(
                PAGE_MATERIAL3_TOOLTIP,
                "Tooltip",
                "Material 3 Tooltip (MVP)",
                "fret-ui-material3",
                CMD_NAV_MATERIAL3_TOOLTIP,
                &["material3", "tooltip", "overlay", "motion"],
            ),
        ],
    },
];

pub(crate) fn page_spec(id: &str) -> Option<&'static PageSpec> {
    PAGE_GROUPS
        .iter()
        .flat_map(|group| group.items.iter())
        .find(|item| item.id == id)
}

pub(crate) fn page_id_for_nav_command(command: &str) -> Option<&'static str> {
    static BY_COMMAND: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    let by_command = BY_COMMAND.get_or_init(|| {
        let mut map: HashMap<&'static str, &'static str> = HashMap::new();
        for group in PAGE_GROUPS {
            for page in group.items {
                map.insert(page.command, page.id);
            }
        }
        map
    });

    by_command.get(command).copied()
}

#[cfg(feature = "gallery-dev")]
pub(crate) fn data_grid_row_command(row: usize) -> Option<CommandId> {
    let row = u64::try_from(row).ok()?;
    Some(CommandId::new(format!("{CMD_DATA_GRID_ROW_PREFIX}{row}")))
}

#[cfg(feature = "gallery-dev")]
pub(crate) fn data_grid_row_for_command(command: &str) -> Option<u64> {
    let suffix = command.strip_prefix(CMD_DATA_GRID_ROW_PREFIX)?;
    suffix.parse::<u64>().ok()
}

pub(crate) fn page_meta(selected: &str) -> (&'static str, &'static str) {
    let fallback = page_spec(PAGE_INTRO).expect("intro page exists");
    let page = page_spec(selected).unwrap_or(fallback);
    (page.title, page.origin)
}
