use fret::app::prelude::*;
use fret::style::Space;
use fret_ui_kit::headless::calendar::CalendarMonth;
use time::{Date, Month};

const TEST_ID_ROOT: &str = "cookbook.date_picker_basics.root";
const TEST_ID_PICKER: &str = "cookbook.date_picker_basics.picker";
const TEST_ID_SELECTED: &str = "cookbook.date_picker_basics.selected";

struct DatePickerBasicsView;

impl View for DatePickerBasicsView {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let initial_selected =
            Date::from_calendar_date(2026, Month::January, 15).expect("valid date");
        let open_state = cx.state().local_init(|| false);
        let month_state = cx
            .state()
            .local_init(|| CalendarMonth::from_date(initial_selected));
        let selected_state = cx.state().local_init(|| Some(initial_selected));

        let selected = cx
            .state()
            .watch(&selected_state)
            .layout()
            .value_or_default()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "<none>".to_string());

        let header = shadcn::card_header(|cx| {
            ui::children![cx;
                shadcn::card_title("Date picker basics"),
                shadcn::card_description(
                    "A minimal DatePicker example (local state bridged into the controlled DatePicker surface).",
                ),
            ]
        });

        let picker = shadcn::DatePicker::new(&open_state, &month_state, &selected_state)
            .format_selected_iso()
            .into_element(cx)
            .test_id(TEST_ID_PICKER);

        let selected_row = ui::h_flex(|cx| {
            ui::children![
                cx;
                shadcn::Label::new("Selected:"),
                shadcn::Badge::new(selected)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .test_id(TEST_ID_SELECTED),
            ]
        })
        .gap(Space::N2)
        .items_center();

        let card = shadcn::card(|cx| {
            ui::children![cx;
                header,
                shadcn::card_content(|cx| {
                    ui::children![cx;
                        ui::v_flex(|cx| ui::children![cx; picker, selected_row]).gap(Space::N3)
                    ]
                }),
            ]
        })
        .ui()
        .w_full()
        .max_w(Px(720.0));

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-date-picker-basics")
        .window("cookbook-date-picker-basics", (720.0, 420.0))
        .config_files(false)
        .setup(fret_cookbook::install_cookbook_defaults)
        .view::<DatePickerBasicsView>()?
        .run()
        .map_err(anyhow::Error::from)
}
