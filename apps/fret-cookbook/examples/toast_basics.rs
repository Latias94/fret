use fret::app::prelude::*;
use fret::style::Space;

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
const EFFECT_DEFAULT_TOAST: u64 = 0x7057_0001;
const EFFECT_SUCCESS_TOAST: u64 = 0x7057_0002;
const EFFECT_DISMISS_ALL: u64 = 0x7057_0003;

struct ToastBasicsView;

impl View for ToastBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        cx.actions()
            .transient::<act::DefaultToast>(EFFECT_DEFAULT_TOAST);
        cx.actions()
            .transient::<act::SuccessToast>(EFFECT_SUCCESS_TOAST);
        cx.actions()
            .transient::<act::DismissAll>(EFFECT_DISMISS_ALL);

        if cx.effects().take_transient(EFFECT_DEFAULT_TOAST) {
            cx.effects().toast_message(
                "Hello from Fret",
                shadcn::ToastMessageOptions::new().description("This is a default toast."),
            );
        }
        if cx.effects().take_transient(EFFECT_SUCCESS_TOAST) {
            cx.effects().toast_success(
                "Saved",
                shadcn::ToastMessageOptions::new().description("Everything worked."),
            );
        }
        if cx.effects().take_transient(EFFECT_DISMISS_ALL) {
            cx.effects().toast_dismiss_all();
        }

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

        let card = shadcn::card(|cx| {
            ui::children![
                cx;
                shadcn::card_header(|cx| {
                    ui::children![
                        cx;
                        shadcn::card_title("Toast basics (Sonner)"),
                        shadcn::card_description(
                            "A minimal Sonner integration: render a Toaster and dispatch toast requests from actions.",
                        ),
                    ]
                }),
                shadcn::card_content(|cx| ui::children![cx; buttons]),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        let mut root = fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card);

        // `Toaster` is layout-neutral but must be in the tree so toast layer + store are installed.
        root.push(shadcn::Toaster::new().into_element(cx.elements()));
        root.into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-toast-basics")
        .window("cookbook-toast-basics", (720.0, 360.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<ToastBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
