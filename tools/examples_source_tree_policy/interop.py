from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


LOW_LEVEL_INTEROP_DIRECT_LEAF_SOURCES = [
    (
        "external_texture_imports_demo.rs",
        [
            "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalTextureImportsView) -> fret::Ui",
            "use fret::advanced::view::AppRenderDataExt as _;",
            "let show = cx.data().selector_model_layout(&st.show, |show| show);",
            "let theme = cx.theme().snapshot();",
            "cx.viewport_surface_props(ViewportSurfaceProps {",
            ".test_id(\"external-texture-imports-root\"),",
        ],
        [
            "fn external_texture_imports_root(",
            "cx.observe_model(&st.show, Invalidation::Layout);",
            "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
            "Theme::global(&*cx.app).snapshot()",
        ],
    ),
    (
        "external_texture_imports_web_demo.rs",
        [
            "use fret::advanced::view::AppRenderDataExt as _;",
            "let show = cx.data().selector_model_layout(&show_model, |show| show);",
            "let theme = cx.theme().snapshot();",
            "cx.viewport_surface_props(ViewportSurfaceProps {",
            ".test_id(\"external-texture-imports-web-root\"),",
            "make_panel(cx, fret_core::ViewportFit::Contain, \"ext-tex-web-contain\")",
        ],
        [
            "fn external_texture_imports_web_root(",
            "cx.observe_model(&show_model, Invalidation::Layout);",
            "cx.app.models().read(&show_model, |v| *v).unwrap_or(true)",
            "Theme::global(&*cx.app).snapshot()",
        ],
    ),
    (
        "external_video_imports_avf_demo.rs",
        [
            "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsAvfView) -> fret::Ui",
            "use fret::advanced::view::AppRenderDataExt as _;",
            "let show = cx.data().selector_model_layout(&st.show, |show| show);",
            "let theme = cx.theme().snapshot();",
            "cx.viewport_surface_props(ViewportSurfaceProps {",
            ".test_id(\"external-video-imports-avf-root\"),",
        ],
        [
            "fn external_video_imports_avf_root(",
            "cx.observe_model(&st.show, Invalidation::Layout);",
            "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
            "Theme::global(&*cx.app).snapshot()",
        ],
    ),
    (
        "external_video_imports_mf_demo.rs",
        [
            "fn render_view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsMfView) -> fret::Ui",
            "use fret::advanced::view::AppRenderDataExt as _;",
            "let show = cx.data().selector_model_layout(&st.show, |show| show);",
            "let theme = cx.theme().snapshot();",
            "cx.viewport_surface_props(ViewportSurfaceProps {",
            ".test_id(\"external-video-imports-mf-root\"),",
        ],
        [
            "fn external_video_imports_mf_root(",
            "cx.observe_model(&st.show, Invalidation::Layout);",
            "cx.app.models().read(&st.show, |v| *v).unwrap_or(true)",
            "Theme::global(&*cx.app).snapshot()",
        ],
    ),
    (
        "chart_declarative_demo.rs",
        [
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "let _ = self.engine.paint(cx).read_ref(|_| ());",
            "chart_canvas_panel_in(cx, props).into()",
        ],
        [
            "fn chart_declarative_root(",
            "cx.elements().observe_model(&self.engine, Invalidation::Paint);",
            "chart_canvas_panel(cx.elements(), props).into()",
        ],
    ),
    (
        "node_graph_demo.rs",
        [
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "self.surface.observe_in(cx);",
            "node_graph_surface_in(cx, props).into()",
        ],
        [
            "fn node_graph_root(",
            "self.surface.observe(cx.elements());",
            "node_graph_surface(cx.elements(), props).into()",
        ],
    ),
]

CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_low_level_interop_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    for name, required, forbidden in LOW_LEVEL_INTEROP_DIRECT_LEAF_SOURCES:
        path = examples_src / name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )
