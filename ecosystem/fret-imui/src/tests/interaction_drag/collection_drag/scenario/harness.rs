use super::super::fixtures::{
    TestCollectionAsset, TestCollectionDragPayload, test_collection_assets,
    test_collection_drag_payload_for_asset,
};
use super::super::*;
use fret_runtime::Model;
use fret_ui_kit::imui::ImUiMultiSelectState;

pub(super) struct CollectionDragScenario {
    window: AppWindowId,
    bounds: Rect,
    ui: UiTree<TestHost>,
    app: TestHost,
    services: FakeTextService,
    assets: Arc<[TestCollectionAsset]>,
    selection_model: Model<ImUiMultiSelectState<Arc<str>>>,
    reverse_order: Rc<Cell<bool>>,
    selected_ids: Rc<RefCell<Vec<Arc<str>>>>,
    preview_ids: Rc<RefCell<Vec<Arc<str>>>>,
    preview_paths: Rc<RefCell<Vec<Arc<str>>>>,
    delivered_ids: Rc<RefCell<Vec<Arc<str>>>>,
    delivered_paths: Rc<RefCell<Vec<Arc<str>>>>,
}

impl CollectionDragScenario {
    pub(super) fn new() -> Self {
        let window = AppWindowId::default();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(360.0), Px(220.0)),
        );

        let mut ui = UiTree::new();
        ui.set_window(window);

        let mut app = TestHost::new();
        app.set_global(PlatformCapabilities::default());

        let selection_model = app
            .models_mut()
            .insert(ImUiMultiSelectState::<Arc<str>>::default());

        Self {
            window,
            bounds,
            ui,
            app,
            services: FakeTextService::default(),
            assets: test_collection_assets(),
            selection_model,
            reverse_order: Rc::new(Cell::new(false)),
            selected_ids: Rc::new(RefCell::new(Vec::new())),
            preview_ids: Rc::new(RefCell::new(Vec::new())),
            preview_paths: Rc::new(RefCell::new(Vec::new())),
            delivered_ids: Rc::new(RefCell::new(Vec::new())),
            delivered_paths: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub(super) fn render_frame(&mut self) {
        let assets = self.assets.clone();
        let selection_model = self.selection_model.clone();
        let reverse_order = self.reverse_order.clone();
        let selected_ids = self.selected_ids.clone();
        let preview_ids = self.preview_ids.clone();
        let preview_paths = self.preview_paths.clone();
        let delivered_ids = self.delivered_ids.clone();
        let delivered_paths = self.delivered_paths.clone();

        let _root = run_frame(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.window,
            self.bounds,
            "imui-collection-dnd",
            |cx| {
                render_collection_drag_surface(
                    cx,
                    assets,
                    selection_model,
                    reverse_order,
                    selected_ids,
                    preview_ids,
                    preview_paths,
                    delivered_ids,
                    delivered_paths,
                )
            },
        );
    }

    pub(super) fn advance_and_render(&mut self) {
        self.app.advance_frame();
        self.render_frame();
    }

    pub(super) fn click_asset(&mut self, asset_id: &str) {
        let point = self.asset_point(asset_id);
        click_at(&mut self.ui, &mut self.app, &mut self.services, point);
    }

    pub(super) fn meta_click_asset(&mut self, asset_id: &str) {
        let point = self.asset_point(asset_id);
        click_at_with_modifiers(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            point,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        );
    }

    pub(super) fn set_reverse_order(&self, reverse_order: bool) {
        self.reverse_order.set(reverse_order);
    }

    pub(super) fn start_drag_to_target(&mut self, asset_id: &str) {
        let drag_source = self.asset_point(asset_id);
        let target = self.target_point();

        pointer_down_at(&mut self.ui, &mut self.app, &mut self.services, drag_source);
        pointer_move_at(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            target,
            MouseButtons {
                left: true,
                ..MouseButtons::default()
            },
        );
    }

    pub(super) fn drop_on_target(&mut self) {
        let target = self.target_point();
        pointer_up_at(&mut self.ui, &mut self.app, &mut self.services, target);
    }

    pub(super) fn selected_ids(&self) -> Vec<Arc<str>> {
        self.selected_ids.borrow().clone()
    }

    pub(super) fn preview_ids(&self) -> Vec<Arc<str>> {
        self.preview_ids.borrow().clone()
    }

    pub(super) fn preview_paths(&self) -> Vec<Arc<str>> {
        self.preview_paths.borrow().clone()
    }

    pub(super) fn delivered_ids(&self) -> Vec<Arc<str>> {
        self.delivered_ids.borrow().clone()
    }

    pub(super) fn delivered_paths(&self) -> Vec<Arc<str>> {
        self.delivered_paths.borrow().clone()
    }

    fn asset_point(&mut self, asset_id: &str) -> Point {
        self.point_for(&format!("imui-collection-dnd.asset.{asset_id}"))
    }

    fn target_point(&mut self) -> Point {
        self.point_for("imui-collection-dnd.target")
    }

    fn point_for(&mut self, test_id: &str) -> Point {
        point_for_test_id(
            &mut self.ui,
            &mut self.app,
            &mut self.services,
            self.bounds,
            test_id,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn render_collection_drag_surface(
    cx: &mut ElementContext<'_, TestHost>,
    assets: Arc<[TestCollectionAsset]>,
    selection_model: Model<ImUiMultiSelectState<Arc<str>>>,
    reverse_order: Rc<Cell<bool>>,
    selected_out: Rc<RefCell<Vec<Arc<str>>>>,
    preview_ids_out: Rc<RefCell<Vec<Arc<str>>>>,
    preview_paths_out: Rc<RefCell<Vec<Arc<str>>>>,
    delivered_ids_out: Rc<RefCell<Vec<Arc<str>>>>,
    delivered_paths_out: Rc<RefCell<Vec<Arc<str>>>>,
) -> crate::Elements {
    crate::imui_raw(cx, |ui| {
        let mut visible_assets = assets.iter().cloned().collect::<Vec<_>>();
        if reverse_order.get() {
            visible_assets.reverse();
        }
        let all_keys = visible_assets
            .iter()
            .map(|asset| asset.id.clone())
            .collect::<Vec<_>>();
        let selection_state = ui
            .cx_mut()
            .app
            .models()
            .get_cloned(&selection_model)
            .unwrap_or_default();

        ui.vertical(|ui| {
            for asset in &visible_assets {
                ui.id(asset.id.clone(), |ui| {
                    let trigger = ui.multi_selectable_with_options(
                        asset.label.clone(),
                        &selection_model,
                        &all_keys,
                        asset.id.clone(),
                        fret_ui_kit::imui::SelectableOptions {
                            test_id: Some(Arc::from(format!(
                                "imui-collection-dnd.asset.{}",
                                asset.id
                            ))),
                            ..Default::default()
                        },
                    );
                    let _ = ui.drag_source(
                        trigger,
                        test_collection_drag_payload_for_asset(
                            &visible_assets,
                            &selection_state,
                            asset,
                        ),
                    );
                });
            }

            let target = ui.button_with_options(
                "Import",
                fret_ui_kit::imui::ButtonOptions {
                    test_id: Some(Arc::from("imui-collection-dnd.target")),
                    ..Default::default()
                },
            );
            let drop = ui.drop_target::<TestCollectionDragPayload>(target);
            preview_ids_out.replace(
                drop.preview_payload()
                    .map(|payload| payload.ids.iter().cloned().collect())
                    .unwrap_or_default(),
            );
            preview_paths_out.replace(
                drop.preview_payload()
                    .map(|payload| payload.paths.iter().cloned().collect())
                    .unwrap_or_default(),
            );
            delivered_ids_out.replace(
                drop.delivered_payload()
                    .map(|payload| payload.ids.iter().cloned().collect())
                    .unwrap_or_default(),
            );
            delivered_paths_out.replace(
                drop.delivered_payload()
                    .map(|payload| payload.paths.iter().cloned().collect())
                    .unwrap_or_default(),
            );
        });

        let state = ui
            .cx_mut()
            .app
            .models()
            .get_cloned(&selection_model)
            .unwrap_or_default();
        selected_out.replace(state.selected().to_vec());
    })
}
