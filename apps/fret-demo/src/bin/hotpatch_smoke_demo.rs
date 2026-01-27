#[cfg(feature = "hotpatch")]
mod hotpatch {
    use fret_app::{App, CommandId};
    use fret_bootstrap::BootstrapBuilder;
    use fret_bootstrap::ui_app_driver::UiAppDriver;
    use fret_core::{AppWindowId, UiServices};
    use fret_runtime::Model;
    use fret_ui::element::AnyElement;
    use fret_ui::{ElementContext, Invalidation, Theme};
    use fret_ui_shadcn as shadcn;

    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    const CMD_INC: &str = "hotpatch_smoke.inc";

    // Change this string and apply a Subsecond patch to confirm that patched code is actually executing.
    const DEMO_HEADLINE: &str = "Hotpatch Smoke Demo (edit me)!!!!!";

    static VIEW_LOG_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct State {
        counter: Model<i64>,
        debug: Model<Arc<str>>,
    }

    pub(super) fn main() -> anyhow::Result<()> {
        eprintln!("[hotpatch_smoke_demo] starting");
        log_line("[hotpatch_smoke_demo] starting");

        dump_env("FRET_HOTPATCH");
        dump_env("FRET_HOTPATCH_DEVSERVER_WS");
        dump_env("FRET_HOTPATCH_BUILD_ID");
        dump_env("FRET_HOTPATCH_DIAG");
        dump_env("DIOXUS_CLI_ENABLED");
        dump_env("DIOXUS_DEVSERVER_IP");
        dump_env("DIOXUS_DEVSERVER_PORT");
        dump_env("DIOXUS_BUILD_ID");
        dump_cwd();
        log_line(&format!(
            "cfg(feature=\"hotpatch\")={}",
            cfg!(feature = "hotpatch")
        ));
        log_line(&format!("cfg(debug_assertions)={}", cfg!(debug_assertions)));

        let driver = UiAppDriver::new("hotpatch-smoke-demo", init_window, view)
            .on_event(on_event)
            .on_command(on_command)
            .into_fn_driver();

        BootstrapBuilder::new(App::new(), driver)
            .with_default_config_files()?
            .with_lucide_icons()
            .run()
            .map_err(anyhow::Error::from)
    }

    fn init_window(app: &mut App, _window: AppWindowId) -> State {
        let counter = app.models_mut().insert(0i64);
        let debug: Model<Arc<str>> = app.models_mut().insert(Arc::<str>::from("ready"));
        State { counter, debug }
    }

    #[inline(never)]
    #[unsafe(export_name = "fret_hotpatch_smoke_demo_view")]
    fn view(
        cx: &mut ElementContext<'_, App>,
        st: &mut State,
    ) -> fret_bootstrap::ui_app_driver::ViewElements {
        if cfg!(debug_assertions)
            && std::env::var_os("DIOXUS_CLI_ENABLED").is_some_and(|v| !v.is_empty())
            && VIEW_LOG_COUNT.fetch_add(1, Ordering::Relaxed) < 3
        {
            log_line(&format!(
                "[hotpatch_smoke_demo] view enter headline={DEMO_HEADLINE}"
            ));
        }

        cx.observe_model(&st.counter, Invalidation::Paint);
        cx.observe_model(&st.debug, Invalidation::Paint);
        let theme = Theme::global(&*cx.app).clone();

        let value = cx
            .app
            .models()
            .read(&st.counter, |v| *v)
            .unwrap_or_default();

        let debug = cx
            .app
            .models()
            .read(&st.debug, |v| v.clone())
            .unwrap_or_else(|_| Arc::<str>::from("<missing debug model>"));

        let content = shadcn::Card::new([
            shadcn::CardHeader::new([
                shadcn::CardTitle::new(DEMO_HEADLINE).into_element(cx),
                shadcn::CardDescription::new(format!(
                    "counter={value} (click, then watch the terminal logs)"
                ))
                .into_element(cx),
                shadcn::CardDescription::new(format!("debug: {debug}")).into_element(cx),
            ])
            .into_element(cx),
            shadcn::CardContent::new([fret_ui_kit::declarative::stack::vstack(
                cx,
                fret_ui_kit::declarative::stack::VStackProps::default()
                    .gap_y(fret_ui_kit::Space::N2),
                |cx| {
                    [
                        shadcn::Button::new("Increment")
                            .on_click(CMD_INC)
                            .into_element(cx),
                        shadcn::Button::new("Increment (Secondary)")
                            .on_click(CMD_INC)
                            .variant(shadcn::ButtonVariant::Secondary)
                            .into_element(cx),
                    ]
                },
            )])
            .into_element(cx),
        ])
        .into_element(cx);

        let bg = theme.color_required("background");
        let wrap = fret_ui_kit::declarative::style::container_props(
            &theme,
            fret_ui_kit::ChromeRefinement::default()
                .bg(fret_ui_kit::ColorRef::Color(bg))
                .p(fret_ui_kit::Space::N6),
            fret_ui_kit::LayoutRefinement::default().w_full().h_full(),
        );

        vec![cx.container(wrap, |_cx| [content])].into()
    }

    fn on_event(
        app: &mut App,
        _services: &mut dyn UiServices,
        _window: AppWindowId,
        _ui: &mut fret_ui::UiTree<App>,
        state: &mut State,
        event: &fret_core::Event,
    ) {
        match event {
            fret_core::Event::Pointer(fret_core::PointerEvent::Down { button, .. }) => {
                let msg = format!("pointer down {button:?}");
                eprintln!("[hotpatch_smoke_demo] {msg}");
                log_line(&format!("[hotpatch_smoke_demo] {msg}"));
                let _ = app
                    .models_mut()
                    .update(&state.debug, |v| *v = Arc::from(msg.as_str()));
            }
            fret_core::Event::Pointer(fret_core::PointerEvent::Up { button, .. }) => {
                let msg = format!("pointer up {button:?}");
                eprintln!("[hotpatch_smoke_demo] {msg}");
                log_line(&format!("[hotpatch_smoke_demo] {msg}"));
                let _ = app
                    .models_mut()
                    .update(&state.debug, |v| *v = Arc::from(msg.as_str()));
            }
            _ => {}
        }
    }

    fn on_command(
        app: &mut App,
        _services: &mut dyn UiServices,
        window: AppWindowId,
        _ui: &mut fret_ui::UiTree<App>,
        state: &mut State,
        cmd: &CommandId,
    ) {
        eprintln!("[hotpatch_smoke_demo] on_command cmd={}", cmd.as_str());
        log_line(&format!(
            "[hotpatch_smoke_demo] on_command cmd={}",
            cmd.as_str()
        ));
        match cmd.as_str() {
            CMD_INC => {
                let _ = app.models_mut().update(&state.counter, |v| *v += 1);
                let value = app
                    .models()
                    .read(&state.counter, |v| *v)
                    .unwrap_or_default();
                eprintln!("[hotpatch_smoke_demo] counter now {value}");
                log_line(&format!("[hotpatch_smoke_demo] counter now {value}"));
                let msg = format!("command {CMD_INC} -> counter {value}");
                let _ = app
                    .models_mut()
                    .update(&state.debug, |v| *v = Arc::from(msg.as_str()));
                app.request_redraw(window);
            }
            _ => {}
        }
    }

    fn log_line(line: &str) {
        let _ = std::fs::create_dir_all(".fret");
        let path = std::path::Path::new(".fret").join("hotpatch_smoke_demo.log");
        let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        else {
            return;
        };

        use std::io::Write as _;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default();
        let _ = writeln!(file, "[{ts}] {line}");
        let _ = file.flush();
    }

    fn dump_env(key: &str) {
        let value = std::env::var(key).unwrap_or_else(|_| "<unset>".to_string());
        let line = format!("{key}={value}");
        eprintln!("[hotpatch_smoke_demo] {line}");
        log_line(&format!("[hotpatch_smoke_demo] {line}"));
    }

    fn dump_cwd() {
        let cwd = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "<unknown>".to_string());
        let line = format!("cwd={cwd}");
        eprintln!("[hotpatch_smoke_demo] {line}");
        log_line(&format!("[hotpatch_smoke_demo] {line}"));
    }
}

#[cfg(feature = "hotpatch")]
fn main() -> anyhow::Result<()> {
    hotpatch::main()
}

#[cfg(not(feature = "hotpatch"))]
fn main() -> anyhow::Result<()> {
    eprintln!(
        "hotpatch_smoke_demo requires `--features hotpatch`.\nTry: cargo run -p fret-demo --bin hotpatch_smoke_demo --features hotpatch"
    );
    Ok(())
}
