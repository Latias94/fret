use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use fret_core::scene::DashPatternV1;
use fret_core::{
    AppWindowId, PathCommand, PathConstraints, PathId, PathMetrics, PathStyle, Point, Px, Rect,
    Scene, Size, SvgId, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
    Transform2D,
};
use fret_ui::retained_bridge::Widget as _;
use fret_ui::{Invalidation, UiTree};

use crate::core::{Edge, EdgeId, EdgeKind};
use crate::ui::{EdgeRenderHint, NodeGraphCanvas, NodeGraphSkin, NodeGraphStyle};

use super::{
    TestUiHostImpl, insert_graph_view_editor_config_with, make_test_graph_two_nodes_with_ports,
};

fn bounds() -> Rect {
    Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(800.0), Px(600.0)),
    )
}

fn paint_once(
    canvas: &mut NodeGraphCanvas,
    host: &mut TestUiHostImpl,
    tree: &mut UiTree<TestUiHostImpl>,
    services: &mut CountingServices,
    bounds: Rect,
) -> Scene {
    let mut scene = Scene::default();
    let mut observe_model = |_id, _inv: Invalidation| {};
    let mut observe_global = |_id, _inv: Invalidation| {};

    let mut cx = fret_ui::retained_bridge::PaintCx::new(
        host,
        tree,
        fret_core::NodeId::default(),
        Some(AppWindowId::default()),
        None,
        &[],
        bounds,
        1.0,
        Transform2D::IDENTITY,
        None,
        services,
        &mut observe_model,
        &mut observe_global,
        &mut scene,
    );

    canvas.paint(&mut cx);
    scene
}

#[derive(Default)]
struct CountingServices {
    path_prepare: usize,
}

impl TextService for CountingServices {
    fn prepare(
        &mut self,
        _input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for CountingServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        self.path_prepare += 1;
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl fret_core::SvgService for CountingServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for CountingServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

#[derive(Debug)]
struct DashSkin {
    rev: AtomicU64,
    dash: std::sync::Mutex<Option<DashPatternV1>>,
}

impl DashSkin {
    fn new() -> Self {
        Self {
            rev: AtomicU64::new(1),
            dash: std::sync::Mutex::new(None),
        }
    }

    fn set_dash(&self, dash: Option<DashPatternV1>) {
        *self.dash.lock().expect("dash lock") = dash;
        self.rev.fetch_add(1, Ordering::Relaxed);
    }
}

impl NodeGraphSkin for DashSkin {
    fn revision(&self) -> u64 {
        self.rev.load(Ordering::Relaxed)
    }

    fn edge_render_hint(
        &self,
        _graph: &crate::core::Graph,
        _edge: EdgeId,
        _style: &NodeGraphStyle,
        base: &EdgeRenderHint,
        _selected: bool,
        _hovered: bool,
    ) -> EdgeRenderHint {
        let dash = *self.dash.lock().expect("dash lock");
        let mut hint = base.clone();
        hint.dash = dash;
        hint
    }
}

#[test]
fn wire_path_cache_key_includes_dash_pattern() {
    let (mut graph_value, _a, _a_in, a_out, _b, b_in) = make_test_graph_two_nodes_with_ports();
    let edge_id = EdgeId::new();
    graph_value.edges.insert(
        edge_id,
        Edge {
            kind: EdgeKind::Data,
            from: a_out,
            to: b_in,
            selectable: None,
            deletable: None,
            reconnectable: None,
        },
    );

    let mut host = TestUiHostImpl::default();
    let (graph, view, editor_config) =
        insert_graph_view_editor_config_with(&mut host, graph_value, |state| {
            state.runtime_tuning.only_render_visible_elements = false;
            state.interaction.frame_view_duration_ms = 0;
        });

    let _ = view.update(&mut host, |s, _cx| {
        s.zoom = 1.0;
    });

    let skin = Arc::new(DashSkin::new());
    let mut canvas = new_canvas!(host, graph, view, editor_config).with_skin(skin.clone());

    let mut tree = UiTree::<TestUiHostImpl>::default();
    let mut services = CountingServices::default();

    skin.set_dash(None);
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());
    let prepares0 = services.path_prepare;

    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());
    let prepares1 = services.path_prepare;
    assert_eq!(
        prepares1, prepares0,
        "wire path should be cached when dash settings are unchanged"
    );

    skin.set_dash(Some(DashPatternV1::new(Px(6.0), Px(4.0), Px(0.0))));
    let _ = paint_once(&mut canvas, &mut host, &mut tree, &mut services, bounds());
    let prepares2 = services.path_prepare;
    assert!(
        prepares2 > prepares1,
        "dash pattern must affect the cached wire path key (avoid reusing a non-dashed path)"
    );
}
