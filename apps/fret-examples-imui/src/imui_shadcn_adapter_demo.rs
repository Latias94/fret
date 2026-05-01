//! Product-validation IMUI surface for the shared control-chrome lane.
//!
//! This stays intentionally small: downstream authors should be able to look at one compact window
//! and tell that IMUI helpers now read like real controls even before shell-specific recipes layer
//! on more product chrome.

use std::sync::Arc;

use fret::{FretApp, advanced::prelude::*, imui::prelude::*};
use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation};
use fret_ui_kit::imui::TableSortDirection;
use fret_ui_kit::{ColorRef, Space, UiExt as _, ui};
use fret_ui_shadcn::facade as shadcn;

const TEST_ID_ROOT: &str = "imui-shadcn-demo.root";
const TEST_ID_HEADER: &str = "imui-shadcn-demo.header";
const TEST_ID_CONTROL_CARD: &str = "imui-shadcn-demo.controls.card";
const TEST_ID_SUMMARY_CARD: &str = "imui-shadcn-demo.summary.card";
const TEST_ID_INSPECTOR_CARD: &str = "imui-shadcn-demo.inspector.card";
const TEST_ID_INCREMENT: &str = "imui-shadcn-demo.controls.increment";
const TEST_ID_ENABLED: &str = "imui-shadcn-demo.controls.enabled";
const TEST_ID_VALUE: &str = "imui-shadcn-demo.controls.value";
const TEST_ID_MODE: &str = "imui-shadcn-demo.controls.mode";
const TEST_ID_DRAFT: &str = "imui-shadcn-demo.controls.draft";
const TEST_ID_SUMMARY_COUNT: &str = "imui-shadcn-demo.summary.count";
const TEST_ID_SUMMARY_ENABLED: &str = "imui-shadcn-demo.summary.enabled";
const TEST_ID_SUMMARY_VALUE: &str = "imui-shadcn-demo.summary.value";
const TEST_ID_SUMMARY_MODE: &str = "imui-shadcn-demo.summary.mode";
const TEST_ID_SUMMARY_DRAFT: &str = "imui-shadcn-demo.summary.draft";
const TEST_ID_TABLE: &str = "imui-shadcn-demo.inspector.table";
const TEST_ID_TABLE_WIDTHS: &str = "imui-shadcn-demo.inspector.widths";
const TEST_ID_RECENT_LIST: &str = "imui-shadcn-demo.inspector.recent";

const STACK_BREAKPOINT_PX: f32 = 840.0;
const COMPACT_SURFACE_WIDTH_PX: f32 = 1024.0;
const COMPACT_SURFACE_HEIGHT_PX: f32 = 700.0;
const SIDE_COLUMN_WIDTH: Px = Px(352.0);
const RECENT_VIEWPORT_HEIGHT_COMPACT: Px = Px(56.0);
const RECENT_VIEWPORT_HEIGHT_REGULAR: Px = Px(156.0);

struct ImUiShadcnAdapterView;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InspectorSort {
    FieldAscending,
    FieldDescending,
}

#[derive(Clone, Copy)]
struct InspectorColumnWidths {
    signal: Px,
    state: Px,
    field: Px,
    value: Px,
    source: Px,
}

impl Default for InspectorColumnWidths {
    fn default() -> Self {
        Self {
            signal: Px(108.0),
            state: Px(176.0),
            field: Px(104.0),
            value: Px(120.0),
            source: Px(72.0),
        }
    }
}

impl InspectorSort {
    fn direction(self) -> TableSortDirection {
        match self {
            Self::FieldAscending => TableSortDirection::Ascending,
            Self::FieldDescending => TableSortDirection::Descending,
        }
    }

    fn toggled(self) -> Self {
        match self {
            Self::FieldAscending => Self::FieldDescending,
            Self::FieldDescending => Self::FieldAscending,
        }
    }

    fn sort_rows(self, rows: &mut [InspectorRow]) {
        rows.sort_by(|a, b| a.field.cmp(b.field).then(a.key.cmp(b.key)));
        if self == Self::FieldDescending {
            rows.reverse();
        }
    }
}

#[derive(Clone)]
struct InspectorRow {
    key: &'static str,
    field: &'static str,
    value: Arc<str>,
    source: &'static str,
}

pub fn run() -> anyhow::Result<()> {
    FretApp::new("imui-shadcn-adapter-demo")
        .window("imui_shadcn_adapter_demo", (960.0, 620.0))
        .view::<ImUiShadcnAdapterView>()?
        .run()?;
    Ok(())
}

impl View for ImUiShadcnAdapterView {
    fn init(app: &mut KernelApp, _window: AppWindowId) -> Self {
        shadcn::themes::apply_shadcn_new_york(
            app,
            shadcn::themes::ShadcnBaseColor::Slate,
            shadcn::themes::ShadcnColorScheme::Light,
        );

        Self
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let viewport = cx.environment_viewport_bounds(Invalidation::Layout);
        let stack_cards = viewport.size.width.0 < STACK_BREAKPOINT_PX;
        let compact_surface = viewport.size.width.0 < COMPACT_SURFACE_WIDTH_PX
            || viewport.size.height.0 < COMPACT_SURFACE_HEIGHT_PX;
        let content_gap = if compact_surface {
            Space::N2
        } else {
            Space::N4
        };
        let surface_padding = if compact_surface {
            Space::N2
        } else {
            Space::N4
        };

        let count_state = cx.state().local_init(|| 0u32);
        let enabled_state = cx.state().local_init(|| false);
        let value_state = cx.state().local_init(|| 32.0f32);
        let mode_state = cx.state().local_init(|| None::<Arc<str>>);
        let draft_state = cx.state().local_init(String::new);
        let inspector_sort_state = cx.state().local_init(|| InspectorSort::FieldAscending);
        let inspector_widths_state = cx.state().local_init(InspectorColumnWidths::default);

        let count = count_state.layout_value(cx);
        let inspector_sort = inspector_sort_state.layout_value(cx);
        let inspector_widths = inspector_widths_state.layout_value(cx);
        let enabled = enabled_state.paint_value(cx);
        let value = value_state.paint_value(cx);
        let mode = mode_state.paint_value(cx);
        let draft = draft_state.paint_value(cx);

        let mode_label: Arc<str> = mode.unwrap_or_else(|| Arc::from("none"));
        let draft_label: Arc<str> = if draft.is_empty() {
            Arc::from("(empty)")
        } else {
            Arc::from(draft.clone())
        };

        imui_in(cx, |ui| {
            let muted_bg = ColorRef::Color(ui.cx_mut().theme().color_token("muted"));

            let root = ui::container(move |cx: &mut ElementContext<'_, KernelApp>| {
                let header = {
                    let badges = ui::h_flex(move |cx| {
                        vec![
                            shadcn::Badge::new("Immediate flow")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                            shadcn::Badge::new("Shared chrome")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                            shadcn::Badge::new("Compact-safe")
                                .variant(shadcn::BadgeVariant::Secondary)
                                .into_element(cx),
                        ]
                    })
                    .gap(Space::N2)
                    .wrap()
                    .into_element(cx);

                    let header_copy = if compact_surface {
                        "Compact proof that shared IMUI controls already read like controls inside a shadcn shell."
                    } else {
                        "The immediate-mode helpers remain model-driven; the surrounding shadcn shell only proves the shared control family now reads as real controls instead of passive text."
                    };
                    let content = shadcn::CardContent::new([
                        ui::text(header_copy)
                        .text_sm()
                        .wrap(fret_core::TextWrap::Word)
                        .into_element(cx),
                        badges,
                    ])
                    .into_element(cx);

                    shadcn::Card::new([
                        shadcn::CardHeader::new([
                            shadcn::CardTitle::new("IMUI + shadcn adapter proof")
                                .into_element(cx),
                            shadcn::CardDescription::new(if compact_surface {
                                "Compact proof surface for control discoverability and live state feedback."
                            } else {
                                "One compact downstream-facing surface for control discoverability, compact tool panels, and live state feedback."
                            })
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        content,
                    ])
                    .into_element(cx)
                    .test_id(TEST_ID_HEADER)
                };

                let controls_card = {
                    let count_state = count_state.clone();
                    let enabled_state = enabled_state.clone();
                    let value_state = value_state.clone();
                    let mode_state = mode_state.clone();
                    let draft_state = draft_state.clone();

                    let control_surface =
                        ui::container(move |cx: &mut ElementContext<'_, KernelApp>| {
                            imui(cx, move |ui| {
                                let select_items = [
                                    Arc::<str>::from("Alpha"),
                                    Arc::<str>::from("Beta"),
                                    Arc::<str>::from("Gamma"),
                                ];

                                if compact_surface {
                                    let note = {
                                        let cx = ui.cx_mut();
                                        ui::text(
                                            "Shared helpers should already feel interactive in a compact tool panel.",
                                        )
                                        .text_xs()
                                        .wrap(fret_core::TextWrap::Word)
                                        .into_element(cx)
                                    };
                                    ui.add_ui(note);
                                } else {
                                    let heading = {
                                        let cx = ui.cx_mut();
                                        ui::text("Interactive controls")
                                            .text_sm()
                                            .font_semibold()
                                            .into_element(cx)
                                    };
                                    ui.add_ui(heading);

                                    let note = {
                                        let cx = ui.cx_mut();
                                        ui::text(
                                            "These are the shared IMUI helpers downstream authors actually call. The shell around them should not need to compensate for weak default affordance.",
                                        )
                                        .text_xs()
                                        .wrap(fret_core::TextWrap::Word)
                                        .into_element(cx)
                                    };
                                    ui.add_ui(note);
                                }

                                let increment = ui.button_with_options(
                                    "Increment count",
                                    kit::ButtonOptions {
                                        test_id: Some(Arc::from(TEST_ID_INCREMENT)),
                                        ..Default::default()
                                    },
                                );
                                if increment.clicked() {
                                    let _ = count_state
                                        .update_in(ui.cx_mut().app.models_mut(), |v| *v += 1);
                                }

                                let _ = ui.switch_model_with_options(
                                    "Enabled (switch)",
                                    enabled_state.model(),
                                    kit::SwitchOptions {
                                        test_id: Some(Arc::from(TEST_ID_ENABLED)),
                                        ..Default::default()
                                    },
                                );

                                let _ = ui.slider_f32_model_with_options(
                                    "Value",
                                    value_state.model(),
                                    kit::SliderOptions {
                                        min: 0.0,
                                        max: 100.0,
                                        step: 1.0,
                                        test_id: Some(Arc::from(TEST_ID_VALUE)),
                                        ..Default::default()
                                    },
                                );

                                let _ = ui.combo_model_with_options(
                                    "imui-shadcn-demo.mode.popup",
                                    "Mode",
                                    mode_state.model(),
                                    &select_items,
                                    kit::ComboModelOptions {
                                        test_id: Some(Arc::from(TEST_ID_MODE)),
                                        ..Default::default()
                                    },
                                );

                                let _ = ui.input_text_model_with_options(
                                    draft_state.model(),
                                    kit::InputTextOptions {
                                        placeholder: Some(Arc::from("Type some text...")),
                                        test_id: Some(Arc::from(TEST_ID_DRAFT)),
                                        ..Default::default()
                                    },
                                );

                                ui.separator_text(if compact_surface {
                                    "Proof goals"
                                } else {
                                    "Why this proof exists"
                                });
                                ui.bullet_text(
                                    if compact_surface {
                                        "Controls should look interactive by default."
                                    } else {
                                        "A control should already look interactive before a recipe crate adds more product chrome."
                                    },
                                );
                                ui.bullet_text(
                                    if compact_surface {
                                        "Compact tool panels should stay readable without demo-local width patches."
                                    } else {
                                        "Compact tool panels should not need demo-local width patches to keep buttons, fields, and triggers readable."
                                    },
                                );
                            })
                        })
                        .into_element(cx);

                    shadcn::Card::new([
                        shadcn::CardHeader::new([
                            shadcn::CardTitle::new("Control lab").into_element(cx),
                            shadcn::CardDescription::new(
                                "Direct IMUI helpers with stable selectors for screenshot/layout diagnostics.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx),
                        shadcn::CardContent::new([control_surface]).into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id(TEST_ID_CONTROL_CARD)
                };

                let summary_card = {
                    let summary_mode_label = mode_label.clone();
                    let summary_draft_label = draft_label.clone();
                    let rows = ui::h_flex(move |cx| {
                        vec![
                            summary_badge(
                                cx,
                                Arc::<str>::from(format!("count: {count}")),
                                TEST_ID_SUMMARY_COUNT,
                            ),
                            summary_badge(
                                cx,
                                Arc::<str>::from(format!("enabled: {enabled}")),
                                TEST_ID_SUMMARY_ENABLED,
                            ),
                            summary_badge(
                                cx,
                                Arc::<str>::from(format!("value: {value:.1}")),
                                TEST_ID_SUMMARY_VALUE,
                            ),
                            summary_badge(
                                cx,
                                Arc::<str>::from(format!("mode: {summary_mode_label}")),
                                TEST_ID_SUMMARY_MODE,
                            ),
                            summary_badge(
                                cx,
                                Arc::<str>::from(format!("draft: {summary_draft_label}")),
                                TEST_ID_SUMMARY_DRAFT,
                            ),
                        ]
                    })
                    .gap(Space::N2)
                    .wrap()
                    .w_full()
                    .min_w_0()
                    .into_element(cx);

                    let summary_header = if compact_surface {
                        shadcn::CardHeader::new([shadcn::CardTitle::new("Live summary")
                            .into_element(cx)])
                        .into_element(cx)
                    } else {
                        shadcn::CardHeader::new([
                            shadcn::CardTitle::new("Live summary").into_element(cx),
                            shadcn::CardDescription::new(
                                "The proof surface should update live as soon as the shared IMUI helpers fire.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    };

                    shadcn::Card::new([
                        summary_header,
                        shadcn::CardContent::new([rows]).into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id(TEST_ID_SUMMARY_CARD)
                };

                let inspector_card = {
                    let count = count;
                    let enabled = enabled;
                    let value = value;
                    let mode_label = mode_label.clone();
                    let draft = draft.clone();
                    let inspector_sort_state = inspector_sort_state.clone();
                    let inspector_widths_state = inspector_widths_state.clone();
                    let inspector_widths = inspector_widths;

                    let inspector_surface =
                        ui::container(move |cx: &mut ElementContext<'_, KernelApp>| {
                            imui(cx, move |ui| {
                                let mut inspector_rows = vec![
                                    InspectorRow {
                                        key: "count",
                                        field: "Count",
                                        value: Arc::<str>::from(format!("{count}")),
                                        source: "Button",
                                    },
                                    InspectorRow {
                                        key: "enabled",
                                        field: "Enabled",
                                        value: Arc::<str>::from(format!("{enabled}")),
                                        source: "Switch",
                                    },
                                    InspectorRow {
                                        key: "value",
                                        field: "Value",
                                        value: Arc::<str>::from(format!("{value:.1}")),
                                        source: "Slider",
                                    },
                                    InspectorRow {
                                        key: "mode",
                                        field: "Mode",
                                        value: mode_label.clone(),
                                        source: "Combo",
                                    },
                                    InspectorRow {
                                        key: "draft",
                                        field: "Draft",
                                        value: Arc::<str>::from(draft.clone()),
                                        source: "Input",
                                    },
                                ];
                                inspector_sort.sort_rows(&mut inspector_rows);
                                let sort_column_id = if compact_surface {
                                    "inspector-signal"
                                } else {
                                    "inspector-field"
                                };
                                let table_columns = if compact_surface {
                                    vec![
                                        kit::TableColumn::px(
                                            "Signal###inspector-signal",
                                            inspector_widths.signal,
                                        )
                                        .sorted(inspector_sort.direction())
                                        .resizable_with_limits(Some(Px(96.0)), Some(Px(180.0))),
                                        kit::TableColumn::px(
                                            "State###inspector-state",
                                            inspector_widths.state,
                                        )
                                        .resizable_with_limits(Some(Px(112.0)), Some(Px(240.0))),
                                    ]
                                } else {
                                    vec![
                                        kit::TableColumn::px(
                                            "Field###inspector-field",
                                            inspector_widths.field,
                                        )
                                        .sorted(inspector_sort.direction())
                                        .resizable_with_limits(Some(Px(88.0)), Some(Px(180.0))),
                                        kit::TableColumn::px(
                                            "Value###inspector-value",
                                            inspector_widths.value,
                                        )
                                        .resizable_with_limits(Some(Px(96.0)), Some(Px(220.0))),
                                        kit::TableColumn::px(
                                            "Source###inspector-source",
                                            inspector_widths.source,
                                        )
                                        .resizable_with_limits(Some(Px(64.0)), Some(Px(140.0))),
                                    ]
                                };

                                if !compact_surface {
                                    ui.separator_text("Inspector snapshot");
                                }
                                let table_response = ui.table_with_options(
                                    "imui-shadcn-demo.inspector.table",
                                    &table_columns,
                                    kit::TableOptions {
                                        striped: true,
                                        test_id: Some(Arc::from(TEST_ID_TABLE)),
                                        ..Default::default()
                                    },
                                    |table| {
                                        if compact_surface {
                                            for row_data in inspector_rows.iter().take(3) {
                                                table.row(row_data.key, |row| {
                                                    row.cell_text(row_data.field);
                                                    row.cell_text(Arc::<str>::from(format!(
                                                        "{} via {}",
                                                        row_data.value, row_data.source
                                                    )));
                                                });
                                            }
                                        } else {
                                            for row_data in &inspector_rows {
                                                table.row(row_data.key, |row| {
                                                    row.cell_text(row_data.field);
                                                    row.cell_text(row_data.value.clone());
                                                    row.cell_text(row_data.source);
                                                });
                                            }
                                        }
                                    },
                                );
                                if table_response
                                    .header(sort_column_id)
                                    .is_some_and(|header| header.clicked())
                                {
                                    let _ = inspector_sort_state
                                        .update_in(ui.cx_mut().app.models_mut(), |sort| {
                                            *sort = sort.toggled();
                                        });
                                }
                                if compact_surface {
                                    apply_inspector_width_delta(
                                        ui,
                                        &table_response,
                                        &inspector_widths_state,
                                        "inspector-signal",
                                        Px(96.0),
                                        Px(180.0),
                                        |widths| &mut widths.signal,
                                    );
                                    apply_inspector_width_delta(
                                        ui,
                                        &table_response,
                                        &inspector_widths_state,
                                        "inspector-state",
                                        Px(112.0),
                                        Px(240.0),
                                        |widths| &mut widths.state,
                                    );
                                } else {
                                    apply_inspector_width_delta(
                                        ui,
                                        &table_response,
                                        &inspector_widths_state,
                                        "inspector-field",
                                        Px(88.0),
                                        Px(180.0),
                                        |widths| &mut widths.field,
                                    );
                                    apply_inspector_width_delta(
                                        ui,
                                        &table_response,
                                        &inspector_widths_state,
                                        "inspector-value",
                                        Px(96.0),
                                        Px(220.0),
                                        |widths| &mut widths.value,
                                    );
                                    apply_inspector_width_delta(
                                        ui,
                                        &table_response,
                                        &inspector_widths_state,
                                        "inspector-source",
                                        Px(64.0),
                                        Px(140.0),
                                        |widths| &mut widths.source,
                                    );
                                }

                                let width_summary = {
                                    let cx = ui.cx_mut();
                                    let text = if compact_surface {
                                        format!(
                                            "Widths: signal {:.0}px, state {:.0}px",
                                            inspector_widths.signal.0, inspector_widths.state.0
                                        )
                                    } else {
                                        format!(
                                            "Widths: field {:.0}px, value {:.0}px, source {:.0}px",
                                            inspector_widths.field.0,
                                            inspector_widths.value.0,
                                            inspector_widths.source.0
                                        )
                                    };
                                    ui::text(text)
                                        .text_xs()
                                        .font_medium()
                                        .into_element(cx)
                                        .test_id(TEST_ID_TABLE_WIDTHS)
                                };
                                ui.add_ui(width_summary);

                                if !compact_surface {
                                    ui.separator_text("Recent entries");
                                }
                                let _ = ui.virtual_list_with_options(
                                    "imui-shadcn-demo.inspector.recent",
                                    if compact_surface { 12 } else { 96 },
                                    kit::VirtualListOptions {
                                        viewport_height: if compact_surface {
                                            RECENT_VIEWPORT_HEIGHT_COMPACT
                                        } else {
                                            RECENT_VIEWPORT_HEIGHT_REGULAR
                                        },
                                        estimate_row_height: Px(28.0),
                                        overscan: 2,
                                        gap: Px(2.0),
                                        measure_mode: kit::VirtualListMeasureMode::Fixed,
                                        test_id: Some(Arc::from(TEST_ID_RECENT_LIST)),
                                        ..Default::default()
                                    },
                                    |index| index as fret_ui::ItemKey,
                                    |ui, index| {
                                        let selected = if compact_surface {
                                            (count as usize % 8) == (index % 8)
                                        } else {
                                            (count as usize % 12) == (index % 12)
                                        };
                                        let _ = ui.selectable(
                                            format!("Recent entry #{index:03}"),
                                            selected,
                                        );
                                    },
                                );
                            })
                        })
                        .into_element(cx);

                    let inspector_header = if compact_surface {
                        shadcn::CardHeader::new([shadcn::CardTitle::new("Inspector")
                            .into_element(cx)])
                        .into_element(cx)
                    } else {
                        shadcn::CardHeader::new([
                            shadcn::CardTitle::new("Inspector").into_element(cx),
                            shadcn::CardDescription::new(
                                "A compact downstream tool panel should stay readable without forking control chrome per demo.",
                            )
                            .into_element(cx),
                        ])
                        .into_element(cx)
                    };

                    shadcn::Card::new([
                        inspector_header,
                        shadcn::CardContent::new([inspector_surface]).into_element(cx),
                    ])
                    .into_element(cx)
                    .test_id(TEST_ID_INSPECTOR_CARD)
                };

                let side_column = ui::v_flex(move |cx| vec![summary_card, inspector_card])
                    .gap(content_gap)
                    .w_full()
                    .min_w_0()
                    .into_element(cx);

                let body = if stack_cards {
                    ui::v_flex(move |cx| vec![controls_card, side_column])
                        .gap(content_gap)
                        .w_full()
                        .min_w_0()
                        .into_element(cx)
                } else {
                    ui::h_flex(move |cx| {
                        vec![
                            ui::container(move |_cx| [controls_card])
                                .flex_1()
                                .min_w_0()
                                .into_element(cx),
                            ui::container(move |_cx| [side_column])
                                .w_px(SIDE_COLUMN_WIDTH)
                                .flex_shrink_0()
                                .min_w_0()
                                .into_element(cx),
                        ]
                    })
                    .gap(content_gap)
                    .items_stretch()
                    .w_full()
                    .min_w_0()
                    .into_element(cx)
                };

                vec![
                    ui::v_flex(move |cx| vec![header, body])
                        .gap(content_gap)
                        .w_full()
                        .min_w_0()
                        .into_element(cx),
                ]
            })
            .p(surface_padding)
            .size_full()
            .bg(muted_bg)
            .into_element(ui.cx_mut())
            .test_id(TEST_ID_ROOT);

            ui.add_ui(root);
        })
    }
}

fn apply_inspector_width_delta(
    ui: &mut ImUi<'_, '_, KernelApp>,
    table_response: &kit::TableResponse,
    widths_state: &LocalState<InspectorColumnWidths>,
    column_id: &str,
    min_width: Px,
    max_width: Px,
    select_width: fn(&mut InspectorColumnWidths) -> &mut Px,
) {
    let Some(header) = table_response.header(column_id) else {
        return;
    };
    let delta_x = header.resize.drag_delta_x();
    if !header.resize.dragging() || !delta_x.is_finite() || delta_x.abs() < f32::EPSILON {
        return;
    }

    let _ = widths_state.update_in(ui.cx_mut().app.models_mut(), |widths| {
        let width = select_width(widths);
        *width = clamped_width_delta(*width, delta_x, min_width, max_width);
    });
}

fn clamped_width_delta(current: Px, delta_x: f32, min_width: Px, max_width: Px) -> Px {
    Px((current.0 + delta_x).clamp(min_width.0, max_width.0.max(min_width.0)))
}

fn summary_badge(
    cx: &mut ElementContext<'_, KernelApp>,
    label: Arc<str>,
    test_id: &'static str,
) -> AnyElement {
    let label = ui::text(label)
        .text_xs()
        .font_medium()
        .nowrap()
        .into_element(cx)
        .test_id(test_id);

    shadcn::Badge::new("")
        .variant(shadcn::BadgeVariant::Secondary)
        .children([label])
        .into_element(cx)
}
