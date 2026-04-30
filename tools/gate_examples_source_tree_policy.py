from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "examples source tree policy"

EXAMPLES_SRC = Path("apps/fret-examples/src")
IMUI_EXAMPLES_SRC = Path("apps/fret-examples-imui/src")
EXAMPLES_SOURCE_ROOTS = [EXAMPLES_SRC, IMUI_EXAMPLES_SRC]
EXCLUDED_SOURCES = {
    "apps/fret-examples/src/lib.rs",
    "apps/fret-examples-imui/src/lib.rs",
}

ALLOWED_FRET_UI_SHADCN_IMPORTS = {
    "use fret_ui_shadcn::facade as shadcn;",
    "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
}

ALLOWED_RAW_SHADCN_ESCAPES = [
    "shadcn::raw::typography::",
    "shadcn::raw::extras::",
    "fret::shadcn::raw::prelude::",
    "shadcn::raw::advanced::sync_theme_from_environment(",
    "fret::shadcn::raw::advanced::sync_theme_from_environment(",
    "shadcn::raw::advanced::install_with_ui_services(",
    "fret::shadcn::raw::advanced::install_with_ui_services(",
]

RAW_ACTION_NOTIFY_MARKERS = [
    "use fret::advanced::AppUiRawActionNotifyExt as _;",
    "cx.on_action_notify::<",
    "cx.on_payload_action_notify::<",
]

VIEW_RUNTIME_APP_UI_ALIAS_SOURCES = [
    EXAMPLES_SRC / "assets_demo.rs",
    EXAMPLES_SRC / "async_playground_demo.rs",
    EXAMPLES_SRC / "chart_declarative_demo.rs",
    EXAMPLES_SRC / "custom_effect_v1_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_demo.rs",
    EXAMPLES_SRC / "custom_effect_v3_demo.rs",
    EXAMPLES_SRC / "drop_shadow_demo.rs",
    EXAMPLES_SRC / "embedded_viewport_demo.rs",
    EXAMPLES_SRC / "external_texture_imports_demo.rs",
    EXAMPLES_SRC / "external_video_imports_avf_demo.rs",
    EXAMPLES_SRC / "external_video_imports_mf_demo.rs",
    EXAMPLES_SRC / "genui_demo.rs",
    EXAMPLES_SRC / "hello_counter_demo.rs",
    EXAMPLES_SRC / "hello_world_compare_demo.rs",
    EXAMPLES_SRC / "image_heavy_memory_demo.rs",
    EXAMPLES_SRC / "imui_editor_proof_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_floating_windows_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_hello_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_interaction_showcase_demo.rs",
    EXAMPLES_SRC / "imui_node_graph_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_response_signals_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_shadcn_adapter_demo.rs",
    EXAMPLES_SRC / "liquid_glass_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
    EXAMPLES_SRC / "node_graph_demo.rs",
    EXAMPLES_SRC / "postprocess_theme_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "query_demo.rs",
    EXAMPLES_SRC / "todo_demo.rs",
]

VIEW_ENTRY_BUILDER_THEN_RUN_SOURCES = [
    EXAMPLES_SRC / "async_playground_demo.rs",
    EXAMPLES_SRC / "chart_declarative_demo.rs",
    EXAMPLES_SRC / "drop_shadow_demo.rs",
    EXAMPLES_SRC / "genui_demo.rs",
    EXAMPLES_SRC / "hello_counter_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_floating_windows_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_hello_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_interaction_showcase_demo.rs",
    EXAMPLES_SRC / "imui_node_graph_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_response_signals_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_shadcn_adapter_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
    EXAMPLES_SRC / "node_graph_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "query_demo.rs",
    EXAMPLES_SRC / "todo_demo.rs",
]

GROUPED_DATA_SURFACE_SOURCES = [
    EXAMPLES_SRC / "async_playground_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "query_demo.rs",
]

FRET_QUERY_FACADE_SOURCES = [
    EXAMPLES_SRC / "async_playground_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "query_demo.rs",
]

ADVANCED_ENTRY_VIEW_ELEMENTS_ALIAS_SOURCES = [
    (EXAMPLES_SRC / "custom_effect_v1_demo.rs", "CustomEffectV1State"),
    (EXAMPLES_SRC / "custom_effect_v2_demo.rs", "CustomEffectV2State"),
    (EXAMPLES_SRC / "custom_effect_v3_demo.rs", "State"),
    (EXAMPLES_SRC / "genui_demo.rs", "GenUiState"),
    (EXAMPLES_SRC / "liquid_glass_demo.rs", "LiquidGlassState"),
]

DROPPING_FRET_DOCKING_OWNER_SOURCES = [
    EXAMPLES_SRC / "container_queries_docking_demo.rs",
    EXAMPLES_SRC / "docking_demo.rs",
]

RAW_FRET_DOCKING_OWNER_SOURCES = [
    EXAMPLES_SRC / "docking_arbitration_demo.rs",
    EXAMPLES_SRC / "imui_editor_proof_demo.rs",
]

DEFAULT_APP_THEME_SNAPSHOT_SOURCES = [
    EXAMPLES_SRC / "hello_counter_demo.rs",
    EXAMPLES_SRC / "query_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
]

ADVANCED_RUNTIME_CONTEXT_THEME_SNAPSHOT_SOURCES = [
    EXAMPLES_SRC / "embedded_viewport_demo.rs",
    EXAMPLES_SRC / "custom_effect_v1_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_demo.rs",
    EXAMPLES_SRC / "genui_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
]

ELEMENT_CONTEXT_THEME_READ_SOURCES = [
    EXAMPLES_SRC / "canvas_datagrid_stress_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_interaction_showcase_demo.rs",
]

RENDERER_THEME_BRIDGE_HOST_THEME_READ_SOURCES = [
    EXAMPLES_SRC / "postprocess_theme_demo.rs",
    EXAMPLES_SRC / "liquid_glass_demo.rs",
]

DEFAULT_APP_LOCAL_STATE_FIRST_SOURCES = [
    EXAMPLES_SRC / "hello_counter_demo.rs",
    EXAMPLES_SRC / "query_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "simple_todo_demo.rs",
    EXAMPLES_SRC / "todo_demo.rs",
]

INIT_PHASE_LOCAL_STATE_NEW_IN_SOURCES = [
    (
        EXAMPLES_SRC / "form_demo.rs",
        [
            "LocalState::new_in(app.models_mut(), String::new())",
            "LocalState::new_in(app.models_mut(), None::<Arc<str>>)",
            "LocalState::new_in(app.models_mut(), form_state)",
        ],
    ),
    (
        EXAMPLES_SRC / "async_playground_demo.rs",
        [
            "LocalState::new_in(app.models_mut(), initial.map(Arc::from))",
            "LocalState::new_in(app.models_mut(), \"2\".to_string())",
            "LocalState::new_in(app.models_mut(), false)",
        ],
    ),
    (
        EXAMPLES_SRC / "table_demo.rs",
        [
            "LocalState::new_in(app.models_mut(), false)",
            "LocalState::new_in(app.models_mut(), true)",
            "Some(Arc::<str>::from(\"reorder\"))",
        ],
    ),
    (
        EXAMPLES_SRC / "genui_demo.rs",
        [
            "LocalState::new_in(app.models_mut(), true)",
            "LocalState::new_in(app.models_mut(), SPEC_JSON.to_string())",
            "LocalState::new_in(app.models_mut(), String::new())",
        ],
    ),
]

APP_UI_RENDER_ROOT_BRIDGE_SOURCES = [
    (
        EXAMPLES_SRC / "form_demo.rs",
        [
            "app_ui_root: AppUiRenderRootState,",
            "form_state: LocalState<FormState>,",
            "LocalState::new_in(app.models_mut(), String::new())",
            "LocalState::new_in(app.models_mut(), None::<Arc<str>>)",
            "LocalState::new_in(app.models_mut(), form_state)",
            "let root = render_root_with_app_ui(",
            "let (submit_count, valid, dirty) = form_state.layout(cx).read_ref(",
            "let status_text = status.layout_value(cx);",
        ],
        [
            "form_state: Model<FormState>,",
            "LocalState::from_model(app.models_mut().insert(",
            ".render_root(\"form-demo\", move |cx| {",
            "cx.observe_model(&form_state, Invalidation::Layout);",
            "cx.app.models().read(&form_state, |st| {",
            "cx.app.models().read(&status, |v| Arc::clone(v))",
            "status.layout(cx).value_or_else(|| Arc::from(\"Idle\"));",
        ],
    ),
    (
        EXAMPLES_SRC / "date_picker_demo.rs",
        [
            "app_ui_root: AppUiRenderRootState,",
            "locals: Option<DatePickerDemoLocals>,",
            "struct DatePickerDemoLocals {",
            "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
            "open: cx.state().local_init(|| false),",
            "month: cx",
            "if locals.is_none() {",
            "let root = render_root_with_app_ui(",
            "let open_value = open.layout_value(cx);",
            "let selected_value = selected.layout_value(cx);",
            "let month_label: Arc<str> = month.layout(cx).read_ref(",
            "let cx = cx.elements();",
        ],
        [
            "open: Model<bool>,",
            "LocalState::from_model(app.models_mut().insert(",
            ".render_root(\"date-picker-demo\", move |cx| {",
            "cx.observe_model(&open, Invalidation::Layout);",
            "cx.app.models().get_copied(&open)",
            "cx.app.models().read(&month, |m| format!(\"{:?} {}\", m.month, m.year))",
            "open.layout(cx).copied_or(false)",
            "selected.layout(cx).value_or_default()",
        ],
    ),
    (
        EXAMPLES_SRC / "sonner_demo.rs",
        [
            "app_ui_root: AppUiRenderRootState,",
            "locals: Option<SonnerDemoLocals>,",
            "struct SonnerDemoLocals {",
            "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
            "last_action: cx.state().local_init(|| Arc::<str>::from(\"<none>\")),",
            "if locals.is_none() {",
            "let root = render_root_with_app_ui(",
            "let last_action_value = last_action.layout_value(cx);",
        ],
        [
            "last_action: Model<Arc<str>>,",
            "LocalState::from_model(app.models_mut().insert(",
            ".render_root(\"sonner-demo\", |cx| {",
            "cx.observe_model(&last_action, Invalidation::Layout);",
            "cx.app.models().get_cloned(&last_action)",
            "last_action.layout(cx).value_or_else(",
        ],
    ),
    (
        EXAMPLES_SRC / "ime_smoke_demo.rs",
        [
            "use fret::app::RenderContextAccess as _;",
            "app_ui_root: AppUiRenderRootState,",
            "locals: Option<ImeSmokeLocals>,",
            "struct ImeSmokeLocals {",
            "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
            "input_single: cx.state().local::<String>(),",
            "last_ime: cx.state().local_init(|| Arc::<str>::from(\"IME: <none>\")),",
            "if locals.is_none() {",
            "let root = render_root_with_app_ui(",
            "let theme = cx.theme_snapshot();",
            "let last = last_ime.paint_value(cx);",
            "shadcn::Input::new(&input_single)",
            "shadcn::Textarea::new(&input_multi)",
        ],
        [
            "input_single: Model<String>,",
            "last_ime: Model<Arc<str>>,",
            "LocalState::from_model(app.models_mut().insert(",
            ".render_root(\"ime-smoke\",",
            "cx.observe_model(&last_ime, Invalidation::Paint);",
            "cx.app.models().read(&last_ime, |v| v.clone())",
            "last_ime.paint(cx).value_or_else(",
            "input_single.clone_model()",
            "input_multi.clone_model()",
            "Theme::global(&*cx.app).snapshot()",
        ],
    ),
    (
        EXAMPLES_SRC / "emoji_conformance_demo.rs",
        [
            "app_ui_root: AppUiRenderRootState,",
            "locals: Option<EmojiConformanceLocals>,",
            "struct EmojiConformanceLocals {",
            "fn new(cx: &mut fret::AppUi<'_, '_>) -> Self {",
            "emoji_font_override: cx.state().local_init(|| None::<Arc<str>>),",
            "if locals.is_none() {",
            "let root = render_root_with_app_ui(",
            "let selected_emoji_font = emoji_font_override.layout_value(cx);",
        ],
        [
            "emoji_font_override: Model<Option<Arc<str>>>,",
            "LocalState::from_model(app.models_mut().insert(",
            ".render_root(\"emoji-conformance\", |cx| {",
            "cx.observe_model(&emoji_font_override, Invalidation::Layout);",
            "cx.app.models().read(&emoji_font_override, |v| v.clone())",
            "emoji_font_override.layout(cx).value_or_default()",
        ],
    ),
    (
        EXAMPLES_SRC / "components_gallery.rs",
        [
            "app_ui_root: AppUiRenderRootState,",
            "fn components_gallery_table_cell(",
            "cx: &mut dyn fret_ui::ElementContextAccess<'_, App>,",
            "let cx = cx.elements();",
            "let cell_at = Arc::new(components_gallery_table_cell);",
            "let root = render_root_with_app_ui(",
            "let theme = cx.theme_snapshot();",
            "let theme_name = cx.theme().name.clone();",
            "let theme = cx.theme();",
            "let state_revision = table_state.layout(cx).revision().unwrap_or(0);",
            "let selected = tree_state.layout(cx).read_ref(|s| s.selected).ok().flatten();",
            "let checkbox_value = checkbox.layout(cx).copied_or(false);",
            "let selected_emoji_font = emoji_font_override.layout(cx).value_or_default();",
            "let last_action_value = last_action.layout(cx).value_or_else(",
        ],
        [
            "move |cx: &mut ElementContext<'_, App>, col: &ColumnDef<u64>, row: &u64| {",
            ".render_root(\"components-gallery\", |cx| {",
            "cx.observe_model(&tree_state, Invalidation::Layout);",
            "cx.app.models().revision(&table_state).unwrap_or(0);",
            "cx.app.models().get_copied(&checkbox).unwrap_or(false);",
            "cx.app.models().get_cloned(&last_action);",
            "cx.app.models().read(&emoji_font_override, |v| v.clone())",
            "Theme::global(&*cx.app)",
        ],
    ),
]

LOCAL_STATE_COMPONENT_BRIDGE_SOURCES = [
    (
        EXAMPLES_SRC / "async_playground_demo.rs",
        ["shadcn::Select::new(&config.cancel_mode.value, &config.cancel_mode.open)"],
        ["config.cancel_mode.open.clone_model()"],
    ),
    (
        EXAMPLES_SRC / "form_demo.rs",
        [
            "shadcn::Select::new(&role, &role_open)",
            "shadcn::DatePicker::new(",
            "&start_date_open,",
            "&start_date_month,",
            "&start_date,",
            "registry.register_field(\"name\", &name, String::new(), |v| {",
            "registry.register_field(\"email\", &email, String::new(), |v| {",
            "registry.register_field(\"role\", &role, None, |v| {",
            "registry.register_field(\"start_date\", &start_date, None, |v| {",
            "shadcn::FormField::new(",
            "&form_state,",
            "shadcn::Input::new(&name)",
            "shadcn::Input::new(&email)",
        ],
        [
            "shadcn::Select::new(role.clone_model(), role_open.clone_model())",
            "DatePicker::new_controllable(",
            "start_date.clone_model()",
            "form_state.clone_model()",
            "name.clone_model()",
            "email.clone_model()",
            "registry.register_field(\"name\", name.clone_model(),",
            "registry.register_field(\"email\", email.clone_model(),",
            "registry.register_field(\"role\", role.clone_model(),",
            "registry.register_field(\"start_date\", start_date.clone_model(),",
        ],
    ),
    (
        EXAMPLES_SRC / "emoji_conformance_demo.rs",
        ["shadcn::Select::new(&emoji_font_override, &emoji_font_override_open)"],
        ["emoji_font_override_open.clone_model()"],
    ),
    (
        EXAMPLES_SRC / "date_picker_demo.rs",
        [
            "shadcn::Switch::new(&week_start_monday)",
            "shadcn::Switch::new(&show_outside_days)",
            "shadcn::Switch::new(&disable_outside_days)",
            "shadcn::Switch::new(&disable_weekends)",
            "shadcn::Switch::new(&disabled)",
            "shadcn::DatePicker::new(&open, &month, &selected)",
            "shadcn::Calendar::new(&month, &selected)",
        ],
        [
            "week_start_monday.clone_model()",
            "show_outside_days.clone_model()",
            "disable_outside_days.clone_model()",
            "disable_weekends.clone_model()",
            "disabled.clone_model()",
            "open.clone_model()",
            "month.clone_model()",
            "selected.clone_model()",
        ],
    ),
    (
        EXAMPLES_SRC / "drop_shadow_demo.rs",
        [
            "shadcn::Switch::new(&enabled_state)",
            "shadcn::Switch::new(&stress_state)",
        ],
        [
            "enabled_state.clone_model()",
            "stress_state.clone_model()",
        ],
    ),
    (
        EXAMPLES_SRC / "markdown_demo.rs",
        [
            "shadcn::Switch::new(&wrap_code_state)",
            "shadcn::Switch::new(&cap_code_height_state)",
        ],
        [
            "wrap_code_state.clone_model()",
            "cap_code_height_state.clone_model()",
        ],
    ),
]

DIRECT_LEAF_VISIBILITY_READ_SOURCES = [
    EXAMPLES_SRC / "custom_effect_v2_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_glass_chrome_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_identity_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_lut_web_demo.rs",
    EXAMPLES_SRC / "external_texture_imports_demo.rs",
    EXAMPLES_SRC / "external_texture_imports_web_demo.rs",
    EXAMPLES_SRC / "external_video_imports_avf_demo.rs",
    EXAMPLES_SRC / "external_video_imports_mf_demo.rs",
]

DIRECT_LEAF_VISIBILITY_READ_FORBIDDEN = [
    "cx.observe_model(&show, Invalidation::Layout);",
    "cx.observe_model(&show_model, Invalidation::Layout);",
    "cx.observe_model(&st.show, Invalidation::Layout);",
    "cx.app.models().read(&show, |v| *v).unwrap_or(true)",
    "cx.app.models().read(&show_model, |v| *v).unwrap_or(true)",
    "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
]

STRESS_RENDER_ROOT_SOURCES = [
    EXAMPLES_SRC / "virtual_list_stress_demo.rs",
    EXAMPLES_SRC / "canvas_datagrid_stress_demo.rs",
]

STRESS_RENDER_ROOT_FORBIDDEN = [
    "cx.observe_model(&state.tall_rows_enabled, Invalidation::Layout);",
    "cx.observe_model(&state.reversed, Invalidation::Layout);",
    "cx.observe_model(&state.items_revision, Invalidation::Layout);",
    "app.models().read(&state.tall_rows_enabled, |v| *v).unwrap_or(false);",
    "app.models().read(&state.reversed, |v| *v).unwrap_or(false);",
    "app.models().read(&state.items_revision, |v| *v).unwrap_or(0);",
    "cx.observe_model(&state.variable_sizes, Invalidation::Layout);",
    "cx.observe_model(&state.clamp_rows, Invalidation::Layout);",
    "cx.observe_model(&state.revision, Invalidation::Layout);",
    "cx.observe_model(&state.grid_output, Invalidation::Layout);",
    "app.models().read(&state.variable_sizes, |v| *v).unwrap_or(false);",
    "app.models().read(&state.clamp_rows, |v| *v).unwrap_or(false);",
    "app.models().read(&state.revision, |v| *v).unwrap_or(1);",
    "cx.app.models().read(&state.grid_output, |v| *v).unwrap_or_default();",
]

GENUI_MESSAGE_LANE_REQUIRED = [
    "impl GenUiState {",
    "fn clear_action_queue(&self, app: &mut KernelApp) {",
    "fn queued_invocations(",
    "fn auto_apply_enabled(&self, app: &KernelApp) -> bool {",
    "fn auto_fix_enabled(&self, app: &KernelApp) -> bool {",
    "fn editor_text_value(&self, app: &KernelApp) -> String {",
    "fn stream_text_value(&self, app: &KernelApp) -> String {",
    "fn stream_patch_only_enabled(&self, app: &KernelApp) -> bool {",
    "let auto_apply = state.auto_apply_enabled(app);",
    "let invocations = state.queued_invocations(app);",
    "state.clear_action_queue(app);",
    "let text = state.editor_text_value(app);",
    "let auto_fix = state.auto_fix_enabled(app);",
    "let text = state.stream_text_value(app);",
    "let patch_only = state.stream_patch_only_enabled(app);",
]

GENUI_MESSAGE_LANE_FORBIDDEN = [
    "state.auto_apply_standard_actions.value_in_or(app.models(), true);",
    "app.models().read(&state.action_queue, |q| q.invocations.clone())",
    "state.editor_text.value_in_or_default(app.models());",
    "state.auto_fix_on_apply.value_in_or(app.models(), true);",
    "state.stream_text.value_in_or_default(app.models());",
    "state.stream_patch_only.value_in_or(app.models(), false);",
    "app.models_mut().update(&state.action_queue, |q| q.invocations.clear());",
]

DRIVER_OWNED_SOURCE_SLICES = [
    (
        EXAMPLES_SRC / "embedded_viewport_demo.rs",
        "fn record_embedded_viewport(",
        "pub fn run() -> anyhow::Result<()> {",
        [
            "embedded::models(app, window)",
            "app.models().read(&m.clicks, |v| *v).ok()",
        ],
    ),
    (
        EXAMPLES_SRC / "external_texture_imports_demo.rs",
        "fn record_engine_frame(",
        "pub fn run() -> anyhow::Result<()> {",
        ["let show = app.models().read(&st.view.show, |v| *v).unwrap_or(true);"],
    ),
    (
        EXAMPLES_SRC / "external_texture_imports_web_demo.rs",
        "fn record_engine_frame(",
        "fn handle_event(",
        ["let show = app.models().read(&state.show, |v| *v).unwrap_or(true);"],
    ),
    (
        EXAMPLES_SRC / "external_video_imports_avf_demo.rs",
        "fn record_engine_frame(",
        "pub fn run() -> anyhow::Result<()> {",
        ["let show = app.models().read(&st.view.show, |v| *v).unwrap_or(true);"],
    ),
    (
        EXAMPLES_SRC / "external_video_imports_mf_demo.rs",
        "fn record_engine_frame(",
        "pub fn run() -> anyhow::Result<()> {",
        ["let show = app.models().read(&st.view.show, |v| *v).unwrap_or(true);"],
    ),
    (
        EXAMPLES_SRC / "workspace_shell_demo.rs",
        "fn handle_command(",
        "fn handle_event(",
        ["let prompt = app.models().get_cloned(&state.dirty_close_prompt).flatten();"],
    ),
    (
        EXAMPLES_SRC / "launcher_utility_window_demo.rs",
        "fn on_command(",
        "fn on_event(",
        ["let next = !st.always_on_top.value_in_or(app.models(), false);"],
    ),
    (
        EXAMPLES_SRC / "plot_stress_demo.rs",
        "fn maybe_animate_bounds(",
        "fn gpu_ready(",
        ["let animate = app.models().read(&state.animate, |v| *v).unwrap_or(false);"],
    ),
    (
        EXAMPLES_SRC / "plot_stress_demo.rs",
        "fn render(driver: &mut PlotStressDriver, context: WinitRenderContext<'_, PlotStressWindowState>) {",
        "fn window_create_spec(",
        ["let animate = app.models().read(&state.animate, |v| *v).unwrap_or(false);"],
    ),
]

DRIVER_OWNED_SOURCE_SLICE_FORBIDDEN = [
    "selector_model_layout(",
    "layout_value_in(",
]

ASSET_HELPER_ENTRYPOINT_SOURCES = [
    (
        EXAMPLES_SRC / "assets_demo.rs",
        [
            "use fret_ui_assets::ui::{image_stats_in, svg_stats_in, use_rgba8_image_state_in};",
            "use_rgba8_image_state_in(cx, 96, 96, checker_rgba.as_slice(), ImageColorSpace::Srgb);",
            "let image_stats = image_stats_in(cx);",
            "let svg_stats = svg_stats_in(cx);",
        ],
        [
            "image_asset_state::use_rgba8_image_state(cx.app, cx.window,",
            "UiAssets::image_stats(cx.app);",
            "UiAssets::svg_stats(cx.app);",
        ],
    ),
    (
        EXAMPLES_SRC / "markdown_demo.rs",
        [
            "use fret_ui_assets::ui::use_rgba8_image_state_in;",
            "let (_key, image, _status) = use_rgba8_image_state_in(cx,",
        ],
        ["image_asset_state::use_rgba8_image_state("],
    ),
]

EMBEDDED_VIEWPORT_DRIVER_EXTENSION_SOURCES = [
    EXAMPLES_SRC / "embedded_viewport_demo.rs",
    EXAMPLES_SRC / "imui_editor_proof_demo.rs",
]

WORKSPACE_SHELL_CAPABILITY_HELPER_REQUIRED = [
    "fn workspace_shell_command_button<'a, Cx>(",
    "Cx: fret::app::ElementContextAccess<'a, App>,",
    "let cx = cx.elements();",
    "workspace_shell_command_button(",
    "fn workspace_shell_editor_rail<'a, Cx>(",
    "workspace_shell_editor_rail(",
    "InspectorPanel::new(None)",
    ".into_element_in(cx,",
    "PropertyGrid::new().into_element_in(cx,",
]

WORKSPACE_SHELL_CAPABILITY_HELPER_FORBIDDEN = [
    "let button = |cx: &mut fret_ui::ElementContext<'_, App>,",
    "fn workspace_shell_editor_rail(cx: &mut fret_ui::ElementContext<'_, App>,",
]

FIRST_PARTY_CURATED_SHADCN_SURFACES = [
    EXAMPLES_SRC / "assets_demo.rs",
    EXAMPLES_SRC / "async_playground_demo.rs",
    EXAMPLES_SRC / "cjk_conformance_demo.rs",
    EXAMPLES_SRC / "components_gallery.rs",
    EXAMPLES_SRC / "custom_effect_v1_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_glass_chrome_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_identity_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_lut_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v2_web_demo.rs",
    EXAMPLES_SRC / "custom_effect_v3_demo.rs",
    EXAMPLES_SRC / "docking_arbitration_demo.rs",
    EXAMPLES_SRC / "docking_demo.rs",
    EXAMPLES_SRC / "drop_shadow_demo.rs",
    EXAMPLES_SRC / "embedded_viewport_demo.rs",
    EXAMPLES_SRC / "emoji_conformance_demo.rs",
    EXAMPLES_SRC / "genui_demo.rs",
    EXAMPLES_SRC / "hello_counter_demo.rs",
    EXAMPLES_SRC / "ime_smoke_demo.rs",
    EXAMPLES_SRC / "imui_editor_proof_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_interaction_showcase_demo.rs",
    IMUI_EXAMPLES_SRC / "imui_shadcn_adapter_demo.rs",
    EXAMPLES_SRC / "liquid_glass_demo.rs",
    EXAMPLES_SRC / "markdown_demo.rs",
    EXAMPLES_SRC / "postprocess_theme_demo.rs",
    EXAMPLES_SRC / "query_async_tokio_demo.rs",
    EXAMPLES_SRC / "simple_todo_demo.rs",
    EXAMPLES_SRC / "sonner_demo.rs",
]

CURATED_SHADCN_FORBIDDEN_MARKERS = [
    "use fret_ui_shadcn as shadcn;",
    "use fret_ui_shadcn::{self as shadcn",
    "shadcn::shadcn_themes::",
    "shadcn::typography::",
]


@dataclass(frozen=True)
class Failure:
    path: Path
    line_no: int | None
    message: str
    line: str | None = None


def normalize(text: str) -> str:
    return "".join(text.split())


def rel_path(path: Path) -> Path:
    return path.resolve().relative_to(WORKSPACE_ROOT)


def read_source(path: Path) -> str:
    full_path = path if path.is_absolute() else WORKSPACE_ROOT / path
    try:
        return full_path.read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {rel_path(full_path).as_posix()}: {exc}")


def source_slice(path: Path, source: str, start_marker: str, end_marker: str) -> str:
    try:
        start = source.index(start_marker)
    except ValueError:
        failures_path = path if path.is_absolute() else WORKSPACE_ROOT / path
        fail(
            GATE_NAME,
            f"missing start marker in {rel_path(failures_path).as_posix()}: {start_marker}",
        )
    try:
        end = source.index(end_marker, start)
    except ValueError:
        failures_path = path if path.is_absolute() else WORKSPACE_ROOT / path
        fail(
            GATE_NAME,
            f"missing end marker in {rel_path(failures_path).as_posix()}: {end_marker}",
        )
    return source[start:end]


def examples_rust_sources() -> list[Path]:
    paths: list[Path] = []
    for root in EXAMPLES_SOURCE_ROOTS:
        paths.extend((WORKSPACE_ROOT / root).rglob("*.rs"))
    return sorted(path for path in paths if rel_path(path).as_posix() not in EXCLUDED_SOURCES)


def push_line_failure(
    failures: list[Failure],
    path: Path,
    line_no: int,
    message: str,
    line: str,
) -> None:
    failures.append(Failure(rel_path(path), line_no, message, line.strip()))


def check_source_tree_policies(path: Path, source: str, failures: list[Failure]) -> None:
    if "use fret_ui_shadcn as shadcn;" in source:
        failures.append(
            Failure(rel_path(path), None, "reintroduced `use fret_ui_shadcn as shadcn;`")
        )
    if "use fret_ui_shadcn::{self as shadcn" in source:
        failures.append(
            Failure(rel_path(path), None, "reintroduced `use fret_ui_shadcn::{self as shadcn`")
        )

    for line_no, line in enumerate(source.splitlines(), start=1):
        trimmed = line.strip()

        if "fret_ui_shadcn::" in line and trimmed not in ALLOWED_FRET_UI_SHADCN_IMPORTS:
            push_line_failure(
                failures,
                path,
                line_no,
                "non-curated fret_ui_shadcn import",
                line,
            )

        if "shadcn::raw::" in trimmed or "fret::shadcn::raw::" in trimmed:
            if not any(marker in trimmed for marker in ALLOWED_RAW_SHADCN_ESCAPES):
                push_line_failure(
                    failures,
                    path,
                    line_no,
                    "undocumented shadcn raw escape hatch",
                    line,
                )

        for marker in RAW_ACTION_NOTIFY_MARKERS:
            if marker in line:
                push_line_failure(
                    failures,
                    path,
                    line_no,
                    f"raw action notify helper: {marker}",
                    line,
                )

    normalized = normalize(source)
    if ".setup(|" in normalized:
        failures.append(Failure(rel_path(path), None, "inline `.setup(|...)` closure"))
    if ".setup(move|" in normalized:
        failures.append(Failure(rel_path(path), None, "inline `.setup(move |...)` closure"))

    if ".setup_with(" in normalized:
        if path.name != "imui_editor_proof_demo.rs":
            failures.append(Failure(rel_path(path), None, "unexpected `.setup_with(...)` usage"))
        elif ".setup_with(move|" not in normalized:
            failures.append(
                Failure(
                    rel_path(path),
                    None,
                    "`imui_editor_proof_demo.rs` should keep `.setup_with(move |...)`",
                )
            )


def check_first_party_curated_shadcn_surfaces(failures: list[Failure]) -> None:
    for relative_path in FIRST_PARTY_CURATED_SHADCN_SURFACES:
        path = WORKSPACE_ROOT / relative_path
        source = read_source(path)
        for marker in CURATED_SHADCN_FORBIDDEN_MARKERS:
            if marker in source:
                failures.append(
                    Failure(
                        rel_path(path),
                        None,
                        f"first-party shadcn surface used forbidden marker: {marker}",
                    )
                )


def check_required_forbidden_markers(
    path: Path,
    source: str,
    required: list[str],
    forbidden: list[str],
    failures: list[Failure],
) -> None:
    normalized = normalize(source)
    for marker in required:
        if normalize(marker) not in normalized:
            failures.append(Failure(path, None, f"missing source marker: {marker}"))
    for marker in forbidden:
        if normalize(marker) in normalized:
            failures.append(Failure(path, None, f"forbidden source marker: {marker}"))


def check_view_runtime_app_ui_aliases(failures: list[Failure]) -> None:
    for path in VIEW_RUNTIME_APP_UI_ALIAS_SOURCES:
        source = read_source(path)
        has_current_signature = (
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui" in source
            or "fn render(&mut self, cx: &mut fret::AppUi<'_, '_, App>) -> fret::Ui" in source
        )
        if not has_current_signature:
            failures.append(
                Failure(
                    path,
                    None,
                    "missing AppUi/Ui render signature for view-runtime example",
                )
            )
        check_required_forbidden_markers(
            path,
            source,
            required=[],
            forbidden=[
                "fn render(&mut self, cx: &mut ViewCx<'_, '_, KernelApp>) -> Elements",
                "fn render(&mut self, cx: &mut fret::view::ViewCx<'_, '_, App>) -> Elements",
                "ViewCx<'_, '_, KernelApp>",
                "ViewCx<'_, '_, App>",
            ],
            failures=failures,
        )


def check_view_entry_builder_then_run(failures: list[Failure]) -> None:
    for path in VIEW_ENTRY_BUILDER_THEN_RUN_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=[".view::<", ".run()"],
            forbidden=[".run_view::<"],
            failures=failures,
        )


def check_grouped_data_surface(failures: list[Failure]) -> None:
    required_any = [
        "cx.data().selector_layout(",
        "cx.data().selector(",
        "cx.data().query(",
        "cx.data().query_async(",
        "cx.data().query_async_local(",
    ]
    forbidden = [
        "fret_query::ui::QueryElementContextExt",
        "fret_selector::ui::SelectorElementContextExt",
        "cx.use_selector(",
        "cx.use_query(",
        "cx.use_query_async(",
        "cx.use_query_async_local(",
    ]
    for path in GROUPED_DATA_SURFACE_SOURCES:
        source = read_source(path)
        if not any(marker in source for marker in required_any):
            failures.append(Failure(path, None, "missing grouped data surface marker"))
        check_required_forbidden_markers(
            path,
            source,
            required=[],
            forbidden=forbidden,
            failures=failures,
        )


def check_fret_query_facade(failures: list[Failure]) -> None:
    for path in FRET_QUERY_FACADE_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["use fret::query::{"],
            forbidden=["use fret_query::{"],
            failures=failures,
        )


def check_advanced_entry_view_elements_alias(failures: list[Failure]) -> None:
    for path, state in ADVANCED_ENTRY_VIEW_ELEMENTS_ALIAS_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=[
                f"fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut {state}) -> ViewElements"
            ],
            forbidden=[
                f"fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut {state}) -> Elements"
            ],
            failures=failures,
        )


def check_fret_docking_owner_imports(failures: list[Failure]) -> None:
    for path in DROPPING_FRET_DOCKING_OWNER_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["use fret_docking::{"],
            forbidden=["use fret::docking::{"],
            failures=failures,
        )
    for path in RAW_FRET_DOCKING_OWNER_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["use fret_docking::{"],
            forbidden=[],
            failures=failures,
        )


def check_workspace_shell_capability_helpers(failures: list[Failure]) -> None:
    path = EXAMPLES_SRC / "workspace_shell_demo.rs"
    check_required_forbidden_markers(
        path,
        read_source(path),
        WORKSPACE_SHELL_CAPABILITY_HELPER_REQUIRED,
        WORKSPACE_SHELL_CAPABILITY_HELPER_FORBIDDEN,
        failures,
    )


def check_theme_snapshot_helpers(failures: list[Failure]) -> None:
    for path in DEFAULT_APP_THEME_SNAPSHOT_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["let theme = cx.theme_snapshot();"],
            forbidden=["Theme::global(&*cx.app).snapshot()"],
            failures=failures,
        )

    for path in ADVANCED_RUNTIME_CONTEXT_THEME_SNAPSHOT_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["cx.theme_snapshot()"],
            forbidden=["Theme::global(&*cx.app).snapshot()"],
            failures=failures,
        )

    for path in ELEMENT_CONTEXT_THEME_READ_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["cx.theme().snapshot()"],
            forbidden=["Theme::global(&*cx.app).snapshot()"],
            failures=failures,
        )

    for path in RENDERER_THEME_BRIDGE_HOST_THEME_READ_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["Theme::global(&*cx.app).snapshot()"],
            forbidden=[],
            failures=failures,
        )


def check_local_state_bridge_sources(failures: list[Failure]) -> None:
    for path in DEFAULT_APP_LOCAL_STATE_FIRST_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["cx.state().local"],
            forbidden=[
                "app.models_mut().insert(",
                "Model<",
                "cx.use_local_with(",
                "cx.actions().models::<",
                "cx.on_action_notify_models::<",
            ],
            failures=failures,
        )

    for path, required in INIT_PHASE_LOCAL_STATE_NEW_IN_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=["LocalState::from_model(app.models_mut().insert("],
            failures=failures,
        )

    for path, required, forbidden in APP_UI_RENDER_ROOT_BRIDGE_SOURCES:
        source = read_source(path)
        check_required_forbidden_markers(
            path,
            source,
            required=["UiTree<App>", *required],
            forbidden=["KernelApp", *forbidden],
            failures=failures,
        )

    for path, required, forbidden in LOCAL_STATE_COMPONENT_BRIDGE_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )


def check_model_read_and_asset_helper_sources(failures: list[Failure]) -> None:
    for path in DIRECT_LEAF_VISIBILITY_READ_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=[
                "use fret::advanced::view::AppRenderDataExt as _;",
                "selector_model_layout(",
            ],
            forbidden=DIRECT_LEAF_VISIBILITY_READ_FORBIDDEN,
            failures=failures,
        )

    for path in STRESS_RENDER_ROOT_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=[
                "use fret::advanced::view::AppRenderDataExt as _;",
                "cx.data().selector_model_layout(",
            ],
            forbidden=STRESS_RENDER_ROOT_FORBIDDEN,
            failures=failures,
        )

    check_required_forbidden_markers(
        EXAMPLES_SRC / "genui_demo.rs",
        read_source(EXAMPLES_SRC / "genui_demo.rs"),
        required=GENUI_MESSAGE_LANE_REQUIRED,
        forbidden=GENUI_MESSAGE_LANE_FORBIDDEN,
        failures=failures,
    )

    for path, start_marker, end_marker, required in DRIVER_OWNED_SOURCE_SLICES:
        source = read_source(path)
        check_required_forbidden_markers(
            path,
            source_slice(path, source, start_marker, end_marker),
            required=required,
            forbidden=DRIVER_OWNED_SOURCE_SLICE_FORBIDDEN,
            failures=failures,
        )

    for path, required, forbidden in ASSET_HELPER_ENTRYPOINT_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )

    for path in EMBEDDED_VIEWPORT_DRIVER_EXTENSION_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=[".drive_embedded_viewport()"],
            forbidden=["EmbeddedViewportUiAppDriverExt"],
            failures=failures,
        )


def print_failures(failures: list[Failure]) -> None:
    if not failures:
        return

    print(f"[gate] {GATE_NAME}")
    print(f"[gate] FAIL: {len(failures)} source policy problem(s)")
    for failure in failures[:60]:
        location = failure.path.as_posix()
        if failure.line_no is not None:
            location = f"{location}:{failure.line_no}"
        print(f"  - {location}: {failure.message}")
        if failure.line is not None:
            print(f"      {failure.line}")
    if len(failures) > 60:
        print(f"  ... and {len(failures) - 60} more")


def main() -> None:
    failures: list[Failure] = []
    for path in examples_rust_sources():
        check_source_tree_policies(path, read_source(path), failures)
    check_first_party_curated_shadcn_surfaces(failures)
    check_view_runtime_app_ui_aliases(failures)
    check_view_entry_builder_then_run(failures)
    check_grouped_data_surface(failures)
    check_fret_query_facade(failures)
    check_advanced_entry_view_elements_alias(failures)
    check_fret_docking_owner_imports(failures)
    check_workspace_shell_capability_helpers(failures)
    check_theme_snapshot_helpers(failures)
    check_local_state_bridge_sources(failures)
    check_model_read_and_asset_helper_sources(failures)

    print_failures(failures)
    if failures:
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
