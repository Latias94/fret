from __future__ import annotations

from pathlib import Path
from typing import Any, Callable, Literal


SourceRoot = Literal["examples", "imui_examples"]
LanePolicy = tuple[SourceRoot, str, tuple[str, ...], tuple[str, ...]]


THEME_LANE_POLICIES: tuple[LanePolicy, ...] = (
    *(
        (
            "examples",
            source_name,
            ("let theme = cx.theme_snapshot();",),
            ("Theme::global(&*cx.app).snapshot()",),
        )
        for source_name in (
            "hello_counter_demo.rs",
            "query_demo.rs",
            "query_async_tokio_demo.rs",
            "embedded_viewport_demo.rs",
            "custom_effect_v1_demo.rs",
            "custom_effect_v2_demo.rs",
            "genui_demo.rs",
            "markdown_demo.rs",
        )
    ),
    (
        "examples",
        "canvas_datagrid_stress_demo.rs",
        ("let theme = cx.theme().snapshot();",),
        ("Theme::global(&*cx.app).snapshot()",),
    ),
    (
        "imui_examples",
        "imui_interaction_showcase_demo.rs",
        ("let theme = cx.theme().snapshot();",),
        ("Theme::global(&*cx.app).snapshot()",),
    ),
    (
        "examples",
        "postprocess_theme_demo.rs",
        ("Theme::global(&*cx.app).snapshot()",),
        (),
    ),
    (
        "examples",
        "liquid_glass_demo.rs",
        ("Theme::global(&*cx.app).snapshot()",),
        (),
    ),
)


LOCAL_STATE_LANE_POLICIES: tuple[LanePolicy, ...] = (
    (
        "examples",
        "hello_counter_demo.rs",
        (
            "cx.state().local_init(|| 0i64)",
            'cx.state().local_init(|| "1".to_string())',
        ),
        (
            "app.models_mut().insert(",
            "LocalState::from_model(",
            "cx.use_local_with(",
        ),
    ),
    (
        "examples",
        "query_demo.rs",
        ("cx.state().local_init(|| false)",),
        (
            "app.models_mut().insert(",
            "LocalState::from_model(",
            "cx.use_local_with(",
        ),
    ),
    (
        "examples",
        "query_async_tokio_demo.rs",
        ("cx.state().local_init(|| false)",),
        (
            "app.models_mut().insert(",
            "LocalState::from_model(",
            "cx.use_local_with(",
        ),
    ),
    (
        "examples",
        "simple_todo_demo.rs",
        (
            "draft: cx.state().local::<String>()",
            "todos: cx.state().local_init",
        ),
        (
            "app.models_mut().insert(",
            "LocalState::from_model(",
            "TodoLocals::new(app)",
        ),
    ),
    (
        "examples",
        "todo_demo.rs",
        (
            "draft: cx.state().local::<String>()",
            "todos: cx.state().local_init",
        ),
        (
            "app.models_mut().insert(",
            "LocalState::from_model(",
            "TodoLocals::new(app)",
        ),
    ),
    (
        "examples",
        "form_demo.rs",
        (
            "app.local_state(String::new())",
            "app.local_state(form_state)",
            "app.local_state_txn(",
        ),
        (
            "LocalState::new_in(",
            "LocalState::from_model(",
        ),
    ),
    (
        "examples",
        "date_picker_demo.rs",
        (
            "open: cx.state().local_init(|| false)",
            "selected: cx.state().local_init(|| None::<time::Date>)",
            "app.local_state_txn(",
        ),
        (
            "open: Model<bool>",
            "LocalState::from_model(",
        ),
    ),
    (
        "examples",
        "emoji_conformance_demo.rs",
        (
            "emoji_font_override: cx.state().local_init(|| None::<Arc<str>>)",
            "app.local_state_txn(",
        ),
        (
            "emoji_font_override: Model<Option<Arc<str>>>",
            "LocalState::from_model(",
        ),
    ),
    (
        "examples",
        "editor_notes_demo.rs",
        (
            "theme: LocalState<EditorThemePreset>",
            "notes: InspectorTextFieldBinding",
            "app.local_state(preset)",
        ),
        (
            "EditorNotesModelOwner",
            "LocalState::from_model(",
        ),
    ),
)


ASSET_HELPER_POLICIES: tuple[LanePolicy, ...] = (
    (
        "examples",
        "assets_demo.rs",
        (
            ".with_ui_assets_budgets(",
            "svg_asset_state::use_svg_bytes_cached_with_stats(",
            "ui_assets::rgba8_image_state(",
            "ui_assets::image_stats(cx)",
            "ui_assets::svg_stats(cx)",
            "cx.watch_global::<AssetsDemoSvg>().layout()",
        ),
        (
            "image_asset_state::use_rgba8_image_state(cx.app",
            "UiAssets::image_stats(cx.app)",
            "UiAssets::svg_stats(cx.app)",
        ),
    ),
    (
        "examples",
        "markdown_demo.rs",
        (
            "ui_assets::rgba8_image_state(",
            "ui_assets::ImageColorSpace::Srgb",
        ),
        ("image_asset_state::use_rgba8_image_state(",),
    ),
    (
        "examples",
        "components_gallery.rs",
        (
            "fret_runtime::register_asset_resolver(app, layer.resolver.clone())",
            "fret_fonts::build_imported_font_asset_batch(",
            "ensure_components_gallery_imported_font_asset_resolver(app).replace_batch(&batch)",
        ),
        (),
    ),
    (
        "examples",
        "embedded_viewport_demo.rs",
        (
            "EmbeddedViewportUiAppDriverExt as _",
            ".drive_embedded_viewport()",
        ),
        (),
    ),
)


CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_core_lane_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    imui_examples_src: Path,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    roots = {
        "examples": examples_src,
        "imui_examples": imui_examples_src,
    }
    for policies in (
        THEME_LANE_POLICIES,
        LOCAL_STATE_LANE_POLICIES,
        ASSET_HELPER_POLICIES,
    ):
        for root_name, source_name, required, forbidden in policies:
            path = roots[root_name] / source_name
            check_required_forbidden_markers(
                path,
                read_source(path),
                required=list(required),
                forbidden=list(forbidden),
                failures=failures,
            )
