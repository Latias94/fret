from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


# These probes freeze the public grouped read/write lanes. The exact UI layout may
# evolve, but examples must not fall back to raw ModelStore reads or legacy notify
# helpers when a grouped AppUi capability exists.
GROUPED_STATE_POLICIES = (
    (
        "hello_counter_demo.rs",
        (
            "let count_state = cx.state().local_init(|| 0i64);",
            'let step_state = cx.state().local_init(|| "1".to_string());',
            "let count = count_state.layout_value(cx);",
            "cx.data().selector_layout(&step_state,",
            "cx.actions().locals_with((&count_state, &step_state))",
        ),
        (
            "count_state.layout(cx).value_or",
            "step_state.layout(cx).value_or",
            "cx.on_action_notify_models",
        ),
    ),
    (
        "query_demo.rs",
        (
            "let fail_mode_state = cx.state().local_init(|| false);",
            "let fail_mode = fail_mode_state.layout_value(cx);",
            "let query_state = query_handle.read_layout(cx);",
            "cx.data().invalidate_query(",
            "cx.actions().transient::<act::Invalidate>",
        ),
        (
            "query_handle.layout(cx).value_or_default()",
            "fail_mode_state.layout(cx).value_or_default()",
            "cx.take_transient_on_action_root(",
        ),
    ),
    (
        "query_async_tokio_demo.rs",
        (
            "let fail_mode_state = cx.state().local_init(|| false);",
            "let fail_mode = fail_mode_state.layout_value(cx);",
            "let query_state = query_handle.read_layout(cx);",
            "cx.data().invalidate_query(",
            "cx.actions().transient::<act::Invalidate>",
        ),
        (
            "query_handle.layout(cx).value_or_default()",
            "fail_mode_state.layout(cx).value_or_default()",
            "cx.take_transient_on_action_root(",
        ),
    ),
    (
        "todo_demo.rs",
        (
            "struct TodoLocals {",
            "let todos = locals.todos.layout_value(cx);",
            "let draft_value = locals.draft.layout_value(cx);",
            "cx.actions().locals_with((&self.draft, &self.next_id, &self.todos))",
            "cx.actions().local(&self.todos)",
            ".payload_update_if::<act::Toggle>",
        ),
        (
            "todos_state.layout(cx).value_or_default()",
            "draft_state.layout(cx).value_or_default()",
            "cx.on_payload_action_notify",
        ),
    ),
    (
        "table_demo.rs",
        (
            "table_state.layout_read_ref(cx, |st|",
            "let enable_grouping = enable_grouping.layout_value(cx);",
            "let grouped_column_mode = grouped_column_mode.layout_value(cx);",
        ),
        (
            "enable_grouping.layout(cx).value_or_default()",
            "grouped_column_mode.layout(cx).value_or_default()",
        ),
    ),
    (
        "genui_demo.rs",
        (
            "let auto_apply_enabled = st.auto_apply_standard_actions.layout_value(cx);",
            "let _auto_fix_enabled = st.auto_fix_on_apply.layout_value(cx);",
            "let stream_patch_only = st.stream_patch_only.layout_value(cx);",
        ),
        (
            "layout_value_in(cx)",
            "clone_model()",
        ),
    ),
    (
        "custom_effect_v1_demo.rs",
        (
            "model.layout_read_ref(cx, |v| v.first().copied().unwrap_or(default))",
            "let enabled = st.enabled.layout_value(cx);",
        ),
        (
            "model.layout_read_ref_in(cx,",
            "st.enabled.clone_model()",
        ),
    ),
    (
        "custom_effect_v2_demo.rs",
        (
            "model.layout_read_ref(cx, |v| v.first().copied().unwrap_or(default))",
            "let view_settings: CustomEffectV2ViewSettings = cx.data().selector_layout(",
            "&st.enabled,",
        ),
        (
            "model.layout_read_ref_in(cx,",
            "st.enabled.clone_model()",
        ),
    ),
)


CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_selected_grouped_state_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    for source_name, required, forbidden in GROUPED_STATE_POLICIES:
        path = examples_src / source_name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=list(required),
            forbidden=list(forbidden),
            failures=failures,
        )
