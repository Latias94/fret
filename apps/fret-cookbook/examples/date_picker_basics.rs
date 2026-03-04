use fret::prelude::*;
use time::{Date, Month};

const TEST_ID_ROOT: &str = "cookbook.date_picker_basics.root";
const TEST_ID_PICKER: &str = "cookbook.date_picker_basics.picker";
const TEST_ID_SELECTED: &str = "cookbook.date_picker_basics.selected";

struct DatePickerBasicsView {
    open: Model<bool>,
    selected: Model<Option<time::Date>>,
}

impl View for DatePickerBasicsView {
    fn init(app: &mut App, _window: AppWindowId) -> Self {
        let open = app.models_mut().insert(false);
        let default_selected =
            Date::from_calendar_date(2026, Month::January, 15).expect("valid date");
        let selected = app.models_mut().insert(Some(default_selected));
        Self { open, selected }
    }

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let selected = cx
            .watch_model(&self.selected)
            .layout()
            .copied_or_default()
            .map(|d| d.to_string())
            .unwrap_or_else(|| "<none>".to_string());

        let header = shadcn::CardHeader::new([
            shadcn::CardTitle::new("Date picker basics").into_element(cx),
            shadcn::CardDescription::new(
                "A minimal DatePicker example (controlled models: open/month/selected).",
            )
            .into_element(cx),
        ])
        .into_element(cx);

        let picker = shadcn::DatePicker::new_controllable(
            cx.elements(),
            Some(self.selected.clone()),
            None,
            Some(self.open.clone()),
            false,
        )
        .format_selected_iso()
        .into_element(cx)
        .test_id(TEST_ID_PICKER);

        let selected_row = ui::h_flex(cx, |cx| {
            [
                shadcn::Label::new("Selected:").into_element(cx),
                shadcn::Badge::new(selected)
                    .variant(shadcn::BadgeVariant::Secondary)
                    .into_element(cx)
                    .test_id(TEST_ID_SELECTED),
            ]
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let card = shadcn::Card::new([
            header,
            shadcn::CardContent::new([ui::v_flex(cx, |_cx| [picker, selected_row])
                .gap(Space::N3)
                .into_element(cx)])
            .into_element(cx),
        ])
        .ui()
        .w_full()
        .max_w(Px(720.0))
        .into_element(cx);

        fret_cookbook::scaffold::centered_page_muted(cx, TEST_ID_ROOT, card).into()
    }
}

fn main() -> anyhow::Result<()> {
    FretApp::new("cookbook-date-picker-basics")
        .window("cookbook-date-picker-basics", (720.0, 420.0))
        .config_files(false)
        .install_app(fret_cookbook::install_cookbook_defaults)
        .run_view::<DatePickerBasicsView>()
        .map_err(anyhow::Error::from)
}
