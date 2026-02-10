use super::*;
use fret_core::{
    PathConstraints, PathMetrics, PathService, PathStyle, Point, Px, Rect, Size, SvgId, SvgService,
    TextBlobId, TextConstraints, TextMetrics, TextService,
};
use std::process::Command;

const ENV_CHILD_REPRO: &str = "FRET_UI_GALLERY_STACK_OVERFLOW_CHILD";

#[derive(Default)]
struct FakeUiServices;

impl TextService for FakeUiServices {
    fn prepare(
        &mut self,
        _input: &fret_core::TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(10.0), Px(10.0)),
                baseline: Px(8.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl PathService for FakeUiServices {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: PathStyle,
        _constraints: PathConstraints,
    ) -> (fret_core::PathId, PathMetrics) {
        (fret_core::PathId::default(), PathMetrics::default())
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl SvgService for FakeUiServices {
    fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
        SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for FakeUiServices {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        false
    }
}

fn run_datatable_layout(configure_stacksafe: bool) {
    if configure_stacksafe {
        stacksafe::set_minimum_stack_size(2 * 1024 * 1024);
        stacksafe::set_stack_allocation_size(8 * 1024 * 1024);
    }

    let mut app = App::new();
    let window = AppWindowId::default();
    let mut state = UiGalleryDriver::build_ui(&mut app, window);
    let _ = app.models_mut().update(&state.selected_page, |v| {
        *v = Arc::<str>::from(PAGE_DATA_TABLE);
    });

    let bounds = Rect::new(
        Point::new(Px(0.0), Px(0.0)),
        Size::new(Px(1280.0), Px(720.0)),
    );
    let mut services = FakeUiServices;

    UiGalleryDriver::render_ui(&mut app, &mut services, window, &mut state, bounds);

    let mut frame =
        fret_ui::UiFrameCx::new(&mut state.ui, &mut app, &mut services, window, bounds, 1.0);
    frame.layout_all();
}

#[test]
fn child_repro_unconfigured() {
    if std::env::var(ENV_CHILD_REPRO).is_err() {
        return;
    }

    let _ = std::thread::Builder::new()
        .name("ui-gallery-stack-overflow-repro".into())
        .stack_size(512 * 1024)
        .spawn(|| run_datatable_layout(false))
        .unwrap()
        .join();
}

#[test]
fn child_repro_configured() {
    if std::env::var(ENV_CHILD_REPRO).is_err() {
        return;
    }

    let _ = std::thread::Builder::new()
        .name("ui-gallery-stack-overflow-repro".into())
        .stack_size(512 * 1024)
        .spawn(|| run_datatable_layout(true))
        .unwrap()
        .join();
}

#[test]
#[ignore]
fn datatable_layout_stack_overflow_repro() {
    let exe = std::env::current_exe().expect("test exe path");

    let status_unconfigured = Command::new(&exe)
        .arg("--exact")
        .arg("driver::stack_overflow_repro_tests::child_repro_unconfigured")
        .env(ENV_CHILD_REPRO, "1")
        .status()
        .expect("spawn child repro (unconfigured)");

    assert!(
        !status_unconfigured.success(),
        "expected unconfigured child to fail (stack overflow); status={status_unconfigured:?}"
    );

    let status_configured = Command::new(&exe)
        .arg("--exact")
        .arg("driver::stack_overflow_repro_tests::child_repro_configured")
        .env(ENV_CHILD_REPRO, "1")
        .status()
        .expect("spawn child repro (configured)");

    assert!(
        status_configured.success(),
        "expected configured child to succeed; status={status_configured:?}"
    );
}
