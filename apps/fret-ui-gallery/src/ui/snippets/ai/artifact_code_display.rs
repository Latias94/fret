pub const SOURCE: &str = include_str!("artifact_code_display.rs");

// region: example
use fret::app::UiCxActionsExt as _;
use fret::{UiChild, UiCx};
use fret_runtime::Model;
use fret_ui::Invalidation;
use fret_ui::action::{ActionCx, UiActionHost};
use fret_ui_ai as ui_ai;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, Space};
use fret_ui_shadcn::prelude::*;
use std::sync::Arc;

fn status_action(
    status: Model<Arc<str>>,
    message: &'static str,
) -> impl Fn(&mut dyn UiActionHost, ActionCx) + Clone {
    move |host, action_cx| {
        let _ = host
            .models_mut()
            .update(&status, |text| *text = Arc::<str>::from(message));
        host.notify(action_cx);
    }
}

pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let status = cx.local_model_keyed("status", || {
        Arc::<str>::from("Ready to run, copy, regenerate, download, or share.")
    });

    let status_text = cx
        .get_model_cloned(&status, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("Ready to run, copy, regenerate, download, or share."));

    let code: Arc<str> = Arc::from(
        "# Dijkstra's Algorithm implementation\nimport heapq\n\ndef dijkstra(graph, start):\n    distances = {node: float('inf') for node in graph}\n    distances[start] = 0\n    heap = [(0, start)]\n    visited = set()\n\n    while heap:\n        current_distance, current_node = heapq.heappop(heap)\n        if current_node in visited:\n            continue\n        visited.add(current_node)\n\n        for neighbor, weight in graph[current_node].items():\n            distance = current_distance + weight\n            if distance < distances[neighbor]:\n                distances[neighbor] = distance\n                heapq.heappush(heap, (distance, neighbor))\n\n    return distances\n\n# Example graph\ngraph = {\n    'A': {'B': 1, 'C': 4},\n    'B': {'A': 1, 'C': 2, 'D': 5},\n    'C': {'A': 4, 'B': 2, 'D': 1},\n    'D': {'B': 5, 'C': 1}\n}\n\nprint(dijkstra(graph, 'A'))\n",
    );

    let title_block = ui::v_flex(move |cx| {
        vec![
            ui_ai::ArtifactTitle::new("Dijkstra's Algorithm Implementation")
                .test_id("ui-ai-artifact-docs-title")
                .into_element(cx),
            ui_ai::ArtifactDescription::new("Updated 1 minute ago")
                .test_id("ui-ai-artifact-docs-description")
                .into_element(cx),
        ]
    })
    .layout(LayoutRefinement::default().min_w_0().flex_1())
    .gap(Space::N1)
    .items_start()
    .into_element(cx);

    let actions = ui_ai::ArtifactActions::new([
        ui_ai::ArtifactAction::new()
            .label("Run")
            .tooltip("Run code")
            .icon(fret_icons::IconId::new_static("lucide.play"))
            .test_id("ui-ai-artifact-docs-run")
            .on_activate(
                cx.actions()
                    .listen(status_action(status.clone(), "Run action triggered.")),
            )
            .into_element(cx),
        ui_ai::ArtifactAction::new()
            .label("Copy")
            .tooltip("Copy to clipboard")
            .icon(fret_icons::IconId::new_static("lucide.copy"))
            .test_id("ui-ai-artifact-docs-copy")
            .on_activate(
                cx.actions()
                    .listen(status_action(status.clone(), "Copy action triggered.")),
            )
            .into_element(cx),
        ui_ai::ArtifactAction::new()
            .label("Regenerate")
            .tooltip("Regenerate content")
            .icon(fret_icons::IconId::new_static("lucide.refresh-cw"))
            .test_id("ui-ai-artifact-docs-regenerate")
            .on_activate(cx.actions().listen(status_action(
                status.clone(),
                "Regenerate action triggered.",
            )))
            .into_element(cx),
        ui_ai::ArtifactAction::new()
            .label("Download")
            .tooltip("Download file")
            .icon(fret_icons::IconId::new_static("lucide.download"))
            .test_id("ui-ai-artifact-docs-download")
            .on_activate(
                cx.actions()
                    .listen(status_action(status.clone(), "Download action triggered.")),
            )
            .into_element(cx),
        ui_ai::ArtifactAction::new()
            .label("Share")
            .tooltip("Share artifact")
            .icon(fret_icons::IconId::new_static("lucide.share"))
            .test_id("ui-ai-artifact-docs-share")
            .on_activate(
                cx.actions()
                    .listen(status_action(status.clone(), "Share action triggered.")),
            )
            .into_element(cx),
    ])
    .test_id("ui-ai-artifact-docs-actions")
    .into_element(cx);

    let actions_row = ui::h_flex(move |_cx| vec![actions])
        .layout(LayoutRefinement::default().flex_shrink_0())
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

    let code_panel = fret_code_view::code_block_with_header_slots(
        cx,
        &code,
        Some("python"),
        true,
        fret_code_view::CodeBlockUiOptions {
            show_header: false,
            header_divider: false,
            header_background: fret_code_view::CodeBlockHeaderBackground::None,
            show_copy_button: false,
            copy_button_on_hover: true,
            copy_button_placement: fret_code_view::CodeBlockCopyButtonPlacement::Overlay,
            border: false,
            wrap: fret_code_view::CodeBlockWrap::ScrollX,
            disable_ligatures: true,
            disable_contextual_alternates: true,
            max_height: None,
            windowed_lines: false,
            windowed_lines_overscan: 6,
            show_scrollbar_x: true,
            scrollbar_x_on_hover: true,
            show_scrollbar_y: true,
            scrollbar_y_on_hover: true,
        },
        fret_code_view::CodeBlockHeaderSlots::default(),
    );

    let artifact = ui_ai::Artifact::new([
        ui_ai::ArtifactHeader::new([title_block, actions_row])
            .test_id("ui-ai-artifact-docs-header")
            .into_element(cx),
        ui_ai::ArtifactContent::new([code_panel])
            .viewport_test_id("ui-ai-artifact-docs-viewport")
            .refine_style(ChromeRefinement::default().p(Space::N0))
            .into_element(cx),
    ])
    .test_id_root("ui-ai-artifact-docs-root")
    .into_element(cx);

    ui::v_flex(move |cx| {
        vec![
            artifact,
            cx.text(format!("Status: {status_text}"))
                .test_id("ui-ai-artifact-docs-status"),
        ]
    })
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .gap(Space::N3)
    .items_start()
    .into_element(cx)
}
// endregion: example
