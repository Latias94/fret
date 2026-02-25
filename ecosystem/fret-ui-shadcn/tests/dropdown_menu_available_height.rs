use fret_app::App;
use fret_core::{
    AppWindowId, FrameId, PathCommand, PathConstraints, PathId, PathMetrics, PathService,
    PathStyle, Point, Px, Rect, SemanticsRole, Size as CoreSize, SvgId, SvgService, TextBlobId,
    TextConstraints, TextMetrics, TextService,
};
use fret_runtime::Model;
use fret_ui::Theme;
use fret_ui::element::{ContainerProps, LayoutStyle, Length};
use fret_ui::tree::UiTree;
use fret_ui_kit::OverlayController;
use fret_ui_shadcn::dropdown_menu::{
    DropdownMenu, DropdownMenuEntry, DropdownMenuItem, DropdownMenuSide,
};
use fret_ui_shadcn::shadcn_themes;
use std::sync::Arc;

struct FakeServices;

impl TextService for FakeServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: CoreSize::new(Px(0.0), Px(0.0)),
                baseline: Px(0.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeServices {
    fn prepare(
        &mut self,
        _commands: &[PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (PathId, PathMetrics) {
        (PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: PathId) {}
}

impl SvgService for FakeServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        true
    }
}

impl fret_core::MaterialService for FakeServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Ok(fret_core::MaterialId::default())
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}

fn render_frame(
    ui: &mut UiTree<App>,
    app: &mut App,
    services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    bounds: Rect,
    open: Model<bool>,
    trigger_top_y: Px,
    request_semantics: bool,
) {
    let next_frame = FrameId(app.frame_id().0.saturating_add(1));
    app.set_frame_id(next_frame);

    OverlayController::begin_frame(app, window);
    let root = fret_ui::declarative::render_root(
        ui,
        app,
        services,
        window,
        bounds,
        "dropdown-menu-available-height",
        move |cx| {
            let open = open.clone();
            let entries =
                (0..64).map(|idx| {
                    DropdownMenuEntry::Item(
                        DropdownMenuItem::new(Arc::<str>::from(format!("Item {idx}")))
                            .test_id(Arc::<str>::from(format!("dropdown-menu-item-{idx}"))),
                    )
                });

            let trigger = cx.pressable(
                fret_ui::element::PressableProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.position = fret_ui::element::PositionStyle::Absolute;
                        layout.inset.left = Some(Px(16.0)).into();
                        layout.inset.top = Some(trigger_top_y).into();
                        layout.size.width = Length::Px(Px(120.0));
                        layout.size.height = Length::Px(Px(40.0));
                        layout
                    },
                    enabled: true,
                    focusable: true,
                    ..Default::default()
                },
                |_cx, _st| Vec::new(),
            );

            let dropdown = DropdownMenu::new(open)
                .side(DropdownMenuSide::Top)
                .window_margin(Px(0.0))
                .side_offset(Px(4.0))
                .into_element(cx, |_cx| trigger, |_cx| entries);

            vec![cx.container(
                ContainerProps {
                    layout: {
                        let mut layout = LayoutStyle::default();
                        layout.size.width = Length::Fill;
                        layout.size.height = Length::Fill;
                        layout
                    },
                    ..Default::default()
                },
                move |_cx| [dropdown],
            )]
        },
    );

    ui.set_root(root);
    OverlayController::render(ui, app, services, window, bounds);
    if request_semantics {
        ui.request_semantics_snapshot();
    }
    ui.layout_all(app, services, bounds, 1.0);
}

#[test]
fn dropdown_menu_content_height_clamps_to_available_height() {
    let window = AppWindowId::default();
    let mut app = App::new();
    let mut ui: UiTree<App> = UiTree::new();
    ui.set_window(window);
    let mut services = FakeServices;

    shadcn_themes::apply_shadcn_new_york_v4(
        &mut app,
        shadcn_themes::ShadcnBaseColor::Neutral,
        shadcn_themes::ShadcnColorScheme::Light,
    );

    let open: Model<bool> = app.models_mut().insert(false);

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        CoreSize::new(Px(480.0), Px(160.0)),
    );
    // Pick a position where the "best fit" side is still small, so available-height clamping is
    // observable while staying deterministic.
    let trigger_top_y = Px(62.0);

    // Frame 1: mount closed so the overlay can read anchor bounds from the previous layout.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        trigger_top_y,
        false,
    );

    let _ = app.models_mut().update(&open, |v| *v = true);

    // Frame 2+: open and render twice so the trigger anchor cache and popper vars settle.
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open.clone(),
        trigger_top_y,
        false,
    );
    render_frame(
        &mut ui,
        &mut app,
        &mut services,
        window,
        bounds,
        open,
        trigger_top_y,
        true,
    );

    let snap = ui.semantics_snapshot().expect("semantics snapshot").clone();
    let content = snap
        .nodes
        .iter()
        .find(|n| n.role == SemanticsRole::Menu)
        .expect("dropdown menu content semantics node");

    let trigger_height = Px(40.0);
    let top_available_height = Px((trigger_top_y.0 - 4.0).max(0.0));
    let bottom_available_height =
        Px((bounds.size.height.0 - (trigger_top_y.0 + trigger_height.0) - 4.0).max(0.0));
    let available_height = if top_available_height.0 >= bottom_available_height.0 {
        top_available_height
    } else {
        bottom_available_height
    };
    let theme = Theme::global(&app).clone();
    let expected_max_height = theme
        .metric_by_key("component.dropdown_menu.max_height")
        .map(|h| Px(h.0.min(available_height.0)))
        .unwrap_or(available_height);

    assert!(
        (content.bounds.size.height.0 - expected_max_height.0).abs() <= 2.0,
        "expected dropdown menu content height to clamp to available height; got {:?}, expected {:?} (available {:?})",
        content.bounds.size.height,
        expected_max_height,
        available_height
    );
}
