from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


QUERY_CAPABILITY_LANDING_SOURCES = [
    "query_demo.rs",
    "query_async_tokio_demo.rs",
]

QUERY_CAPABILITY_LANDING_REQUIRED = [
    "use fret_ui_kit::IntoUiElementInExt as _;",
    "}).gap(Space::N2).items_center().into_element_in(cx);",
    "}).gap(Space::N2).into_element_in(cx);",
]

QUERY_CAPABILITY_LANDING_FORBIDDEN = [
    "}).gap(Space::N2).items_center().into_element(cx);",
    "}).gap(Space::N2).into_element(cx);",
]

MARKDOWN_LAYOUT_QUERY_REQUIRED = [
    "cx.layout_query_bounds(anchor_id, Invalidation::Layout)",
    "cx.layout_query_bounds(viewport_region, Invalidation::Layout)",
    "cx.layout_query_region_with_id(props, move |_cx, id| {",
    "let scroll = cx.layout_query_region_with_id(",
    "pending_anchor.set_in(cx.app_mut().models_mut(), None);",
]

MARKDOWN_LAYOUT_QUERY_FORBIDDEN = [
    "pending_anchor.set_in(cx.app.models_mut(), None);",
    "cx.elements().layout_query_bounds(",
    "cx.elements().layout_query_region_with_id(",
]

MARKDOWN_CAPABILITY_LANDING_REQUIRED = [
    "use fret_ui_kit::IntoUiElementInExt as _;",
    "}).gap(Space::N3).wrap().items_center().into_element_in(cx);",
    "}).w_full().padding_px(padding_md).into_element_in(cx)])",
    ".refine_layout(LayoutRefinement::default().w_full().flex_1()).into_element_in(cx);",
    "}).w_full().h_full().gap(Space::N3).padding_px(padding_md).into_element_in(cx);",
    "ui::container(|_cx| [content]).bg(ColorRef::Color(theme.color_token(\"background\"))).w_full().h_full().into_element_in(cx).into()",
]

MARKDOWN_CAPABILITY_LANDING_FORBIDDEN = [
    "}).gap(Space::N3).wrap().items_center().into_element(cx);",
    "}).w_full().padding_px(padding_md).into_element(cx)])",
    ".refine_layout(LayoutRefinement::default().w_full().flex_1()).into_element(cx);",
    "}).w_full().h_full().gap(Space::N3).padding_px(padding_md).into_element(cx);",
    "}).bg(ColorRef::Color(theme.color_token(\"background\"))).w_full().h_full().into_element(cx).into()",
]

EDITOR_NOTES_REUSABLE_PANEL_REQUIRED = [
    "fn selection_button<'a, Cx>(",
    "pub(crate) fn render_selection_panel<'a, Cx>(",
    "pub(crate) fn render_center_panel<'a, Cx>(",
    "pub(crate) fn render_inspector_panel<'a, Cx>(",
    "Cx: fret::app::ElementContextAccess<'a, App>,",
]

EDITOR_NOTES_REUSABLE_PANEL_FORBIDDEN = [
    "fn selection_button(cx: &mut AppUi<'_, '_>,",
    "fn render_selection_panel(cx: &mut AppUi<'_, '_>,",
    "fn render_center_panel(cx: &mut AppUi<'_, '_>,",
    "fn render_inspector_panel(cx: &mut AppUi<'_, '_>,",
    "fn render_selection_panel(cx: &mut AppComponentCx<'_>,",
    "fn render_center_panel(cx: &mut AppComponentCx<'_>,",
    "fn render_inspector_panel(cx: &mut AppComponentCx<'_>,",
]

EDITOR_NOTES_RENDER_SLICE_REQUIRED = [
    ".h_full().into_element_in(cx).test_id(TEST_ID_LEFT_RAIL);",
    ".h_full().into_element_in(cx).test_id(TEST_ID_RIGHT_RAIL);",
    ".background(Some(theme.color_token(\"background\"))).into_element_in(cx);",
    "ui::container(|_cx| [frame]).p(Space::N4).size_full().into_element_in(cx).test_id(TEST_ID_ROOT).into()",
]

EDITOR_NOTES_RENDER_SLICE_FORBIDDEN = [
    ".h_full().into_element(cx).test_id(TEST_ID_LEFT_RAIL);",
    ".h_full().into_element(cx).test_id(TEST_ID_RIGHT_RAIL);",
    ".background(Some(theme.color_token(\"background\"))).into_element(cx);",
    "ui::container(|_cx| [frame]).p(Space::N4).size_full().into_element(cx).test_id(TEST_ID_ROOT).into()",
]

TODO_DEFAULT_APP_REQUIRED = [
    "use fret::app::prelude::*;",
    "use fret::env::{ ViewportQueryHysteresis, primary_pointer_can_hover, viewport_tailwind, viewport_width_at_least, };",
    "fn init(_app: &mut App, _window: WindowId) -> Self",
    "ui::single(cx, todo_page(theme, responsive, card))",
    "fn todo_page(",
    "responsive: TodoResponsiveLayout,",
    "struct TodoLocals {",
    "fn new(cx: &mut AppUi<'_, '_>) -> Self {",
    "struct TodoDemoView;",
    "let locals = TodoLocals::new(cx);",
    "locals.bind_actions(cx);",
    "draft: cx.state().local::<String>(),",
    "filter: cx.state().local_init(|| Some(Arc::<str>::from(TodoFilter::All.value()))),",
    "next_id: cx.state().local_init(|| 4u64),",
    "todos: cx.state().local_init(|| {",
    "fn bind_actions(&self, cx: &mut AppUi<'_, '_>) {",
    "let filter_value = TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());",
    ".setup(fret_icons_lucide::app::install)",
    ".window_min_size(TODO_WINDOW_MIN_SIZE)",
    ".window_position_logical(TODO_WINDOW_POSITION_LOGICAL)",
    ".window_resize_increments(TODO_WINDOW_RESIZE_INCREMENTS)",
    "ui::for_each_keyed_with_cx(",
    "fn todo_row<'a, Cx>(",
    "Cx: fret::app::ElementContextAccess<'a, App>,",
    "shadcn::Progress::from_value(progress_pct).a11y_label(\"Todo completion progress\").ui().rounded(Radius::Full).w_full().build()",
    ".viewport_test_id(TEST_ID_ROWS).ui().w_full().h_full().flex_1().min_h_0().build()",
    ".corner_radii_override(Corners::all(Px(14.0))).ui().shadow_sm().build()",
    ".test_id(TEST_ID_DRAFT).ui().shadow_sm().flex_1().min_w_0().build()",
    ".a11y_label(format!(\"Show {} tasks\", filter.label().to_lowercase())).test_id(test_id).refine_style(ChromeRefinement::default().rounded(Radius::Full)).refine_layout(fret_ui_kit::LayoutRefinement::default().h_px(Px(28.0)).min_h(Px(28.0)),)",
    "ui::hover_region(move |cx, hovered| {",
    "ui::rich_text(rich)",
    "ui::v_flex(move |cx| ui::single(cx, content))",
]

TODO_DEFAULT_APP_FORBIDDEN = [
    "let card = card.into_element(cx);",
    "todo_page(theme, card).into_element(cx).into()",
    "fret_cookbook::scaffold::",
    "centered_page_muted(",
    "centered_page_background(",
    "let cx = cx.elements();",
    "rows_max_height",
    ".a11y_label(\"Todo completion progress\").refine_style(",
    ".viewport_test_id(TEST_ID_ROWS).refine_layout(",
    "use fret_ui_kit::declarative::{ ElementContextThemeExt as _, ViewportQueryHysteresis, primary_pointer_can_hover, viewport_tailwind, viewport_width_at_least, };",
    "footer_pill_chrome()",
    "footer_pill_layout()",
    "HoverRegionProps",
    "StyledTextProps",
    "ui::v_flex(move |cx| ui::children![cx; content])",
    "cx: &mut fret_ui::ElementContext<'_, App>,",
    "TodoLocals::new(app)",
    "LocalState::from_model(app.models_mut().insert(",
]

TODO_ROOT_SLICE_REQUIRED = [
    "ui::text(\"Add a task to get started\").text_sm().text_color(ColorRef::Color(muted_foreground)).into_element_in(cx)",
    ".gap(Space::N1).items_center().into_element_in(cx)",
    "ui::text(format!(\"{active_count} {task_label} left\")).text_sm().text_color(ColorRef::Color(muted_foreground)).into_element_in(cx)",
    ".gap(Space::N1p5).w_full().into_element_in(cx)",
    "shadcn::ScrollArea::new([rows_body.into_element_in(cx)])",
    ".min_h_0().build().into_element_in(cx);",
    "let footer = if responsive.stack_footer {",
    "children }).gap(Space::N2).items_stretch().w_full().into_element_in(cx)",
    "children }).gap(Space::N3).items_center().justify_between().w_full().into_element_in(cx)",
]

TODO_ROOT_SLICE_FORBIDDEN = [
    "ui::text(\"Add a task to get started\").text_sm().text_color(ColorRef::Color(muted_foreground)).into_element(cx)",
    ".gap(Space::N1).items_center().into_element(cx)",
    "ui::text(format!(\"{active_count} {task_label} left\")).text_sm().text_color(ColorRef::Color(muted_foreground)).into_element(cx)",
    ".gap(Space::N1p5).w_full().into_element(cx)",
    "shadcn::ScrollArea::new([rows_body.into_element(cx)])",
    ".min_h_0().build().into_element(cx);",
    "children }).gap(Space::N2).items_stretch().w_full().into_element(cx)",
    "children }).gap(Space::N3).items_center().justify_between().w_full().into_element(cx)",
]

ASYNC_PLAYGROUND_APP_RENDER_CONTEXT_REQUIRED = [
    "AppRenderContext<'a>",
    "use fret::app::{AppRenderContext, RenderContextAccess as _};",
    "use fret_ui_kit::IntoUiElementInExt as _;",
    "fn tracked_query_inputs<'a, Cx>(",
    "fn header_bar<'a, Cx>(",
    "fn body<'a, Cx>(",
    "fn query_panel_for_mode<'a, Cx>(",
    "fn status_badge<'a, Cx>(",
    "Cx: AppRenderContext<'a>,",
    "cx.elements().pressable(",
    "let state = handle.read_layout(cx);",
    "locals.tabs.layout_read_ref(cx, |tab| match tab.as_deref() {",
    "config.fail_mode.layout_value(cx)",
]

ASYNC_PLAYGROUND_APP_RENDER_CONTEXT_FORBIDDEN = [
    "fn tracked_query_inputs(cx: &mut AppComponentCx<'_>,",
    "fn header_bar(cx: &mut AppComponentCx<'_>,",
    "fn body(cx: &mut AppComponentCx<'_>,",
    "fn query_panel_for_mode(cx: &mut AppComponentCx<'_>,",
    "fn status_badge(cx: &mut AppComponentCx<'_>,",
    "handle.layout_query(cx).value_or_default()",
    "locals.tabs.layout_read_ref_in(cx, |tab| match tab.as_deref() {",
    "config.fail_mode.layout_value_in(cx)",
]

ASYNC_PLAYGROUND_RENDER_SLICE_REQUIRED = [
    "let query_inputs = tracked_query_inputs(cx, &locals);",
    "let header = header_bar(cx, &locals, theme.clone(), global_slow, dark);",
    "let body = body(cx, &mut self.st, &locals, theme, global_slow, selected);",
    "ui::v_flex(|_cx| [header, body]).w_full().h_full().into_element_in(cx).into()",
]

ASYNC_PLAYGROUND_RENDER_SLICE_FORBIDDEN = [
    "let header = header_bar(cx, &locals, theme.clone(), global_slow, dark).into_element(cx);",
    "let body = body(cx, &mut self.st, &locals, theme, global_slow, selected).into_element(cx);",
    "ui::v_flex(|_cx| [header, body]).w_full().h_full().into_element(cx).into()",
]

API_WORKBENCH_LITE_REQUIRED = [
    "use fret::app::prelude::*;",
    "fn init(_app: &mut App, window: WindowId) -> Self",
    "Cx: AppRenderContext<'a>,",
    "cx.app().global::<HistoryDbGlobal>()",
    "shadcn::Dialog::new(&locals.settings_open).into_element_in(",
    ".with_in(cx, |cx| {",
    ".into_element_in(cx)",
    "cx.data().query_async(",
    "cx.data().mutation_async(",
    "cx.data().update_after_mutation_completion(",
    "move |models, state| apply_response_snapshot(models, &locals, state)",
    "QueryKey::<Vec<PersistedHistoryEntry>>::new(HISTORY_QUERY_NS, &())",
    "persist_history_snapshot(",
    "load_saved_history(",
    "sqlx::query(",
    "cx.data().invalidate_query_namespace_after_mutation_success(",
    "MutationConcurrencyPolicy::AllowParallelLatestWins",
    "response_mutation.retry_last(",
    "history_save_mutation.retry_last(",
]

API_WORKBENCH_LITE_FORBIDDEN = [
    "fn shell_frame(\n    cx: &mut AppUi<'_, '_>,",
    "fn request_panel(cx: &mut AppUi<'_, '_>,",
    "fn response_panel(cx: &mut AppUi<'_, '_>,",
    "cx.app.global::<HistoryDbGlobal>()",
    "shadcn::Dialog::new(&locals.settings_open).into_element(",
    ".with(cx.elements(), |cx| {",
    ".into_element(cx.elements())",
    "cx.elements()",
    "maybe_invalidate_saved_history_query(",
    "locals.history",
    "next_history_id",
    ".take_mutation_completion(",
    "last_applied_seq",
    "next_seq",
]

EMBEDDED_VIEWPORT_REQUIRED_TOGGLE_GROUP_REQUIRED = [
    "use fret_ui_kit::IntoUiElementInExt as _;",
    "shadcn::ToggleGroup::single(&size_preset_state)",
    ".deselectable(false)",
    "cx.state().local_init(|| Some(Arc::<str>::from(SIZE_PRESET_960)))",
]

EMBEDDED_VIEWPORT_REQUIRED_TOGGLE_GROUP_FORBIDDEN = [
    ".disabled(preset == 0)",
    "PickSize640",
]

EMBEDDED_VIEWPORT_CAPABILITY_RENDER_REQUIRED = [
    ".gap(Space::N1).into_element_in(cx);",
    "[ui::text(\"640\u00d7360\").into_element_in(cx)]",
    "[ui::text(\"960\u00d7540\").into_element_in(cx)]",
    "[ui::text(\"1280\u00d7720\").into_element_in(cx)]",
    ".refine_layout(LayoutRefinement::default().flex_none()).into_element_in(cx);",
    ".panel(cx.elements(), embedded::EmbeddedViewportPanelProps {",
    ".max_w(Px(980.0)).into_element_in(cx);",
]

EMBEDDED_VIEWPORT_CAPABILITY_RENDER_FORBIDDEN = [
    ".gap(Space::N1).into_element(cx);",
    "[cx.text(\"640\u00d7360\")]",
    "[cx.text(\"960\u00d7540\")]",
    "[cx.text(\"1280\u00d7720\")]",
    ".panel(cx, embedded::EmbeddedViewportPanelProps {",
    ".max_w(Px(980.0)).into_element(cx);",
]

HELLO_WORLD_COMPARE_APP_RENDER_CX_REQUIRED = [
    "use fret_ui_kit::IntoUiElementInExt as _;",
    "cx.set_continuous_frames(self.flags.uses_continuous_frames_lease());",
    "let swatch = |_cx: &mut AppRenderCx<'_>, fill_rgb: u32, border_rgb: u32|",
    "fn hello_world_compare_root<'a, Cx>(",
    "Cx: fret::app::ElementContextAccess<'a, KernelApp>",
    "ui::text(\"Hello, World!\")",
    ".text_size_px(Px(24.0))",
    ".font_semibold()",
    ".text_align(TextAlign::Center)",
    ".nowrap()",
    ".into_element_in(cx)",
    "panel_bg: Color,",
    "children: Vec<AnyElement>)",
    "hello_world_compare_root(cx, panel_bg, children)",
]

HELLO_WORLD_COMPARE_APP_RENDER_CX_FORBIDDEN = [
    "set_continuous_frames(cx, self.flags.uses_continuous_frames_lease());",
    "let swatch = |_cx: &mut AppComponentCx<'_>, fill_rgb: u32, border_rgb: u32|",
    "let swatch = |cx: &mut ElementContext<'_, KernelApp>,",
    "let swatch = |cx: &mut AppComponentCx<'_>, fill_rgb: u32, border_rgb: u32| -> AnyElement",
    "fn hello_world_compare_root(cx: &mut AppComponentCx<'_>, panel_bg: Color, children: Vec<AnyElement>) -> Ui",
    "let cx = cx.elements();",
    "cx.text_props(TextProps {",
    ".into_element(cx)",
    "hello_world_compare_root(cx.elements(), panel_bg, children)",
]

APP_UI_RENDER_ACCESSOR_SLICE_POLICIES = [
    (
        "embedded_viewport_demo.rs",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
        "fn embedded_viewport_page<'a, Cx, C>(",
        [
            "let window = cx.window_id();",
            "embedded::models(cx.app(), window)",
            "embedded::ensure_models(cx.app_mut(), window)",
        ],
        [
            "let window = cx.window;",
            "embedded::models(&*cx.app, window)",
            "embedded::ensure_models(cx.app, window)",
        ],
    ),
    (
        "async_playground_demo.rs",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
        "fn header_bar<'a, Cx>(",
        [
            "apply_theme(cx.app_mut(), dark);",
            "cx.data().invalidate_query(key);",
            "cx.data().cancel_query(key);",
            "cx.data().invalidate_query_namespace(ns);",
        ],
        [
            "apply_theme(cx.app, dark);",
            "with_query_client(cx.app_mut(), |client, app|",
            "with_query_client(cx.app_mut(), |client, _app|",
            "with_query_client(cx.app, |client, app|",
            "with_query_client(cx.app, |client, _app|",
        ],
    ),
    (
        "markdown_demo.rs",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
        "fn checkerboard_rgba8(",
        [
            "cx.data().invalidate_query_namespace(REMOTE_IMAGE_NAMESPACE);",
        ],
        [
            "with_query_client(",
        ],
    ),
    (
        "api_workbench_lite_demo.rs",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
        "fn bind_actions(",
        [
            "cx.app().global::<HistoryDbGlobal>()",
            "shadcn::Dialog::new(&locals.settings_open).into_element_in(",
        ],
        [
            "cx.app.global::<HistoryDbGlobal>()",
            "shadcn::Dialog::new(&locals.settings_open).into_element(",
        ],
    ),
    (
        "postprocess_theme_demo.rs",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
        "fn srgb(",
        [
            "cx.app().global::<DemoEffect>()",
        ],
        [
            "cx.app.global::<DemoEffect>()",
        ],
    ),
    (
        "genui_demo.rs",
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
        "fn view(cx: &mut ElementContext<'_, KernelApp>, st: &mut GenUiState) -> ViewElements {",
        [
            "Self::handle_msg(cx.app_mut(), &mut self.st, Msg::ClearActions);",
            "Self::handle_msg(cx.app_mut(), &mut self.st, Msg::AutoApplyToggled);",
        ],
        [
            "Self::handle_msg(cx.app, &mut self.st, Msg::ClearActions);",
            "Self::handle_msg(cx.app, &mut self.st, Msg::AutoApplyToggled);",
        ],
    ),
    (
        "hello_world_compare_demo.rs",
        "fn update_runtime_frame_sample_state(cx: &mut AppUi<'_, '_>) {",
        "fn capture_runtime_frame_sample_json(",
        [
            "let window = cx.window_id();",
            "state.last_frame_id = cx.app().frame_id().0;",
            "capture_element_runtime_frame_sample(cx.app_mut(),",
        ],
        [
            "let window = cx.window;",
            "state.last_frame_id = cx.app.frame_id().0;",
            "capture_element_runtime_frame_sample(cx.app, window);",
        ],
    ),
]

APP_UI_RENDER_ACCESSOR_WHOLE_SOURCE_POLICIES = [
    (
        "emoji_conformance_demo.rs",
        [
            "cx.app().global::<FontCatalogCache>()",
        ],
        [
            "cx.app.global::<FontCatalogCache>()",
        ],
    ),
]

VIEW_RUNTIME_GROUPED_STATE_POLICIES = [
    (
        "hello_counter_demo.rs",
        [
            "let count_state = cx.state().local_init(|| 0i64);",
            "let step_state = cx.state().local_init(|| \"1\".to_string());",
            "let count = count_state.layout_value(cx);",
            "selector_layout(&step_state,",
            "parse_step(step_text.as_str())",
            ".locals_with((&count_state, &step_state))",
            ".on::<act::Inc>(|tx, (count_state, step_state)| {",
            ".on::<act::Dec>(|tx, (count_state, step_state)| {",
            "cx.actions().local(&count_state).set::<act::Reset>(0);",
        ],
        [
            "cx.use_local_with(|| 0i64)",
            "cx.on_action_notify_models::<act::Inc>",
            "cx.on_action_notify_local_set::<act::Reset, i64>",
            "let count = count_state.layout(cx).value_or(0);",
            "let step_text = step_state.layout(cx).value_or_else(String::new);",
            "tx.value_or_else(&step_state, || \"1\".to_string())",
        ],
    ),
    (
        "query_demo.rs",
        [
            "let fail_mode_state = cx.state().local_init(|| false);",
            "let fail_mode = fail_mode_state.layout_value(cx);",
            "let query_state = query_handle.read_layout(cx);",
            "let status_label = query_state.status.as_str();",
            "let info_line = if query_state.is_refreshing() {",
            "let error_color = if query_state.has_error() {",
            "if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY)",
            "cx.data().invalidate_query(demo_key());",
            "cx.data().invalidate_query_namespace(key.namespace());",
            "cx.actions().local(&fail_mode_state)",
            ".toggle_bool::<act::ToggleFailMode>();",
            "cx.actions().transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);",
        ],
        [
            "with_query_client(",
            "cx.use_local_with(|| false)",
            "query_handle.layout(cx).value_or_default()",
            "let status_label = match query_state.status {",
            "QueryStatus::Loading if query_state.data.is_some() =>",
            "fail_mode_state.layout(cx).value_or_default()",
            "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY)",
            "cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>",
        ],
    ),
    (
        "query_async_tokio_demo.rs",
        [
            "let fail_mode_state = cx.state().local_init(|| false);",
            "let fail_mode = fail_mode_state.layout_value(cx);",
            "let query_state = query_handle.read_layout(cx);",
            "let status_label = query_state.status.as_str();",
            "let info_line = if query_state.is_refreshing() {",
            "let error_color = if query_state.has_error() {",
            "if cx.effects().take_transient(TRANSIENT_INVALIDATE_KEY)",
            "cx.data().invalidate_query(demo_key());",
            "cx.data().invalidate_query_namespace(key.namespace());",
            "cx.actions().local(&fail_mode_state)",
            ".toggle_bool::<act::ToggleFailMode>();",
            "cx.actions().transient::<act::Invalidate>(TRANSIENT_INVALIDATE_KEY);",
        ],
        [
            "with_query_client(",
            "cx.use_local_with(|| false)",
            "query_handle.layout(cx).value_or_default()",
            "let status_label = match query_state.status {",
            "QueryStatus::Loading if query_state.data.is_some() =>",
            "fail_mode_state.layout(cx).value_or_default()",
            "cx.take_transient_on_action_root(TRANSIENT_INVALIDATE_KEY)",
            "cx.on_action_notify_toggle_local_bool::<act::ToggleFailMode>",
        ],
    ),
    (
        "todo_demo.rs",
        [
            "struct TodoLocals {",
            "fn new(cx: &mut AppUi<'_, '_>) -> Self {",
            "struct TodoDemoView;",
            "let locals = TodoLocals::new(cx);",
            "locals.bind_actions(cx);",
            "let todos = locals.todos.layout_value(cx);",
            "let draft_value = locals.draft.layout_value(cx);",
            "draft: cx.state().local::<String>(),",
            "filter: cx.state().local_init(|| Some(Arc::<str>::from(TodoFilter::All.value()))),",
            "next_id: cx.state().local_init(|| 4u64),",
            "todos: cx.state().local_init(|| {",
            "let filter_value = TodoFilter::from_value(locals.filter.layout_value(cx).as_deref());",
            ".locals_with((&self.draft, &self.next_id, &self.todos))",
            ".on::<act::Add>(|tx, (draft, next_id, todos)| {",
            "let text = tx.value(&draft).trim().to_string();",
            "let id = tx.value(&next_id);",
            ".locals_with(&self.todos)",
            ".on::<act::ClearDone>(|tx, todos| {",
            "cx.actions().local(&self.todos)",
            ".payload_update_if::<act::Toggle>(|rows, id| {",
            ".payload_update_if::<act::Remove>(|rows, id| {",
            "fn todo_row<'a, Cx>(",
            "Cx: fret::app::ElementContextAccess<'a, App>,",
        ],
        [
            "bind_todo_actions(",
            "cx.use_local::<String>()",
            "cx.on_action_notify_models::<act::Add>",
            "cx.on_payload_action_notify_local_update_if::<act::Toggle, Vec<TodoRow>>",
            "cx: &mut fret_ui::ElementContext<'_, App>,",
            "todos_state.layout(cx).value_or_default()",
            "draft_state.layout(cx).value_or_default()",
            "tx.value_or_else(&draft_state, String::new)",
            "tx.value_or(&next_id_state, 1)",
            "TodoLocals::new(app)",
        ],
    ),
    (
        "embedded_viewport_demo.rs",
        [
            "let size_preset_state = cx.state().local_init(|| Some(Arc::<str>::from(SIZE_PRESET_960)));",
            "let preset = size_preset_state.layout_value(cx);",
            "shadcn::ToggleGroup::single(&size_preset_state)",
            ".deselectable(false)",
        ],
        [
            "cx.use_local_with(|| 1usize)",
            "cx.on_action_notify_local_set::<act::PickSize640, usize>",
            "let preset = size_preset_state.layout(cx).value_or_default();",
            "cx.actions().local(&size_preset_state).set::<act::PickSize640>(0);",
            ".disabled(preset == 0)",
        ],
    ),
]

CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]
SourceSlice = Callable[[Path, str, str, str], str]


def check_app_facing_demo_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    default_app_surface_common_forbidden: list[str],
    read_source: ReadSource,
    source_slice: SourceSlice,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    for name in QUERY_CAPABILITY_LANDING_SOURCES:
        path = examples_src / name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=QUERY_CAPABILITY_LANDING_REQUIRED,
            forbidden=QUERY_CAPABILITY_LANDING_FORBIDDEN,
            failures=failures,
        )

    markdown_path = examples_src / "markdown_demo.rs"
    check_required_forbidden_markers(
        markdown_path,
        read_source(markdown_path),
        required=MARKDOWN_LAYOUT_QUERY_REQUIRED + MARKDOWN_CAPABILITY_LANDING_REQUIRED,
        forbidden=MARKDOWN_LAYOUT_QUERY_FORBIDDEN + MARKDOWN_CAPABILITY_LANDING_FORBIDDEN,
        failures=failures,
    )

    editor_notes_path = examples_src / "editor_notes_demo.rs"
    editor_notes_source = read_source(editor_notes_path)
    check_required_forbidden_markers(
        editor_notes_path,
        editor_notes_source,
        required=[
            "use fret_ui_kit::{ColorRef, IntoUiElementInExt as _, Space};",
            *EDITOR_NOTES_REUSABLE_PANEL_REQUIRED,
        ],
        forbidden=EDITOR_NOTES_REUSABLE_PANEL_FORBIDDEN,
        failures=failures,
    )
    check_required_forbidden_markers(
        editor_notes_path,
        source_slice(
            editor_notes_path,
            editor_notes_source,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "impl EditorNotesDemoView {",
        ),
        required=EDITOR_NOTES_RENDER_SLICE_REQUIRED,
        forbidden=EDITOR_NOTES_RENDER_SLICE_FORBIDDEN,
        failures=failures,
    )

    todo_path = examples_src / "todo_demo.rs"
    todo_source = read_source(todo_path)
    check_required_forbidden_markers(
        todo_path,
        todo_source,
        required=TODO_DEFAULT_APP_REQUIRED,
        forbidden=default_app_surface_common_forbidden + TODO_DEFAULT_APP_FORBIDDEN,
        failures=failures,
    )
    check_required_forbidden_markers(
        todo_path,
        source_slice(
            todo_path,
            todo_source,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn todo_page(",
        ),
        required=TODO_ROOT_SLICE_REQUIRED,
        forbidden=TODO_ROOT_SLICE_FORBIDDEN,
        failures=failures,
    )

    async_path = examples_src / "async_playground_demo.rs"
    async_source = read_source(async_path)
    check_required_forbidden_markers(
        async_path,
        async_source,
        required=ASYNC_PLAYGROUND_APP_RENDER_CONTEXT_REQUIRED,
        forbidden=ASYNC_PLAYGROUND_APP_RENDER_CONTEXT_FORBIDDEN,
        failures=failures,
    )
    check_required_forbidden_markers(
        async_path,
        source_slice(
            async_path,
            async_source,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn header_bar<'a, Cx>(",
        ),
        required=ASYNC_PLAYGROUND_RENDER_SLICE_REQUIRED,
        forbidden=ASYNC_PLAYGROUND_RENDER_SLICE_FORBIDDEN,
        failures=failures,
    )

    api_workbench_path = examples_src / "api_workbench_lite_demo.rs"
    check_required_forbidden_markers(
        api_workbench_path,
        read_source(api_workbench_path),
        required=API_WORKBENCH_LITE_REQUIRED,
        forbidden=default_app_surface_common_forbidden + API_WORKBENCH_LITE_FORBIDDEN,
        failures=failures,
    )

    embedded_path = examples_src / "embedded_viewport_demo.rs"
    embedded_source = read_source(embedded_path)
    check_required_forbidden_markers(
        embedded_path,
        embedded_source,
        required=EMBEDDED_VIEWPORT_REQUIRED_TOGGLE_GROUP_REQUIRED,
        forbidden=EMBEDDED_VIEWPORT_REQUIRED_TOGGLE_GROUP_FORBIDDEN,
        failures=failures,
    )
    check_required_forbidden_markers(
        embedded_path,
        source_slice(
            embedded_path,
            embedded_source,
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {",
            "fn embedded_viewport_page<'a, Cx, C>(",
        ),
        required=EMBEDDED_VIEWPORT_CAPABILITY_RENDER_REQUIRED,
        forbidden=EMBEDDED_VIEWPORT_CAPABILITY_RENDER_FORBIDDEN,
        failures=failures,
    )

    hello_world_compare_path = examples_src / "hello_world_compare_demo.rs"
    check_required_forbidden_markers(
        hello_world_compare_path,
        read_source(hello_world_compare_path),
        required=HELLO_WORLD_COMPARE_APP_RENDER_CX_REQUIRED,
        forbidden=HELLO_WORLD_COMPARE_APP_RENDER_CX_FORBIDDEN,
        failures=failures,
    )

    for name, start, end, required, forbidden in APP_UI_RENDER_ACCESSOR_SLICE_POLICIES:
        path = examples_src / name
        source = read_source(path)
        check_required_forbidden_markers(
            path,
            source_slice(path, source, start, end),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )

    for name, required, forbidden in APP_UI_RENDER_ACCESSOR_WHOLE_SOURCE_POLICIES:
        path = examples_src / name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )

    for name, required, forbidden in VIEW_RUNTIME_GROUPED_STATE_POLICIES:
        path = examples_src / name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )
