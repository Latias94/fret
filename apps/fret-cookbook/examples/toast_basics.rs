use fret::prelude::*;
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
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        // `Sonner::global` needs `UiHost`, which we have during render (but not inside handlers).
        // Capture a clone into action handlers.
        let sonner = shadcn::Sonner::global(cx.app);

        cx.on_action::<act::DefaultToast>({
            let sonner = sonner.clone();
            move |host, acx| {
                sonner.toast_message(
                    host,
                    acx.window,
                    "Hello from Fret",
                    shadcn::ToastMessageOptions::new().description("This is a default toast."),
                );
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action::<act::SuccessToast>({
            let sonner = sonner.clone();
            move |host, acx| {
                sonner.toast_success_message(
                    host,
                    acx.window,
                    "Saved",
                    shadcn::ToastMessageOptions::new().description("Everything worked."),
                );
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action::<act::DismissAll>({
            let sonner = sonner.clone();
            move |host, acx| {
                sonner.dismiss_all(host, acx.window);
                host.request_redraw(acx.window);
                host.notify(acx);
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

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Toast basics (Sonner)").into_element(cx),
            shadcn::CardDescription::new(
                "A minimal Sonner integration: render a Toaster and dispatch toast requests from actions.",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let buttons = ui::h_flex(|cx| {
            [
                shadcn::Button::new("Default toast")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::DefaultToast)
                    .test_id(TEST_ID_DEFAULT)
                    .into_element(cx),
                shadcn::Button::new("Success toast")
                    .variant(shadcn::ButtonVariant::Outline)
                    .action(act::SuccessToast)
                    .test_id(TEST_ID_SUCCESS)
                    .into_element(cx),
                shadcn::Button::new("Dismiss all")
                    .variant(shadcn::ButtonVariant::Secondary)
                    .action(act::DismissAll)
                    .test_id(TEST_ID_DISMISS_ALL)
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let card =
            shadcn::Card::new([header, shadcn::CardContent::new([buttons]).into_element(cx)])
                .ui()
                .w_full()
                .max_w(Px(720.0))
                .into_element(cx);

        let toaster = shadcn::Toaster::new().into_element(cx);

        // `Toaster` is layout-neutral but must be in the tree so toast layer + store are installed.
        let body = ui::v_flex(|_cx| [card, toaster])
            .gap(Space::N4)
            .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, body).into()
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
