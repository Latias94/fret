use fret::app::prelude::*;
use fret_ui::CommandAvailability;

mod act {
    fret::actions!([
        DefaultToast = "cookbook.toast_basics.default_toast.v1",
        SuccessToast = "cookbook.toast_basics.success_toast.v1",
        DismissAll = "cookbook.toast_basics.dismiss_all.v1"
    ]);
}

const TEST_ID_ROOT: &str = "cookbook.toast_basics.root";
const TEST_ID_DEFAULT: &str = "cookbook.toast_basics.default";
const TEST_ID_SUCCESS: &str = "cookbook.toast_basics.success";
const TEST_ID_DISMISS_ALL: &str = "cookbook.toast_basics.dismiss_all";

struct ToastBasicsView;

impl View for ToastBasicsView {
    fn init(_app: &mut KernelApp, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        // Toast actions stay on the advanced helper because Sonner dispatch is a host-owned
        // imperative integration: the handler needs `UiActionHost` + window, and the default
        // model/transient teaching paths do not expose that host surface directly.
        let sonner = shadcn::Sonner::global(cx.app);

        cx.on_action_notify::<act::DefaultToast>({
            let sonner = sonner.clone();
            move |host, acx| {
                sonner.toast_message(
                    host,
                    acx.window,
                    "Hello from Fret",
                    shadcn::ToastMessageOptions::new().description("This is a default toast."),
                );
                true
            }
        });

        cx.on_action_notify::<act::SuccessToast>({
            let sonner = sonner.clone();
            move |host, acx| {
                sonner.toast_success_message(
                    host,
                    acx.window,
                    "Saved",
                    shadcn::ToastMessageOptions::new().description("Everything worked."),
                );
                true
            }
        });

        cx.on_action_notify::<act::DismissAll>({
            let sonner = sonner.clone();
            move |host, acx| {
                sonner.dismiss_all(host, acx.window);
                true
            }
        });

        cx.on_action_availability::<act::DefaultToast>(|_host, _acx| {
            CommandAvailability::Available
        });
        cx.on_action_availability::<act::SuccessToast>(|_host, _acx| {
            CommandAvailability::Available
        });
        cx.on_action_availability::<act::DismissAll>(|_host, _acx| CommandAvailability::Available);

        let buttons = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Button::new("Default toast")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::DefaultToast)
                    .test_id(TEST_ID_DEFAULT),
                shadcn::Button::new("Success toast")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::SuccessToast)
                    .test_id(TEST_ID_SUCCESS),
                shadcn::Button::new("Dismiss all")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::DismissAll)
                    .test_id(TEST_ID_DISMISS_ALL),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let card = shadcn::Card::build(|cx, out| {
            out.push_ui(
                cx,
                shadcn::CardHeader::build(|cx, out| {
                    out.push_ui(cx, shadcn::CardTitle::new("Toast basics (Sonner)"));
                    out.push_ui(
                        cx,
                        shadcn::CardDescription::new(
                            "A minimal Sonner integration: render a Toaster and dispatch toast requests from actions.",
                        ),
                    );
                }),
            );
            out.push_ui(
                cx,
                shadcn::CardContent::build(|cx, out| {
                    out.push_ui(cx, buttons);
                }),
            );
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        let page = fret_cookbook::scaffold::centered_page_muted_ui(cx, TEST_ID_ROOT, card);
        let toaster = shadcn::Toaster::new();

        // `Toaster` is layout-neutral but must be in the tree so toast layer + store are installed.
        ui::stack(|cx| ui::children![cx; page, toaster])
            .size_full()
            .into_element(cx)
            .into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-toast-basics")
        .window("cookbook-toast-basics", (720.0, 360.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<ToastBasicsView>()
        .map_err(anyhow::Error::from)
}
